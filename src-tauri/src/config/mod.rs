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
//! **Log redaction:** Optional `LOG_REDACTION=0` in env or `.config.env` disables
//! pattern-based masking of secrets in `debug.log` and stderr. Optional
//! `LOG_REDACT_EXTRA_REGEX` (semicolon-separated regexes) adds custom patterns.
//!
//! **JSON config reload (no restart needed):**
//! - `config.json` — read on every access (window decorations, scheduler interval, maxSchedules, heartbeat, ollamaChatTimeoutSecs, ollamaGlobalConcurrency (max concurrent Ollama /api/chat calls app-wide), agentRouterTurnTimeoutSecsDiscord / Ui / Remote (session wall-clock for one full agent run; max 48h), agentRouterMaxToolIterationsDiscord / Ui / Remote (default tool-loop cap when no per-agent override), agentRouterTurnTimeoutCleanupGraceSecs, browserViewportWidth/Height, browserLlmScreenshotWidth/Height (optional vision resize), browserArtifactMaxBytes (max size for browser screenshots/PDF artifacts), browserIdleTimeoutSecs, **browserCdpPort** (loopback remote-debugging port, default 9222), **browserCdpHttpTimeoutSecs** (per-request `reqwest` timeout for `/json/version` discovery; default **5**), **browserCdpWsConnectTimeoutSecs** (WebSocket handshake for CDP attach; default **60**), **browserCdpPostLaunchMaxWaitSecs** / **browserCdpPostLaunchPollIntervalMs** (visible-Chrome auto-launch: poll `/json/version` until ready), **browserChromiumExecutable** (optional path to Chrome / Brave / Edge / Chromium binary), **browserChromiumUserDataDir** (optional profile directory for visible launches), optional **browserCdpEmulateViewportWidth/Height** (+ **browserCdpEmulateDeviceScaleFactor**, **browserCdpEmulateMobile**) and **browserCdpEmulateGeolocationLatitude/Longitude** (+ optional **Accuracy**) for CDP `Emulation.setDeviceMetricsOverride` / `setGeolocationOverride`, browserAllowedDomains / browserBlockedDomains (BROWSER_* navigation policy), browserToolsEnabled, **browserCdpTraceEnabled** / **browserCdpTraceWallClockMinutes** / **browserCdpTraceMaxFileBytes** / **browserCdpTraceMaxRetainedFiles** (optional CDP `Tracing` JSON under `~/.mac-stats/traces/`), **runJsEnabled** (host RUN_JS via Node; default true), perplexityMaxResults, perplexitySnippetMaxChars, discord_draft_throttle_ms, extraAttachmentRoots, screenshotPruneMaxAgeDays / screenshotPruneMaxTotalBytes (`~/.mac-stats/screenshots/` lifecycle), downloadsOrganizer*, beforeResetTranscriptPath, beforeResetHook, beforeCompactionTranscriptPath, beforeCompactionHook, afterCompactionHook).
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

/// Optional hash-based tool-loop repeat detection (agent router / Discord path).
/// See `Config::tool_loop_detection_config()` and docs/007_discord_agent.md §18.
#[derive(Clone, Debug)]
pub struct ToolLoopDetectionConfig {
    pub history_size: usize,
    pub warning_threshold: u32,
    pub critical_threshold: u32,
}

/// Optional periodic agent heartbeat (`config.json` key `heartbeat`).
#[derive(Clone, Debug)]
pub struct HeartbeatSettings {
    pub enabled: bool,
    /// Seconds between heartbeat runs when enabled. Clamped 60–86400; default 1800 (30m).
    pub interval_secs: u64,
    /// Path to checklist markdown (`~` allowed). If missing/unreadable, `checklist_prompt` or built-in default is used.
    pub checklist_path: Option<String>,
    /// Inline checklist when no file or as fallback.
    pub checklist_prompt: Option<String>,
    /// Discord channel snowflake for non-ack replies; omit or empty = log only (no Discord post).
    pub reply_to_channel_id: Option<String>,
    /// Max characters allowed besides HEARTBEAT_OK when treating a reply as a silent ack (default 300).
    pub ack_max_chars: usize,
}

impl Default for HeartbeatSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_secs: 30 * 60,
            checklist_path: None,
            checklist_prompt: None,
            reply_to_channel_id: None,
            ack_max_chars: 300,
        }
    }
}

/// Upper clamp (seconds) for agent-router **session wall-clock** — one full `answer_with_ollama_and_fetch` turn.
/// Default values in config stay short (interactive/menu-bar responsiveness); operators may raise up to 48 hours for long unattended runs.
pub const AGENT_ROUTER_SESSION_WALL_CLOCK_MAX_SECS: u64 = 172800;

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

    /// Heartbeat subsection in `config.json` under key `heartbeat`.
    /// Env `MAC_STATS_HEARTBEAT_INTERVAL_SECS` overrides `intervalSecs` when set (clamped 60–86400).
    pub fn heartbeat_settings() -> HeartbeatSettings {
        let mut s = HeartbeatSettings::default();
        if let Some(v) = std::env::var("MAC_STATS_HEARTBEAT_INTERVAL_SECS")
            .ok()
            .and_then(|x| x.parse::<u64>().ok())
        {
            s.interval_secs = v.clamp(60, 86400);
        }
        let config_path = Self::config_file_path();
        let Ok(content) = std::fs::read_to_string(&config_path) else {
            return s;
        };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
            return s;
        };
        let Some(obj) = json.get("heartbeat").and_then(|v| v.as_object()) else {
            return s;
        };
        if let Some(b) = obj.get("enabled").and_then(|v| v.as_bool()) {
            s.enabled = b;
        }
        if std::env::var("MAC_STATS_HEARTBEAT_INTERVAL_SECS").is_err() {
            if let Some(n) = obj.get("intervalSecs").and_then(|v| v.as_u64()) {
                s.interval_secs = n.clamp(60, 86400);
            }
        }
        if let Some(p) = obj
            .get("checklistPath")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|t| !t.is_empty())
        {
            s.checklist_path = Some(p.to_string());
        }
        if let Some(p) = obj
            .get("checklistPrompt")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|t| !t.is_empty())
        {
            s.checklist_prompt = Some(p.to_string());
        }
        if let Some(c) = obj
            .get("replyToChannelId")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|t| !t.is_empty())
        {
            s.reply_to_channel_id = Some(c.to_string());
        }
        if let Some(n) = obj.get("ackMaxChars").and_then(|v| v.as_u64()) {
            s.ack_max_chars = (n as usize).clamp(0, 2000);
        }
        s
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

    /// Max concurrent Ollama `/api/chat` requests **globally** (across Discord, UI, scheduler, etc.).
    /// Default **1** (single-GPU friendly). Config: `ollamaGlobalConcurrency`; env:
    /// `MAC_STATS_OLLAMA_GLOBAL_CONCURRENCY`. Clamped to 1..=16. Read once on first queue use per process.
    pub fn ollama_global_concurrency() -> u32 {
        const DEFAULT_N: u32 = 1;
        const MIN_N: u32 = 1;
        const MAX_N: u32 = 16;
        let from_env = std::env::var("MAC_STATS_OLLAMA_GLOBAL_CONCURRENCY")
            .ok()
            .and_then(|s| s.parse::<u32>().ok());
        if let Some(n) = from_env {
            return n.clamp(MIN_N, MAX_N);
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("ollamaGlobalConcurrency")
                    .and_then(|v| v.as_u64())
                {
                    return (n as u32).clamp(MIN_N, MAX_N);
                }
            }
        }
        DEFAULT_N
    }

    /// Wall-clock budget for a full agent-router turn (planning + tool loop + verification) when
    /// the request is from **Discord**. Independent of per-request `ollamaChatTimeoutSecs`.
    /// Default 300 (5 min). Config: `agentRouterTurnTimeoutSecsDiscord`;
    /// env: `MAC_STATS_AGENT_ROUTER_TURN_TIMEOUT_SECS_DISCORD`. Clamped 60..=[`AGENT_ROUTER_SESSION_WALL_CLOCK_MAX_SECS`] (48h cap for opt-in long runs).
    pub fn agent_router_turn_timeout_secs_discord() -> u64 {
        const DEFAULT_SECS: u64 = 300;
        const MIN_SECS: u64 = 60;
        const MAX_SECS: u64 = AGENT_ROUTER_SESSION_WALL_CLOCK_MAX_SECS;
        let from_env = std::env::var("MAC_STATS_AGENT_ROUTER_TURN_TIMEOUT_SECS_DISCORD")
            .ok()
            .and_then(|s| s.parse::<u64>().ok());
        if let Some(secs) = from_env {
            return secs.clamp(MIN_SECS, MAX_SECS);
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("agentRouterTurnTimeoutSecsDiscord")
                    .and_then(|v| v.as_u64())
                {
                    return n.clamp(MIN_SECS, MAX_SECS);
                }
            }
        }
        DEFAULT_SECS
    }

    /// Wall-clock budget for a full agent-router turn from the **in-app / CPU** path
    /// (`from_remote == false`). Default 180 (3 min). Config: `agentRouterTurnTimeoutSecsUi`;
    /// env: `MAC_STATS_AGENT_ROUTER_TURN_TIMEOUT_SECS_UI`. Clamped 60..=[`AGENT_ROUTER_SESSION_WALL_CLOCK_MAX_SECS`].
    pub fn agent_router_turn_timeout_secs_ui() -> u64 {
        const DEFAULT_SECS: u64 = 180;
        const MIN_SECS: u64 = 60;
        const MAX_SECS: u64 = AGENT_ROUTER_SESSION_WALL_CLOCK_MAX_SECS;
        let from_env = std::env::var("MAC_STATS_AGENT_ROUTER_TURN_TIMEOUT_SECS_UI")
            .ok()
            .and_then(|s| s.parse::<u64>().ok());
        if let Some(secs) = from_env {
            return secs.clamp(MIN_SECS, MAX_SECS);
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("agentRouterTurnTimeoutSecsUi")
                    .and_then(|v| v.as_u64())
                {
                    return n.clamp(MIN_SECS, MAX_SECS);
                }
            }
        }
        DEFAULT_SECS
    }

    /// Wall-clock budget for Discord-less **remote** runs (scheduler free-text, task runner,
    /// heartbeat). Default 300. Config: `agentRouterTurnTimeoutSecsRemote`;
    /// env: `MAC_STATS_AGENT_ROUTER_TURN_TIMEOUT_SECS_REMOTE`. Clamped 60..=[`AGENT_ROUTER_SESSION_WALL_CLOCK_MAX_SECS`].
    pub fn agent_router_turn_timeout_secs_remote() -> u64 {
        const DEFAULT_SECS: u64 = 300;
        const MIN_SECS: u64 = 60;
        const MAX_SECS: u64 = AGENT_ROUTER_SESSION_WALL_CLOCK_MAX_SECS;
        let from_env = std::env::var("MAC_STATS_AGENT_ROUTER_TURN_TIMEOUT_SECS_REMOTE")
            .ok()
            .and_then(|s| s.parse::<u64>().ok());
        if let Some(secs) = from_env {
            return secs.clamp(MIN_SECS, MAX_SECS);
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("agentRouterTurnTimeoutSecsRemote")
                    .and_then(|v| v.as_u64())
                {
                    return n.clamp(MIN_SECS, MAX_SECS);
                }
            }
        }
        DEFAULT_SECS
    }

    /// Max seconds to wait for turn-timeout browser cleanup (`about:blank`) before logging and moving on.
    /// Default 3. Config: `agentRouterTurnTimeoutCleanupGraceSecs`;
    /// env: `MAC_STATS_AGENT_ROUTER_TURN_TIMEOUT_CLEANUP_GRACE_SECS`. Clamped 1..=30.
    pub fn agent_router_turn_timeout_cleanup_grace_secs() -> u64 {
        const DEFAULT_SECS: u64 = 3;
        const MIN_SECS: u64 = 1;
        const MAX_SECS: u64 = 30;
        let from_env = std::env::var("MAC_STATS_AGENT_ROUTER_TURN_TIMEOUT_CLEANUP_GRACE_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok());
        if let Some(secs) = from_env {
            return secs.clamp(MIN_SECS, MAX_SECS);
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("agentRouterTurnTimeoutCleanupGraceSecs")
                    .and_then(|v| v.as_u64())
                {
                    return n.clamp(MIN_SECS, MAX_SECS);
                }
            }
        }
        DEFAULT_SECS
    }

    /// Default **max tool iterations** for the main agent router when no `agent_override` is set,
    /// for **Discord** agent runs. Per-agent `max_tool_iterations` in `agent.json` overrides this.
    /// Default 15. Config: `agentRouterMaxToolIterationsDiscord`;
    /// env: `MAC_STATS_AGENT_ROUTER_MAX_TOOL_ITERATIONS_DISCORD`. Clamped 1..=200.
    pub fn agent_router_max_tool_iterations_discord() -> u32 {
        const DEFAULT: u32 = 15;
        const MIN: u32 = 1;
        const MAX: u32 = 200;
        if let Ok(s) = std::env::var("MAC_STATS_AGENT_ROUTER_MAX_TOOL_ITERATIONS_DISCORD") {
            if let Ok(v) = s.parse::<u32>() {
                return v.clamp(MIN, MAX);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("agentRouterMaxToolIterationsDiscord")
                    .and_then(|v| v.as_u64())
                {
                    return (n as u32).clamp(MIN, MAX);
                }
            }
        }
        DEFAULT
    }

    /// Default **max tool iterations** for the main agent router when no `agent_override`, **in-app** path.
    /// Default 15. Config: `agentRouterMaxToolIterationsUi`;
    /// env: `MAC_STATS_AGENT_ROUTER_MAX_TOOL_ITERATIONS_UI`. Clamped 1..=200.
    pub fn agent_router_max_tool_iterations_ui() -> u32 {
        const DEFAULT: u32 = 15;
        const MIN: u32 = 1;
        const MAX: u32 = 200;
        if let Ok(s) = std::env::var("MAC_STATS_AGENT_ROUTER_MAX_TOOL_ITERATIONS_UI") {
            if let Ok(v) = s.parse::<u32>() {
                return v.clamp(MIN, MAX);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("agentRouterMaxToolIterationsUi")
                    .and_then(|v| v.as_u64())
                {
                    return (n as u32).clamp(MIN, MAX);
                }
            }
        }
        DEFAULT
    }

    /// Default **max tool iterations** for the main agent router when no `agent_override`, **remote**
    /// path (scheduler, task runner, heartbeat). Default 15. Config: `agentRouterMaxToolIterationsRemote`;
    /// env: `MAC_STATS_AGENT_ROUTER_MAX_TOOL_ITERATIONS_REMOTE`. Clamped 1..=200.
    pub fn agent_router_max_tool_iterations_remote() -> u32 {
        const DEFAULT: u32 = 15;
        const MIN: u32 = 1;
        const MAX: u32 = 200;
        if let Ok(s) = std::env::var("MAC_STATS_AGENT_ROUTER_MAX_TOOL_ITERATIONS_REMOTE") {
            if let Ok(v) = s.parse::<u32>() {
                return v.clamp(MIN, MAX);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("agentRouterMaxToolIterationsRemote")
                    .and_then(|v| v.as_u64())
                {
                    return (n as u32).clamp(MIN, MAX);
                }
            }
        }
        DEFAULT
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

    /// Maximum consecutive tool or follow-up LLM failures in one agent-router request before
    /// the tool loop stops and returns partial results. Separate from `max_tool_iterations`.
    /// Default 3. Config: `config.json` `maxConsecutiveToolFailures` (u32); env
    /// `MAC_STATS_MAX_CONSECUTIVE_TOOL_FAILURES`.
    pub fn max_consecutive_tool_failures() -> u32 {
        const DEFAULT: u32 = 3;
        const MIN: u32 = 1;
        const MAX: u32 = 20;
        if let Ok(s) = std::env::var("MAC_STATS_MAX_CONSECUTIVE_TOOL_FAILURES") {
            if let Ok(v) = s.parse::<u32>() {
                return v.clamp(MIN, MAX);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("maxConsecutiveToolFailures")
                    .and_then(|v| v.as_u64())
                {
                    return (n as u32).clamp(MIN, MAX);
                }
            }
        }
        DEFAULT
    }

    /// Optional OpenClaw-style repeat detection for the agent tool loop (Discord, scheduler,
    /// `run-ollama`). **Default: off** — the legacy `ToolLoopGuard` still applies (block after 3
    /// identical tool+arg calls and short A→B→A→B cycles). When enabled, that legacy rule is
    /// replaced by a SHA-256 signature of **normalized** arguments, a bounded history window, a
    /// configurable **warning** threshold (text appended to the tool result for the model), and a
    /// **critical** threshold (run stops with a short user-facing message).
    ///
    /// Config: `~/.mac-stats/config.json` — object `toolLoopDetection`:
    /// `{ "enabled": true, "historySize": 25, "warningThreshold": 8, "criticalThreshold": 12 }`.
    /// Env `MAC_STATS_TOOL_LOOP_DETECTION_ENABLED`: `true`/`1`/`yes` forces enable; `false`/`0`/`no`
    /// forces disable (overrides JSON `enabled`).
    pub fn tool_loop_detection_config() -> Option<ToolLoopDetectionConfig> {
        const DEFAULT_HISTORY: usize = 25;
        const DEFAULT_WARN: u32 = 8;
        const DEFAULT_CRIT: u32 = 12;

        let mut enabled = false;
        let mut history_size = DEFAULT_HISTORY;
        let mut warning_threshold = DEFAULT_WARN;
        let mut critical_threshold = DEFAULT_CRIT;

        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(obj) = json.get("toolLoopDetection") {
                    enabled = obj.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
                    if let Some(n) = obj.get("historySize").and_then(|v| v.as_u64()) {
                        history_size = n as usize;
                    }
                    if let Some(n) = obj.get("warningThreshold").and_then(|v| v.as_u64()) {
                        warning_threshold = n as u32;
                    }
                    if let Some(n) = obj.get("criticalThreshold").and_then(|v| v.as_u64()) {
                        critical_threshold = n as u32;
                    }
                }
            }
        }

        if let Ok(s) = std::env::var("MAC_STATS_TOOL_LOOP_DETECTION_ENABLED") {
            let lower = s.to_lowercase();
            if matches!(lower.as_str(), "0" | "false" | "no" | "off") {
                return None;
            }
            if matches!(lower.as_str(), "1" | "true" | "yes" | "on") {
                enabled = true;
            }
        }

        if !enabled {
            return None;
        }

        let history_size = history_size.clamp(10, 60);
        let warning_threshold = warning_threshold.clamp(2, 100);
        let mut critical_threshold = critical_threshold.clamp(3, 200);
        if critical_threshold <= warning_threshold {
            critical_threshold = (warning_threshold + 1).min(200);
        }

        Some(ToolLoopDetectionConfig {
            history_size,
            warning_threshold,
            critical_threshold,
        })
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

    /// When true, before each agent-router Ollama call in the tool loop (and the first execution
    /// call), estimate total message size and compact older large tool-style results so the
    /// request is less likely to hit context overflow. Default **false** (opt-in).
    /// Config: `proactiveToolResultContextBudgetEnabled`; env: `MAC_STATS_PROACTIVE_CTX_BUDGET`
    /// (`1` / `true` / `on` to enable, `0` / `false` / `off` to disable).
    pub fn proactive_tool_result_context_budget_enabled() -> bool {
        if let Ok(s) = std::env::var("MAC_STATS_PROACTIVE_CTX_BUDGET") {
            let lower = s.to_lowercase();
            if matches!(lower.as_str(), "1" | "true" | "yes" | "on") {
                return true;
            }
            if matches!(lower.as_str(), "0" | "false" | "no" | "off") {
                return false;
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(b) = json
                    .get("proactiveToolResultContextBudgetEnabled")
                    .and_then(|v| v.as_bool())
                {
                    return b;
                }
            }
        }
        false
    }

    /// Headroom fraction: proactive compaction runs when estimated tokens exceed
    /// `(1 - ratio) * token_budget` where `token_budget` is model context minus safety margin.
    /// Default **0.12** (~88% fill). Clamped to **0.05..0.45**.
    /// Config: `proactiveContextBudgetHeadroomRatio`; env: `MAC_STATS_PROACTIVE_CTX_HEADROOM_RATIO`.
    pub fn proactive_context_budget_headroom_ratio() -> f64 {
        if let Ok(s) = std::env::var("MAC_STATS_PROACTIVE_CTX_HEADROOM_RATIO") {
            if let Ok(v) = s.parse::<f64>() {
                return v.clamp(0.05, 0.45);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(v) = json
                    .get("proactiveContextBudgetHeadroomRatio")
                    .and_then(|v| v.as_f64())
                {
                    return v.clamp(0.05, 0.45);
                }
            }
        }
        0.12
    }

    /// Max characters per tool-result body after one proactive compaction step. If unset in
    /// config, uses [`Self::context_overflow_max_result_chars`].
    /// Config: `proactiveContextMaxResultChars`; env: `MAC_STATS_PROACTIVE_CTX_MAX_RESULT_CHARS`.
    pub fn proactive_context_max_result_chars() -> usize {
        if let Ok(s) = std::env::var("MAC_STATS_PROACTIVE_CTX_MAX_RESULT_CHARS") {
            if let Ok(v) = s.parse::<usize>() {
                return v.max(256);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(v) = json
                    .get("proactiveContextMaxResultChars")
                    .and_then(|v| v.as_u64())
                {
                    return (v as usize).max(256);
                }
            }
        }
        Self::context_overflow_max_result_chars()
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

    /// Delete screenshot files whose filename starts with `YYYYMMDD_HHMMSS` when that timestamp is older than this many days.
    /// **`0` disables** age-based pruning. Default: **7**. Config: `config.json` `screenshotPruneMaxAgeDays`.
    /// Env: `MAC_STATS_SCREENSHOT_PRUNE_MAX_AGE_DAYS` (clamped to `0..=3650`).
    pub fn screenshot_prune_max_age_days() -> u32 {
        const DEFAULT: u32 = 7;
        const MAX_DAYS: u32 = 3650;
        if let Ok(s) = std::env::var("MAC_STATS_SCREENSHOT_PRUNE_MAX_AGE_DAYS") {
            if let Ok(n) = s.parse::<u32>() {
                return n.min(MAX_DAYS);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json.get("screenshotPruneMaxAgeDays").and_then(|v| v.as_u64()) {
                    return (n as u32).min(MAX_DAYS);
                }
            }
        }
        DEFAULT
    }

    /// When total size of `screenshots_dir` exceeds this many bytes, delete oldest files first (by filename timestamp, else mtime) until under the cap.
    /// **`0` disables** the size cap. Default: **524288000** (500 MiB). Config: `config.json` `screenshotPruneMaxTotalBytes`.
    /// Env: `MAC_STATS_SCREENSHOT_PRUNE_MAX_TOTAL_BYTES`.
    pub fn screenshot_prune_max_total_bytes() -> u64 {
        const DEFAULT: u64 = 500 * 1024 * 1024;
        if let Ok(s) = std::env::var("MAC_STATS_SCREENSHOT_PRUNE_MAX_TOTAL_BYTES") {
            if let Ok(n) = s.parse::<u64>() {
                return n;
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json.get("screenshotPruneMaxTotalBytes").and_then(|v| v.as_u64()) {
                    return n;
                }
            }
        }
        DEFAULT
    }

    /// Read-only file roots for **BROWSER_UPLOAD** (model-supplied paths must resolve under here or [`Self::screenshots_dir`]): `$HOME/.mac-stats/uploads/`
    pub fn browser_uploads_dir() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".mac-stats").join("uploads")
        } else {
            std::env::temp_dir().join("mac-stats-uploads")
        }
    }

    /// Persisted browser cookie jar (CDP `Network.getAllCookies` snapshot): `$HOME/.mac-stats/browser_storage_state.json`
    pub fn browser_storage_state_json_path() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
                .join(".mac-stats")
                .join("browser_storage_state.json")
        } else {
            std::env::temp_dir().join("mac-stats-browser_storage_state.json")
        }
    }

    /// Domain-scoped secrets for `BROWSER_INPUT` `<secret>name</secret>` substitution: `$HOME/.mac-stats/browser-credentials.toml`
    pub fn browser_credentials_toml_path() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
                .join(".mac-stats")
                .join("browser-credentials.toml")
        } else {
            std::env::temp_dir().join("mac-stats-browser-credentials.toml")
        }
    }

    /// PDF exports directory (outbound Discord attachments when PDF tools write here): `$HOME/.mac-stats/pdfs/`
    pub fn pdfs_dir() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".mac-stats").join("pdfs")
        } else {
            std::env::temp_dir().join("mac-stats-pdfs")
        }
    }

    /// CDP trace archives (`Tracing.start` / `ReturnAsStream` JSON): `$HOME/.mac-stats/traces/`
    pub fn browser_cdp_traces_dir() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".mac-stats").join("traces")
        } else {
            std::env::temp_dir().join("mac-stats-traces")
        }
    }

    /// When **true**, mac-stats records a Chrome DevTools trace over the CDP WebSocket for each cached browser session
    /// (from first successful `BROWSER_*` CDP use until idle timeout, explicit session clear, or app shutdown).
    /// Default **false** (no extra WebSocket, no `Tracing` overhead). Config: **`browserCdpTraceEnabled`**.
    /// Env: **`MAC_STATS_BROWSER_CDP_TRACE_ENABLED`** (`true`/`1`/`yes` / `false`/`0`/`no`).
    pub fn browser_cdp_trace_enabled() -> bool {
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_TRACE_ENABLED") {
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
                if let Some(b) = json
                    .get("browserCdpTraceEnabled")
                    .and_then(|v| v.as_bool())
                {
                    return b;
                }
            }
        }
        false
    }

    /// Wall-clock cap from the **first** `Tracing.start` in this process (minutes). **`0`** = no time limit while enabled.
    /// Config: **`browserCdpTraceWallClockMinutes`**. Env: **`MAC_STATS_BROWSER_CDP_TRACE_MINUTES`** (clamped **0..=10080**).
    pub fn browser_cdp_trace_wall_clock_minutes() -> u64 {
        const MAX_MIN: u64 = 10080;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_TRACE_MINUTES") {
            if let Ok(n) = s.parse::<u64>() {
                return n.min(MAX_MIN);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("browserCdpTraceWallClockMinutes")
                    .and_then(|v| v.as_u64())
                {
                    return n.min(MAX_MIN);
                }
            }
        }
        0
    }

    /// Max bytes read from the CDP trace stream for one session file. Config: **`browserCdpTraceMaxFileBytes`**.
    /// Default **52428800** (50 MiB). Clamped **1048576..=524288000** (1 MiB–500 MiB).
    pub fn browser_cdp_trace_max_file_bytes() -> u64 {
        const DEFAULT: u64 = 50 * 1024 * 1024;
        const MIN: u64 = 1024 * 1024;
        const MAX: u64 = 500 * 1024 * 1024;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_TRACE_MAX_FILE_BYTES") {
            if let Ok(n) = s.parse::<u64>() {
                return n.clamp(MIN, MAX);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("browserCdpTraceMaxFileBytes")
                    .and_then(|v| v.as_u64())
                {
                    return n.clamp(MIN, MAX);
                }
            }
        }
        DEFAULT
    }

    /// Max `*_cdp_trace.json` files to keep under `browser_cdp_traces_dir`; oldest removed after each successful write.
    /// **`0`** disables pruning. Default **10**. Config: **`browserCdpTraceMaxRetainedFiles`**. Clamped **0..=500**.
    pub fn browser_cdp_trace_max_retained_files() -> usize {
        const DEFAULT: usize = 10;
        const MAX: usize = 500;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_TRACE_MAX_RETAINED_FILES") {
            if let Ok(n) = s.parse::<usize>() {
                return n.min(MAX);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("browserCdpTraceMaxRetainedFiles")
                    .and_then(|v| v.as_u64())
                {
                    return (n as usize).min(MAX);
                }
            }
        }
        DEFAULT
    }

    /// Browser download artifacts directory (reserved for outbound attachments): `$HOME/.mac-stats/browser-downloads/`
    pub fn browser_downloads_dir() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
                .join(".mac-stats")
                .join("browser-downloads")
        } else {
            std::env::temp_dir().join("mac-stats-browser-downloads")
        }
    }

    /// Extra directory roots allowed for outbound attachments (Discord, etc.), from `config.json` **`extraAttachmentRoots`**.
    ///
    /// Each entry is a path string: absolute, or `~/…`, or relative to `$HOME`. After canonicalization, the directory must lie under canonical `$HOME/.mac-stats` or under canonical `$HOME`; otherwise it is skipped with a log line.
    pub fn extra_attachment_roots() -> Vec<PathBuf> {
        let config_path = Self::config_file_path();
        let Ok(content) = std::fs::read_to_string(&config_path) else {
            return Vec::new();
        };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
            return Vec::new();
        };
        let Some(arr) = json
            .get("extraAttachmentRoots")
            .and_then(|v| v.as_array())
        else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for v in arr {
            let Some(s) = v.as_str() else {
                continue;
            };
            let p = Self::resolve_user_config_path(s);
            out.push(p);
        }
        out
    }

    /// Resolve a path from config: `~/` → `$HOME/…`; relative → `$HOME/<path>`; absolute unchanged.
    fn resolve_user_config_path(raw: &str) -> PathBuf {
        let t = raw.trim();
        if let Some(rest) = t.strip_prefix("~/") {
            if let Ok(home) = std::env::var("HOME") {
                return PathBuf::from(home).join(rest);
            }
        }
        let p = PathBuf::from(t);
        if p.is_absolute() {
            p
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(t)
        } else {
            p
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

    /// TCP port on loopback for Chromium remote debugging (CDP HTTP `/json/version` and visible auto-launch).
    ///
    /// Default **9222**. Config: `browserCdpPort`; env: `MAC_STATS_BROWSER_CDP_PORT`. Clamped to **1024–65535**.
    /// When attaching to a browser you started yourself, use the same port here and start the browser with
    /// `--remote-debugging-port=<port>`.
    pub fn browser_cdp_port() -> u16 {
        const DEFAULT: u16 = 9222;
        const MIN: u16 = 1024;
        const MAX: u16 = 65535;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_PORT") {
            if let Ok(n) = s.trim().parse::<u16>() {
                return n.clamp(MIN, MAX);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json.get("browserCdpPort").and_then(|v| v.as_u64()) {
                    if n <= u64::from(u16::MAX) {
                        return (n as u16).clamp(MIN, MAX);
                    }
                }
            }
        }
        DEFAULT
    }

    /// Max wall-clock time to poll loopback **`/json/version`** after mac-stats spawns visible Chromium (default **15** seconds).
    ///
    /// Each probe uses a short HTTP timeout in the browser agent; this value bounds total wait when startup is slow.
    /// Config: `browserCdpPostLaunchMaxWaitSecs`; env: `MAC_STATS_BROWSER_CDP_POST_LAUNCH_MAX_WAIT_SECS`. Clamped to **3–120**.
    pub fn browser_cdp_post_launch_max_wait_secs() -> u64 {
        const DEFAULT: u64 = 15;
        const MIN: u64 = 3;
        const MAX: u64 = 120;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_POST_LAUNCH_MAX_WAIT_SECS") {
            if let Ok(n) = s.trim().parse::<u64>() {
                return n.clamp(MIN, MAX);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("browserCdpPostLaunchMaxWaitSecs")
                    .and_then(|v| v.as_u64())
                {
                    return n.clamp(MIN, MAX);
                }
            }
        }
        DEFAULT
    }

    /// Pause between failed post-launch CDP HTTP probes (default **250** ms).
    ///
    /// Config: `browserCdpPostLaunchPollIntervalMs`; env: `MAC_STATS_BROWSER_CDP_POST_LAUNCH_POLL_MS`. Clamped to **50–2000**.
    pub fn browser_cdp_post_launch_poll_interval_ms() -> u64 {
        const DEFAULT: u64 = 250;
        const MIN: u64 = 50;
        const MAX: u64 = 2000;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_POST_LAUNCH_POLL_MS") {
            if let Ok(n) = s.trim().parse::<u64>() {
                return n.clamp(MIN, MAX);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("browserCdpPostLaunchPollIntervalMs")
                    .and_then(|v| v.as_u64())
                {
                    return n.clamp(MIN, MAX);
                }
            }
        }
        DEFAULT
    }

    /// Per-request HTTP timeout in seconds for CDP discovery (`GET /json/version` and equivalent probes).
    ///
    /// Default **5** (matches the historical `get_ws_url` single-shot timeout). Config: **`browserCdpHttpTimeoutSecs`**;
    /// env: **`MAC_STATS_BROWSER_CDP_HTTP_TIMEOUT_SECS`**. Clamped to **1–60** (minimum avoids unusable sub-second probes; maximum avoids accidental long hangs).
    pub fn browser_cdp_http_timeout_secs() -> u64 {
        const DEFAULT: u64 = 5;
        const MIN: u64 = 1;
        const MAX: u64 = 60;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_HTTP_TIMEOUT_SECS") {
            if let Ok(n) = s.trim().parse::<u64>() {
                return n.clamp(MIN, MAX);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("browserCdpHttpTimeoutSecs")
                    .and_then(|v| v.as_u64())
                {
                    return n.clamp(MIN, MAX);
                }
            }
        }
        DEFAULT
    }

    /// WebSocket connect handshake timeout in seconds for attaching to Chromium CDP (`Browser::connect_with_timeout`).
    ///
    /// Default **60**. Config: **`browserCdpWsConnectTimeoutSecs`**; env: **`MAC_STATS_BROWSER_CDP_WS_CONNECT_TIMEOUT_SECS`**.
    /// Clamped to **5–120**.
    pub fn browser_cdp_ws_connect_timeout_secs() -> u64 {
        const DEFAULT: u64 = 60;
        const MIN: u64 = 5;
        const MAX: u64 = 120;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_WS_CONNECT_TIMEOUT_SECS") {
            if let Ok(n) = s.trim().parse::<u64>() {
                return n.clamp(MIN, MAX);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("browserCdpWsConnectTimeoutSecs")
                    .and_then(|v| v.as_u64())
                {
                    return n.clamp(MIN, MAX);
                }
            }
        }
        DEFAULT
    }

    /// CDP [`Browser.grantPermissions`](https://chromedevtools.github.io/devtools-protocol/tot/Browser/#method-grantPermissions)
    /// permission names to apply **once** when mac-stats opens a **new** CDP browser session (not on tab switch or session reuse).
    ///
    /// Default **empty** (no behaviour change). Each entry must be a Chromium **`Browser.PermissionType`** string, for example:
    /// `geolocation`, `notifications`, `clipboardReadWrite`, `clipboardSanitizedWrite`, `durableStorage`, `idleDetection`, …
    /// (see the protocol / DevTools schema; typos are skipped with a warning). **Microphone/camera-class** permissions such as
    /// `audioCapture` and `videoCapture` are **not** implied—list them explicitly only if you intend to grant them. Failures are
    /// logged at **warn** and do not abort the session. Config: **`browserCdpGrantPermissions`** (JSON array of strings).
    /// Env: **`MAC_STATS_BROWSER_CDP_GRANT_PERMISSIONS`** — comma-separated list (optional; replaces config array when set).
    pub fn browser_cdp_grant_permissions() -> Vec<String> {
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_GRANT_PERMISSIONS") {
            let v: Vec<String> = s
                .split(',')
                .map(|x| x.trim().to_string())
                .filter(|x| !x.is_empty())
                .collect();
            if !v.is_empty() {
                return v;
            }
        }
        let config_path = Self::config_file_path();
        let Ok(content) = std::fs::read_to_string(&config_path) else {
            return Vec::new();
        };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
            return Vec::new();
        };
        let Some(arr) = json
            .get("browserCdpGrantPermissions")
            .and_then(|v| v.as_array())
        else {
            return Vec::new();
        };
        arr.iter()
            .filter_map(|v| v.as_str().map(str::trim).filter(|s| !s.is_empty()).map(str::to_string))
            .collect()
    }

    /// Default visible Chromium path when `browserChromiumExecutable` is unset (Google Chrome on macOS, `google-chrome` on Linux).
    #[cfg(target_os = "macos")]
    pub fn default_browser_chromium_executable() -> PathBuf {
        PathBuf::from("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome")
    }

    /// Default visible Chromium path when `browserChromiumExecutable` is unset (Google Chrome on macOS, `google-chrome` on Linux).
    #[cfg(not(target_os = "macos"))]
    pub fn default_browser_chromium_executable() -> PathBuf {
        PathBuf::from("google-chrome")
    }

    fn browser_chromium_executable_raw_opt() -> Option<String> {
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CHROMIUM_EXECUTABLE") {
            let t = s.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(s) = json
                    .get("browserChromiumExecutable")
                    .and_then(|v| v.as_str())
                {
                    let t = s.trim();
                    if !t.is_empty() {
                        return Some(t.to_string());
                    }
                }
            }
        }
        None
    }

    /// True when **`browserChromiumExecutable`** or **`MAC_STATS_BROWSER_CHROMIUM_EXECUTABLE`** is set (non-empty).
    /// Used to avoid silently falling back to a different Chromium binary after a failed launch.
    pub fn browser_chromium_executable_configured() -> bool {
        Self::browser_chromium_executable_raw_opt().is_some()
    }

    /// Resolved Chromium executable for visible CDP launch: configured path (with `~/` expansion) or platform default.
    pub fn browser_chromium_executable_path() -> PathBuf {
        match Self::browser_chromium_executable_raw_opt() {
            Some(s) => Self::resolve_user_config_path(&s),
            None => Self::default_browser_chromium_executable(),
        }
    }

    /// Optional Chromium user-data directory for visible launches (and headless when set). `~/` is expanded like other config paths.
    ///
    /// Config: `browserChromiumUserDataDir`; env: `MAC_STATS_BROWSER_CHROMIUM_USER_DATA_DIR`.
    pub fn browser_chromium_user_data_dir() -> Option<PathBuf> {
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CHROMIUM_USER_DATA_DIR") {
            let t = s.trim();
            if !t.is_empty() {
                return Some(Self::resolve_user_config_path(t));
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(s) = json
                    .get("browserChromiumUserDataDir")
                    .and_then(|v| v.as_str())
                {
                    let t = s.trim();
                    if !t.is_empty() {
                        return Some(Self::resolve_user_config_path(t));
                    }
                }
            }
        }
        None
    }

    /// Optional proxy username for **CDP Chrome only** (`Fetch.authRequired` when the challenge source is **Proxy**).
    ///
    /// No effect unless [`Self::browser_cdp_proxy_password`] is also a non-empty string. Not applied to mac-stats’
    /// own `reqwest` / **FETCH_URL** client. Config: `browserCdpProxyUsername`. Env: `MAC_STATS_BROWSER_CDP_PROXY_USERNAME`.
    pub fn browser_cdp_proxy_username() -> Option<String> {
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_PROXY_USERNAME") {
            let t = s.trim();
            if t.is_empty() {
                return None;
            }
            return Some(t.to_string());
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(s) = json
                    .get("browserCdpProxyUsername")
                    .and_then(|v| v.as_str())
                {
                    let t = s.trim();
                    if !t.is_empty() {
                        return Some(t.to_string());
                    }
                }
            }
        }
        None
    }

    /// Optional proxy password for **CDP Chrome only** (pairs with [`Self::browser_cdp_proxy_username`]).
    ///
    /// Config: `browserCdpProxyPassword`. Env: `MAC_STATS_BROWSER_CDP_PROXY_PASSWORD`. Values are never logged.
    pub fn browser_cdp_proxy_password() -> Option<String> {
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_PROXY_PASSWORD") {
            let t = s.trim();
            if t.is_empty() {
                return None;
            }
            return Some(t.to_string());
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(s) = json
                    .get("browserCdpProxyPassword")
                    .and_then(|v| v.as_str())
                {
                    let t = s.trim();
                    if !t.is_empty() {
                        return Some(t.to_string());
                    }
                }
            }
        }
        None
    }

    /// True when both CDP proxy username and password are configured with non-empty values.
    pub fn browser_cdp_proxy_credentials_active() -> bool {
        match (Self::browser_cdp_proxy_username(), Self::browser_cdp_proxy_password()) {
            (Some(u), Some(p)) => !u.trim().is_empty() && !p.trim().is_empty(),
            _ => false,
        }
    }

    /// Master enable/disable switch for all browser automation tools (BROWSER_*).
    ///
    /// When set to `false`, the app refuses BROWSER_* tool calls (no Chrome/CDP launch and no HTTP fallback).
    /// Default: `true`.
    /// Config: config.json `browserToolsEnabled` (boolean).
    pub fn browser_tools_enabled() -> bool {
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(b) = json.get("browserToolsEnabled").and_then(|v| v.as_bool()) {
                    return b;
                }
            }
        }
        true
    }

    /// Enable/disable **RUN_JS** (model-triggered JavaScript executed on the host via Node.js).
    ///
    /// When `false`, the tool loop returns a stable refusal without spawning Node. Orthogonal to
    /// **BROWSER_*** tools (`browserToolsEnabled`). Default: `true`.
    /// Config: `config.json` **`runJsEnabled`** (boolean).
    pub fn run_js_enabled() -> bool {
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(b) = json.get("runJsEnabled").and_then(|v| v.as_bool()) {
                    return b;
                }
            }
        }
        true
    }

    /// Pause (seconds) after each browser tool completes and before the next `BROWSER_*` tool runs
    /// in the same agent request, so the page can settle (DOM, network, animations). The first
    /// browser tool in a request has no leading wait. Non-browser tools (e.g. FETCH_URL) are unaffected.
    ///
    /// Default: `0` (no extra delay). Config: `browserWaitBetweenActionsSecs` (number, fractional OK).
    /// Env: `MAC_STATS_BROWSER_WAIT_BETWEEN_ACTIONS_SECS`. Clamped to `0.0..=60.0`.
    pub fn browser_wait_between_actions_secs() -> f64 {
        const DEFAULT: f64 = 0.0;
        const MAX: f64 = 60.0;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_WAIT_BETWEEN_ACTIONS_SECS") {
            if let Ok(n) = s.trim().parse::<f64>() {
                if n.is_finite() {
                    return n.clamp(0.0, MAX);
                }
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(v) = json.get("browserWaitBetweenActionsSecs") {
                    let n = v
                        .as_f64()
                        .or_else(|| v.as_u64().map(|u| u as f64))
                        .or_else(|| v.as_i64().map(|i| i as f64));
                    if let Some(n) = n {
                        if n.is_finite() {
                            return n.clamp(0.0, MAX);
                        }
                    }
                }
            }
        }
        DEFAULT
    }

    /// Minimum time (seconds) to wait after `wait_until_navigated` completes on the CDP path,
    /// before returning page state (in addition to optional network-idle wait). Mirrors browser-use
    /// `minimum_wait_page_load_time`; keeps a short default so SPAs can paint after the load event.
    ///
    /// Default **0.25** (≈ former fixed 250ms post-nav settle). Config: `browserPostNavigateMinDwellSecs`.
    /// Env: `MAC_STATS_BROWSER_POST_NAV_MIN_DWELL_SECS`. Clamped to `0.0..=10.0`.
    ///
    /// Applies uniformly (same-domain shorter **navigation timeout** does not skip this dwell).
    pub fn browser_post_navigate_min_dwell_secs() -> f64 {
        const DEFAULT: f64 = 0.25;
        const MAX: f64 = 10.0;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_POST_NAV_MIN_DWELL_SECS") {
            if let Ok(n) = s.trim().parse::<f64>() {
                if n.is_finite() {
                    return n.clamp(0.0, MAX);
                }
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(v) = json.get("browserPostNavigateMinDwellSecs") {
                    let n = v
                        .as_f64()
                        .or_else(|| v.as_u64().map(|u| u as f64))
                        .or_else(|| v.as_i64().map(|i| i as f64));
                    if let Some(n) = n {
                        if n.is_finite() {
                            return n.clamp(0.0, MAX);
                        }
                    }
                }
            }
        }
        DEFAULT
    }

    /// When true, after the minimum dwell mac-stats waits until **Network** CDP shows no in-flight
    /// requests for `browser_post_navigate_network_idle_quiet_secs`, capped by
    /// `browser_post_navigate_network_idle_max_extra_secs`. Default **false** (latency-preserving).
    ///
    /// Config: `browserPostNavigateNetworkIdleEnabled`. Env:
    /// `MAC_STATS_BROWSER_POST_NAV_NETWORK_IDLE_ENABLED` (`1`/`true`/`yes` / `0`/`false`/`no`).
    pub fn browser_post_navigate_network_idle_enabled() -> bool {
        const DEFAULT: bool = false;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_POST_NAV_NETWORK_IDLE_ENABLED") {
            let lower = s.to_lowercase();
            if matches!(lower.as_str(), "1" | "true" | "yes" | "on") {
                return true;
            }
            if matches!(lower.as_str(), "0" | "false" | "no" | "off") {
                return false;
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(b) = json
                    .get("browserPostNavigateNetworkIdleEnabled")
                    .and_then(|v| v.as_bool())
                {
                    return b;
                }
            }
        }
        DEFAULT
    }

    /// Required quiet period (seconds) with **zero** in-flight Network requests before navigation
    /// stabilization completes. Config: `browserPostNavigateNetworkIdleQuietSecs`. Env:
    /// `MAC_STATS_BROWSER_POST_NAV_NETWORK_IDLE_QUIET_SECS`. Default **0.5**, clamped to `0.0..=10.0`.
    pub fn browser_post_navigate_network_idle_quiet_secs() -> f64 {
        const DEFAULT: f64 = 0.5;
        const MAX: f64 = 10.0;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_POST_NAV_NETWORK_IDLE_QUIET_SECS") {
            if let Ok(n) = s.trim().parse::<f64>() {
                if n.is_finite() {
                    return n.clamp(0.0, MAX);
                }
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(v) = json.get("browserPostNavigateNetworkIdleQuietSecs") {
                    let n = v
                        .as_f64()
                        .or_else(|| v.as_u64().map(|u| u as f64))
                        .or_else(|| v.as_i64().map(|i| i as f64));
                    if let Some(n) = n {
                        if n.is_finite() {
                            return n.clamp(0.0, MAX);
                        }
                    }
                }
            }
        }
        DEFAULT
    }

    /// Hard cap (seconds) for the optional post-navigate **network in-flight** wait. When elapsed,
    /// stabilization proceeds even if requests remain (logged at debug). Config:
    /// `browserPostNavigateNetworkIdleMaxExtraSecs`. Env:
    /// `MAC_STATS_BROWSER_POST_NAV_NETWORK_IDLE_MAX_EXTRA_SECS`. Default **5.0**, clamped to `0.0..=120.0`.
    /// **0** disables the network-idle phase entirely (min dwell still applies).
    pub fn browser_post_navigate_network_idle_max_extra_secs() -> f64 {
        const DEFAULT: f64 = 5.0;
        const MAX: f64 = 120.0;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_POST_NAV_NETWORK_IDLE_MAX_EXTRA_SECS") {
            if let Ok(n) = s.trim().parse::<f64>() {
                if n.is_finite() {
                    return n.clamp(0.0, MAX);
                }
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(v) = json.get("browserPostNavigateNetworkIdleMaxExtraSecs") {
                    let n = v
                        .as_f64()
                        .or_else(|| v.as_u64().map(|u| u as f64))
                        .or_else(|| v.as_i64().map(|i| i as f64));
                    if let Some(n) = n {
                        if n.is_finite() {
                            return n.clamp(0.0, MAX);
                        }
                    }
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

    /// Maximum open CDP **page** tabs before best-effort pruning (OpenClaw-style tab cap). **0** disables (default).
    /// When the count exceeds this value, the automation session closes older non-focused tabs until within the cap.
    /// Config: `config.json` key `browserMaxPageTabs`. Env: `MAC_STATS_BROWSER_MAX_PAGE_TABS`. Clamped to 0..=64.
    pub fn browser_max_page_tabs() -> usize {
        const DEFAULT: usize = 0;
        const MAX: usize = 64;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_MAX_PAGE_TABS") {
            if let Ok(n) = s.parse::<usize>() {
                return n.min(MAX);
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("browserMaxPageTabs")
                    .and_then(|v| v.as_u64())
                {
                    return (n as usize).min(MAX);
                }
            }
        }
        DEFAULT
    }

    /// Whether to include bounded CDP console and page-level JavaScript error diagnostics
    /// in `BROWSER_NAVIGATE` tool results (and optionally other browser tool results).
    ///
    /// Off by default to preserve existing context size and tool output stability.
    /// Config: config.json `browserIncludeDiagnosticsInState` (boolean);
    /// Env: `MAC_STATS_BROWSER_INCLUDE_DIAGNOSTICS_IN_STATE` (true/1/yes or false/0/no).
    pub fn browser_include_diagnostics_in_state() -> bool {
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_INCLUDE_DIAGNOSTICS_IN_STATE") {
            let lower = s.to_lowercase();
            return matches!(lower.as_str(), "1" | "true" | "yes" | "on");
        }

        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(b) = json
                    .get("browserIncludeDiagnosticsInState")
                    .and_then(|v| v.as_bool())
                {
                    return b;
                }
            }
        }

        false
    }

    /// After `BROWSER_NAVIGATE`, detect skeleton/blank pages (sparse text vs. many DOM nodes) and retry
    /// with extra waits plus a full reload (browser-use style). Baseline post-navigate sleep is unchanged.
    /// Disable for fast static pages or tests. Config: `browserSpaRetryEnabled` (boolean).
    /// Env: `MAC_STATS_BROWSER_SPA_RETRY_ENABLED` (true/1/yes or false/0/no). Default **true**.
    pub fn browser_spa_retry_enabled() -> bool {
        const DEFAULT: bool = true;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_SPA_RETRY_ENABLED") {
            let lower = s.to_lowercase();
            if matches!(lower.as_str(), "1" | "true" | "yes" | "on") {
                return true;
            }
            if matches!(lower.as_str(), "0" | "false" | "no" | "off") {
                return false;
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(b) = json
                    .get("browserSpaRetryEnabled")
                    .and_then(|v| v.as_bool())
                {
                    return b;
                }
            }
        }
        DEFAULT
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

    /// Optional CDP `Emulation.setDeviceMetricsOverride` width/height (CSS pixels). **Both** `browserCdpEmulateViewportWidth`
    /// and **`browserCdpEmulateViewportHeight`** must be set in `config.json` (or both env vars) to enable; if only one is set,
    /// a warning is logged and emulation is disabled. Clamped to 200..=3840 × 200..=2160.
    /// Env: **`MAC_STATS_BROWSER_CDP_EMULATE_VIEWPORT_WIDTH`**, **`MAC_STATS_BROWSER_CDP_EMULATE_VIEWPORT_HEIGHT`** (non-empty parses as u32).
    pub fn browser_cdp_emulate_viewport_dimensions() -> Option<(u32, u32)> {
        const MIN_W: u32 = 200;
        const MAX_W: u32 = 3840;
        const MIN_H: u32 = 200;
        const MAX_H: u32 = 2160;
        let w_env = std::env::var("MAC_STATS_BROWSER_CDP_EMULATE_VIEWPORT_WIDTH")
            .ok()
            .and_then(|s| {
                let t = s.trim();
                if t.is_empty() {
                    None
                } else {
                    t.parse::<u32>().ok()
                }
            });
        let h_env = std::env::var("MAC_STATS_BROWSER_CDP_EMULATE_VIEWPORT_HEIGHT")
            .ok()
            .and_then(|s| {
                let t = s.trim();
                if t.is_empty() {
                    None
                } else {
                    t.parse::<u32>().ok()
                }
            });
        let config_path = Self::config_file_path();
        let (w_cfg, h_cfg) = if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let w = json
                    .get("browserCdpEmulateViewportWidth")
                    .and_then(|v| {
                        if v.is_null() {
                            None
                        } else {
                            v.as_u64().map(|n| n as u32)
                        }
                    });
                let h = json
                    .get("browserCdpEmulateViewportHeight")
                    .and_then(|v| {
                        if v.is_null() {
                            None
                        } else {
                            v.as_u64().map(|n| n as u32)
                        }
                    });
                (w, h)
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };
        let w = w_env.or(w_cfg);
        let h = h_env.or(h_cfg);
        match (w, h) {
            (Some(w), Some(h)) => Some((w.clamp(MIN_W, MAX_W), h.clamp(MIN_H, MAX_H))),
            (None, None) => None,
            _ => {
                tracing::warn!(
                    "config: browserCdpEmulateViewportWidth and browserCdpEmulateViewportHeight must both be set to enable CDP device-metrics emulation; ignoring partial config"
                );
                None
            }
        }
    }

    /// Device scale factor for [`Self::browser_cdp_emulate_viewport_dimensions`] when enabled. Default **1.0**. Clamped 0.1..=10.0.
    /// Config: **`browserCdpEmulateDeviceScaleFactor`**. Env: **`MAC_STATS_BROWSER_CDP_EMULATE_DEVICE_SCALE_FACTOR`**.
    pub fn browser_cdp_emulate_device_scale_factor() -> f64 {
        const DEFAULT: f64 = 1.0;
        const MIN: f64 = 0.1;
        const MAX: f64 = 10.0;
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_EMULATE_DEVICE_SCALE_FACTOR") {
            let t = s.trim();
            if !t.is_empty() {
                if let Ok(f) = t.parse::<f64>() {
                    if f.is_finite() {
                        return f.clamp(MIN, MAX);
                    }
                }
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(f) = json
                    .get("browserCdpEmulateDeviceScaleFactor")
                    .and_then(|v| v.as_f64())
                {
                    if f.is_finite() {
                        return f.clamp(MIN, MAX);
                    }
                }
            }
        }
        DEFAULT
    }

    /// Mobile flag for CDP device-metrics emulation when [`Self::browser_cdp_emulate_viewport_dimensions`] is enabled. Default **false**.
    /// Config: **`browserCdpEmulateMobile`**. Env: **`MAC_STATS_BROWSER_CDP_EMULATE_MOBILE`** (`true`/`1`/`yes` / `false`/`0`/`no`).
    pub fn browser_cdp_emulate_mobile() -> bool {
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_CDP_EMULATE_MOBILE") {
            let lower = s.to_lowercase();
            if matches!(lower.as_str(), "1" | "true" | "yes" | "on") {
                return true;
            }
            if matches!(lower.as_str(), "0" | "false" | "no" | "off") {
                return false;
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(b) = json
                    .get("browserCdpEmulateMobile")
                    .and_then(|v| v.as_bool())
                {
                    return b;
                }
            }
        }
        false
    }

    /// Optional CDP `Emulation.setGeolocationOverride`: **both** latitude and longitude must be set (config or env), or geolocation emulation is off
    /// (`Emulation.clearGeolocationOverride`). Optional accuracy (meters). Lat clamped −90..=90, lon −180..=180, accuracy 0.1..=100_000 when set.
    /// Config: **`browserCdpEmulateGeolocationLatitude`**, **`browserCdpEmulateGeolocationLongitude`**, optional **`browserCdpEmulateGeolocationAccuracy`**.
    /// Env: **`MAC_STATS_BROWSER_CDP_EMULATE_GEO_LATITUDE`**, **`MAC_STATS_BROWSER_CDP_EMULATE_GEO_LONGITUDE`**, optional **`MAC_STATS_BROWSER_CDP_EMULATE_GEO_ACCURACY`**.
    pub fn browser_cdp_emulate_geolocation() -> Option<(f64, f64, Option<f64>)> {
        const MIN_LAT: f64 = -90.0;
        const MAX_LAT: f64 = 90.0;
        const MIN_LON: f64 = -180.0;
        const MAX_LON: f64 = 180.0;
        const MIN_ACC: f64 = 0.1;
        const MAX_ACC: f64 = 100_000.0;

        let lat_env = std::env::var("MAC_STATS_BROWSER_CDP_EMULATE_GEO_LATITUDE")
            .ok()
            .and_then(|s| {
                let t = s.trim();
                if t.is_empty() {
                    None
                } else {
                    t.parse::<f64>().ok()
                }
            });
        let lon_env = std::env::var("MAC_STATS_BROWSER_CDP_EMULATE_GEO_LONGITUDE")
            .ok()
            .and_then(|s| {
                let t = s.trim();
                if t.is_empty() {
                    None
                } else {
                    t.parse::<f64>().ok()
                }
            });
        let acc_env = std::env::var("MAC_STATS_BROWSER_CDP_EMULATE_GEO_ACCURACY")
            .ok()
            .and_then(|s| {
                let t = s.trim();
                if t.is_empty() {
                    None
                } else {
                    t.parse::<f64>().ok()
                }
            });

        let config_path = Self::config_file_path();
        let (lat_cfg, lon_cfg, acc_cfg) = if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let lat = json
                    .get("browserCdpEmulateGeolocationLatitude")
                    .and_then(|v| {
                        if v.is_null() {
                            None
                        } else {
                            v.as_f64()
                        }
                    });
                let lon = json
                    .get("browserCdpEmulateGeolocationLongitude")
                    .and_then(|v| {
                        if v.is_null() {
                            None
                        } else {
                            v.as_f64()
                        }
                    });
                let acc = json
                    .get("browserCdpEmulateGeolocationAccuracy")
                    .and_then(|v| {
                        if v.is_null() {
                            None
                        } else {
                            v.as_f64()
                        }
                    });
                (lat, lon, acc)
            } else {
                (None, None, None)
            }
        } else {
            (None, None, None)
        };

        let lat = lat_env.or(lat_cfg);
        let lon = lon_env.or(lon_cfg);
        let acc = acc_env.or(acc_cfg);

        match (lat, lon) {
            (Some(la), Some(lo)) => {
                if !la.is_finite() || !lo.is_finite() {
                    tracing::warn!(
                        "config: browserCdpEmulateGeolocationLatitude/Longitude must be finite; clearing geolocation emulation"
                    );
                    return None;
                }
                let la = la.clamp(MIN_LAT, MAX_LAT);
                let lo = lo.clamp(MIN_LON, MAX_LON);
                let acc_opt = acc.filter(|a| a.is_finite()).map(|a| a.clamp(MIN_ACC, MAX_ACC));
                Some((la, lo, acc_opt))
            }
            (None, None) => None,
            _ => {
                tracing::warn!(
                    "config: browserCdpEmulateGeolocationLatitude and browserCdpEmulateGeolocationLongitude must both be set to enable CDP geolocation emulation; ignoring partial config"
                );
                None
            }
        }
    }

    /// Maximum size in bytes for one browser-produced artifact: raw/annotated PNG from **BROWSER_SCREENSHOT**,
    /// PDF bytes from **BROWSER_SAVE_PDF** / **Page.printToPDF**, and the same limit when loading those files for Discord or vision.
    /// Config: `config.json` **`browserArtifactMaxBytes`**. Default **10485760** (10 MiB). Clamped to **65536**–**104857600** (64 KiB–100 MiB).
    pub fn browser_artifact_max_bytes() -> u64 {
        const DEFAULT: u64 = 10 * 1024 * 1024;
        const MIN: u64 = 64 * 1024;
        const MAX: u64 = 100 * 1024 * 1024;
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = json
                    .get("browserArtifactMaxBytes")
                    .and_then(|v| v.as_u64())
                {
                    return n.clamp(MIN, MAX);
                }
            }
        }
        DEFAULT
    }

    /// When **true**, **BROWSER_SAVE_PDF** passes **`printBackground: true`** to CDP `Page.printToPDF`.
    /// Default **false** (smaller files, text-forward). Config: **`browserPrintPdfBackground`** (boolean).
    /// Env override: **`MAC_STATS_BROWSER_PRINT_PDF_BACKGROUND`** (`true`/`1`/`yes` / `false`/`0`/`no`).
    pub fn browser_print_pdf_background() -> bool {
        if let Ok(s) = std::env::var("MAC_STATS_BROWSER_PRINT_PDF_BACKGROUND") {
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
                if let Some(b) = json
                    .get("browserPrintPdfBackground")
                    .and_then(|v| v.as_bool())
                {
                    return b;
                }
            }
        }
        false
    }

    /// Optional width/height for resizing images before they are sent to a vision LLM (e.g. completion verification).
    /// Both `browserLlmScreenshotWidth` and `browserLlmScreenshotHeight` must be set in `config.json`; each is
    /// clamped to 100..=8192. If either is missing, returns `None` (full-size image bytes are sent, base64-only).
    /// Does not change files on disk or Discord attachments.
    pub fn browser_llm_screenshot_size() -> Option<(u32, u32)> {
        const MIN: u32 = 100;
        const MAX: u32 = 8192;
        let config_path = Self::config_file_path();
        let Ok(content) = std::fs::read_to_string(&config_path) else {
            return None;
        };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
            return None;
        };
        let w = json
            .get("browserLlmScreenshotWidth")
            .and_then(|v| v.as_u64())
            .map(|n| (n as u32).clamp(MIN, MAX));
        let h = json
            .get("browserLlmScreenshotHeight")
            .and_then(|v| v.as_u64())
            .map(|n| (n as u32).clamp(MIN, MAX));
        match (w, h) {
            (Some(w), Some(h)) => Some((w, h)),
            (None, None) => None,
            _ => {
                tracing::warn!(
                    "config: browserLlmScreenshotWidth/Height must both be set to enable LLM screenshot resize; ignoring partial config"
                );
                None
            }
        }
    }

    /// Optional SSRF allowlist: hostnames that are permitted even when they resolve to private IPs.
    /// Config: config.json `ssrfAllowedHosts` (JSON array of strings). Each entry is matched against the
    /// **hostname only** (not the full URL): plain strings are case-insensitive **exact** hostname match
    /// (backward compatible); `*` / `**` enable shell-style globs (`*.app.internal`, `127.0.*`), anchored
    /// to the full hostname; `contains:` prefix enables a case-insensitive substring rule
    /// (e.g. `contains:.staging.`). Same list is used for FETCH_URL, redirect hops, and CDP navigation.
    /// Default: empty (no exceptions).
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

    /// When **true**, refuse **FETCH_URL** (reqwest) and CDP navigations (**BROWSER_NAVIGATE**,
    /// **BROWSER_SCREENSHOT** with a URL) if standard proxy environment variables are set
    /// (`HTTP_PROXY` / `HTTPS_PROXY` / `ALL_PROXY` and lowercase variants), because DNS-based
    /// SSRF pre-checks may not match actual egress through a proxy.
    ///
    /// **Default: false** — corporate VPN / dev-proxy setups keep working; operators who need strict
    /// parity between pre-check and egress should unset proxy vars for the mac-stats process or
    /// accept the documented limitation.
    ///
    /// Config: `~/.mac-stats/config.json` `strictSsrfRejectWhenProxyEnv` (boolean). Env override:
    /// `MAC_STATS_STRICT_SSRF_REJECT_WHEN_PROXY_ENV` (`true`/`1`/`yes` / `false`/`0`/`no`), same
    /// precedence as other boolean env toggles.
    pub fn strict_ssrf_reject_when_proxy_env() -> bool {
        if let Ok(s) = std::env::var("MAC_STATS_STRICT_SSRF_REJECT_WHEN_PROXY_ENV") {
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
                if let Some(b) = json
                    .get("strictSsrfRejectWhenProxyEnv")
                    .and_then(|v| v.as_bool())
                {
                    return b;
                }
            }
        }
        false
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

    /// Default JSONL path for optional **before-reset** export when a hook is set but no custom path is configured.
    pub fn default_before_reset_transcript_path() -> PathBuf {
        Self::agents_dir().join("last_session_before_reset.jsonl")
    }

    /// Raw path from env `MAC_STATS_BEFORE_RESET_TRANSCRIPT_PATH` or `config.json` **`beforeResetTranscriptPath`** (expand `~/`). Empty = disabled unless **`beforeResetHook`** is set (then default path is used).
    pub fn before_reset_transcript_path_raw() -> String {
        if let Ok(s) = std::env::var("MAC_STATS_BEFORE_RESET_TRANSCRIPT_PATH") {
            if !s.trim().is_empty() {
                return s;
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(v) = json
                    .get("beforeResetTranscriptPath")
                    .and_then(|v| v.as_str())
                {
                    if !v.trim().is_empty() {
                        return v.to_string();
                    }
                }
            }
        }
        String::new()
    }

    /// Optional shell command for **before-reset** hook (non-blocking). Env `MAC_STATS_BEFORE_RESET_HOOK` overrides `config.json` **`beforeResetHook`**.
    /// The transcript absolute path is passed as **`$1`**; also set as env **`MAC_STATS_BEFORE_RESET_TRANSCRIPT`**.
    pub fn before_reset_hook_raw() -> String {
        if let Ok(s) = std::env::var("MAC_STATS_BEFORE_RESET_HOOK") {
            if !s.trim().is_empty() {
                return s;
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(v) = json.get("beforeResetHook").and_then(|v| v.as_str()) {
                    if !v.trim().is_empty() {
                        return v.to_string();
                    }
                }
            }
        }
        String::new()
    }

    /// Expand `~` / `~/` in a path string. Returns `None` if non-empty but `HOME` is missing for a `~` path.
    pub fn expand_user_path_str(path_str: &str) -> Option<PathBuf> {
        let t = path_str.trim();
        if t.is_empty() {
            return None;
        }
        if let Some(rest) = t.strip_prefix("~/") {
            let home = std::env::var("HOME").ok()?;
            return Some(PathBuf::from(home).join(rest));
        }
        if t == "~" {
            let home = std::env::var("HOME").ok()?;
            return Some(PathBuf::from(home));
        }
        Some(PathBuf::from(t))
    }

    /// Resolved **`beforeResetTranscriptPath`** (config/env). `None` when unset or invalid `~` without `HOME`.
    pub fn before_reset_transcript_path_resolved() -> Option<PathBuf> {
        let raw = Self::before_reset_transcript_path_raw();
        if raw.trim().is_empty() {
            return None;
        }
        Self::expand_user_path_str(&raw)
    }

    /// Raw path for **before-compaction** JSONL transcript. Env `MAC_STATS_BEFORE_COMPACTION_TRANSCRIPT_PATH` overrides `config.json` **`beforeCompactionTranscriptPath`**.
    pub fn before_compaction_transcript_path_raw() -> String {
        if let Ok(s) = std::env::var("MAC_STATS_BEFORE_COMPACTION_TRANSCRIPT_PATH") {
            if !s.trim().is_empty() {
                return s;
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(v) = json
                    .get("beforeCompactionTranscriptPath")
                    .and_then(|v| v.as_str())
                {
                    if !v.trim().is_empty() {
                        return v.to_string();
                    }
                }
            }
        }
        String::new()
    }

    pub fn before_compaction_transcript_path_resolved() -> Option<PathBuf> {
        let raw = Self::before_compaction_transcript_path_raw();
        if raw.trim().is_empty() {
            return None;
        }
        Self::expand_user_path_str(&raw)
    }

    /// Optional shell for **before-compaction** (non-blocking). Env `MAC_STATS_BEFORE_COMPACTION_HOOK` overrides **`beforeCompactionHook`**. Transcript path is **`$1`**.
    pub fn before_compaction_hook_raw() -> String {
        if let Ok(s) = std::env::var("MAC_STATS_BEFORE_COMPACTION_HOOK") {
            if !s.trim().is_empty() {
                return s;
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(v) = json
                    .get("beforeCompactionHook")
                    .and_then(|v| v.as_str())
                {
                    if !v.trim().is_empty() {
                        return v.to_string();
                    }
                }
            }
        }
        String::new()
    }

    /// Optional shell run **after successful compaction** only. Env `MAC_STATS_AFTER_COMPACTION_HOOK` overrides **`afterCompactionHook`**. Uses env vars only (no `$1`).
    pub fn after_compaction_hook_raw() -> String {
        if let Ok(s) = std::env::var("MAC_STATS_AFTER_COMPACTION_HOOK") {
            if !s.trim().is_empty() {
                return s;
            }
        }
        let config_path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(v) = json.get("afterCompactionHook").and_then(|v| v.as_str()) {
                    if !v.trim().is_empty() {
                        return v.to_string();
                    }
                }
            }
        }
        String::new()
    }

    /// Parse boolean from `MAC_STATS_*` first, then optional `ORI_*` compatibility alias.
    fn ori_env_bool(mac_stats_key: &str, compat_key: &str) -> Option<bool> {
        for key in [mac_stats_key, compat_key] {
            if let Ok(s) = std::env::var(key) {
                let l = s.to_lowercase();
                if matches!(l.as_str(), "1" | "true" | "yes" | "on") {
                    return Some(true);
                }
                if matches!(l.as_str(), "0" | "false" | "no" | "off") {
                    return Some(false);
                }
            }
        }
        None
    }

    /// Master gate: no Ori subprocesses or prompt injection unless true.
    /// Env: `MAC_STATS_ORI_LIFECYCLE_ENABLED` or `ORI_LIFECYCLE_ENABLED`.
    pub fn ori_lifecycle_enabled() -> bool {
        Self::ori_env_bool("MAC_STATS_ORI_LIFECYCLE_ENABLED", "ORI_LIFECYCLE_ENABLED")
            .unwrap_or(false)
    }

    /// Vault root (must contain `.ori`). Env: `MAC_STATS_ORI_VAULT` or `ORI_VAULT`.
    pub fn ori_vault_path_raw() -> String {
        if let Ok(s) = std::env::var("MAC_STATS_ORI_VAULT") {
            if !s.trim().is_empty() {
                return s;
            }
        }
        std::env::var("ORI_VAULT").unwrap_or_default()
    }

    /// `ori` binary on PATH unless overridden. Env: `MAC_STATS_ORI_BINARY`.
    pub fn ori_binary() -> String {
        std::env::var("MAC_STATS_ORI_BINARY")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "ori".to_string())
    }

    /// Session-start briefing from vault markdown. Env: `MAC_STATS_ORI_HOOK_ORIENT` or `ORI_HOOK_ORIENT_ON_SESSION_START`.
    pub fn ori_hook_orient_on_session_start() -> bool {
        if !Self::ori_lifecycle_enabled() {
            return false;
        }
        Self::ori_env_bool("MAC_STATS_ORI_HOOK_ORIENT", "ORI_HOOK_ORIENT_ON_SESSION_START")
            .unwrap_or(false)
    }

    /// Push compaction lessons to Ori inbox. Env: `MAC_STATS_ORI_HOOK_CAPTURE_COMPACTION` or `ORI_HOOK_CAPTURE_ON_COMPACTION`.
    pub fn ori_hook_capture_on_compaction() -> bool {
        if !Self::ori_lifecycle_enabled() {
            return false;
        }
        Self::ori_env_bool(
            "MAC_STATS_ORI_HOOK_CAPTURE_COMPACTION",
            "ORI_HOOK_CAPTURE_ON_COMPACTION",
        )
        .unwrap_or(false)
    }

    /// Capture before reset. Env: `MAC_STATS_ORI_HOOK_BEFORE_RESET` or `ORI_HOOK_BEFORE_SESSION_RESET`.
    pub fn ori_hook_before_session_reset() -> bool {
        if !Self::ori_lifecycle_enabled() {
            return false;
        }
        Self::ori_env_bool(
            "MAC_STATS_ORI_HOOK_BEFORE_RESET",
            "ORI_HOOK_BEFORE_SESSION_RESET",
        )
        .unwrap_or(false)
    }

    /// Server-side prefetch. Env: `MAC_STATS_ORI_PREFETCH` or `ORI_PREFETCH_ENABLED`.
    pub fn ori_prefetch_enabled() -> bool {
        if !Self::ori_lifecycle_enabled() {
            return false;
        }
        Self::ori_env_bool("MAC_STATS_ORI_PREFETCH", "ORI_PREFETCH_ENABLED").unwrap_or(false)
    }

    /// When true (default), scheduler / heartbeat / task_runner skip orient + prefetch.
    /// Opt out with `MAC_STATS_ORI_ALLOW_ON_SCHEDULER=true`. Mirrors `ORI_SKIP_FOR_SCHEDULER` intent.
    pub fn ori_skip_for_scheduler() -> bool {
        if !Self::ori_lifecycle_enabled() {
            return true;
        }
        if Self::ori_env_bool("MAC_STATS_ORI_ALLOW_ON_SCHEDULER", "ORI_ALLOW_ON_SCHEDULER")
            == Some(true)
        {
            return false;
        }
        if Self::ori_env_bool("MAC_STATS_ORI_SKIP_FOR_SCHEDULER", "ORI_SKIP_FOR_SCHEDULER")
            == Some(false)
        {
            return false;
        }
        true
    }

    /// `off` | `excerpt_to_ori` | `full_lessons_duplicate`. Env: `MAC_STATS_ORI_COMPACTION_CAPTURE_MODE` or `ORI_COMPACTION_CAPTURE_MODE`.
    pub fn ori_compaction_capture_mode() -> String {
        if let Ok(s) = std::env::var("MAC_STATS_ORI_COMPACTION_CAPTURE_MODE") {
            let t = s.trim().to_lowercase();
            if !t.is_empty() {
                return t;
            }
        }
        if let Ok(s) = std::env::var("ORI_COMPACTION_CAPTURE_MODE") {
            let t = s.trim().to_lowercase();
            if !t.is_empty() {
                return t;
            }
        }
        "off".to_string()
    }

    pub fn ori_orient_max_chars() -> usize {
        Self::parse_usize_env("MAC_STATS_ORI_ORIENT_MAX_CHARS", 8000).clamp(500, 50_000)
    }

    pub fn ori_prefetch_max_chars() -> usize {
        Self::parse_usize_env("MAC_STATS_ORI_PREFETCH_MAX_CHARS", 6000).clamp(500, 50_000)
    }

    pub fn ori_prefetch_top_k() -> u32 {
        Self::parse_usize_env("MAC_STATS_ORI_PREFETCH_TOP_K", 5).clamp(1, 20) as u32
    }

    pub fn ori_prefetch_timeout_secs() -> u64 {
        Self::parse_usize_env("MAC_STATS_ORI_PREFETCH_TIMEOUT_SECS", 12).clamp(1, 60) as u64
    }

    pub fn ori_prefetch_cooldown_secs() -> u64 {
        Self::parse_usize_env("MAC_STATS_ORI_PREFETCH_COOLDOWN_SECS", 5).clamp(0, 3600) as u64
    }

    pub fn ori_reset_capture_max_chars() -> usize {
        Self::parse_usize_env("MAC_STATS_ORI_RESET_CAPTURE_MAX_CHARS", 12_000).clamp(2000, 100_000)
    }

    fn parse_usize_env(key: &str, default: usize) -> usize {
        std::env::var(key)
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(default)
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
