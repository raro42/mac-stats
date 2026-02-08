//! Discord Gateway integration for mac-stats.
//!
//! Connects to Discord as a bot, listens for DMs and @mentions,
//! and can reply using a shared pipeline (Ollama / browser agent).
//! Token is resolved (in order) from: DISCORD_BOT_TOKEN env, .config.env file, Keychain.
//! Token is never logged or exposed.

use serenity::client::{Client, Context, EventHandler};
use serenity::gateway::ShardManager;
use serenity::model::gateway::GatewayIntents;
use serenity::model::id::UserId;
use serenity::model::channel::Message;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use tokio::sync::mpsc;
use tracing::{debug, error, info};

/// True if we already spawned the gateway thread (only one gateway per process).
static GATEWAY_STARTED: AtomicBool = AtomicBool::new(false);

/// Shared shard manager for graceful disconnect on app exit (user appears offline).
static DISCORD_SHARD_MANAGER: OnceLock<Arc<ShardManager>> = OnceLock::new();

/// Keychain account name for the Discord bot token.
pub const DISCORD_TOKEN_KEYCHAIN_ACCOUNT: &str = "discord_bot_token";

/// Bot user id (set on Ready, used to filter self and mentions).
static BOT_USER_ID: OnceLock<UserId> = OnceLock::new();

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

        info!(
            "Discord: {} from {} (channel {})",
            if is_dm { "DM" } else { "mention" },
            new_message.author.name,
            new_message.channel_id
        );

        const LOG_MAX: usize = 800;
        let to_ollama = if content.len() <= LOG_MAX {
            content.to_string()
        } else {
            format!("{}... ({} chars)", &content[..LOG_MAX], content.len())
        };
        info!("Discord→Ollama: sending: {}", to_ollama);

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

        let reply = match crate::commands::ollama::answer_with_ollama_and_fetch(content, Some(status_tx)).await {
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

        if let Err(e) = new_message.channel_id.say(&ctx, &reply).await {
            error!("Discord: Failed to send reply: {}", e);
        }
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
