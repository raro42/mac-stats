//! Structured logging module using tracing
//!
//! This module provides structured logging using the `tracing` crate.
//! It replaces the hand-rolled logging system with proper structured logging.

use std::path::PathBuf;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// If we haven't rotated today (UTC), copy debug.log to debug.log_sic and truncate debug.log.
/// State is stored in ~/.mac-stats/.debug_log_last_rotated (YYYY-MM-DD). Called once at init.
fn rotate_debug_log_if_due(log_path: &std::path::Path) {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let state_path = crate::config::Config::debug_log_last_rotated_path();
    let sic_path = crate::config::Config::debug_log_sic_path();
    let already_rotated = std::fs::read_to_string(&state_path)
        .ok()
        .map(|s| s.trim().to_string())
        .as_ref()
        .map(|s| s == &today)
        .unwrap_or(false);
    if already_rotated {
        return;
    }
    if log_path.exists() {
        if std::fs::copy(log_path, &sic_path).is_err() {
            return;
        }
    }
    if let Ok(f) = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(log_path)
    {
        let _ = f.sync_all();
    }
    let _ = std::fs::write(&state_path, today);
}

mod legacy;

// Re-export legacy logging for compatibility during migration
pub use legacy::{
    set_verbosity, shorten_file_path_internal, write_structured_log,
    write_structured_log_with_verbosity, VERBOSITY,
};

/// Ellipse a string for display: first half + "..." + last half (no truncation of one end).
/// If `s` has ≤ `max_len` chars, returns `s` unchanged. Otherwise returns
/// `s[0..first_n] + "..." + s[last_n..]` where first_n + 3 + last_n ≤ max_len.
/// Ensures `max_len >= sep_len + 1` so first_count/last_count are never negative.
pub fn ellipse(s: &str, max_len: usize) -> String {
    const SEP: &str = "...";
    let sep_len = 3;
    let max_len = max_len.max(sep_len + 1);
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
    // -v (1): warn only. -vv (2): info + mac_stats=debug (no HTTP client noise). -vvv (3): full trace.
    let filter = match verbosity {
        0 => EnvFilter::new("error"),
        1 => EnvFilter::new("warn"),
        2 => EnvFilter::try_new("info,mac_stats=debug").unwrap_or_else(|_| EnvFilter::new("debug")),
        3 => EnvFilter::new("trace"),
        _ => EnvFilter::new("trace"),
    };

    // CRITICAL: Always use command-line verbosity, ignore RUST_LOG environment variable
    // This ensures that -v flags control logging, not environment variables.
    // At -vv we enable mac_stats=debug but not reqwest/hyper, so monitor checks stay compact.

    // Build subscriber with console and file output
    let registry = tracing_subscriber::registry().with(filter);

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

        // Daily rotation: copy debug.log to debug.log_sic and truncate (once per calendar day, UTC)
        rotate_debug_log_if_due(&log_path);

        // Rotate: if log file exceeds 10 MB, truncate to avoid unbounded growth
        const MAX_LOG_BYTES: u64 = 10 * 1024 * 1024;
        if log_path.exists() {
            if let Ok(meta) = std::fs::metadata(&log_path) {
                if meta.len() > MAX_LOG_BYTES {
                    let _ = std::fs::OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .open(&log_path);
                }
            }
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

            registry.with(console_layer).with(file_layer).init();
        } else {
            // Fallback to console only if file creation fails
            registry.with(console_layer).init();
        }
    } else {
        // Console only
        registry.with(console_layer).init();
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
