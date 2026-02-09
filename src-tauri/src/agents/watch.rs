//! Watch ~/.mac-stats/agents/ and ~/.mac-stats/skills/ for file changes and emit Tauri events
//! so the frontend (and next load_agents/load_skills) see updates without restart.

use crate::config::Config;
use crate::state::APP_HANDLE;
use tauri::Manager;
use notify::{RecursiveMode, Watcher};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Debounce interval: wait this long after the last filesystem event before emitting.
const DEBOUNCE_MS: u64 = 500;

/// Spawn a background thread that watches the agents and skills directories.
/// When any file under those dirs changes, emits "agents-changed" and "skills-changed"
/// after a short debounce. Call once after APP_HANDLE is set (e.g. in setup).
pub fn spawn_agents_and_skills_watcher() {
    let app_handle = match APP_HANDLE.get() {
        Some(h) => h.clone(),
        None => {
            warn!("Agents watch: APP_HANDLE not set, skipping file watcher");
            return;
        }
    };

    thread::spawn(move || {
        let agents_dir = Config::agents_dir();
        let skills_dir = Config::skills_dir();

        let (tx, rx) = mpsc::channel();
        let mut watcher = match notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if tx.send(res).is_err() {
                // Receiver dropped, watcher shutting down
            }
        }) {
            Ok(w) => w,
            Err(e) => {
                warn!("Agents watch: failed to create watcher: {}", e);
                return;
            }
        };

        if agents_dir.exists() {
            if watcher.watch(&agents_dir, RecursiveMode::Recursive).is_err() {
                warn!("Agents watch: failed to watch agents dir {:?}", agents_dir);
            } else {
                info!("Agents watch: watching {:?}", agents_dir);
            }
        } else {
            debug!("Agents watch: agents dir does not exist yet {:?}", agents_dir);
        }

        if skills_dir.exists() {
            if watcher.watch(&skills_dir, RecursiveMode::Recursive).is_err() {
                warn!("Agents watch: failed to watch skills dir {:?}", skills_dir);
            } else {
                info!("Agents watch: watching {:?}", skills_dir);
            }
        } else {
            debug!("Agents watch: skills dir does not exist yet {:?}", skills_dir);
        }

        let mut last_event = Instant::now();
        let mut pending = false;

        loop {
            let timeout = if pending {
                let elapsed = last_event.elapsed();
                if elapsed >= Duration::from_millis(DEBOUNCE_MS) {
                    pending = false;
                    let _ = app_handle.emit_all("agents-changed", ());
                    let _ = app_handle.emit_all("skills-changed", ());
                    debug!("Agents watch: emitted agents-changed and skills-changed");
                    Duration::from_millis(DEBOUNCE_MS)
                } else {
                    Duration::from_millis(DEBOUNCE_MS) - elapsed
                }
            } else {
                Duration::from_millis(DEBOUNCE_MS)
            };

            match rx.recv_timeout(timeout) {
                Ok(Ok(_event)) => {
                    last_event = Instant::now();
                    pending = true;
                }
                Ok(Err(e)) => {
                    debug!("Agents watch: event error: {:?}", e);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });
}
