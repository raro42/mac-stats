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

/// Parse leading "model: ...", "temperature: ...", "num_ctx: ...", "skill: ...", "agent: ..." from a Discord message.
/// Returns (rest of message as question, model_override, options_override, skill_content, agent_selector).
fn parse_discord_ollama_overrides(
    content: &str,
) -> (
    String,
    Option<String>,
    Option<crate::ollama::ChatOptions>,
    Option<String>,
    Option<String>,
) {
    let mut model_override: Option<String> = None;
    let mut temperature: Option<f32> = None;
    let mut num_ctx: Option<u32> = None;
    let mut skill_selector: Option<String> = None;
    let mut agent_selector: Option<String> = None;
    let lines: Vec<&str> = content.lines().collect();
    let mut consumed = 0;

    for line in lines.iter() {
        let line = line.trim();
        if line.is_empty() {
            consumed += 1;
            continue;
        }
        let lower = line.to_lowercase();
        if lower.starts_with("model:") {
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
    (question, model_override, options_override, skill_content, agent_selector)
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
    async fn ready(&self, _ctx: Context, data_about_bot: serenity::model::gateway::Ready) {
        let id = data_about_bot.user.id;
        let _ = BOT_USER_ID.set(id);
        info!("Discord: Bot connected as {} (id: {})", data_about_bot.user.name, id);
    }

    async fn message(&self, ctx: Context, new_message: Message) {
        let bot_id = match BOT_USER_ID.get() {
            Some(id) => *id,
            None => {
                debug!("Discord: Ignoring message (bot id not set yet)");
                return;
            }
        };

        // Ignore our own messages
        if new_message.author.id == bot_id {
            return;
        }

        // Respond only to DMs or when we are mentioned
        let is_dm = new_message.guild_id.is_none();
        let mentions_bot = new_message.mentions.iter().any(|u| u.id == bot_id);
        if !is_dm && !mentions_bot {
            return;
        }

        let content = new_message.content.trim();
        if content.is_empty() {
            debug!("Discord: Ignoring empty message");
            return;
        }

        let (question, model_override, options_override, skill_content, agent_selector) =
            parse_discord_ollama_overrides(content);
        let agent_override = agent_selector.and_then(|sel| {
            let agents = crate::agents::load_agents();
            crate::agents::find_agent_by_id_or_name(&agents, &sel).cloned()
        });

        info!(
            "Discord: {} from {} (channel {})",
            if is_dm { "DM" } else { "mention" },
            new_message.author.name,
            new_message.channel_id
        );

        const LOG_MAX: usize = 800;
        let to_ollama = if question.len() <= LOG_MAX {
            question.to_string()
        } else {
            format!("{}... ({} chars)", question.chars().take(LOG_MAX).collect::<String>(), question.len())
        };
        info!("Discord→Ollama: sending: {}", to_ollama);

        // Short-term memory: add user message when we receive the request (store original content)
        let channel_id_u64 = new_message.channel_id.get();
        crate::session_memory::add_message("discord", channel_id_u64, "user", content);

        // Record author's display name for reuse in prompts and API context
        let author_id_u64 = new_message.author.id.get();
        let display_name = new_message
            .author
            .global_name
            .as_deref()
            .unwrap_or(&new_message.author.name)
            .to_string();
        set_discord_user_name(author_id_u64, display_name.clone());

        // Channel for status updates so the user sees we're still working (Thinking…, Fetching page…, etc.)
        let (status_tx, mut status_rx) = mpsc::unbounded_channel();
        let ctx_send = ctx.clone();
        let channel_id = new_message.channel_id;
        let status_task = tokio::spawn(async move {
            while let Some(msg) = status_rx.recv().await {
                if let Err(e) = channel_id.say(&ctx_send, &msg).await {
                    debug!("Discord: status message failed: {}", e);
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
        ).await {
            Ok(r) => r,
            Err(e) => {
                error!("Discord: Failed to generate reply: {}", e);
                format!("Sorry, I couldn't generate a reply: {}. (Is Ollama configured?)", e)
            }
        };

        // Sender was moved into answer_with_ollama_and_fetch and is dropped when it returns, so status_rx gets None.
        // Wait for the status task to finish so all status messages are sent before we send the final reply.
        let _ = status_task.await;

        // Log full reply if ≤500 chars, else first 500 and clip.
        const RECV_LOG_MAX: usize = 500;
        let nchars = reply.chars().count();
        if nchars <= RECV_LOG_MAX {
            info!("Discord←Ollama: received ({} chars): {}", nchars, reply);
        } else {
            let head: String = reply.chars().take(RECV_LOG_MAX).collect();
            info!("Discord←Ollama: received ({} chars, first {}): {}... [truncated]", nchars, RECV_LOG_MAX, head);
        }

        let chunks = split_message_for_discord(&reply);
        for (i, chunk) in chunks.iter().enumerate() {
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
        format!("{}... [truncated]", content.chars().take(MAX_LEN - 20).collect::<String>())
    } else {
        content.to_string()
    };
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
