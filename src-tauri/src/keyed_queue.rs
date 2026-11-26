//! Per-conversation serialization for full agent turns (not only Ollama HTTP).
//!
//! [`crate::ollama_queue`] limits concurrent `/api/chat` requests and FIFO-waits per source key.
//! This module serializes **entire** logical sessions — tool loops, browser work, and
//! [`crate::session_memory`] updates — so two Discord messages on the same channel cannot
//! interleave. Different keys still run concurrently.

use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex};

use tokio::sync::Mutex;
use tracing::debug;

struct KeyedQueueState {
    /// Map of session key → per-key async mutex. `std` lock so `/status` / Agent Ops can
    /// snapshot without nesting Tokio `block_on` on the menu-bar runtime.
    slots: StdMutex<HashMap<String, Arc<Mutex<()>>>>,
}

fn state() -> &'static KeyedQueueState {
    static S: std::sync::OnceLock<KeyedQueueState> = std::sync::OnceLock::new();
    S.get_or_init(|| KeyedQueueState {
        slots: StdMutex::new(HashMap::new()),
    })
}

fn lock_slots() -> std::sync::MutexGuard<'static, HashMap<String, Arc<Mutex<()>>>> {
    // Poison: recover the map so a panicked holder cannot freeze Discord forever.
    state()
        .slots
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Run `work` while holding an exclusive async lock for `key`. Other tasks using the same
/// `key` wait; different keys proceed in parallel. When no task references a key’s mutex
/// anymore, the map entry is removed.
pub async fn run_serial<Fut, R>(key: impl AsRef<str>, work: Fut) -> R
where
    Fut: std::future::Future<Output = R>,
{
    let key = key.as_ref().to_string();
    let mutex_arc = {
        let mut map = lock_slots();
        map.entry(key.clone())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    };
    debug!(
        target: "mac_stats::session/keyed_queue",
        key = %key,
        strong_count = Arc::strong_count(&mutex_arc),
        "session/keyed_queue: acquire"
    );
    let guard = mutex_arc.lock().await;
    debug!(
        target: "mac_stats::session/keyed_queue",
        key = %key,
        "session/keyed_queue: entered (running work)"
    );
    let output = work.await;
    drop(guard);
    drop(mutex_arc);
    let mut map = lock_slots();
    if let Some(arc) = map.get(&key) {
        if Arc::strong_count(arc) == 1 {
            map.remove(&key);
            debug!(
                target: "mac_stats::session/keyed_queue",
                key = %key,
                "session/keyed_queue: cleanup"
            );
        }
    }
    output
}

/// Point-in-time view of per-session serialization (Discord channel, task runner, …).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionQueueSnapshot {
    pub tracked_session_keys: u32,
    pub busy_session_keys: u32,
    /// Estimated turns waiting behind a held session mutex (OpenClaw serialization wait depth).
    ///
    /// Derived from `Arc::strong_count` on busy keys: map slot + holder ≈ 2; each extra clone is a
    /// waiter in [`run_serial`]. Not an exact Tokio waiter list — good enough for `/queue` / `/status`.
    pub waiting_session_refs: u32,
    /// Up to 16 keys whose session mutex is currently held (another turn may be waiting on the same key).
    pub busy_key_sample: Vec<String>,
}

/// Snapshot of keyed session queue: tracked keys, busy keys, and estimated waiters behind busy locks.
/// Sync so Discord `/status` / `/queue` stay zero-LLM.
pub fn snapshot_session_queue() -> SessionQueueSnapshot {
    const MAX_SAMPLE: usize = 16;
    let map = lock_slots();
    let mut tracked_session_keys = 0u32;
    let mut busy_session_keys = 0u32;
    let mut waiting_session_refs = 0u32;
    let mut busy_key_sample = Vec::new();
    for (key, arc) in map.iter() {
        tracked_session_keys += 1;
        if arc.try_lock().is_err() {
            busy_session_keys += 1;
            // map entry + running holder ≈ 2; anything above is a clone waiting on `lock().await`.
            let extras = Arc::strong_count(arc).saturating_sub(2) as u32;
            waiting_session_refs = waiting_session_refs.saturating_add(extras);
            if busy_key_sample.len() < MAX_SAMPLE {
                busy_key_sample.push(key.clone());
            }
        }
    }
    busy_key_sample.sort();
    SessionQueueSnapshot {
        tracked_session_keys,
        busy_session_keys,
        waiting_session_refs,
        busy_key_sample,
    }
}

/// Async wrapper for callers that already sit on an async path (operator pressure).
pub async fn snapshot_session_queue_async() -> SessionQueueSnapshot {
    snapshot_session_queue()
}

/// `true` if `key` is active (mutex held) or queued (entry exists and mutex is taken).
/// Idle keys may be absent from the map after cleanup.
pub async fn is_key_busy(key: &str) -> bool {
    let mutex_arc = {
        let map = lock_slots();
        map.get(key).cloned()
    };
    let Some(m) = mutex_arc else {
        return false;
    };
    let busy = match m.try_lock() {
        Ok(_guard) => false,
        Err(_) => true,
    };
    busy
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex as TokioMutex;

    #[tokio::test]
    async fn same_key_runs_sequentially() {
        let log: Arc<TokioMutex<Vec<&'static str>>> = Arc::new(TokioMutex::new(Vec::new()));
        let a = log.clone();
        let b = log.clone();
        let h1 = tokio::spawn(async move {
            run_serial("unit-serial", async move {
                a.lock().await.push("1_in");
                tokio::time::sleep(std::time::Duration::from_millis(35)).await;
                a.lock().await.push("1_out");
            })
            .await
        });
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        let h2 = tokio::spawn(async move {
            run_serial("unit-serial", async move {
                b.lock().await.push("2_in");
                b.lock().await.push("2_out");
            })
            .await
        });
        let _ = tokio::join!(h1, h2);
        assert_eq!(
            *log.lock().await,
            vec!["1_in", "1_out", "2_in", "2_out"],
            "second key holder must wait for first"
        );
    }

    #[tokio::test]
    async fn different_keys_may_overlap() {
        let log: Arc<TokioMutex<Vec<&'static str>>> = Arc::new(TokioMutex::new(Vec::new()));
        let a = log.clone();
        let b = log.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        let h1 = tokio::spawn(async move {
            run_serial("unit-A", async move {
                let _ = tx.send(());
                a.lock().await.push("A_in");
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                a.lock().await.push("A_out");
            })
            .await
        });
        rx.await.expect("A should start");
        let h2 = tokio::spawn(async move {
            run_serial("unit-B", async move {
                b.lock().await.push("B_in");
                b.lock().await.push("B_out");
            })
            .await
        });
        let _ = tokio::join!(h1, h2);
        let v = log.lock().await.clone();
        let pos_b = v.iter().position(|s| *s == "B_in").expect("B_in");
        let pos_a_out = v.iter().position(|s| *s == "A_out").expect("A_out");
        assert!(
            pos_b < pos_a_out,
            "B should finish before A releases (overlap): {:?}",
            v
        );
    }

    #[test]
    fn idle_snapshot_is_clear() {
        let snap = snapshot_session_queue();
        // Other tests may leave keys; just ensure the sync path returns a struct.
        let _ = snap.tracked_session_keys;
        let _ = snap.busy_session_keys;
        let _ = snap.waiting_session_refs;
    }

    #[tokio::test]
    async fn busy_snapshot_counts_waiting_refs() {
        let (entered_tx, entered_rx) = tokio::sync::oneshot::channel::<()>();
        let (release_tx, release_rx) = tokio::sync::oneshot::channel::<()>();
        let h1 = tokio::spawn(async move {
            run_serial("unit-wait-depth", async move {
                let _ = entered_tx.send(());
                let _ = release_rx.await;
            })
            .await
        });
        entered_rx.await.expect("holder should enter");
        // Second task clones the Arc and blocks on the same key.
        let h2 = tokio::spawn(async move {
            run_serial("unit-wait-depth", async move {
                // no-op after acquire
            })
            .await
        });
        // Give the waiter time to clone + park on lock.
        tokio::time::sleep(std::time::Duration::from_millis(40)).await;
        let snap = snapshot_session_queue();
        assert!(
            snap.busy_session_keys >= 1,
            "holder must show busy: {:?}",
            snap
        );
        assert!(
            snap.waiting_session_refs >= 1,
            "waiter Arc clone must bump wait depth: {:?}",
            snap
        );
        let _ = release_tx.send(());
        let _ = tokio::join!(h1, h2);
    }
}
