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

struct KeyWaiters {
    busy: bool,
    waiters: VecDeque<tokio::sync::oneshot::Sender<()>>,
}

impl Default for KeyWaiters {
    fn default() -> Self {
        Self {
            busy: false,
            waiters: VecDeque::new(),
        }
    }
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
            let out = f().await;
            drop(permit);
            release_key(&state, &key).await;
            out
        }
    }
}
