//! Global + keyed serialization for Ollama HTTP traffic (`/api/chat`).
//!
//! Limits concurrent Ollama work (default one request at a time globally) while
//! queueing waiters per logical source key (Discord channel, `scheduler`, `cpu_ui`, …).

use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::sync::{Arc, OnceLock};

use tokio::sync::{Mutex, OwnedSemaphorePermit, Semaphore};

use crate::config::Config;
use crate::debug2;
use crate::mac_stats_info;

#[derive(Default)]
struct KeyWaiters {
    busy: bool,
    waiters: VecDeque<tokio::sync::oneshot::Sender<()>>,
}

struct OllamaQueueState {
    global: Arc<Semaphore>,
    keys: Mutex<HashMap<String, KeyWaiters>>,
}

fn queue_state() -> Arc<OllamaQueueState> {
    static STATE: OnceLock<Arc<OllamaQueueState>> = OnceLock::new();
    STATE
        .get_or_init(|| {
            let n = Config::ollama_global_concurrency().max(1) as usize;
            Arc::new(OllamaQueueState {
                global: Arc::new(Semaphore::new(n)),
                keys: Mutex::new(HashMap::new()),
            })
        })
        .clone()
}

/// How [`with_ollama_http_queue`] should handle the Ollama HTTP slot.
#[derive(Clone)]
pub enum OllamaHttpQueue {
    /// Caller already holds the slot (e.g. inside [`crate::commands::ollama::answer_with_ollama_and_fetch`]).
    Nested,
    /// Acquire global + per-key FIFO slot for this HTTP operation.
    Acquire {
        key: String,
        wait_hook: Option<Arc<dyn Fn() + Send + Sync>>,
    },
}

async fn acquire_key_then_global(
    state: &Arc<OllamaQueueState>,
    key: &str,
    wait_hook: Option<&Arc<dyn Fn() + Send + Sync>>,
) -> OwnedSemaphorePermit {
    let wait_start = std::time::Instant::now();
    let mut per_key_depth: usize = 0;
    let rx = {
        let mut map = state.keys.lock().await;
        let entry = map.entry(key.to_string()).or_default();
        if !entry.busy {
            entry.busy = true;
            None
        } else {
            let (tx, rx) = tokio::sync::oneshot::channel();
            entry.waiters.push_back(tx);
            per_key_depth = entry.waiters.len();
            Some(rx)
        }
    };
    if let Some(rx) = rx {
        debug2!(
            "ollama/queue: blocked waiting for key slot key={} per_key_waiters_ahead={}",
            key,
            per_key_depth
        );
        if let Some(h) = wait_hook {
            (h)();
        }
        let _ = rx.await;
    }
    let key_wait_ms = wait_start.elapsed().as_millis() as u64;
    let global_avail = state.global.available_permits();
    debug2!(
        "ollama/queue: acquired key slot key={} per_key_waiters_ahead={} key_wait_ms={} global_available_permits={}",
        key,
        per_key_depth,
        key_wait_ms,
        global_avail
    );
    state
        .global
        .clone()
        .acquire_owned()
        .await
        .expect("ollama global semaphore closed")
}

async fn release_key(state: &Arc<OllamaQueueState>, key: &str) {
    let mut map = state.keys.lock().await;
    let Some(entry) = map.get_mut(key) else {
        return;
    };
    if let Some(tx) = entry.waiters.pop_front() {
        let _ = tx.send(());
    } else {
        entry.busy = false;
        map.remove(key);
    }
}

/// Run `f` while holding the Ollama HTTP queue slot when `spec` is [`OllamaHttpQueue::Acquire`].
pub async fn with_ollama_http_queue<F, Fut, T>(spec: OllamaHttpQueue, f: F) -> T
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = T>,
{
    match spec {
        OllamaHttpQueue::Nested => f().await,
        OllamaHttpQueue::Acquire { key, wait_hook } => {
            let state = queue_state();
            let permit = acquire_key_then_global(&state, &key, wait_hook.as_ref()).await;
            let global_avail_after = state.global.available_permits();
            debug2!(
                "ollama/queue: acquired global permit key={} global_available_permits_after={}",
                key,
                global_avail_after
            );
            mac_stats_info!(
                "ollama/queue",
                "Ollama HTTP queue: global permit acquired (key={}, global_available_permits_after={})",
                key,
                global_avail_after
            );
            let out = f().await;
            drop(permit);
            release_key(&state, &key).await;
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Single test body so the process-wide queue singleton is not stressed by parallel tests.
    #[tokio::test]
    async fn ollama_http_queue_serializes_and_fires_wait_hook() {
        // Different keys, global concurrency 1: inner work for B only after A releases global permit.
        {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let log: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));
            let l1 = log.clone();
            let l2 = log.clone();
            let h1 = tokio::spawn(async move {
                with_ollama_http_queue(
                    OllamaHttpQueue::Acquire {
                        key: "unit_dk1".to_string(),
                        wait_hook: None,
                    },
                    || async {
                        let _ = tx.send(());
                        l1.lock().await.push("A_in");
                        tokio::time::sleep(std::time::Duration::from_millis(40)).await;
                        l1.lock().await.push("A_out");
                    },
                )
                .await
            });
            rx.await.expect("A should signal from inside queue");
            let h2 = tokio::spawn(async move {
                with_ollama_http_queue(
                    OllamaHttpQueue::Acquire {
                        key: "unit_dk2".to_string(),
                        wait_hook: None,
                    },
                    || async {
                        l2.lock().await.push("B_in");
                    },
                )
                .await
            });
            let _ = tokio::join!(h1, h2);
            assert_eq!(
                *log.lock().await,
                vec!["A_in", "A_out", "B_in"],
                "global limit should serialize different keys"
            );
        }

        // Same key: second acquire waits on per-key FIFO (still ordered after first completes).
        {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let log: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));
            let l1 = log.clone();
            let l2 = log.clone();
            let h1 = tokio::spawn(async move {
                with_ollama_http_queue(
                    OllamaHttpQueue::Acquire {
                        key: "unit_same".to_string(),
                        wait_hook: None,
                    },
                    || async {
                        let _ = tx.send(());
                        l1.lock().await.push("S1_in");
                        tokio::time::sleep(std::time::Duration::from_millis(40)).await;
                        l1.lock().await.push("S1_out");
                    },
                )
                .await
            });
            rx.await.expect("S1 signal");
            let h2 = tokio::spawn(async move {
                with_ollama_http_queue(
                    OllamaHttpQueue::Acquire {
                        key: "unit_same".to_string(),
                        wait_hook: None,
                    },
                    || async {
                        l2.lock().await.push("S2_in");
                    },
                )
                .await
            });
            let _ = tokio::join!(h1, h2);
            assert_eq!(
                *log.lock().await,
                vec!["S1_in", "S1_out", "S2_in"],
                "same-key FIFO"
            );
        }

        // wait_hook runs when the caller blocks on the per-key queue.
        {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let fired = Arc::new(AtomicBool::new(false));
            let hook_f = fired.clone();
            let wait_hook: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
                hook_f.store(true, Ordering::SeqCst);
            });
            let h1 = tokio::spawn(async move {
                with_ollama_http_queue(
                    OllamaHttpQueue::Acquire {
                        key: "unit_hook".to_string(),
                        wait_hook: None,
                    },
                    || async {
                        let _ = tx.send(());
                        tokio::time::sleep(std::time::Duration::from_millis(40)).await;
                    },
                )
                .await
            });
            rx.await.expect("hook test A");
            let h2 = tokio::spawn(async move {
                with_ollama_http_queue(
                    OllamaHttpQueue::Acquire {
                        key: "unit_hook".to_string(),
                        wait_hook: Some(wait_hook),
                    },
                    || async {},
                )
                .await
            });
            let _ = tokio::join!(h1, h2);
            assert!(
                fired.load(Ordering::SeqCst),
                "wait_hook should run when second request queues on same key"
            );
        }
    }
}
