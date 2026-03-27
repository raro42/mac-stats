//! Per-conversation serialization for full agent turns (not only Ollama HTTP).
//!
//! [`crate::ollama_queue`] limits concurrent `/api/chat` requests and FIFO-waits per source key.
//! This module serializes **entire** logical sessions — tool loops, browser work, and
//! [`crate::session_memory`] updates — so two Discord messages on the same channel cannot
//! interleave. Different keys still run concurrently.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;
use tracing::debug;

struct KeyedQueueState {
    slots: Mutex<HashMap<String, Arc<Mutex<()>>>>,
}

fn state() -> &'static KeyedQueueState {
    static S: std::sync::OnceLock<KeyedQueueState> = std::sync::OnceLock::new();
    S.get_or_init(|| KeyedQueueState {
        slots: Mutex::new(HashMap::new()),
    })
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
        let mut map = state().slots.lock().await;
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
    let mut map = state().slots.lock().await;
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

/// `true` if `key` is active (mutex held) or queued (entry exists and mutex is taken).
/// Idle keys may be absent from the map after cleanup.
pub async fn is_key_busy(key: &str) -> bool {
    let mutex_arc = {
        let map = state().slots.lock().await;
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
}
