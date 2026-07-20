//! Hermes-style cooperative interrupt: user can cancel an in-flight tool loop.
//!
//! Keyed by the same `coord_key` as [`crate::commands::turn_lifecycle`]. Tools and the
//! agent tool loop poll [`is_interrupted`]; Discord sets the flag when the user says
//! stop/cancel **before** waiting on the per-channel serial queue.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use tracing::info;

fn map() -> &'static Mutex<HashMap<u64, Arc<AtomicBool>>> {
    static M: OnceLock<Mutex<HashMap<u64, Arc<AtomicBool>>>> = OnceLock::new();
    M.get_or_init(|| Mutex::new(HashMap::new()))
}

fn flag_for(coord_key: u64) -> Arc<AtomicBool> {
    let mut g = map().lock().unwrap_or_else(|e| e.into_inner());
    g.entry(coord_key)
        .or_insert_with(|| Arc::new(AtomicBool::new(false)))
        .clone()
}

/// Clear interrupt at the start of a new turn.
pub fn clear(coord_key: u64) {
    flag_for(coord_key).store(false, Ordering::SeqCst);
}

/// Request interrupt for an in-flight turn on this coordination key.
pub fn request(coord_key: u64) {
    flag_for(coord_key).store(true, Ordering::SeqCst);
    info!(
        target: "mac_stats::turn_interrupt",
        coord_key,
        "cooperative interrupt requested"
    );
}

pub fn is_interrupted(coord_key: u64) -> bool {
    flag_for(coord_key).load(Ordering::SeqCst)
}

/// True when the message is a short stop/cancel/abort ask (Hermes interrupt UX).
pub fn looks_like_stop_request(content: &str) -> bool {
    let n = content
        .trim()
        .trim_start_matches('@')
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    // Strip bot mention leftovers like "werner stop"
    let n = n
        .trim_start_matches("werner")
        .trim_start_matches(',')
        .trim()
        .trim_start_matches("please")
        .trim();
    matches!(
        n,
        "stop"
            | "cancel"
            | "abort"
            | "halt"
            | "quit"
            | "enough"
            | "nevermind"
            | "never mind"
            | "stop it"
            | "cancel that"
            | "abort that"
            | "stop please"
            | "cancel please"
    ) || (n.chars().count() <= 24
        && (n.starts_with("stop ")
            || n.starts_with("cancel ")
            || n.starts_with("abort ")
            || n == "s top"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_stop() {
        assert!(looks_like_stop_request("stop"));
        assert!(looks_like_stop_request("Cancel that"));
        assert!(looks_like_stop_request("@Werner stop"));
        assert!(!looks_like_stop_request("stop the redmine ticket and summarize"));
    }

    #[test]
    fn flag_roundtrip() {
        clear(999001);
        assert!(!is_interrupted(999001));
        request(999001);
        assert!(is_interrupted(999001));
        clear(999001);
        assert!(!is_interrupted(999001));
    }
}
