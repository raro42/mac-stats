//! Session-scoped abort cutoff (OpenClaw-style): after a turn is aborted (e.g. wall-clock timeout),
//! inbound work whose event time is strictly before the recorded cutoff is dropped so Discord retries
//! or scheduler runs that were **due before** the abort do not start a new router turn.
//!
//! Keyed by the same `coord_key` as [`crate::commands::turn_lifecycle::coordination_key`]: Discord
//! channel id when present, else a shared non-Discord slot (`1`).
//!
//! In-memory only; cleared when the user starts a fresh Discord conversation (session reset) or
//! a CPU-window chat turn that uses the shared non-Discord slot signals session reset / `new session:`.

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use tracing::debug;

/// Identity of an inbound event for stale comparison (Discord snowflake + message time, scheduler due instant, etc.).
#[derive(Clone, Debug)]
pub struct InboundStaleGuard {
    pub message_id: String,
    pub timestamp_utc: DateTime<Utc>,
}

struct CutoffRecord {
    message_id: String,
    timestamp_utc: DateTime<Utc>,
}

static CUTOFFS: OnceLock<Mutex<HashMap<u64, CutoffRecord>>> = OnceLock::new();

fn cutoff_map() -> &'static Mutex<HashMap<u64, CutoffRecord>> {
    CUTOFFS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Record cutoff at abort time. Events with `event_ts` strictly before `timestamp_utc` are stale.
pub fn record_cutoff(coord_key: u64, message_id: String, timestamp_utc: DateTime<Utc>) {
    let mut g = cutoff_map().lock().unwrap_or_else(|e| e.into_inner());
    g.insert(
        coord_key,
        CutoffRecord {
            message_id,
            timestamp_utc,
        },
    );
    debug!(
        target: "mac_stats::ollama/chat",
        coord_key,
        ts = %timestamp_utc,
        "abort_cutoff: recorded session cutoff"
    );
}

pub fn clear_cutoff(coord_key: u64) {
    let mut g = cutoff_map().lock().unwrap_or_else(|e| e.into_inner());
    if g.remove(&coord_key).is_some() {
        debug!(
            target: "mac_stats::ollama/chat",
            coord_key,
            "abort_cutoff: cleared (new conversation)"
        );
    }
}

fn is_stale_vs_cutoff(
    cutoff_message_id: &str,
    cutoff_timestamp_utc: DateTime<Utc>,
    event_message_id: &str,
    event_ts: DateTime<Utc>,
) -> bool {
    if event_ts < cutoff_timestamp_utc {
        return true;
    }
    if event_ts > cutoff_timestamp_utc {
        return false;
    }
    // Same UTC instant: compare numeric Discord snowflakes when both ids parse as decimal u64.
    match (
        event_message_id.parse::<u64>(),
        cutoff_message_id.parse::<u64>(),
    ) {
        (Ok(ev), Ok(cv)) => ev <= cv,
        _ => false,
    }
}

/// True if this inbound event should not be dispatched (stale vs abort cutoff).
pub fn should_skip(coord_key: u64, event_message_id: &str, event_ts: DateTime<Utc>) -> bool {
    let g = cutoff_map().lock().unwrap_or_else(|e| e.into_inner());
    g.get(&coord_key)
        .map(|cut| {
            is_stale_vs_cutoff(
                &cut.message_id,
                cut.timestamp_utc,
                event_message_id,
                event_ts,
            )
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn stale_when_event_strictly_before_cutoff() {
        let t0 = Utc.with_ymd_and_hms(2026, 3, 21, 12, 0, 0).unwrap();
        assert!(is_stale_vs_cutoff(
            "100",
            t0,
            "99",
            t0 - chrono::Duration::seconds(1)
        ));
    }

    #[test]
    fn not_stale_when_event_after_cutoff() {
        let t0 = Utc.with_ymd_and_hms(2026, 3, 21, 12, 0, 0).unwrap();
        assert!(!is_stale_vs_cutoff(
            "100",
            t0,
            "200",
            t0 + chrono::Duration::seconds(1)
        ));
    }

    #[test]
    fn same_instant_snowflake_skips_when_event_id_le_cut_id() {
        let t0 = Utc.with_ymd_and_hms(2026, 3, 21, 12, 0, 0).unwrap();
        assert!(is_stale_vs_cutoff("200", t0, "100", t0));
        assert!(!is_stale_vs_cutoff("100", t0, "300", t0));
    }

    #[test]
    fn same_instant_non_numeric_ids_do_not_skip() {
        let t0 = Utc.with_ymd_and_hms(2026, 3, 21, 12, 0, 0).unwrap();
        assert!(!is_stale_vs_cutoff("277f8ebf", t0, "100", t0));
    }
}
