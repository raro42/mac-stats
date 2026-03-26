//! Optional before/after compaction hooks and CPU-window compaction lifecycle events.
//!
//! Hooks mirror `before_session_reset_export`: transcript JSONL + fire-and-forget shell, or env-only after hook.

use std::path::PathBuf;

use tauri::Manager;
use tracing::{debug, info, warn};

use crate::config::Config;
use crate::state::APP_HANDLE;

/// Per-invocation settings for compaction (hooks + optional CPU UI events).
#[derive(Clone, Debug)]
pub struct CompactionLifecycleContext {
    pub hook_source: String,
    pub hook_session_id: u64,
    pub emit_cpu_compaction_ui: bool,
}

impl Default for CompactionLifecycleContext {
    fn default() -> Self {
        Self {
            hook_source: "test".to_string(),
            hook_session_id: 0,
            emit_cpu_compaction_ui: false,
        }
    }
}

/// Emit OpenClaw-style payload for the CPU window (`stream` + `data.phase`).
pub fn emit_mac_stats_compaction_event(
    phase: &str,
    will_retry: bool,
    request_id: &str,
    ok: Option<bool>,
) {
    let Some(handle) = APP_HANDLE.get() else {
        return;
    };
    let mut data = serde_json::json!({
        "phase": phase,
        "willRetry": will_retry,
        "requestId": request_id,
    });
    if let Some(v) = ok {
        data.as_object_mut()
            .expect("object")
            .insert("ok".to_string(), serde_json::Value::Bool(v));
    }
    let payload = serde_json::json!({
        "stream": "compaction",
        "data": data,
    });
    if let Err(e) = handle.emit_all("mac-stats-compaction", payload) {
        debug!(
            target: "mac_stats::compaction",
            "emit mac-stats-compaction failed: {}",
            e
        );
    } else {
        debug!(
            target: "mac_stats::compaction",
            "emitted mac-stats-compaction phase={} request_id={}",
            phase,
            request_id
        );
    }
}

fn default_before_compaction_transcript_path() -> PathBuf {
    Config::agents_dir().join("last_session_before_compaction.jsonl")
}

/// Before compaction: optional JSONL transcript + optional shell hook (background). No-op if neither configured.
pub fn run_before_compaction_fire_and_forget(
    source: &str,
    session_id: u64,
    messages: &[(String, String)],
    request_id: &str,
) {
    let hook_raw = Config::before_compaction_hook_raw();
    let path_configured = !Config::before_compaction_transcript_path_raw().trim().is_empty();
    let hook_configured = !hook_raw.trim().is_empty();

    if !hook_configured && !path_configured {
        return;
    }

    let transcript_path: PathBuf = if path_configured {
        match Config::before_compaction_transcript_path_resolved() {
            Some(p) => p,
            None => {
                warn!(
                    target: "mac_stats::compaction",
                    "before_compaction transcript path set but could not resolve (~ requires HOME); skipping"
                );
                return;
            }
        }
    } else {
        default_before_compaction_transcript_path()
    };

    let ts = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let meta = serde_json::json!({
        "kind": "before_compaction_meta",
        "source": source,
        "session_id": session_id,
        "request_id": request_id,
        "exported_at_utc": ts,
        "message_count": messages.len(),
    });
    let mut body = String::new();
    body.push_str(&meta.to_string());
    body.push('\n');
    for (role, content) in messages {
        let line = serde_json::json!({ "role": role, "content": content });
        body.push_str(&line.to_string());
        body.push('\n');
    }

    if let Some(parent) = transcript_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            warn!(
                target: "mac_stats::compaction",
                "before_compaction could not create parent dir {}: {}",
                parent.display(),
                e
            );
        }
    }

    match std::fs::write(&transcript_path, &body) {
        Ok(()) => info!(
            target: "mac_stats::compaction",
            "before_compaction wrote {} ({} messages, source={} session_id={} request_id={})",
            transcript_path.display(),
            messages.len(),
            source,
            session_id,
            request_id
        ),
        Err(e) => {
            warn!(
                target: "mac_stats::compaction",
                "before_compaction write failed {}: {}",
                transcript_path.display(),
                e
            );
            return;
        }
    }

    if !hook_configured {
        return;
    }

    let hook_cmd = hook_raw.trim().to_string();
    let path_for_thread = transcript_path.clone();
    let source_owned = source.to_string();
    let request_owned = request_id.to_string();
    let message_count = messages.len();
    std::thread::spawn(move || {
        let path_str = path_for_thread.to_string_lossy().into_owned();
        let script = format!("{} \"$1\"", hook_cmd);
        let status = std::process::Command::new("/bin/sh")
            .arg("-c")
            .arg(&script)
            .arg("_")
            .arg(&path_str)
            .env("MAC_STATS_BEFORE_COMPACTION_TRANSCRIPT", &path_str)
            .env("MAC_STATS_BEFORE_COMPACTION_SOURCE", &source_owned)
            .env(
                "MAC_STATS_BEFORE_COMPACTION_SESSION_ID",
                format!("{}", session_id),
            )
            .env(
                "MAC_STATS_BEFORE_COMPACTION_MESSAGE_COUNT",
                format!("{}", message_count),
            )
            .env("MAC_STATS_BEFORE_COMPACTION_REQUEST_ID", &request_owned)
            .status();
        match status {
            Ok(s) if s.success() => {
                debug!(target: "mac_stats::compaction", "before_compaction hook finished OK");
            }
            Ok(s) => {
                warn!(
                    target: "mac_stats::compaction",
                    "before_compaction hook exited with status {:?}",
                    s.code()
                );
            }
            Err(e) => {
                warn!(
                    target: "mac_stats::compaction",
                    "before_compaction hook spawn failed: {}",
                    e
                );
            }
        }
    });
}

/// After a **successful** compaction only. Fire-and-forget shell; failures logged only.
pub fn run_after_compaction_fire_and_forget(
    source: &str,
    session_id: u64,
    message_count_before: usize,
    lessons_written: bool,
    request_id: &str,
) {
    let hook_raw = Config::after_compaction_hook_raw();
    if hook_raw.trim().is_empty() {
        return;
    }
    let hook_cmd = hook_raw.trim().to_string();
    let source_owned = source.to_string();
    let request_owned = request_id.to_string();
    let lw = lessons_written;
    std::thread::spawn(move || {
        let status = std::process::Command::new("/bin/sh")
            .arg("-c")
            .arg(&hook_cmd)
            .env("MAC_STATS_AFTER_COMPACTION_SOURCE", &source_owned)
            .env(
                "MAC_STATS_AFTER_COMPACTION_SESSION_ID",
                format!("{}", session_id),
            )
            .env(
                "MAC_STATS_AFTER_COMPACTION_MESSAGE_COUNT_BEFORE",
                format!("{}", message_count_before),
            )
            .env(
                "MAC_STATS_AFTER_COMPACTION_LESSONS_WRITTEN",
                if lw { "true" } else { "false" },
            )
            .env("MAC_STATS_AFTER_COMPACTION_REQUEST_ID", &request_owned)
            .status();
        match status {
            Ok(s) if s.success() => {
                debug!(target: "mac_stats::compaction", "after_compaction hook finished OK");
            }
            Ok(s) => {
                warn!(
                    target: "mac_stats::compaction",
                    "after_compaction hook exited with status {:?}",
                    s.code()
                );
            }
            Err(e) => {
                warn!(
                    target: "mac_stats::compaction",
                    "after_compaction hook spawn failed: {}",
                    e
                );
            }
        }
    });
}
