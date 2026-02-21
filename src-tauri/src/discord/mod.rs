//! Discord Gateway integration for mac-stats.
//!
//! Connects to Discord as a bot, listens for DMs and @mentions,
//! and can reply using a shared pipeline (Ollama / browser agent).
//! Token is resolved (in order) from: DISCORD_BOT_TOKEN env, .config.env file, Keychain.
//! Token is never logged or exposed.

pub mod api;

use serenity::client::{Client, Context, EventHandler};
use serenity::gateway::ShardManager;
use serenity::model::gateway::GatewayIntents;
use serenity::model::id::UserId;
use serenity::model::channel::Message;
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::mpsc;
use tracing::{debug, error, info};

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

/// Per-channel settings: mode + optional prompt injected into the system context.
#[derive(Debug, Clone)]
struct ChannelSettings {
    mode: ChannelMode,
    prompt: Option<String>,
}

/// Cached channel config, loaded once at startup from `discord_channels.json`.
static CHANNEL_CONFIG: OnceLock<(ChannelSettings, HashMap<u64, ChannelSettings>)> = OnceLock::new();

fn parse_mode(s: &str) -> ChannelMode {
    match s {
        "all_messages" => ChannelMode::AllMessages,
        "having_fun" => ChannelMode::HavingFun,
        _ => ChannelMode::MentionOnly,
    }
}

fn load_channel_config() -> (ChannelSettings, HashMap<u64, ChannelSettings>) {
    let default_settings = ChannelSettings { mode: ChannelMode::MentionOnly, prompt: None };
    let path = crate::config::Config::discord_channels_path();
    let json = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => {
            info!("Discord channels config not found at {:?}, using mention_only default", path);
            return (default_settings, HashMap::new());
        }
    };
    let parsed: serde_json::Value = match serde_json::from_str(&json) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("Discord channels config parse error: {}, using mention_only default", e);
            return (default_settings, HashMap::new());
        }
    };
    let default_mode = parsed
        .get("default")
        .and_then(|v| v.as_str())
        .map(parse_mode)
        .unwrap_or(ChannelMode::MentionOnly);
    let default_prompt = parsed
        .get("default_prompt")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let default_settings = ChannelSettings { mode: default_mode, prompt: default_prompt };

    let mut channels = HashMap::new();
    if let Some(obj) = parsed.get("channels").and_then(|v| v.as_object()) {
        for (k, v) in obj {
            let Ok(id) = k.parse::<u64>() else { continue };
            let settings = if let Some(mode_str) = v.as_str() {
                // Simple format: "channel_id": "mode"
                ChannelSettings { mode: parse_mode(mode_str), prompt: None }
            } else if let Some(obj) = v.as_object() {
                // Extended format: "channel_id": { "mode": "...", "prompt": "..." }
                let mode = obj.get("mode")
                    .and_then(|v| v.as_str())
                    .map(parse_mode)
                    .unwrap_or(ChannelMode::MentionOnly);
                let prompt = obj.get("prompt")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                ChannelSettings { mode, prompt }
            } else {
                continue;
            };
            channels.insert(id, settings);
        }
    }
    info!(
        "Discord channels config: default={:?}, {} channel overrides",
        default_settings.mode,
        channels.len()
    );
    (default_settings, channels)
}

fn channel_settings(channel_id: u64) -> ChannelSettings {
    let (default, overrides) = CHANNEL_CONFIG.get_or_init(load_channel_config);
    overrides.get(&channel_id).cloned().unwrap_or_else(|| default.clone())
}

// ---------------------------------------------------------------------------
// having_fun: buffered responses + idle thoughts
// ---------------------------------------------------------------------------

const HAVING_FUN_MAX_CONSECUTIVE_BOT_REPLIES: u32 = 5;
const HAVING_FUN_RESPONSE_DELAY_SECS: u64 = 30;
const HAVING_FUN_IDLE_THOUGHT_SECS: u64 = 300; // 5 min of silence
const HAVING_FUN_TICK_SECS: u64 = 10;

struct BufferedMessage {
    author_name: String,
    content: String,
    is_bot: bool,
}

struct HavingFunState {
    buffer: Vec<BufferedMessage>,
    consecutive_bot_replies: u32,
    last_response: std::time::Instant,
    last_activity: std::time::Instant,
    last_thought: std::time::Instant,
}

static HAVING_FUN_STATES: OnceLock<Mutex<HashMap<u64, HavingFunState>>> = OnceLock::new();

fn having_fun_states() -> &'static Mutex<HashMap<u64, HavingFunState>> {
    HAVING_FUN_STATES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn buffer_having_fun_message(channel_id: u64, author_name: String, content: String, is_bot: bool) {
    if let Ok(mut map) = having_fun_states().lock() {
        let state = map.entry(channel_id).or_insert_with(|| {
            let now = std::time::Instant::now();
            HavingFunState {
                buffer: Vec::new(),
                consecutive_bot_replies: 0,
                last_response: now,
                last_activity: now,
                last_thought: now,
            }
        });
        if !is_bot {
            state.consecutive_bot_replies = 0;
        }
        if is_bot && state.consecutive_bot_replies >= HAVING_FUN_MAX_CONSECUTIVE_BOT_REPLIES {
            debug!("Discord: dropping bot message in having_fun channel {} (loop protection)", channel_id);
            return;
        }
        state.buffer.push(BufferedMessage { author_name, content, is_bot });
        state.last_activity = std::time::Instant::now();
    }
}

/// Background loop for having_fun channels: flushes buffered messages every 30s,
/// posts random thoughts after 5min of silence.
async fn having_fun_background_loop(ctx: Context) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(HAVING_FUN_TICK_SECS));
    loop {
        interval.tick().await;

        // --- Phase 1: flush channels with buffered messages ---
        let channels_to_flush: Vec<(u64, Vec<BufferedMessage>)> = {
            let mut map = match having_fun_states().lock() {
                Ok(m) => m,
                Err(_) => continue,
            };
            let mut flush = Vec::new();
            for (channel_id, state) in map.iter_mut() {
                if !state.buffer.is_empty()
                    && state.last_response.elapsed()
                        >= std::time::Duration::from_secs(HAVING_FUN_RESPONSE_DELAY_SECS)
                {
                    flush.push((*channel_id, std::mem::take(&mut state.buffer)));
                    state.last_response = std::time::Instant::now();
                }
            }
            flush
        };

        for (channel_id, messages) in channels_to_flush {
            let had_bot = messages.iter().any(|m| m.is_bot);
            having_fun_respond(channel_id, messages, &ctx).await;
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
            let (_, overrides) = CHANNEL_CONFIG.get_or_init(load_channel_config);
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
                                >= std::time::Duration::from_secs(HAVING_FUN_IDLE_THOUGHT_SECS)
                            && state.last_thought.elapsed()
                                >= std::time::Duration::from_secs(HAVING_FUN_IDLE_THOUGHT_SECS);
                        if idle { Some(*id) } else { None }
                    } else {
                        None
                    }
                })
                .collect()
        };

        for channel_id in idle_channels {
            if let Ok(mut map) = having_fun_states().lock() {
                if let Some(state) = map.get_mut(&channel_id) {
                    state.last_thought = std::time::Instant::now();
                    state.last_activity = std::time::Instant::now();
                }
            }
            having_fun_idle_thought(channel_id, &ctx).await;
        }
    }
}

/// Flush buffered messages: send them as context to Ollama (direct chat, no planning/tools)
/// and post the reply to the channel.
async fn having_fun_respond(channel_id: u64, messages: Vec<BufferedMessage>, ctx: &Context) {
    let chan = channel_settings(channel_id);
    let soul = crate::config::Config::load_soul_content();

    let mut prior = crate::session_memory::get_messages("discord", channel_id);
    if prior.is_empty() {
        prior = crate::session_memory::load_messages_from_latest_session_file("discord", channel_id);
    }

    let mut ollama_msgs: Vec<crate::ollama::ChatMessage> = Vec::new();
    let mut system = soul;
    if let Some(ref prompt) = chan.prompt {
        system.push_str("\n\n");
        system.push_str(prompt);
    }
    ollama_msgs.push(crate::ollama::ChatMessage {
        role: "system".to_string(),
        content: system,
    });

    const HISTORY_CAP: usize = 20;
    for (role, content) in prior.into_iter().rev().take(HISTORY_CAP).rev() {
        ollama_msgs.push(crate::ollama::ChatMessage { role, content });
    }

    let new_context: String = messages
        .iter()
        .map(|m| format!("{}: {}", m.author_name, m.content))
        .collect::<Vec<_>>()
        .join("\n");
    ollama_msgs.push(crate::ollama::ChatMessage {
        role: "user".to_string(),
        content: new_context,
    });

    let channel = serenity::model::id::ChannelId::new(channel_id);
    let _ = channel.broadcast_typing(&ctx).await;

    match crate::commands::ollama::send_ollama_chat_messages(ollama_msgs, None, None).await {
        Ok(response) => {
            let reply = response.message.content.trim().to_string();
            if reply.is_empty() {
                return;
            }
            info!(
                "Having fun (channel {}): reply ({} chars): {}",
                channel_id,
                reply.len(),
                crate::logging::ellipse(&reply, 200)
            );
            let chunks = split_message_for_discord(&reply);
            for chunk in &chunks {
                let _ = channel.say(&ctx, chunk).await;
            }
            crate::session_memory::add_message("discord", channel_id, "assistant", &reply);
        }
        Err(e) => {
            debug!("Having fun: Ollama failed for channel {}: {}", channel_id, e);
        }
    }
}

/// Generate and post a random thought when the channel has been quiet.
async fn having_fun_idle_thought(channel_id: u64, ctx: &Context) {
    let chan = channel_settings(channel_id);
    let soul = crate::config::Config::load_soul_content();

    let mut prior = crate::session_memory::get_messages("discord", channel_id);
    if prior.is_empty() {
        prior = crate::session_memory::load_messages_from_latest_session_file("discord", channel_id);
    }

    let mut ollama_msgs: Vec<crate::ollama::ChatMessage> = Vec::new();
    let mut system = soul;
    if let Some(ref prompt) = chan.prompt {
        system.push_str("\n\n");
        system.push_str(prompt);
    }
    ollama_msgs.push(crate::ollama::ChatMessage {
        role: "system".to_string(),
        content: system,
    });

    const HISTORY_CAP: usize = 10;
    for (role, content) in prior.into_iter().rev().take(HISTORY_CAP).rev() {
        ollama_msgs.push(crate::ollama::ChatMessage { role, content });
    }

    ollama_msgs.push(crate::ollama::ChatMessage {
        role: "user".to_string(),
        content: "The chat has been quiet for a while. Share a random thought, observation, or bring up something interesting. Be casual and brief — one or two sentences.".to_string(),
    });

    let channel = serenity::model::id::ChannelId::new(channel_id);
    let _ = channel.broadcast_typing(&ctx).await;

    match crate::commands::ollama::send_ollama_chat_messages(ollama_msgs, None, None).await {
        Ok(response) => {
            let reply = response.message.content.trim().to_string();
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
            debug!("Having fun: idle thought failed for channel {}: {}", channel_id, e);
        }
    }
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
            (head[..split_at].to_string(), format!("{}{}", &head[split_at..], tail))
        } else {
            (head.to_string(), tail.to_string())
        };
        out.push(chunk);
        remaining = put_back;
    }
    out
}

/// Parse leading "model: ...", "temperature: ...", "num_ctx: ...", "skill: ...", "agent: ...", "verbose" from a Discord message.
/// Returns (rest of message as question, model_override, options_override, skill_content, agent_selector, verbose).
/// When `verbose` is false (the default), status/thinking messages are suppressed in the channel.
fn parse_discord_ollama_overrides(
    content: &str,
) -> (
    String,
    Option<String>,
    Option<crate::ollama::ChatOptions>,
    Option<String>,
    Option<String>,
    bool,
) {
    let mut model_override: Option<String> = None;
    let mut temperature: Option<f32> = None;
    let mut num_ctx: Option<u32> = None;
    let mut skill_selector: Option<String> = None;
    let mut agent_selector: Option<String> = None;
    let mut verbose = false;
    let lines: Vec<&str> = content.lines().collect();
    let mut consumed = 0;

    for line in lines.iter() {
        let line = line.trim();
        if line.is_empty() {
            consumed += 1;
            continue;
        }
        let lower = line.to_lowercase();
        if lower == "verbose" || lower == "verbose:" || lower == "verbose: true" || lower == "verbose=true" {
            verbose = true;
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
    let skill_content = skill_selector.and_then(|sel| {
        let skills = crate::skills::load_skills();
        crate::skills::find_skill_by_number_or_topic(&skills, &sel).map(|s| s.content.clone())
    });
    (question, model_override, options_override, skill_content, agent_selector, verbose)
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
    discord_user_names().lock().ok().and_then(|map| map.get(&user_id).cloned())
}

struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, data_about_bot: serenity::model::gateway::Ready) {
        let id = data_about_bot.user.id;
        let _ = BOT_USER_ID.set(id);
        info!("Discord: Bot connected as {} (id: {})", data_about_bot.user.name, id);
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
            raw.replace(&mention_tag, "").trim().to_string()
        };
        if content.is_empty() {
            debug!("Discord: Ignoring empty message");
            return;
        }

        if is_bot {
            if mode != ChannelMode::HavingFun {
                return;
            }
        } else {
            if !is_dm && !mentions_bot && mode == ChannelMode::MentionOnly {
                return;
            }
        }

        // having_fun channels: buffer the message and let the background loop respond
        if mode == ChannelMode::HavingFun {
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
            crate::session_memory::add_message("discord", chan_id, "user",
                &format!("{}: {}", author_name, content));
            buffer_having_fun_message(chan_id, author_name, content, is_bot);
            return;
        }

        let (question, model_override, options_override, skill_content, agent_selector, verbose) =
            parse_discord_ollama_overrides(&content);
        // Channel prompt from discord_channels.json; used when no explicit skill: override
        let skill_content = skill_content.or(chan.prompt);
        let agent_override = agent_selector.and_then(|sel| {
            let agents = crate::agents::load_agents();
            crate::agents::find_agent_by_id_or_name(&agents, &sel).cloned()
        });

        let trigger = if is_dm {
            "DM"
        } else if mentions_bot {
            "mention"
        } else {
            "all_messages"
        };
        info!(
            "Discord: {} from {} (channel {}) verbose={}",
            trigger,
            new_message.author.name,
            new_message.channel_id,
            verbose
        );

        let channel_id_u64 = new_message.channel_id.get();

        // "New session:" prefix clears conversation history so the model starts fresh.
        let question = {
            let lower = question.trim().to_lowercase();
            if lower.starts_with("new session:") || lower.starts_with("new session ") {
                crate::session_memory::clear_session("discord", channel_id_u64);
                info!("Discord: new session requested, cleared history for channel {}", channel_id_u64);
                let stripped = question.trim();
                let colon_pos = stripped.find(':').or_else(|| stripped.find(' '));
                match colon_pos {
                    Some(i) if stripped[..i].to_lowercase().trim() == "new session" => {
                        stripped[i+1..].trim().to_string()
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
            format!("{} ({} chars)", crate::logging::ellipse(&question, LOG_MAX), question.chars().count())
        };
        info!("Discord→Ollama: sending: {}", to_ollama);

        // Load prior conversation (in-memory, or from latest session file after restart) before adding this turn
        let mut prior = crate::session_memory::get_messages("discord", channel_id_u64);
        if prior.is_empty() {
            prior = crate::session_memory::load_messages_from_latest_session_file("discord", channel_id_u64);
        }
        let conversation_history: Option<Vec<crate::ollama::ChatMessage>> = if prior.is_empty() {
            None
        } else {
            Some(
                prior
                    .into_iter()
                    .map(|(role, content)| crate::ollama::ChatMessage { role, content })
                    .collect(),
            )
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

        // Channel for status updates. Only posted to Discord when verbose mode is on;
        // otherwise they are only logged internally to keep the channel clean for other bots.
        let (status_tx, mut status_rx) = mpsc::unbounded_channel();
        let ctx_send = ctx.clone();
        let channel_id = new_message.channel_id;
        let status_task = tokio::spawn(async move {
            while let Some(msg) = status_rx.recv().await {
                debug!("Discord status (verbose={}): {}", verbose, msg);
                if verbose {
                    if let Err(e) = channel_id.say(&ctx_send, &msg).await {
                        debug!("Discord: status message failed: {}", e);
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

        let reply = match crate::commands::ollama::answer_with_ollama_and_fetch(
            &question,
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
        ).await {
            Ok(r) => r,
            Err(e) => {
                error!("Discord: Failed to generate reply: {}", e);
                format!("Sorry, I couldn't generate a reply: {}. (Is Ollama configured?)", e)
            }
        };

        typing_cancel.cancel();
        let _ = typing_task.await;

        // Sender was moved into answer_with_ollama_and_fetch and is dropped when it returns, so status_rx gets None.
        // Wait for the status task to finish so all status messages are sent before we send the final reply.
        let _ = status_task.await;

        // Log full reply if ≤500 chars (or always in -vv), else first 500 + ellipsis.
        const RECV_LOG_MAX: usize = 500;
        let nchars = reply.chars().count();
        let verbosity = crate::logging::VERBOSITY.load(Ordering::Relaxed);
        if verbosity >= 2 || nchars <= RECV_LOG_MAX {
            info!("Discord←Ollama: received ({} chars): {}", nchars, reply);
        } else {
            info!("Discord←Ollama: received ({} chars): {}", nchars, crate::logging::ellipse(&reply, RECV_LOG_MAX));
        }

        let chunks = split_message_for_discord(&reply);
        for (i, chunk) in chunks.iter().enumerate() {
            if verbosity >= 3 {
                debug!("Discord outbound (decoded) reply part {}/{}: {}", i + 1, chunks.len(), chunk);
            }
            if let Err(e) = new_message.channel_id.say(&ctx, chunk).await {
                error!("Discord: Failed to send reply (part {}/{}): {}", i + 1, chunks.len(), e);
                break;
            }
            if chunks.len() > 1 && i < chunks.len() - 1 {
                tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
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
        debug!("Discord outbound (decoded) send_message_to_channel: {}", content);
    }
    let url = format!("https://discord.com/api/v10/channels/{}/messages", channel_id);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("HTTP client: {}", e))?;
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bot {}", token))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "content": content }))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;
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
            debug!("Discord: Keychain read failed (using env/file instead): {}", e);
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
