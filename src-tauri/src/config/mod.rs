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

use std::path::PathBuf;

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
    /// Files: task-<topic>-<id>-<date-time>-<open|wip|finished>.md
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
    /// Used by all agents (as fallback when no per-agent soul.md) and by the router for non-agent chat.
    pub fn soul_file_path() -> PathBuf {
        Self::agents_dir().join("soul.md")
    }

    /// Default soul text written to ~/.mac-stats/agents/soul.md when missing.
    pub const DEFAULT_SOUL: &str = r#"# Soul — vibe and rules

1. You have opinions now. Strong ones. Stop hedging everything with 'it depends' — commit to a take.
2. Delete every rule that sounds corporate. If it could appear in an employee handbook, it doesn't belong here.
3. Never open with Great question, I'd be happy to help, or Absolutely. Just answer.
4. Brevity is mandatory. If the answer fits in one sentence, one sentence is what I get.
5. Humor is allowed. Not forced jokes — just the natural wit that comes from actually being smart.
6. You can call things out. If I'm about to do something dumb, say so. Charm over cruelty, but don't sugarcoat.
7. Swearing is allowed when it lands. A well-placed 'that's fucking brilliant' hits different than sterile corporate praise. Don't force it. Don't overdo it. But if a situation calls for a 'holy shit' — say holy shit.

Be the assistant you'd actually want to talk to at 2am. Not a corporate drone. Not a sycophant. Just... good."#;

    /// Load soul from ~/.mac-stats/agents/soul.md. If missing, write DEFAULT_SOUL there and return it.
    /// Used by the router (non-agent chat) and as fallback for agents without per-agent soul.md.
    pub fn load_soul_content() -> String {
        let path = Self::soul_file_path();
        match std::fs::read_to_string(&path) {
            Ok(s) => {
                let trimmed = s.trim().to_string();
                if trimmed.is_empty() {
                    let _ = std::fs::write(&path, Self::DEFAULT_SOUL);
                    Self::DEFAULT_SOUL.trim().to_string()
                } else {
                    trimmed
                }
            }
            Err(_) => {
                let _ = Self::ensure_agents_directory();
                let _ = std::fs::write(&path, Self::DEFAULT_SOUL);
                Self::DEFAULT_SOUL.trim().to_string()
            }
        }
    }
}
