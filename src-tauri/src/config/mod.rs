//! Configuration module for portable paths and build information
//! 
//! This module provides a centralized way to access configuration values
//! like log file paths and build information, replacing hard-coded values.

use std::path::{PathBuf, Path};

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
}
