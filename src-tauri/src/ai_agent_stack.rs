//! Start (and lazily re-start) the optional AI agent stack when `aiAgentEnabled` flips on.
//!
//! `Config::ai_agent_enabled()` already re-reads `~/.mac-stats/config.json` on each call.
//! Discord / scheduler / heartbeat / task-review / compaction historically only spawned at
//! process start — so install.sh or a hand-edit of config.json needed a restart.
//!
//! This module:
//! - starts that stack once (idempotent) when AI is enabled
//! - watches `config.json` so an external edit (or install.sh) can enable AI without restart

use crate::commands;
use crate::config::Config;
use crate::discord;
use crate::scheduler;
use crate::state::APP_HANDLE;
use crate::task;
use notify::{RecursiveMode, Watcher};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::Emitter;
use tracing::{debug, info, warn};

static AI_STACK_STARTED: AtomicBool = AtomicBool::new(false);

const CONFIG_WATCH_DEBOUNCE_MS: u64 = 400;

/// Start Ollama warmup + Discord + scheduler + heartbeat + task review + compaction.
/// Safe to call many times: the heavy stack starts at most once; Discord spawn is itself gated.
pub fn ensure_ai_agent_stack_started() {
    if !Config::ai_agent_enabled() {
        debug!(
            target: "mac_stats::ai_agent_stack",
            "ensure_ai_agent_stack_started: aiAgentEnabled=false, skipping"
        );
        return;
    }

    if AI_STACK_STARTED.swap(true, Ordering::SeqCst) {
        // Stack already up — still poke Discord in case a token appeared later.
        discord::spawn_discord_if_configured();
        return;
    }

    info!(
        target: "mac_stats::ai_agent_stack",
        "Starting AI agent stack (Ollama warmup, Discord, scheduler, heartbeat, task review)"
    );

    tauri::async_runtime::spawn(async {
        commands::ollama_config::ensure_ollama_agent_ready_at_startup().await;
        tracing::debug!(
            target: "mac_stats::ai_agent_stack",
            "Ollama warmup finished after AI stack enable"
        );
    });

    thread::spawn(|| {
        discord::spawn_discord_if_configured();
    });

    scheduler::spawn_scheduler_thread();
    scheduler::heartbeat::spawn_heartbeat_thread();
    task::review::spawn_review_thread();

    thread::spawn(|| {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(r) => r,
            Err(_) => return,
        };
        const INTERVAL_SECS: u64 = 30 * 60;
        loop {
            thread::sleep(Duration::from_secs(INTERVAL_SECS));
            rt.block_on(commands::compaction::run_periodic_session_compaction());
        }
    });
}

/// Watch `~/.mac-stats/config.json` (via the parent dir) and start the AI stack when enabled.
pub fn spawn_config_ai_watcher() {
    let config_path = Config::config_file_path();
    let watch_dir = match config_path.parent() {
        Some(p) => p.to_path_buf(),
        None => {
            warn!(
                target: "mac_stats::ai_agent_stack",
                "config watch: no parent for {:?}",
                config_path
            );
            return;
        }
    };

    thread::spawn(move || {
        // Ensure the directory exists so notify can attach.
        if let Err(e) = std::fs::create_dir_all(&watch_dir) {
            warn!(
                target: "mac_stats::ai_agent_stack",
                "config watch: create_dir_all {:?}: {}",
                watch_dir, e
            );
            return;
        }

        let (tx, rx) = mpsc::channel();
        let mut watcher =
            match notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                let _ = tx.send(res);
            }) {
                Ok(w) => w,
                Err(e) => {
                    warn!(
                        target: "mac_stats::ai_agent_stack",
                        "config watch: failed to create watcher: {}",
                        e
                    );
                    return;
                }
            };

        if let Err(e) = watcher.watch(&watch_dir, RecursiveMode::NonRecursive) {
            warn!(
                target: "mac_stats::ai_agent_stack",
                "config watch: failed to watch {:?}: {}",
                watch_dir, e
            );
            return;
        }

        info!(
            target: "mac_stats::ai_agent_stack",
            "Watching {:?} for config.json changes (AI enable without restart)",
            watch_dir
        );

        let mut last_event = Instant::now();
        let mut pending = false;
        let mut last_enabled = Config::ai_agent_enabled();

        loop {
            let timeout = if pending {
                let elapsed = last_event.elapsed();
                if elapsed >= Duration::from_millis(CONFIG_WATCH_DEBOUNCE_MS) {
                    pending = false;
                    let enabled = Config::ai_agent_enabled();
                    if enabled && !last_enabled {
                        info!(
                            target: "mac_stats::ai_agent_stack",
                            "config.json enabled aiAgentEnabled — starting AI stack"
                        );
                        ensure_ai_agent_stack_started();
                        if let Some(app) = APP_HANDLE.get() {
                            let _ = app.emit("ai-agent-enabled-changed", true);
                        }
                    } else if enabled != last_enabled {
                        info!(
                            target: "mac_stats::ai_agent_stack",
                            "config.json aiAgentEnabled -> {}",
                            enabled
                        );
                        if let Some(app) = APP_HANDLE.get() {
                            let _ = app.emit("ai-agent-enabled-changed", enabled);
                        }
                    }
                    last_enabled = enabled;
                    Duration::from_millis(CONFIG_WATCH_DEBOUNCE_MS)
                } else {
                    Duration::from_millis(CONFIG_WATCH_DEBOUNCE_MS) - elapsed
                }
            } else {
                Duration::from_millis(CONFIG_WATCH_DEBOUNCE_MS)
            };

            match rx.recv_timeout(timeout) {
                Ok(Ok(event)) => {
                    if event_touches_config(&event, &config_path) {
                        last_event = Instant::now();
                        pending = true;
                    }
                }
                Ok(Err(e)) => {
                    debug!(
                        target: "mac_stats::ai_agent_stack",
                        "config watch event error: {:?}",
                        e
                    );
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });
}

fn event_touches_config(event: &notify::Event, config_path: &Path) -> bool {
    let name = config_path.file_name().and_then(|s| s.to_str());
    event.paths.iter().any(|p| {
        if p == config_path {
            return true;
        }
        // Atomic replace: temp then rename to config.json
        match (name, p.file_name().and_then(|s| s.to_str())) {
            (Some(cfg), Some(n)) => n == cfg || n.starts_with("config.json"),
            _ => false,
        }
    })
}
