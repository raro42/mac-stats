//! Records successful scheduler-initiated Discord deliveries so the CPU (menu bar) chat can inject
//! a short system block — mirrors “main session awareness” after isolated cron delivery in OpenClaw.

use crate::config::Config;
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

const MAX_STORED_ENTRIES: usize = 24;
const MAX_SUMMARY_CHARS: usize = 480;
/// How many recent lines to inject into the in-app Ollama system context.
const CHAT_CONTEXT_MAX_ITEMS: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryAwarenessEntry {
    pub context_key: String,
    /// RFC3339 UTC with millis.
    pub utc: String,
    pub schedule_id: Option<String>,
    pub channel_id: String,
    pub summary: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct DeliveryAwarenessFile {
    entries: Vec<DeliveryAwarenessEntry>,
}

fn awareness_path() -> std::path::PathBuf {
    Config::schedules_file_path()
        .parent()
        .map(|p| p.join("scheduler_delivery_awareness.json"))
        .unwrap_or_else(|| std::env::temp_dir().join("mac-stats-scheduler_delivery_awareness.json"))
}

/// Stable per-run id: schedule label + monotonic wall time in nanoseconds (scheduler runs one task at a time).
pub fn new_context_key_for_schedule(schedule_id_label: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("sched:{}:{}", schedule_id_label, nanos)
}

fn truncate_summary(s: &str) -> String {
    let t = s.trim();
    if t.chars().count() <= MAX_SUMMARY_CHARS {
        return t.to_string();
    }
    let mut out = String::new();
    for ch in t.chars().take(MAX_SUMMARY_CHARS.saturating_sub(1)) {
        out.push(ch);
    }
    out.push('…');
    out
}

/// Append one entry after Discord accepted the message. Skips if `context_key` already exists (idempotent).
pub fn record_if_new(
    context_key: &str,
    schedule_id: Option<&str>,
    channel_id: u64,
    delivered_body: &str,
) {
    if let Err(e) = Config::ensure_schedules_directory() {
        warn!(
            "Scheduler delivery awareness: could not ensure data dir: {}",
            e
        );
        return;
    }
    let path = awareness_path();
    let mut file: DeliveryAwarenessFile = if path.exists() {
        match std::fs::read_to_string(&path) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
            Err(e) => {
                warn!(
                    "Scheduler delivery awareness: read {:?} failed: {}",
                    path, e
                );
                DeliveryAwarenessFile::default()
            }
        }
    } else {
        DeliveryAwarenessFile::default()
    };

    if file
        .entries
        .iter()
        .any(|e| e.context_key.as_str() == context_key)
    {
        debug!(
            "Scheduler delivery awareness: skip duplicate context_key={}",
            context_key
        );
        return;
    }

    let utc = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    file.entries.push(DeliveryAwarenessEntry {
        context_key: context_key.to_string(),
        utc,
        schedule_id: schedule_id.map(|s| s.to_string()),
        channel_id: channel_id.to_string(),
        summary: truncate_summary(delivered_body),
    });

    while file.entries.len() > MAX_STORED_ENTRIES {
        file.entries.remove(0);
    }

    match serde_json::to_string_pretty(&file) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&path, json) {
                warn!(
                    "Scheduler delivery awareness: write {:?} failed: {}",
                    path, e
                );
            } else {
                info!(
                    "Scheduler delivery awareness: recorded schedule={:?} channel={} key_len={}",
                    schedule_id,
                    channel_id,
                    context_key.len()
                );
            }
        }
        Err(e) => warn!("Scheduler delivery awareness: serialize failed: {}", e),
    }
}

fn load_file() -> DeliveryAwarenessFile {
    let path = awareness_path();
    if !path.exists() {
        return DeliveryAwarenessFile::default();
    }
    match std::fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => DeliveryAwarenessFile::default(),
    }
}

/// Newest-first rows for Settings / debug.
pub fn list_entries_newest_first() -> Vec<DeliveryAwarenessEntry> {
    let mut v = load_file().entries;
    v.reverse();
    v
}

/// Short block appended to the CPU window system prompt so the model knows what was already posted to Discord.
pub fn format_for_chat_context() -> String {
    let file = load_file();
    if file.entries.is_empty() {
        return String::new();
    }
    let n = file.entries.len();
    let start = n.saturating_sub(CHAT_CONTEXT_MAX_ITEMS);
    let mut lines: Vec<String> = Vec::new();
    lines.push(
        "[Scheduler → Discord: recent successful deliveries — use for continuity; do not re-send unless the user asks.]".to_string(),
    );
    for e in file.entries[start..].iter() {
        let sid = e
            .schedule_id
            .as_deref()
            .filter(|s| !s.is_empty())
            .map(|s| format!("schedule `{}` · ", s))
            .unwrap_or_default();
        lines.push(format!(
            "- {} · {}channel {} · {}",
            e.utc, sid, e.channel_id, e.summary
        ));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::path::Path;
    use std::sync::Mutex;

    fn awareness_json_path() -> std::path::PathBuf {
        Config::schedules_file_path()
            .parent()
            .expect("schedules path has parent")
            .join("scheduler_delivery_awareness.json")
    }

    static HOME_DELIVERY_AWARENESS_TEST_LOCK: Mutex<()> = Mutex::new(());

    struct HomeOverride {
        previous: Option<String>,
    }

    impl HomeOverride {
        fn set(home: &Path) -> Self {
            let previous = std::env::var("HOME").ok();
            std::env::set_var("HOME", home.as_os_str());
            Self { previous }
        }
    }

    impl Drop for HomeOverride {
        fn drop(&mut self) {
            match &self.previous {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
        }
    }

    #[test]
    fn new_context_key_has_stable_prefix() {
        let k = new_context_key_for_schedule("morning-job");
        assert!(k.starts_with("sched:morning-job:"));
        assert!(k.len() > "sched:morning-job:".len());
    }

    #[test]
    fn record_if_new_skips_duplicate_context_key() {
        let _guard = HOME_DELIVERY_AWARENESS_TEST_LOCK
            .lock()
            .expect("home test lock");
        let base = std::env::temp_dir().join(format!(
            "mac-stats-delivery-awareness-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join(".mac-stats")).unwrap();
        let _home = HomeOverride::set(&base);

        let path = awareness_json_path();

        record_if_new("dup-key-test", Some("sched-a"), 999001, "first body");
        record_if_new("dup-key-test", Some("sched-a"), 999001, "second body ignored");

        let raw = std::fs::read_to_string(&path).unwrap();
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let arr = v["entries"].as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["summary"], "first body");

        let ctx = format_for_chat_context();
        assert!(ctx.contains("[Scheduler → Discord:"));
        assert!(ctx.contains("first body"));
        assert!(!ctx.contains("second body ignored"));
    }

    #[test]
    fn list_entries_newest_first_order() {
        let _guard = HOME_DELIVERY_AWARENESS_TEST_LOCK
            .lock()
            .expect("home test lock");
        let base = std::env::temp_dir().join(format!(
            "mac-stats-delivery-awareness-order-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join(".mac-stats")).unwrap();
        let _home = HomeOverride::set(&base);

        record_if_new("order-key-a", None, 1, "older");
        std::thread::sleep(std::time::Duration::from_millis(5));
        record_if_new("order-key-b", None, 1, "newer");

        let rows = list_entries_newest_first();
        assert_eq!(rows.len(), 2);
        assert!(rows[0].summary.contains("newer"), "{rows:?}");
        assert!(rows[1].summary.contains("older"), "{rows:?}");
    }
}
