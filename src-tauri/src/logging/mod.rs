//! Structured logging module using tracing
//!
//! This module provides structured logging using the `tracing` crate.
//! It replaces the hand-rolled logging system with proper structured logging.

use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use tracing::Metadata;
use tracing_subscriber::filter::FilterFn;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;
use tracing_subscriber::{fmt, EnvFilter};

pub mod redact;
pub mod subsystem;

pub use redact::redact_secrets;

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
    if log_path.exists() && std::fs::copy(log_path, &sic_path).is_err() {
        return;
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

/// Handle to `~/.mac-stats/debug.log` when file logging is enabled (for shutdown flush).
static DEBUG_LOG_FILE: OnceLock<Arc<Mutex<std::fs::File>>> = OnceLock::new();

/// Flush and sync the debug log file so shutdown lines survive abrupt process teardown.
pub fn sync_debug_log_best_effort() {
    if let Some(arc) = DEBUG_LOG_FILE.get() {
        if let Ok(mut g) = arc.lock() {
            let _ = g.flush();
            let _ = g.sync_all();
        }
    }
}

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
    redact::init_from_env();
    let redact_logs = redact::redaction_active();

    // Convert verbosity level (0-3) to tracing level.
    // -v (1): warn + discord/draft=info (draft placeholder/edits visible in debug.log for reviewers).
    // -vv (2): info + mac_stats=debug + ollama/untrusted=debug (untrusted wrap trace; no HTTP noise). -vvv (3): full trace.
    let filter = match verbosity {
        0 => EnvFilter::new("error"),
        1 => {
            EnvFilter::try_new("warn,discord/draft=info").unwrap_or_else(|_| EnvFilter::new("warn"))
        }
        2 => EnvFilter::try_new("info,mac_stats=debug,ollama/untrusted=debug,discord/draft=info")
            .unwrap_or_else(|_| EnvFilter::new("debug")),
        3 => EnvFilter::new("trace"),
        _ => EnvFilter::new("trace"),
    };

    // CRITICAL: Always use command-line verbosity, ignore RUST_LOG environment variable
    // This ensures that -v flags control logging, not environment variables.
    // At -vv we enable mac_stats=debug but not reqwest/hyper, so monitor checks stay compact.
    // `ollama/untrusted` and `discord/draft` are custom tracing targets (not under mac_stats::); include them explicitly so those lines appear in debug.log.

    // Build subscriber with console and file output
    let registry = tracing_subscriber::registry().with(filter);

    // Console-only subsystem filter: when `MAC_STATS_LOG` is set, stderr shows only matching targets.
    let parsed_allow = subsystem::parse_subsystem_allowlist_from_env();
    if let Some(ref names) = parsed_allow {
        eprintln!(
            "mac-stats: MAC_STATS_LOG is set — stderr shows only mac_stats:: targets: {}",
            names.join(", ")
        );
    }
    let subsystem_allow: Option<Arc<Vec<String>>> = parsed_allow.map(Arc::new);
    let console_subsystem_filter = FilterFn::new({
        let subsystem_allow = subsystem_allow.clone();
        move |meta: &Metadata<'_>| match &subsystem_allow {
            None => true,
            Some(list) => subsystem::target_matches_allowlist(meta.target(), list.as_slice()),
        }
    });

    // Add console layer (stderr); optional secret redaction on full lines
    let console_layer = fmt::layer()
        .with_writer(move || redact::RedactingLineWriter::new(std::io::stderr(), redact_logs))
        .with_target(subsystem_allow.is_some())
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_filter(console_subsystem_filter);

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
            let file_mk = redact::RedactingFileMakeWriter::new(file, redact_logs);
            let _ = DEBUG_LOG_FILE.set(file_mk.shared_file());
            let file_layer = fmt::layer()
                .with_writer(file_mk)
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

    tracing::info!(
        target: "mac_stats::logging",
        "Log secret redaction: {} (set LOG_REDACTION=0 in env or ~/.mac-stats/.config.env for raw output)",
        if redact::redaction_active() {
            "enabled"
        } else {
            "disabled"
        }
    );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ellipse_short_string_unchanged() {
        assert_eq!(ellipse("hello", 10), "hello");
    }

    #[test]
    fn ellipse_exact_max_len_unchanged() {
        assert_eq!(ellipse("abcde", 5), "abcde");
    }

    #[test]
    fn ellipse_longer_string_truncated() {
        let result = ellipse("abcdefghij", 7);
        assert_eq!(result, "ab...ij");
    }

    #[test]
    fn ellipse_preserves_start_and_end() {
        let input = "The quick brown fox jumps over the lazy dog";
        let result = ellipse(input, 20);
        assert!(result.starts_with("The quic"));
        assert!(result.ends_with("lazy dog"));
        assert!(result.contains("..."));
        assert!(result.chars().count() <= 20);
    }

    #[test]
    fn ellipse_max_len_zero_clamped() {
        let result = ellipse("abcdef", 0);
        assert!(result.contains("..."));
        assert!(result.chars().count() <= 4);
    }

    #[test]
    fn ellipse_max_len_one_clamped() {
        let result = ellipse("abcdef", 1);
        assert!(result.contains("..."));
    }

    #[test]
    fn ellipse_max_len_three_clamped() {
        let result = ellipse("abcdef", 3);
        assert!(result.contains("..."));
        assert!(result.chars().count() <= 4);
    }

    #[test]
    fn ellipse_max_len_two_clamped() {
        // F7: max_len below sep_len+1 (4) clamps; same effective budget as max_len 4 for long strings.
        let result = ellipse("abcdef", 2);
        assert_eq!(result, "...f");
        assert_eq!(result.chars().count(), 4);
    }

    #[test]
    fn ellipse_empty_string() {
        assert_eq!(ellipse("", 10), "");
    }

    #[test]
    fn ellipse_single_char_under_limit() {
        assert_eq!(ellipse("x", 5), "x");
    }

    #[test]
    fn ellipse_unicode_chars() {
        let input = "こんにちは世界テスト";
        let result = ellipse(input, 7);
        assert_eq!(result, "こん...スト");
        assert_eq!(result.chars().count(), 7);
    }

    #[test]
    fn ellipse_result_length_within_max() {
        for max_len in 0..=30 {
            let result = ellipse("abcdefghijklmnopqrstuvwxyz", max_len);
            let effective_max = max_len.max(4);
            assert!(
                result.chars().count() <= effective_max,
                "max_len={}, result='{}' has {} chars (expected <= {})",
                max_len,
                result,
                result.chars().count(),
                effective_max
            );
        }
    }

    #[test]
    fn ellipse_odd_even_max_len_splits() {
        let result_even = ellipse("abcdefghij", 8);
        assert_eq!(result_even, "ab...hij");

        let result_odd = ellipse("abcdefghij", 9);
        assert_eq!(result_odd, "abc...hij");
    }

    /// Regression: `wrap_untrusted_content` uses target `ollama/untrusted`, which is not under `mac_stats::`.
    #[test]
    fn vv_env_filter_accepts_ollama_untrusted_directive() {
        let s = "info,mac_stats=debug,ollama/untrusted=debug";
        let _ = EnvFilter::try_new(s)
            .expect("vv filter must include ollama/untrusted for untrusted wrap logs");
    }
}
