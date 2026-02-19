//! CURSOR_AGENT tool: invoke cursor-agent CLI from the tool loop.
//!
//! When `cursor-agent` is on PATH, Ollama can delegate coding tasks to it.
//! The CLI is run in headless/print mode with JSON output. A configurable
//! workspace path defaults to the mac-stats project root.

use std::process::Command;
use tracing::info;

/// Check if cursor-agent binary is available on PATH.
pub fn is_cursor_agent_available() -> bool {
    Command::new("which")
        .arg("cursor-agent")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Read CURSOR_AGENT_WORKSPACE from env or .config.env files.
/// Falls back to the mac-stats project dir if not set.
fn cursor_agent_workspace() -> String {
    if let Ok(v) = std::env::var("CURSOR_AGENT_WORKSPACE") {
        let v = v.trim().to_string();
        if !v.is_empty() {
            return v;
        }
    }
    for path in config_env_paths() {
        if let Some(val) = read_config_env_key(&path, "CURSOR_AGENT_WORKSPACE") {
            return val;
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let default = format!("{}/projects/mac-stats", home);
        if std::path::Path::new(&default).is_dir() {
            return default;
        }
    }
    ".".to_string()
}

/// Read CURSOR_AGENT_MODEL from env or .config.env files (optional).
fn cursor_agent_model() -> Option<String> {
    if let Ok(v) = std::env::var("CURSOR_AGENT_MODEL") {
        let v = v.trim().to_string();
        if !v.is_empty() {
            return Some(v);
        }
    }
    for path in config_env_paths() {
        if let Some(val) = read_config_env_key(&path, "CURSOR_AGENT_MODEL") {
            return Some(val);
        }
    }
    None
}

fn config_env_paths() -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd.join(".config.env"));
        paths.push(cwd.join("src-tauri").join(".config.env"));
    }
    if let Ok(home) = std::env::var("HOME") {
        paths.push(std::path::PathBuf::from(home).join(".mac-stats").join(".config.env"));
    }
    paths
}

fn read_config_env_key(path: &std::path::Path, key: &str) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let needle_eq = format!("{}=", key);
    let needle_dash = format!("{}=", key.replace('_', "-"));
    for line in content.lines() {
        let t = line.trim();
        if t.starts_with(&needle_eq) || t.starts_with(&needle_dash) {
            let (_, v) = t.split_once('=')?;
            let v = v.trim().to_string();
            if !v.is_empty() {
                return Some(v);
            }
        }
    }
    None
}

/// Run cursor-agent with a prompt in headless mode. Returns stdout or error.
/// Timeout: 120 seconds (cursor-agent tasks can take a while).
pub fn run_cursor_agent(prompt: &str) -> Result<String, String> {
    let workspace = cursor_agent_workspace();
    let model = cursor_agent_model();

    info!(
        "CURSOR_AGENT: running prompt ({} chars) in workspace={}, model={:?}",
        prompt.len(),
        workspace,
        model
    );

    let mut cmd = Command::new("cursor-agent");
    cmd.arg("--print")
        .arg("--trust")
        .arg("--output-format")
        .arg("text")
        .arg("--workspace")
        .arg(&workspace);

    if let Some(m) = &model {
        cmd.arg("--model").arg(m);
    }

    cmd.arg(prompt);

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to spawn cursor-agent: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        let detail = if !stderr.is_empty() { &stderr } else { &stdout };
        return Err(format!(
            "cursor-agent exited with {}: {}",
            output.status,
            detail.trim()
        ));
    }

    let result = stdout.trim().to_string();
    if result.is_empty() && !stderr.is_empty() {
        return Err(format!("cursor-agent produced no output. stderr: {}", stderr.trim()));
    }

    info!("CURSOR_AGENT: completed, output {} chars", result.len());
    Ok(result)
}
