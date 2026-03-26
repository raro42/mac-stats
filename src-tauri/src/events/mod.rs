//! In-process event bus for cross-cutting reactions (OpenClaw-style internal hooks).
//!
//! Handlers are best-effort: panics and errors in one handler do not block others or the caller.
//! There is no external hook discovery; this is for decoupling inside the Tauri app only.

use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

use tokio::sync::mpsc::UnboundedSender;

/// Payload for [`emit`]. Extend with new variants as more events are wired.
#[derive(Clone, Debug)]
pub enum EventPayload {
    ScreenshotSaved { path: PathBuf },
    OllamaTurnComplete {
        model: String,
        tokens_used: u64,
        duration_ms: u128,
    },
    ToolInvoked {
        tool_name: String,
        success: bool,
        duration_ms: u128,
    },
    DiscordMessageReceived { channel: String, user: String },
    MonitorCheckComplete { monitor_id: String, status: String },
}

type Handler = Box<dyn Fn(EventPayload) + Send + Sync + 'static>;

static HANDLERS: OnceLock<RwLock<HashMap<String, Vec<Handler>>>> = OnceLock::new();

fn handlers_map() -> &'static RwLock<HashMap<String, Vec<Handler>>> {
    HANDLERS.get_or_init(|| RwLock::new(HashMap::new()))
}

/// While held, the default `screenshot:saved` handler may forward
/// `ATTACH:` lines to the active agent-router status channel (e.g. Discord draft / attachments).
pub struct AgentStatusTxGuard {
    previous: Option<UnboundedSender<String>>,
    previous_gate: Option<Arc<AtomicBool>>,
}

impl AgentStatusTxGuard {
    fn swap_in(
        tx: Option<UnboundedSender<String>>,
        gate: Option<Arc<AtomicBool>>,
    ) -> Self {
        let mut g = AGENT_STATUS_TX.lock().unwrap_or_else(|e| e.into_inner());
        let previous = g.take();
        *g = tx;
        let mut gg = AGENT_STATUS_GATE.lock().unwrap_or_else(|e| e.into_inner());
        let previous_gate = std::mem::replace(&mut *gg, gate);
        Self {
            previous,
            previous_gate,
        }
    }
}

impl Drop for AgentStatusTxGuard {
    fn drop(&mut self) {
        let mut g = AGENT_STATUS_TX.lock().unwrap_or_else(|e| e.into_inner());
        *g = self.previous.take();
        let mut gg = AGENT_STATUS_GATE.lock().unwrap_or_else(|e| e.into_inner());
        *gg = self.previous_gate.take();
    }
}

static AGENT_STATUS_TX: Mutex<Option<UnboundedSender<String>>> = Mutex::new(None);
static AGENT_STATUS_GATE: Mutex<Option<Arc<AtomicBool>>> = Mutex::new(None);

/// Install the status channel for the current agent tool loop (nested calls restore the previous sender).
/// When `output_gate` is `Some`, `screenshot:saved` and similar forwards are suppressed after the gate closes (turn timeout).
pub fn push_agent_status_tx(
    tx: Option<UnboundedSender<String>>,
    output_gate: Option<Arc<AtomicBool>>,
) -> AgentStatusTxGuard {
    AgentStatusTxGuard::swap_in(tx, output_gate)
}

pub fn subscribe<F>(event_key: impl Into<String>, handler: F)
where
    F: Fn(EventPayload) + Send + Sync + 'static,
{
    let key = event_key.into();
    let mut map = handlers_map()
        .write()
        .unwrap_or_else(|e| e.into_inner());
    map.entry(key).or_default().push(Box::new(handler));
}

/// Deliver `payload` to all subscribers of `event_key`. Never panics to callers; handler panics are logged.
pub fn emit(event_key: &str, payload: EventPayload) {
    let map = handlers_map()
        .read()
        .unwrap_or_else(|e| e.into_inner());
    let Some(list) = map.get(event_key) else {
        return;
    };
    for h in list.iter() {
        let p = payload.clone();
        let r = catch_unwind(AssertUnwindSafe(|| (*h)(p)));
        if r.is_err() {
            tracing::warn!(
                target: "mac_stats::events",
                "event handler panicked for key={} (isolated; continuing)",
                event_key
            );
        }
    }
}

/// Subscribe built-in handlers (screenshot log + Discord ATTACH, tool observability).
pub fn register_default_handlers() {
    subscribe("screenshot:saved", |p| {
        let EventPayload::ScreenshotSaved { path } = p else {
            return;
        };
        crate::mac_stats_info!(
            "events/screenshot",
            "internal event screenshot:saved path={}",
            path.display()
        );
        let msg = format!("ATTACH:{}", path.display());
        let allow = match AGENT_STATUS_GATE.lock() {
            Ok(gg) => gg
                .as_ref()
                .map(|gate| gate.load(Ordering::Acquire))
                .unwrap_or(true),
            Err(_) => true,
        };
        if !allow {
            return;
        }
        let g = AGENT_STATUS_TX.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(tx) = g.as_ref() {
            let _ = tx.send(msg);
        }
    });

    subscribe("tool:invoked", |p| {
        let EventPayload::ToolInvoked {
            tool_name,
            success,
            duration_ms,
        } = p
        else {
            return;
        };
        crate::mac_stats_debug!(
            "events/tool",
            "internal event tool:invoked tool={} success={} duration_ms={}",
            tool_name,
            success,
            duration_ms
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn emit_delivers_to_subscriber() {
        static C: AtomicU32 = AtomicU32::new(0);
        C.store(0, Ordering::SeqCst);
        // fresh map for test: use unique key
        let key = "test:ping";
        subscribe(key, |_| {
            C.fetch_add(1, Ordering::SeqCst);
        });
        emit(
            key,
            EventPayload::ToolInvoked {
                tool_name: "X".into(),
                success: true,
                duration_ms: 1,
            },
        );
        assert_eq!(C.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn panicking_handler_does_not_block_next() {
        static C: AtomicU32 = AtomicU32::new(0);
        C.store(0, Ordering::SeqCst);
        let key = "test:panic_chain";
        subscribe(key, |_| panic!("boom"));
        subscribe(key, |_| {
            C.fetch_add(1, Ordering::SeqCst);
        });
        emit(
            key,
            EventPayload::ToolInvoked {
                tool_name: "Y".into(),
                success: false,
                duration_ms: 0,
            },
        );
        assert_eq!(C.load(Ordering::SeqCst), 1);
    }
}
