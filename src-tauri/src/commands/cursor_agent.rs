//! CURSOR_AGENT tool: invoke cursor-agent CLI from the tool loop.
//!
//! When `cursor-agent` is on PATH (or `cursorAgentExecutable` is set), Ollama can
//! delegate coding tasks to it. The CLI runs in headless/print mode. Workspace
//! defaults to the mac-stats project root; operators may set
//! `cursorAgentWorkspace` / `cursorAgentExecutable` in `~/.mac-stats/config.json`
//! (Settings Credentials) or `CURSOR_AGENT_WORKSPACE` / `CURSOR_AGENT_EXECUTABLE`
//! in env / `.config.env`.

use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::info;

fn read_app_config_json() -> serde_json::Value {
    use serde_json::json;
    let path = crate::config::Config::config_file_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({}))
}

fn write_app_config_json(v: &serde_json::Value) -> Result<(), String> {
    let path = crate::config::Config::config_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let pretty = serde_json::to_string_pretty(v).map_err(|e| e.to_string())?;
    crate::config::write_text_atomic(&path, &pretty)
}

fn config_string_key(key: &str) -> Option<String> {
    let cfg = read_app_config_json();
    cfg.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Optional absolute/PATH name for the cursor-agent binary (config / env).
pub fn cursor_agent_executable_configured() -> bool {
    cursor_agent_executable_raw().is_some()
}

fn cursor_agent_executable_raw() -> Option<String> {
    if let Ok(v) = std::env::var("CURSOR_AGENT_EXECUTABLE") {
        let v = v.trim().to_string();
        if !v.is_empty() {
            return Some(v);
        }
    }
    for path in config_env_paths() {
        if let Some(val) = read_config_env_key(&path, "CURSOR_AGENT_EXECUTABLE") {
            return Some(val);
        }
    }
    config_string_key("cursorAgentExecutable")
}

/// Resolve the binary used for CURSOR_AGENT / Ready checks.
pub fn cursor_agent_executable() -> String {
    cursor_agent_executable_raw().unwrap_or_else(|| "cursor-agent".to_string())
}

fn path_looks_available(bin: &str) -> bool {
    let p = Path::new(bin);
    if p.is_absolute() || bin.contains('/') {
        return p.is_file();
    }
    let mut cmd = Command::new("which");
    crate::security::host_exec_env::apply_host_exec_env_hardening(&mut cmd);
    cmd.arg(bin)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if cursor-agent binary is available (configured path or PATH).
pub fn is_cursor_agent_available() -> bool {
    path_looks_available(&cursor_agent_executable())
}

/// Workspace for CURSOR_AGENT runs (env → .config.env → config.json → default).
pub fn cursor_agent_workspace() -> String {
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
    if let Some(v) = config_string_key("cursorAgentWorkspace") {
        return v;
    }
    if let Ok(home) = std::env::var("HOME") {
        let default = format!("{}/projects/mac-stats", home);
        if Path::new(&default).is_dir() {
            return default;
        }
    }
    ".".to_string()
}

/// True when workspace comes from config.json (not env / default alone).
pub fn cursor_agent_workspace_configured() -> bool {
    config_string_key("cursorAgentWorkspace").is_some()
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

fn config_env_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd.join(".config.env"));
        paths.push(cwd.join("src-tauri").join(".config.env"));
    }
    if let Ok(home) = std::env::var("HOME") {
        paths.push(
            PathBuf::from(home)
                .join(".mac-stats")
                .join(".config.env"),
        );
    }
    paths
}

fn read_config_env_key(path: &Path, key: &str) -> Option<String> {
    // Do not log file content or path; file may contain secrets.
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

fn short_path_label(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
        .to_string()
}

/// Settings Credentials status for Cursor agent (config only; no CLI probe).
#[tauri::command]
pub fn get_cursor_agent_settings_status() -> Result<serde_json::Value, String> {
    let ready = is_cursor_agent_available();
    let exec = cursor_agent_executable();
    let exec_configured = cursor_agent_executable_configured();
    let workspace = cursor_agent_workspace();
    let workspace_configured = cursor_agent_workspace_configured();
    let workspace_exists = Path::new(&workspace).is_dir();
    Ok(serde_json::json!({
        "ready": ready,
        "executable": if exec_configured { exec.clone() } else { String::new() },
        "executableDisplay": short_path_label(&exec),
        "executableConfigured": exec_configured,
        "workspace": workspace,
        "workspaceDisplay": short_path_label(&workspace),
        "workspaceConfigured": workspace_configured,
        "workspaceExists": workspace_exists,
    }))
}

/// Persist Cursor agent workspace and/or executable into `~/.mac-stats/config.json`.
#[tauri::command]
pub fn save_cursor_agent_settings(
    workspace: Option<String>,
    executable: Option<String>,
) -> Result<serde_json::Value, String> {
    let ws = workspace
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let exe = executable
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    if ws.is_none() && exe.is_none() {
        return Err(
            "Paste a workspace path and/or a cursor-agent binary path first.".into(),
        );
    }
    let mut cfg = read_app_config_json();
    let obj = cfg
        .as_object_mut()
        .ok_or_else(|| "config.json is not an object".to_string())?;
    if let Some(w) = ws {
        obj.insert("cursorAgentWorkspace".into(), serde_json::json!(w));
    }
    if let Some(e) = exe {
        obj.insert("cursorAgentExecutable".into(), serde_json::json!(e));
    }
    write_app_config_json(&cfg)?;
    get_cursor_agent_settings_status()
}

/// Clear custom Cursor agent workspace + executable from config.json.
#[tauri::command]
pub fn clear_cursor_agent_settings() -> Result<serde_json::Value, String> {
    let mut cfg = read_app_config_json();
    if let Some(obj) = cfg.as_object_mut() {
        obj.remove("cursorAgentWorkspace");
        obj.remove("cursorAgentExecutable");
    }
    write_app_config_json(&cfg)?;
    get_cursor_agent_settings_status()
}

/// Build a handoff prompt for verification fallback (research / general tasks, not repo coding).
pub fn build_cursor_handoff_prompt(user_request: &str, verification_reason: Option<&str>) -> String {
    let mut prompt = String::from(
        "Complete this Discord user request thoroughly. Use web search and any tools you have. \
         Prefer factual answers with sources/links for people, companies, or current events. \
         This is not a coding task in the mac-stats repository unless the user explicitly asked for code changes.\n\n",
    );
    prompt.push_str("User request:\n");
    prompt.push_str(user_request.trim());
    prompt.push('\n');
    if let Some(reason) = verification_reason.map(str::trim).filter(|s| !s.is_empty()) {
        prompt.push_str("\nPrior local attempt failed verification because:\n");
        prompt.push_str(reason);
        prompt.push('\n');
    }
    prompt.push_str(
        "\nReturn a clear final answer the user can read in Discord. Do not say web tools are blocked if you can search.",
    );
    prompt
}

/// Run cursor-agent with a prompt in headless mode. Returns stdout or error.
/// Timeout: 120 seconds (cursor-agent tasks can take a while).
pub fn run_cursor_agent(prompt: &str) -> Result<String, String> {
    let workspace = cursor_agent_workspace();
    let model = cursor_agent_model();
    let bin = cursor_agent_executable();

    info!(
        "CURSOR_AGENT: running prompt ({} chars) in workspace={}, bin={}, model={:?}",
        prompt.len(),
        workspace,
        bin,
        model
    );

    let mut cmd = Command::new(&bin);
    crate::security::host_exec_env::apply_host_exec_env_hardening(&mut cmd);
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
        .map_err(|e| format!("Failed to spawn {}: {}", bin, e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        let detail = if !stderr.is_empty() { &stderr } else { &stdout };
        return Err(format!(
            "{} exited with {}: {}",
            bin,
            output.status,
            detail.trim()
        ));
    }

    let result = stdout.trim().to_string();
    if result.is_empty() && !stderr.is_empty() {
        return Err(format!(
            "{} produced no output. stderr: {}",
            bin,
            stderr.trim()
        ));
    }

    info!("CURSOR_AGENT: completed, output {} chars", result.len());
    Ok(result)
}
