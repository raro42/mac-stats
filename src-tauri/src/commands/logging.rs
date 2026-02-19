//! Logging Tauri commands for forwarding JavaScript console messages to Rust logs
//! and for runtime verbosity control (e.g. from chat reserved words -v, -vv, -vvv).

use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

#[derive(Debug, Serialize, Deserialize)]
pub struct LogMessage {
    pub level: String,
    pub message: String,
    pub source: Option<String>,
}

/// Log a message from JavaScript console
#[tauri::command]
pub fn log_from_js(log: LogMessage) -> Result<(), String> {
    let source_prefix = log.source
        .as_ref()
        .map(|s| format!("[{}] ", s))
        .unwrap_or_default();
    
    let full_message = format!("{}{}", source_prefix, log.message);
    
    match log.level.to_lowercase().as_str() {
        "error" => error!("JS: {}", full_message),
        "warn" => warn!("JS: {}", full_message),
        "info" => info!("JS: {}", full_message),
        "debug" => debug!("JS: {}", full_message),
        "log" => info!("JS: {}", full_message),
        _ => info!("JS: {}", full_message),
    }
    
    Ok(())
}

/// Set log verbosity from chat (reserved words -v, -vv, -vvv).
/// Level: 0 = error, 1 = warn (-v), 2 = debug (-vv), 3 = trace (-vvv).
#[tauri::command]
pub fn set_chat_verbosity(level: u8) -> Result<(), String> {
    let level = level.min(3);
    crate::logging::set_verbosity(level);
    Ok(())
}
