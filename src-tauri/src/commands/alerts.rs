//! Alert Tauri commands

use crate::alerts::channels::{MastodonChannel, SlackChannel, TelegramChannel};
use crate::alerts::{Alert, AlertContext, AlertManager};
use std::sync::Mutex;
use std::sync::OnceLock;

// Global alert manager (in production, use proper state management)
fn get_alert_manager() -> &'static Mutex<AlertManager> {
    static ALERT_MANAGER: OnceLock<Mutex<AlertManager>> = OnceLock::new();
    ALERT_MANAGER.get_or_init(|| Mutex::new(AlertManager::new()))
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
