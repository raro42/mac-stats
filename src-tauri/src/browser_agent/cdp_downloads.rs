//! Auxiliary CDP WebSocket client for `Browser.setDownloadBehavior` and download progress events.
//! headless_chrome keeps `Browser::call_method` private; a second connection receives
//! `Browser.downloadWillBegin` / `Browser.downloadProgress` while the main session drives the tab.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, SystemTime};

use serde_json::{json, Value};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Message};

use crate::{mac_stats_debug, mac_stats_info, mac_stats_warn};

static CDP_WS_URL: OnceLock<Mutex<Option<String>>> = OnceLock::new();

fn cdp_ws_url_store() -> &'static Mutex<Option<String>> {
    CDP_WS_URL.get_or_init(|| Mutex::new(None))
}

pub(crate) fn store_cdp_ws_url(url: Option<String>) {
    if let Ok(mut g) = cdp_ws_url_store().lock() {
        *g = url;
    }
}

pub(crate) fn clear_stored_cdp_ws_url() {
    if let Ok(mut g) = cdp_ws_url_store().lock() {
        g.take();
    }
}

/// Best-effort WebSocket URL for auxiliary Browser-domain CDP (same browser as headless_chrome).
pub fn peek_cdp_ws_url() -> Option<String> {
    cdp_ws_url_store()
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .filter(|s| !s.is_empty() && !s.contains("not running"))
}

fn set_tcp_read_timeout(stream: &mut MaybeTlsStream<TcpStream>, t: Option<Duration>) {
    if let MaybeTlsStream::Plain(s) = stream {
        let _ = s.set_read_timeout(t);
    }
}

#[derive(Debug, Default, Clone)]
struct GuidState {
    completed_path: Option<PathBuf>,
    received: u64,
    total: u64,
}

fn is_partial_download_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.ends_with(".crdownload") || lower.ends_with(".part")
}

pub(crate) fn download_dir_file_snapshot(dir: &Path) -> HashSet<PathBuf> {
    let Ok(rd) = fs::read_dir(dir) else {
        return HashSet::new();
    };
    rd.filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .collect()
}

fn normalize_download_path(p: &Path, download_dir: &Path) -> Option<PathBuf> {
    let ok = fs::metadata(p).ok()?;
    if !ok.is_file() {
        return None;
    }
    let canon_file = fs::canonicalize(p).ok()?;
    let canon_dir = fs::canonicalize(download_dir).ok()?;
    if canon_file.starts_with(&canon_dir) {
        Some(canon_file)
    } else {
        None
    }
}

/// Send `Browser.setDownloadBehavior` and read events until `stop` is true (check between reads).
fn aux_download_session(
    ws_url: &str,
    download_dir: &Path,
    stop: &AtomicBool,
    states: &Mutex<HashMap<String, GuidState>>,
) -> Result<(), String> {
    let (mut socket, _) =
        connect(ws_url).map_err(|e| format!("CDP download aux: websocket connect: {}", e))?;
    set_tcp_read_timeout(socket.get_mut(), Some(Duration::from_millis(300)));

    let path_str = download_dir.to_string_lossy().to_string();
    let cmd = json!({
        "id": 1,
        "method": "Browser.setDownloadBehavior",
        "params": {
            "behavior": "allowAndName",
            "downloadPath": path_str,
            "eventsEnabled": true
        }
    });
    socket
        .send(Message::Text(cmd.to_string().into()))
        .map_err(|e| format!("CDP download aux: send setDownloadBehavior: {}", e))?;

    let mut got_set_result = false;

    while !stop.load(Ordering::SeqCst) {
        match socket.read() {
            Ok(Message::Text(t)) => {
                let v: Value = match serde_json::from_str(t.as_str()) {
                    Ok(x) => x,
                    Err(_) => continue,
                };
                if let Some(id) = v.get("id") {
                    if v.get("error").is_some() {
                        mac_stats_warn!(
                            "browser/cdp",
                            "CDP download aux: CDP error response: {}",
                            crate::logging::ellipse(t.as_str(), 200)
                        );
                    }
                    if id == 1 {
                        got_set_result = true;
                    }
                    continue;
                }
                let method = v
                    .get("method")
                    .and_then(|m| m.as_str())
                    .unwrap_or("")
                    .to_string();
                let params = v.get("params").cloned().unwrap_or(json!({}));
                match method.as_str() {
                    "Browser.downloadWillBegin" => {
                        let guid = params
                            .get("guid")
                            .and_then(|g| g.as_str())
                            .unwrap_or("")
                            .to_string();
                        if guid.is_empty() {
                            continue;
                        }
                        let suggested = params
                            .get("suggestedFilename")
                            .and_then(|g| g.as_str())
                            .unwrap_or("")
                            .to_string();
                        mac_stats_debug!(
                            "browser/cdp",
                            "CDP download aux: downloadWillBegin guid={} file={}",
                            crate::logging::ellipse(&guid, 12),
                            crate::logging::ellipse(&suggested, 40)
                        );
                        if let Ok(mut map) = states.lock() {
                            map.entry(guid).or_default();
                        }
                    }
                    "Browser.downloadProgress" => {
                        let guid = params
                            .get("guid")
                            .and_then(|g| g.as_str())
                            .unwrap_or("")
                            .to_string();
                        if guid.is_empty() {
                            continue;
                        }
                        let state = params
                            .get("state")
                            .and_then(|g| g.as_str())
                            .unwrap_or("");
                        let received = params
                            .get("receivedBytes")
                            .and_then(|x| x.as_f64())
                            .unwrap_or(0.0) as u64;
                        let total = params
                            .get("totalBytes")
                            .and_then(|x| x.as_f64())
                            .unwrap_or(0.0) as u64;
                        let file_path = params
                            .get("filePath")
                            .and_then(|g| g.as_str())
                            .map(PathBuf::from);
                        if let Ok(mut map) = states.lock() {
                            let ent = map.entry(guid.clone()).or_default();
                            ent.received = received;
                            ent.total = total;
                            if state == "completed" {
                                if let Some(ref fp) = file_path {
                                    if let Some(canon) = normalize_download_path(fp, download_dir) {
                                        ent.completed_path = Some(canon);
                                    }
                                }
                                mac_stats_info!(
                                    "browser/cdp",
                                    "CDP download aux: download completed guid={} path={}",
                                    crate::logging::ellipse(&guid, 12),
                                    ent.completed_path
                                        .as_ref()
                                        .map(|p| p.display().to_string())
                                        .unwrap_or_else(|| "(no path)".into())
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => {}
            Err(e) => {
                let es = e.to_string();
                if es.contains("Connection reset")
                    || es.contains("closed")
                    || es.contains("WebSocket")
                {
                    break;
                }
            }
        }
    }

    let _ = socket.close(None);
    if !got_set_result {
        mac_stats_debug!(
            "browser/cdp",
            "CDP download aux: did not observe id=1 response (session may still be OK)"
        );
    }
    Ok(())
}

/// One-shot Browser.setDownloadBehavior (no long-lived listener). Used when aux thread is undesirable.
pub fn apply_browser_download_behavior_best_effort(ws_url: &str, download_dir: &Path) {
    let Ok((mut socket, _)) = connect(ws_url) else {
        return;
    };
    set_tcp_read_timeout(socket.get_mut(), Some(Duration::from_secs(2)));
    let path_str = download_dir.to_string_lossy().to_string();
    let id = 2u64;
    let cmd = json!({
        "id": id,
        "method": "Browser.setDownloadBehavior",
        "params": {
            "behavior": "allowAndName",
            "downloadPath": path_str,
            "eventsEnabled": true
        }
    });
    if socket
        .send(Message::Text(cmd.to_string().into()))
        .is_err()
    {
        return;
    }
    for _ in 0..20 {
        if let Ok(Message::Text(t)) = socket.read() {
            if let Ok(v) = serde_json::from_str::<Value>(t.as_str()) {
                if v.get("id").and_then(|x| x.as_u64()) == Some(id) {
                    break;
                }
            }
        }
    }
    let _ = socket.close(None);
}

/// Run an auxiliary CDP session in a background thread; call `stop` when the browser action finished
/// plus [`POST_ACTION_DOWNLOAD_WAIT`]. Collects completed download paths under `download_dir`.
pub fn spawn_download_aux_listener(
    ws_url: String,
    download_dir: PathBuf,
    stop: Arc<AtomicBool>,
    out: Arc<Mutex<Vec<PathBuf>>>,
) {
    std::thread::spawn(move || {
        let states: Mutex<HashMap<String, GuidState>> = Mutex::new(HashMap::new());
        if let Err(e) = aux_download_session(&ws_url, &download_dir, &stop, &states) {
            mac_stats_debug!(
                "browser/cdp",
                "CDP download aux session ended: {}",
                crate::logging::ellipse(&e, 120)
            );
        }
        let mut paths: Vec<PathBuf> = states
            .lock()
            .map(|m| {
                m.values()
                    .filter_map(|s| s.completed_path.clone())
                    .collect()
            })
            .unwrap_or_default();
        paths.sort();
        paths.dedup();
        if let Ok(mut g) = out.lock() {
            *g = paths;
        }
    });
}

/// Wall-clock wait after navigate/click before signaling the aux listener to stop.
pub const POST_ACTION_DOWNLOAD_WAIT: Duration = Duration::from_secs(3);

fn human_bytes(n: u64) -> String {
    if n < 1024 {
        format!("{} B", n)
    } else if n < 1024 * 1024 {
        format!("{:.1} KB", n as f64 / 1024.0)
    } else {
        format!("{:.1} MB", n as f64 / (1024.0 * 1024.0))
    }
}

/// Merge CDP-reported paths with new files on disk (excluding partial downloads).
pub fn merge_with_directory_diff(
    download_dir: &Path,
    pre: &HashSet<PathBuf>,
    cdp_paths: &[PathBuf],
) -> Vec<PathBuf> {
    let post = download_dir_file_snapshot(download_dir);
    let mut out: Vec<PathBuf> = cdp_paths.to_vec();
    for p in post.difference(pre) {
        if is_partial_download_name(&p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()) {
            continue;
        }
        if let Some(c) = normalize_download_path(p, download_dir) {
            if !out.contains(&c) {
                out.push(c);
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

pub fn format_download_attachment_note(paths: &[PathBuf]) -> String {
    if paths.is_empty() {
        return String::new();
    }
    let mut s = String::from("\n**Download:** ");
    for p in paths {
        let sz = fs::metadata(p).map(|m| m.len()).unwrap_or(0);
        s.push_str(&format!(
            "{} ({}) ",
            p.display(),
            human_bytes(sz)
        ));
    }
    s.push_str("\n");
    for p in paths {
        s.push_str(&format!("[download: {}]\n", p.display()));
    }
    s
}

/// Delete completed browser-download files older than `max_age` (by mtime). Logs at info when anything removed.
pub fn prune_old_browser_downloads(max_age: Duration) {
    let dir = crate::config::Config::browser_downloads_dir();
    if !dir.is_dir() {
        return;
    }
    let Ok(rd) = fs::read_dir(&dir) else {
        return;
    };
    let mut removed = 0u32;
    let mut freed = 0u64;
    let now = SystemTime::now();
    for ent in rd.flatten() {
        let path = ent.path();
        let Ok(meta) = fs::metadata(&path) else {
            continue;
        };
        if !meta.is_file() {
            continue;
        }
        let Ok(mt) = meta.modified() else {
            continue;
        };
        let age = now.duration_since(mt).unwrap_or(Duration::ZERO);
        if age > max_age {
            let sz = meta.len();
            if fs::remove_file(&path).is_ok() {
                removed += 1;
                freed += sz;
            }
        }
    }
    if removed > 0 {
        crate::mac_stats_info!(
            "browser/downloads",
            "Pruned {} browser download file(s), freed {} bytes (older than {:?})",
            removed,
            freed,
            max_age
        );
    }
}
