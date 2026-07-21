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
}
