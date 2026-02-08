//! RUN_CMD agent: restricted local command execution for Ollama.
//!
//! Allows read-only commands (cat, head, tail, ls) with paths under ~/.mac-stats.
//! No shell; allowlist and path validation only. See docs/011_local_cmd_agent.md.

use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::info;

const ALLOWED_COMMANDS: &[&str] = &["cat", "head", "tail", "ls", "grep"];

/// Read ALLOW_LOCAL_CMD from env or .config.env. "0", "false", "no" => false; default true.
fn allow_local_cmd_from_config_env_file(path: &Path) -> Option<bool> {
    let content = std::fs::read_to_string(path).ok()?;
    let line = content.lines().find(|l| {
        let t = l.trim();
        t.starts_with("ALLOW_LOCAL_CMD=") || t.starts_with("ALLOW-LOCAL-CMD=")
    })?;
    let (_, v) = line.split_once('=')?;
    let v = v.trim().to_lowercase();
    Some(!matches!(v.as_str(), "0" | "false" | "no" | "off"))
}

/// Whether RUN_CMD agent is enabled. Default true; set ALLOW_LOCAL_CMD=0 to disable.
pub fn is_local_cmd_allowed() -> bool {
    if let Ok(v) = std::env::var("ALLOW_LOCAL_CMD") {
        let v = v.trim().to_lowercase();
        if matches!(v.as_str(), "0" | "false" | "no" | "off") {
            return false;
        }
        return true;
    }
    if let Ok(cwd) = std::env::current_dir() {
        let p = cwd.join(".config.env");
        if p.is_file() {
            if let Some(allowed) = allow_local_cmd_from_config_env_file(&p) {
                return allowed;
            }
        }
        let p_src = cwd.join("src-tauri").join(".config.env");
        if p_src.is_file() {
            if let Some(allowed) = allow_local_cmd_from_config_env_file(&p_src) {
                return allowed;
            }
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let p = Path::new(&home).join(".mac-stats").join(".config.env");
        if p.is_file() {
            if let Some(allowed) = allow_local_cmd_from_config_env_file(&p) {
                return allowed;
            }
        }
    }
    true
}

/// Permitted base directory for path arguments (~/.mac-stats or temp fallback).
fn permitted_base_dir() -> Result<PathBuf, String> {
    let base = crate::config::Config::schedules_file_path();
    let parent = base
        .parent()
        .ok_or_else(|| "No parent for schedules path".to_string())?
        .to_path_buf();
    if parent.exists() {
        parent
            .canonicalize()
            .map_err(|e| format!("Canonicalize base dir: {}", e))
    } else {
        Ok(parent)
    }
}

/// Expand leading ~ in path using HOME.
fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{}{}", home, &path[1..]);
        }
    }
    if path == "~" {
        if let Ok(home) = std::env::var("HOME") {
            return home;
        }
    }
    path.to_string()
}

/// Check that canonical path is under permitted base.
fn path_under_base(path: &Path, base: &Path) -> Result<bool, String> {
    let canonical = path
        .canonicalize()
        .map_err(|e| format!("Path not found or not accessible: {}", e))?;
    Ok(canonical.starts_with(base))
}

/// Parse RUN_CMD argument into (command, args). Simple whitespace split; no quoting.
fn parse_arg(arg: &str) -> Vec<String> {
    arg.split_whitespace().map(|s| s.to_string()).collect::<Vec<_>>()
}

/// Validate and resolve path args: expand ~ and ensure under permitted base.
/// Only treats an arg as a path if it contains '/' or starts with '~' (so -n 5 is passed through).
fn validate_path_args(args: &[String], base: &Path) -> Result<Vec<String>, String> {
    let mut out = Vec::with_capacity(args.len());
    for a in args {
        let a = a.as_str();
        if a.starts_with('-') {
            out.push(a.to_string());
            continue;
        }
        let looks_like_path = a.contains('/') || a.starts_with('~');
        if looks_like_path {
            let expanded = expand_tilde(a);
            let path = Path::new(&expanded);
            if path.exists() {
                if !path_under_base(path, base)? {
                    return Err("Path not allowed (must be under ~/.mac-stats).".to_string());
                }
            } else {
                // Path doesn't exist: ensure parent is under base so we don't allow e.g. /etc/passwd
                if let Some(parent) = path.parent() {
                    if parent.exists() && !path_under_base(parent, base)? {
                        return Err("Path not allowed (must be under ~/.mac-stats).".to_string());
                    }
                } else {
                    return Err("Path not allowed (must be under ~/.mac-stats).".to_string());
                }
            }
            out.push(expanded);
        } else {
            out.push(a.to_string());
        }
    }
    Ok(out)
}

/// Run a restricted local command. No shell; allowlist cat, head, tail, ls; paths under ~/.mac-stats.
pub fn run_local_command(arg: &str) -> Result<String, String> {
    let tokens = parse_arg(arg);
    if tokens.is_empty() {
        return Err("RUN_CMD requires: RUN_CMD: <command> [args] (e.g. RUN_CMD: cat ~/.mac-stats/schedules.json).".to_string());
    }
    let cmd = tokens[0].to_lowercase();
    if !ALLOWED_COMMANDS.contains(&cmd.as_str()) {
        return Err(format!(
            "Command not allowed (allowed: {}).",
            ALLOWED_COMMANDS.join(", ")
        ));
    }
    let base = permitted_base_dir()?;
    let args = if tokens.len() > 1 {
        validate_path_args(&tokens[1..], &base)?
    } else if cmd == "ls" {
        vec![base.to_string_lossy().to_string()]
    } else {
        vec![]
    };

    if args.is_empty() && cmd != "ls" {
        return Err("RUN_CMD: command requires a path (e.g. RUN_CMD: cat ~/.mac-stats/schedules.json).".to_string());
    }
    if cmd == "grep" && args.len() < 2 {
        return Err("RUN_CMD: grep requires pattern and path (e.g. RUN_CMD: grep pattern ~/.mac-stats/task/file.md).".to_string());
    }

    info!("RUN_CMD: executing {} with {} args", cmd, args.len());
    let output = Command::new(&cmd)
        .args(&args)
        .output()
        .map_err(|e| format!("Command failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Command failed: {}", stderr.trim()));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.trim().to_string())
}
