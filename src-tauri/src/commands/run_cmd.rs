//! RUN_CMD agent: restricted local command execution for Ollama.
//!
//! Allowlist is read from the first enabled orchestrator's skill.md (section "## RUN_CMD allowlist");
//! if missing, the default list below is used. Paths under ~/.mac-stats where applicable.
//! Each pipeline stage is executed via `sh -c`; stages are split only at top-level `|`.
//! Compound shell in one stage (`;`, `&&`, nested `|`, command substitution, etc.) is rejected
//! fail-closed. See docs/011_local_cmd_agent.md.

use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn};

/// Default allowlist when orchestrator skill.md has no "## RUN_CMD allowlist" section.
/// Security: cursor-agent is an exception — it runs user/agent-controlled prompts in the user
/// environment and receives args without path validation; treat it as a privileged capability.
const DEFAULT_ALLOWED_COMMANDS: &[&str] = &[
    "cat",
    "head",
    "tail",
    "ls",
    "grep",
    "date",
    "whoami",
    "ps",
    "wc",
    "uptime",
    "cursor-agent",
];

/// Commands that always require a path (under ~/.mac-stats). All others in the allowlist are treated as no-path.
const PATH_REQUIRED_COMMANDS: &[&str] = &["cat", "head", "tail", "grep"];

/// First token cannot be a nested shell / env wrapper even if mistakenly added to the skill allowlist.
const BLOCKED_FIRST_COMMANDS: &[&str] = &[
    "sh",
    "bash",
    "zsh",
    "dash",
    "fish",
    "ksh",
    "csh",
    "tcsh",
    "pwsh",
    "powershell",
    "env",
    "exec",
    "nix-shell",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum RunCmdQuote {
    None,
    Single,
    Double,
}

/// Fail-closed validation for one pipeline stage before `sh -c`.
/// Quote-aware (`'` / `"`). Allows redirects such as `>`, `>>`, `<`, `2>`, `2>&1`, `&>`.
/// Rejects command chaining and nested composition inside the stage; use top-level `|` or separate RUN_CMD lines.
pub(crate) fn validate_run_cmd_stage_shape(stage: &str) -> Result<(), String> {
    let s = stage.trim();
    if s.is_empty() {
        return Err("RUN_CMD: empty command.".to_string());
    }
    if s.contains('\n') || s.contains('\r') {
        return Err(
            "RUN_CMD: newlines are not allowed inside a command stage; use separate RUN_CMD lines."
                .to_string(),
        );
    }
    if s.trim_start().starts_with('(') {
        return Err(
            "RUN_CMD: a leading subshell '(' is not allowed in one stage; use separate RUN_CMD lines."
                .to_string(),
        );
    }

    let chars: Vec<char> = s.chars().collect();
    let mut i = 0usize;
    let mut quote = RunCmdQuote::None;

    while i < chars.len() {
        let c = chars[i];
        match quote {
            RunCmdQuote::None => {
                if c == '\'' {
                    quote = RunCmdQuote::Single;
                    i += 1;
                    continue;
                }
                if c == '"' {
                    quote = RunCmdQuote::Double;
                    i += 1;
                    continue;
                }
                if c == '\\' {
                    i += 1;
                    if i < chars.len() {
                        i += 1;
                    }
                    continue;
                }
                if c == '$' && i + 1 < chars.len() && chars[i + 1] == '(' {
                    return Err(
                        "RUN_CMD: command substitution $(...) is not allowed; use a pipeline stage or another RUN_CMD."
                            .to_string(),
                    );
                }
                if c == '`' {
                    return Err(
                        "RUN_CMD: backticks are not allowed; use a pipeline or another RUN_CMD."
                            .to_string(),
                    );
                }
                if c == '<' && i + 1 < chars.len() && chars[i + 1] == '(' {
                    return Err("RUN_CMD: process substitution <(...) is not allowed.".to_string());
                }
                if c == '>' && i + 1 < chars.len() && chars[i + 1] == '(' {
                    return Err("RUN_CMD: process substitution >(...) is not allowed.".to_string());
                }
                if c == ';' {
                    return Err(
                        "RUN_CMD: ';' is not allowed in one stage; use top-level '|' between commands or separate RUN_CMD lines."
                            .to_string(),
                    );
                }
                if c == '|' {
                    if i + 1 < chars.len() && chars[i + 1] == '|' {
                        return Err(
                            "RUN_CMD: '||' is not allowed in one stage; use separate RUN_CMD lines."
                                .to_string(),
                        );
                    }
                    return Err(
                        "RUN_CMD: '|' inside a stage is not allowed; split pipelines only at the top level (e.g. RUN_CMD: cmd1 | cmd2)."
                            .to_string(),
                    );
                }
                if c == '&' {
                    if i + 1 < chars.len() && chars[i + 1] == '&' {
                        return Err(
                            "RUN_CMD: '&&' is not allowed in one stage; use separate RUN_CMD lines."
                                .to_string(),
                        );
                    }
                    if i + 1 < chars.len() && chars[i + 1] == '>' {
                        // &>word
                        i += 2;
                        continue;
                    }
                    if i > 0 && chars[i - 1] == '>' {
                        // >&, 2>&
                        i += 1;
                        continue;
                    }
                    return Err(
                        "RUN_CMD: '&' is not allowed here (no background jobs); use separate RUN_CMD lines or '|' between whole stages."
                            .to_string(),
                    );
                }
                i += 1;
            }
            RunCmdQuote::Single => {
                if c == '\'' {
                    quote = RunCmdQuote::None;
                }
                i += 1;
            }
            RunCmdQuote::Double => {
                if c == '\\' && i + 1 < chars.len() {
                    i += 2;
                    continue;
                }
                if c == '"' {
                    quote = RunCmdQuote::None;
                }
                i += 1;
            }
        }
    }
    Ok(())
}

pub(crate) fn reject_nested_interpreter(first: &str) -> Result<(), String> {
    if BLOCKED_FIRST_COMMANDS.contains(&first) {
        return Err(
            "RUN_CMD: nested shells and env wrappers (sh, bash, env, …) are not allowed; run an allowlisted command directly."
                .to_string(),
        );
    }
    Ok(())
}

/// Return the current allowlist (from orchestrator skill.md or default). Used by the tool loop for retry prompts.
pub fn allowed_commands() -> Vec<String> {
    crate::agents::get_run_cmd_allowlist().unwrap_or_else(|| {
        DEFAULT_ALLOWED_COMMANDS
            .iter()
            .map(|s| (*s).to_string())
            .collect()
    })
}

fn get_allowed_commands() -> Vec<String> {
    allowed_commands()
}

/// Read ALLOW_LOCAL_CMD from env or .config.env. "0", "false", "no" => false; default true.
fn allow_local_cmd_from_config_env_file(path: &Path) -> Option<bool> {
    // Do not log file content or path; file may contain secrets.
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

/// Parse RUN_CMD argument into tokens (whitespace split). Used for allowlist and path validation.
fn parse_arg(arg: &str) -> Vec<String> {
    arg.split_whitespace()
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
}

/// First token (command name) from a stage string, ignoring redirects for the purpose of allowlist.
fn first_command_token(stage: &str) -> Option<String> {
    parse_arg(stage)
        .into_iter()
        .next()
        .map(|s| s.to_lowercase())
}

/// Run a single pipeline stage via shell so redirects (e.g. `>`, `2>&1`) are interpreted. Returns stdout bytes.
/// Validates: stage shape (no compound shell inside the stage); first token not a blocked interpreter; allowlist; paths.
fn run_single_command_shell(
    stage: &str,
    base: &Path,
    stdin_data: Option<&[u8]>,
) -> Result<Vec<u8>, String> {
    use std::process::Stdio;

    if let Err(e) = validate_run_cmd_stage_shape(stage) {
        warn!(
            "RUN_CMD: rejected stage (shape): {} — {}",
            crate::logging::ellipse(stage, 120),
            e
        );
        return Err(e);
    }

    let allowed = get_allowed_commands();
    let first = first_command_token(stage).ok_or_else(|| "RUN_CMD: empty command.".to_string())?;
    if let Err(e) = reject_nested_interpreter(&first) {
        warn!(
            "RUN_CMD: rejected stage (interpreter): {} — {}",
            crate::logging::ellipse(stage, 120),
            e
        );
        return Err(e);
    }
    if !allowed.contains(&first) {
        return Err(format!(
            "Command not allowed (allowed: {}).",
            allowed.join(", ")
        ));
    }
    if first == "cursor-agent" {
        // cursor-agent: no path validation; run as-is via shell (args are prompt/CLI).
        info!(
            "RUN_CMD: executing via shell: {}",
            crate::logging::ellipse(stage, 120)
        );
        let mut child = Command::new("sh")
            .args(["-c", stage])
            .stdin(if stdin_data.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Command failed: {}", e))?;
        if let Some(data) = stdin_data {
            use std::io::Write;
            if let Some(ref mut stdin) = child.stdin {
                let _ = stdin.write_all(data);
            }
            drop(child.stdin.take());
        }
        let output = child
            .wait_with_output()
            .map_err(|e| format!("Command failed: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.trim().is_empty() {
                return Err(format!("Command failed: {}", stderr.trim()));
            }
        }
        return Ok(output.stdout);
    }
    // Validate path-like tokens (contain / or start with ~) are under base.
    let tokens = parse_arg(stage);
    for t in &tokens {
        let looks_like_path = t.contains('/') || t.starts_with('~');
        if looks_like_path && !t.starts_with('-') {
            let expanded = expand_tilde(t);
            let path = Path::new(&expanded);
            if path.exists() {
                if !path_under_base(path, base)? {
                    return Err("Path not allowed (must be under ~/.mac-stats).".to_string());
                }
            } else if let Some(parent) = path.parent() {
                if parent.exists() && !path_under_base(parent, base)? {
                    return Err("Path not allowed (must be under ~/.mac-stats).".to_string());
                }
            }
        }
    }
    info!(
        "RUN_CMD: executing via shell: {}",
        crate::logging::ellipse(stage, 120)
    );
    let mut child = Command::new("sh")
        .args(["-c", stage])
        .stdin(if stdin_data.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Command failed: {}", e))?;
    if let Some(data) = stdin_data {
        use std::io::Write;
        if let Some(ref mut stdin) = child.stdin {
            let _ = stdin.write_all(data);
        }
        drop(child.stdin.take());
    }
    let output = child
        .wait_with_output()
        .map_err(|e| format!("Command failed: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.trim().is_empty() {
            return Err(format!("Command failed: {}", stderr.trim()));
        }
    }
    Ok(output.stdout)
}

/// Run a restricted local command. Stages are split at top-level `|` only; each stage runs via `sh -c`.
/// Allowlist from orchestrator skill.md "## RUN_CMD allowlist" or default (cat, head, tail, ls, grep, date, whoami, ps, wc, uptime, cursor-agent).
/// cursor-agent runs user/agent-controlled prompts and is not path-bound; document or restrict via skill.md if locking down.
/// Compound shell inside a stage (`;`, `&&`, inner `|`, command substitution, …) is rejected.
/// Path-like arguments must be under ~/.mac-stats where applicable.
pub fn run_local_command(arg: &str) -> Result<String, String> {
    info!("RUN_CMD: exact command: {}", arg);
    let stages: Vec<&str> = arg
        .split('|')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if stages.is_empty() {
        return Err("RUN_CMD requires: RUN_CMD: <command> [args] (e.g. RUN_CMD: cat ~/.mac-stats/schedules.json).".to_string());
    }

    let base = permitted_base_dir()?;
    let mut prev_stdout: Option<Vec<u8>> = None;

    for (i, stage) in stages.iter().enumerate() {
        let cmd = first_command_token(stage)
            .ok_or_else(|| format!("Empty command in pipeline stage {}", i + 1))?;
        let allowed = get_allowed_commands();
        if !allowed.contains(&cmd) {
            return Err(format!(
                "Command not allowed (allowed: {}).",
                allowed.join(", ")
            ));
        }
        let no_path_needed = !PATH_REQUIRED_COMMANDS.contains(&cmd.as_str());
        let has_stdin = prev_stdout.is_some();
        if cmd != "cursor-agent" && cmd != "grep" && !no_path_needed && !has_stdin {
            // ls with no args is allowed (we run the stage as-is; shell will get ls from PATH).
            let tokens = parse_arg(stage);
            if tokens.len() <= 1 && !has_stdin {
                return Err("RUN_CMD: command requires a path (e.g. RUN_CMD: cat ~/.mac-stats/schedules.json). Use date, whoami, ps, wc, or uptime with no path for system info.".to_string());
            }
        }
        if cmd == "grep" && !has_stdin {
            let tokens = parse_arg(stage);
            if tokens.len() < 2 {
                return Err("RUN_CMD: grep requires pattern and path, or pipe input (e.g. RUN_CMD: ps aux | grep tail).".to_string());
            }
        }

        let stdin_data = prev_stdout.as_deref();
        prev_stdout = Some(run_single_command_shell(stage, &base, stdin_data)?);
    }

    let stdout = prev_stdout.unwrap_or_default();
    let out = String::from_utf8_lossy(&stdout);
    Ok(out.trim().to_string())
}

#[cfg(test)]
mod run_cmd_stage_validate_tests {
    use super::{reject_nested_interpreter, validate_run_cmd_stage_shape};

    #[test]
    fn shape_rejects_semicolon_chain() {
        assert!(validate_run_cmd_stage_shape("cat ~/.mac-stats/foo; ps").is_err());
    }

    #[test]
    fn shape_rejects_and_and() {
        assert!(validate_run_cmd_stage_shape("date && date").is_err());
    }

    #[test]
    fn shape_rejects_or_or() {
        assert!(validate_run_cmd_stage_shape("date || date").is_err());
    }

    #[test]
    fn shape_rejects_inner_pipe() {
        assert!(validate_run_cmd_stage_shape("cat a | wc").is_err());
    }

    #[test]
    fn shape_allows_top_level_split_only_in_runner() {
        // Validator is per-stage; inner | must fail.
        assert!(validate_run_cmd_stage_shape("date").is_ok());
        assert!(validate_run_cmd_stage_shape("wc -c").is_ok());
    }

    #[test]
    fn shape_rejects_command_substitution() {
        assert!(validate_run_cmd_stage_shape("echo $(whoami)").is_err());
    }

    #[test]
    fn shape_rejects_backticks() {
        assert!(validate_run_cmd_stage_shape("echo `whoami`").is_err());
    }

    #[test]
    fn shape_rejects_leading_subshell() {
        assert!(validate_run_cmd_stage_shape("( date )").is_err());
    }

    #[test]
    fn shape_allows_redir_merge() {
        assert!(validate_run_cmd_stage_shape("date 2>&1").is_ok());
    }

    #[test]
    fn shape_allows_amp_redirect() {
        assert!(validate_run_cmd_stage_shape("date &> /dev/null").is_ok());
    }

    #[test]
    fn shape_ignores_metachars_in_single_quotes() {
        assert!(validate_run_cmd_stage_shape("grep ';' ~/.mac-stats/schedules.json").is_ok());
    }

    #[test]
    fn nested_interpreter_rejects_sh() {
        assert!(reject_nested_interpreter("sh").is_err());
    }

    #[test]
    fn nested_interpreter_rejects_env() {
        assert!(reject_nested_interpreter("env").is_err());
    }

    #[test]
    fn pipeline_date_wc_integration() {
        let out = super::run_local_command("date | wc -c").expect("legal pipeline");
        assert!(!out.is_empty());
    }

    #[test]
    fn pipeline_rejects_injected_second_command() {
        let err = super::run_local_command("date; wc").expect_err("compound should fail");
        assert!(
            err.contains(';') || err.contains("not allowed"),
            "unexpected err: {}",
            err
        );
    }
}
