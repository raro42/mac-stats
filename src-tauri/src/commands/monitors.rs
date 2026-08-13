//! Monitor Tauri commands

use crate::config::Config;
use crate::monitors::{
    social::MastodonMonitor, website::WebsiteMonitor, Monitor, MonitorCheck,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;
use std::sync::OnceLock;

/// Stored `check_interval_secs` must be at least 1. Zero makes `elapsed >= interval` true even
/// when `elapsed` is 0 (same instant as `last_check`), so a monitor could be scheduled every
/// `run_due_monitor_checks` pass if anything invoked it more than once per wake.
const MIN_MONITOR_CHECK_INTERVAL_SECS: u64 = 1;

fn clamp_monitor_check_interval_secs(stored: u64) -> u64 {
    stored.max(MIN_MONITOR_CHECK_INTERVAL_SECS)
}

/// Minimum background gap while a monitor is already DOWN (reduces DNS thrash + log spam).
const DOWN_BACKOFF_SECS: u64 = 180;
/// Longer gap when the last failure was DNS (hostname usually stays broken for minutes).
const DNS_DOWN_BACKOFF_SECS: u64 = 300;
/// While outcome is unchanged, rewrite `monitors.json` at most this often (last_check checkpoint).
const STATS_DISK_CHECKPOINT_SECS: i64 = 300;

/// True when up/down or error text changed (ignore response-time jitter).
fn monitor_outcome_changed(
    prev: Option<&crate::monitors::MonitorStatus>,
    next: &crate::monitors::MonitorStatus,
) -> bool {
    match prev {
        None => true,
        Some(p) => {
            p.is_up != next.is_up || p.error.as_deref() != next.error.as_deref()
        }
    }
}

/// Persist after every outcome flip; otherwise checkpoint last_check about every 5 minutes.
fn should_persist_monitor_stats(
    prev: Option<&crate::monitors::MonitorStatus>,
    next: &crate::monitors::MonitorStatus,
    last_disk_persist: Option<chrono::DateTime<chrono::Utc>>,
    now: chrono::DateTime<chrono::Utc>,
) -> bool {
    if monitor_outcome_changed(prev, next) {
        return true;
    }
    match last_disk_persist {
        None => true,
        Some(t) => (now - t).num_seconds() >= STATS_DISK_CHECKPOINT_SECS,
    }
}

fn get_last_disk_persist() -> &'static Mutex<HashMap<String, chrono::DateTime<chrono::Utc>>> {
    static LAST_DISK: OnceLock<Mutex<HashMap<String, chrono::DateTime<chrono::Utc>>>> =
        OnceLock::new();
    LAST_DISK.get_or_init(|| Mutex::new(HashMap::new()))
}

fn mark_monitors_disk_persisted(ids: impl IntoIterator<Item = String>, at: chrono::DateTime<chrono::Utc>) {
    if let Ok(mut map) = get_last_disk_persist().lock() {
        for id in ids {
            map.insert(id, at);
        }
    }
}

/// Effective background interval: honor stored cadence when UP; stretch while DOWN.
fn background_interval_secs(
    stored_interval_secs: Option<u64>,
    last_status: Option<&crate::monitors::MonitorStatus>,
) -> u64 {
    let base = stored_interval_secs
        .map(clamp_monitor_check_interval_secs)
        .unwrap_or(60);
    match last_status {
        Some(s) if !s.is_up => {
            let err = s.error.as_deref().unwrap_or("");
            if err == "DNS lookup failed" {
                base.max(DNS_DOWN_BACKOFF_SECS)
            } else {
                base.max(DOWN_BACKOFF_SECS)
            }
        }
        _ => base,
    }
}

/// Whether `run_due_monitor_checks` should invoke `check_monitor` for this monitor.
/// `stored_interval_secs` is `None` when the monitor id has no config row (treat as 60s).
/// Manual `check_monitor` from the UI is unchanged (always runs when requested).
fn is_monitor_due_for_background(
    now: chrono::DateTime<chrono::Utc>,
    last_check: Option<chrono::DateTime<chrono::Utc>>,
    stored_interval_secs: Option<u64>,
    last_status: Option<&crate::monitors::MonitorStatus>,
) -> bool {
    let interval_secs = background_interval_secs(stored_interval_secs, last_status) as i64;
    match last_check {
        None => true,
        Some(t) => (now - t).num_seconds() >= interval_secs,
    }
}

// Global monitor storage (in production, use proper state management)
fn get_monitors() -> &'static Mutex<HashMap<String, Box<dyn crate::monitors::MonitorCheck + Send>>>
{
    static MONITORS: OnceLock<
        Mutex<HashMap<String, Box<dyn crate::monitors::MonitorCheck + Send>>>,
    > = OnceLock::new();
    MONITORS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddWebsiteMonitorRequest {
    pub id: String,
    pub name: String,
    pub url: String,
    pub timeout_secs: Option<u64>,
    pub check_interval_secs: Option<u64>,
    pub verify_ssl: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddMastodonMonitorRequest {
    pub id: String,
    pub name: String,
    pub instance_url: String,
    pub account_username: String,
}

// Store monitor URLs separately since Monitor struct doesn't include URL
fn get_monitor_urls() -> &'static Mutex<HashMap<String, String>> {
    static MONITOR_URLS: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    MONITOR_URLS.get_or_init(|| Mutex::new(HashMap::new()))
}

// Store monitor configurations for persistence
fn get_monitor_configs() -> &'static Mutex<HashMap<String, PersistentMonitor>> {
    static MONITOR_CONFIGS: OnceLock<Mutex<HashMap<String, PersistentMonitor>>> = OnceLock::new();
    MONITOR_CONFIGS.get_or_init(|| Mutex::new(HashMap::new()))
}

// Store monitor stats (last_check, last_status) separately
fn get_monitor_stats() -> &'static Mutex<HashMap<String, MonitorStats>> {
    static MONITOR_STATS: OnceLock<Mutex<HashMap<String, MonitorStats>>> = OnceLock::new();
    MONITOR_STATS.get_or_init(|| Mutex::new(HashMap::new()))
}

// Monitor stats for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MonitorStats {
    last_check: Option<chrono::DateTime<chrono::Utc>>,
    last_status: Option<crate::monitors::MonitorStatus>,
}

// Serializable monitor data for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistentMonitor {
    id: String,
    name: String,
    url: String,
    monitor_type: String, // "Website" or "Mastodon"
    timeout_secs: u64,
    check_interval_secs: u64,
    verify_ssl: bool,
    // For Mastodon monitors
    instance_url: Option<String>,
    account_username: Option<String>,
    // Monitor stats
    last_check: Option<chrono::DateTime<chrono::Utc>>,
    last_status: Option<crate::monitors::MonitorStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MonitorsFile {
    monitors: Vec<PersistentMonitor>,
}

/// Save monitors to disk
/// Uses try_lock to avoid blocking if locks are busy (non-blocking)
fn save_monitors() -> Result<(), String> {
    use tracing::debug;

    Config::ensure_monitors_directory()
        .map_err(|e| format!("Failed to create monitors directory: {}", e))?;

    // Use try_lock to avoid blocking - if locks are busy, skip this save
    let _monitors = get_monitors()
        .try_lock()
        .map_err(|_| "Failed to lock monitors (busy)")?;
    let urls = get_monitor_urls()
        .try_lock()
        .map_err(|_| "Failed to lock monitor URLs (busy)")?;
    let stats = get_monitor_stats()
        .try_lock()
        .map_err(|_| "Failed to lock monitor stats (busy)")?;

    let mut persistent_monitors = Vec::new();

    // We need to extract data from trait objects
    // Since we can't directly serialize trait objects, we'll use the URLs map
    // and reconstruct from that. For now, we'll store website monitors only.
    // Get monitor configs
    let configs = get_monitor_configs()
        .try_lock()
        .map_err(|_| "Failed to lock monitor configs (busy)")?;

    for (id, url) in urls.iter() {
        // Try to get saved config, or use defaults
        let mut pm = if let Some(config) = configs.get(id) {
            config.clone()
        } else {
            // Fallback: create from URL with defaults
            PersistentMonitor {
                id: id.clone(),
                name: id.clone(),
                url: url.clone(),
                monitor_type: "Website".to_string(),
                timeout_secs: 10,
                check_interval_secs: 30,
                verify_ssl: true,
                instance_url: None,
                account_username: None,
                last_check: None,
                last_status: None,
            }
        };

        // Update stats from separate storage
        if let Some(monitor_stats) = stats.get(id) {
            pm.last_check = monitor_stats.last_check;
            pm.last_status = monitor_stats.last_status.clone();
        }

        persistent_monitors.push(pm);
    }

    let file_data = MonitorsFile {
        monitors: persistent_monitors.clone(),
    };

    let monitors_path = Config::monitors_file_path();
    let json = serde_json::to_string_pretty(&file_data)
        .map_err(|e| format!("Failed to serialize monitors: {}", e))?;

    crate::config::write_text_atomic(&monitors_path, &json)
        .map_err(|e| format!("Failed to write monitors file: {}", e))?;

    let persisted_at = chrono::Utc::now();
    mark_monitors_disk_persisted(
        persistent_monitors.iter().map(|pm| pm.id.clone()),
        persisted_at,
    );

    // Routine checkpoints are common; keep INFO for structural add/remove paths only.
    debug!(
        "Monitor: Saved {} monitors to disk - Path: {:?}",
        persistent_monitors.len(),
        monitors_path
    );
    for pm in persistent_monitors {
        debug!(
            "Monitor: Saved monitor - ID: {}, URL: {}, Last check: {:?}",
            pm.id, pm.url, pm.last_check
        );
    }

    Ok(())
}

/// Load monitors from disk (public for use in setup)
pub fn load_monitors_internal() -> Result<(), String> {
    load_monitors()
}

/// Load monitors from disk
fn load_monitors() -> Result<(), String> {
    use tracing::{debug, info};

    let monitors_path = Config::monitors_file_path();

    // If file doesn't exist, that's okay - just return empty
    if !monitors_path.exists() {
        debug!(
            "Monitor: Monitors file does not exist at {:?}, starting with empty monitors",
            monitors_path
        );
        return Ok(());
    }

    info!(
        "Monitor: Loading monitors from disk - Path: {:?}",
        monitors_path
    );

    let content = fs::read_to_string(&monitors_path)
        .map_err(|e| format!("Failed to read monitors file: {}", e))?;

    let file_data: MonitorsFile = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse monitors file: {}", e))?;

    let mut monitors = get_monitors()
        .lock()
        .map_err(|e| format!("Failed to lock monitors: {}", e))?;
    let mut urls = get_monitor_urls()
        .lock()
        .map_err(|e| format!("Failed to lock monitor URLs: {}", e))?;
    let mut configs = get_monitor_configs()
        .lock()
        .map_err(|e| format!("Failed to lock monitor configs: {}", e))?;
    let mut stats = get_monitor_stats()
        .lock()
        .map_err(|e| format!("Failed to lock monitor stats: {}", e))?;

    let mut loaded_count = 0;
    for pm in file_data.monitors {
        if pm.monitor_type == "Website" {
            let mut monitor = WebsiteMonitor::new(pm.id.clone(), pm.name.clone(), pm.url.clone());

            // Apply saved settings using the struct fields directly
            // Since WebsiteMonitor fields are public, we can set them
            let check_interval_secs = clamp_monitor_check_interval_secs(pm.check_interval_secs);
            monitor.timeout_secs = pm.timeout_secs;
            monitor.check_interval_secs = check_interval_secs;
            monitor.verify_ssl = pm.verify_ssl;

            // Restore stats if available
            monitor.last_check = pm.last_check;
            monitor.last_status = pm.last_status.clone();

            let monitor_box: Box<dyn crate::monitors::MonitorCheck + Send> = Box::new(monitor);
            monitors.insert(pm.id.clone(), monitor_box);
            urls.insert(pm.id.clone(), pm.url.clone());

            // Store config without stats (stats stored separately)
            let mut config_without_stats = pm.clone();
            config_without_stats.check_interval_secs = check_interval_secs;
            config_without_stats.last_check = None;
            config_without_stats.last_status = None;
            configs.insert(pm.id.clone(), config_without_stats);

            // Store stats separately
            stats.insert(
                pm.id.clone(),
                MonitorStats {
                    last_check: pm.last_check,
                    last_status: pm.last_status,
                },
            );

            info!("Monitor: Loaded website monitor - ID: {}, Name: {}, URL: {}, Timeout: {}s, Interval: {}s, SSL: {}, Last check: {:?}", 
                  pm.id, pm.name, pm.url, pm.timeout_secs, check_interval_secs, pm.verify_ssl, pm.last_check);
            loaded_count += 1;
        }
        // Add other monitor types here as needed
    }

    info!(
        "Monitor: Successfully loaded {} monitors from disk",
        loaded_count
    );
    Ok(())
}

/// Run checks for all monitors that are due (last_check + check_interval_secs <= now).
/// Called from the background monitor thread in `lib.rs` (30s sleep loop) so website monitoring runs
/// even when no window is open.
pub fn run_due_monitor_checks() {
    use chrono::Utc;
    use tracing::debug;

    let configs = match get_monitor_configs().try_lock() {
        Ok(c) => c,
        Err(_) => {
            debug!("Monitor: skip run_due (configs lock busy)");
            return;
        }
    };
    let stats = match get_monitor_stats().try_lock() {
        Ok(s) => s,
        Err(_) => {
            debug!("Monitor: skip run_due (stats lock busy)");
            return;
        }
    };
    let now = Utc::now();
    let due_ids: Vec<String> = configs
        .keys()
        .filter(|id| {
            let stored = configs.get(*id).map(|c| c.check_interval_secs);
            let st = stats.get(*id);
            let last = st.and_then(|s| s.last_check);
            let last_status = st.and_then(|s| s.last_status.as_ref());
            is_monitor_due_for_background(now, last, stored, last_status)
        })
        .cloned()
        .collect();
    drop(stats);
    drop(configs);

    for id in due_ids {
        if let Err(e) = check_monitor(id.clone()) {
            debug!("Monitor: background check failed for {}: {}", id, e);
        }
    }
}

/// Snapshot of (monitor_id, status) for each monitor that has a last_status.
/// Used by periodic alert evaluation to run SiteDown and similar rules per monitor.
pub fn get_monitor_statuses_snapshot() -> Vec<(String, crate::monitors::MonitorStatus)> {
    let stats = match get_monitor_stats().try_lock() {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    stats
        .iter()
        .filter_map(|(id, s)| {
            s.last_status
                .as_ref()
                .cloned()
                .map(|status| (id.clone(), status))
        })
        .collect()
}

/// Add a website monitor
#[tauri::command]
pub fn add_website_monitor(request: AddWebsiteMonitorRequest) -> Result<Monitor, String> {
    use tracing::{debug, info};

    info!(
        "Monitor: Adding website monitor - ID: {}, Name: {}, URL: {}",
        request.id, request.name, request.url
    );

    let mut monitor = WebsiteMonitor::new(
        request.id.clone(),
        request.name.clone(),
        request.url.clone(),
    );

    if let Some(timeout) = request.timeout_secs {
        monitor.timeout_secs = timeout;
        debug!("Monitor: Set timeout to {} seconds", timeout);
    }
    if let Some(interval) = request.check_interval_secs {
        let interval = clamp_monitor_check_interval_secs(interval);
        monitor.check_interval_secs = interval;
        debug!("Monitor: Set check interval to {} seconds", interval);
    }
    if let Some(verify) = request.verify_ssl {
        monitor.verify_ssl = verify;
        debug!("Monitor: SSL verification: {}", verify);
    }

    monitor.validate_url().map_err(|e| {
        debug!("Monitor: URL validation failed for {}: {}", request.url, e);
        e.to_string()
    })?;

    let monitor_box: Box<dyn crate::monitors::MonitorCheck + Send> = Box::new(monitor.clone());

    get_monitors()
        .lock()
        .map_err(|e| e.to_string())?
        .insert(request.id.clone(), monitor_box);

    // Store URL separately for retrieval
    get_monitor_urls()
        .lock()
        .map_err(|e| e.to_string())?
        .insert(request.id.clone(), request.url.clone());

    // Store monitor config for persistence
    let persistent_monitor = PersistentMonitor {
        id: request.id.clone(),
        name: request.name.clone(),
        url: request.url.clone(),
        monitor_type: "Website".to_string(),
        timeout_secs: monitor.timeout_secs,
        check_interval_secs: monitor.check_interval_secs,
        verify_ssl: monitor.verify_ssl,
        instance_url: None,
        account_username: None,
        last_check: None,
        last_status: None,
    };
    get_monitor_configs()
        .lock()
        .map_err(|e| e.to_string())?
        .insert(request.id.clone(), persistent_monitor);

    // Initialize stats storage
    get_monitor_stats()
        .lock()
        .map_err(|e| e.to_string())?
        .insert(
            request.id.clone(),
            MonitorStats {
                last_check: None,
                last_status: None,
            },
        );

    // Save to disk
    save_monitors().map_err(|e| format!("Failed to save monitors: {}", e))?;

    info!(
        "Monitor: Successfully added website monitor - ID: {}, URL: {}",
        request.id, request.url
    );
    Ok(monitor.into())
}

/// Add a Mastodon monitor
#[tauri::command]
pub fn add_mastodon_monitor(request: AddMastodonMonitorRequest) -> Result<Monitor, String> {
    let monitor = MastodonMonitor::new(
        request.id.clone(),
        request.name,
        request.instance_url,
        request.account_username,
    );

    let monitor_box: Box<dyn crate::monitors::MonitorCheck + Send> = Box::new(monitor.clone());

    get_monitors()
        .lock()
        .map_err(|e| e.to_string())?
        .insert(request.id.clone(), monitor_box);

    Ok(monitor.into())
}

/// Check a monitor
#[tauri::command]
pub fn check_monitor(monitor_id: String) -> Result<crate::monitors::MonitorStatus, String> {
    use chrono::Utc;
    use tracing::{debug, trace};

    // Snapshot config under a short lock, then HTTP outside the lock.
    // Holding `get_monitors()` during reqwest blocked `remove_monitor` for the full
    // timeout (e.g. 10s on a dead host), so Remove looked like a no-op.
    let (monitor_url, website) = {
        let configs = get_monitor_configs()
            .lock()
            .map_err(|e| e.to_string())?;
        let config = configs.get(&monitor_id).ok_or_else(|| {
            debug!("Monitor: Monitor not found - ID: {}", monitor_id);
            format!("Monitor not found: {}", monitor_id)
        })?;
        let mut wm = crate::monitors::website::WebsiteMonitor::new(
            config.id.clone(),
            config.name.clone(),
            config.url.clone(),
        );
        wm.timeout_secs = config.timeout_secs;
        wm.check_interval_secs = config.check_interval_secs;
        wm.verify_ssl = config.verify_ssl;
        (config.url.clone(), wm)
    };

    trace!(
        "Monitor: Checking monitor - ID: {}, URL: {}",
        monitor_id,
        monitor_url
    );

    let result = website.check().map_err(|e| {
        debug!(
            "Monitor: Check failed - ID: {}, URL: {}, Error: {}",
            monitor_id, monitor_url, e
        );
        e.to_string()
    })?;

    let now = Utc::now();
    let (prev_status, last_disk) = {
        let prev_status = get_monitor_stats()
            .lock()
            .ok()
            .and_then(|stats| stats.get(&monitor_id).and_then(|s| s.last_status.clone()));
        let last_disk = get_last_disk_persist()
            .lock()
            .ok()
            .and_then(|m| m.get(&monitor_id).copied());
        (prev_status, last_disk)
    };

    // DEBUG on first check or up/down/error flip; unchanged rechecks stay at TRACE
    // (pairs with disk persist throttle — chronic DNS hosts no longer flood debug.log).
    let outcome_changed = monitor_outcome_changed(prev_status.as_ref(), &result);
    if result.is_up {
        let ms = result.response_time_ms.unwrap_or(0);
        if outcome_changed {
            debug!("Monitor: {} UP {}ms", monitor_url, ms);
        } else {
            trace!("Monitor: {} UP {}ms (unchanged)", monitor_url, ms);
        }
    } else {
        let err = result.error.as_deref().unwrap_or("error");
        let ms = result.response_time_ms.unwrap_or(0);
        if outcome_changed {
            debug!("Monitor: {} DOWN {}ms {}", monitor_url, ms, err);
        } else {
            trace!(
                "Monitor: {} DOWN {}ms {} (unchanged)",
                monitor_url,
                ms,
                err
            );
        }
    }
    {
        if let Ok(mut stats) = get_monitor_stats().lock() {
            stats.insert(
                monitor_id.clone(),
                MonitorStats {
                    last_check: Some(now),
                    last_status: Some(result.clone()),
                },
            );
            trace!(
                "Monitor: Saved stats in memory for monitor - ID: {}, Last check: {:?}",
                monitor_id,
                now
            );
        }
    }

    // Persist on outcome change, else ~5 min last_check checkpoint (cuts SSD + log thrash).
    // `save_monitors` uses try_lock on each map — skip quietly if busy.
    if should_persist_monitor_stats(prev_status.as_ref(), &result, last_disk, now) {
        let _ = save_monitors();
    } else {
        trace!(
            "Monitor: Skip disk persist (unchanged outcome) - ID: {}",
            monitor_id
        );
    }

    Ok(result)
}

/// List all monitors
#[tauri::command]
pub fn list_monitors() -> Result<Vec<String>, String> {
    let monitors = get_monitors().lock().map_err(|e| e.to_string())?;

    Ok(monitors.keys().cloned().collect())
}

/// List all monitors with details (id, name, url, type) for Settings UI.
#[tauri::command]
pub fn list_monitors_with_details() -> Result<Vec<MonitorDetails>, String> {
    let configs = get_monitor_configs().lock().map_err(|e| e.to_string())?;

    Ok(configs
        .values()
        .map(|pm| MonitorDetails {
            id: pm.id.clone(),
            name: pm.name.clone(),
            url: Some(pm.url.clone()),
            monitor_type: pm.monitor_type.clone(),
        })
        .collect())
}

/// Remove a monitor
#[tauri::command]
pub fn remove_monitor(monitor_id: String) -> Result<(), String> {
    use tracing::{debug, info, warn};

    // Get monitor URL for logging before removal
    let monitor_url = get_monitor_urls()
        .lock()
        .ok()
        .and_then(|urls| urls.get(&monitor_id).cloned())
        .unwrap_or_else(|| "unknown".to_string());

    info!(
        "Monitor: Removing monitor - ID: {}, URL: {}",
        monitor_id, monitor_url
    );

    // Remove from every map even if one entry is already gone (UI retry / partial state).
    let mut found = false;
    if let Ok(mut monitors) = get_monitors().lock() {
        found |= monitors.remove(&monitor_id).is_some();
    } else {
        return Err("Failed to lock monitors".to_string());
    }
    if let Ok(mut urls) = get_monitor_urls().lock() {
        found |= urls.remove(&monitor_id).is_some();
    }
    if let Ok(mut configs) = get_monitor_configs().lock() {
        found |= configs.remove(&monitor_id).is_some();
    }
    if let Ok(mut stats) = get_monitor_stats().lock() {
        found |= stats.remove(&monitor_id).is_some();
    }

    if !found {
        debug!(
            "Monitor: Monitor not found for removal - ID: {}",
            monitor_id
        );
        return Err(format!("Monitor not found: {}", monitor_id));
    }

    // Retry save: concurrent check_monitor may briefly hold try_locks.
    let mut last_err = None;
    for attempt in 1..=5 {
        match save_monitors() {
            Ok(()) => {
                last_err = None;
                break;
            }
            Err(e) => {
                warn!(
                    "Monitor: save after remove attempt {} failed: {}",
                    attempt, e
                );
                last_err = Some(e);
                std::thread::sleep(std::time::Duration::from_millis(50 * attempt));
            }
        }
    }
    if let Some(e) = last_err {
        return Err(format!("Failed to save monitors: {}", e));
    }

    info!(
        "Monitor: Successfully removed monitor - ID: {}, URL: {}",
        monitor_id, monitor_url
    );
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonitorDetails {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
    pub monitor_type: String,
}

/// Get monitor details including URL and name from config.
#[tauri::command]
pub fn get_monitor_details(monitor_id: String) -> Result<MonitorDetails, String> {
    let monitors = get_monitors().lock().map_err(|e| e.to_string())?;

    let _monitor = monitors
        .get(&monitor_id)
        .ok_or_else(|| format!("Monitor not found: {}", monitor_id))?;

    let configs = get_monitor_configs().lock().map_err(|e| e.to_string())?;
    let name = configs
        .get(&monitor_id)
        .map(|pm| pm.name.clone())
        .unwrap_or_else(|| monitor_id.clone());
    let url = get_monitor_urls()
        .lock()
        .map_err(|e| e.to_string())?
        .get(&monitor_id)
        .cloned();
    let monitor_type = configs
        .get(&monitor_id)
        .map(|pm| pm.monitor_type.clone())
        .unwrap_or_else(|| "Website".to_string());

    Ok(MonitorDetails {
        id: monitor_id.clone(),
        name,
        url,
        monitor_type,
    })
}

/// Attach DOWN backoff timing so the UI can show “next auto-check”.
fn enrich_status_with_backoff(
    mut status: crate::monitors::MonitorStatus,
    last_check: Option<chrono::DateTime<chrono::Utc>>,
    stored_interval_secs: Option<u64>,
) -> crate::monitors::MonitorStatus {
    if status.is_up {
        return status;
    }
    let interval = background_interval_secs(stored_interval_secs, Some(&status));
    let anchor = last_check.unwrap_or(status.checked_at);
    let elapsed = (chrono::Utc::now() - anchor).num_seconds().max(0) as u64;
    let next = interval.saturating_sub(elapsed);
    status.extra.insert(
        "background_interval_secs".into(),
        serde_json::json!(interval),
    );
    status.extra.insert(
        "next_background_check_secs".into(),
        serde_json::json!(next),
    );
    status
}

/// Get cached monitor status without performing a check
#[tauri::command]
pub fn get_monitor_status(
    monitor_id: String,
) -> Result<Option<crate::monitors::MonitorStatus>, String> {
    let stats = get_monitor_stats().lock().map_err(|e| e.to_string())?;
    let configs = get_monitor_configs().lock().map_err(|e| e.to_string())?;
    let Some(st) = stats.get(&monitor_id) else {
        return Ok(None);
    };
    let Some(status) = st.last_status.clone() else {
        return Ok(None);
    };
    let stored = configs.get(&monitor_id).map(|c| c.check_interval_secs);
    Ok(Some(enrich_status_with_backoff(
        status,
        st.last_check,
        stored,
    )))
}

#[cfg(test)]
mod monitor_interval_tests {
    use super::{
        background_interval_secs, clamp_monitor_check_interval_secs, enrich_status_with_backoff,
        is_monitor_due_for_background, monitor_outcome_changed, should_persist_monitor_stats,
        DNS_DOWN_BACKOFF_SECS, DOWN_BACKOFF_SECS, STATS_DISK_CHECKPOINT_SECS,
    };
    use chrono::{TimeZone, Utc};
    use crate::monitors::MonitorStatus;

    fn down_status(error: &str) -> MonitorStatus {
        MonitorStatus {
            is_up: false,
            response_time_ms: Some(5),
            error: Some(error.to_string()),
            checked_at: Utc.with_ymd_and_hms(2026, 3, 22, 12, 0, 0).unwrap(),
            extra: Default::default(),
        }
    }

    fn up_status() -> MonitorStatus {
        MonitorStatus {
            is_up: true,
            response_time_ms: Some(40),
            error: None,
            checked_at: Utc.with_ymd_and_hms(2026, 3, 22, 12, 0, 0).unwrap(),
            extra: Default::default(),
        }
    }

    #[test]
    fn clamp_zero_becomes_one() {
        assert_eq!(clamp_monitor_check_interval_secs(0), 1);
    }

    #[test]
    fn clamp_one_unchanged() {
        assert_eq!(clamp_monitor_check_interval_secs(1), 1);
    }

    #[test]
    fn clamp_large_unchanged() {
        assert_eq!(clamp_monitor_check_interval_secs(60), 60);
    }

    #[test]
    fn background_interval_up_uses_stored() {
        assert_eq!(
            background_interval_secs(Some(60), Some(&up_status())),
            60
        );
        assert_eq!(background_interval_secs(Some(60), None), 60);
    }

    #[test]
    fn background_interval_dns_down_stretches() {
        let st = down_status("DNS lookup failed");
        assert_eq!(
            background_interval_secs(Some(60), Some(&st)),
            DNS_DOWN_BACKOFF_SECS
        );
    }

    #[test]
    fn background_interval_other_down_stretches() {
        let st = down_status("Connection timed out");
        assert_eq!(
            background_interval_secs(Some(60), Some(&st)),
            DOWN_BACKOFF_SECS
        );
    }

    #[test]
    fn due_never_checked() {
        let now = Utc.with_ymd_and_hms(2026, 3, 22, 12, 0, 0).unwrap();
        assert!(is_monitor_due_for_background(now, None, Some(60), None));
    }

    #[test]
    fn due_elapsed_equals_interval() {
        let now = Utc.with_ymd_and_hms(2026, 3, 22, 12, 1, 0).unwrap();
        let last = Utc.with_ymd_and_hms(2026, 3, 22, 12, 0, 0).unwrap();
        assert!(is_monitor_due_for_background(
            now,
            Some(last),
            Some(60),
            Some(&up_status())
        ));
    }

    #[test]
    fn not_due_elapsed_below_interval() {
        let now = Utc.with_ymd_and_hms(2026, 3, 22, 12, 0, 30).unwrap();
        let last = Utc.with_ymd_and_hms(2026, 3, 22, 12, 0, 0).unwrap();
        assert!(!is_monitor_due_for_background(
            now,
            Some(last),
            Some(60),
            Some(&up_status())
        ));
    }

    #[test]
    fn dns_down_not_due_at_sixty_seconds() {
        let now = Utc.with_ymd_and_hms(2026, 3, 22, 12, 1, 0).unwrap();
        let last = Utc.with_ymd_and_hms(2026, 3, 22, 12, 0, 0).unwrap();
        let st = down_status("DNS lookup failed");
        assert!(!is_monitor_due_for_background(
            now,
            Some(last),
            Some(60),
            Some(&st)
        ));
        let later = Utc.with_ymd_and_hms(2026, 3, 22, 12, 5, 0).unwrap();
        assert!(is_monitor_due_for_background(
            later,
            Some(last),
            Some(60),
            Some(&st)
        ));
    }

    #[test]
    fn missing_config_interval_defaults_sixty() {
        let now = Utc.with_ymd_and_hms(2026, 3, 22, 12, 0, 30).unwrap();
        let last = Utc.with_ymd_and_hms(2026, 3, 22, 12, 0, 0).unwrap();
        assert!(!is_monitor_due_for_background(now, Some(last), None, None));
        let now2 = Utc.with_ymd_and_hms(2026, 3, 22, 12, 1, 0).unwrap();
        assert!(is_monitor_due_for_background(now2, Some(last), None, None));
    }

    #[test]
    fn enrich_down_status_sets_next_check_extra() {
        let st = down_status("DNS lookup failed");
        let last = Utc.with_ymd_and_hms(2026, 3, 22, 12, 0, 0).unwrap();
        // Freeze-ish: enrich uses Utc::now(); just assert keys + interval value.
        let enriched = enrich_status_with_backoff(st, Some(last), Some(60));
        assert_eq!(
            enriched
                .extra
                .get("background_interval_secs")
                .and_then(|v| v.as_u64()),
            Some(DNS_DOWN_BACKOFF_SECS)
        );
        assert!(enriched.extra.contains_key("next_background_check_secs"));
        let up = enrich_status_with_backoff(up_status(), Some(last), Some(60));
        assert!(!up.extra.contains_key("background_interval_secs"));
    }

    #[test]
    fn stored_interval_zero_clamps_so_not_due_same_second() {
        let now = Utc.with_ymd_and_hms(2026, 3, 22, 12, 0, 0).unwrap();
        let last = Utc.with_ymd_and_hms(2026, 3, 22, 12, 0, 0).unwrap();
        assert!(!is_monitor_due_for_background(
            now,
            Some(last),
            Some(0),
            None
        ));
    }

    /// docs/022_feature_review_plan.md §F10: guard against removing the background wake loop.
    #[test]
    fn lib_rs_invokes_run_due_monitor_checks_in_background_loop() {
        let lib_rs = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"));
        assert!(
            lib_rs.contains("run_due_monitor_checks()"),
            "lib.rs should call run_due_monitor_checks from the 30s monitor thread"
        );
    }

    #[test]
    fn outcome_ignores_response_time_jitter() {
        let mut a = up_status();
        let mut b = up_status();
        a.response_time_ms = Some(10);
        b.response_time_ms = Some(99);
        assert!(!monitor_outcome_changed(Some(&a), &b));
    }

    #[test]
    fn outcome_detects_up_to_down() {
        assert!(monitor_outcome_changed(Some(&up_status()), &down_status("DNS lookup failed")));
    }

    #[test]
    fn outcome_same_down_reason_unchanged() {
        // Unchanged DOWN rechecks must not DEBUG (check_monitor uses this gate).
        let a = down_status("DNS lookup failed");
        let mut b = down_status("DNS lookup failed");
        b.response_time_ms = Some(50);
        assert!(!monitor_outcome_changed(Some(&a), &b));
        assert!(monitor_outcome_changed(None, &a));
    }

    #[test]
    fn persist_on_first_check_and_outcome_change() {
        let now = Utc.with_ymd_and_hms(2026, 3, 22, 12, 0, 0).unwrap();
        assert!(should_persist_monitor_stats(None, &up_status(), None, now));
        let recent = Utc.with_ymd_and_hms(2026, 3, 22, 11, 59, 0).unwrap();
        assert!(should_persist_monitor_stats(
            Some(&up_status()),
            &down_status("DNS lookup failed"),
            Some(recent),
            now
        ));
    }

    #[test]
    fn skip_persist_when_unchanged_and_checkpoint_fresh() {
        let now = Utc.with_ymd_and_hms(2026, 3, 22, 12, 1, 0).unwrap();
        let recent = Utc.with_ymd_and_hms(2026, 3, 22, 12, 0, 0).unwrap();
        assert!(!should_persist_monitor_stats(
            Some(&up_status()),
            &up_status(),
            Some(recent),
            now
        ));
    }

    #[test]
    fn persist_checkpoint_after_five_minutes() {
        let now = Utc.with_ymd_and_hms(2026, 3, 22, 12, 5, 0).unwrap();
        let older = Utc.with_ymd_and_hms(2026, 3, 22, 12, 0, 0).unwrap();
        assert_eq!(STATS_DISK_CHECKPOINT_SECS, 300);
        assert!(should_persist_monitor_stats(
            Some(&up_status()),
            &up_status(),
            Some(older),
            now
        ));
    }
}
