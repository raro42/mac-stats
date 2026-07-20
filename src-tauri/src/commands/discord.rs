//! Tauri commands for Discord agent (Gateway, token configuration).

use crate::discord::DISCORD_TOKEN_KEYCHAIN_ACCOUNT;
use crate::security;
use tracing::info;

/// Outcome of the Keychain operation (for logging in the async context).
enum ConfigureOutcome {
    Stored,
    ClearedEmpty,
    Removed,
}

/// Configure Discord bot token. Stores in Keychain; never logged.
/// Pass Some(token) to set, None to remove.
/// When a token is saved, the gateway starts immediately (no restart needed). Clearing requires restart to disconnect.
/// Keychain runs on the command thread so all logs appear in order.
#[tauri::command]
pub async fn configure_discord(token: Option<String>) -> Result<(), String> {
    let has_token = token
        .as_ref()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    info!(
        "Discord: configure_discord invoked (has_token={}, len={})",
        has_token,
        token.as_ref().map(|s| s.len()).unwrap_or(0)
    );

    let outcome = match token {
        Some(t) => {
            let trimmed = t.trim();
            if trimmed.is_empty() {
                security::delete_credential(DISCORD_TOKEN_KEYCHAIN_ACCOUNT)
                    .map_err(|e| e.to_string())?;
                ConfigureOutcome::ClearedEmpty
            } else {
                security::store_credential(DISCORD_TOKEN_KEYCHAIN_ACCOUNT, trimmed)
                    .map_err(|e| e.to_string())?;
                ConfigureOutcome::Stored
            }
        }
        None => {
            security::delete_credential(DISCORD_TOKEN_KEYCHAIN_ACCOUNT)
                .map_err(|e| e.to_string())?;
            ConfigureOutcome::Removed
        }
    };

    match outcome {
        ConfigureOutcome::Stored => {
            info!("Discord: Token stored, starting gateway");
            crate::discord::spawn_discord_if_configured();
        }
        ConfigureOutcome::ClearedEmpty => info!("Discord: Token cleared (empty string)"),
        ConfigureOutcome::Removed => info!("Discord: Token removed"),
    }
    info!("Discord: configure_discord finished");
    Ok(())
}

/// Check if Discord is configured (token from env, .config.env, or Keychain). Does not reveal the token.
#[tauri::command]
pub fn is_discord_configured() -> Result<bool, String> {
    Ok(crate::discord::get_discord_token().is_some())
}

/// Whether the Discord gateway has connected (Ready) in this process.
#[tauri::command]
pub fn is_discord_gateway_ready() -> Result<bool, String> {
    Ok(crate::discord::discord_bot_gateway_ready())
}

/// Enable or disable the Discord gateway (CPU icon toggle). Does not change the stored token.
/// Returns the desired online state after the call.
#[tauri::command]
pub fn set_discord_gateway_enabled(enabled: bool) -> Result<bool, String> {
    Ok(crate::discord::set_discord_gateway_enabled(enabled))
}

/// Whether the user wants Discord online (may still be connecting after enable).
#[tauri::command]
pub fn is_discord_gateway_desired_online() -> Result<bool, String> {
    Ok(crate::discord::discord_gateway_desired_online())
}
