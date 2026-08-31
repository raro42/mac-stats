//! Alert Tauri commands

use crate::alerts::channels::{MastodonChannel, SlackChannel, TelegramChannel};
use crate::alerts::{Alert, AlertContext, AlertManager};
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;

/// Default Settings channel id for Telegram alerts (Keychain + in-memory registry).
pub const TELEGRAM_SETTINGS_CHANNEL_ID: &str = "default";
/// Keychain account for the Settings Telegram bot token (`telegram_bot_{id}`).
pub const TELEGRAM_BOT_KEYCHAIN_ACCOUNT: &str = "telegram_bot_default";
/// Keychain account for the Settings Telegram chat id (persistence across restarts).
pub const TELEGRAM_CHAT_KEYCHAIN_ACCOUNT: &str = "telegram_chat_default";

// Global alert manager (in production, use proper state management)
fn get_alert_manager() -> &'static Mutex<AlertManager> {
    static ALERT_MANAGER: OnceLock<Mutex<AlertManager>> = OnceLock::new();
    ALERT_MANAGER.get_or_init(|| Mutex::new(AlertManager::new()))
}

fn keychain_nonempty(account: &str) -> Option<String> {
    crate::security::get_credential(account)
        .ok()
        .flatten()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// Bot token from Keychain (Settings default channel), if set.
pub fn get_telegram_bot_token() -> Option<String> {
    keychain_nonempty(TELEGRAM_BOT_KEYCHAIN_ACCOUNT)
}

/// Chat id from Keychain (Settings default channel), if set.
pub fn get_telegram_chat_id() -> Option<String> {
    keychain_nonempty(TELEGRAM_CHAT_KEYCHAIN_ACCOUNT)
}

/// Re-register the Settings Telegram channel after restart when both Keychain values exist.
pub fn restore_persisted_telegram_channel() {
    let Some(chat_id) = get_telegram_chat_id() else {
        return;
    };
    if get_telegram_bot_token().is_none() {
        return;
    }
    let channel = TelegramChannel::new(TELEGRAM_SETTINGS_CHANNEL_ID.to_string(), chat_id);
    if let Ok(mut mgr) = get_alert_manager().lock() {
        mgr.register_channel(
            TELEGRAM_SETTINGS_CHANNEL_ID.to_string(),
            Box::new(channel),
        );
        tracing::info!(
            "Alert: restored Telegram channel '{}' from Keychain",
            TELEGRAM_SETTINGS_CHANNEL_ID
        );
    }
}

/// Settings status for Telegram alert Credentials (token + chat + registered).
#[tauri::command]
pub fn get_telegram_settings_status() -> Result<serde_json::Value, String> {
    let token = get_telegram_bot_token().is_some();
    let chat = get_telegram_chat_id().is_some();
    let registered = count_registered_alert_channels("Telegram") > 0;
    Ok(serde_json::json!({
        "token": token,
        "chat": chat,
        "registered": registered,
        "ready": token && chat && registered,
    }))
}

/// Save Telegram bot token and/or chat id to Keychain; register channel when both present.
#[tauri::command]
pub fn save_telegram_alert_settings(
    bot_token: Option<String>,
    chat_id: Option<String>,
) -> Result<(), String> {
    let token_in = bot_token
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let chat_in = chat_id
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    if token_in.is_none() && chat_in.is_none() {
        return Err("Paste a Telegram bot token and/or chat id first.".to_string());
    }
    if let Some(token) = token_in {
        crate::security::store_credential(TELEGRAM_BOT_KEYCHAIN_ACCOUNT, &token)
            .map_err(|e| format!("store bot token: {e}"))?;
    }
    if let Some(chat) = chat_in {
        crate::security::store_credential(TELEGRAM_CHAT_KEYCHAIN_ACCOUNT, &chat)
            .map_err(|e| format!("store chat id: {e}"))?;
    }
    let token_ok = get_telegram_bot_token().is_some();
    let chat = get_telegram_chat_id();
    if token_ok {
        if let Some(chat) = chat {
            let channel =
                TelegramChannel::new(TELEGRAM_SETTINGS_CHANNEL_ID.to_string(), chat.clone());
            get_alert_manager()
                .lock()
                .map_err(|e| e.to_string())?
                .register_channel(TELEGRAM_SETTINGS_CHANNEL_ID.to_string(), Box::new(channel));
        }
    }
    Ok(())
}

/// Clear Settings Telegram Keychain values and remove the default channel.
#[tauri::command]
pub fn clear_telegram_alert_settings() -> Result<(), String> {
    let _ = crate::security::delete_credential(TELEGRAM_BOT_KEYCHAIN_ACCOUNT);
    let _ = crate::security::delete_credential(TELEGRAM_CHAT_KEYCHAIN_ACCOUNT);
    get_alert_manager()
        .lock()
        .map_err(|e| e.to_string())?
        .remove_channel(TELEGRAM_SETTINGS_CHANNEL_ID);
    Ok(())
}

/// Add an alert
#[tauri::command]
pub fn add_alert(alert: Alert) -> Result<(), String> {
    get_alert_manager()
        .lock()
        .map_err(|e| e.to_string())?
        .add_alert(alert);

    Ok(())
}

/// Remove an alert
#[tauri::command]
pub fn remove_alert(alert_id: String) -> Result<(), String> {
    get_alert_manager()
        .lock()
        .map_err(|e| e.to_string())?
        .remove_alert(&alert_id);

    Ok(())
}

/// Evaluate alerts (called periodically or on events)
#[tauri::command]
pub fn evaluate_alerts(context: AlertContext) -> Result<Vec<String>, String> {
    get_alert_manager()
        .lock()
        .map_err(|e| e.to_string())?
        .evaluate(context)
        .map_err(|e| e.to_string())
}

/// Register a Telegram channel for alerts. Store the bot token in Keychain under `telegram_bot_{id}`.
#[tauri::command]
pub fn register_telegram_channel(id: String, chat_id: String) -> Result<(), String> {
    let channel = TelegramChannel::new(id.clone(), chat_id);
    get_alert_manager()
        .lock()
        .map_err(|e| e.to_string())?
        .register_channel(id, Box::new(channel));
    Ok(())
}

/// Register a Slack channel for alerts. Store the webhook URL in Keychain under `slack_webhook_{id}`.
#[tauri::command]
pub fn register_slack_channel(id: String) -> Result<(), String> {
    let channel = SlackChannel::new(id.clone());
    get_alert_manager()
        .lock()
        .map_err(|e| e.to_string())?
        .register_channel(id, Box::new(channel));
    Ok(())
}

/// Register a Mastodon channel for alerts. Store the API token in Keychain under `mastodon_alert_{id}`.
#[tauri::command]
pub fn register_mastodon_channel(id: String, instance_url: String) -> Result<(), String> {
    let channel = MastodonChannel::new(id.clone(), instance_url);
    get_alert_manager()
        .lock()
        .map_err(|e| e.to_string())?
        .register_channel(id, Box::new(channel));
    Ok(())
}

/// Remove an alert channel by id (Telegram, Slack, or Mastodon).
#[tauri::command]
pub fn remove_alert_channel(channel_id: String) -> Result<(), String> {
    get_alert_manager()
        .lock()
        .map_err(|e| e.to_string())?
        .remove_channel(&channel_id);
    Ok(())
}

/// List registered alert channel IDs (for Settings UI).
#[tauri::command]
pub fn list_alert_channels() -> Result<Vec<String>, String> {
    Ok(get_alert_manager()
        .lock()
        .map_err(|e| e.to_string())?
        .list_channel_ids())
}

/// Count registered alert channels by display name (Telegram / Slack / Signal / Mastodon).
pub fn count_registered_alert_channels(name: &str) -> usize {
    get_alert_manager()
        .lock()
        .map(|m| m.count_channels_named(name))
        .unwrap_or(0)
}

/// Count Keychain accounts with a given prefix that still resolve (config cue; no live send).
pub fn count_alert_keychain_prefix(prefix: &str) -> usize {
    crate::security::list_credentials()
        .unwrap_or_default()
        .into_iter()
        .filter(|a| a.starts_with(prefix))
        .filter(|a| {
            crate::security::get_credential(a)
                .ok()
                .flatten()
                .map(|v| !v.trim().is_empty())
                .unwrap_or(false)
        })
        .count()
}

/// Run alert evaluation in the background. Builds context from current metrics and monitor
/// statuses, then evaluates all alerts. Called periodically from a background thread so
/// SiteDown, BatteryLow, TemperatureHigh, CpuHigh etc. can fire without user action.
pub fn run_periodic_alert_evaluation() {
    use tracing::debug;

    // Build context data without holding the alert manager lock (metrics can be slow)
    let system_metrics = Some(crate::metrics::get_metrics());
    let cpu_details = Some(crate::metrics::get_cpu_details());
    let monitor_snapshot = crate::commands::monitors::get_monitor_statuses_snapshot();

    let mut manager = match get_alert_manager().try_lock() {
        Ok(m) => m,
        Err(_) => {
            debug!("Alert: skip periodic evaluation (lock busy)");
            return;
        }
    };

    // System-only context for BatteryLow, TemperatureHigh, CpuHigh
    let ctx_system = AlertContext {
        monitor_id: None,
        monitor_status: None,
        system_metrics: system_metrics.clone(),
        cpu_details: cpu_details.clone(),
        custom_data: HashMap::new(),
    };
    if let Err(e) = manager.evaluate(ctx_system) {
        debug!("Alert: periodic system evaluation failed: {}", e);
    }

    // Per-monitor context for SiteDown and similar rules
    for (monitor_id, status) in monitor_snapshot {
        let ctx = AlertContext {
            monitor_id: Some(monitor_id.clone()),
            monitor_status: Some(status),
            system_metrics: system_metrics.clone(),
            cpu_details: cpu_details.clone(),
            custom_data: HashMap::new(),
        };
        if let Err(e) = manager.evaluate(ctx) {
            debug!(
                "Alert: periodic evaluation failed for monitor {}: {}",
                monitor_id, e
            );
        }
    }
}
