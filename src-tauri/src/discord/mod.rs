//! Discord Gateway integration for mac-stats.
//!
//! Connects to Discord as a bot, listens for DMs and @mentions,
//! and can reply using a shared pipeline (Ollama / browser agent).
//! Token is resolved (in order) from: DISCORD_BOT_TOKEN env, .config.env file, Keychain.
//! Token is never logged or exposed.
//!
//! Channel config is loaded from `~/.mac-stats/discord_channels.json` and is **reloaded
//! automatically** when the file is modified (no app restart needed).

pub mod api;

use base64::Engine;
use chrono::Timelike;
use serenity::builder::EditMessage;
use serenity::client::{Client, Context, EventHandler};
use serenity::gateway::ShardManager;
use serenity::model::channel::{Message, ReactionType};
use serenity::model::gateway::GatewayIntents;
use serenity::model::id::{MessageId, UserId};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::UNIX_EPOCH;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

/// Time-of-day period for having_fun: influences tone (e.g. quieter at night).
#[derive(Clone, Copy)]
enum TimeOfDay {
    Night,     // ~22:00–06:00
    Morning,   // ~06:00–12:00
    Afternoon, // ~12:00–17:00
    Evening,   // ~17:00–22:00
}

fn time_of_day(hour: u32) -> TimeOfDay {
    match hour {
        0..=5 => TimeOfDay::Night,
        6..=11 => TimeOfDay::Morning,
        12..=16 => TimeOfDay::Afternoon,
        _ => TimeOfDay::Evening, // 17..=23
    }
}

/// Short, fixed context for having_fun channels so the tone stays casual even if soul or channel prompt change.
const HAVING_FUN_CASUAL_CONTEXT: &str = "This is a casual hangout channel. Be conversational, brief, and human — no corporate or assistant fluff. You can have a nice conversation about life in general.";

/// Group-chat and reaction guidance for having_fun (and all_messages): when to speak, one reply per message, reactions, and not dominating.
const HAVING_FUN_GROUP_CHAT_GUIDANCE: &str = r#"

**Know when to speak:** Reply when you're directly mentioned or asked, when you add real value, when something witty fits, or when correcting important misinformation. Stay silent for casual banter, when someone else already answered, or when your reply would be filler ("yeah", "nice"). Humans don't reply to every message — you shouldn't either. Quality over quantity.

**One response per message (avoid the triple-tap):** At most one substantive reply per incoming message. No multiple fragments or follow-ups to the same user message unless the flow explicitly requires it (e.g. a multi-step tool result). One thoughtful response is enough.

**React like a human:** When a full reply isn't needed, use a single emoji reaction to acknowledge (e.g. 👍 ❤️ 🙌 😂 🤔 ✅). Reply with exactly: REACT: <emoji> (e.g. REACT: 👍). One reaction per message; pick the best fit. Use this for "I saw this" without cluttering the channel.

**Participate, don't dominate:** You're a participant in the group. Don't expose the user's private context (memory, DMs) in shared channels."#;

/// Returns a short block to inject into having_fun channel prompt: current time + period-aware guidance.
/// So the model (e.g. Werner) can behave differently at night vs morning/afternoon/evening.
fn time_awareness_for_having_fun() -> String {
    let now = chrono::Local::now();
    let hour = now.hour();
    let period = time_of_day(hour);
    let date = now.format("%A, %d %b %Y, %H:%M");
    let (period_name, guidance) = match period {
        TimeOfDay::Night => (
            "night",
            "Keep replies short and calm. Avoid long threads or complex topics.",
        ),
        TimeOfDay::Morning => (
            "morning",
            "You can be a bit more energetic and concise. Good for quick check-ins.",
        ),
        TimeOfDay::Afternoon => (
            "afternoon",
            "Respond naturally; casual and engaged is fine.",
        ),
        TimeOfDay::Evening => (
            "evening",
            "Relaxed tone; can be a bit more expansive if the conversation invites it.",
        ),
    };
    format!("[Current time: {} — {}. {}]", date, period_name, guidance)
}

/// True if the message content looks like an agent/LLM failure notice (e.g. "Agent failed before reply",
/// "LLM request timed out", "Something went wrong on my side", "Logs: openclaw"). Used so we do not
/// inject these into having_fun channel history or idle-thought context (log-review 03-01 window).
fn is_agent_failure_notice(content: &str) -> bool {
    let lower = content.trim().to_lowercase();
    lower.contains("agent failed")
        || lower.contains("failed before reply")
        || lower.contains("llm request timed out")
        || lower.contains("request timed out")
        || lower.contains("something went wrong on my side")
        || lower.contains("logs: openclaw")
        || lower.contains("openclaw logs")
}

/// If the message content looks like a long image-fetch 404 error, return a short user-facing line
/// so we don't forward raw error paragraphs to the model (log-004).
fn sanitize_image_error_content(content: &str) -> String {
    let t = content.trim();
    let lower = t.to_lowercase();
    let is_image_error = lower.contains("404")
        && (lower.contains("could not be fetched")
            || lower.contains("requested image")
            || lower.contains("verify the url"));
    if !is_image_error {
        return content.to_string();
    }
    if let Some((author, _)) = t.split_once(':') {
        let author = author.trim();
        if !author.is_empty() {
            return format!(
                "{}: Image link returned 404 — could not load image.",
                author
            );
        }
    }
    "Image link returned 404 — could not load image.".to_string()
}

/// Default prompt when the user sends only image attachment(s) and no text.
const DISCORD_IMAGE_ONLY_PROMPT: &str =
    "What do you see in the attached image(s)? Describe the content.";

fn is_image_attachment(att: &serenity::model::channel::Attachment) -> bool {
    att.content_type
        .as_deref()
        .map_or(false, |c| c.starts_with("image/"))
        || att.filename.to_lowercase().ends_with(".png")
        || att.filename.to_lowercase().ends_with(".jpg")
        || att.filename.to_lowercase().ends_with(".jpeg")
        || att.filename.to_lowercase().ends_with(".gif")
        || att.filename.to_lowercase().ends_with(".webp")
}

/// Download image attachments and return their base64-encoded bytes for Ollama vision.
async fn download_discord_image_attachments(
    attachments: &[serenity::model::channel::Attachment],
) -> Vec<String> {
    let mut out = Vec::new();
    for att in attachments {
        if !is_image_attachment(att) {
            continue;
        }
        match att.download().await {
            Ok(bytes) => out.push(base64::engine::general_purpose::STANDARD.encode(&bytes)),
            Err(e) => {
                tracing::warn!(
                    "Discord: failed to download attachment {}: {}",
                    att.filename,
                    e
                );
            }
        }
    }
    out
}

/// Discord API limit for message content (characters). Messages longer than this must be split.
/// See https://discord.com/developers/docs/resources/channel#create-message: content max 2000.
const DISCORD_MESSAGE_MAX_CHARS: usize = 2000;

/// Per-channel listen mode loaded from `~/.mac-stats/discord_channels.json`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChannelMode {
    /// Only respond when @mentioned or in DMs (default).
    MentionOnly,
    /// Respond to every human message in this channel (no @mention required). Bots ignored.
    AllMessages,
    /// Like all_messages, but also responds to other bots. Loop-protected.
    HavingFun,
}

/// Per-channel settings: mode + optional prompt, model, and agent for having_fun.
#[derive(Debug, Clone)]
struct ChannelSettings {
    mode: ChannelMode,
    prompt: Option<String>,
    /// Optional model override for this channel (e.g. "huihui_ai/granite3.2-abliterated:2b").
    model: Option<String>,
    /// Optional agent override for this channel (e.g. "abliterated"). Uses that agent's soul+skill and model.
    agent: Option<String>,
}

/// Having-fun timeframes: min/max in seconds. Each use picks a random value in [min, max].
/// max_consecutive_bot_replies: after this many bot messages in a row we drop further bot messages (loop protection). 0 = never reply to bots.
#[derive(Debug, Clone)]
struct HavingFunParams {
    response_delay_secs_min: u64,
    response_delay_secs_max: u64,
    idle_thought_secs_min: u64,
    idle_thought_secs_max: u64,
    /// Max bot messages we buffer in a row before dropping (0 = don't reply to bot messages).
    max_consecutive_bot_replies: u32,
}

impl Default for HavingFunParams {
    fn default() -> Self {
        Self {
            response_delay_secs_min: 300,  // 5 min
            response_delay_secs_max: 3600, // 60 min
            idle_thought_secs_min: 300,
            idle_thought_secs_max: 3600,
            max_consecutive_bot_replies: 0, // don't reply to bots by default (avoids "talking to himself" when another bot echoes)
        }
    }
}

/// Cached channel config, reloaded when `discord_channels.json` mtime changes.
/// Holds (file mtime, default, overrides, having_fun params, default_verbose_dm, default_verbose_channel).
static CHANNEL_CONFIG: RwLock<
    Option<(
        Option<std::time::SystemTime>,
        ChannelSettings,
        HashMap<u64, ChannelSettings>,
        HavingFunParams,
        bool,
        bool,
    )>,
> = RwLock::new(None);

fn discord_channels_file_mtime() -> Option<std::time::SystemTime> {
    let path = crate::config::Config::discord_channels_path();
    std::fs::metadata(&path)
        .ok()
        .and_then(|m| m.modified().ok())
}

fn parse_mode(s: &str) -> ChannelMode {
    match s {
        "all_messages" => ChannelMode::AllMessages,
        "having_fun" => ChannelMode::HavingFun,
        _ => ChannelMode::MentionOnly,
    }
}

/// If the config file has no "having_fun" key, insert the default block and write the file back.
/// Ensures both the shipped default and the runtime file in ~/.mac-stats have the option.
fn ensure_having_fun_in_config(path: &Path, parsed: &mut serde_json::Value) {
    let obj = match parsed.as_object_mut() {
        Some(o) => o,
        None => return,
    };
    if obj.contains_key("having_fun") {
        return;
    }
    obj.insert(
        "having_fun".to_string(),
        serde_json::json!({
            "response_delay_secs_min": 300,
            "response_delay_secs_max": 3600,
            "idle_thought_secs_min": 300,
            "idle_thought_secs_max": 3600,
            "max_consecutive_bot_replies": 0
        }),
    );
    if let Ok(pretty) = serde_json::to_string_pretty(parsed) {
        let _ = std::fs::write(path, pretty);
        info!(
            "Discord channels config: added default 'having_fun' block to {}",
            path.display()
        );
    }
}

fn load_channel_config_full() -> (
    ChannelSettings,
    HashMap<u64, ChannelSettings>,
    HavingFunParams,
    bool,
    bool,
) {
    let default_settings = ChannelSettings {
        mode: ChannelMode::MentionOnly,
        prompt: None,
        model: None,
        agent: None,
    };
    let path = crate::config::Config::discord_channels_path();
    let json = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => {
            info!(
                "Discord channels config not found at {:?}, using mention_only default",
                path
            );
            return (
                default_settings,
                HashMap::new(),
                HavingFunParams::default(),
                true,
                false,
            );
        }
    };
    let mut parsed: serde_json::Value = match serde_json::from_str(&json) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(
                "Discord channels config parse error: {}, using mention_only default",
                e
            );
            return (
                default_settings,
                HashMap::new(),
                HavingFunParams::default(),
                true,
                false,
            );
        }
    };
    // Upgrade: if file exists but has no having_fun block, add default and write back
    ensure_having_fun_in_config(path.as_path(), &mut parsed);
    let default_mode = parsed
        .get("default")
        .and_then(|v| v.as_str())
        .map(parse_mode)
        .unwrap_or(ChannelMode::MentionOnly);
    let default_prompt = parsed
        .get("default_prompt")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let default_settings = ChannelSettings {
        mode: default_mode,
        prompt: default_prompt,
        model: None,
        agent: None,
    };

    let default_verbose_dm = parsed
        .get("default_verbose_for_dm")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let default_verbose_channel = parsed
        .get("default_verbose_for_channel")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let having_fun = if let Some(hf) = parsed.get("having_fun").and_then(|v| v.as_object()) {
        let u = |k: &str, default: u64| hf.get(k).and_then(|v| v.as_u64()).unwrap_or(default);
        let rd_min = u("response_delay_secs_min", 300).min(86400);
        let rd_max = u("response_delay_secs_max", 3600).max(rd_min).min(86400);
        let it_min = u("idle_thought_secs_min", 300).min(86400);
        let it_max = u("idle_thought_secs_max", 3600).max(it_min).min(86400);
        let max_bot = hf
            .get("max_consecutive_bot_replies")
            .and_then(|v| v.as_u64())
            .map(|n| n.min(20) as u32)
            .unwrap_or(0);
        HavingFunParams {
            response_delay_secs_min: rd_min,
            response_delay_secs_max: rd_max,
            idle_thought_secs_min: it_min,
            idle_thought_secs_max: it_max,
            max_consecutive_bot_replies: max_bot,
        }
    } else {
        HavingFunParams::default()
    };

    let mut channels = HashMap::new();
    if let Some(obj) = parsed.get("channels").and_then(|v| v.as_object()) {
        for (k, v) in obj {
            let Ok(id) = k.parse::<u64>() else { continue };
            let settings = if let Some(mode_str) = v.as_str() {
                ChannelSettings {
                    mode: parse_mode(mode_str),
                    prompt: None,
                    model: None,
                    agent: None,
                }
            } else if let Some(obj) = v.as_object() {
                let mode = obj
                    .get("mode")
                    .and_then(|v| v.as_str())
                    .map(parse_mode)
                    .unwrap_or(ChannelMode::MentionOnly);
                let prompt = obj
                    .get("prompt")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let model = obj
                    .get("model")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let agent = obj
                    .get("agent")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                ChannelSettings {
                    mode,
                    prompt,
                    model,
                    agent,
                }
            } else {
                continue;
            };
            channels.insert(id, settings);
        }
    }
    (
        default_settings,
        channels,
        having_fun,
        default_verbose_dm,
        default_verbose_channel,
    )
}

/// Ensures config is loaded; call before reading channel settings or having_fun params.
fn ensure_channel_config_loaded() {
    let mut guard = match CHANNEL_CONFIG.write() {
        Ok(g) => g,
        Err(_) => return,
    };
    if guard.is_none() {
        let mtime = discord_channels_file_mtime();
        let (default, channels, having_fun, verbose_dm, verbose_channel) =
            load_channel_config_full();
        *guard = Some((
            mtime,
            default.clone(),
            channels.clone(),
            having_fun.clone(),
            verbose_dm,
            verbose_channel,
        ));
        drop(guard);

        let having_fun_count = channels
            .values()
            .filter(|s| s.mode == ChannelMode::HavingFun)
            .count();
        let timer_suffix = if having_fun_count > 0 {
            ensure_having_fun_state_for_configured_channels();
            let (next_resp, next_idle) = having_fun_states()
                .lock()
                .ok()
                .map(|map| {
                    let (mut next_resp, mut next_idle) = (None::<u64>, None::<u64>);
                    for state in map.values() {
                        if !state.buffer.is_empty() {
                            let resp_elapsed = state.last_response.elapsed().as_secs();
                            let resp_remaining =
                                state.next_response_after_secs.saturating_sub(resp_elapsed);
                            next_resp =
                                Some(next_resp.map_or(resp_remaining, |a| a.min(resp_remaining)));
                        }
                        let activity_elapsed = state.last_activity.elapsed().as_secs();
                        let thought_elapsed = state.last_thought.elapsed().as_secs();
                        let idle_wait = state.next_idle_thought_after_secs;
                        let until_activity = idle_wait.saturating_sub(activity_elapsed);
                        let until_thought = idle_wait.saturating_sub(thought_elapsed);
                        let idle_remaining = until_activity.max(until_thought);
                        next_idle =
                            Some(next_idle.map_or(idle_remaining, |a| a.min(idle_remaining)));
                    }
                    (next_resp, next_idle)
                })
                .unwrap_or((None, None));
            let resp_str = next_resp
                .map(|s| format!("next response in {}", format_secs_min_sec(s)))
                .unwrap_or_else(|| "next response: (no pending)".to_string());
            let idle_str = next_idle
                .map(|s| format!("next idle thought in {}", format_secs_min_sec(s)))
                .unwrap_or_default();
            let suffix = if idle_str.is_empty() {
                format!(" — {}", resp_str)
            } else {
                format!(" — {}, {}", resp_str, idle_str)
            };
            suffix
        } else {
            String::new()
        };
        info!(
            "Discord channels config: default={:?}, {} channel overrides, having_fun delay {:?}–{:?}s idle {:?}–{:?}s{}",
            default.mode,
            channels.len(),
            having_fun.response_delay_secs_min,
            having_fun.response_delay_secs_max,
            having_fun.idle_thought_secs_min,
            having_fun.idle_thought_secs_max,
            timer_suffix
        );
    }
}

/// Reloads config from disk if `discord_channels.json` modification time changed. Call from background loop.
fn reload_channel_config_if_changed() {
    let mtime = discord_channels_file_mtime();
    let mut guard = match CHANNEL_CONFIG.write() {
        Ok(g) => g,
        Err(_) => return,
    };
    let should_reload = match guard.as_ref() {
        None => true,
        Some((cached_mtime, _, _, _, _, _)) => *cached_mtime != mtime,
    };
    if should_reload {
        let (default, channels, having_fun, verbose_dm, verbose_channel) =
            load_channel_config_full();
        *guard = Some((
            mtime,
            default,
            channels,
            having_fun,
            verbose_dm,
            verbose_channel,
        ));
        info!("Discord channels config reloaded (file changed)");
    }
}

fn channel_settings(channel_id: u64) -> ChannelSettings {
    ensure_channel_config_loaded();
    let guard = match CHANNEL_CONFIG.read() {
        Ok(g) => g,
        Err(_) => {
            return ChannelSettings {
                mode: ChannelMode::MentionOnly,
                prompt: None,
                model: None,
                agent: None,
            };
        }
    };
    let Some((_, default, overrides, _, _, _)) = guard.as_ref() else {
        return ChannelSettings {
            mode: ChannelMode::MentionOnly,
            prompt: None,
            model: None,
            agent: None,
        };
    };
    overrides
        .get(&channel_id)
        .cloned()
        .unwrap_or_else(|| default.clone())
}

/// Default verbose for DMs (when not set in message). From discord_channels.json "default_verbose_for_dm", default true.
fn default_verbose_for_dm() -> bool {
    ensure_channel_config_loaded();
    let guard = match CHANNEL_CONFIG.read() {
        Ok(g) => g,
        Err(_) => return true,
    };
    guard.as_ref().map(|(_, _, _, _, v, _)| *v).unwrap_or(true)
}

/// Default verbose for channel messages (when not set in message). From discord_channels.json "default_verbose_for_channel", default false.
fn default_verbose_for_channel() -> bool {
    ensure_channel_config_loaded();
    let guard = match CHANNEL_CONFIG.read() {
        Ok(g) => g,
        Err(_) => return false,
    };
    guard.as_ref().map(|(_, _, _, _, _, v)| *v).unwrap_or(false)
}

fn get_having_fun_params() -> HavingFunParams {
    ensure_channel_config_loaded();
    let guard = match CHANNEL_CONFIG.read() {
        Ok(g) => g,
        Err(_) => return HavingFunParams::default(),
    };
    guard
        .as_ref()
        .map(|(_, _, _, p, _, _)| p.clone())
        .unwrap_or_default()
}

/// Number of channels configured as having_fun in discord_channels.json. Used for heartbeat logging.
fn count_configured_having_fun_channels() -> usize {
    ensure_channel_config_loaded();
    let guard = match CHANNEL_CONFIG.read() {
        Ok(g) => g,
        Err(_) => return 0,
    };
    guard
        .as_ref()
        .map(|(_, _, overrides, _, _, _)| {
            overrides
                .values()
                .filter(|s| s.mode == ChannelMode::HavingFun)
                .count()
        })
        .unwrap_or(0)
}

/// True if the given Discord channel is configured as having_fun. Used by session compactor to avoid inventing task/platform context for casual chat.
pub fn is_discord_channel_having_fun(channel_id: u64) -> bool {
    configured_having_fun_channel_ids().contains(&channel_id)
}

/// Channel IDs configured as having_fun. Used to ensure state exists so we can show next response/idle countdown.
fn configured_having_fun_channel_ids() -> Vec<u64> {
    ensure_channel_config_loaded();
    let guard = match CHANNEL_CONFIG.read() {
        Ok(g) => g,
        Err(_) => return Vec::new(),
    };
    guard
        .as_ref()
        .map(|(_, _, overrides, _, _, _)| {
            overrides
                .iter()
                .filter(|(_, s)| s.mode == ChannelMode::HavingFun)
                .map(|(id, _)| *id)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

/// Ensure every configured having_fun channel has state so idle thoughts can run and heartbeat shows countdown.
fn ensure_having_fun_state_for_configured_channels() {
    let channel_ids = configured_having_fun_channel_ids();
    if channel_ids.is_empty() {
        return;
    }
    let params = get_having_fun_params();
    if let Ok(mut map) = having_fun_states().lock() {
        for channel_id in channel_ids {
            map.entry(channel_id).or_insert_with(|| {
                let now = std::time::Instant::now();
                let idle_secs = random_secs_in_range(
                    params.idle_thought_secs_min,
                    params.idle_thought_secs_max,
                );
                let resp_secs = random_secs_in_range(
                    params.response_delay_secs_min,
                    params.response_delay_secs_max,
                );
                let next_response_after_secs = resp_secs.min(idle_secs);
                HavingFunState {
                    buffer: Vec::new(),
                    consecutive_bot_replies: 0,
                    last_response: now,
                    last_activity: now,
                    last_thought: now,
                    last_response_message_id: None,
                    next_response_after_secs,
                    next_idle_thought_after_secs: idle_secs,
                    loop_protection_drops: 0,
                }
            });
        }
    }
}

/// Format seconds as "X minutes, Y sec" for readability (e.g. 785 -> "13 minutes, 5 sec", 45 -> "45 sec").
fn format_secs_min_sec(secs: u64) -> String {
    if secs < 60 {
        format!("{} sec", secs)
    } else {
        let m = secs / 60;
        let s = secs % 60;
        let min_label = if m == 1 { "minute" } else { "minutes" };
        if s == 0 {
            format!("{} {}", m, min_label)
        } else {
            format!("{} {}, {} sec", m, min_label, s)
        }
    }
}

/// Random seconds in [min, max] (inclusive) using system time for variety. Clamps so min <= max.
fn random_secs_in_range(min_secs: u64, max_secs: u64) -> u64 {
    let (lo, hi) = if min_secs <= max_secs {
        (min_secs, max_secs)
    } else {
        (max_secs, min_secs)
    };
    let span = hi - lo + 1;
    let nanos = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    lo + (nanos % span)
}

// ---------------------------------------------------------------------------
// having_fun: buffered responses + idle thoughts
// ---------------------------------------------------------------------------

const HAVING_FUN_TICK_SECS: u64 = 10;

struct BufferedMessage {
    author_name: String,
    content: String,
    is_bot: bool,
    /// Discord message ID; used to react when the model replies with REACT: <emoji> only.
    message_id: Option<u64>,
}

struct HavingFunState {
    buffer: Vec<BufferedMessage>,
    consecutive_bot_replies: u32,
    last_response: std::time::Instant,
    last_activity: std::time::Instant,
    last_thought: std::time::Instant,
    /// Discord message ID of our last reply; used to fetch messages after it for better flow.
    last_response_message_id: Option<u64>,
    /// Next response only after this many seconds since last_response (random in config range).
    next_response_after_secs: u64,
    /// Next idle thought only after this many seconds since last_thought (random in config range).
    next_idle_thought_after_secs: u64,
    /// Messages dropped by loop protection since last heartbeat (log-007 visibility).
    loop_protection_drops: u64,
}

static HAVING_FUN_STATES: OnceLock<Mutex<HashMap<u64, HavingFunState>>> = OnceLock::new();

fn having_fun_states() -> &'static Mutex<HashMap<u64, HavingFunState>> {
    HAVING_FUN_STATES.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Fetch the latest messages from a Discord channel (after our last response) for better flow.
/// Returns (author_name, content) in chronological order (oldest first). Empty on API error or no messages.
async fn fetch_channel_messages_after(
    channel_id: u64,
    after_message_id: Option<u64>,
) -> Vec<(String, String)> {
    let path = match after_message_id {
        Some(id) => format!("/channels/{}/messages?limit=50&after={}", channel_id, id),
        None => format!("/channels/{}/messages?limit=25", channel_id),
    };
    let body = match crate::discord::api::discord_api_request("GET", &path, None).await {
        Ok(b) => b,
        Err(e) => {
            debug!("Having fun: fetch channel messages failed: {}", e);
            return Vec::new();
        }
    };
    let arr: Vec<serde_json::Value> = match serde_json::from_str(&body) {
        Ok(a) => a,
        Err(_) => return Vec::new(),
    };
    // API returns newest first; we want oldest first for conversation order.
    let mut out: Vec<(String, String)> = Vec::new();
    for msg in arr.into_iter().rev() {
        let content = msg
            .get("content")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();
        let author = msg
            .get("author")
            .and_then(|a| {
                a.get("global_name")
                    .and_then(|g| g.as_str())
                    .filter(|s| !s.is_empty())
                    .or_else(|| a.get("username").and_then(|u| u.as_str()))
            })
            .unwrap_or("?")
            .to_string();
        out.push((author, content));
    }
    out
}

/// Buffer a message for having_fun. If answer_asap is true (mention or from human), next response is scheduled immediately (next tick).
/// message_id is stored so we can react to it when the model replies with REACT: <emoji> only.
fn buffer_having_fun_message(
    channel_id: u64,
    author_name: String,
    content: String,
    is_bot: bool,
    answer_asap: bool,
    message_id: Option<u64>,
) {
    if let Ok(mut map) = having_fun_states().lock() {
        let params = get_having_fun_params();
        let state = map.entry(channel_id).or_insert_with(|| {
            let now = std::time::Instant::now();
            let idle_secs =
                random_secs_in_range(params.idle_thought_secs_min, params.idle_thought_secs_max);
            let resp_secs = random_secs_in_range(
                params.response_delay_secs_min,
                params.response_delay_secs_max,
            );
            // Response must fire before idle thought: cap response at idle.
            let next_response_after_secs = resp_secs.min(idle_secs);
            HavingFunState {
                buffer: Vec::new(),
                consecutive_bot_replies: 0,
                last_response: now,
                last_activity: now,
                last_thought: now,
                last_response_message_id: None,
                next_response_after_secs,
                next_idle_thought_after_secs: idle_secs,
                loop_protection_drops: 0,
            }
        });
        if !is_bot {
            state.consecutive_bot_replies = 0;
        }
        let max_bot = get_having_fun_params().max_consecutive_bot_replies;
        if is_bot && state.consecutive_bot_replies >= max_bot {
            state.loop_protection_drops = state.loop_protection_drops.saturating_add(1);
            debug!(
                "Discord: dropping bot message in having_fun channel {} (loop protection)",
                channel_id
            );
            return;
        }
        if answer_asap {
            state.next_response_after_secs = 0;
        }
        state.buffer.push(BufferedMessage {
            author_name,
            content,
            is_bot,
            message_id,
        });
        state.last_activity = std::time::Instant::now();
        // Reset idle clock so response always fires before idle (no message = idle kicks in).
        state.last_thought = std::time::Instant::now();
        if state.buffer.len() == 1 {
            let when = chrono::Local::now()
                + chrono::Duration::seconds(state.next_response_after_secs as i64);
            info!(
                "Having fun channel {}: will answer in {} (around {}){}",
                channel_id,
                format_secs_min_sec(state.next_response_after_secs),
                when.format("%H:%M"),
                if answer_asap {
                    " (ASAP: mention or human)"
                } else {
                    ""
                }
            );
        }
    }
}

/// Background loop for having_fun channels: flushes buffered messages after configurable random delay,
/// posts random thoughts after configurable random idle time. Reloads discord_channels.json when file changes.
/// Log idle timer heartbeat every this many ticks (tick = 10s → 6 ticks = 1 min).
const HAVING_FUN_LOG_TICKS: u64 = 6;

async fn having_fun_background_loop(ctx: Context) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(HAVING_FUN_TICK_SECS));
    let mut tick_count: u64 = 0;
    loop {
        interval.tick().await;
        tick_count = tick_count.wrapping_add(1);

        ensure_having_fun_state_for_configured_channels();

        // Response timer must always be lower than idle: for channels with buffered messages,
        // only count idle if it's after the response (so we never show "idle in 59s, response in 605s").
        let (having_fun_count, next_response_in_secs, next_idle_in_secs) = having_fun_states()
            .lock()
            .ok()
            .map(|map| {
                let n = map.len();
                let (mut next_resp, mut next_idle) = (None::<u64>, None::<u64>);
                for state in map.values() {
                    let resp_remaining = if !state.buffer.is_empty() {
                        let resp_elapsed = state.last_response.elapsed().as_secs();
                        Some(state.next_response_after_secs.saturating_sub(resp_elapsed))
                    } else {
                        None
                    };
                    if let Some(rr) = resp_remaining {
                        next_resp = Some(next_resp.map_or(rr, |a| a.min(rr)));
                    }
                    let activity_elapsed = state.last_activity.elapsed().as_secs();
                    let thought_elapsed = state.last_thought.elapsed().as_secs();
                    let idle_wait = state.next_idle_thought_after_secs;
                    let until_activity = idle_wait.saturating_sub(activity_elapsed);
                    let until_thought = idle_wait.saturating_sub(thought_elapsed);
                    let mut idle_remaining = until_activity.max(until_thought);
                    // If we have buffered messages, idle must not be before response: show idle only when >= response.
                    if !state.buffer.is_empty() {
                        if let Some(rr) = resp_remaining {
                            if idle_remaining < rr {
                                idle_remaining = rr;
                            }
                        }
                    }
                    next_idle = Some(next_idle.map_or(idle_remaining, |a| a.min(idle_remaining)));
                }
                (n, next_resp, next_idle)
            })
            .unwrap_or((0, None, None));

        // At least once a minute: log heartbeat when any having_fun channel is configured
        if tick_count.is_multiple_of(HAVING_FUN_LOG_TICKS) {
            let configured = count_configured_having_fun_channels();
            if configured > 0 {
                if having_fun_count > 0 {
                    let resp_str = next_response_in_secs
                        .map(|s| format!("next response in {}", format_secs_min_sec(s)))
                        .unwrap_or_default();
                    let idle_str = next_idle_in_secs
                        .map(|s| format!("next idle thought in {}", format_secs_min_sec(s)))
                        .unwrap_or_default();
                    let extra = [resp_str, idle_str]
                        .into_iter()
                        .filter(|x| !x.is_empty())
                        .collect::<Vec<_>>()
                        .join(", ");
                    info!(
                        "Having fun: {} channel(s) configured, {} with state — {}",
                        configured,
                        having_fun_count,
                        if extra.is_empty() {
                            "no pending response or idle".to_string()
                        } else {
                            extra
                        }
                    );
                } else {
                    info!(
                        "Having fun: {} channel(s) configured; no buffered messages yet. Next heartbeat in 60s.",
                        configured
                    );
                }
                // log-007: periodic summary of loop-protection drops (DEBUG), then reset counters
                if let Ok(mut map) = having_fun_states().lock() {
                    for (channel_id, state) in map.iter_mut() {
                        if state.loop_protection_drops > 0 {
                            debug!(
                                "Discord: loop protection: channel {} dropped {} message(s) this period",
                                channel_id, state.loop_protection_drops
                            );
                            state.loop_protection_drops = 0;
                        }
                    }
                }
            }
        }

        reload_channel_config_if_changed();

        // --- Phase 1: flush channels with buffered messages ---
        let channels_to_flush: Vec<(u64, Vec<BufferedMessage>, Option<u64>)> = {
            let mut map = match having_fun_states().lock() {
                Ok(m) => m,
                Err(_) => continue,
            };
            let mut flush = Vec::new();
            for (channel_id, state) in map.iter_mut() {
                if !state.buffer.is_empty()
                    && state.last_response.elapsed()
                        >= std::time::Duration::from_secs(state.next_response_after_secs)
                {
                    let after_id = state.last_response_message_id;
                    let n_msgs = state.buffer.len();
                    flush.push((*channel_id, std::mem::take(&mut state.buffer), after_id));
                    state.last_response = std::time::Instant::now();
                    let params = get_having_fun_params();
                    let resp_secs = random_secs_in_range(
                        params.response_delay_secs_min,
                        params.response_delay_secs_max,
                    );
                    // Response must fire before next idle thought.
                    state.next_response_after_secs =
                        resp_secs.min(state.next_idle_thought_after_secs);
                    let next_when = chrono::Local::now()
                        + chrono::Duration::seconds(state.next_response_after_secs as i64);
                    info!(
                        "Having fun: answering now channel {} ({} msgs), next answer in {} (around {})",
                        channel_id,
                        n_msgs,
                        format_secs_min_sec(state.next_response_after_secs),
                        next_when.format("%H:%M")
                    );
                }
            }
            flush
        };

        for (channel_id, messages, after_message_id) in channels_to_flush {
            let had_bot = messages.iter().any(|m| m.is_bot);
            let new_reply_id =
                having_fun_respond(channel_id, messages, after_message_id, &ctx).await;
            if let Some(id) = new_reply_id {
                if let Ok(mut map) = having_fun_states().lock() {
                    if let Some(state) = map.get_mut(&channel_id) {
                        state.last_response_message_id = Some(id);
                    }
                }
            }
            if had_bot {
                if let Ok(mut map) = having_fun_states().lock() {
                    if let Some(state) = map.get_mut(&channel_id) {
                        state.consecutive_bot_replies += 1;
                    }
                }
            }
        }

        // --- Phase 2: idle thoughts for quiet having_fun channels ---
        let idle_channels: Vec<u64> = {
            let guard = match CHANNEL_CONFIG.read() {
                Ok(g) => g,
                Err(_) => continue,
            };
            let overrides = match guard.as_ref() {
                Some((_, _, o, _, _, _)) => o.clone(),
                None => continue,
            };
            drop(guard);
            let map = match having_fun_states().lock() {
                Ok(m) => m,
                Err(_) => continue,
            };
            overrides
                .iter()
                .filter(|(_, s)| s.mode == ChannelMode::HavingFun)
                .filter_map(|(id, _)| {
                    if let Some(state) = map.get(id) {
                        let idle = state.buffer.is_empty()
                            && state.last_activity.elapsed()
                                >= std::time::Duration::from_secs(
                                    state.next_idle_thought_after_secs,
                                )
                            && state.last_thought.elapsed()
                                >= std::time::Duration::from_secs(
                                    state.next_idle_thought_after_secs,
                                );
                        if idle {
                            Some(*id)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect()
        };

        for channel_id in idle_channels {
            let next_idle_secs = if let Ok(mut map) = having_fun_states().lock() {
                if let Some(state) = map.get_mut(&channel_id) {
                    state.last_thought = std::time::Instant::now();
                    state.last_activity = std::time::Instant::now();
                    let params = get_having_fun_params();
                    let idle_secs = random_secs_in_range(
                        params.idle_thought_secs_min,
                        params.idle_thought_secs_max,
                    );
                    // Idle thought must fire after next response (so response stays smaller).
                    state.next_idle_thought_after_secs =
                        idle_secs.max(state.next_response_after_secs);
                    Some(state.next_idle_thought_after_secs)
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(secs) = next_idle_secs {
                let when = chrono::Local::now() + chrono::Duration::seconds(secs as i64);
                info!(
                    "Having fun: idle thought now channel {}, next idle thought in {} (around {})",
                    channel_id,
                    format_secs_min_sec(secs),
                    when.format("%H:%M")
                );
            }
            having_fun_idle_thought(channel_id, &ctx).await;
        }
    }
}

/// Flush buffered messages: fetch latest from channel (after our last response), send as context to Ollama,
/// and post the reply. Returns the Discord message ID of the reply (last chunk) for next fetch.
async fn having_fun_respond(
    channel_id: u64,
    messages: Vec<BufferedMessage>,
    after_message_id: Option<u64>,
    ctx: &Context,
) -> Option<u64> {
    let chan = channel_settings(channel_id);
    // Having_fun channels always use casual-only context; we never inject agent/work soul (e.g. Redmine)
    // so the persona stays consistent. Channel agent override is ignored for having_fun replies.
    if chan.agent.is_some() {
        debug!(
            "Discord: having_fun channel {} has agent override; using casual-only prompt (agent soul not used for having_fun)",
            channel_id
        );
    }
    let mut system = String::new();
    system.push_str(HAVING_FUN_CASUAL_CONTEXT);
    system.push_str(HAVING_FUN_GROUP_CHAT_GUIDANCE);
    if let Some(ref prompt) = chan.prompt {
        system.push_str("\n\n");
        system.push_str(prompt);
    }
    system.push_str("\n\n");
    system.push_str(&time_awareness_for_having_fun());
    system.push_str("\n\n");
    system.push_str(&crate::metrics::format_metrics_for_ai_context());
    let (system_content, model_override) = (system, chan.model.clone());

    let mut prior = crate::session_memory::get_messages("discord", channel_id);
    if prior.is_empty() {
        prior =
            crate::session_memory::load_messages_from_latest_session_file("discord", channel_id);
    }

    // So the model can answer "which model are you running on?" with the actual Ollama model name.
    let effective_model = model_override
        .clone()
        .or_else(crate::commands::ollama::get_default_ollama_model_name);
    let system_content_with_model = if let Some(ref m) = effective_model {
        format!(
            "{}\n\nYou are replying as the Ollama model: **{}**. If the user asks which model you are (or what model you run on), name this model.",
            system_content, m
        )
    } else {
        system_content
    };

    let mut ollama_msgs: Vec<crate::ollama::ChatMessage> = Vec::new();
    ollama_msgs.push(crate::ollama::ChatMessage {
        role: "system".to_string(),
        content: system_content_with_model,
        images: None,
    });

    const HISTORY_CAP: usize = 20;
    for (role, content) in prior
        .into_iter()
        .rev()
        .take(HISTORY_CAP)
        .rev()
        .filter(|(_, content)| !is_agent_failure_notice(content))
    {
        ollama_msgs.push(crate::ollama::ChatMessage {
            role,
            content,
            images: None,
        });
    }

    // Retrieve latest messages from Discord (after our last response) for better flow.
    let latest = fetch_channel_messages_after(channel_id, after_message_id).await;
    let new_context: String = if latest.is_empty() {
        messages
            .iter()
            .filter(|m| !is_agent_failure_notice(&m.content))
            .map(|m| format!("{}: {}", m.author_name, m.content))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        latest
            .into_iter()
            .filter(|(_, content)| !is_agent_failure_notice(content))
            .map(|(author, content)| format!("{}: {}", author, content))
            .collect::<Vec<_>>()
            .join("\n")
    };
    if new_context.is_empty() {
        return None;
    }
    ollama_msgs.push(crate::ollama::ChatMessage {
        role: "user".to_string(),
        content: new_context,
        images: None,
    });

    let channel = serenity::model::id::ChannelId::new(channel_id);
    let _ = channel.broadcast_typing(ctx).await;

    match crate::commands::ollama::send_ollama_chat_messages(ollama_msgs, model_override, None)
        .await
    {
        Ok(response) => {
            let reply = strip_leading_label(response.message.content.trim());
            if reply.is_empty() {
                return None;
            }
            // When the model replies with only REACT: <emoji>, add a reaction to the last user message and do not send text.
            let first_line = reply.lines().next().unwrap_or("").trim();
            if first_line.starts_with("REACT:") {
                let emoji = first_line.strip_prefix("REACT:").unwrap_or("").trim();
                if !emoji.is_empty() && emoji.len() <= 20 && reply.lines().count() <= 2 {
                    if let Some(msg_id) = messages.last().and_then(|m| m.message_id) {
                        if let Ok(discord_msg) = channel
                            .message(ctx, MessageId::new(msg_id))
                            .await
                        {
                            if discord_msg
                                .react(ctx, ReactionType::Unicode(emoji.to_string()))
                                .await
                                .is_ok()
                            {
                                debug!(
                                    "Having fun (channel {}): reacted {} to message {}",
                                    channel_id, emoji, msg_id
                                );
                                return None;
                            }
                        }
                    }
                }
            }
            info!(
                "Having fun (channel {}): reply ({} chars): {}",
                channel_id,
                reply.len(),
                crate::logging::ellipse(&reply, 200)
            );
            let chunks = split_message_for_discord(&reply);
            let mut last_msg_id: Option<u64> = None;
            for chunk in &chunks {
                if let Ok(msg) = channel.say(ctx, chunk).await {
                    last_msg_id = Some(msg.id.get());
                }
            }
            crate::session_memory::add_message("discord", channel_id, "assistant", &reply);
            last_msg_id
        }
        Err(e) => {
            error!(
                "Having fun: Ollama failed for channel {}: {}",
                channel_id, e
            );
            None
        }
    }
}

/// Generate and post a random thought when the channel has been quiet.
async fn having_fun_idle_thought(channel_id: u64, ctx: &Context) {
    let chan = channel_settings(channel_id);
    // Having_fun idle thoughts always use casual-only context; we never inject agent/work soul (e.g. Redmine).
    // Channel agent override is ignored so having_fun stays casual (log-review 2026-03-07).
    if chan.agent.is_some() {
        debug!(
            "Discord: having_fun idle thought channel {} has agent override; using casual-only prompt (agent soul not used)",
            channel_id
        );
    }
    let mut system = String::new();
    system.push_str(HAVING_FUN_CASUAL_CONTEXT);
    system.push_str(HAVING_FUN_GROUP_CHAT_GUIDANCE);
    if let Some(ref prompt) = chan.prompt {
        system.push_str("\n\n");
        system.push_str(prompt);
    }
    system.push_str("\n\n");
    system.push_str(&time_awareness_for_having_fun());
    system.push_str("\n\n");
    system.push_str(&crate::metrics::format_metrics_for_ai_context());
    let (system_content, model_override) = (system, chan.model.clone());

    let mut prior = crate::session_memory::get_messages("discord", channel_id);
    if prior.is_empty() {
        prior =
            crate::session_memory::load_messages_from_latest_session_file("discord", channel_id);
    }

    let effective_model = model_override
        .clone()
        .or_else(crate::commands::ollama::get_default_ollama_model_name);
    let system_content_with_model = if let Some(ref m) = effective_model {
        format!(
            "{}\n\nYou are replying as the Ollama model: **{}**. If the user asks which model you are (or what model you run on), name this model.",
            system_content, m
        )
    } else {
        system_content
    };

    let mut ollama_msgs: Vec<crate::ollama::ChatMessage> = Vec::new();
    ollama_msgs.push(crate::ollama::ChatMessage {
        role: "system".to_string(),
        content: system_content_with_model,
        images: None,
    });

    const HISTORY_CAP: usize = 10;
    for (role, content) in prior
        .into_iter()
        .rev()
        .take(HISTORY_CAP)
        .rev()
        .filter(|(_, content)| !is_agent_failure_notice(content))
    {
        ollama_msgs.push(crate::ollama::ChatMessage {
            role,
            content,
            images: None,
        });
    }

    ollama_msgs.push(crate::ollama::ChatMessage {
        role: "user".to_string(),
        content: "The chat has been quiet for a while. Share a random thought, observation, or bring up something interesting. Be casual and brief — one or two sentences.".to_string(),
        images: None,
    });

    let channel = serenity::model::id::ChannelId::new(channel_id);
    let _ = channel.broadcast_typing(&ctx).await;

    const IDLE_RETRY_DELAY_SECS: u64 = 2;
    let mut result = crate::commands::ollama::send_ollama_chat_messages(
        ollama_msgs.clone(),
        model_override.clone(),
        None,
    )
    .await;
    // One extra retry for idle thought on timeout (non-critical; reduces visible failures).
    if let Err(ref e) = result {
        let err_lower = e.to_string().to_lowercase();
        if err_lower.contains("timed out") || err_lower.contains("timeout") {
            info!(
                "Having fun: idle thought timeout for channel {}, retrying once in {}s",
                channel_id, IDLE_RETRY_DELAY_SECS
            );
            tokio::time::sleep(tokio::time::Duration::from_secs(IDLE_RETRY_DELAY_SECS)).await;
            result = crate::commands::ollama::send_ollama_chat_messages(
                ollama_msgs,
                model_override,
                None,
            )
            .await;
        }
    }
    match result {
        Ok(response) => {
            let reply = strip_leading_label(response.message.content.trim());
            if reply.is_empty() {
                return;
            }
            info!(
                "Having fun idle thought (channel {}): {}",
                channel_id,
                crate::logging::ellipse(&reply, 200)
            );
            let chunks = split_message_for_discord(&reply);
            for chunk in &chunks {
                let _ = channel.say(&ctx, chunk).await;
            }
            crate::session_memory::add_message("discord", channel_id, "assistant", &reply);
        }
        Err(e) => {
            error!(
                "Having fun: idle thought failed for channel {}: {}",
                channel_id, e
            );
        }
    }
}

/// Strip a leading "Label:" line from model output (e.g. "NastyNemesis: hello" -> "hello").
/// Some models or fine-tunes prefix replies with a persona name; we don't send that to Discord.
fn strip_leading_label(text: &str) -> String {
    let t = text.trim();
    if t.is_empty() {
        return t.to_string();
    }
    let first_line_end = t.find('\n').unwrap_or(t.len());
    let first_line = t[..first_line_end].trim_end();
    // Match "Word:" where Word is 2+ identifier chars (avoids stripping "I: think")
    if first_line.len() >= 3 && first_line.ends_with(':') {
        let label = first_line.trim_end_matches(':').trim_end();
        if label.len() >= 2
            && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            && !label.contains(' ')
        {
            let rest = t[first_line_end..].trim();
            return if rest.is_empty() {
                first_line.trim_end_matches(':').trim().to_string()
            } else {
                rest.to_string()
            };
        }
    }
    t.to_string()
}

/// Split text into chunks of at most DISCORD_MESSAGE_MAX_CHARS. Prefer splitting at newlines.
fn split_message_for_discord(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut remaining = text.to_string();
    while !remaining.is_empty() {
        let nchars = remaining.chars().count();
        if nchars <= DISCORD_MESSAGE_MAX_CHARS {
            out.push(remaining.clone());
            break;
        }
        let byte_pos = remaining
            .char_indices()
            .take(DISCORD_MESSAGE_MAX_CHARS)
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(0);
        let (head, tail) = remaining.split_at(byte_pos);
        let split_at = head.rfind('\n').map(|i| i + 1).unwrap_or(byte_pos);
        let (chunk, put_back) = if split_at > 0 && split_at < byte_pos {
            (
                head[..split_at].to_string(),
                format!("{}{}", &head[split_at..], tail),
            )
        } else {
            (head.to_string(), tail.to_string())
        };
        out.push(chunk);
        remaining = put_back;
    }
    out
}

/// If the question starts with "switch your model to: X" or "switch model to: X" or "use model X",
/// extract the model name and the rest of the question (after " and " / " then "). Used so the user
/// can say "switch model to: llama3 and explain Y" and we use that model for the reply.
fn extract_model_switch_from_question(question: &str) -> Option<(String, String)> {
    let t = question.trim();
    if t.is_empty() {
        return None;
    }
    let lower = t.to_lowercase();
    let (prefix_len, after_prefix): (usize, &str) = if lower.starts_with("switch your model to:") {
        (21, t.get(21..).unwrap_or("").trim_start())
    } else if lower.starts_with("switch model to:") {
        (15, t.get(15..).unwrap_or("").trim_start())
    } else if lower.starts_with("use model ") {
        (10, t.get(10..).unwrap_or("").trim_start())
    } else if lower.starts_with("use the model ") {
        (14, t.get(14..).unwrap_or("").trim_start())
    } else {
        return None;
    };
    if after_prefix.is_empty() {
        return None;
    }
    // Model name can contain / and : (e.g. huihui_ai/gpt-oss-abliterated:latest). Split on " and " or " then ".
    let rest_lower = lower.get(prefix_len..).unwrap_or("");
    let (model_name, rest) = if let Some(and_pos) = rest_lower.find(" and ") {
        let model = after_prefix[..and_pos.min(after_prefix.len())]
            .trim()
            .to_string();
        let tail = after_prefix
            .get(and_pos + 5..)
            .unwrap_or("")
            .trim()
            .to_string();
        (model, tail)
    } else if let Some(then_pos) = rest_lower.find(" then ") {
        let model = after_prefix[..then_pos.min(after_prefix.len())]
            .trim()
            .to_string();
        let tail = after_prefix
            .get(then_pos + 6..)
            .unwrap_or("")
            .trim()
            .to_string();
        (model, tail)
    } else {
        let model = after_prefix
            .split_whitespace()
            .next()
            .map(|s| s.to_string())
            .unwrap_or_else(|| after_prefix.to_string());
        (model, String::new())
    };
    if model_name.is_empty() {
        return None;
    }
    Some((model_name, rest))
}

/// Parse leading "model: ...", "temperature: ...", "num_ctx: ...", "skill: ...", "agent: ...", "verbose" from a Discord message.
/// Returns (rest of message, model_override, options_override, skill_content, requested_skill_selector, agent_selector, verbose).
/// `requested_skill_selector`: Some if user wrote "skill: X" (so caller can detect "skill not found" when skill_content is None).
/// `verbose`: None = not set (use default from config: DM vs channel), Some(true/false) = explicit.
/// When verbose is false, status/thinking messages are suppressed in the channel.
fn parse_discord_ollama_overrides(
    content: &str,
) -> (
    String,
    Option<String>,
    Option<crate::ollama::ChatOptions>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<bool>,
) {
    let mut model_override: Option<String> = None;
    let mut temperature: Option<f32> = None;
    let mut num_ctx: Option<u32> = None;
    let mut skill_selector: Option<String> = None;
    let mut agent_selector: Option<String> = None;
    let mut verbose: Option<bool> = None;
    let lines: Vec<&str> = content.lines().collect();
    let mut consumed = 0;

    for line in lines.iter() {
        let line = line.trim();
        if line.is_empty() {
            consumed += 1;
            continue;
        }
        let lower = line.to_lowercase();
        if lower == "verbose"
            || lower == "verbose:"
            || lower == "verbose: true"
            || lower == "verbose=true"
        {
            verbose = Some(true);
            consumed += 1;
        } else if lower == "verbose: false" || lower == "verbose=false" {
            verbose = Some(false);
            consumed += 1;
        } else if lower.starts_with("model:") {
            let v = line["model:".len()..].trim().to_string();
            if !v.is_empty() {
                model_override = Some(v);
            }
            consumed += 1;
        } else if lower.starts_with("model=") {
            let v = line["model=".len()..].trim().to_string();
            if !v.is_empty() {
                model_override = Some(v);
            }
            consumed += 1;
        } else if lower.starts_with("skill:") {
            let v = line["skill:".len()..].trim().to_string();
            if !v.is_empty() {
                skill_selector = Some(v);
            }
            consumed += 1;
        } else if lower.starts_with("skill=") {
            let v = line["skill=".len()..].trim().to_string();
            if !v.is_empty() {
                skill_selector = Some(v);
            }
            consumed += 1;
        } else if lower.starts_with("agent:") {
            let v = line["agent:".len()..].trim().to_string();
            if !v.is_empty() {
                agent_selector = Some(v);
            }
            consumed += 1;
        } else if lower.starts_with("agent=") {
            let v = line["agent=".len()..].trim().to_string();
            if !v.is_empty() {
                agent_selector = Some(v);
            }
            consumed += 1;
        } else if lower.starts_with("temperature:") {
            if let Ok(t) = line["temperature:".len()..].trim().parse::<f32>() {
                temperature = Some(t);
            }
            consumed += 1;
        } else if lower.starts_with("temperature=") {
            if let Ok(t) = line["temperature=".len()..].trim().parse::<f32>() {
                temperature = Some(t);
            }
            consumed += 1;
        } else if lower.starts_with("num_ctx:") {
            if let Ok(n) = line["num_ctx:".len()..].trim().parse::<u32>() {
                num_ctx = Some(n);
            }
            consumed += 1;
        } else if lower.starts_with("num_ctx=") {
            if let Ok(n) = line["num_ctx=".len()..].trim().parse::<u32>() {
                num_ctx = Some(n);
            }
            consumed += 1;
        } else if lower.starts_with("params:") {
            let rest = line["params:".len()..].trim();
            for part in rest.split_whitespace() {
                if let Some((k, v)) = part.split_once('=') {
                    let k = k.to_lowercase();
                    if k == "temperature" {
                        if let Ok(t) = v.parse::<f32>() {
                            temperature = Some(t);
                        }
                    } else if k == "num_ctx" {
                        if let Ok(n) = v.parse::<u32>() {
                            num_ctx = Some(n);
                        }
                    }
                }
            }
            consumed += 1;
        } else {
            break;
        }
    }

    let question = lines[consumed..].join("\n").trim().to_string();
    let options_override = if temperature.is_some() || num_ctx.is_some() {
        Some(crate::ollama::ChatOptions {
            temperature,
            num_ctx,
        })
    } else {
        None
    };
    let (skill_content, requested_skill_selector) = match skill_selector {
        Some(ref sel) => {
            let skills = crate::skills::load_skills();
            let content = crate::skills::find_skill_by_number_or_topic(&skills, sel).map(|s| s.content.clone());
            (content, Some(sel.clone()))
        }
        None => (None, None),
    };
    (
        question,
        model_override,
        options_override,
        skill_content,
        requested_skill_selector,
        agent_selector,
        verbose,
    )
}

/// True if the message clearly requests tools (search, browser, screenshot, send here) that only the full agent router can fulfill.
/// In having_fun channels we use this to route such messages to answer_with_ollama_and_fetch instead of casual chat.
fn message_wants_agent_tools(content: &str) -> bool {
    let lower = content.trim().to_lowercase();
    if lower.len() < 10 {
        return false;
    }
    let has_search = lower.contains("perplexity")
        || lower.contains("brave search")
        || lower.contains("search the web")
        || lower.contains("search for ");
    let has_browser = lower.contains("browser")
        || lower.contains("visit ")
        || lower.contains(" open ")
        || lower.contains("url")
        || lower.contains("extract the url");
    let has_screenshot = lower.contains("screenshot")
        || lower.contains("take a screen")
        || lower.contains("capture");
    let has_send_here = lower.contains("send me")
        || lower.contains("send the")
        || lower.contains("here in discord")
        || lower.contains("in discord");
    (has_search || has_browser) && (has_screenshot || has_send_here)
        || (has_screenshot && has_send_here)
        || (lower.contains("perplexity")
            && lower.contains("url")
            && (lower.contains("visit") || lower.contains("screenshot")))
}

/// True if the message indicates the user is not satisfied and wants the task actually completed.
/// Patterns are loaded from ~/.mac-stats/agents/escalation_patterns.md (one phrase per line; user-editable).
fn is_escalation_message(question: &str) -> bool {
    let lower = question.trim().to_lowercase();
    if lower.is_empty() {
        return false;
    }
    let patterns = crate::config::Config::load_escalation_patterns();
    patterns.iter().any(|p| lower.contains(&p.to_lowercase()))
}

/// True if we already spawned the gateway thread (only one gateway per process).
static GATEWAY_STARTED: AtomicBool = AtomicBool::new(false);

/// Shared shard manager for graceful disconnect on app exit (user appears offline).
static DISCORD_SHARD_MANAGER: OnceLock<Arc<ShardManager>> = OnceLock::new();

/// Keychain account name for the Discord bot token.
pub const DISCORD_TOKEN_KEYCHAIN_ACCOUNT: &str = "discord_bot_token";

/// Bot user id (set on Ready, used to filter self and mentions).
static BOT_USER_ID: OnceLock<UserId> = OnceLock::new();

/// Cache of Discord user id -> display name for reuse in prompts. Updated on each message.
static DISCORD_USER_NAMES: OnceLock<Mutex<HashMap<u64, String>>> = OnceLock::new();

fn discord_user_names() -> &'static Mutex<HashMap<u64, String>> {
    DISCORD_USER_NAMES.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Record a Discord user's display name (call when we receive a message from them).
pub fn set_discord_user_name(user_id: u64, display_name: String) {
    if let Ok(mut map) = discord_user_names().lock() {
        map.insert(user_id, display_name);
    }
}

/// Get a cached Discord display name for a user id, if known.
pub fn get_discord_display_name(user_id: u64) -> Option<String> {
    discord_user_names()
        .lock()
        .ok()
        .and_then(|map| map.get(&user_id).cloned())
}

struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, data_about_bot: serenity::model::gateway::Ready) {
        let id = data_about_bot.user.id;
        let _ = BOT_USER_ID.set(id);
        info!(
            "Discord: Bot connected as {} (id: {})",
            data_about_bot.user.name, id
        );
        tokio::spawn(having_fun_background_loop(ctx));
    }

    async fn message(&self, ctx: Context, new_message: Message) {
        let bot_id = match BOT_USER_ID.get() {
            Some(id) => *id,
            None => {
                debug!("Discord: Ignoring message (bot id not set yet)");
                return;
            }
        };

        // Always ignore our own messages
        if new_message.author.id == bot_id {
            return;
        }

        let is_dm = new_message.guild_id.is_none();
        let mentions_bot = new_message.mentions.iter().any(|u| u.id == bot_id);
        let is_bot = new_message.author.bot;
        let chan_id = new_message.channel_id.get();
        let chan = channel_settings(chan_id);
        let mode = chan.mode;

        let content = {
            let raw = new_message.content.trim();
            let mention_tag = format!("<@{}>", bot_id);
            sanitize_image_error_content(raw.replace(&mention_tag, "").trim())
        };
        let attachment_images_base64 =
            download_discord_image_attachments(&new_message.attachments).await;
        let mut content = content;
        if content.is_empty() && !attachment_images_base64.is_empty() {
            content = DISCORD_IMAGE_ONLY_PROMPT.to_string();
        }
        if content.is_empty() {
            debug!("Discord: Ignoring empty message");
            return;
        }

        if is_bot {
            if mode != ChannelMode::HavingFun {
                return;
            }
        } else if !is_dm && !mentions_bot && mode == ChannelMode::MentionOnly {
            return;
        }

        // having_fun channels: buffer the message and let the background loop respond — unless the user clearly wants tools (search, browser, screenshot, send here), then use full agent router.
        if mode == ChannelMode::HavingFun {
            let from_human_or_mention = !is_bot || mentions_bot;
            if from_human_or_mention && message_wants_agent_tools(&content) {
                info!(
                    "Discord: having_fun channel but message requests tools (perplexity/browser/screenshot/send) — using full agent router"
                );
                // Fall through to full flow (answer_with_ollama_and_fetch) below; do not buffer.
            } else {
                let author_name = new_message
                    .author
                    .global_name
                    .as_deref()
                    .unwrap_or(&new_message.author.name)
                    .to_string();
                info!(
                    "Discord: having_fun buffered from {} (bot={}) in channel {}: {}",
                    author_name,
                    is_bot,
                    chan_id,
                    crate::logging::ellipse(&content, 100)
                );
                // Do not store bot failure/error notices in session so they never appear in idle-thought context.
                if !(is_bot && is_agent_failure_notice(&content)) {
                    crate::session_memory::add_message(
                        "discord",
                        chan_id,
                        "user",
                        &format!("{}: {}", author_name, content),
                    );
                } else {
                    info!(
                        "Discord: having_fun channel {} — not storing failure notice in session (idle-thought context kept casual)",
                        chan_id
                    );
                }
                let answer_asap = mentions_bot || !is_bot;
                buffer_having_fun_message(
                    chan_id,
                    author_name,
                    content,
                    is_bot,
                    answer_asap,
                    Some(new_message.id.get()),
                );
                return;
            }
        }

        let (
            mut question,
            mut model_override,
            options_override,
            skill_content,
            requested_skill_selector,
            agent_selector,
            verbose_opt,
        ) = parse_discord_ollama_overrides(&content);

        // User requested a skill (e.g. "skill: 99") but it was not found — reply with available skills and return.
        if requested_skill_selector.is_some() && skill_content.is_none() {
            let selector = requested_skill_selector.as_deref().unwrap_or("?");
            let skills = crate::skills::load_skills();
            let available: String = if skills.is_empty() {
                "none (add skill-N-topic.md files to ~/.mac-stats/agents/skills/)".to_string()
            } else {
                skills
                    .iter()
                    .map(|s| format!("{}-{}", s.number, s.topic))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let err_msg = format!(
                "Skill \"{}\" not found. Available: {}.",
                selector, available
            );
            info!("Discord: {}", err_msg);
            if let Err(e) = new_message.channel_id.say(&ctx, &err_msg).await {
                error!("Discord: failed to send skill-not-found message: {}", e);
            }
            return;
        }

        let escalation = is_escalation_message(&question);
        if escalation {
            info!("Discord: escalation detected (user wants task actually completed)");
            crate::config::Config::append_escalation_pattern_if_new(&question);
        }
        let verbose = match verbose_opt {
            Some(v) => v,
            None if is_dm => default_verbose_for_dm(),
            None => default_verbose_for_channel(),
        };
        // Natural-language model switch: "switch model to: X and do Y" -> use model X, question = Y
        if model_override.is_none() {
            if let Some((model, rest)) = extract_model_switch_from_question(&question) {
                model_override = Some(model);
                if !rest.is_empty() {
                    question = rest;
                }
            }
        }
        // Channel prompt from discord_channels.json; used when no explicit skill: override
        let skill_content = skill_content.or(chan.prompt);
        // Agent: from message (e.g. "agent: abliterated") or from channel config (e.g. agents-aliberated channel)
        let agents = crate::agents::load_agents();
        let agent_override = agent_selector
            .as_ref()
            .and_then(|sel| crate::agents::find_agent_by_id_or_name(&agents, sel).cloned())
            .or_else(|| {
                chan.agent
                    .as_ref()
                    .and_then(|sel| crate::agents::find_agent_by_id_or_name(&agents, sel).cloned())
            });
        // Model: from message or from channel config (when no agent override)
        let model_override = model_override.or(chan.model.clone());

        let trigger = if is_dm {
            "DM"
        } else if mentions_bot {
            "mention"
        } else {
            "all_messages"
        };
        info!(
            "Discord: {} from {} (channel {}) verbose={}",
            trigger, new_message.author.name, new_message.channel_id, verbose
        );

        let channel_id_u64 = new_message.channel_id.get();

        // "New session:" prefix clears conversation history so the model starts fresh.
        let question = {
            let lower = question.trim().to_lowercase();
            if lower.starts_with("new session:") || lower.starts_with("new session ") {
                crate::session_memory::clear_session("discord", channel_id_u64);
                info!(
                    "Discord: new session requested, cleared history for channel {}",
                    channel_id_u64
                );
                let stripped = question.trim();
                let colon_pos = stripped.find(':').or_else(|| stripped.find(' '));
                match colon_pos {
                    Some(i) if stripped[..i].to_lowercase().trim() == "new session" => {
                        stripped[i + 1..].trim().to_string()
                    }
                    _ => stripped.replacen("new session", "", 1).trim().to_string(),
                }
            } else {
                question.to_string()
            }
        };

        const LOG_MAX: usize = 800;
        let to_ollama = if question.chars().count() <= LOG_MAX {
            question.to_string()
        } else {
            format!(
                "{} ({} chars)",
                crate::logging::ellipse(&question, LOG_MAX),
                question.chars().count()
            )
        };
        info!("Discord→Ollama: sending: {}", to_ollama);

        // Load prior conversation (in-memory, or from latest session file after restart) before adding this turn.
        // If the user asks to clear/new session (any language), clear session and start fresh (see docs/035).
        let did_session_reset = crate::session_memory::user_wants_session_reset(&content);
        let prior = if did_session_reset {
            crate::session_memory::clear_session("discord", channel_id_u64);
            tracing::info!(
                "Discord: user requested session reset (e.g. clear session / new session), starting fresh"
            );
            vec![]
        } else {
            let mut p = crate::session_memory::get_messages("discord", channel_id_u64);
            if p.is_empty() {
                p = crate::session_memory::load_messages_from_latest_session_file(
                    "discord",
                    channel_id_u64,
                );
            }
            p
        };
        let conversation_history: Option<Vec<crate::ollama::ChatMessage>> = if prior.is_empty() {
            None
        } else {
            Some(
                prior
                    .into_iter()
                    .map(|(role, content)| crate::ollama::ChatMessage {
                        role,
                        content,
                        images: None,
                    })
                    .collect(),
            )
        };
        // After session reset, inject Session Startup instruction + current date so the agent knows which memory to read (see Session Startup in docs).
        let question_for_ollama = if did_session_reset {
            tracing::info!("Discord: injected Session Startup + current date (session reset)");
            format!(
                "{}\n\n{}",
                crate::session_memory::session_reset_instruction_with_date_utc(),
                question
            )
        } else {
            question.clone()
        };
        // Short-term memory: add user message when we receive the request (store original content)
        crate::session_memory::add_message("discord", channel_id_u64, "user", &content);

        // Record author's display name for reuse in prompts and API context
        let author_id_u64 = new_message.author.id.get();
        let display_name = new_message
            .author
            .global_name
            .as_deref()
            .unwrap_or(&new_message.author.name)
            .to_string();
        set_discord_user_name(author_id_u64, display_name.clone());
        crate::user_info::maybe_update_display_name_from_discord(author_id_u64, &display_name);

        // Channel for status updates. Only posted to Discord when verbose mode is on;
        // otherwise they are only logged internally to keep the channel clean for other bots.
        let (status_tx, mut status_rx) = mpsc::unbounded_channel::<String>();
        let ctx_send = ctx.clone();
        let channel_id = new_message.channel_id;
        const EDIT_PREFIX: &str = "EDIT:";
        const ATTACH_PREFIX: &str = "ATTACH:";
        const CRITERIA_PROGRESS: &str = "Extracting success criteria…";
        let status_task = tokio::spawn(async move {
            let mut last_criteria_message: Option<Message> = None;
            while let Some(msg) = status_rx.recv().await {
                debug!("Discord status (verbose={}): {}", verbose, msg);
                if !verbose {
                    continue;
                }
                if let Some(path_str) = msg.strip_prefix(ATTACH_PREFIX) {
                    let path = PathBuf::from(path_str.trim());
                    if allowed_attachment_path(&path) {
                        use serenity::builder::CreateAttachment;
                        use serenity::builder::CreateMessage;
                        if let Ok(att) = CreateAttachment::path(&path).await {
                            let builder = CreateMessage::new()
                                .content("Screenshot:")
                                .add_files(vec![att]);
                            if let Err(e) = channel_id.send_message(&ctx_send, builder).await {
                                debug!("Discord: send screenshot now failed: {}", e);
                            }
                        }
                    }
                    continue;
                }
                if let Some(edit_content) = msg.strip_prefix(EDIT_PREFIX) {
                    if let Some(mut m) = last_criteria_message.take() {
                        if let Err(e) = m
                            .edit(&ctx_send, EditMessage::new().content(edit_content))
                            .await
                        {
                            debug!("Discord: edit status message failed: {}", e);
                        }
                    }
                } else {
                    match channel_id.say(&ctx_send, &msg).await {
                        Ok(message) if msg == CRITERIA_PROGRESS => {
                            last_criteria_message = Some(message);
                        }
                        Err(e) => {
                            debug!("Discord: status message failed: {}", e);
                        }
                        _ => {}
                    }
                }
            }
        });

        // Show "Werner_Amvara is typing..." while processing. Fires immediately,
        // then every 8s (indicator lasts ~10s server-side). Cancelled when reply is ready.
        let typing_ctx = ctx.clone();
        let typing_channel = new_message.channel_id;
        let typing_cancel = tokio_util::sync::CancellationToken::new();
        let typing_token = typing_cancel.clone();
        let typing_task = tokio::spawn(async move {
            loop {
                let _ = typing_channel.broadcast_typing(&typing_ctx).await;
                tokio::select! {
                    _ = tokio::time::sleep(std::time::Duration::from_secs(8)) => {}
                    _ = typing_token.cancelled() => break,
                }
            }
        });

        let attachment_images_for_ollama = if attachment_images_base64.is_empty() {
            None
        } else {
            Some(attachment_images_base64)
        };
        if let Some(ref imgs) = attachment_images_for_ollama {
            info!(
                "Discord: sending {} image attachment(s) to Ollama (user_id={}, channel_id={})",
                imgs.len(),
                author_id_u64,
                channel_id_u64
            );
        }
        info!(
            "Discord: processing message (channel {}) — {}",
            channel_id_u64,
            crate::config::Config::version_display()
        );
        let (reply_text, attachment_paths) =
            match crate::commands::ollama::answer_with_ollama_and_fetch(
                &question_for_ollama,
                Some(status_tx),
                Some(channel_id_u64),
                Some(author_id_u64),
                Some(display_name),
                model_override,
                options_override,
                skill_content,
                agent_override,
                true,
                conversation_history,
                escalation,
                true, // retry once when verification says NO
                true, // from_remote: use headless browser unless user asks to see it
                attachment_images_for_ollama,
                None,  // discord_intermediate: set only when recursing from retry path
                false, // is_verification_retry
                None,  // original_user_request
                None,  // success_criteria_override
                Some(is_dm), // load global memory only in main session (DM or in-app)
                None,  // request_id_override: generated on first run
                0,     // retry_count
            )
            .await
            {
                Ok(r) => (r.text, r.attachment_paths),
                Err(e) => {
                    error!(
                        "Discord: Failed to generate reply (channel {}): {}",
                        channel_id_u64, e
                    );
                    // In having_fun channels, do not post technical errors or CLI hints; use a short user-friendly message only.
                    let (reply_text, attachments) = if mode == ChannelMode::HavingFun {
                        (
                            "Something went wrong on my side — try again in a bit.".to_string(),
                            Vec::new(),
                        )
                    } else {
                        let err_lower = e.to_string().to_lowercase();
                        let hint = if err_lower.contains("timed out") || err_lower.contains("timeout") {
                            "Request timed out — Ollama may be busy; try again in a moment."
                        } else {
                            "Is Ollama configured?"
                        };
                        (
                            format!("Sorry, I couldn't generate a reply: {}. ({})", e, hint),
                            Vec::new(),
                        )
                    };
                    (reply_text, attachments)
                }
            };

        typing_cancel.cancel();
        let _ = typing_task.await;

        // Sender was moved into answer_with_ollama_and_fetch and is dropped when it returns, so status_rx gets None.
        // Wait for the status task to finish so all status messages are sent before we send the final reply.
        let _ = status_task.await;

        // Optional agent judge: when enabled, evaluate run and log verdict to debug log (no user impact).
        crate::commands::judge::run_judge_if_enabled(
            &question_for_ollama,
            &reply_text,
            &attachment_paths,
            None,
        )
        .await;

        // Log full reply if ≤500 chars (or always in -vv), else first 500 + ellipsis.
        const RECV_LOG_MAX: usize = 500;
        let reply = strip_leading_label(reply_text.trim());
        let nchars = reply.chars().count();
        let verbosity = crate::logging::VERBOSITY.load(Ordering::Relaxed);
        if verbosity >= 2 || nchars <= RECV_LOG_MAX {
            info!("Discord←Ollama: received ({} chars): {}", nchars, reply);
        } else {
            info!(
                "Discord←Ollama: received ({} chars): {}",
                nchars,
                crate::logging::ellipse(&reply, RECV_LOG_MAX)
            );
        }

        let chunks = split_message_for_discord(&reply);
        for (i, chunk) in chunks.iter().enumerate() {
            if verbosity >= 3 {
                debug!(
                    "Discord outbound (decoded) reply part {}/{}: {}",
                    i + 1,
                    chunks.len(),
                    chunk
                );
            }
            let mut say_result = new_message.channel_id.say(&ctx, chunk).await;
            if say_result.is_err() {
                if let Err(ref e) = say_result {
                    let err_str = e.to_string();
                    error!(
                        "Discord: Failed to send reply (part {}/{}): {}",
                        i + 1,
                        chunks.len(),
                        err_str
                    );
                    let lower = err_str.to_lowercase();
                    if lower.contains("permission") || lower.contains("missing permissions") {
                        info!(
                            "Discord: missing permissions for channel {} — ensure bot has Send Messages and View Channel in this channel (and in server invite: bot scope with these permissions)",
                            channel_id_u64
                        );
                    }
                }
                // One retry on transient/rate errors
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                say_result = new_message.channel_id.say(&ctx, chunk).await;
            }
            if let Err(e) = say_result {
                let err_str = e.to_string();
                let is_permission = err_str.to_lowercase().contains("permission");
                let fallback = if is_permission {
                    "Reply could not be sent to this channel (missing permissions). Check bot permissions for this channel."
                } else {
                    "Reply could not be sent to this channel. Check bot permissions or try again later."
                };
                if let Err(e2) = new_message.channel_id.say(&ctx, fallback).await {
                    error!("Discord: could not send fallback message either: {}", e2);
                } else {
                    info!("Discord: sent fallback message to channel {} (reply send failed: {})", channel_id_u64, err_str);
                }
                break;
            }
            if chunks.len() > 1 && i < chunks.len() - 1 {
                tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
            }
        }

        // Send attachment(s) if any (e.g. BROWSER_SCREENSHOT); only paths under ~/.mac-stats/screenshots/.
        // Always send the batch so screenshots reliably reach Discord (verbose per-ATTACH can be unreliable).
        let allowed: Vec<_> = attachment_paths
            .iter()
            .filter(|p| allowed_attachment_path(p))
            .cloned()
            .collect();
        if allowed.len() != attachment_paths.len() && !attachment_paths.is_empty() {
            info!(
                "Discord: {} of {} attachment(s) under screenshots dir (rest skipped)",
                allowed.len(),
                attachment_paths.len()
            );
        }
        if !attachment_paths.is_empty() && allowed.is_empty() {
            info!(
                "Discord: had {} attachment path(s) but none allowed (must be under ~/.mac-stats/screenshots/)",
                attachment_paths.len()
            );
        }
        if !allowed.is_empty() {
            info!(
                "Discord: sending {} screenshot(s) to channel {}",
                allowed.len(),
                channel_id_u64
            );
            use serenity::builder::CreateAttachment;
            use serenity::builder::CreateMessage;
            let mut attachments = Vec::with_capacity(allowed.len());
            for path in &allowed {
                match CreateAttachment::path(path).await {
                    Ok(att) => attachments.push(att),
                    Err(e) => {
                        error!(
                            "Discord: failed to read attachment {}: {}",
                            path.display(),
                            e
                        );
                    }
                }
            }
            if !attachments.is_empty() {
                let builder = CreateMessage::new()
                    .content("Screenshot(s) as requested:")
                    .add_files(attachments);
                let send_result = new_message.channel_id.send_message(&ctx, builder).await;
                if let Err(ref e) = send_result {
                    let err_str = e.to_string();
                    error!(
                        "Discord: Failed to send attachment(s) to channel {}: {}",
                        channel_id_u64, err_str
                    );
                    let lower = err_str.to_lowercase();
                    if lower.contains("permission") || lower.contains("missing permissions") {
                        info!(
                            "Discord: missing permissions for channel {} (attachments) — ensure bot has Send Messages and Attach Files in this channel",
                            channel_id_u64
                        );
                    }
                }
                if let Err(_e) = send_result {
                    let fallback = "Could not send attachment(s) to this channel (check bot permissions: Send Messages, Attach Files).";
                    if let Err(e2) = new_message.channel_id.say(&ctx, fallback).await {
                        error!("Discord: could not send fallback message for attachment failure: {}", e2);
                    }
                } else {
                    info!(
                        "Discord: sent {} attachment(s) to channel {}",
                        allowed.len(),
                        channel_id_u64
                    );
                }
            } else {
                error!(
                    "Discord: all {} path(s) failed CreateAttachment::path",
                    allowed.len()
                );
            }
        }

        // Short-term memory: add assistant reply (user was added when request received); persist when > 3 messages
        crate::session_memory::add_message("discord", channel_id_u64, "assistant", &reply);
    }
}

/// Run the Discord client (async). Call from a tokio runtime or block_on.
/// Token must be non-empty.
pub async fn run_discord_client(token: String) -> Result<(), String> {
    if token.trim().is_empty() {
        return Err("Discord token is empty".to_string());
    }

    info!("Discord: Connecting to Discord Gateway (discord.com)…");

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .map_err(|e| format!("Discord client build failed: {}", e))?;

    // Store shard manager so we can call shutdown_all() on app exit (user appears offline).
    let _ = DISCORD_SHARD_MANAGER.set(client.shard_manager.clone());

    info!("Discord: Gateway client built, starting connection…");
    client
        .start()
        .await
        .map_err(|e| format!("Discord gateway error: {}", e))?;
    Ok(())
}

/// Disconnect from Discord on app shutdown so the user appears offline.
/// Safe to call even if Discord was never started or already disconnected.
pub fn disconnect_discord() {
    let Some(manager) = DISCORD_SHARD_MANAGER.get() else {
        debug!("Discord: No shard manager (gateway was not started), skipping disconnect");
        return;
    };
    info!("Discord: Logging off (shutting down gateway)…");
    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            error!("Discord: Failed to create runtime for shutdown: {}", e);
            return;
        }
    };
    rt.block_on(manager.shutdown_all());
    info!("Discord: Gateway shut down (user offline)");
}

/// Read Discord token from a .config.env-style file (DISCORD_BOT_TOKEN= or DISCORD-USER1/2-TOKEN=).
fn token_from_config_env_file(path: &Path) -> Option<String> {
    // Do not log file content or path; file may contain secrets.
    let content = std::fs::read_to_string(path).ok()?;
    let token = content
        .lines()
        .find(|l| {
            l.starts_with("DISCORD_BOT_TOKEN=")
                || l.starts_with("DISCORD-USER1-TOKEN=")
                || l.starts_with("DISCORD-USER2-TOKEN=")
        })
        .and_then(|l| l.split_once('='))
        .map(|(_, v)| v.trim().to_string());
    token.filter(|t| !t.is_empty())
}

/// Returns true only if `path` is under `~/.mac-stats/screenshots/` (canonicalized) so we never send arbitrary files.
fn allowed_attachment_path(path: &Path) -> bool {
    let Ok(canon_path) = path.canonicalize() else {
        return false;
    };
    let Ok(allowed_dir) = crate::config::Config::screenshots_dir().canonicalize() else {
        return false;
    };
    canon_path.starts_with(allowed_dir)
}

/// Send a message to a Discord channel with optional file attachments (e.g. screenshots).
/// Paths must be under ~/.mac-stats/screenshots/; others are skipped.
pub async fn send_message_to_channel_with_attachments(
    channel_id: u64,
    content: &str,
    attachment_paths: &[PathBuf],
) -> Result<(), String> {
    let token = match get_discord_token() {
        Some(t) => t,
        None => return Err("Discord not configured (no token)".to_string()),
    };
    let allowed: Vec<_> = attachment_paths
        .iter()
        .filter(|p| allowed_attachment_path(p))
        .collect();
    if allowed.is_empty() {
        return send_message_to_channel(channel_id, content).await;
    }
    const MAX_LEN: usize = 2000;
    let content = if content.chars().count() > MAX_LEN {
        crate::logging::ellipse(content, MAX_LEN)
    } else {
        content.to_string()
    };
    let url = format!(
        "https://discord.com/api/v10/channels/{}/messages",
        channel_id
    );
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("HTTP client: {}", e))?;
    let mut form = reqwest::multipart::Form::new().text("content", content);
    for (i, path) in allowed.iter().enumerate() {
        let name = format!("files[{}]", i);
        let data = tokio::fs::read(path.as_path())
            .await
            .map_err(|e| format!("Read attachment {}: {}", path.display(), e))?;
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("screenshot.png")
            .to_string();
        form = form.part(
            name,
            reqwest::multipart::Part::bytes(data).file_name(filename),
        );
    }
    // Do not log request/response headers or bodies that may contain credentials.
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bot {}", token))
        .multipart(form)
        .send()
        .await
        .map_err(|e| crate::discord::api::user_message_for_discord_request_error(&e))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Discord API {}: {}", status, body));
    }
    Ok(())
}

/// Send a message to a Discord channel (DM or guild channel). Used by the scheduler to post task results.
/// Requires the bot token; uses Discord HTTP API so it works from any thread/runtime.
pub async fn send_message_to_channel(channel_id: u64, content: &str) -> Result<(), String> {
    const MAX_LEN: usize = 2000;
    let token = match get_discord_token() {
        Some(t) => t,
        None => return Err("Discord not configured (no token)".to_string()),
    };
    let content = if content.chars().count() > MAX_LEN {
        crate::logging::ellipse(content, MAX_LEN)
    } else {
        content.to_string()
    };
    if crate::logging::VERBOSITY.load(Ordering::Relaxed) >= 3 {
        debug!(
            "Discord outbound (decoded) send_message_to_channel: {}",
            content
        );
    }
    let url = format!(
        "https://discord.com/api/v10/channels/{}/messages",
        channel_id
    );
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("HTTP client: {}", e))?;
    // Do not log request/response headers or bodies that may contain credentials.
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bot {}", token))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "content": content }))
        .send()
        .await
        .map_err(|e| crate::discord::api::user_message_for_discord_request_error(&e))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Discord API {}: {}", status, body));
    }
    Ok(())
}

/// Get Discord token: DISCORD_BOT_TOKEN env, then .config.env (cwd then ~/.mac-stats), then Keychain.
/// Prefer env and file so the app works without Keychain access.
pub fn get_discord_token() -> Option<String> {
    if let Ok(t) = std::env::var("DISCORD_BOT_TOKEN") {
        let t = t.trim().to_string();
        if !t.is_empty() {
            info!("Discord: Token from DISCORD_BOT_TOKEN env");
            return Some(t);
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        let p = cwd.join(".config.env");
        if p.is_file() {
            if let Some(t) = token_from_config_env_file(&p) {
                info!("Discord: Token from .config.env (current dir)");
                return Some(t);
            }
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let p = Path::new(&home).join(".mac-stats").join(".config.env");
        if p.is_file() {
            if let Some(t) = token_from_config_env_file(&p) {
                info!("Discord: Token from ~/.mac-stats/.config.env");
                return Some(t);
            }
        }
    }
    match crate::security::get_credential(DISCORD_TOKEN_KEYCHAIN_ACCOUNT) {
        Ok(Some(t)) if !t.trim().is_empty() => {
            info!("Discord: Token from Keychain");
            Some(t)
        }
        Ok(Some(_)) => None,
        Ok(None) => None,
        Err(e) => {
            debug!(
                "Discord: Keychain read failed (using env/file instead): {}",
                e
            );
            None
        }
    }
}

/// Spawn the Discord gateway in a background thread if token is present.
/// Loads token via get_discord_token() (env, .config.env, then Keychain).
/// Safe to call multiple times: only one gateway thread is started per process.
pub fn spawn_discord_if_configured() {
    if GATEWAY_STARTED.swap(true, Ordering::SeqCst) {
        debug!("Discord: Gateway already started, skipping");
        return;
    }

    let token = match get_discord_token() {
        Some(t) => {
            info!("Discord: Token found, spawning gateway thread");
            t
        }
        None => {
            info!("Discord: No token (env, .config.env, or Keychain), skipping gateway");
            GATEWAY_STARTED.store(false, Ordering::SeqCst);
            return;
        }
    };

    std::thread::spawn(move || {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(r) => r,
            Err(e) => {
                error!("Discord: Failed to create tokio runtime: {}", e);
                return;
            }
        };
        if let Err(e) = rt.block_on(run_discord_client(token)) {
            error!("Discord: Gateway stopped: {}", e);
        }
    });
    info!("Discord: Gateway thread spawned (connecting to Discord API)");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowed_attachment_path_under_screenshots_only() {
        let screenshots = crate::config::Config::screenshots_dir();
        let _ = std::fs::create_dir_all(&screenshots);
        let under = screenshots.join("test_attachment_allowed.png");
        let _ = std::fs::write(&under, b"x");
        assert!(
            allowed_attachment_path(&under),
            "path under screenshots_dir should be allowed"
        );
        let _ = std::fs::remove_file(&under);

        let outside = std::env::temp_dir()
            .join("mac-stats-attachment-test-outside")
            .join("file.png");
        let _ = std::fs::create_dir_all(outside.parent().unwrap());
        let _ = std::fs::write(&outside, b"x");
        assert!(
            !allowed_attachment_path(&outside),
            "path outside screenshots_dir should be rejected"
        );
        let _ = std::fs::remove_file(&outside);
        let _ = std::fs::remove_dir(outside.parent().unwrap());
    }
}
