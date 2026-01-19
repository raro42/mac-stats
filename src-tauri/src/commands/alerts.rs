//! Alert Tauri commands

use crate::alerts::{Alert, AlertManager, AlertContext};
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
    get_alert_manager().lock()
        .map_err(|e| e.to_string())?
        .add_alert(alert);

    Ok(())
}

/// Remove an alert
#[tauri::command]
pub fn remove_alert(alert_id: String) -> Result<(), String> {
    get_alert_manager().lock()
        .map_err(|e| e.to_string())?
        .remove_alert(&alert_id);

    Ok(())
}

/// Evaluate alerts (called periodically or on events)
#[tauri::command]
pub fn evaluate_alerts(context: AlertContext) -> Result<Vec<String>, String> {
    get_alert_manager().lock()
        .map_err(|e| e.to_string())?
        .evaluate(context)
        .map_err(|e| e.to_string())
}
