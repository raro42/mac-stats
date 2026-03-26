//! Session-scoped abort cutoff (OpenClaw-style): after a turn is aborted (e.g. wall-clock timeout),
//! inbound work whose event time is strictly before the recorded cutoff is dropped so Discord retries
//! or scheduler runs that were **due before** the abort do not start a new router turn.
//!
//! Keyed by the same `coord_key` as [`crate::commands::turn_lifecycle::coordination_key`]: Discord
//! channel id when present, else a shared non-Discord slot (`1`).
//!
//! In-memory only; cleared when the user starts a fresh Discord conversation (session reset).

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
    let mut g = cutoff_map()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
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
    let mut g = cutoff_map()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if g.remove(&coord_key).is_some() {
        debug!(
            target: "mac_stats::ollama/chat",
            coord_key,
            "abort_cutoff: cleared (new conversation)"
        );
    }
}

/// True if this inbound event should not be dispatched (stale vs abort cutoff).
pub fn should_skip(coord_key: u64, event_message_id: &str, event_ts: DateTime<Utc>) -> bool {
    let g = cutoff_map()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let Some(cut) = g.get(&coord_key) else {
        return false;
    };
    if event_ts < cut.timestamp_utc {
        return true;
    }
    if event_ts > cut.timestamp_utc {
        return false;
    }
    // Same UTC instant: compare numeric Discord snowflakes when both ids parse as decimal u64.
    match (
        event_message_id.parse::<u64>(),
        cut.message_id.parse::<u64>(),
    ) {
        (Ok(ev), Ok(cv)) => ev <= cv,
        _ => false,
    }
}
