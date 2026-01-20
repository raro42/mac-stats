//! Monitor Tauri commands

use crate::monitors::{website::WebsiteMonitor, social::MastodonMonitor, Monitor};
use crate::config::Config;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::fs;

// Global monitor storage (in production, use proper state management)
fn get_monitors() -> &'static Mutex<HashMap<String, Box<dyn crate::monitors::MonitorCheck + Send>>> {
    static MONITORS: OnceLock<Mutex<HashMap<String, Box<dyn crate::monitors::MonitorCheck + Send>>>> = OnceLock::new();
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
fn save_monitors() -> Result<(), String> {
    use tracing::{debug, info};
    
    Config::ensure_monitors_directory()
        .map_err(|e| format!("Failed to create monitors directory: {}", e))?;
    
    let _monitors = get_monitors().lock()
        .map_err(|e| format!("Failed to lock monitors: {}", e))?;
    let urls = get_monitor_urls().lock()
        .map_err(|e| format!("Failed to lock monitor URLs: {}", e))?;
    let stats = get_monitor_stats().lock()
        .map_err(|e| format!("Failed to lock monitor stats: {}", e))?;
    
    let mut persistent_monitors = Vec::new();
    
    // We need to extract data from trait objects
    // Since we can't directly serialize trait objects, we'll use the URLs map
    // and reconstruct from that. For now, we'll store website monitors only.
    // Get monitor configs
    let configs = get_monitor_configs().lock()
        .map_err(|e| format!("Failed to lock monitor configs: {}", e))?;
    
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
    
    fs::write(&monitors_path, json)
        .map_err(|e| format!("Failed to write monitors file: {}", e))?;
    
    info!("Monitor: Saved {} monitors to disk - Path: {:?}", persistent_monitors.len(), monitors_path);
    for pm in persistent_monitors {
        debug!("Monitor: Saved monitor - ID: {}, URL: {}, Last check: {:?}", 
               pm.id, pm.url, pm.last_check);
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
        debug!("Monitor: Monitors file does not exist at {:?}, starting with empty monitors", monitors_path);
        return Ok(());
    }
    
    info!("Monitor: Loading monitors from disk - Path: {:?}", monitors_path);
    
    let content = fs::read_to_string(&monitors_path)
        .map_err(|e| format!("Failed to read monitors file: {}", e))?;
    
    let file_data: MonitorsFile = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse monitors file: {}", e))?;
    
    let mut monitors = get_monitors().lock()
        .map_err(|e| format!("Failed to lock monitors: {}", e))?;
    let mut urls = get_monitor_urls().lock()
        .map_err(|e| format!("Failed to lock monitor URLs: {}", e))?;
    let mut configs = get_monitor_configs().lock()
        .map_err(|e| format!("Failed to lock monitor configs: {}", e))?;
    let mut stats = get_monitor_stats().lock()
        .map_err(|e| format!("Failed to lock monitor stats: {}", e))?;
    
    let mut loaded_count = 0;
    for pm in file_data.monitors {
        if pm.monitor_type == "Website" {
            let mut monitor = WebsiteMonitor::new(
                pm.id.clone(),
                pm.name.clone(),
                pm.url.clone(),
            );
            
            // Apply saved settings using the struct fields directly
            // Since WebsiteMonitor fields are public, we can set them
            monitor.timeout_secs = pm.timeout_secs;
            monitor.check_interval_secs = pm.check_interval_secs;
            monitor.verify_ssl = pm.verify_ssl;
            
            // Restore stats if available
            monitor.last_check = pm.last_check;
            monitor.last_status = pm.last_status.clone();
            
            let monitor_box: Box<dyn crate::monitors::MonitorCheck + Send> = Box::new(monitor);
            monitors.insert(pm.id.clone(), monitor_box);
            urls.insert(pm.id.clone(), pm.url.clone());
            
            // Store config without stats (stats stored separately)
            let mut config_without_stats = pm.clone();
            config_without_stats.last_check = None;
            config_without_stats.last_status = None;
            configs.insert(pm.id.clone(), config_without_stats);
            
            // Store stats separately
            stats.insert(pm.id.clone(), MonitorStats {
                last_check: pm.last_check,
                last_status: pm.last_status,
            });
            
            info!("Monitor: Loaded website monitor - ID: {}, Name: {}, URL: {}, Timeout: {}s, Interval: {}s, SSL: {}, Last check: {:?}", 
                  pm.id, pm.name, pm.url, pm.timeout_secs, pm.check_interval_secs, pm.verify_ssl, pm.last_check);
            loaded_count += 1;
        }
        // Add other monitor types here as needed
    }
    
    info!("Monitor: Successfully loaded {} monitors from disk", loaded_count);
    Ok(())
}

/// Add a website monitor
#[tauri::command]
pub fn add_website_monitor(request: AddWebsiteMonitorRequest) -> Result<Monitor, String> {
    use tracing::{debug, info};
    
    info!("Monitor: Adding website monitor - ID: {}, Name: {}, URL: {}", 
          request.id, request.name, request.url);
    
    let mut monitor = WebsiteMonitor::new(request.id.clone(), request.name.clone(), request.url.clone());
    
    if let Some(timeout) = request.timeout_secs {
        monitor.timeout_secs = timeout;
        debug!("Monitor: Set timeout to {} seconds", timeout);
    }
    if let Some(interval) = request.check_interval_secs {
        monitor.check_interval_secs = interval;
        debug!("Monitor: Set check interval to {} seconds", interval);
    }
    if let Some(verify) = request.verify_ssl {
        monitor.verify_ssl = verify;
        debug!("Monitor: SSL verification: {}", verify);
    }

    monitor.validate_url()
        .map_err(|e| {
            debug!("Monitor: URL validation failed for {}: {}", request.url, e);
            e.to_string()
        })?;

    let monitor_box: Box<dyn crate::monitors::MonitorCheck + Send> = Box::new(monitor.clone());
    
    get_monitors().lock()
        .map_err(|e| e.to_string())?
        .insert(request.id.clone(), monitor_box);
    
    // Store URL separately for retrieval
    get_monitor_urls().lock()
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
    get_monitor_configs().lock()
        .map_err(|e| e.to_string())?
        .insert(request.id.clone(), persistent_monitor);
    
    // Initialize stats storage
    get_monitor_stats().lock()
        .map_err(|e| e.to_string())?
        .insert(request.id.clone(), MonitorStats {
            last_check: None,
            last_status: None,
        });
    
    // Save to disk
    save_monitors()
        .map_err(|e| format!("Failed to save monitors: {}", e))?;

    info!("Monitor: Successfully added website monitor - ID: {}, URL: {}", 
          request.id, request.url);
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
    
    get_monitors().lock()
        .map_err(|e| e.to_string())?
        .insert(request.id.clone(), monitor_box);

    Ok(monitor.into())
}

/// Check a monitor
#[tauri::command]
pub fn check_monitor(monitor_id: String) -> Result<crate::monitors::MonitorStatus, String> {
    use tracing::{debug, info};
    use chrono::Utc;
    
    // Get monitor URL for logging
    let monitor_url = get_monitor_urls().lock()
        .ok()
        .and_then(|urls| urls.get(&monitor_id).cloned())
        .unwrap_or_else(|| "unknown".to_string());
    
    info!("Monitor: Checking monitor - ID: {}, URL: {}", monitor_id, monitor_url);
    
    let monitors = get_monitors().lock()
        .map_err(|e| e.to_string())?;

    let monitor = monitors.get(&monitor_id)
        .ok_or_else(|| {
            debug!("Monitor: Monitor not found - ID: {}", monitor_id);
            format!("Monitor not found: {}", monitor_id)
        })?;

    let start_time = std::time::Instant::now();
    let result = monitor.check()
        .map_err(|e| {
            debug!("Monitor: Check failed - ID: {}, URL: {}, Error: {}", monitor_id, monitor_url, e);
            e.to_string()
        })?;
    let duration = start_time.elapsed();
    
    if result.is_up {
        if let Some(response_time) = result.response_time_ms {
            info!("Monitor: Check successful - ID: {}, URL: {}, Status: UP, Response time: {}ms, Duration: {:?}", 
                  monitor_id, monitor_url, response_time, duration);
        } else {
            info!("Monitor: Check successful - ID: {}, URL: {}, Status: UP, Duration: {:?}", 
                  monitor_id, monitor_url, duration);
        }
    } else {
        let error_msg = result.error.as_deref().unwrap_or("unknown error");
        info!("Monitor: Check failed - ID: {}, URL: {}, Status: DOWN, Error: {}, Duration: {:?}", 
              monitor_id, monitor_url, error_msg, duration);
    }

    // Save stats after check
    let now = Utc::now();
    if let Ok(mut stats) = get_monitor_stats().lock() {
        stats.insert(monitor_id.clone(), MonitorStats {
            last_check: Some(now),
            last_status: Some(result.clone()),
        });
        debug!("Monitor: Saved stats for monitor - ID: {}, Last check: {:?}", monitor_id, now);
        
        // Save to disk (async, don't block on errors)
        if let Err(e) = save_monitors() {
            debug!("Monitor: Failed to save monitors after check - ID: {}, Error: {}", monitor_id, e);
        }
    }

    Ok(result)
}

/// List all monitors
#[tauri::command]
pub fn list_monitors() -> Result<Vec<String>, String> {
    let monitors = get_monitors().lock()
        .map_err(|e| e.to_string())?;

    Ok(monitors.keys().cloned().collect())
}

/// Remove a monitor
#[tauri::command]
pub fn remove_monitor(monitor_id: String) -> Result<(), String> {
    use tracing::{debug, info};
    
    // Get monitor URL for logging before removal
    let monitor_url = get_monitor_urls().lock()
        .ok()
        .and_then(|urls| urls.get(&monitor_id).cloned())
        .unwrap_or_else(|| "unknown".to_string());
    
    info!("Monitor: Removing monitor - ID: {}, URL: {}", monitor_id, monitor_url);
    
    get_monitors().lock()
        .map_err(|e| e.to_string())?
        .remove(&monitor_id)
        .ok_or_else(|| {
            debug!("Monitor: Monitor not found for removal - ID: {}", monitor_id);
            format!("Monitor not found: {}", monitor_id)
        })?;
    
    // Also remove URL from storage
    get_monitor_urls().lock()
        .map_err(|e| e.to_string())?
        .remove(&monitor_id);
    
    // Remove from configs
    get_monitor_configs().lock()
        .map_err(|e| e.to_string())?
        .remove(&monitor_id);
    
    // Remove from stats
    get_monitor_stats().lock()
        .map_err(|e| e.to_string())?
        .remove(&monitor_id);
    
    // Save to disk
    save_monitors()
        .map_err(|e| format!("Failed to save monitors: {}", e))?;

    info!("Monitor: Successfully removed monitor - ID: {}, URL: {}", monitor_id, monitor_url);
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonitorDetails {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
    pub monitor_type: String,
}

/// Get monitor details including URL
#[tauri::command]
pub fn get_monitor_details(monitor_id: String) -> Result<MonitorDetails, String> {
    let monitors = get_monitors().lock()
        .map_err(|e| e.to_string())?;

    let _monitor = monitors.get(&monitor_id)
        .ok_or_else(|| format!("Monitor not found: {}", monitor_id))?;
    
    // Get URL from separate storage
    let url = get_monitor_urls().lock()
        .map_err(|e| e.to_string())?
        .get(&monitor_id)
        .cloned();
    
    Ok(MonitorDetails {
        id: monitor_id.clone(),
        name: monitor_id, // We don't have name stored separately, use ID for now
        url,
        monitor_type: "Website".to_string(), // Assume website for now
    })
}

/// Get cached monitor status without performing a check
#[tauri::command]
pub fn get_monitor_status(monitor_id: String) -> Result<Option<crate::monitors::MonitorStatus>, String> {
    let stats = get_monitor_stats().lock()
        .map_err(|e| e.to_string())?;
    
    Ok(stats.get(&monitor_id)
        .and_then(|s| s.last_status.clone()))
}
