//! Plugin system module
//!
//! Script-based plugins that output JSON.
//! Plugins are executable scripts (bash/python) that follow a contract.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;
use tracing::warn;

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
            let msg = format!(
                "Plugin script not found: {} (plugin: {})",
                self.script_path.display(),
                self.id
            );
            warn!("{}", msg);
            return Err(anyhow::anyhow!("{}", msg));
        }

        // Determine interpreter based on file extension
        let (interpreter, _args): (&str, Vec<&str>) = if self
            .script_path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s == "py")
            .unwrap_or(false)
        {
            ("python3", vec![])
        } else if self
            .script_path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s == "sh" || s == "bash")
            .unwrap_or(false)
        {
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

        let child = cmd.spawn().with_context(|| {
            format!(
                "Failed to spawn plugin script '{}' (plugin: {})",
                self.script_path.display(),
                self.id
            )
        })?;
        #[cfg(unix)]
        let pid = child.id();
        let timeout_duration = Duration::from_secs(self.timeout_secs.max(1));
        let (tx, rx) = mpsc::channel();
        let child_handle = std::thread::spawn(move || {
            let _ = tx.send(child.wait_with_output());
        });

        let output = match rx.recv_timeout(timeout_duration) {
            Ok(Ok(out)) => out,
            Ok(Err(e)) => {
                let msg = format!(
                    "Plugin process wait failed (plugin: {}, script: {}): {}",
                    self.id,
                    self.script_path.display(),
                    e
                );
                warn!("{}", msg);
                return Err(anyhow::anyhow!("{}", msg));
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                #[cfg(unix)]
                {
                    // SAFETY: pid is our child process; we kill it on timeout. libc::kill is async-signal-safe.
                    unsafe { libc::kill(pid as i32, libc::SIGKILL); }
                }
                let _ = rx.recv(); // Let the thread receive wait_with_output result and exit.
                let _ = child_handle.join();
                let execution_time_ms = start_time.elapsed().as_millis() as u64;
                warn!(
                    "Plugin script timed out after {}s (plugin: {}, script: {})",
                    self.timeout_secs,
                    self.id,
                    self.script_path.display()
                );
                return Ok(PluginResult {
                    plugin_id: self.id.clone(),
                    output: PluginOutput {
                        status: "error".to_string(),
                        message: Some(format!(
                            "Plugin script timed out after {}s (plugin: {})",
                            self.timeout_secs,
                            self.id
                        )),
                        data: None,
                        metrics: None,
                        timestamp: Utc::now(),
                    },
                    execution_time_ms,
                });
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                let _ = child_handle.join();
                let msg = format!(
                    "Plugin wait thread disconnected (plugin: {}, script: {})",
                    self.id,
                    self.script_path.display()
                );
                warn!("{}", msg);
                return Err(anyhow::anyhow!("{}", msg));
            }
        };

        let _ = child_handle.join();

        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        // Parse JSON output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let plugin_output: PluginOutput = if output.status.success() {
            serde_json::from_str(&stdout).with_context(|| {
                const SNIP: usize = 400;
                let snippet = if stdout.len() > SNIP {
                    format!("{}... (truncated, total {} chars)", &stdout[..SNIP], stdout.len())
                } else {
                    stdout.to_string()
                };
                format!(
                    "Failed to parse plugin JSON output (plugin: {}, script: {}). Stdout: {:?}",
                    self.id,
                    self.script_path.display(),
                    snippet
                )
            })?
        } else {
            // Script failed - create error output with exit code and trimmed stderr
            let stderr_raw = String::from_utf8_lossy(&output.stderr);
            const MAX_STDERR: usize = 1000;
            let stderr = if stderr_raw.len() > MAX_STDERR {
                format!(
                    "... ({} chars omitted)\n{}",
                    stderr_raw.len() - MAX_STDERR,
                    stderr_raw[stderr_raw.len() - MAX_STDERR..].trim_start()
                )
            } else {
                stderr_raw.trim().to_string()
            };
            let exit_code = output
                .status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let stderr_display = if stderr.is_empty() {
                "(no stderr)"
            } else {
                stderr.as_str()
            };
            let msg = format!(
                "Script execution failed (plugin: {}, exit code: {}): {}",
                self.id,
                exit_code,
                stderr_display
            );
            warn!("{}", msg);
            PluginOutput {
                status: "error".to_string(),
                message: Some(msg),
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
            return Err(anyhow::anyhow!(
                "Plugin script not found: {} (plugin: {})",
                self.script_path.display(),
                self.id
            ));
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
