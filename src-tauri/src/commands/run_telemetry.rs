//! Structured per-turn telemetry for continuous improvement (`~/.mac-stats/runs.jsonl`).
//!
//! Cheap (no extra LLM). Digested by `scripts/digest_agent_runs.py`.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use serde::Serialize;
use tracing::{debug, warn};

/// One completed agent-router turn.
#[derive(Debug, Clone, Serialize)]
pub struct TurnRunRecord {
    pub ts: String,
    pub request_id: String,
    pub channel_id: Option<u64>,
    pub entry: String,
    pub lane: String,
    pub question_preview: String,
    pub wall_ms: u64,
    pub tools: Vec<String>,
    pub tool_steps: u32,
    pub pre_routed: bool,
    pub verify_passed: Option<bool>,
    pub reply_chars: usize,
    pub skipped_criteria_llm: bool,
    pub skipped_topic_llm: bool,
    pub skipped_verify_llm: bool,
    pub skipped_plan_llm: bool,
    pub ok: bool,
    pub error: Option<String>,
}

impl TurnRunRecord {
    pub fn question_preview_from(question: &str) -> String {
        let plain = crate::commands::untrusted_content::humanize_for_discord_status(question);
        let one_line: String = plain
            .chars()
            .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
            .take(160)
            .collect();
        one_line.trim().to_string()
    }
}

/// Path: `$HOME/.mac-stats/runs.jsonl`
pub fn runs_jsonl_path() -> PathBuf {
    std::env::var_os("HOME")
        .map(|h| PathBuf::from(h).join(".mac-stats").join("runs.jsonl"))
        .unwrap_or_else(|| std::env::temp_dir().join("mac-stats-runs.jsonl"))
}

/// Append one JSON line. Best-effort; never panics the agent loop.
pub fn record_turn(rec: &TurnRunRecord) {
    let path = runs_jsonl_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let line = match serde_json::to_string(rec) {
        Ok(s) => s,
        Err(e) => {
            warn!(target: "mac_stats::telemetry", "runs.jsonl serialize failed: {}", e);
            return;
        }
    };
    match OpenOptions::new().create(true).append(true).open(&path) {
        Ok(mut f) => {
            if let Err(e) = writeln!(f, "{}", line) {
                warn!(target: "mac_stats::telemetry", "runs.jsonl write failed: {}", e);
            } else {
                let _ = f.flush();
                // Best-effort durability (Hermes-style crash resilience for append-only telemetry).
                let _ = f.sync_data();
                debug!(
                    target: "mac_stats::telemetry",
                    request_id = %rec.request_id,
                    lane = %rec.lane,
                    wall_ms = rec.wall_ms,
                    "turn recorded"
                );
            }
        }
        Err(e) => {
            warn!(target: "mac_stats::telemetry", "runs.jsonl open failed: {}: {}", path.display(), e);
        }
    }
}

/// Keep at most `runs_prune_max_lines` lines in `runs.jsonl` (newest kept). `0` disables.
pub fn prune_runs_jsonl_if_needed() -> u64 {
    prune_runs_jsonl_at(&runs_jsonl_path(), crate::config::Config::runs_prune_max_lines())
}

/// Testable core: trim `path` to the last `max` non-empty lines.
pub(crate) fn prune_runs_jsonl_at(path: &std::path::Path, max: usize) -> u64 {
    if max == 0 {
        return 0;
    }
    let Ok(text) = std::fs::read_to_string(path) else {
        return 0;
    };
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.len() <= max {
        return 0;
    }
    let keep = &lines[lines.len() - max..];
    let body = keep.join("\n") + "\n";
    match crate::config::write_text_atomic(path, &body) {
        Ok(()) => {
            let removed = (lines.len() - max) as u64;
            tracing::info!(
                target: "mac_stats::telemetry",
                "runs.jsonl prune: removed {} old line(s); kept {}",
                removed,
                max
            );
            removed
        }
        Err(e) => {
            warn!(target: "mac_stats::telemetry", "runs.jsonl prune failed: {}", e);
            0
        }
    }
}

/// Wall-clock helper for a turn.
pub struct TurnClock {
    start: Instant,
}

impl TurnClock {
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn wall_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_strips_newlines() {
        let p = TurnRunRecord::question_preview_from("hello\nworld");
        assert!(!p.contains('\n'));
        assert!(p.contains("hello"));
    }

    #[test]
    fn prune_runs_jsonl_keeps_tail() {
        let dir = std::env::temp_dir().join(format!("mac-stats-runs-prune-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("runs.jsonl");
        let mut body = String::new();
        for i in 0..5 {
            body.push_str(&format!("{{\"i\":{}}}\n", i));
        }
        std::fs::write(&path, &body).unwrap();
        assert_eq!(prune_runs_jsonl_at(&path, 2), 3);
        let kept = std::fs::read_to_string(&path).unwrap();
        assert_eq!(kept.lines().filter(|l| !l.trim().is_empty()).count(), 2);
        assert!(kept.contains("\"i\":3"));
        assert!(kept.contains("\"i\":4"));
        assert!(!kept.contains("\"i\":0"));
        assert_eq!(prune_runs_jsonl_at(&path, 2), 0);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
