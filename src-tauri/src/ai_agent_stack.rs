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

/// Resolve the local Ollama base URL (same defaults as install.sh / Ollama docs).
fn local_ollama_base_url() -> String {
    let raw = std::env::var("OLLAMA_HOST")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "http://127.0.0.1:11434".to_string());
    let with_scheme = if raw.starts_with("http://") || raw.starts_with("https://") {
        raw
    } else {
        format!("http://{raw}")
    };
    with_scheme.trim_end_matches('/').to_string()
}

/// True when GET `{base}/api/tags` succeeds quickly (local Ollama is up).
fn local_ollama_api_reachable() -> bool {
    let base = local_ollama_base_url();
    let url = format!("{base}/api/tags");
    let client = match reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .connect_timeout(Duration::from_secs(1))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };
    match client.get(&url).send() {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}

/// One-shot: if AI is off, Ollama answers locally, and we have not already probed / the user
/// has not chosen monitor-only, turn `aiAgentEnabled` on (same idea as `install.sh`).
///
/// Opt out: `MAC_STATS_NO_AI=1`, or Settings → AI off / Reset to monitor defaults
/// (sets `aiAgentOllamaAutoProbeDone`).
///
/// Returns true when this call enabled AI.
pub fn maybe_auto_enable_ai_from_local_ollama() -> bool {
    if Config::ai_agent_enabled() {
        return false;
    }
    match std::env::var("MAC_STATS_NO_AI") {
        Ok(v) if matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes") => {
            info!(
                target: "mac_stats::ai_agent_stack",
                "MAC_STATS_NO_AI set — skipping Ollama auto-enable"
            );
            return false;
        }
        _ => {}
    }
    if Config::ai_agent_ollama_auto_probe_done() {
        debug!(
            target: "mac_stats::ai_agent_stack",
            "aiAgentOllamaAutoProbeDone — skipping Ollama auto-enable"
        );
        return false;
    }
    if !local_ollama_api_reachable() {
        debug!(
            target: "mac_stats::ai_agent_stack",
            "local Ollama not reachable — leaving aiAgentEnabled=false (will retry next launch)"
        );
        return false;
    }

    info!(
        target: "mac_stats::ai_agent_stack",
        "Local Ollama API reachable — enabling aiAgentEnabled (one-shot auto-enable)"
    );
    if let Err(e) = Config::set_ai_agent_enabled(true) {
        warn!(
            target: "mac_stats::ai_agent_stack",
            "Ollama auto-enable: set_ai_agent_enabled failed: {}",
            e
        );
        return false;
    }
    if let Err(e) = Config::set_ai_agent_ollama_auto_probe_done(true) {
        warn!(
            target: "mac_stats::ai_agent_stack",
            "Ollama auto-enable: set probe-done failed: {}",
            e
        );
    }
    if let Some(app) = APP_HANDLE.get() {
        let _ = app.emit("ai-agent-enabled-changed", true);
    }
    true
}

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

#[cfg(test)]
mod tests {
    use super::local_ollama_base_url;

    #[test]
    fn local_ollama_base_url_default() {
        // Do not assert against a polluted OLLAMA_HOST from the parent process in CI;
        // only check scheme/port shape when unset is hard — normalize helper instead.
        let u = local_ollama_base_url();
        assert!(u.starts_with("http://") || u.starts_with("https://"));
        assert!(!u.ends_with('/'));
    }

    #[test]
    fn local_ollama_base_url_adds_scheme() {
        std::env::set_var("OLLAMA_HOST", "127.0.0.1:11434");
        let u = local_ollama_base_url();
        std::env::remove_var("OLLAMA_HOST");
        assert_eq!(u, "http://127.0.0.1:11434");
    }

    #[test]
    fn local_ollama_base_url_strips_trailing_slash() {
        std::env::set_var("OLLAMA_HOST", "http://localhost:11434/");
        let u = local_ollama_base_url();
        std::env::remove_var("OLLAMA_HOST");
        assert_eq!(u, "http://localhost:11434");
    }
}
