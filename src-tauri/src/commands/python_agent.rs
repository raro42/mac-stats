//! PYTHON_SCRIPT agent: create and run Python scripts for Ollama.
//!
//! Writes scripts under ~/.mac-stats/scripts/python-script-<id>-<topic>.py,
//! runs them with python3, and returns stdout (exit 0) or error (non-zero).
//! See docs/014_python_agent.md.

use std::path::Path;
use std::process::Command;
use tracing::info;

/// Read ALLOW_PYTHON_SCRIPT from env or .config.env. "0", "false", "no" => false; default true.
fn allow_python_script_from_config_env_file(path: &Path) -> Option<bool> {
    let content = std::fs::read_to_string(path).ok()?;
    let line = content.lines().find(|l| {
        let t = l.trim();
        t.starts_with("ALLOW_PYTHON_SCRIPT=") || t.starts_with("ALLOW-PYTHON-SCRIPT=")
    })?;
    let (_, v) = line.split_once('=')?;
    let v = v.trim().to_lowercase();
    Some(!matches!(v.as_str(), "0" | "false" | "no" | "off"))
}

/// Whether PYTHON_SCRIPT agent is enabled. Default true; set ALLOW_PYTHON_SCRIPT=0 to disable.
pub fn is_python_script_allowed() -> bool {
    if let Ok(v) = std::env::var("ALLOW_PYTHON_SCRIPT") {
        let v = v.trim().to_lowercase();
        if matches!(v.as_str(), "0" | "false" | "no" | "off") {
            return false;
        }
        return true;
    }
    if let Ok(cwd) = std::env::current_dir() {
        let p = cwd.join(".config.env");
        if p.is_file() {
            if let Some(allowed) = allow_python_script_from_config_env_file(&p) {
                return allowed;
            }
        }
        let p_src = cwd.join("src-tauri").join(".config.env");
        if p_src.is_file() {
            if let Some(allowed) = allow_python_script_from_config_env_file(&p_src) {
                return allowed;
            }
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let p = Path::new(&home).join(".mac-stats").join(".config.env");
        if p.is_file() {
            if let Some(allowed) = allow_python_script_from_config_env_file(&p) {
                return allowed;
            }
        }
    }
    true
}

/// Sanitize a string for use in a filename: keep only [a-zA-Z0-9_-], replace rest with underscore.
fn sanitize_filename_part(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Create script at ~/.mac-stats/scripts/python-script-<id>-<topic>.py, run with python3, return stdout or error.
pub fn run_python_script(id: &str, topic: &str, script_body: &str) -> Result<String, String> {
    let id = sanitize_filename_part(id);
    let topic = sanitize_filename_part(topic);
    if id.is_empty() || topic.is_empty() {
        return Err("PYTHON_SCRIPT requires non-empty id and topic (after sanitization).".to_string());
    }

    crate::config::Config::ensure_scripts_directory()
        .map_err(|e| format!("Could not create scripts directory: {}", e))?;

    let script_path = crate::config::Config::scripts_dir()
        .join(format!("python-script-{}-{}.py", id, topic));

    std::fs::write(&script_path, script_body).map_err(|e| format!("Write script failed: {}", e))?;
    info!(
        "PYTHON_SCRIPT: wrote {:?} ({} bytes), running with python3",
        script_path,
        script_body.len()
    );

    let output = Command::new("python3")
        .arg(&script_path)
        .output()
        .map_err(|e| format!("Failed to run python3: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().to_string())
    } else {
        let code = output.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("exit code {}: {}", code, stderr.trim()))
    }
}
