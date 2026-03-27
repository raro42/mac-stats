//! Second CDP WebSocket client to observe `Target.targetCrashed` for the automation tab.
//!
//! `headless_chrome` receives this event on the browser connection but treats it as unhandled
//! trace noise; mac-stats needs an immediate signal to invalidate the cached session when the
//! focused page renderer dies while the browser process stays up.

use std::net::TcpStream;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use serde_json::{json, Value};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Message};

use crate::{mac_stats_debug, mac_stats_warn};

/// Bumped whenever the cached CDP session is torn down so prior side-listener threads exit.
static LISTENER_GEN: AtomicU64 = AtomicU64::new(0);

pub(crate) fn invalidate_listener_generation() {
    LISTENER_GEN.fetch_add(1, Ordering::SeqCst);
}

fn set_tcp_read_timeout(stream: &mut MaybeTlsStream<TcpStream>, t: Option<Duration>) {
    if let MaybeTlsStream::Plain(s) = stream {
        let _ = s.set_read_timeout(t);
    }
}

fn run_side_listener(ws_url: String, session_token: u64) {
    let (mut socket, _) = match connect(ws_url.as_str()) {
        Ok(x) => x,
        Err(e) => {
            mac_stats_debug!(
                "browser/cdp",
                "CDP target-crash side listener: WebSocket connect failed: {}",
                e
            );
            return;
        }
    };
    set_tcp_read_timeout(socket.get_mut(), Some(Duration::from_millis(400)));

    // Omit optional `filter` — Chrome 146+ rejects `"filter": null` (expects array or absent key).
    let cmd = json!({
        "id": 1,
        "method": "Target.setDiscoverTargets",
        "params": { "discover": true }
    });
    if let Err(e) = socket.send(Message::Text(cmd.to_string().into())) {
        mac_stats_debug!(
            "browser/cdp",
            "CDP target-crash side listener: send setDiscoverTargets failed: {}",
            e
        );
        return;
    }

    loop {
        if LISTENER_GEN.load(Ordering::SeqCst) != session_token {
            break;
        }
        match socket.read() {
            Ok(Message::Text(t)) => {
                let Ok(v) = serde_json::from_str::<Value>(t.as_str()) else {
                    continue;
                };
                if v.get("id").is_some() {
                    if v.get("error").is_some() {
                        mac_stats_warn!(
                            "browser/cdp",
                            "CDP target-crash side listener: CDP error response: {}",
                            crate::logging::ellipse(t.as_str(), 200)
                        );
                    } else if v.get("id").and_then(|x| x.as_u64()) == Some(1) {
                        mac_stats_debug!(
                            "browser/cdp",
                            "CDP target-crash side listener: Target.setDiscoverTargets ok (listening for Target.targetCrashed)"
                        );
                    }
                    continue;
                }
                let method = v.get("method").and_then(|m| m.as_str()).unwrap_or("");
                if method == "Target.targetCrashed" {
                    if let Some(tid) = v
                        .get("params")
                        .and_then(|p| p.get("targetId"))
                        .and_then(|x| x.as_str())
                    {
                        super::notify_target_renderer_crashed_side(tid);
                    }
                }
            }
            Ok(Message::Ping(p)) => {
                let _ = socket.send(Message::Pong(p));
            }
            Ok(Message::Close(_)) | Err(_) => break,
            Ok(_) => {}
        }
    }
    let _ = socket.close(None);
}

/// Spawn a background thread that shares the browser CDP WebSocket URL and listens for renderer crashes.
pub(crate) fn spawn_target_crash_side_listener(ws_url: &str) {
    let s = ws_url.trim().to_string();
    if s.is_empty() || s.contains("not running") {
        return;
    }
    if !s.starts_with("ws://") && !s.starts_with("wss://") {
        return;
    }
    let session_token = LISTENER_GEN.load(Ordering::SeqCst);
    if std::thread::Builder::new()
        .name("mac-stats-cdp-target-crash".into())
        .spawn(move || run_side_listener(s, session_token))
        .is_err()
    {
        mac_stats_warn!(
            "browser/cdp",
            "CDP target-crash side listener: failed to spawn background thread"
        );
    }
}
