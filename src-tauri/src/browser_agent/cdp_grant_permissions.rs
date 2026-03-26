//! One-shot CDP `Browser.grantPermissions` over an auxiliary WebSocket.
//!
//! `headless_chrome` keeps [`Browser::call_method`](headless_chrome::Browser) private, so we mirror
//! [`cdp_downloads`](super::cdp_downloads): open a second connection to the same `webSocketDebuggerUrl`.

use std::net::TcpStream;
use std::time::Duration;

use headless_chrome::protocol::cdp::Browser::{GrantPermissions, PermissionType};
use serde_json::{json, Value};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Message};

use crate::{mac_stats_debug, mac_stats_info, mac_stats_warn};

fn set_tcp_read_timeout(stream: &mut MaybeTlsStream<TcpStream>, t: Option<Duration>) {
    if let MaybeTlsStream::Plain(s) = stream {
        let _ = s.set_read_timeout(t);
    }
}

/// Parse a config token into a CDP `Browser.PermissionType`. Unknown strings are rejected (operator typo guard).
fn parse_permission_token(raw: &str) -> Option<PermissionType> {
    let t = raw.trim();
    if t.is_empty() {
        return None;
    }
    serde_json::from_value(Value::String(t.to_string())).ok()
}

/// Grant configured permissions once per new CDP session. No-op when `raw` is empty or all tokens invalid.
/// On CDP or transport failure: **warn** and return (session continues).
pub(crate) fn apply_browser_grant_permissions_best_effort(ws_url: &str, raw: &[String]) {
    if raw.is_empty() {
        return;
    }
    let mut permissions: Vec<PermissionType> = Vec::new();
    for s in raw {
        match parse_permission_token(s) {
            Some(p) => permissions.push(p),
            None => {
                mac_stats_warn!(
                    "browser/cdp",
                    "Browser agent [CDP]: skipping unknown browserCdpGrantPermissions entry {:?} (not a CDP Browser.PermissionType)",
                    s.chars().take(80).collect::<String>()
                );
            }
        }
    }
    if permissions.is_empty() {
        mac_stats_warn!(
            "browser/cdp",
            "Browser agent [CDP]: browserCdpGrantPermissions had no valid CDP permission types; grant skipped"
        );
        return;
    }

    let grant_count = permissions.len();

    let Ok((mut socket, _)) = connect(ws_url) else {
        mac_stats_warn!(
            "browser/cdp",
            "Browser agent [CDP]: Browser.grantPermissions: websocket connect failed (continuing)"
        );
        return;
    };
    set_tcp_read_timeout(socket.get_mut(), Some(Duration::from_secs(2)));

    let id = 901_u64;
    let params = GrantPermissions {
        permissions,
        origin: None,
        browser_context_id: None,
    };
    let cmd = json!({
        "id": id,
        "method": "Browser.grantPermissions",
        "params": params
    });
    if let Err(e) = socket.send(Message::Text(cmd.to_string().into())) {
        mac_stats_warn!(
            "browser/cdp",
            "Browser agent [CDP]: Browser.grantPermissions: send failed: {} (continuing)",
            e
        );
        let _ = socket.close(None);
        return;
    }

    let mut saw_response = false;
    for _ in 0..30 {
        match socket.read() {
            Ok(Message::Text(t)) => {
                let Ok(v) = serde_json::from_str::<Value>(t.as_str()) else {
                    continue;
                };
                if v.get("id").and_then(|x| x.as_u64()) != Some(id) {
                    continue;
                }
                saw_response = true;
                if let Some(err) = v.get("error") {
                    mac_stats_warn!(
                        "browser/cdp",
                        "Browser agent [CDP]: Browser.grantPermissions CDP error: {} (continuing)",
                        err
                    );
                } else {
                    mac_stats_info!(
                        "browser/cdp",
                        "Browser agent [CDP]: Browser.grantPermissions OK ({} permission type(s), global origin)",
                        grant_count
                    );
                }
                break;
            }
            Ok(_) => {}
            Err(e) => {
                mac_stats_warn!(
                    "browser/cdp",
                    "Browser agent [CDP]: Browser.grantPermissions read: {} (continuing)",
                    e
                );
                break;
            }
        }
    }
    if !saw_response {
        mac_stats_debug!(
            "browser/cdp",
            "Browser agent [CDP]: Browser.grantPermissions: no id={} response observed (continuing)",
            id
        );
    }
    let _ = socket.close(None);
}
