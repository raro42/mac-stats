//! Optional CDP `Tracing` capture for operator diagnostics (Chrome trace JSON, `chrome://tracing`-compatible).
//!
//! Uses a **dedicated WebSocket** to the same `webSocketDebuggerUrl` as [`super::cdp_grant_permissions`]
//! so we can call `Tracing.start` / `Tracing.end` / `IO.read` without `headless_chrome`'s private `Browser::call_method`.
//! When disabled (default), this module does **nothing** — no connections, no CDP overhead.

use std::net::TcpStream;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use base64::Engine;
use serde_json::{json, Value};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Message, WebSocket};

use super::artifact_atomic;
use crate::config::Config;
use crate::{mac_stats_debug, mac_stats_info, mac_stats_warn};

fn set_tcp_read_timeout(stream: &mut MaybeTlsStream<TcpStream>, t: Option<Duration>) {
    if let MaybeTlsStream::Plain(s) = stream {
        let _ = s.set_read_timeout(t);
    }
}

struct TraceSocket {
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    next_id: u64,
}

impl TraceSocket {
    fn next_cmd_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        id
    }

    fn send_cmd(&mut self, id: u64, method: &str, params: Value) -> Result<(), String> {
        let cmd = json!({
            "id": id,
            "method": method,
            "params": params
        });
        let text = cmd.to_string();
        self.socket
            .send(Message::Text(text.into()))
            .map_err(|e| format!("CDP trace: send {} failed: {}", method, e))
    }

    fn wait_response_ok(&mut self, expect_id: u64, label: &str) -> Result<(), String> {
        const MAX_READS: usize = 200;
        for _ in 0..MAX_READS {
            match self.socket.read() {
                Ok(Message::Text(t)) => {
                    let Ok(v) = serde_json::from_str::<Value>(t.as_str()) else {
                        continue;
                    };
                    if v.get("id").and_then(|x| x.as_u64()) != Some(expect_id) {
                        continue;
                    }
                    if let Some(err) = v.get("error") {
                        return Err(format!("CDP trace: {} CDP error: {}", label, err));
                    }
                    return Ok(());
                }
                Ok(Message::Ping(p)) => {
                    let _ = self.socket.send(Message::Pong(p));
                }
                Ok(_) => {}
                Err(e) => return Err(format!("CDP trace: {} read: {}", label, e)),
            }
        }
        Err(format!(
            "CDP trace: no response for id={} ({})",
            expect_id, label
        ))
    }

    fn drain_until_tracing_complete(
        &mut self,
        end_ack_id: u64,
    ) -> Result<(Option<String>, bool), String> {
        let mut saw_end_ack = false;
        let mut saw_complete = false;
        let mut stream: Option<String> = None;
        let mut data_loss = false;
        const MAX_READS: usize = 5000;
        for _ in 0..MAX_READS {
            match self.socket.read() {
                Ok(Message::Text(t)) => {
                    let Ok(v) = serde_json::from_str::<Value>(t.as_str()) else {
                        continue;
                    };
                    if v.get("method").and_then(|m| m.as_str()) == Some("Tracing.tracingComplete") {
                        saw_complete = true;
                        if let Some(p) = v.get("params") {
                            data_loss = p
                                .get("dataLossOccurred")
                                .and_then(|b| b.as_bool())
                                .unwrap_or(false);
                            stream = p
                                .get("stream")
                                .and_then(|s| s.as_str())
                                .map(std::string::ToString::to_string);
                        }
                    } else if v.get("id").and_then(|x| x.as_u64()) == Some(end_ack_id) {
                        if v.get("error").is_some() {
                            return Err(format!(
                                "CDP trace: Tracing.end error: {}",
                                v.get("error").unwrap()
                            ));
                        }
                        saw_end_ack = true;
                    }
                    if saw_end_ack && saw_complete {
                        return Ok((stream, data_loss));
                    }
                }
                Ok(Message::Ping(p)) => {
                    let _ = self.socket.send(Message::Pong(p));
                }
                Ok(_) => {}
                Err(e) => return Err(format!("CDP trace: drain read: {}", e)),
            }
        }
        Err(format!(
            "CDP trace: timeout waiting for Tracing.end ack + tracingComplete (end_ack={} complete={})",
            saw_end_ack, saw_complete
        ))
    }

    fn read_stream_to_vec(&mut self, handle: &str, max_bytes: u64) -> Result<Vec<u8>, String> {
        let mut out: Vec<u8> = Vec::new();
        let max_us = max_bytes as usize;
        let mut offset: u64 = 0;
        loop {
            let id = self.next_cmd_id();
            self.send_cmd(
                id,
                "IO.read",
                json!({
                    "handle": handle,
                    "offset": offset
                }),
            )?;
            let mut got = false;
            let mut chunk_eof = false;
            const MAX_READS: usize = 50;
            for _ in 0..MAX_READS {
                match self.socket.read() {
                    Ok(Message::Text(t)) => {
                        let Ok(v) = serde_json::from_str::<Value>(t.as_str()) else {
                            continue;
                        };
                        if v.get("id").and_then(|x| x.as_u64()) != Some(id) {
                            continue;
                        }
                        if let Some(err) = v.get("error") {
                            return Err(format!("CDP trace: IO.read error: {}", err));
                        }
                        let Some(res) = v.get("result") else {
                            continue;
                        };
                        got = true;
                        chunk_eof = res.get("eof").and_then(|b| b.as_bool()).unwrap_or(false);
                        let data = res
                            .get("data")
                            .and_then(|d| d.as_str())
                            .unwrap_or("")
                            .to_string();
                        let b64 = res
                            .get("base64Encoded")
                            .and_then(|b| b.as_bool())
                            .unwrap_or(false);
                        let bytes = if b64 {
                            base64::engine::general_purpose::STANDARD
                                .decode(data.as_bytes())
                                .map_err(|e| format!("CDP trace: IO.read base64: {}", e))?
                        } else {
                            data.into_bytes()
                        };
                        let room = max_us.saturating_sub(out.len());
                        if bytes.len() > room {
                            mac_stats_warn!(
                                "browser/cdp",
                                "CDP trace: trace size exceeds browserCdpTraceMaxFileBytes ({} bytes); truncating",
                                max_bytes
                            );
                            out.extend_from_slice(&bytes[..room]);
                            return Ok(out);
                        }
                        out.extend_from_slice(&bytes);
                        offset = out.len() as u64;
                        break;
                    }
                    Ok(Message::Ping(p)) => {
                        let _ = self.socket.send(Message::Pong(p));
                    }
                    Ok(_) => {}
                    Err(e) => return Err(format!("CDP trace: IO.read transport: {}", e)),
                }
            }
            if !got {
                return Err("CDP trace: no IO.read response".to_string());
            }
            if chunk_eof {
                break;
            }
        }
        Ok(out)
    }

    fn close_io_stream(&mut self, handle: &str) {
        let id = self.next_cmd_id();
        let _ = self.send_cmd(id, "IO.close", json!({ "handle": handle }));
        let _ = self.wait_response_ok(id, "IO.close");
    }
}

enum RecorderState {
    Idle,
    Recording(TraceSocket),
}

static RECORDER: OnceLock<Mutex<RecorderState>> = OnceLock::new();
static TRACE_DEADLINE: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();
static NEXT_TRACE_SESSION_ID: AtomicU64 = AtomicU64::new(1);
static LOGGED_TRACE_DISABLED_EXPIRED: AtomicBool = AtomicBool::new(false);

fn recorder() -> &'static Mutex<RecorderState> {
    RECORDER.get_or_init(|| Mutex::new(RecorderState::Idle))
}

fn trace_deadline_store() -> &'static Mutex<Option<Instant>> {
    TRACE_DEADLINE.get_or_init(|| Mutex::new(None))
}

fn past_trace_deadline() -> bool {
    let Ok(g) = trace_deadline_store().lock() else {
        return false;
    };
    match *g {
        Some(t) => Instant::now() > t,
        None => false,
    }
}

fn ensure_trace_deadline_from_config() {
    let mins = Config::browser_cdp_trace_wall_clock_minutes();
    if mins == 0 {
        return;
    }
    let Ok(mut g) = trace_deadline_store().lock() else {
        return;
    };
    if g.is_none() {
        *g = Some(Instant::now() + Duration::from_secs(mins * 60));
        mac_stats_info!(
            "browser/cdp",
            "CDP trace: wall-clock recording limit {} minute(s) from first start",
            mins
        );
    }
}

/// Start CDP tracing on a fresh auxiliary WebSocket when the operator enabled it and we have a browser WS URL.
pub(crate) fn maybe_start_recording_after_cdp_session_ready() {
    if !Config::browser_cdp_trace_enabled() {
        return;
    }
    if past_trace_deadline() {
        if !LOGGED_TRACE_DISABLED_EXPIRED.swap(true, Ordering::SeqCst) {
            mac_stats_info!(
                "browser/cdp",
                "CDP trace: not starting (wall-clock limit elapsed); disable or extend browserCdpTraceWallClockMinutes / MAC_STATS_BROWSER_CDP_TRACE_MINUTES"
            );
        }
        return;
    }
    let Some(ws_url) = super::cdp_downloads::peek_cdp_ws_url() else {
        mac_stats_debug!(
            "browser/cdp",
            "CDP trace: skip start (no CDP WebSocket URL — headless transport or not connected yet)"
        );
        return;
    };
    if ws_url.contains("not running") {
        return;
    }

    let mut guard = match recorder().lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    if matches!(*guard, RecorderState::Recording(_)) {
        return;
    }

    let Ok((mut socket, _)) = connect(ws_url.as_str()) else {
        mac_stats_warn!(
            "browser/cdp",
            "CDP trace: Tracing.start websocket connect failed (recording skipped)"
        );
        return;
    };
    set_tcp_read_timeout(socket.get_mut(), Some(Duration::from_secs(30)));

    let mut ts = TraceSocket {
        socket,
        next_id: 10_000,
    };
    let start_id = ts.next_cmd_id();
    let params = json!({
        "transferMode": "ReturnAsStream",
        "streamFormat": "json",
        "streamCompression": "none",
        "traceConfig": {
            "recordMode": "recordUntilFull",
            "traceBufferSizeInKb": 4096,
            "includedCategories": [
                "-*",
                "devtools.timeline",
                "disabled-by-default-devtools.timeline",
                "browser",
                "blink.user_timing",
                "v8.execute"
            ]
        }
    });
    if let Err(e) = ts.send_cmd(start_id, "Tracing.start", params) {
        mac_stats_warn!("browser/cdp", "{}", e);
        let _ = ts.socket.close(None);
        return;
    }
    if let Err(e) = ts.wait_response_ok(start_id, "Tracing.start") {
        mac_stats_warn!("browser/cdp", "{}", e);
        let _ = ts.socket.close(None);
        return;
    }

    ensure_trace_deadline_from_config();
    let sid = NEXT_TRACE_SESSION_ID.fetch_add(1, Ordering::SeqCst);
    mac_stats_info!(
        "browser/cdp",
        "CDP trace: Tracing.start OK (session_id={}, ReturnAsStream json); will finalize on browser session end or idle eviction",
        sid
    );
    *guard = RecorderState::Recording(ts);
}

/// Stop tracing, persist under `~/.mac-stats/traces/`, prune retention; best-effort (logs on failure).
pub(crate) fn stop_and_persist_best_effort(ws_url_hint: Option<&str>, reason: &str) {
    let _ = ws_url_hint; // same browser as peek; socket already bound
    let mut guard = match recorder().lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    let RecorderState::Recording(mut ts) = std::mem::replace(&mut *guard, RecorderState::Idle)
    else {
        return;
    };
    drop(guard);

    set_tcp_read_timeout(ts.socket.get_mut(), Some(Duration::from_secs(120)));
    let end_id = ts.next_cmd_id();
    if let Err(e) = ts.send_cmd(end_id, "Tracing.end", json!({})) {
        mac_stats_warn!(
            "browser/cdp",
            "CDP trace: Tracing.end send failed ({}): {}",
            reason,
            e
        );
        let _ = ts.socket.close(None);
        return;
    }

    let (stream, data_loss) = match ts.drain_until_tracing_complete(end_id) {
        Ok(x) => x,
        Err(e) => {
            mac_stats_warn!(
                "browser/cdp",
                "CDP trace: finalize failed after Tracing.end ({}): {}",
                reason,
                e
            );
            let _ = ts.socket.close(None);
            return;
        }
    };
    if data_loss {
        mac_stats_warn!(
            "browser/cdp",
            "CDP trace: Chrome reported dataLossOccurred=1 (ring buffer wrapped); trace may be incomplete ({})",
            reason
        );
    }
    let Some(handle) = stream.filter(|s| !s.is_empty()) else {
        mac_stats_warn!(
            "browser/cdp",
            "CDP trace: tracingComplete had no stream handle ({})",
            reason
        );
        let _ = ts.socket.close(None);
        return;
    };

    let max_bytes = Config::browser_cdp_trace_max_file_bytes();
    let bytes = match ts.read_stream_to_vec(&handle, max_bytes) {
        Ok(b) => b,
        Err(e) => {
            mac_stats_warn!(
                "browser/cdp",
                "CDP trace: IO.read stream failed ({}): {}",
                reason,
                e
            );
            ts.close_io_stream(&handle);
            let _ = ts.socket.close(None);
            return;
        }
    };
    ts.close_io_stream(&handle);
    let _ = ts.socket.close(None);

    if bytes.is_empty() {
        mac_stats_warn!(
            "browser/cdp",
            "CDP trace: empty trace payload after IO.read ({})",
            reason
        );
        return;
    }

    let dir = Config::browser_cdp_traces_dir();
    if let Err(e) = std::fs::create_dir_all(&dir) {
        mac_stats_warn!(
            "browser/cdp",
            "CDP trace: create traces dir {:?}: {}",
            dir,
            e
        );
        return;
    }
    let name = format!(
        "{}_cdp_trace.json",
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    );
    match artifact_atomic::write_bytes_atomic_same_dir(&dir, &name, &bytes) {
        Ok(path) => {
            mac_stats_info!(
                "browser/cdp",
                "CDP trace: wrote {} ({} bytes, reason={})",
                path.display(),
                bytes.len(),
                reason
            );
            prune_traces_dir_best_effort(&dir);
        }
        Err(e) => {
            mac_stats_warn!(
                "browser/cdp",
                "CDP trace: atomic write failed ({}): {}",
                reason,
                e
            );
        }
    }
}

fn prune_traces_dir_best_effort(dir: &Path) {
    let max_files = Config::browser_cdp_trace_max_retained_files();
    if max_files == 0 {
        return;
    }
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<std::path::PathBuf> = rd
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|s| s.ends_with("_cdp_trace.json"))
        })
        .collect();
    if entries.len() <= max_files {
        return;
    }
    entries.sort_by_key(|p| {
        std::fs::metadata(p)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0)
    });
    let remove_n = entries.len().saturating_sub(max_files);
    let mut freed: u64 = 0;
    let mut removed: usize = 0;
    for p in entries.into_iter().take(remove_n) {
        if let Ok(m) = std::fs::metadata(&p) {
            freed = freed.saturating_add(m.len());
        }
        if std::fs::remove_file(&p).is_ok() {
            removed += 1;
        }
    }
    if removed > 0 {
        mac_stats_info!(
            "browser/cdp",
            "CDP trace: pruned {} oldest file(s) (maxRetainedFiles={}), ~{} bytes freed",
            removed,
            max_files,
            freed
        );
    }
}

/// Reset deadline tracking when a new process wants fresh timing (optional — currently unused).
#[allow(dead_code)]
pub(crate) fn reset_trace_deadline_for_tests() {
    if let Ok(mut g) = trace_deadline_store().lock() {
        *g = None;
    }
    LOGGED_TRACE_DISABLED_EXPIRED.store(false, Ordering::SeqCst);
}
