//! Configuration module for portable paths and build information
//!
//! This module provides a centralized way to access configuration values
//! like log file paths and build information, replacing hard-coded values.

//! Configuration management module
//!
//! Provides centralized configuration including:
//! - Log file paths (replaces hard-coded paths)
//! - Build information (date, version, authors)
//!
//! All configuration is environment-aware and portable.
//!
//! **Secrets and `.config.env`:**
//! For production use, store secrets (e.g. Discord token, API keys) in
//! `~/.mac-stats/.config.env` or in Keychain where supported. Avoid placing
//! `.config.env` in the repo or in a shared cwd so it is not exposed. Code that
//! reads `.config.env` must not log file content or paths in a way that could
//! leak secrets.
//!
//! **JSON config reload (no restart needed):**
//! - `config.json` — read on every access (window decorations, scheduler interval, maxSchedules, ollamaChatTimeoutSecs, browserViewportWidth/Height, browserIdleTimeoutSecs, perplexityMaxResults, perplexitySnippetMaxChars, discord_draft_throttle_ms, downloadsOrganizer*).
//! - `schedules.json` — scheduler checks file mtime each loop and reloads when changed.
//! - `discord_channels.json` — Discord loop checks mtime every tick and reloads when changed.

use std::path::PathBuf;

/// Build one default-agent entry from an id. Add new agents by creating defaults/agents/agent-<id>/ and adding default_agent_entry!("<id>") to DEFAULT_AGENT_IDS.
macro_rules! default_agent_entry {
    ($id:literal) => {
        (
            concat!("agent-", $id),
            &[
                (
                    "agent.json",
                    include_str!(concat!("../../defaults/agents/agent-", $id, "/agent.json")),
                ),
                (
                    "skill.md",
                    include_str!(concat!("../../defaults/agents/agent-", $id, "/skill.md")),
                ),
                (
                    "testing.md",
                    include_str!(concat!("../../defaults/agents/agent-", $id, "/testing.md")),
                ),
            ] as &[(&str, &str)],
        )
    };
}

/// Like default_agent_entry but includes soul.md (for agents with a custom persona, e.g. abliterated).
macro_rules! default_agent_entry_with_soul {
    ($id:literal) => {
        (
            concat!("agent-", $id),
            &[
                (
                    "agent.json",
                    include_str!(concat!("../../defaults/agents/agent-", $id, "/agent.json")),
                ),
                (
                    "skill.md",
                    include_str!(concat!("../../defaults/agents/agent-", $id, "/skill.md")),
                ),
                (
                    "soul.md",
                    include_str!(concat!("../../defaults/agents/agent-", $id, "/soul.md")),
                ),
                (
                    "testing.md",
                    include_str!(concat!("../../defaults/agents/agent-", $id, "/testing.md")),
                ),
            ] as &[(&str, &str)],
        )
    };
}

/// Configuration manager
pub struct Config;

impl Config {
    /// Get the log file path
    ///
    /// Returns a path in the user's home directory: `$HOME/.mac-stats/debug.log`
    /// Falls back to a temporary directory if HOME is not available.
    pub fn log_file_path() -> PathBuf {
        // Try to use $HOME/.mac-stats/debug.log
        if let Ok(home) = std::env::var("HOME") {
            let home_path = PathBuf::from(home);
            return home_path.join(".mac-stats").join("debug.log");
        }

        // Fallback to temp directory
        std::env::temp_dir().join("mac-stats-debug.log")
    }

    /// Path for daily backup of debug.log: `$HOME/.mac-stats/debug.log_sic`. Used when rotating: copy debug.log here, then truncate debug.log once per day.
    pub fn debug_log_sic_path() -> PathBuf {
        Self::log_file_path()
            .parent()
            .map(|p| p.join("debug.log_sic"))
            .unwrap_or_else(|| std::env::temp_dir().join("mac-stats-debug.log_sic"))
    }

    /// Path for state file that stores the last rotation date (YYYY-MM-DD): `$HOME/.mac-stats/.debug_log_last_rotated`.
    pub fn debug_log_last_rotated_path() -> PathBuf {
        Self::log_file_path()
            .parent()
            .map(|p| p.join(".debug_log_last_rotated"))
            .unwrap_or_else(|| std::env::temp_dir().join(".mac-stats-debug_log_last_rotated"))
    }

    /// Get the build date
    ///
    /// Returns the build date from the BUILD_DATE environment variable,
    /// or "unknown" if not available.
    pub fn build_date() -> String {
        std::env::var("BUILD_DATE").unwrap_or_else(|_| "unknown".to_string())
    }

    /// Get the version string
    ///
    /// Returns the package version from CARGO_PKG_VERSION.
    pub fn version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    /// Version string for logs and UI when a session/interaction starts (version + short git hash).
    /// Use this so you can see in logs whether the running binary is the latest build (e.g. "v0.1.28 (a1b2c3d4)").
    pub fn version_display() -> String {
        let v = Self::version();
        let hash = option_env!("GIT_HASH").unwrap_or("unknown");
        if hash.is_empty() || hash == "unknown" {
            format!("v{}", v)
        } else {
            format!("v{} ({})", v, hash)
        }
    }

    /// Get the authors string
    ///
    /// Returns the package authors from CARGO_PKG_AUTHORS.
    pub fn authors() -> String {
        env!("CARGO_PKG_AUTHORS").to_string()
    }

    /// Ensure the log directory exists
    ///
    /// Creates the directory containing the log file if it doesn't exist.
    pub fn ensure_log_directory() -> std::io::Result<()> {
        let log_path = Self::log_file_path();
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    /// Get the config file path
    ///
    /// Returns a path in the user's home directory: `$HOME/.mac-stats/config.json`
    /// Falls back to a temporary directory if HOME is not available.
    pub fn config_file_path() -> PathBuf {
        // Try to use $HOME/.mac-stats/config.json
        if let Ok(home) = std::env::var("HOME") {
            let home_path = PathBuf::from(home);
            return home_path.join(".mac-stats").join("config.json");
        }

        // Fallback to temp directory
        std::env::temp_dir().join("mac-stats-config.json")
    }

    /// Path for persisted list of Keychain credential account names: `$HOME/.mac-stats/credential_accounts.json`.
    /// Used by the security module to list accounts without Keychain attribute enumeration.
    pub fn credential_accounts_file_path() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            let home_path = PathBuf::from(home);
            return home_path
                .join(".mac-stats")
                .join("credential_accounts.json");
        }
        std::env::temp_dir().join("mac-stats-credential_accounts.json")
    }

    /// Read window decorations preference from config file
    ///
    /// Returns true (show decorations) by default if file doesn't exist or can't be read.
    pub fn get_window_decorations() -> bool {
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(decorations) = json.get("windowDecorations").and_then(|v| v.as_bool()) {
                    return decorations;
                }
            }
        }
        // Default to true (show decorations)
        true
    }

    /// Get the monitors file path
    ///
    /// Returns a path in the user's home directory: `$HOME/.mac-stats/monitors.json`
    /// Falls back to a temporary directory if HOME is not available.
    pub fn monitors_file_path() -> PathBuf {
        // Try to use $HOME/.mac-stats/monitors.json
        if let Ok(home) = std::env::var("HOME") {
            let home_path = PathBuf::from(home);
            return home_path.join(".mac-stats").join("monitors.json");
        }

        // Fallback to temp directory
        std::env::temp_dir().join("mac-stats-monitors.json")
    }

    /// Ensure the monitors directory exists
    ///
    /// Creates the directory containing the monitors file if it doesn't exist.
    pub fn ensure_monitors_directory() -> std::io::Result<()> {
        let monitors_path = Self::monitors_file_path();
        if let Some(parent) = monitors_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    /// Get the schedules file path
    ///
    /// Returns a path in the user's home directory: `$HOME/.mac-stats/schedules.json`
    /// Falls back to a temporary directory if HOME is not available.
    pub fn schedules_file_path() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            let home_path = PathBuf::from(home);
            return home_path.join(".mac-stats").join("schedules.json");
        }
        std::env::temp_dir().join("mac-stats-schedules.json")
    }

    /// Ensure the schedules directory exists
    ///
    /// Creates the directory containing the schedules file if it doesn't exist.
    pub fn ensure_schedules_directory() -> std::io::Result<()> {
        let schedules_path = Self::schedules_file_path();
        if let Some(parent) = schedules_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    /// Maximum number of schedule entries allowed. When set, SCHEDULE adds are rejected when at cap.
    /// Config: config.json `maxSchedules` (optional number). If missing or 0, no limit. Clamped to 1..=1000.
    pub fn max_schedules() -> Option<u32> {
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json.get("maxSchedules").and_then(|v| v.as_u64()) {
                    let capped = n.clamp(1, 1000);
                    if capped > 0 {
                        return Some(capped as u32);
                    }
                }
            }
        }
        None
    }

    /// Scheduler check interval in seconds: how often to reload schedules from disk.
    /// Default 60 (every minute). Config: config.json `schedulerCheckIntervalSecs`;
    /// override: env `MAC_STATS_SCHEDULER_CHECK_SECS`. Clamped to 1..=86400.
    pub fn scheduler_check_interval_secs() -> u64 {
        let default_secs = 60u64;
        let from_env = std::env::var("MAC_STATS_SCHEDULER_CHECK_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok());
        if let Some(secs) = from_env {
            return secs.clamp(1, 86400);
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("schedulerCheckIntervalSecs")
                    .and_then(|v| v.as_u64())
                {
                    return n.clamp(1, 86400);
                }
            }
        }
        default_secs
    }

    /// Wall-clock timeout per scheduler task in seconds. Prevents one stuck task from blocking the
    /// scheduler loop indefinitely. Default 600 (10 minutes).
    /// Config: config.json `schedulerTaskTimeoutSecs`; override: env `MAC_STATS_SCHEDULER_TASK_TIMEOUT_SECS`.
    /// Clamped to 30..=3600.
    pub fn scheduler_task_timeout_secs() -> u64 {
        const DEFAULT_SECS: u64 = 600;
        const MIN_SECS: u64 = 30;
        const MAX_SECS: u64 = 3600;
        let from_env = std::env::var("MAC_STATS_SCHEDULER_TASK_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok());
        if let Some(secs) = from_env {
            return secs.clamp(MIN_SECS, MAX_SECS);
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("schedulerTaskTimeoutSecs")
                    .and_then(|v| v.as_u64())
                {
                    return n.clamp(MIN_SECS, MAX_SECS);
                }
            }
        }
        DEFAULT_SECS
    }

    /// Milliseconds to wait after the last Discord message in a channel before merging buffered
    /// messages and calling Ollama once. Default 2000. Set to 0 to disable debouncing (every message
    /// is immediate). Config: `discord_debounce_ms`; override: env `MAC_STATS_DISCORD_DEBOUNCE_MS`.
    /// Clamped to 0..=60_000. Per-channel override: `discord_channels.json` on a channel object
    /// as `debounce_ms` (0 = immediate for that channel) or `immediate_ollama`: true.
    pub fn discord_debounce_ms() -> u64 {
        const DEFAULT_MS: u64 = 2000;
        const MAX_MS: u64 = 60_000;
        let from_env = std::env::var("MAC_STATS_DISCORD_DEBOUNCE_MS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok());
        if let Some(ms) = from_env {
            return ms.min(MAX_MS);
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json.get("discord_debounce_ms").and_then(|v| v.as_u64()) {
                    return n.min(MAX_MS);
                }
            }
        }
        DEFAULT_MS
    }

    /// Minimum milliseconds between Discord **draft** message edits while the agent router runs tools.
    /// Default 1500. Config: `discord_draft_throttle_ms`; override: env `MAC_STATS_DISCORD_DRAFT_THROTTLE_MS`.
    /// Clamped to 200..=60_000.
    pub fn discord_draft_throttle_ms() -> u64 {
        const DEFAULT_MS: u64 = 1500;
        const MIN_MS: u64 = 200;
        const MAX_MS: u64 = 60_000;
        let from_env = std::env::var("MAC_STATS_DISCORD_DRAFT_THROTTLE_MS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok());
        if let Some(ms) = from_env {
            return ms.clamp(MIN_MS, MAX_MS);
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("discord_draft_throttle_ms")
                    .and_then(|v| v.as_u64())
                {
                    return n.clamp(MIN_MS, MAX_MS);
                }
            }
        }
        DEFAULT_MS
    }

    /// Ollama /api/chat request timeout in seconds. Used for all chat requests (UI, Discord, session compaction).
    /// Default 300 (5 min). Config: config.json `ollamaChatTimeoutSecs`;
    /// override: env `MAC_STATS_OLLAMA_CHAT_TIMEOUT_SECS`. Clamped to 15..=900.
    pub fn ollama_chat_timeout_secs() -> u64 {
        const DEFAULT_SECS: u64 = 300;
        const MIN_SECS: u64 = 15;
        const MAX_SECS: u64 = 900;
        let from_env = std::env::var("MAC_STATS_OLLAMA_CHAT_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok());
        if let Some(secs) = from_env {
            return secs.clamp(MIN_SECS, MAX_SECS);
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json.get("ollamaChatTimeoutSecs").and_then(|v| v.as_u64()) {
                    return n.clamp(MIN_SECS, MAX_SECS);
                }
            }
        }
        DEFAULT_SECS
    }

    /// Per-prompt timeout for `mac_stats agent test`.
    /// Default 45s so a stuck or overloaded model fails fast during CLI regression runs.
    /// Config: config.json `agentTestTimeoutSecs`; override: env `MAC_STATS_AGENT_TEST_TIMEOUT_SECS`.
    /// Clamped to 5..=300.
    pub fn agent_test_timeout_secs() -> u64 {
        const DEFAULT_SECS: u64 = 45;
        const MIN_SECS: u64 = 5;
        const MAX_SECS: u64 = 300;
        let from_env = std::env::var("MAC_STATS_AGENT_TEST_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok());
        if let Some(secs) = from_env {
            return secs.clamp(MIN_SECS, MAX_SECS);
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json.get("agentTestTimeoutSecs").and_then(|v| v.as_u64()) {
                    return n.clamp(MIN_SECS, MAX_SECS);
                }
            }
        }
        DEFAULT_SECS
    }

    /// Optional post-run agent judge: when true, after an agent run (e.g. Discord reply) we call an LLM
    /// to evaluate whether the task was satisfied and log the verdict. Off by default.
    /// Config: config.json `agentJudgeEnabled` (boolean); override: env `MAC_STATS_AGENT_JUDGE_ENABLED` (true/1/yes).
    pub fn agent_judge_enabled() -> bool {
        if let Ok(s) = std::env::var("MAC_STATS_AGENT_JUDGE_ENABLED") {
            let lower = s.to_lowercase();
            if lower == "true" || lower == "1" || lower == "yes" {
                return true;
            }
            if lower == "false" || lower == "0" || lower == "no" {
                return false;
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(b) = json.get("agentJudgeEnabled").and_then(|v| v.as_bool()) {
                    return b;
                }
            }
        }
        false
    }

    /// Ratio (0.0–1.0) at which a budget warning is injected into the agent tool loop.
    /// When (tool_count + 1) / max_tool_iterations >= this ratio, a warning is appended telling
    /// the model to consolidate results. Set to 0.0 or 1.0 to disable. Default 0.75.
    /// Config: config.json `toolBudgetWarningRatio` (number); override: env `MAC_STATS_TOOL_BUDGET_WARNING_RATIO`.
    pub fn tool_budget_warning_ratio() -> f64 {
        if let Ok(s) = std::env::var("MAC_STATS_TOOL_BUDGET_WARNING_RATIO") {
            if let Ok(v) = s.parse::<f64>() {
                return v.clamp(0.0, 1.0);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(v) = json.get("toolBudgetWarningRatio").and_then(|v| v.as_f64()) {
                    return v.clamp(0.0, 1.0);
                }
            }
        }
        0.75
    }

    /// Maximum number of conversation history messages sent to the planning step.
    /// Fewer messages reduce bias from past tool outputs; more messages help with follow-up context.
    /// Default 6 (3 user + 3 assistant turns). 0 disables planning history entirely.
    /// Config: config.json `planningHistoryCap`; override: env `MAC_STATS_PLANNING_HISTORY_CAP`.
    pub fn planning_history_cap() -> usize {
        if let Ok(s) = std::env::var("MAC_STATS_PLANNING_HISTORY_CAP") {
            if let Ok(v) = s.parse::<usize>() {
                return v.min(40);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(v) = json.get("planningHistoryCap").and_then(|v| v.as_u64()) {
                    return (v as usize).min(40);
                }
            }
        }
        6
    }

    /// Whether to attempt truncating oversized tool results and retrying when
    /// the Ollama API returns a context-overflow error, instead of failing immediately.
    /// Default: true (enabled). Set to false to surface overflow errors without retry.
    /// Config: config.json `contextOverflowTruncateEnabled` (bool); override: env `MAC_STATS_CTX_OVERFLOW_TRUNCATE`.
    pub fn context_overflow_truncate_enabled() -> bool {
        if let Ok(s) = std::env::var("MAC_STATS_CTX_OVERFLOW_TRUNCATE") {
            return !matches!(s.to_lowercase().as_str(), "0" | "false" | "no" | "off");
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(b) = json
                    .get("contextOverflowTruncateEnabled")
                    .and_then(|v| v.as_bool())
                {
                    return b;
                }
            }
        }
        true
    }

    /// When true (default), FETCH_URL / BROWSER_EXTRACT / HTTP-fallback page text is passed
    /// through Unicode homoglyph normalization (fullwidth ASCII → ASCII; confusable angle
    /// brackets → `<` / `>`) before it enters the tool loop. Disable for rollback:
    /// config.json `normalizeUntrustedHomoglyphs`: false; env `MAC_STATS_NORMALIZE_UNTRUSTED_HOMOGLYPHS`: 0/false/off.
    pub fn normalize_untrusted_homoglyphs_enabled() -> bool {
        if let Ok(s) = std::env::var("MAC_STATS_NORMALIZE_UNTRUSTED_HOMOGLYPHS") {
            let lower = s.to_lowercase();
            if matches!(lower.as_str(), "0" | "false" | "no" | "off") {
                return false;
            }
            if matches!(lower.as_str(), "1" | "true" | "yes" | "on") {
                return true;
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(b) = json
                    .get("normalizeUntrustedHomoglyphs")
                    .and_then(|v| v.as_bool())
                {
                    return b;
                }
            }
        }
        true
    }

    /// Maximum character length a single tool-result message is allowed after
    /// truncation during context-overflow recovery. Larger results are cut to
    /// this size plus a "[truncated]" marker. Default: 4096.
    /// Config: config.json `contextOverflowMaxResultChars` (number); override: env `MAC_STATS_CTX_OVERFLOW_MAX_RESULT_CHARS`.
    pub fn context_overflow_max_result_chars() -> usize {
        if let Ok(s) = std::env::var("MAC_STATS_CTX_OVERFLOW_MAX_RESULT_CHARS") {
            if let Ok(v) = s.parse::<usize>() {
                return v.max(256);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(v) = json
                    .get("contextOverflowMaxResultChars")
                    .and_then(|v| v.as_u64())
                {
                    return (v as usize).max(256);
                }
            }
        }
        4096
    }

    /// Get the user-info file path
    ///
    /// Returns a path in the user's home directory: `$HOME/.mac-stats/user-info.json`
    /// Contains information about many users (e.g. Discord user id -> details). Falls back to temp if HOME is not available.
    pub fn user_info_file_path() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            let home_path = PathBuf::from(home);
            return home_path.join(".mac-stats").join("user-info.json");
        }
        std::env::temp_dir().join("mac-stats-user-info.json")
    }

    /// Session directory for persisted chat memory: `$HOME/.mac-stats/session/`
    ///
    /// When `MAC_STATS_SESSION_DIR` is set to a non-empty path, that directory is used instead
    /// (unit tests and isolated runs; caller should create it before writing session files).
    pub fn session_dir() -> PathBuf {
        if let Ok(override_dir) = std::env::var("MAC_STATS_SESSION_DIR") {
            let t = override_dir.trim();
            if !t.is_empty() {
                return PathBuf::from(t);
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".mac-stats").join("session")
        } else {
            std::env::temp_dir().join("mac-stats-session")
        }
    }

    /// Ensure the session directory exists
    pub fn ensure_session_directory() -> std::io::Result<()> {
        std::fs::create_dir_all(Self::session_dir())
    }

    /// Screenshots directory for BROWSER_SCREENSHOT: `$HOME/.mac-stats/screenshots/`
    pub fn screenshots_dir() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".mac-stats").join("screenshots")
        } else {
            std::env::temp_dir().join("mac-stats-screenshots")
        }
    }

    /// Idle timeout in seconds for the CDP browser session. If the browser is not used for this long, it is closed.
    /// Default: 300 (5 minutes). Config: config.json `browserIdleTimeoutSecs`.
    /// Env override: `MAC_STATS_BROWSER_IDLE_TIMEOUT_SECS`. Clamped to 30..=3600.
    pub fn browser_idle_timeout_secs() -> u64 {
        const DEFAULT: u64 = 300;
        const MIN: u64 = 30;
        const MAX: u64 = 3600;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_IDLE_TIMEOUT_SECS") {
            if let Ok(n) = s.parse::<u64>() {
                return n.clamp(MIN, MAX);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json.get("browserIdleTimeoutSecs").and_then(|v| v.as_u64()) {
                    return n.clamp(MIN, MAX);
                }
            }
        }
        DEFAULT
    }

    /// Maximum navigation wait timeout in seconds for BROWSER_NAVIGATE, BROWSER_GO_BACK, BROWSER_GO_FORWARD, and BROWSER_RELOAD. Slow or stuck navigations fail with a clear message instead of hanging. Config: config.json `browserNavigationTimeoutSecs`. Env: `MAC_STATS_BROWSER_NAVIGATION_TIMEOUT_SECS`. Default 30, clamped to 5..=120.
    pub fn browser_navigation_timeout_secs() -> u64 {
        const DEFAULT: u64 = 30;
        const MIN: u64 = 5;
        const MAX: u64 = 120;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_NAVIGATION_TIMEOUT_SECS") {
            if let Ok(n) = s.parse::<u64>() {
                return n.clamp(MIN, MAX);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("browserNavigationTimeoutSecs")
                    .and_then(|v| v.as_u64())
                {
                    return n.clamp(MIN, MAX);
                }
            }
        }
        DEFAULT
    }

    /// Optional shorter navigation wait timeout in seconds for same-domain navigations (BROWSER_NAVIGATE when target host equals current page host). When `None`, same-domain uses the same timeout as cross-domain. Config: config.json `browserSameDomainNavigationTimeoutSecs`. Env: `MAC_STATS_BROWSER_SAME_DOMAIN_NAVIGATION_TIMEOUT_SECS`. When set, clamped to 1..=120. Typical value 5 (or 3 to mirror browser-use).
    pub fn browser_same_domain_navigation_timeout_secs() -> Option<u64> {
        const MIN: u64 = 1;
        const MAX: u64 = 120;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_SAME_DOMAIN_NAVIGATION_TIMEOUT_SECS") {
            if let Ok(n) = s.parse::<u64>() {
                if n > 0 {
                    return Some(n.clamp(MIN, MAX));
                }
                return None;
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("browserSameDomainNavigationTimeoutSecs")
                    .and_then(|v| v.as_u64())
                {
                    if n > 0 {
                        return Some(n.clamp(MIN, MAX));
                    }
                }
            }
        }
        None
    }

    /// Browser viewport width in pixels (CDP/headless window size). Config: config.json `browserViewportWidth`.
    /// Default 1800. Clamped to 800..=3840.
    pub fn browser_viewport_width() -> u32 {
        const DEFAULT: u32 = 1800;
        const MIN: u32 = 800;
        const MAX: u32 = 3840;
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json.get("browserViewportWidth").and_then(|v| v.as_u64()) {
                    return (n as u32).clamp(MIN, MAX);
                }
            }
        }
        DEFAULT
    }

    /// Perplexity search: max number of results to request from the API. Config: config.json `perplexityMaxResults`.
    /// Default 8. Clamped to 1..=20. Env override: `MAC_STATS_PERPLEXITY_MAX_RESULTS`.
    pub fn perplexity_max_results() -> u32 {
        const DEFAULT: u32 = 8;
        const MIN: u32 = 1;
        const MAX: u32 = 20;
        if let Ok(s) = std::env::var("MAC_STATS_PERPLEXITY_MAX_RESULTS") {
            if let Ok(n) = s.parse::<u32>() {
                return n.clamp(MIN, MAX);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json.get("perplexityMaxResults").and_then(|v| v.as_u64()) {
                    return (n as u32).clamp(MIN, MAX);
                }
            }
        }
        DEFAULT
    }

    /// Perplexity search: max characters per snippet when formatting results for the model. Config: config.json `perplexitySnippetMaxChars`.
    /// Default 280. Clamped to 80..=2000. Env override: `MAC_STATS_PERPLEXITY_SNIPPET_MAX_CHARS`.
    pub fn perplexity_snippet_max_chars() -> usize {
        const DEFAULT: usize = 280;
        const MIN: usize = 80;
        const MAX: usize = 2000;
        if let Ok(s) = std::env::var("MAC_STATS_PERPLEXITY_SNIPPET_MAX_CHARS") {
            if let Ok(n) = s.parse::<usize>() {
                return n.clamp(MIN, MAX);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("perplexitySnippetMaxChars")
                    .and_then(|v| v.as_u64())
                {
                    return (n as usize).clamp(MIN, MAX);
                }
            }
        }
        DEFAULT
    }

    /// Browser viewport height in pixels (CDP/headless window size). Config: config.json `browserViewportHeight`.
    /// Default 2400. Clamped to 600..=2160.
    pub fn browser_viewport_height() -> u32 {
        const DEFAULT: u32 = 2400;
        const MIN: u32 = 600;
        const MAX: u32 = 2160;
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json.get("browserViewportHeight").and_then(|v| v.as_u64()) {
                    return (n as u32).clamp(MIN, MAX);
                }
            }
        }
        DEFAULT
    }

    /// Optional SSRF allowlist: hostnames that are permitted even when they resolve to private IPs.
    /// Config: config.json `ssrfAllowedHosts` (JSON array of strings, e.g. `["my-local-service", "192.168.1.50"]`).
    /// Default: empty (no exceptions). Use when the user explicitly wants to allow fetching from a local service.
    pub fn ssrf_allowed_hosts() -> Vec<String> {
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(arr) = json.get("ssrfAllowedHosts").and_then(|v| v.as_array()) {
                    return arr
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                }
            }
        }
        Vec::new()
    }

    /// Skills directory for agent prompt overlays: `$HOME/.mac-stats/agents/skills/`
    /// Files: skill-<number>-<topic>.md (e.g. skill-1-summarize.md, skill-2-code.md).
    pub fn skills_dir() -> PathBuf {
        Self::agents_dir().join("skills")
    }

    /// Ensure the skills directory exists.
    pub fn ensure_skills_directory() -> std::io::Result<()> {
        std::fs::create_dir_all(Self::skills_dir())
    }

    /// Task directory for task files: `$HOME/.mac-stats/task/`
    /// Files: task-<date-time>-<open|wip|finished|unsuccessful>.md (topic and id stored in-file)
    ///
    /// When `MAC_STATS_TASK_DIR` is set to a non-empty path, that directory is used instead
    /// (tests and isolated runs; must exist or `ensure_task_directory` will create it).
    pub fn task_dir() -> PathBuf {
        if let Ok(override_dir) = std::env::var("MAC_STATS_TASK_DIR") {
            let t = override_dir.trim();
            if !t.is_empty() {
                return PathBuf::from(t);
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".mac-stats").join("task")
        } else {
            std::env::temp_dir().join("mac-stats-task")
        }
    }

    /// Ensure the task directory exists.
    pub fn ensure_task_directory() -> std::io::Result<()> {
        std::fs::create_dir_all(Self::task_dir())
    }

    /// Scripts directory for agent-written scripts: `$HOME/.mac-stats/scripts/`
    /// Files: python-script-<id>-<topic>.py (from PYTHON_SCRIPT agent).
    pub fn scripts_dir() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".mac-stats").join("scripts")
        } else {
            std::env::temp_dir().join("mac-stats-scripts")
        }
    }

    /// Ensure the scripts directory exists.
    pub fn ensure_scripts_directory() -> std::io::Result<()> {
        std::fs::create_dir_all(Self::scripts_dir())
    }

    /// Agents directory: `$HOME/.mac-stats/agents/`
    /// Each agent is a subdirectory: agent-<id>/ with agent.json, skill.md, optional soul.md, mood.md, testing.md.
    pub fn agents_dir() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".mac-stats").join("agents")
        } else {
            std::env::temp_dir().join("mac-stats-agents")
        }
    }

    /// Ensure the agents directory exists.
    pub fn ensure_agents_directory() -> std::io::Result<()> {
        std::fs::create_dir_all(Self::agents_dir())
    }

    /// Path to shared soul: `$HOME/.mac-stats/agents/soul.md`
    pub fn soul_file_path() -> PathBuf {
        Self::agents_dir().join("soul.md")
    }

    /// Path to shared (global) memory: `$HOME/.mac-stats/agents/memory.md`
    /// Loaded into every agent's prompt. Contains lessons learned across all sessions.
    pub fn memory_file_path() -> PathBuf {
        Self::agents_dir().join("memory.md")
    }

    /// Path to per-channel Discord memory: `$HOME/.mac-stats/agents/memory-discord-{channel_id}.md`
    /// When replying in a Discord channel (or DM), lessons and compaction output for that channel
    /// are stored here so context is not mixed between channels.
    pub fn memory_file_path_for_discord_channel(channel_id: u64) -> PathBuf {
        Self::agents_dir().join(format!("memory-discord-{}.md", channel_id))
    }

    /// Path to main-session (in-app) memory: `$HOME/.mac-stats/agents/memory-main.md`
    /// Loaded when the request is from the in-app CPU window (no Discord channel), so the main
    /// session has its own persistent memory like Discord channels have memory-discord-{id}.md.
    pub fn memory_file_path_for_main_session() -> PathBuf {
        Self::agents_dir().join("memory-main.md")
    }

    /// Path to Discord channel config: `$HOME/.mac-stats/discord_channels.json`
    pub fn discord_channels_path() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
                .join(".mac-stats")
                .join("discord_channels.json")
        } else {
            std::env::temp_dir().join("mac-stats-discord_channels.json")
        }
    }

    /// Path to escalation patterns: `$HOME/.mac-stats/agents/escalation_patterns.md`
    /// One phrase per line; when a user message contains any phrase (case-insensitive), escalation mode is triggered.
    pub fn escalation_patterns_path() -> PathBuf {
        Self::agents_dir().join("escalation_patterns.md")
    }

    /// Path to session reset phrases: `$HOME/.mac-stats/agents/session_reset_phrases.md`
    /// One phrase per line; when a user message contains any phrase (case-insensitive substring), the session is cleared (like OpenClaw's resetTriggers, but in an MD file).
    pub fn session_reset_phrases_path() -> PathBuf {
        Self::agents_dir().join("session_reset_phrases.md")
    }

    /// Path to cookie reject patterns: `$HOME/.mac-stats/agents/cookie_reject_patterns.md`
    /// One pattern per line; after BROWSER_NAVIGATE we look for a button/link whose text contains any pattern (case-insensitive) and click it to dismiss the cookie banner. User-editable for translation or extra sites.
    pub fn cookie_reject_patterns_path() -> PathBuf {
        Self::agents_dir().join("cookie_reject_patterns.md")
    }

    /// User-editable rules for the Downloads organizer: `$HOME/.mac-stats/agents/downloads-organizer-rules.md`
    pub fn downloads_organizer_rules_path() -> PathBuf {
        Self::agents_dir().join("downloads-organizer-rules.md")
    }

    /// Persisted last-run summary for the Downloads organizer: `$HOME/.mac-stats/downloads-organizer-state.json`
    pub fn downloads_organizer_state_path() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
                .join(".mac-stats")
                .join("downloads-organizer-state.json")
        } else {
            std::env::temp_dir().join("mac-stats-downloads-organizer-state.json")
        }
    }

    /// When true, the background loop may run the Downloads organizer (still respects interval and `off`).
    /// Config: `downloadsOrganizerEnabled` (bool). Env: `MAC_STATS_DOWNLOADS_ORGANIZER_ENABLED` (true/false/1/0).
    pub fn downloads_organizer_enabled() -> bool {
        if let Ok(s) = std::env::var("MAC_STATS_DOWNLOADS_ORGANIZER_ENABLED") {
            let l = s.to_lowercase();
            if matches!(l.as_str(), "1" | "true" | "yes" | "on") {
                return true;
            }
            if matches!(l.as_str(), "0" | "false" | "no" | "off") {
                return false;
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(b) = json
                    .get("downloadsOrganizerEnabled")
                    .and_then(|v| v.as_bool())
                {
                    return b;
                }
            }
        }
        false
    }

    /// `hourly`, `daily`, or `off`. Config: `downloadsOrganizerInterval`. Env: `MAC_STATS_DOWNLOADS_ORGANIZER_INTERVAL`.
    pub fn downloads_organizer_interval() -> String {
        if let Ok(s) = std::env::var("MAC_STATS_DOWNLOADS_ORGANIZER_INTERVAL") {
            let l = s.to_lowercase();
            if l == "hourly" || l == "daily" || l == "off" {
                return l;
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(v) = json
                    .get("downloadsOrganizerInterval")
                    .and_then(|v| v.as_str())
                {
                    let l = v.to_lowercase();
                    if l == "hourly" || l == "daily" || l == "off" {
                        return l;
                    }
                }
            }
        }
        "off".to_string()
    }

    /// Local time of day for `daily` interval (`HH:MM`, 24h). Config: `downloadsOrganizerDailyAtLocal`. Default 09:00 when missing/invalid.
    pub fn downloads_organizer_daily_at_local() -> (u32, u32) {
        let config_path = Self::config_file_path();
        let default_pair = (9u32, 0u32);
        let Ok(content) = std::fs::read_to_string(&config_path) else {
            return default_pair;
        };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
            return default_pair;
        };
        let raw = json
            .get("downloadsOrganizerDailyAtLocal")
            .and_then(|v| v.as_str())
            .unwrap_or("09:00");
        parse_hhmm_local(raw).unwrap_or(default_pair)
    }

    /// Override Downloads root (expand `~/...`). Empty = `~/Downloads`. Config: `downloadsOrganizerPath`. Env: `MAC_STATS_DOWNLOADS_ORGANIZER_PATH`.
    pub fn downloads_organizer_path_raw() -> String {
        if let Ok(s) = std::env::var("MAC_STATS_DOWNLOADS_ORGANIZER_PATH") {
            if !s.trim().is_empty() {
                return s;
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(v) = json.get("downloadsOrganizerPath").and_then(|v| v.as_str()) {
                    if !v.trim().is_empty() {
                        return v.to_string();
                    }
                }
            }
        }
        String::new()
    }

    /// Log planned moves only; no renames. Default **true** (safe). Config: `downloadsOrganizerDryRun`. Env: `MAC_STATS_DOWNLOADS_ORGANIZER_DRY_RUN`.
    pub fn downloads_organizer_dry_run() -> bool {
        if let Ok(s) = std::env::var("MAC_STATS_DOWNLOADS_ORGANIZER_DRY_RUN") {
            let l = s.to_lowercase();
            if matches!(l.as_str(), "1" | "true" | "yes" | "on") {
                return true;
            }
            if matches!(l.as_str(), "0" | "false" | "no" | "off") {
                return false;
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(b) = json
                    .get("downloadsOrganizerDryRun")
                    .and_then(|v| v.as_bool())
                {
                    return b;
                }
            }
        }
        true
    }

    /// Prompts directory: `$HOME/.mac-stats/agents/prompts/`
    pub fn prompts_dir() -> PathBuf {
        Self::agents_dir().join("prompts")
    }

    pub fn planning_prompt_path() -> PathBuf {
        Self::prompts_dir().join("planning_prompt.md")
    }

    pub fn execution_prompt_path() -> PathBuf {
        Self::prompts_dir().join("execution_prompt.md")
    }

    /// Temporary directory for runtime files: `$HOME/.mac-stats/tmp/`
    /// Used for JS execution scratch files, etc. Fallback to system temp if HOME is not set.
    pub fn tmp_dir() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".mac-stats").join("tmp")
        } else {
            std::env::temp_dir().join("mac-stats-tmp")
        }
    }

    /// Subdirectory for JS execution scratch files: `$HOME/.mac-stats/tmp/js/`
    pub fn tmp_js_dir() -> PathBuf {
        Self::tmp_dir().join("js")
    }

    // --- Embedded defaults (from src-tauri/defaults/, baked in at compile time) ---
    // Add a new default agent: create defaults/agents/agent-<id>/ with agent.json, skill.md, testing.md,
    // then add one line below (default_agent_entry!("<id>")).

    pub const DEFAULT_SOUL: &str = include_str!("../../defaults/agents/soul.md");
    const DEFAULT_PLANNING_PROMPT: &str = include_str!("../../defaults/prompts/planning_prompt.md");
    const DEFAULT_EXECUTION_PROMPT: &str =
        include_str!("../../defaults/prompts/execution_prompt.md");

    /// List of default agents. Agents are read in a loop; add new ids here and add the files under defaults/agents/agent-<id>/.
    const DEFAULT_AGENT_IDS: &[(&str, &[(&str, &str)])] = &[
        default_agent_entry!("000"),
        default_agent_entry!("001"),
        default_agent_entry!("002"),
        default_agent_entry!("003"),
        default_agent_entry!("004"),
        default_agent_entry!("005"),
        default_agent_entry_with_soul!("006-redmine"),
        default_agent_entry_with_soul!("abliterated"),
    ];

    const DEFAULT_DISCORD_CHANNELS: &str = include_str!("../../defaults/discord_channels.json");
    const DEFAULT_ESCALATION_PATTERNS: &str = include_str!("../../defaults/escalation_patterns.md");
    const DEFAULT_SESSION_RESET_PHRASES: &str =
        include_str!("../../defaults/session_reset_phrases.md");
    const DEFAULT_COOKIE_REJECT_PATTERNS: &str =
        include_str!("../../defaults/cookie_reject_patterns.md");
    pub const DEFAULT_DOWNLOADS_ORGANIZER_RULES: &str =
        include_str!("../../defaults/downloads-organizer-rules.md");

    /// Write a default file if the target path does not exist (never overwrites user edits).
    fn write_default_if_missing(path: &std::path::Path, content: &str) {
        if path.exists() {
            return;
        }
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, content);
    }

    /// First-line key for a paragraph (trimmed, up to 80 chars). Used to detect same section.
    fn paragraph_key(block: &str) -> String {
        block
            .trim()
            .lines()
            .next()
            .map(|l| l.trim())
            .unwrap_or("")
            .chars()
            .take(80)
            .collect()
    }

    /// Merge default content into existing: append any default paragraph whose first-line key
    /// is not present in existing. Preserves user content; adds new sections from defaults.
    /// See docs/024_mac_stats_merge_defaults.md.
    fn merge_prompt_content(existing: &str, default: &str) -> String {
        let existing_trim = existing.trim();
        let default_trim = default.trim();
        if existing_trim.is_empty() {
            return default_trim.to_string();
        }
        let existing_blocks: Vec<&str> = existing_trim
            .split("\n\n")
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        let existing_keys: std::collections::HashSet<String> = existing_blocks
            .iter()
            .map(|b| Self::paragraph_key(b))
            .collect();
        let default_blocks: Vec<&str> = default_trim
            .split("\n\n")
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        let mut out = existing_trim.to_string();
        for block in default_blocks {
            let key = Self::paragraph_key(block);
            if !existing_keys.contains(&key) {
                out.push_str("\n\n");
                out.push_str(block);
            }
        }
        out
    }

    /// Ensure prompt file exists: if missing write default; if present merge in any new default paragraphs and write back if changed.
    fn merge_default_prompt_if_exists(path: &std::path::Path, default: &str) {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if !path.exists() {
            let _ = std::fs::write(path, default);
            return;
        }
        let existing = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => return,
        };
        let merged = Self::merge_prompt_content(&existing, default);
        if merged != existing.trim() {
            let _ = std::fs::write(path, merged);
        }
    }

    /// Write content to path (overwrite). Used to sync skill.md and testing.md from bundled defaults into ~/.mac-stats/agents/.
    fn write_agent_file(path: &std::path::Path, content: &str) {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, content);
    }

    /// Ensure all default files exist under ~/.mac-stats/. Shared soul.md is written if missing.
    /// For each default agent: agent.json is written only if missing; skill.md and testing.md are
    /// overwritten from the bundled defaults so changes in defaults/ propagate to ~/.mac-stats/agents/.
    pub fn ensure_defaults() {
        let agents = Self::agents_dir();
        let prompts = Self::prompts_dir();
        let skills = Self::skills_dir();
        let tmp = Self::tmp_dir();
        let _ = std::fs::create_dir_all(&agents);
        let _ = std::fs::create_dir_all(&prompts);
        let _ = std::fs::create_dir_all(&skills);
        let _ = std::fs::create_dir_all(&tmp);
        let _ = std::fs::create_dir_all(Self::tmp_js_dir());

        // Migrate skills from old ~/.mac-stats/skills/ to agents/skills/ (one-time)
        if let Ok(home) = std::env::var("HOME") {
            let old_skills = PathBuf::from(&home).join(".mac-stats").join("skills");
            if old_skills.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&old_skills) {
                    for e in entries.flatten() {
                        let p = e.path();
                        if p.extension().is_some_and(|e| e == "md") {
                            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                            let dest = skills.join(name);
                            if !dest.exists() {
                                let _ = std::fs::copy(&p, &dest);
                            }
                        }
                    }
                }
            }
        }

        // Migrate prompts from old ~/.mac-stats/prompts/ to agents/prompts/ (one-time)
        if let Ok(home) = std::env::var("HOME") {
            let old_prompts = PathBuf::from(&home).join(".mac-stats").join("prompts");
            if old_prompts.is_dir() {
                for name in ["planning_prompt.md", "execution_prompt.md"] {
                    let src = old_prompts.join(name);
                    let dest = prompts.join(name);
                    if src.exists() && !dest.exists() {
                        let _ = std::fs::copy(&src, &dest);
                    }
                }
            }
        }

        // Shared soul (personality/tone for all agents when no per-agent soul.md). Read by load_soul_content().
        Self::write_default_if_missing(&agents.join("soul.md"), Self::DEFAULT_SOUL);

        // Prompts: merge defaults into existing so new sections (e.g. Search → screenshot → Discord) are added without overwriting user edits.
        Self::merge_default_prompt_if_exists(
            &prompts.join("planning_prompt.md"),
            Self::DEFAULT_PLANNING_PROMPT,
        );
        Self::merge_default_prompt_if_exists(
            &prompts.join("execution_prompt.md"),
            Self::DEFAULT_EXECUTION_PROMPT,
        );

        // Discord channel config
        Self::write_default_if_missing(
            &Self::discord_channels_path(),
            Self::DEFAULT_DISCORD_CHANNELS,
        );

        // Migrate escalation/session_reset from old ~/.mac-stats/ location to agents/ if user had them there
        let old_escalation = std::env::var("HOME").ok().map(|h| {
            PathBuf::from(h)
                .join(".mac-stats")
                .join("escalation_patterns.md")
        });
        let old_session_reset = std::env::var("HOME").ok().map(|h| {
            PathBuf::from(h)
                .join(".mac-stats")
                .join("session_reset_phrases.md")
        });
        let new_escalation = Self::escalation_patterns_path();
        let new_session_reset = Self::session_reset_phrases_path();
        if let Some(ref p) = old_escalation {
            if p.exists() && !new_escalation.exists() {
                let _ = std::fs::copy(p, &new_escalation);
            }
        }
        if let Some(ref p) = old_session_reset {
            if p.exists() && !new_session_reset.exists() {
                let _ = std::fs::copy(p, &new_session_reset);
            }
        }

        // Escalation patterns (user-editable; triggers "think harder" / completion-oriented run)
        Self::write_default_if_missing(&new_escalation, Self::DEFAULT_ESCALATION_PATTERNS);

        // Session reset phrases (user-editable; phrases that clear session / start fresh, any language)
        Self::write_default_if_missing(&new_session_reset, Self::DEFAULT_SESSION_RESET_PHRASES);

        // Cookie reject patterns (user-editable; translated/localized patterns for auto-dismissing cookie banners after BROWSER_NAVIGATE)
        Self::write_default_if_missing(
            &Self::cookie_reject_patterns_path(),
            Self::DEFAULT_COOKIE_REJECT_PATTERNS,
        );

        Self::write_default_if_missing(
            &Self::downloads_organizer_rules_path(),
            Self::DEFAULT_DOWNLOADS_ORGANIZER_RULES,
        );

        // Default agents: loop over DEFAULT_AGENT_IDS. agent.json only if missing; skill.md and testing.md overwritten from bundle.
        for (dir_name, files) in Self::DEFAULT_AGENT_IDS {
            let dir = agents.join(dir_name);
            let _ = std::fs::create_dir_all(&dir);
            for (file_name, content) in *files {
                let path = dir.join(file_name);
                if file_name == &"agent.json" {
                    Self::write_default_if_missing(&path, content);
                } else {
                    Self::write_agent_file(&path, content);
                }
            }
        }
    }

    /// Reset all default agent files to bundled defaults (force overwrite).
    /// Unlike `ensure_defaults`, this overwrites agent.json, skill.md, testing.md, and soul.md
    /// for every default agent. Optionally reset only a single agent by id (e.g. "000").
    /// Returns the list of agent directory names that were reset.
    pub fn reset_agent_defaults(agent_id_filter: Option<&str>) -> Vec<String> {
        let agents_dir = Self::agents_dir();
        let _ = std::fs::create_dir_all(&agents_dir);

        let mut reset = Vec::new();
        for (dir_name, files) in Self::DEFAULT_AGENT_IDS {
            if let Some(filter) = agent_id_filter {
                let entry_id = dir_name.strip_prefix("agent-").unwrap_or(dir_name);
                if entry_id != filter {
                    continue;
                }
            }
            let dir = agents_dir.join(dir_name);
            let _ = std::fs::create_dir_all(&dir);
            for (file_name, content) in *files {
                Self::write_agent_file(&dir.join(file_name), content);
            }
            reset.push(dir_name.to_string());
        }

        // Also reset shared soul.md when resetting all agents
        if agent_id_filter.is_none() {
            let _ = std::fs::write(agents_dir.join("soul.md"), Self::DEFAULT_SOUL);
        }

        reset
    }

    /// Load soul from ~/.mac-stats/agents/soul.md. If missing, write default and return it.
    pub fn load_soul_content() -> String {
        let path = Self::soul_file_path();
        Self::load_file_or_default(&path, Self::DEFAULT_SOUL)
    }

    /// Load planning prompt from ~/.mac-stats/agents/prompts/planning_prompt.md.
    pub fn load_planning_prompt() -> String {
        let path = Self::planning_prompt_path();
        Self::load_file_or_default(&path, Self::DEFAULT_PLANNING_PROMPT)
    }

    /// Load execution prompt from ~/.mac-stats/agents/prompts/execution_prompt.md.
    pub fn load_execution_prompt() -> String {
        let path = Self::execution_prompt_path();
        Self::load_file_or_default(&path, Self::DEFAULT_EXECUTION_PROMPT)
    }

    /// Load escalation patterns from ~/.mac-stats/agents/escalation_patterns.md.
    /// Returns a list of phrases (one per non-empty, non-comment line). When a user message
    /// contains any phrase case-insensitively, escalation mode is triggered.
    /// If the file is missing, writes the default and returns the default list.
    pub fn load_escalation_patterns() -> Vec<String> {
        let path = Self::escalation_patterns_path();
        let content = Self::load_file_or_default(&path, Self::DEFAULT_ESCALATION_PATTERNS);
        content
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty() && !s.starts_with('#'))
            .map(String::from)
            .collect::<Vec<_>>()
    }

    /// Load session reset phrases from ~/.mac-stats/agents/session_reset_phrases.md.
    /// Returns a list of phrases (one per non-empty, non-comment line). When a user message
    /// contains any phrase (case-insensitive substring), the session is cleared. If the file
    /// is missing, writes the default and returns the default list.
    pub fn load_session_reset_phrases() -> Vec<String> {
        let path = Self::session_reset_phrases_path();
        let content = Self::load_file_or_default(&path, Self::DEFAULT_SESSION_RESET_PHRASES);
        content
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty() && !s.starts_with('#'))
            .map(String::from)
            .collect::<Vec<_>>()
    }

    /// Load cookie reject patterns from ~/.mac-stats/agents/cookie_reject_patterns.md.
    /// Returns a list of patterns (one per non-empty, non-comment line). After BROWSER_NAVIGATE
    /// we look for a button/link whose text contains any pattern (case-insensitive) and click it.
    /// User-editable for translation or extra sites. If the file is missing, writes the default and returns the default list.
    pub fn load_cookie_reject_patterns() -> Vec<String> {
        let path = Self::cookie_reject_patterns_path();
        let content = Self::load_file_or_default(&path, Self::DEFAULT_COOKIE_REJECT_PATTERNS);
        content
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty() && !s.starts_with('#'))
            .map(String::from)
            .collect::<Vec<_>>()
    }

    /// When we detect user dissatisfaction, append their phrase to agents/escalation_patterns.md if it's not already there.
    /// Normalizes to one trimmed line (collapses whitespace). Skips very short phrases and duplicates.
    pub fn append_escalation_pattern_if_new(phrase: &str) {
        let normalized = phrase
            .trim()
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .chars()
            .fold(
                (String::with_capacity(phrase.len()), true),
                |(mut s, mut prev_ws), c| {
                    let is_ws = c.is_ascii_whitespace();
                    if is_ws && !prev_ws {
                        s.push(' ');
                    } else if !is_ws {
                        s.push(c);
                    }
                    prev_ws = is_ws;
                    (s, prev_ws)
                },
            )
            .0;
        let normalized = normalized.trim();
        if normalized.len() < 2 || normalized.starts_with('#') {
            return;
        }
        let existing = Self::load_escalation_patterns();
        let already = existing.iter().any(|p| p.eq_ignore_ascii_case(normalized));
        if already {
            tracing::debug!(
                "Escalation pattern already in file, not appending: \"{}\"",
                normalized
            );
            return;
        }
        let path = Self::escalation_patterns_path();
        if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(&path) {
            use std::io::Write;
            if let Err(e) = writeln!(f, "{}", normalized) {
                tracing::debug!("Could not append escalation pattern: {}", e);
            } else {
                tracing::info!("Escalation pattern added (user phrase): \"{}\"", normalized);
            }
        }
    }

    /// Read a file; if missing or empty, write the default content and return it.
    fn load_file_or_default(path: &std::path::Path, default: &str) -> String {
        match std::fs::read_to_string(path) {
            Ok(s) => {
                let trimmed = s.trim().to_string();
                if trimmed.is_empty() {
                    let _ = std::fs::write(path, default);
                    default.trim().to_string()
                } else {
                    trimmed
                }
            }
            Err(_) => {
                Self::write_default_if_missing(path, default);
                default.trim().to_string()
            }
        }
    }
}

fn parse_hhmm_local(raw: &str) -> Option<(u32, u32)> {
    let raw = raw.trim();
    let mut parts = raw.split(':');
    let h: u32 = parts.next()?.trim().parse().ok()?;
    let m: u32 = parts.next()?.trim().parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    if h < 24 && m < 60 {
        Some((h, m))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn paragraph_key_first_line() {
        assert_eq!(
            Config::paragraph_key("**Tool-first rule:** If the user...\nMore text here"),
            "**Tool-first rule:** If the user..."
        );
    }

    #[test]
    fn paragraph_key_empty() {
        assert_eq!(Config::paragraph_key(""), "");
        assert_eq!(Config::paragraph_key("  "), "");
    }

    #[test]
    fn paragraph_key_truncates_at_80_chars() {
        let long_line = "A".repeat(120);
        let key = Config::paragraph_key(&long_line);
        assert_eq!(key.len(), 80);
    }

    #[test]
    fn merge_prompt_content_empty_existing_returns_default() {
        let result =
            Config::merge_prompt_content("", "Default paragraph one.\n\nDefault paragraph two.");
        assert_eq!(result, "Default paragraph one.\n\nDefault paragraph two.");
    }

    #[test]
    fn merge_prompt_content_identical_no_change() {
        let content = "Paragraph A.\n\nParagraph B.";
        let result = Config::merge_prompt_content(content, content);
        assert_eq!(result, content);
    }

    #[test]
    fn merge_prompt_content_adds_missing_block() {
        let existing = "Paragraph A.\n\nParagraph B.";
        let default = "Paragraph A.\n\nParagraph B.\n\nParagraph C (new).";
        let result = Config::merge_prompt_content(existing, default);
        assert!(result.contains("Paragraph A."));
        assert!(result.contains("Paragraph B."));
        assert!(result.contains("Paragraph C (new)."));
    }

    #[test]
    fn merge_prompt_content_preserves_user_edits() {
        let existing = "Paragraph A — user edited this.\n\nParagraph B.";
        let default = "Paragraph A.\n\nParagraph B.\n\nParagraph C.";
        let result = Config::merge_prompt_content(existing, default);
        assert!(
            result.contains("user edited this"),
            "User edits should be preserved"
        );
        assert!(
            result.contains("Paragraph C."),
            "New default paragraph should be appended"
        );
    }

    #[test]
    fn merge_prompt_content_does_not_duplicate() {
        let existing = "Block one.\n\nBlock two.\n\nBlock three.";
        let default = "Block one.\n\nBlock two.\n\nBlock three.";
        let result = Config::merge_prompt_content(existing, default);
        assert_eq!(result.matches("Block one.").count(), 1);
        assert_eq!(result.matches("Block two.").count(), 1);
        assert_eq!(result.matches("Block three.").count(), 1);
    }

    #[test]
    fn merge_prompt_content_key_matching_is_exact_first_line() {
        let existing =
            "**Tool-first rule:** If the user request can be fulfilled\nwith extra details.";
        let default = "**Tool-first rule:** If the user request can be fulfilled\nwith default details.\n\n**New rule:** Something else.";
        let result = Config::merge_prompt_content(existing, default);
        assert!(
            result.contains("extra details"),
            "Existing block with same first-line key should not be overwritten"
        );
        assert!(
            !result.contains("default details"),
            "Default block with matching first-line key should not be added"
        );
        assert!(
            result.contains("**New rule:** Something else."),
            "Block with new key should be appended"
        );
    }

    #[test]
    fn merge_prompt_content_different_first_line_adds_both() {
        let existing = "**Rule A:** Custom version\nuser details.";
        let default = "**Rule A:** Default version\ndefault details.";
        let result = Config::merge_prompt_content(existing, default);
        assert!(
            result.contains("Custom version") && result.contains("Default version"),
            "Different first lines = different keys, both kept"
        );
    }

    #[test]
    fn merge_prompt_content_whitespace_tolerance() {
        let existing = "  Block A.  \n\n  Block B.  ";
        let default = "Block A.\n\nBlock C.";
        let result = Config::merge_prompt_content(existing, default);
        assert!(result.contains("Block A."));
        assert!(result.contains("Block B."));
        assert!(result.contains("Block C."));
    }

    #[test]
    fn planning_history_cap_default() {
        assert_eq!(Config::planning_history_cap(), 6);
    }
}
