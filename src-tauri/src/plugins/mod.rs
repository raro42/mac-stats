//! Plugin system module
//! 
//! Script-based plugins that output JSON.
//! Plugins are executable scripts (bash/python) that follow a contract.

use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use std::process::{Command, Stdio};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub script_path: PathBuf,
    pub enabled: bool,
    pub schedule_interval_secs: u64,
    pub timeout_secs: u64,
    pub last_run: Option<DateTime<Utc>>,
    pub last_output: Option<PluginOutput>,
}

/// Plugin output (JSON schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginOutput {
    pub status: String, // "ok", "error", "warning"
    pub message: Option<String>,
    pub data: Option<serde_json::Value>,
    pub metrics: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

/// Plugin execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResult {
    pub plugin_id: String,
    pub output: PluginOutput,
    pub execution_time_ms: u64,
}

impl Plugin {
    pub fn new(id: String, name: String, script_path: PathBuf) -> Self {
        Self {
            id,
            name,
            script_path,
            enabled: true,
            schedule_interval_secs: 300, // 5 minutes default
            timeout_secs: 30,
            last_run: None,
            last_output: None,
        }
    }

    /// Execute plugin script and parse JSON output
    pub fn execute(&self) -> Result<PluginResult> {
        let start_time = std::time::Instant::now();

        // Check if script exists and is executable
        if !self.script_path.exists() {
            return Err(anyhow::anyhow!("Plugin script not found: {:?}", self.script_path));
        }

        // Determine interpreter based on file extension
        let (interpreter, _args): (&str, Vec<&str>) = if self.script_path.extension()
            .and_then(|s| s.to_str())
            .map(|s| s == "py")
            .unwrap_or(false) {
            ("python3", vec![])
        } else if self.script_path.extension()
            .and_then(|s| s.to_str())
            .map(|s| s == "sh" || s == "bash")
            .unwrap_or(false) {
            ("bash", vec![])
        } else {
            // Try to execute directly (might be a binary)
            ("", vec![])
        };

        // Build command
        let mut cmd = if interpreter.is_empty() {
            Command::new(&self.script_path)
        } else {
            let mut c = Command::new(interpreter);
            c.arg(&self.script_path);
            c
        };

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Execute with timeout using spawn and wait with timeout
        // Note: std::process::Command doesn't have timeout, so we'll handle it differently
        // For now, just execute without timeout (can be improved with tokio or crossbeam)
        let output = cmd.output()
            .context("Failed to execute plugin script")?;

        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        // Parse JSON output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let plugin_output: PluginOutput = if output.status.success() {
            serde_json::from_str(&stdout)
                .context("Failed to parse plugin JSON output")?
        } else {
            // Script failed - create error output
            let stderr = String::from_utf8_lossy(&output.stderr);
            PluginOutput {
                status: "error".to_string(),
                message: Some(format!("Script execution failed: {}", stderr)),
                data: None,
                metrics: None,
                timestamp: Utc::now(),
            }
        };

        Ok(PluginResult {
            plugin_id: self.id.clone(),
            output: plugin_output,
            execution_time_ms,
        })
    }

    /// Validate plugin script
    pub fn validate(&self) -> Result<()> {
        if !self.script_path.exists() {
            return Err(anyhow::anyhow!("Plugin script not found"));
        }

        // Try a dry run to validate JSON output format
        // For now, just check if file exists and is readable
        Ok(())
    }
}

/// Plugin manager
pub struct PluginManager {
    plugins: std::collections::HashMap<String, Plugin>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: std::collections::HashMap::new(),
        }
    }

    pub fn add_plugin(&mut self, plugin: Plugin) {
        self.plugins.insert(plugin.id.clone(), plugin);
    }

    pub fn remove_plugin(&mut self, plugin_id: &str) {
        self.plugins.remove(plugin_id);
    }

    pub fn get_plugin(&self, plugin_id: &str) -> Option<&Plugin> {
        self.plugins.get(plugin_id)
    }

    pub fn list_plugins(&self) -> Vec<&Plugin> {
        self.plugins.values().collect()
    }

    /// Run all enabled plugins that are due
    pub fn run_due_plugins(&mut self) -> Vec<Result<PluginResult>> {
        let now = Utc::now();
        let mut results = Vec::new();

        for plugin in self.plugins.values_mut() {
            if !plugin.enabled {
                continue;
            }

            // Check if plugin is due
            let should_run = if let Some(last_run) = plugin.last_run {
                let elapsed = now.signed_duration_since(last_run).num_seconds();
                elapsed >= plugin.schedule_interval_secs as i64
            } else {
                true // Never run before
            };

            if should_run {
                let result = plugin.execute();
                if let Ok(ref result) = result {
                    plugin.last_run = Some(result.output.timestamp);
                    plugin.last_output = Some(result.output.clone());
                }
                results.push(result);
            }
        }

        results
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
