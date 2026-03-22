//! Tauri commands and config merge for the Downloads folder organizer.

use crate::config::Config;
use crate::downloads_organizer;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadsOrganizerStatus {
    pub enabled: bool,
    pub interval: String,
    pub daily_at_local: String,
    pub path_raw: String,
    pub dry_run: bool,
    pub rules_path: String,
    pub rules_error: Option<String>,
    pub last_run_utc: Option<String>,
    pub last_dry_run: bool,
    pub moved: u32,
    pub skipped: u32,
    pub failed: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadsOrganizerSettingsPatch {
    pub enabled: Option<bool>,
    pub interval: Option<String>,
    #[serde(default)]
    pub daily_at_local: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    pub dry_run: Option<bool>,
}

fn read_config_value() -> Value {
    let path = Config::config_file_path();
    let Ok(content) = fs::read_to_string(&path) else {
        return json!({});
    };
    serde_json::from_str(&content).unwrap_or(json!({}))
}

fn write_config_value(v: &Value) -> Result<(), String> {
    let path = Config::config_file_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let pretty = serde_json::to_string_pretty(v).map_err(|e| e.to_string())?;
    fs::write(&path, pretty).map_err(|e| e.to_string())
}

fn write_rules_default_if_missing(path: &std::path::Path) {
    if path.exists() {
        return;
    }
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, Config::DEFAULT_DOWNLOADS_ORGANIZER_RULES);
}

#[tauri::command]
pub fn read_downloads_organizer_rules() -> Result<String, String> {
    let _ = Config::ensure_agents_directory();
    let path = Config::downloads_organizer_rules_path();
    write_rules_default_if_missing(&path);
    fs::read_to_string(&path).map_err(|e| format!("read {:?}: {}", path, e))
}

#[tauri::command]
pub fn save_downloads_organizer_rules(content: String) -> Result<(), String> {
    if let Err(e) = downloads_organizer::parse_rules_markdown(&content) {
        return Err(format!("Invalid rules: {}", e));
    }
    let _ = Config::ensure_agents_directory();
    let path = Config::downloads_organizer_rules_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&path, content).map_err(|e| e.to_string())
}

fn parse_hhmm_settings(raw: &str) -> Option<()> {
    let raw = raw.trim();
    let mut parts = raw.split(':');
    let h: u32 = parts.next()?.trim().parse().ok()?;
    let m: u32 = parts.next()?.trim().parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    if h < 24 && m < 60 {
        Some(())
    } else {
        None
    }
}

#[tauri::command]
pub fn get_downloads_organizer_status() -> DownloadsOrganizerStatus {
    let st = downloads_organizer::load_organizer_state();
    let v = read_config_value();
    let daily = v
        .get("downloadsOrganizerDailyAtLocal")
        .and_then(|x| x.as_str())
        .unwrap_or("09:00")
        .to_string();
    DownloadsOrganizerStatus {
        enabled: Config::downloads_organizer_enabled(),
        interval: Config::downloads_organizer_interval(),
        daily_at_local: daily,
        path_raw: Config::downloads_organizer_path_raw(),
        dry_run: Config::downloads_organizer_dry_run(),
        rules_path: Config::downloads_organizer_rules_path()
            .to_string_lossy()
            .to_string(),
        rules_error: st.rules_error.clone(),
        last_run_utc: st.last_run_utc.map(|t| t.to_rfc3339()),
        last_dry_run: st.last_dry_run,
        moved: st.moved,
        skipped: st.skipped,
        failed: st.failed,
    }
}

#[tauri::command]
pub fn set_downloads_organizer_settings(
    patch: DownloadsOrganizerSettingsPatch,
) -> Result<(), String> {
    let mut v = read_config_value();
    if let Some(b) = patch.enabled {
        v["downloadsOrganizerEnabled"] = json!(b);
    }
    if let Some(ref s) = patch.interval {
        let l = s.to_lowercase();
        if l != "hourly" && l != "daily" && l != "off" {
            return Err("interval must be hourly, daily, or off".to_string());
        }
        v["downloadsOrganizerInterval"] = json!(l);
    }
    if let Some(ref s) = patch.daily_at_local {
        let t = s.trim();
        if t.is_empty() {
            v["downloadsOrganizerDailyAtLocal"] = json!("09:00");
        } else if parse_hhmm_settings(t).is_none() {
            return Err("downloadsOrganizerDailyAtLocal must be HH:MM (24h)".to_string());
        } else {
            v["downloadsOrganizerDailyAtLocal"] = json!(t);
        }
    }
    if let Some(ref p) = patch.path {
        v["downloadsOrganizerPath"] = json!(p.trim());
    }
    if let Some(b) = patch.dry_run {
        v["downloadsOrganizerDryRun"] = json!(b);
    }
    write_config_value(&v)
}

/// Run one pass immediately (ignores hourly/daily schedule). Requires organizer enabled.
#[tauri::command]
pub fn run_downloads_organizer_now() -> Result<String, String> {
    if !Config::downloads_organizer_enabled() {
        return Err("Enable the Downloads organizer first (Settings, Downloads tab).".to_string());
    }
    downloads_organizer::run_organizer_pass();
    Ok("Organizer run finished. See ~/.mac-stats/debug.log and last run summary.".to_string())
}
