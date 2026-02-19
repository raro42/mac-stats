//! Structured logging module using tracing
//! 
//! This module provides structured logging using the `tracing` crate.
//! It replaces the hand-rolled logging system with proper structured logging.

use std::path::PathBuf;
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

mod legacy;

// Re-export legacy logging for compatibility during migration
pub use legacy::{set_verbosity, write_structured_log, write_structured_log_with_verbosity, shorten_file_path_internal, VERBOSITY};

/// Ellipse a string for display: first half + "..." + last half (no truncation of one end).
/// If `s` has ≤ `max_len` chars, returns `s` unchanged. Otherwise returns
/// `s[0..first_n] + "..." + s[last_n..]` where first_n + 3 + last_n ≤ max_len.
pub fn ellipse(s: &str, max_len: usize) -> String {
    const SEP: &str = "...";
    let sep_len = 3;
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    if n <= max_len {
        return s.to_string();
    }
    let first_count = (max_len - sep_len) / 2;
    let last_count = (max_len - sep_len) - first_count;
    let first: String = chars[..first_count].iter().collect();
    let last: String = chars[n - last_count..].iter().collect();
    format!("{}{}{}", first, SEP, last)
}

/// Initialize tracing with file and console output
/// 
/// The log file path will be determined by the config module (when available).
/// For now, uses a temporary path that will be replaced in Phase 3.
pub fn init_tracing(verbosity: u8, log_file_path: Option<PathBuf>) {
    // Convert verbosity level (0-3) to tracing level.
    // -v (1): warn only, no debug. -vv (2): debug. -vvv (3): trace.
    let filter_level = match verbosity {
        0 => "error",
        1 => "warn",   // -v: no debug logs
        2 => "debug",  // -vv: show debug
        3 => "trace",
        _ => "trace",
    };

    // Create env filter
    // CRITICAL: Always use command-line verbosity, ignore RUST_LOG environment variable
    // This ensures that -v flags control logging, not environment variables
    // Default to "error" level (verbosity 0) for minimal logging and CPU usage
    let filter = EnvFilter::new(filter_level);

    // Build subscriber with console and file output
    let registry = tracing_subscriber::registry()
        .with(filter);

    // Add console layer (stderr)
    let console_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false);

    // Add file layer if path is provided
    if let Some(log_path) = log_file_path {
        // Ensure directory exists
        if let Some(parent) = log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        // Create file layer
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .ok();

        if let Some(file) = file {
            let file_layer = fmt::layer()
                .with_writer(file)
                .with_target(false)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_ansi(false); // No ANSI in files

            registry
                .with(console_layer)
                .with(file_layer)
                .init();
        } else {
            // Fallback to console only if file creation fails
            registry
                .with(console_layer)
                .init();
        }
    } else {
        // Console only
        registry
            .with(console_layer)
            .init();
    }
}

/// Set verbosity level (compatibility function)
/// 
/// This function updates both the legacy VERBOSITY and tracing filter.
/// Currently unused but kept for potential future use.
#[allow(dead_code)]
pub fn set_verbosity_with_tracing(level: u8) {
    // Update legacy verbosity for compatibility
    legacy::set_verbosity(level);
    
    // Note: Tracing filter is set at init time, so we'd need to reload
    // For now, this is mainly for compatibility during migration
}
