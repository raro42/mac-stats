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
//! **JSON config reload (no restart needed):**
//! - `config.json` — read on every access (window decorations, scheduler interval, ollamaChatTimeoutSecs).
//! - `schedules.json` — scheduler checks file mtime each loop and reloads when changed.
//! - `discord_channels.json` — Discord loop checks mtime every tick and reloads when changed.

use std::path::PathBuf;

/// Build one default-agent entry from an id. Add new agents by creating defaults/agents/agent-<id>/ and adding default_agent_entry!("<id>") to DEFAULT_AGENT_IDS.
macro_rules! default_agent_entry {
    ($id:literal) => {
        (
            concat!("agent-", $id),
            &[
                ("agent.json", include_str!(concat!("../../defaults/agents/agent-", $id, "/agent.json"))),
                ("skill.md", include_str!(concat!("../../defaults/agents/agent-", $id, "/skill.md"))),
                ("testing.md", include_str!(concat!("../../defaults/agents/agent-", $id, "/testing.md"))),
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
                ("agent.json", include_str!(concat!("../../defaults/agents/agent-", $id, "/agent.json"))),
                ("skill.md", include_str!(concat!("../../defaults/agents/agent-", $id, "/skill.md"))),
                ("soul.md", include_str!(concat!("../../defaults/agents/agent-", $id, "/soul.md"))),
                ("testing.md", include_str!(concat!("../../defaults/agents/agent-", $id, "/testing.md"))),
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
    
    /// Get the build date
    /// 
    /// Returns the build date from the BUILD_DATE environment variable,
    /// or "unknown" if not available.
    pub fn build_date() -> String {
        std::env::var("BUILD_DATE")
            .unwrap_or_else(|_| "unknown".to_string())
    }
    
    /// Get the version string
    /// 
    /// Returns the package version from CARGO_PKG_VERSION.
    pub fn version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
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
                if let Some(n) = json.get("schedulerCheckIntervalSecs").and_then(|v| v.as_u64()) {
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

    /// Skills directory for agent prompt overlays: `$HOME/.mac-stats/skills/`
    /// Files: skill-<number>-<topic>.md (e.g. skill-1-summarize.md, skill-2-code.md).
    pub fn skills_dir() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".mac-stats").join("skills")
        } else {
            std::env::temp_dir().join("mac-stats-skills")
        }
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

    /// Path to Discord channel config: `$HOME/.mac-stats/discord_channels.json`
    pub fn discord_channels_path() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".mac-stats").join("discord_channels.json")
        } else {
            std::env::temp_dir().join("mac-stats-discord_channels.json")
        }
    }

    /// Prompts directory: `$HOME/.mac-stats/prompts/`
    pub fn prompts_dir() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".mac-stats").join("prompts")
        } else {
            std::env::temp_dir().join("mac-stats-prompts")
        }
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
    const DEFAULT_EXECUTION_PROMPT: &str = include_str!("../../defaults/prompts/execution_prompt.md");

    /// List of default agents. Agents are read in a loop; add new ids here and add the files under defaults/agents/agent-<id>/.
    const DEFAULT_AGENT_IDS: &[(&str, &[(&str, &str)])] = &[
        default_agent_entry!("000"),
        default_agent_entry!("001"),
        default_agent_entry!("002"),
        default_agent_entry!("003"),
        default_agent_entry!("004"),
        default_agent_entry!("005"),
        default_agent_entry_with_soul!("abliterated"),
    ];

    const DEFAULT_DISCORD_CHANNELS: &str = include_str!("../../defaults/discord_channels.json");

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
        let tmp = Self::tmp_dir();
        let _ = std::fs::create_dir_all(&agents);
        let _ = std::fs::create_dir_all(&prompts);
        let _ = std::fs::create_dir_all(&tmp);
        let _ = std::fs::create_dir_all(&Self::tmp_js_dir());

        // Shared soul (personality/tone for all agents when no per-agent soul.md). Read by load_soul_content().
        Self::write_default_if_missing(&agents.join("soul.md"), Self::DEFAULT_SOUL);

        // Prompts
        Self::write_default_if_missing(&prompts.join("planning_prompt.md"), Self::DEFAULT_PLANNING_PROMPT);
        Self::write_default_if_missing(&prompts.join("execution_prompt.md"), Self::DEFAULT_EXECUTION_PROMPT);

        // Discord channel config
        Self::write_default_if_missing(&Self::discord_channels_path(), Self::DEFAULT_DISCORD_CHANNELS);

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

    /// Load soul from ~/.mac-stats/agents/soul.md. If missing, write default and return it.
    pub fn load_soul_content() -> String {
        let path = Self::soul_file_path();
        Self::load_file_or_default(&path, Self::DEFAULT_SOUL)
    }

    /// Load planning prompt from ~/.mac-stats/prompts/planning_prompt.md.
    pub fn load_planning_prompt() -> String {
        let path = Self::planning_prompt_path();
        Self::load_file_or_default(&path, Self::DEFAULT_PLANNING_PROMPT)
    }

    /// Load execution prompt from ~/.mac-stats/prompts/execution_prompt.md.
    pub fn load_execution_prompt() -> String {
        let path = Self::execution_prompt_path();
        Self::load_file_or_default(&path, Self::DEFAULT_EXECUTION_PROMPT)
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
