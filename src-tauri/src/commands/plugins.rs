//! Plugin Tauri commands

use crate::plugins::{Plugin, PluginManager, PluginResult};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::sync::OnceLock;
use std::path::PathBuf;

// Global plugin manager (in production, use proper state management)
fn get_plugin_manager() -> &'static Mutex<PluginManager> {
    static PLUGIN_MANAGER: OnceLock<Mutex<PluginManager>> = OnceLock::new();
    PLUGIN_MANAGER.get_or_init(|| Mutex::new(PluginManager::new()))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddPluginRequest {
    pub id: String,
    pub name: String,
    pub script_path: String,
    pub schedule_interval_secs: Option<u64>,
    pub timeout_secs: Option<u64>,
}

/// Add a plugin
#[tauri::command]
pub fn add_plugin(request: AddPluginRequest) -> Result<Plugin, String> {
    let mut plugin = Plugin::new(
        request.id.clone(),
        request.name,
        PathBuf::from(request.script_path),
    );

    if let Some(interval) = request.schedule_interval_secs {
        plugin.schedule_interval_secs = interval;
    }
    if let Some(timeout) = request.timeout_secs {
        plugin.timeout_secs = timeout;
    }

    plugin.validate()
        .map_err(|e| e.to_string())?;

    let plugin_clone = plugin.clone();
    
    get_plugin_manager().lock()
        .map_err(|e| e.to_string())?
        .add_plugin(plugin);

    Ok(plugin_clone)
}

/// Remove a plugin
#[tauri::command]
pub fn remove_plugin(plugin_id: String) -> Result<(), String> {
    get_plugin_manager().lock()
        .map_err(|e| e.to_string())?
        .remove_plugin(&plugin_id);

    Ok(())
}

/// Execute a plugin
#[tauri::command]
pub fn execute_plugin(plugin_id: String) -> Result<PluginResult, String> {
    let manager = get_plugin_manager().lock()
        .map_err(|e| e.to_string())?;

    let plugin = manager.get_plugin(&plugin_id)
        .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?
        .clone();

    plugin.execute()
        .map_err(|e| e.to_string())
}

/// List all plugins
#[tauri::command]
pub fn list_plugins() -> Result<Vec<Plugin>, String> {
    let manager = get_plugin_manager().lock()
        .map_err(|e| e.to_string())?;

    Ok(manager.list_plugins().into_iter().cloned().collect())
}

/// Run all due plugins
#[tauri::command]
pub fn run_due_plugins() -> Result<Vec<PluginResult>, String> {
    let mut manager = get_plugin_manager().lock()
        .map_err(|e| e.to_string())?;

    let results = manager.run_due_plugins();
    
    // Convert Results to Vec, filtering out errors
    let mut successful_results = Vec::new();
    for result in results {
        if let Ok(plugin_result) = result {
            successful_results.push(plugin_result);
        }
    }

    Ok(successful_results)
}
