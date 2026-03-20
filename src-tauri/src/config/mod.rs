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
//! - `config.json` — read on every access (window decorations, scheduler interval, maxSchedules, ollamaChatTimeoutSecs, browserViewportWidth/Height, perplexityMaxResults, perplexitySnippetMaxChars).
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
            return home_path.join(".mac-stats").join("credential_accounts.json");
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
    pub fn session_dir() -> PathBuf {
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

    /// Idle timeout in seconds for the CDP browser session. If the browser is not used for this long, it is closed. Default: 3600 (1 hour).
    pub fn browser_idle_timeout_secs() -> u64 {
        3600
    }

    /// Maximum navigation wait timeout in seconds for BROWSER_NAVIGATE and BROWSER_GO_BACK. Slow or stuck navigations fail with a clear message instead of hanging. Config: config.json `browserNavigationTimeoutSecs`. Env: `MAC_STATS_BROWSER_NAVIGATION_TIMEOUT_SECS`. Default 30, clamped to 5..=120.
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
    pub fn task_dir() -> PathBuf {
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
                        if p.extension().map_or(false, |e| e == "md") {
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
