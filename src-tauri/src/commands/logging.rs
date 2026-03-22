//! Logging Tauri commands for forwarding JavaScript console messages to Rust logs,
//! runtime verbosity control (e.g. from chat reserved words -v, -vv, -vvv),
//! and exposing the debug log path / opening the log file for the user (e.g. in Settings).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
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
    let source_prefix = log
        .source
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

/// Return the absolute path of the app debug log file (e.g. for display in Settings).
/// Used by the "View logs" feature so users can open or locate the Discord/app log.
#[tauri::command]
pub fn get_debug_log_path() -> Result<String, String> {
    let path = crate::config::Config::log_file_path();
    path.into_os_string()
        .into_string()
        .map_err(|_| "Invalid log path".to_string())
}

/// Open the app debug log file with the system default application (e.g. TextEdit on macOS).
/// On macOS uses `open path`; no-op or error on other platforms.
#[tauri::command]
pub fn open_debug_log() -> Result<(), String> {
    let path: PathBuf = crate::config::Config::log_file_path();
    #[cfg(target_os = "macos")]
    {
        let path_str = path
            .into_os_string()
            .into_string()
            .map_err(|_| "Invalid log path".to_string())?;
        std::process::Command::new("open")
            .arg(&path_str)
            .status()
            .map_err(|e| format!("Failed to open log file: {}", e))?;
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        Err("Open log file is supported only on macOS".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::set_chat_verbosity;
    use crate::logging::VERBOSITY;
    use std::sync::atomic::Ordering;
    use std::sync::Mutex;

    /// Serialize tests that mutate the global verbosity atomic.
    static VERBOSITY_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn set_chat_verbosity_updates_legacy_verbosity_atomic() {
        let _g = VERBOSITY_TEST_LOCK.lock().unwrap();
        let saved = VERBOSITY.load(Ordering::Relaxed);
        set_chat_verbosity(2).unwrap();
        assert_eq!(VERBOSITY.load(Ordering::Relaxed), 2);
        crate::logging::set_verbosity(saved);
    }

    #[test]
    fn set_chat_verbosity_clamps_above_three() {
        let _g = VERBOSITY_TEST_LOCK.lock().unwrap();
        let saved = VERBOSITY.load(Ordering::Relaxed);
        set_chat_verbosity(255).unwrap();
        assert_eq!(VERBOSITY.load(Ordering::Relaxed), 3);
        crate::logging::set_verbosity(saved);
    }
}
