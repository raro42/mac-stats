//! Named subsystems for console filtering via `MAC_STATS_LOG`.
//!
//! Log events that should respect the allowlist use a `target` of the form
//! `mac_stats::<path>` where `path` is a subsystem name or `parent/child` (see
//! [`Subsystem`]). The file log (`~/.mac-stats/debug.log`) is **not** filtered;
//! only stderr console output is gated when `MAC_STATS_LOG` is set.

/// Prefix for all mac-stats subsystem `tracing` targets (must match filter parsing).
pub const TARGET_PREFIX: &str = "mac_stats::";

/// Major modules operators can enable in `MAC_STATS_LOG`.
#[allow(dead_code)] // Referenced from tests and docs; string paths in macros need not use this enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Subsystem {
    Metrics,
    Ollama,
    Discord,
    Browser,
    Monitors,
    Alerts,
    Scheduler,
    Plugins,
    Ui,
    Config,
}

#[allow(dead_code)]
impl Subsystem {
    pub const fn as_str(self) -> &'static str {
        match self {
            Subsystem::Metrics => "metrics",
            Subsystem::Ollama => "ollama",
            Subsystem::Discord => "discord",
            Subsystem::Browser => "browser",
            Subsystem::Monitors => "monitors",
            Subsystem::Alerts => "alerts",
            Subsystem::Scheduler => "scheduler",
            Subsystem::Plugins => "plugins",
            Subsystem::Ui => "ui",
            Subsystem::Config => "config",
        }
    }
}

/// `MAC_STATS_LOG=browser,ollama` → `Some(["browser", "ollama"])`. Unset or empty → `None` (no filter).
pub fn parse_subsystem_allowlist_from_env() -> Option<Vec<String>> {
    let raw = std::env::var("MAC_STATS_LOG").ok()?;
    let v: Vec<String> = raw
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}

/// True if `meta_target` is `mac_stats::…` and the path is exactly `selected` or a child `selected/…`.
pub fn target_matches_allowlist(meta_target: &str, allow: &[String]) -> bool {
    let Some(rest) = meta_target.strip_prefix(TARGET_PREFIX) else {
        return false;
    };
    allow.iter().any(|a| path_matches_selected(rest, a))
}

fn path_matches_selected(rest: &str, selected: &str) -> bool {
    rest.starts_with(selected)
        && (rest.len() == selected.len()
            || rest.as_bytes().get(selected.len()).copied() == Some(b'/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowlist_parent_includes_child() {
        let allow = vec!["ollama".to_string()];
        assert!(target_matches_allowlist("mac_stats::ollama", &allow));
        assert!(target_matches_allowlist("mac_stats::ollama/api", &allow));
        assert!(!target_matches_allowlist("mac_stats::ollama_legacy", &allow));
    }

    #[test]
    fn non_mac_stats_targets_never_match() {
        let allow = vec!["browser".to_string()];
        assert!(!target_matches_allowlist("hyper::client", &allow));
    }

    #[test]
    fn subsystem_enum_matches_mac_stats_log_names() {
        assert_eq!(Subsystem::Browser.as_str(), "browser");
        assert_eq!(Subsystem::Ollama.as_str(), "ollama");
    }
}

/// `tracing::info!(target: mac_stats_target!("browser/cdp"), ...)` — use inside `mac_stats_*` macros only.
#[macro_export]
macro_rules! mac_stats_target {
    ($path:literal) => {
        ::std::concat!("mac_stats::", $path)
    };
}

#[macro_export]
macro_rules! mac_stats_trace {
    ($path:literal, $($arg:tt)*) => {
        ::tracing::trace!(target: $crate::mac_stats_target!($path), $($arg)*)
    };
}

#[macro_export]
macro_rules! mac_stats_debug {
    ($path:literal, $($arg:tt)*) => {
        ::tracing::debug!(target: $crate::mac_stats_target!($path), $($arg)*)
    };
}

#[macro_export]
macro_rules! mac_stats_info {
    ($path:literal, $($arg:tt)*) => {
        ::tracing::info!(target: $crate::mac_stats_target!($path), $($arg)*)
    };
}

#[macro_export]
macro_rules! mac_stats_warn {
    ($path:literal, $($arg:tt)*) => {
        ::tracing::warn!(target: $crate::mac_stats_target!($path), $($arg)*)
    };
}
