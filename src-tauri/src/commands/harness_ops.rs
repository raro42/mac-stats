//! Dashboard "Agent Ops" APIs — agents overview helpers, sessions, memory, runs.
//! OpenClaw-shaped ops surface over ~/.mac-stats data (Hermes mental model).

use crate::config::Config;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Crash-safe text write (Hermes-style temp + fsync + rename).
pub(crate) fn write_text_atomic(path: &Path, text: &str) -> Result<(), String> {
    crate::config::write_text_atomic(path, text)
}

#[derive(Debug, Clone, Serialize)]
pub struct LiveSessionSummary {
    pub source: String,
    pub session_id: u64,
    pub message_count: usize,
    pub last_activity: String,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionMessageRow {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionFileSummary {
    pub name: String,
    pub path: String,
    pub source_hint: String,
    pub slug: String,
    pub modified_ms: u64,
    pub size_bytes: u64,
    /// Last user message preview (Agent Ops resume UX).
    pub preview: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryFileSummary {
    pub name: String,
    pub path: String,
    pub kind: String,
    pub size_bytes: u64,
    pub line_count: usize,
    pub modified_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunTurnSummary {
    pub ts: String,
    pub lane: String,
    pub wall_ms: u64,
    pub tools: Vec<String>,
    pub question_preview: String,
    pub ok: bool,
    pub request_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunInsightCandidate {
    pub kind: String,
    pub reason: String,
    pub wall_ms: u64,
    pub lane: String,
    pub question_preview: String,
    pub request_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunsInsights {
    pub turns: usize,
    pub ok_count: usize,
    pub fail_count: usize,
    pub p50_ms: u64,
    pub mean_ms: u64,
    pub max_ms: u64,
    /// Turns included in p50/mean/max/slowest after shipped-instant noise filters.
    pub latency_sample: usize,
    pub by_lane: Vec<(String, usize)>,
    pub by_tool: Vec<(String, usize)>,
    pub candidates: Vec<RunInsightCandidate>,
    pub slowest: Vec<RunTurnSummary>,
    pub recent: Vec<RunTurnSummary>,
    /// Discord gateway reconnect line (process lifetime).
    pub discord_gateway: String,
    /// From `~/.mac-stats/improvements/latest.json` (digester).
    pub digest_open_count: usize,
    pub digest_stale_count: usize,
    pub digest_generated_at: String,
    pub digest_open_hints: Vec<String>,
    /// Digester provenance: `python`, `rust-native`, or empty if missing.
    pub digest_source: String,
    /// Seconds since this mac-stats process started (Agent Ops Version card).
    pub process_uptime_secs: u64,
    /// When set, stats cover only the last N days of runs.jsonl (Hermes `/insights [days]`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct DigestSummary {
    pub open_count: usize,
    pub stale_count: usize,
    pub turns: usize,
    pub generated_at: String,
    pub open_hints: Vec<String>,
    pub stale_hints: Vec<String>,
    pub path: String,
    pub source: String,
}

fn file_mtime_ms(meta: &fs::Metadata) -> u64 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn parse_session_filename(name: &str) -> (String, String) {
    // session-memory-<sourceOrId>-<ts>-<slug>.md  or  session-memory-<id>-...
    let stem = name.trim_end_matches(".md");
    let rest = stem
        .strip_prefix("session-memory-")
        .unwrap_or(stem);
    let parts: Vec<&str> = rest.split('-').collect();
    if parts.len() >= 3 {
        let source = parts[0].to_string();
        let slug = parts[2..].join("-");
        (source, slug)
    } else {
        ("unknown".into(), rest.to_string())
    }
}

/// In-memory Discord/UI sessions currently held by the process.
#[tauri::command]
pub fn list_live_sessions() -> Vec<LiveSessionSummary> {
    let mut rows: Vec<_> = crate::session_memory::list_sessions()
        .into_iter()
        .map(|e| LiveSessionSummary {
            source: e.source,
            session_id: e.session_id,
            message_count: e.message_count,
            last_activity: e.last_activity.to_rfc3339(),
            preview: e.preview,
        })
        .collect();
    rows.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));
    rows
}

/// Messages for a live in-memory session (Agent Ops resume / preview).
#[tauri::command]
pub fn read_live_session_messages(source: String, session_id: u64) -> Vec<SessionMessageRow> {
    crate::session_memory::get_messages(source.trim(), session_id)
        .into_iter()
        .map(|(role, content)| SessionMessageRow { role, content })
        .collect()
}

/// Parsed user/assistant turns from a session markdown file under ~/.mac-stats/session/.
#[tauri::command]
pub fn read_session_file_messages(path: String) -> Result<Vec<SessionMessageRow>, String> {
    let text = read_session_file(path)?;
    Ok(crate::session_memory::parse_session_markdown(&text)
        .into_iter()
        .map(|(role, content)| SessionMessageRow { role, content })
        .collect())
}

/// Recent persisted session markdown under ~/.mac-stats/session/.
#[tauri::command]
pub fn list_session_files(limit: Option<u32>) -> Result<Vec<SessionFileSummary>, String> {
    let dir = Config::session_dir();
    if !dir.is_dir() {
        return Ok(vec![]);
    }
    let lim = limit.unwrap_or(40).clamp(1, 200) as usize;
    let mut rows = Vec::new();
    for ent in fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let ent = ent.map_err(|e| e.to_string())?;
        let path = ent.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        if !name.starts_with("session-memory-") {
            continue;
        }
        let meta = ent.metadata().map_err(|e| e.to_string())?;
        let (source_hint, slug) = parse_session_filename(&name);
        let size_bytes = meta.len();
        let preview =
            crate::session_memory::last_user_preview_from_session_path(&path, size_bytes);
        rows.push(SessionFileSummary {
            name,
            path: path.display().to_string(),
            source_hint,
            slug,
            modified_ms: file_mtime_ms(&meta),
            size_bytes,
            preview,
        });
    }
    rows.sort_by(|a, b| b.modified_ms.cmp(&a.modified_ms));
    rows.truncate(lim);
    Ok(rows)
}

/// Read a session markdown file. Path must be under ~/.mac-stats/session/.
#[tauri::command]
pub fn read_session_file(path: String) -> Result<String, String> {
    let p = sanitize_under_dir(&path, &Config::session_dir())?;
    fs::read_to_string(&p).map_err(|e| e.to_string())
}

/// Global + Discord channel memory files.
#[tauri::command]
pub fn list_memory_files() -> Result<Vec<MemoryFileSummary>, String> {
    let dir = Config::agents_dir();
    if !dir.is_dir() {
        return Ok(vec![]);
    }
    let mut rows = Vec::new();
    for ent in fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let ent = ent.map_err(|e| e.to_string())?;
        let path = ent.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let kind = if name == "memory.md" {
            "global"
        } else if name == "soul.md" {
            "soul"
        } else if name.starts_with("memory-discord-") {
            "discord"
        } else if name == "memory-main.md" {
            "main"
        } else {
            continue;
        };
        let meta = ent.metadata().map_err(|e| e.to_string())?;
        let content = fs::read_to_string(&path).unwrap_or_default();
        rows.push(MemoryFileSummary {
            name,
            path: path.display().to_string(),
            kind: kind.into(),
            size_bytes: meta.len(),
            line_count: content.lines().count(),
            modified_ms: file_mtime_ms(&meta),
        });
    }
    rows.sort_by(|a, b| a.kind.cmp(&b.kind).then(a.name.cmp(&b.name)));
    Ok(rows)
}

#[tauri::command]
pub fn read_memory_file(path: String) -> Result<String, String> {
    let p = sanitize_under_dir(&path, &Config::agents_dir())?;
    let name = p
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    if !(name == "memory.md"
        || name == "soul.md"
        || name == "memory-main.md"
        || name.starts_with("memory-discord-"))
    {
        return Err("Not a memory/soul file".into());
    }
    fs::read_to_string(&p).map_err(|e| e.to_string())
}

/// Tail + Hermes-lite insights over ~/.mac-stats/runs.jsonl.
#[tauri::command]
pub fn get_runs_insights(limit: Option<u32>) -> Result<RunsInsights, String> {
    Ok(compute_runs_insights(limit.unwrap_or(50)))
}

/// Shared analytics used by Agent Ops UI and Discord `/insights`.
pub fn compute_runs_insights(limit: u32) -> RunsInsights {
    compute_runs_insights_for(limit, None)
}

/// Agent Ops Runs Slow filter parity (wall time ≥ this ms).
pub const OPS_RUNS_SLOW_MS: u64 = 2000;

/// Parse optional day window from `/insights 7`, `/failed 7`, `/slow 7`, `/instant 7`, etc.
pub fn parse_insights_days(content: &str) -> Option<u32> {
    let n = content
        .trim()
        .trim_start_matches('@')
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    let n = n
        .trim_start_matches("werner")
        .trim_start_matches(',')
        .trim()
        .trim_start_matches("please")
        .trim();
    let rest = if let Some(r) = n.strip_prefix("/insights") {
        r.trim()
    } else if let Some(r) = n.strip_prefix("insights") {
        r.trim()
    } else if let Some(r) = n.strip_prefix("/failed") {
        r.trim()
    } else if let Some(r) = n.strip_prefix("/slow") {
        r.trim()
    } else if let Some(r) = n.strip_prefix("/instant") {
        r.trim()
    } else if let Some(r) = n.strip_prefix("/direct") {
        r.trim()
    } else if let Some(r) = n.strip_prefix("/lite") {
        r.trim()
    } else if let Some(r) = n.strip_prefix("failed runs") {
        r.trim()
    } else if let Some(r) = n.strip_prefix("slow runs") {
        r.trim()
    } else if let Some(r) = n.strip_prefix("instant runs") {
        r.trim()
    } else if let Some(r) = n.strip_prefix("direct runs") {
        r.trim()
    } else if let Some(r) = n.strip_prefix("lite runs") {
        r.trim()
    } else {
        return None;
    };
    if rest.is_empty() {
        return None;
    }
    let parts: Vec<&str> = rest.split_whitespace().collect();
    let mut i = 0;
    while i < parts.len() {
        if parts[i] == "--days" && i + 1 < parts.len() {
            if let Ok(d) = parts[i + 1].parse::<u32>() {
                return Some(d.clamp(1, 90));
            }
            return None;
        }
        if let Ok(d) = parts[i].parse::<u32>() {
            return Some(d.clamp(1, 90));
        }
        i += 1;
    }
    None
}

/// `days`: when set, only include turns with `ts` within the last N days (1–90).
pub fn compute_runs_insights_for(limit: u32, days: Option<u32>) -> RunsInsights {
    let path = crate::commands::run_telemetry::runs_jsonl_path();
    let lim = limit.clamp(1, 200) as usize;
    let gateway = crate::discord::format_discord_gateway_insights_line();
    let digest = load_digest_summary();
    let window_days = days.map(|d| d.clamp(1, 90));
    let since = window_days.map(|d| chrono::Utc::now() - chrono::Duration::days(d as i64));
    let empty = RunsInsights {
        turns: 0,
        ok_count: 0,
        fail_count: 0,
        p50_ms: 0,
        mean_ms: 0,
        max_ms: 0,
        latency_sample: 0,
        by_lane: vec![],
        by_tool: vec![],
        candidates: vec![],
        slowest: vec![],
        recent: vec![],
        discord_gateway: gateway.clone(),
        digest_open_count: digest.open_count,
        digest_stale_count: digest.stale_count,
        digest_generated_at: digest.generated_at.clone(),
        digest_open_hints: digest.open_hints.clone(),
        digest_source: digest.source.clone(),
        process_uptime_secs: crate::state::process_uptime_secs(),
        window_days,
    };
    if !path.is_file() {
        return empty;
    }
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return empty,
    };
    let mut recent = Vec::new();
    let mut walls: Vec<u64> = Vec::new();
    let mut latency_walls: Vec<u64> = Vec::new();
    let mut slowest_pool: Vec<RunTurnSummary> = Vec::new();
    let mut ok_count = 0usize;
    let mut fail_count = 0usize;
    let mut lane_counts: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    let mut tool_counts: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    let mut candidates: Vec<RunInsightCandidate> = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let v: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let Some(since) = since {
            let ts = v
                .get("ts")
                .and_then(|x| x.as_str())
                .and_then(parse_run_ts);
            match ts {
                Some(t) if t >= since => {}
                Some(_) => continue,
                None => continue,
            }
        }
        let wall = v.get("wall_ms").and_then(|x| x.as_u64()).unwrap_or(0);
        walls.push(wall);
        let ok = v.get("ok").and_then(|x| x.as_bool()).unwrap_or(true);
        if ok {
            ok_count += 1;
        } else {
            fail_count += 1;
        }
        let lane = v
            .get("lane")
            .and_then(|x| x.as_str())
            .unwrap_or("?")
            .to_string();
        *lane_counts.entry(lane.clone()).or_default() += 1;
        let tools = v
            .get("tools")
            .and_then(|t| t.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for t in &tools {
            *tool_counts.entry(t.clone()).or_default() += 1;
        }
        let question_preview = v
            .get("question_preview")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let request_id = v
            .get("request_id")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        if let Some(c) = classify_candidate(&lane, wall, &tools, &question_preview, &request_id) {
            candidates.push(c);
        }
        let summary = RunTurnSummary {
            ts: v
                .get("ts")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            lane: lane.clone(),
            wall_ms: wall,
            tools: tools.clone(),
            question_preview: question_preview.clone(),
            ok,
            request_id,
        };
        // Digester Slowest parity: exclude shipped instant noise from latency + Slowest.
        if !is_insights_slowest_noise(&lane, wall, &tools, &question_preview) {
            latency_walls.push(wall);
            slowest_pool.push(summary.clone());
        }
        recent.push(summary);
    }

    let turns = walls.len();
    let latency_sample = latency_walls.len();
    let max_ms = latency_walls.iter().copied().max().unwrap_or(0);
    let mean_ms = if latency_sample == 0 {
        0
    } else {
        latency_walls.iter().sum::<u64>() / latency_sample as u64
    };
    let mut sorted = latency_walls;
    sorted.sort_unstable();
    let p50_ms = if sorted.is_empty() {
        0
    } else {
        sorted[sorted.len() / 2]
    };

    let mut slowest = slowest_pool;
    slowest.sort_by(|a, b| b.wall_ms.cmp(&a.wall_ms));
    slowest.truncate(5);

    candidates.sort_by(|a, b| b.wall_ms.cmp(&a.wall_ms));
    candidates.truncate(8);

    let mut by_tool: Vec<_> = tool_counts.into_iter().collect();
    by_tool.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    by_tool.truncate(12);

    if recent.len() > lim {
        recent = recent.split_off(recent.len() - lim);
    }
    recent.reverse();
    let by_lane: Vec<_> = lane_counts.into_iter().collect();
    RunsInsights {
        turns,
        ok_count,
        fail_count,
        p50_ms,
        mean_ms,
        max_ms,
        latency_sample,
        by_lane,
        by_tool,
        candidates,
        slowest,
        recent,
        discord_gateway: gateway,
        digest_open_count: digest.open_count,
        digest_stale_count: digest.stale_count,
        digest_generated_at: digest.generated_at,
        digest_open_hints: digest.open_hints,
        digest_source: digest.source,
        process_uptime_secs: crate::state::process_uptime_secs(),
        window_days,
    }
}

fn digest_json_path() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
            .join(".mac-stats")
            .join("improvements")
            .join("latest.json")
    } else {
        std::env::temp_dir()
            .join("mac-stats-improvements")
            .join("latest.json")
    }
}

/// Load digester summary written by `scripts/digest_agent_runs.py`.
pub fn load_digest_summary() -> DigestSummary {
    let path = digest_json_path();
    let mut summary = DigestSummary {
        path: path.display().to_string(),
        ..Default::default()
    };
    let Ok(text) = fs::read_to_string(&path) else {
        return summary;
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
        return summary;
    };
    summary.generated_at = v
        .get("generated_at")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    summary.turns = v.get("turns").and_then(|x| x.as_u64()).unwrap_or(0) as usize;
    summary.open_count = v.get("open_count").and_then(|x| x.as_u64()).unwrap_or(0) as usize;
    summary.stale_count = v.get("stale_count").and_then(|x| x.as_u64()).unwrap_or(0) as usize;
    summary.source = v
        .get("source")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    summary.open_hints = v
        .get("open")
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    item.get("hint")
                        .and_then(|h| h.as_str())
                        .map(|s| s.to_string())
                })
                .take(5)
                .collect()
        })
        .unwrap_or_default();
    summary.stale_hints = v
        .get("stale")
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    item.get("hint")
                        .and_then(|h| h.as_str())
                        .map(|s| s.to_string())
                })
                .take(5)
                .collect()
        })
        .unwrap_or_default();
    summary
}

#[tauri::command]
pub fn get_digest_summary() -> DigestSummary {
    load_digest_summary()
}

/// Candidate digester script locations (dev tree + optional override).
fn digest_script_candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(p) = std::env::var("MAC_STATS_DIGEST_SCRIPT") {
        let t = p.trim();
        if !t.is_empty() {
            out.push(PathBuf::from(t));
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let home = PathBuf::from(home);
        out.push(home.join("projects/mac-stats/scripts/digest_agent_runs.py"));
        out.push(home.join("src/mac-stats/scripts/digest_agent_runs.py"));
    }
    // Relative to cwd when running from repo
    out.push(PathBuf::from("scripts/digest_agent_runs.py"));
    out.push(PathBuf::from("../scripts/digest_agent_runs.py"));
    out
}

/// Refresh `~/.mac-stats/improvements/latest.{md,json}` via Python digester when available,
/// otherwise a Rust-native fallback that writes `latest.json` (Agent Ops still works offline).
#[tauri::command]
pub fn refresh_agent_digest() -> String {
    let out_dir = digest_json_path()
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let _ = fs::create_dir_all(&out_dir);

    if let Some(script) = digest_script_candidates().into_iter().find(|p| p.is_file()) {
        match std::process::Command::new("python3")
            .arg(&script)
            .arg("--days")
            .arg("7")
            .arg("--out")
            .arg(out_dir.join("latest.md"))
            .output()
        {
            Ok(o) if o.status.success() => {
                let summary = load_digest_summary();
                return format!(
                    "Digest refreshed (python {}): {} open · {} stale · {} turns",
                    script.display(),
                    summary.open_count,
                    summary.stale_count,
                    summary.turns
                );
            }
            Ok(o) => {
                let err = String::from_utf8_lossy(&o.stderr);
                tracing::warn!(
                    target: "mac_stats::digest",
                    "python digester failed (exit {:?}): {} — using Rust fallback",
                    o.status.code(),
                    err.chars().take(160).collect::<String>()
                );
            }
            Err(e) => {
                tracing::warn!(
                    target: "mac_stats::digest",
                    "python digester spawn failed: {} — using Rust fallback",
                    e
                );
            }
        }
    }

    match write_digest_native(7) {
        Ok(summary) => format!(
            "Digest refreshed (rust-native): {} open · {} stale · {} turns",
            summary.open_count, summary.stale_count, summary.turns
        ),
        Err(e) => format!("Digest refresh failed (rust-native): {}", e),
    }
}

fn parse_run_ts(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    let s = if s.ends_with('Z') {
        format!("{}+00:00", &s[..s.len().saturating_sub(1)])
    } else {
        s.to_string()
    };
    chrono::DateTime::parse_from_rfc3339(&s)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .or_else(|| {
            chrono::DateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f%z")
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
        })
}

fn shipped_cutoffs() -> (
    chrono::DateTime<chrono::Utc>,
    chrono::DateTime<chrono::Utc>,
    chrono::DateTime<chrono::Utc>,
    chrono::DateTime<chrono::Utc>,
    chrono::DateTime<chrono::Utc>,
) {
    use chrono::{TimeZone, Utc};
    (
        Utc.with_ymd_and_hms(2026, 7, 20, 15, 45, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 7, 20, 14, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 7, 20, 21, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 7, 20, 14, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 7, 21, 4, 30, 0).unwrap(),
    )
}

fn is_stale_shipped_candidate(
    hint: &str,
    q: &str,
    ts: Option<chrono::DateTime<chrono::Utc>>,
) -> bool {
    let Some(ts) = ts else {
        return false;
    };
    let (ver, time, weather, greet, wakeup) = shipped_cutoffs();
    let hl = hint.to_lowercase();
    let ql = q.to_lowercase();
    if hl.contains("instant version") && ts < ver {
        return true;
    }
    if hl.contains("instant time") && ts < time {
        return true;
    }
    if hl.contains("greeting") && ts < greet {
        return true;
    }
    if ts < weather && (ql.contains("wether") || ql.contains("weather")) {
        if hl.contains("open-meteo")
            || hl.contains("weather via search")
            || hl.contains("brave")
            || hl.contains("zero-tool")
            || hl.contains("instant")
        {
            return true;
        }
    }
    if ts < wakeup
        && (ql.contains("wake-up") || ql.contains("wakeup") || ql.contains("wake up"))
        && (hl.contains("zero-tool") || hl.contains("instant") || hl.contains("wake"))
    {
        return true;
    }
    false
}

fn hint_for_run(rec: &serde_json::Value) -> Option<String> {
    let q = rec
        .get("question_preview")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_lowercase();
    let wall = rec.get("wall_ms").and_then(|x| x.as_u64()).unwrap_or(0);
    let lane = rec.get("lane").and_then(|x| x.as_str()).unwrap_or("?");
    let tools = rec
        .get("tools")
        .and_then(|t| t.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let tool_steps = rec.get("tool_steps").and_then(|x| x.as_u64()).unwrap_or(0);
    if wall >= 5_000 && matches!(lane, "lite" | "direct" | "full") && tools == 0 && tool_steps == 0
    {
        if q.contains("version") {
            return Some("Promote to INSTANT version lane".into());
        }
        if q.contains("wake-up") || q.contains("wakeup") || q.contains("wake up") {
            return Some("Promote to INSTANT wake-up / morning greeting lane".into());
        }
        if q.contains("time") || q.contains("uhr") || q.contains("hora") || q.contains("date") {
            return Some("Promote to INSTANT time/date lane".into());
        }
        if matches!(
            q.trim(),
            "ping" | "hi" | "hello" | "hey" | "thanks" | "thank you"
        ) {
            return Some("Promote to INSTANT greeting/thanks lane".into());
        }
        if wall >= 15_000 {
            return Some(
                "Zero-tool slow turn — consider instant/pre-route or smaller model".into(),
            );
        }
    }
    if wall >= 15_000
        && lane == "direct"
        && tools > 0
        && (q.contains("weather") || q.contains("wether"))
    {
        let tool_names = rec
            .get("tools")
            .and_then(|t| t.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str())
                    .collect::<Vec<_>>()
                    .join(",")
                    .to_uppercase()
            })
            .unwrap_or_default();
        if tool_names.contains("BRAVE") || tool_names.contains("PERPLEXITY") {
            return Some(
                "Weather via search — prefer Open-Meteo INSTANT when place is clear".into(),
            );
        }
    }
    None
}

/// Write `latest.json` (+ short `latest.md`) without Python.
fn write_digest_native(days: i64) -> Result<DigestSummary, String> {
    let path = crate::commands::run_telemetry::runs_jsonl_path();
    let since = chrono::Utc::now() - chrono::Duration::days(days);
    let mut latency_walls: Vec<u64> = Vec::new();
    let mut by_lane: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    let mut open: Vec<serde_json::Value> = Vec::new();
    let mut stale: Vec<serde_json::Value> = Vec::new();
    let mut turns = 0usize;

    if path.is_file() {
        let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let Ok(rec) = serde_json::from_str::<serde_json::Value>(line) else {
                continue;
            };
            let ts = rec
                .get("ts")
                .and_then(|x| x.as_str())
                .and_then(parse_run_ts);
            if let Some(t) = ts {
                if t < since {
                    continue;
                }
            } else {
                continue;
            }
            turns += 1;
            let wall = rec.get("wall_ms").and_then(|x| x.as_u64()).unwrap_or(0);
            let lane = rec
                .get("lane")
                .and_then(|x| x.as_str())
                .unwrap_or("?")
                .to_string();
            *by_lane.entry(lane.clone()).or_default() += 1;
            let tools = rec
                .get("tools")
                .and_then(|t| t.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let preview = rec
                .get("question_preview")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            if !is_insights_slowest_noise(&lane, wall, &tools, &preview) {
                latency_walls.push(wall);
            }
            let rid = rec
                .get("request_id")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            if let Some(hint) = hint_for_run(&rec) {
                let item = serde_json::json!({
                    "wall_ms": wall,
                    "hint": hint,
                    "question_preview": preview.chars().take(120).collect::<String>(),
                    "request_id": rid,
                    "ts": ts.map(|t| t.to_rfc3339()),
                });
                if is_stale_shipped_candidate(
                    item.get("hint").and_then(|h| h.as_str()).unwrap_or(""),
                    &preview,
                    ts,
                ) {
                    stale.push(item);
                } else {
                    open.push(item);
                }
            }
        }
    }

    latency_walls.sort_unstable();
    let p50 = if latency_walls.is_empty() {
        0
    } else {
        latency_walls[latency_walls.len() / 2]
    };
    let max_ms = latency_walls.iter().copied().max().unwrap_or(0);
    let generated = chrono::Utc::now().to_rfc3339();
    let payload = serde_json::json!({
        "generated_at": generated,
        "days": days,
        "turns": turns,
        "latency_sample": latency_walls.len(),
        "open_count": open.len(),
        "stale_count": stale.len(),
        "p50_ms": p50,
        "max_ms": max_ms,
        "by_lane": by_lane,
        "open": open,
        "stale": stale,
        "markdown_path": digest_json_path().with_extension("md").display().to_string(),
        "source": "rust-native",
    });

    let json_path = digest_json_path();
    if let Some(parent) = json_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    write_text_atomic(
        &json_path,
        &(serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())? + "\n"),
    )?;

    let md_path = json_path.with_extension("md");
    let md = format!(
        "# Agent run digest ({days}d)\n\nGenerated: {generated} (rust-native)\nTurns: **{turns}**\n\n## Improvement candidates\n{}\n\n## Stale / already shipped\n{}\n",
        if open.is_empty() {
            "_None this window (open)._".to_string()
        } else {
            open.iter()
                .take(10)
                .filter_map(|i| {
                    Some(format!(
                        "- **{}** — {} ms",
                        i.get("hint")?.as_str()?,
                        i.get("wall_ms")?.as_u64()?
                    ))
                })
                .collect::<Vec<_>>()
                .join("\n")
        },
        if stale.is_empty() {
            "_None._".to_string()
        } else {
            format!("_{} stale candidate(s) ignored._", stale.len())
        }
    );
    let _ = write_text_atomic(&md_path, &md);

    Ok(load_digest_summary())
}

/// True for `/digest` / `run digest` operator asks.
pub fn looks_like_digest_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    // Long “digest this report…” stays with the agent.
    if n.chars().count() > 48 || n.contains(" this ") || n.contains("research") {
        return false;
    }
    matches!(
        n.as_str(),
        "digest"
            | "/digest"
            | "run digest"
            | "refresh digest"
            | "agent digest"
            | "run digester"
            | "refresh digester"
            |         "update digest"
            | "rerun digest"
            | "digest open"
            | "open digest"
            | "open candidates"
            | "digest candidates"
            | "open digest hints"
            | "any open candidates"
            | "show open candidates"
    )
}

/// Normalize operator command text (strip @mention / Werner / please / show me).
fn normalize_operator_command(content: &str) -> String {
    let n = content
        .trim()
        .trim_start_matches('@')
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    let n = n
        .trim_start_matches("werner")
        .trim_start_matches(',')
        .trim()
        .trim_start_matches("please")
        .trim()
        .trim_start_matches("can you")
        .trim()
        .trim_start_matches("could you")
        .trim()
        .trim_start_matches("show me")
        .trim()
        .trim_start_matches("show")
        .trim()
        .trim_end_matches('?')
        .trim()
        .to_string();
    n
}

/// Agent Ops Schedules All · Jobs · Deliveries filter for `/schedules` instant replies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulesListFilter {
    All,
    Jobs,
    Deliveries,
}

/// Parse Jobs/Deliveries from `/schedules jobs`, `recent deliveries`, etc. Default All.
pub fn parse_schedules_list_filter(content: &str) -> SchedulesListFilter {
    let n = normalize_operator_command(content);
    if n.ends_with(" deliveries")
        || n.ends_with(" delivery")
        || n == "deliveries"
        || n == "delivery"
        || n == "recent deliveries"
        || n == "last deliveries"
        || n == "last delivery"
        || n == "list deliveries"
        || n == "show deliveries"
        || n == "my deliveries"
        || n == "schedules deliveries"
        || n == "/schedules deliveries"
        || n == "delivery list"
    {
        return SchedulesListFilter::Deliveries;
    }
    if n.ends_with(" jobs")
        || n == "list jobs"
        || n == "show jobs"
        || n == "my jobs"
        || n == "active jobs"
        || n == "schedules jobs"
        || n == "/schedules jobs"
        || n == "cron jobs"
        || n == "upcoming jobs"
        || n == "scheduled jobs"
        || n == "list scheduled jobs"
        || n == "my cron jobs"
    {
        return SchedulesListFilter::Jobs;
    }
    SchedulesListFilter::All
}

/// True for Hermes-style `/schedules` / `/cron list` — Agent Ops Jobs/Deliveries parity; not create asks.
pub fn looks_like_schedules_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    // Free-form “schedule a task…” stays with the agent.
    if n.starts_with("schedule a")
        || n.starts_with("schedule me")
        || n.contains(" for tomorrow")
        || n.contains("create")
        || n.contains("add ")
        || n.contains("remove")
        || n.contains("delete")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains("why")
        || n.contains("ticket")
        || n.contains("redmine")
    {
        return false;
    }
    if n.chars().count() > 48 {
        return false;
    }
    matches!(
        n.as_str(),
        "schedules"
            | "/schedules"
            | "list schedules"
            | "show schedules"
            | "my schedules"
            | "what's scheduled"
            | "whats scheduled"
            | "what is scheduled"
            | "upcoming schedules"
            | "upcoming jobs"
            | "scheduled jobs"
            | "list scheduled"
            | "list scheduled jobs"
            | "list jobs"
            | "show jobs"
            | "my jobs"
            | "active jobs"
            | "schedules jobs"
            | "/schedules jobs"
            | "my cron"
            | "my cron jobs"
            | "cron jobs"
            | "/cron"
            | "cron"
            | "/cron list"
            | "cron list"
            | "list cron"
            | "show cron"
            | "deliveries"
            | "delivery"
            | "recent deliveries"
            | "last deliveries"
            | "last delivery"
            | "list deliveries"
            | "show deliveries"
            | "my deliveries"
            | "schedules deliveries"
            | "/schedules deliveries"
            | "delivery list"
    )
}

/// True for short `scrub memory` / `/scrub-memory` operator asks.
pub fn looks_like_memory_scrub_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 40 {
        return false;
    }
    matches!(
        n.as_str(),
        "scrub memory"
            | "scrub memories"
            | "/scrub-memory"
            | "memory scrub"
            | "clean memory"
            | "clean memories"
            | "clean up memory"
            | "purge memory"
            | "purge memories"
            | "remove polluted memory"
            | "scrub polluted memory"
    )
}

/// Zero-LLM schedules report (Agent Ops Schedules All · Jobs · Deliveries filter parity).
pub fn format_schedules_gateway(filter: SchedulesListFilter) -> String {
    let jobs = crate::scheduler::list_schedules_for_ui();
    let deliveries = crate::scheduler::list_scheduler_delivery_awareness();
    let jobs_n = jobs.len();
    let del_n = deliveries.len();
    let title = match filter {
        SchedulesListFilter::All => {
            format!("**Schedules** — {jobs_n} jobs · {del_n} deliveries")
        }
        SchedulesListFilter::Jobs => format!("**Schedules · Jobs** — {jobs_n}"),
        SchedulesListFilter::Deliveries => format!("**Schedules · Deliveries** — {del_n}"),
    };
    let mut lines = vec![title];

    fn job_row(j: &crate::scheduler::ScheduleForUi) -> String {
        let id = j.id.as_deref().unwrap_or("(no id)");
        let kind = if j.cron.is_some() {
            "cron"
        } else if j.at.is_some() {
            "one-shot"
        } else {
            "?"
        };
        let next = j.next_run.as_deref().unwrap_or("—");
        let task = truncate_preview(&j.task, 48);
        format!("• `{id}` · {kind} · next {next} · {task}")
    }

    fn delivery_row(d: &crate::scheduler::DeliveryAwarenessEntry) -> String {
        let age = age_from_rfc3339(&d.utc);
        let sid = d
            .schedule_id
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or("—");
        let preview = truncate_preview(&d.summary, 60);
        format!("• `{sid}` · ch {channel} · {age} · {preview}", channel = d.channel_id)
    }

    const MAX_ROWS: usize = 12;
    match filter {
        SchedulesListFilter::All => {
            if jobs_n == 0 && del_n == 0 {
                lines.push(
                    "_No schedules or deliveries yet — add a job under Agent Ops · Schedules._"
                        .to_string(),
                );
            } else {
                if jobs_n > 0 {
                    lines.push("**Jobs**".to_string());
                    for j in jobs.iter().take(MAX_ROWS) {
                        lines.push(job_row(j));
                    }
                    if jobs_n > MAX_ROWS {
                        lines.push(format!("_…+{} more_", jobs_n - MAX_ROWS));
                    }
                } else {
                    lines.push("_No active jobs._".to_string());
                }
                if del_n > 0 {
                    lines.push("**Deliveries**".to_string());
                    for d in deliveries.iter().take(MAX_ROWS) {
                        lines.push(delivery_row(d));
                    }
                    if del_n > MAX_ROWS {
                        lines.push(format!("_…+{} more_", del_n - MAX_ROWS));
                    }
                } else {
                    lines.push("_No deliveries yet._".to_string());
                }
            }
        }
        SchedulesListFilter::Jobs => {
            if jobs.is_empty() {
                lines.push("_No active jobs — add one under Agent Ops · Schedules._".to_string());
            } else {
                for j in jobs.iter().take(MAX_ROWS) {
                    lines.push(job_row(j));
                }
                if jobs_n > MAX_ROWS {
                    lines.push(format!("_…+{} more_", jobs_n - MAX_ROWS));
                }
            }
        }
        SchedulesListFilter::Deliveries => {
            if deliveries.is_empty() {
                lines.push(
                    "_No deliveries yet — runs a schedule with a Discord channel ID._".to_string(),
                );
            } else {
                for d in deliveries.iter().take(MAX_ROWS) {
                    lines.push(delivery_row(d));
                }
                if del_n > MAX_ROWS {
                    lines.push(format!("_…+{} more_", del_n - MAX_ROWS));
                }
            }
        }
    }
    let mut out = lines.join("\n");
    if out.chars().count() > 1800 {
        out = out.chars().take(1790).collect::<String>() + "…";
    }
    out
}

/// Monitors All · Up · Down · Slow filter for `/monitors` instant replies (UI parity).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorsListFilter {
    All,
    Up,
    Down,
    Slow,
}

/// Menu-bar / UI Slow threshold for UP monitors (ms).
pub const OPS_MONITOR_SLOW_MS: u64 = 2000;

/// Parse Up/Down/Slow from `/monitors down`, `slow monitors`, etc. Default All.
pub fn parse_monitors_list_filter(content: &str) -> MonitorsListFilter {
    let n = normalize_operator_command(content);
    if n.ends_with(" down")
        || n == "down"
        || n == "down monitors"
        || n == "monitors down"
        || n == "/monitors down"
        || n == "sites down"
        || n == "which sites are down"
        || n == "what's down"
        || n == "whats down"
        || n == "what is down"
        || n == "list down"
        || n == "show down"
    {
        return MonitorsListFilter::Down;
    }
    if n.ends_with(" slow")
        || n == "slow monitors"
        || n == "monitors slow"
        || n == "/monitors slow"
        || n == "slow sites"
        || n == "slow websites"
        || n == "which sites are slow"
    {
        return MonitorsListFilter::Slow;
    }
    if n.ends_with(" up")
        || n == "up monitors"
        || n == "monitors up"
        || n == "/monitors up"
        || n == "sites up"
        || n == "up sites"
    {
        return MonitorsListFilter::Up;
    }
    MonitorsListFilter::All
}

fn monitor_row_is_slow(r: &crate::commands::monitors::OpsMonitorRow) -> bool {
    r.is_up == Some(true)
        && r.response_time_ms
            .map(|ms| ms >= OPS_MONITOR_SLOW_MS)
            .unwrap_or(false)
}

/// True for `/monitors` / `list monitors` — Monitors Up/Down/Slow parity; not add/check asks.
pub fn looks_like_monitors_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    if n.contains("create")
        || n.contains("add ")
        || n.contains("remove")
        || n.contains("delete")
        || n.contains("check ")
        || n.contains("check now")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains("why")
        || n.contains("ticket")
        || n.contains("redmine")
        || n.contains("how to")
        || (n.starts_with("monitor ")
            && n != "monitor list"
            && n != "monitor status")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/monitors"
            | "monitors"
            | "list monitors"
            | "show monitors"
            | "my monitors"
            | "all monitors"
            | "monitor list"
            | "website monitors"
            | "site monitors"
            | "sites status"
            | "monitor status"
            | "monitors status"
            | "/monitors up"
            | "monitors up"
            | "up monitors"
            | "sites up"
            | "up sites"
            | "/monitors down"
            | "monitors down"
            | "down monitors"
            | "down"
            | "sites down"
            | "which sites are down"
            | "what's down"
            | "whats down"
            | "what is down"
            | "list down"
            | "/monitors slow"
            | "monitors slow"
            | "slow monitors"
            | "slow sites"
            | "slow websites"
            | "which sites are slow"
    )
}

/// Zero-LLM monitors report (All · Up · Down · Slow filter parity; cached status only).
pub fn format_monitors_gateway(filter: MonitorsListFilter) -> String {
    let rows = crate::commands::monitors::list_monitors_for_ops();
    let up_n = rows.iter().filter(|r| r.is_up == Some(true)).count();
    let down_n = rows.iter().filter(|r| r.is_up == Some(false)).count();
    let slow_n = rows.iter().filter(|r| monitor_row_is_slow(r)).count();
    let total = rows.len();
    let title = match filter {
        MonitorsListFilter::All => {
            format!("**Monitors** — {total} · {up_n} up · {down_n} down · {slow_n} slow")
        }
        MonitorsListFilter::Up => format!("**Monitors · Up** — {up_n}"),
        MonitorsListFilter::Down => format!("**Monitors · Down** — {down_n}"),
        MonitorsListFilter::Slow => {
            format!("**Monitors · Slow** — {slow_n} (≥{OPS_MONITOR_SLOW_MS} ms)")
        }
    };
    let mut lines = vec![title];

    fn row_line(r: &crate::commands::monitors::OpsMonitorRow) -> String {
        let host = {
            let u = r.url.trim();
            if let Some(rest) = u.strip_prefix("https://").or_else(|| u.strip_prefix("http://"))
            {
                rest.split('/').next().unwrap_or(rest)
            } else {
                u
            }
        };
        let label = if r.name.eq_ignore_ascii_case(host) || r.name.is_empty() {
            host.to_string()
        } else {
            format!("{} · {}", r.name, host)
        };
        let age = r
            .checked_at
            .map(|t| age_from_rfc3339(&t.to_rfc3339()))
            .unwrap_or_else(|| "—".into());
        match r.is_up {
            Some(true) => {
                let ms = r
                    .response_time_ms
                    .map(|m| format!("{m} ms"))
                    .unwrap_or_else(|| "—".into());
                let slow_mark = if monitor_row_is_slow(r) { " · slow" } else { "" };
                format!("• ✅ {label} · {ms}{slow_mark} · {age}")
            }
            Some(false) => {
                let reason = r
                    .error
                    .as_deref()
                    .map(|e| truncate_preview(e, 40))
                    .filter(|e| !e.is_empty())
                    .unwrap_or_else(|| "DOWN".into());
                format!("• ❌ {label} · {reason} · {age}")
            }
            None => format!("• ⏳ {label} · waiting · {age}"),
        }
    }

    const MAX_ROWS: usize = 12;
    let filtered: Vec<_> = match filter {
        MonitorsListFilter::All => rows.iter().collect(),
        MonitorsListFilter::Up => rows.iter().filter(|r| r.is_up == Some(true)).collect(),
        MonitorsListFilter::Down => rows.iter().filter(|r| r.is_up == Some(false)).collect(),
        MonitorsListFilter::Slow => rows.iter().filter(|r| monitor_row_is_slow(r)).collect(),
    };

    if filtered.is_empty() {
        let empty = match filter {
            MonitorsListFilter::All => {
                "_No monitors yet — add one under External / Monitors._"
            }
            MonitorsListFilter::Up => "_No UP monitors right now._",
            MonitorsListFilter::Down => "_Nothing is DOWN right now._",
            MonitorsListFilter::Slow => {
                "_No UP site is slow right now (≥2000 ms)._"
            }
        };
        lines.push(empty.to_string());
    } else {
        for r in filtered.iter().take(MAX_ROWS) {
            lines.push(row_line(r));
        }
        if filtered.len() > MAX_ROWS {
            lines.push(format!("_…+{} more_", filtered.len() - MAX_ROWS));
        }
    }
    let mut out = lines.join("\n");
    if out.chars().count() > 1800 {
        out = out.chars().take(1790).collect::<String>() + "…";
    }
    out
}

/// Agent Ops Agents All · On · Off filter for `/agents` instant replies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentsListFilter {
    All,
    On,
    Off,
}

/// Parse On/Off from `/agents on`, `enabled agents`, etc. Default All.
pub fn parse_agents_list_filter(content: &str) -> AgentsListFilter {
    let n = normalize_operator_command(content);
    if n.ends_with(" on")
        || n.ends_with(" enabled")
        || n == "enabled agents"
        || n == "on agents"
        || n == "agents on"
        || n == "/agents on"
    {
        return AgentsListFilter::On;
    }
    if n.ends_with(" off")
        || n.ends_with(" disabled")
        || n == "disabled agents"
        || n == "off agents"
        || n == "agents off"
        || n == "/agents off"
    {
        return AgentsListFilter::Off;
    }
    AgentsListFilter::All
}

/// True for `/agents` / `list agents` — Agent Ops On/Off parity; not create/edit asks.
pub fn looks_like_agents_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 40 {
        return false;
    }
    if n.contains("create")
        || n.contains("add ")
        || n.contains("edit")
        || n.contains("delete")
        || n.contains("remove")
        || n.contains("enable ")
        || n.contains("disable ")
        || n.contains("write")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains("why")
        || n.contains("ticket")
        || n.contains("redmine")
        || n.starts_with("agent:")
        || (n.starts_with("agent ") && !n.starts_with("agent list"))
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/agents"
            | "agents"
            | "list agents"
            | "my agents"
            | "which agents"
            | "what agents"
            | "all agents"
            | "agent list"
            | "agents list"
            | "/agents on"
            | "agents on"
            | "enabled agents"
            | "on agents"
            | "agents enabled"
            | "/agents off"
            | "agents off"
            | "disabled agents"
            | "off agents"
            | "agents disabled"
    )
}

/// Zero-LLM agents report (Agent Ops Agents All · On · Off filter parity).
pub fn format_agents_gateway(filter: AgentsListFilter) -> String {
    let mut agents = crate::agents::load_all_agents();
    agents.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then_with(|| a.id.cmp(&b.id))
    });
    let on_n = agents.iter().filter(|a| a.enabled).count();
    let off_n = agents.len().saturating_sub(on_n);
    let title = match filter {
        AgentsListFilter::All => format!(
            "**Agents** — {on_n} on · {off_n} off ({total} total)",
            total = agents.len()
        ),
        AgentsListFilter::On => format!("**Agents · On** — {on_n}"),
        AgentsListFilter::Off => format!("**Agents · Off** — {off_n}"),
    };
    let mut lines = vec![title];
    fn agent_row(a: &crate::agents::Agent) -> String {
        let mut row = format!("• {}", a.name);
        if let Some(slug) = a.slug.as_deref().filter(|s| !s.is_empty()) {
            row.push_str(&format!(" · `{slug}`"));
        }
        if a.orchestrator {
            row.push_str(" · orchestrator");
        }
        if let Some(model) = a.model.as_deref().filter(|s| !s.is_empty()) {
            row.push_str(&format!(" · {model}"));
        }
        row
    }
    match filter {
        AgentsListFilter::All => {
            if agents.is_empty() {
                lines.push("_No agents yet — add one under Agent Ops._".to_string());
            } else {
                if on_n > 0 {
                    lines.push("**On**".to_string());
                    for a in agents.iter().filter(|a| a.enabled) {
                        lines.push(agent_row(a));
                    }
                }
                if off_n > 0 {
                    lines.push("**Off**".to_string());
                    for a in agents.iter().filter(|a| !a.enabled) {
                        lines.push(agent_row(a));
                    }
                }
            }
        }
        AgentsListFilter::On => {
            let ons: Vec<_> = agents.iter().filter(|a| a.enabled).collect();
            if ons.is_empty() {
                lines.push("_None on right now._".to_string());
            } else {
                for a in ons {
                    lines.push(agent_row(a));
                }
            }
        }
        AgentsListFilter::Off => {
            let offs: Vec<_> = agents.iter().filter(|a| !a.enabled).collect();
            if offs.is_empty() {
                lines.push("_None off right now._".to_string());
            } else {
                for a in offs {
                    lines.push(agent_row(a));
                }
            }
        }
    }
    let mut out = lines.join("\n");
    if out.chars().count() > 1800 {
        out = out.chars().take(1790).collect::<String>() + "…";
    }
    out
}

/// Agent Ops Sessions All · Live · Files filter for `/sessions` instant replies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionsListFilter {
    All,
    Live,
    Files,
}

/// Parse Live/Files from `/sessions live`, `live sessions`, etc. Default All.
pub fn parse_sessions_list_filter(content: &str) -> SessionsListFilter {
    let n = normalize_operator_command(content);
    if n.ends_with(" live")
        || n == "live sessions"
        || n == "sessions live"
        || n == "/sessions live"
        || n == "live session"
        || n == "active sessions"
    {
        return SessionsListFilter::Live;
    }
    if n.ends_with(" files")
        || n == "session files"
        || n == "sessions files"
        || n == "/sessions files"
        || n == "saved sessions"
        || n == "session file"
    {
        return SessionsListFilter::Files;
    }
    SessionsListFilter::All
}

/// True for `/sessions` / `list sessions` — Agent Ops Live/Files parity; not resume/delete asks.
pub fn looks_like_sessions_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 40 {
        return false;
    }
    if n.contains("create")
        || n.contains("add ")
        || n.contains("edit")
        || n.contains("delete")
        || n.contains("remove")
        || n.contains("resume")
        || n.contains("open ")
        || n.contains("write")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains("why")
        || n.contains("ticket")
        || n.contains("redmine")
        || n.contains("scrub")
        || n.starts_with("session:")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/sessions"
            | "sessions"
            | "list sessions"
            | "my sessions"
            | "which sessions"
            | "what sessions"
            | "all sessions"
            | "session list"
            | "sessions list"
            | "/sessions live"
            | "sessions live"
            | "live sessions"
            | "live session"
            | "active sessions"
            | "/sessions files"
            | "sessions files"
            | "session files"
            | "session file"
            | "saved sessions"
    )
}

fn truncate_preview(s: &str, max: usize) -> String {
    let t = s.trim().replace('\n', " ");
    if t.chars().count() <= max {
        return t;
    }
    t.chars().take(max.saturating_sub(1)).collect::<String>() + "…"
}

fn age_from_rfc3339(ts: &str) -> String {
    let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(ts.trim()) else {
        return "—".into();
    };
    let secs = chrono::Utc::now()
        .signed_duration_since(parsed.with_timezone(&chrono::Utc))
        .num_seconds()
        .max(0) as u64;
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86400 {
        format!("{}h", secs / 3600)
    } else {
        format!("{}d", secs / 86400)
    }
}

fn age_from_ms(ms: u64) -> String {
    let now_ms = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(ms);
    let secs = now_ms.saturating_sub(ms) / 1000;
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86400 {
        format!("{}h", secs / 3600)
    } else {
        format!("{}d", secs / 86400)
    }
}

/// Zero-LLM sessions report (Agent Ops Sessions All · Live · Files filter parity).
pub fn format_sessions_gateway(filter: SessionsListFilter) -> String {
    let live = list_live_sessions();
    let files = list_session_files(Some(20)).unwrap_or_default();
    let live_n = live.len();
    let files_n = files.len();
    let title = match filter {
        SessionsListFilter::All => {
            format!("**Sessions** — {live_n} live · {files_n} files")
        }
        SessionsListFilter::Live => format!("**Sessions · Live** — {live_n}"),
        SessionsListFilter::Files => format!("**Sessions · Files** — {files_n}"),
    };
    let mut lines = vec![title];

    fn live_row(s: &LiveSessionSummary) -> String {
        let preview = truncate_preview(&s.preview, 60);
        let age = age_from_rfc3339(&s.last_activity);
        let mut row = format!(
            "• `{}:{}` · {} msg · {age}",
            s.source, s.session_id, s.message_count
        );
        if !preview.is_empty() {
            row.push_str(&format!(" · {preview}"));
        }
        row
    }

    fn file_row(f: &SessionFileSummary) -> String {
        let label = if f.slug.is_empty() {
            f.name.clone()
        } else {
            f.slug.clone()
        };
        let preview = truncate_preview(&f.preview, 50);
        let age = age_from_ms(f.modified_ms);
        let mut row = format!("• `{label}` · {} · {age}", f.source_hint);
        if !preview.is_empty() {
            row.push_str(&format!(" · {preview}"));
        }
        row
    }

    const MAX_ROWS: usize = 12;
    match filter {
        SessionsListFilter::All => {
            if live_n == 0 && files_n == 0 {
                lines.push(
                    "_No live sessions or saved files yet — chat in Discord or AI Chat._"
                        .to_string(),
                );
            } else {
                if live_n > 0 {
                    lines.push("**Live**".to_string());
                    for s in live.iter().take(MAX_ROWS) {
                        lines.push(live_row(s));
                    }
                    if live_n > MAX_ROWS {
                        lines.push(format!("_…+{} more live_", live_n - MAX_ROWS));
                    }
                }
                if files_n > 0 {
                    lines.push("**Files**".to_string());
                    for f in files.iter().take(MAX_ROWS) {
                        lines.push(file_row(f));
                    }
                    if files_n > MAX_ROWS {
                        lines.push(format!("_…+{} more files_", files_n - MAX_ROWS));
                    }
                }
            }
        }
        SessionsListFilter::Live => {
            if live.is_empty() {
                lines.push("_None live right now._".to_string());
            } else {
                for s in live.iter().take(MAX_ROWS) {
                    lines.push(live_row(s));
                }
                if live_n > MAX_ROWS {
                    lines.push(format!("_…+{} more_", live_n - MAX_ROWS));
                }
            }
        }
        SessionsListFilter::Files => {
            if files.is_empty() {
                lines.push("_No session files on disk yet._".to_string());
            } else {
                for f in files.iter().take(MAX_ROWS) {
                    lines.push(file_row(f));
                }
                if files_n > MAX_ROWS {
                    lines.push(format!("_…+{} more_", files_n - MAX_ROWS));
                }
            }
        }
    }
    let mut out = lines.join("\n");
    if out.chars().count() > 1800 {
        out = out.chars().take(1790).collect::<String>() + "…";
    }
    out
}

/// Agent Ops Knowledge All · Discord · Core filter for `/knowledge` instant replies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnowledgeListFilter {
    All,
    Discord,
    Core,
}

/// Parse Discord/Core from `/knowledge discord`, `core knowledge`, etc. Default All.
pub fn parse_knowledge_list_filter(content: &str) -> KnowledgeListFilter {
    let n = normalize_operator_command(content);
    if n.ends_with(" discord")
        || n == "discord knowledge"
        || n == "knowledge discord"
        || n == "/knowledge discord"
        || n == "discord memory"
        || n == "discord memories"
        || n == "channel memory"
        || n == "channel memories"
    {
        return KnowledgeListFilter::Discord;
    }
    if n.ends_with(" core")
        || n == "core knowledge"
        || n == "knowledge core"
        || n == "/knowledge core"
        || n == "soul knowledge"
        || n == "global knowledge"
        || n == "main knowledge"
    {
        return KnowledgeListFilter::Core;
    }
    KnowledgeListFilter::All
}

fn knowledge_row_is_discord(kind: &str) -> bool {
    kind.eq_ignore_ascii_case("discord")
}

fn knowledge_row_is_core(kind: &str) -> bool {
    matches!(
        kind.to_ascii_lowercase().as_str(),
        "soul" | "global" | "main"
    )
}

/// True for `/knowledge` / `list knowledge` — Agent Ops Discord/Core parity; not scrub/edit asks.
pub fn looks_like_knowledge_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 40 {
        return false;
    }
    if n.contains("create")
        || n.contains("add ")
        || n.contains("edit")
        || n.contains("delete")
        || n.contains("remove")
        || n.contains("scrub")
        || n.contains("write")
        || n.contains("append")
        || n.contains("save ")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains("why")
        || n.contains("ticket")
        || n.contains("redmine")
        || n.contains("pollut")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/knowledge"
            | "knowledge"
            | "list knowledge"
            | "my knowledge"
            | "which knowledge"
            | "what knowledge"
            | "all knowledge"
            | "knowledge list"
            | "knowledge files"
            | "knowledge file"
            | "/knowledge discord"
            | "knowledge discord"
            | "discord knowledge"
            | "discord memory"
            | "discord memories"
            | "channel memory"
            | "channel memories"
            | "/knowledge core"
            | "knowledge core"
            | "core knowledge"
            | "soul knowledge"
            | "global knowledge"
            | "main knowledge"
    )
}

/// Zero-LLM knowledge report (Agent Ops Knowledge All · Discord · Core filter parity).
pub fn format_knowledge_gateway(filter: KnowledgeListFilter) -> String {
    let files = list_memory_files().unwrap_or_default();
    let discord_n = files
        .iter()
        .filter(|f| knowledge_row_is_discord(&f.kind))
        .count();
    let core_n = files
        .iter()
        .filter(|f| knowledge_row_is_core(&f.kind))
        .count();
    let title = match filter {
        KnowledgeListFilter::All => {
            format!("**Knowledge** — {discord_n} Discord · {core_n} Core")
        }
        KnowledgeListFilter::Discord => format!("**Knowledge · Discord** — {discord_n}"),
        KnowledgeListFilter::Core => format!("**Knowledge · Core** — {core_n}"),
    };
    let mut lines = vec![title];

    fn knowledge_row(f: &MemoryFileSummary) -> String {
        let age = age_from_ms(f.modified_ms);
        format!(
            "• `{}` · {} · {} lines · {age}",
            f.name, f.kind, f.line_count
        )
    }

    const MAX_ROWS: usize = 12;
    match filter {
        KnowledgeListFilter::All => {
            if files.is_empty() {
                lines.push(
                    "_No knowledge files yet — soul/global/main or Discord channel memory._"
                        .to_string(),
                );
            } else {
                if discord_n > 0 {
                    lines.push("**Discord**".to_string());
                    for f in files
                        .iter()
                        .filter(|f| knowledge_row_is_discord(&f.kind))
                        .take(MAX_ROWS)
                    {
                        lines.push(knowledge_row(f));
                    }
                    if discord_n > MAX_ROWS {
                        lines.push(format!("_…+{} more_", discord_n - MAX_ROWS));
                    }
                }
                if core_n > 0 {
                    lines.push("**Core**".to_string());
                    for f in files
                        .iter()
                        .filter(|f| knowledge_row_is_core(&f.kind))
                        .take(MAX_ROWS)
                    {
                        lines.push(knowledge_row(f));
                    }
                    if core_n > MAX_ROWS {
                        lines.push(format!("_…+{} more_", core_n - MAX_ROWS));
                    }
                }
            }
        }
        KnowledgeListFilter::Discord => {
            let rows: Vec<_> = files
                .iter()
                .filter(|f| knowledge_row_is_discord(&f.kind))
                .collect();
            if rows.is_empty() {
                lines.push("_No Discord channel memory files yet._".to_string());
            } else {
                for f in rows.iter().take(MAX_ROWS) {
                    lines.push(knowledge_row(f));
                }
                if rows.len() > MAX_ROWS {
                    lines.push(format!("_…+{} more_", rows.len() - MAX_ROWS));
                }
            }
        }
        KnowledgeListFilter::Core => {
            let rows: Vec<_> = files
                .iter()
                .filter(|f| knowledge_row_is_core(&f.kind))
                .collect();
            if rows.is_empty() {
                lines.push("_No Core knowledge files (soul / global / main) yet._".to_string());
            } else {
                for f in rows.iter().take(MAX_ROWS) {
                    lines.push(knowledge_row(f));
                }
                if rows.len() > MAX_ROWS {
                    lines.push(format!("_…+{} more_", rows.len() - MAX_ROWS));
                }
            }
        }
    }
    let mut out = lines.join("\n");
    if out.chars().count() > 1800 {
        out = out.chars().take(1790).collect::<String>() + "…";
    }
    out
}

/// True for short `/status` / `/health` operator asks — not free-form “status of …”.
pub fn looks_like_status_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.contains(" of ") || n.contains(" for ") || n.contains("ticket") || n.contains("redmine")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/status"
            | "bot status"
            | "app status"
            | "mac-stats status"
            | "system status"
            | "/health"
            | "health check"
            | "bot health"
            | "system health"
            | "how healthy"
            | "are you healthy"
            |         "/version"
            | "app version"
            | "mac-stats version"
            | "what version"
            | "which version"
            | "is everything ok"
            | "everything ok"
            | "everything working"
            | "all systems go"
            | "systems ok"
            | "system ok"
            | "all good on your end"
            | "you all good"
            | "are you ok"
            | "are you okay"
    )
}

/// One-screen operator status: version, Discord gateway, digest, next schedule, last delivery.
pub fn format_status_gateway() -> String {
    let version = crate::config::Config::version();
    let digest = load_digest_summary();
    let snap = crate::scheduler::scheduler_operator_snapshot();
    let mut lines = vec![
        format!("**mac-stats v{version}**"),
        crate::discord::format_discord_gateway_insights_line(),
        format!(
            "Digest: **{}** open · **{}** stale{}",
            digest.open_count,
            digest.stale_count,
            if digest.source.is_empty() {
                String::new()
            } else {
                format!(" · {}", digest.source)
            }
        ),
    ];
    let mut sched = format!("Schedules: **{}**", snap.total_entries);
    if let Some(secs) = snap.seconds_until_next_fire {
        let when = if secs < 3600 {
            format!("{}m", (secs / 60).max(1))
        } else {
            format!("{}h", (secs + 1800) / 3600)
        };
        let preview = snap
            .next_task_preview
            .as_deref()
            .map(|p| format!(" ({})", p.chars().take(36).collect::<String>()))
            .unwrap_or_default();
        sched.push_str(&format!(" · next {when}{preview}"));
    }
    lines.push(sched);
    if let Some(last) = crate::scheduler::list_scheduler_delivery_awareness()
        .into_iter()
        .next()
    {
        let preview: String = last.summary.chars().take(72).collect();
        lines.push(format!("Last delivery: {} · {}", last.utc, preview));
    }
    lines.join("\n")
}

/// True for `/ops` / `/help` operator command list — not free-form “help me with …”.
pub fn looks_like_ops_help_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    // Free-form help (“help me write…”) stays with the agent.
    if n.starts_with("help me") || n.starts_with("help with") || n.contains(" write ") {
        return false;
    }
    matches!(
        n.as_str(),
        "/ops"
            | "ops"
            | "/ops help"
            | "ops help"
            | "/help"
            | "help"
            | "operator help"
            | "operator commands"
            | "bot commands"
            | "/commands"
            | "commands"
            | "command list"
            | "list commands"
            | "command menu"
            | "what commands"
            | "what can you do"
            | "available commands"
    )
}

/// Zero-LLM operator replies for AI Chat / Ollama paths (parity with Discord fast handlers).
pub fn try_operator_instant_reply(content: &str) -> Option<String> {
    if looks_like_ops_help_request(content) {
        return Some(format_ops_help_gateway());
    }
    if looks_like_status_request(content) {
        return Some(format_status_gateway());
    }
    if looks_like_insights_request(content) {
        let days = parse_insights_days(content);
        let insights = compute_runs_insights_for(80, days);
        return Some(format_runs_insights_gateway(&insights));
    }
    if looks_like_failed_runs_request(content) {
        let days = parse_insights_days(content);
        return Some(format_failed_runs_gateway(days));
    }
    if looks_like_slow_runs_request(content) {
        let days = parse_insights_days(content);
        return Some(format_slow_runs_gateway(days));
    }
    if looks_like_instant_runs_request(content) {
        let days = parse_insights_days(content);
        return Some(format_instant_runs_gateway(days));
    }
    if looks_like_direct_runs_request(content) {
        let days = parse_insights_days(content);
        return Some(format_direct_runs_gateway(days));
    }
    if looks_like_lite_runs_request(content) {
        let days = parse_insights_days(content);
        return Some(format_lite_runs_gateway(days));
    }
    if looks_like_agents_request(content) {
        let filter = parse_agents_list_filter(content);
        return Some(format_agents_gateway(filter));
    }
    if looks_like_sessions_request(content) {
        let filter = parse_sessions_list_filter(content);
        return Some(format_sessions_gateway(filter));
    }
    if looks_like_knowledge_request(content) {
        let filter = parse_knowledge_list_filter(content);
        return Some(format_knowledge_gateway(filter));
    }
    if looks_like_schedules_request(content) {
        let filter = parse_schedules_list_filter(content);
        return Some(format_schedules_gateway(filter));
    }
    if looks_like_monitors_request(content) {
        let filter = parse_monitors_list_filter(content);
        return Some(format_monitors_gateway(filter));
    }
    if looks_like_memory_scrub_request(content) {
        let (files, removed) = crate::commands::session_search::scrub_polluted_memory_files();
        return Some(if removed == 0 {
            "Memory scrub: nothing polluted to remove.".to_string()
        } else {
            format!(
                "Memory scrub: removed **{}** polluted line(s) from **{}** file(s).",
                removed, files
            )
        });
    }
    if looks_like_digest_request(content) {
        let line = refresh_agent_digest();
        let summary = load_digest_summary();
        let mut reply = line;
        if summary.open_count > 0 {
            reply.push_str("\n**Open:** ");
            reply.push_str(&summary.open_hints.join("; "));
        }
        return Some(reply);
    }
    None
}

/// Short Discord menu of cheap operator commands (no Ollama).
pub fn format_ops_help_gateway() -> String {
    let version = crate::config::Config::version();
    format!(
        "**mac-stats v{version} — operator commands** (instant, no Ollama)\n\
• `/status` · `/health` · `/version` — one-screen health\n\
• `/insights` · `/insights 7` — runs.jsonl report (+ optional day window)\n\
• `/failed` · `/failed 7` — recent failed turns from runs.jsonl\n\
• `/slow` · `/slow 7` — recent slow turns (≥{slow_ms} ms wall time)\n\
• `/instant` · `/lite` · `/direct` · `/instant 7` — recent instant-, lite-, or direct-lane turns\n\
• `/agents` · `/agents on` · `/agents off` — Agent Ops On/Off list\n\
• `/sessions` · `/sessions live` · `/sessions files` — Agent Ops Live/Files list\n\
• `/knowledge` · `/knowledge discord` · `/knowledge core` — Agent Ops Knowledge list\n\
• `/schedules` · `/schedules jobs` · `/schedules deliveries` · `/cron list` — Agent Ops Jobs/Deliveries list\n\
• `/monitors` · `/monitors up` · `/monitors down` · `/monitors slow` — External / Monitors list\n\
• `/digest` — refresh digester (latest.md/json)\n\
• `scrub memory` — remove polluted memory lines\n\
• `stop` / `cancel` / `interrupt` — interrupt an in-flight run\n\
• `/ops` · `/help` — this menu\n\
• Voice notes — transcribed locally (Ollama audio) then answered like text\n\
\n\
**Scheduled:** wake-up 06:00 · CHANGELOG hygiene Mondays 10:00 · UI review Wednesdays 11:00 (`docs/041_ui_command_center.md`)",
        slow_ms = OPS_RUNS_SLOW_MS,
    )
}

/// Digester Slowest parity: exclude shipped instant noise from insights p50/slowest.
fn is_insights_slowest_noise(lane: &str, wall_ms: u64, tools: &[String], question: &str) -> bool {
    if lane == "instant" && wall_ms < 2_000 {
        return true;
    }
    let q = question.to_lowercase();
    // Scheduled SKILL weekly reviews — harness work, not Discord UX latency (v0.1.377).
    if q.trim_start().starts_with("skill:") {
        return true;
    }
    let has_search_tool = tools.iter().any(|t| {
        let u = t.to_uppercase();
        u.contains("BRAVE") || u.contains("PERPLEXITY")
    });
    // Pre-Open-Meteo weather that burned Brave/Perplexity (incl. climate/clima/klima voice STT).
    if q.contains("climate") || q.contains("clima") || q.contains("klima") {
        return true;
    }
    if (q.contains("weather") || q.contains("wether") || q.contains("masnou")) && has_search_tool
    {
        return true;
    }
    // Pre-ship over-tooled turns that are now instant (may still list tools).
    if ((q.contains("improvement") || q.contains("what shipped") || q.contains("last night"))
        && (q.contains("overnight")
            || q.contains("coding")
            || q.contains("session")
            || q.contains("lately")
            || q.contains("recently")
            || q.contains("improvement loop")
            || q.contains("harness")
            || q.contains("today")
            || q.contains("this morning")
            || q.contains("so far today")
            || q.contains("morning surprise"))
        && !q.contains("workflow")
        && !q.contains("ticket")
        && !q.contains("redmine"))
        || ((q.contains("planned")
            || q.contains("what's the plan")
            || q.contains("whats the plan")
            || q.contains("plan for")
            || q.contains("agenda"))
            && (q.contains("tonight")
                || q.contains("this night")
                || q.contains("this evening")
                || q.contains("for the night")
                || q.contains("evening"))
            && !q.contains("ticket")
            && !q.contains("redmine"))
    {
        return true;
    }
    // `/failed` / failed-runs operator asks (v0.1.695).
    if (q.contains("failed run")
        || q.contains("what failed")
        || q.contains("any failures")
        || q.contains("/failed")
        || q == "failures"
        || q == "failed")
        && !q.contains("why did")
        && !q.contains("why ")
        && !q.contains("explain")
        && !q.contains("ticket")
        && !q.contains("redmine")
    {
        return true;
    }
    // `/slow` / slow-runs operator asks (v0.1.696).
    if (q.contains("slow run")
        || q.contains("what's slow")
        || q.contains("whats slow")
        || q.contains("what is slow")
        || q.contains("/slow")
        || q.contains("slowest runs")
        || q == "slow")
        && !q.contains("why ")
        && !q.contains("why is")
        && !q.contains("explain")
        && !q.contains("monitor")
        && !q.contains("website")
        && !q.contains(" ticket")
        && !q.contains("redmine")
    {
        return true;
    }
    // `/instant` / instant-lane operator asks (v0.1.697).
    if (q.contains("instant run")
        || q.contains("instant lane")
        || q.contains("instant turns")
        || q.contains("/instant")
        || q == "instant")
        && !q.contains("why ")
        && !q.contains("explain")
        && !q.contains("make ")
        && !q.contains(" ticket")
        && !q.contains("redmine")
    {
        return true;
    }
    // `/direct` / direct-lane operator asks (v0.1.697).
    if (q.contains("direct run")
        || q.contains("direct lane")
        || q.contains("direct turns")
        || q.contains("/direct")
        || q == "direct")
        && !q.contains("why ")
        && !q.contains("explain")
        && !q.contains(" ticket")
        && !q.contains("redmine")
    {
        return true;
    }
    // `/lite` / lite-lane operator asks (v0.1.704).
    if (q.contains("lite run")
        || q.contains("lite lane")
        || q.contains("lite turns")
        || q.contains("/lite")
        || q == "lite")
        && !q.contains("why ")
        && !q.contains("explain")
        && !q.contains(" ticket")
        && !q.contains("redmine")
    {
        return true;
    }
    // `/agents` operator asks (v0.1.705).
    if (q.contains("/agents")
        || q == "agents"
        || q.contains("list agents")
        || q.contains("enabled agents")
        || q.contains("disabled agents")
        || q == "agents on"
        || q == "agents off")
        && !q.contains("why")
        && !q.contains("create")
        && !q.contains(" ticket")
        && !q.contains("redmine")
    {
        return true;
    }
    // `/sessions` operator asks (v0.1.706).
    if (q.contains("/sessions")
        || q == "sessions"
        || q.contains("list sessions")
        || q.contains("live sessions")
        || q.contains("session files")
        || q.contains("saved sessions")
        || q == "sessions live"
        || q == "sessions files")
        && !q.contains("why")
        && !q.contains("create")
        && !q.contains("resume")
        && !q.contains(" ticket")
        && !q.contains("redmine")
    {
        return true;
    }
    // `/knowledge` operator asks (v0.1.707).
    if (q.contains("/knowledge")
        || q == "knowledge"
        || q.contains("list knowledge")
        || q.contains("knowledge files")
        || q.contains("discord knowledge")
        || q.contains("core knowledge")
        || q == "knowledge discord"
        || q == "knowledge core")
        && !q.contains("why")
        && !q.contains("create")
        && !q.contains("scrub")
        && !q.contains("save ")
        && !q.contains(" ticket")
        && !q.contains("redmine")
    {
        return true;
    }
    // `/schedules` jobs/deliveries operator asks (v0.1.708).
    if (q.contains("/schedules")
        || q == "schedules"
        || q.contains("list schedules")
        || q.contains("schedules jobs")
        || q.contains("schedules deliveries")
        || q.contains("list deliveries")
        || q.contains("recent deliveries")
        || q.contains("upcoming jobs")
        || q.contains("/cron")
        || q == "cron list")
        && !q.contains("why")
        && !q.contains("schedule a")
        && !q.contains("create")
        && !q.contains(" for tomorrow")
        && !q.contains(" ticket")
        && !q.contains("redmine")
    {
        return true;
    }
    // `/monitors` up/down/slow operator asks (v0.1.709).
    if (q.contains("/monitors")
        || q == "monitors"
        || q.contains("list monitors")
        || q.contains("show monitors")
        || q.contains("monitors up")
        || q.contains("monitors down")
        || q.contains("monitors slow")
        || q.contains("down monitors")
        || q.contains("slow monitors")
        || q.contains("up monitors")
        || q.contains("sites down")
        || q.contains("which sites are down")
        || q.contains("which sites are slow")
        || q == "what's down"
        || q == "whats down"
        || q == "what is down")
        && !q.contains("why")
        && !q.contains("add ")
        && !q.contains("create")
        && !q.contains("check ")
        && !q.contains(" ticket")
        && !q.contains("redmine")
    {
        return true;
    }
    if !tools.is_empty() {
        return false;
    }
    // Zero-tool patterns now covered by instant lanes.
    if !q.contains('?')
        && (matches!(
            q.trim(),
            "ok" | "okay"
                | "k"
                | "kk"
                | "cool"
                | "nice"
                | "nice one"
                | "nice answer"
                | "got it"
                | "all good"
                | "np"
                | "no worries"
                | "bye"
                | "goodbye"
                | "cya"
                | "see you"
                | "later"
                | "perfect"
                | "great"
                | "awesome"
                | "neat"
                | "sweet"
                | "alright"
                | "sounds good"
                | "fair enough"
                | "👍"
                | "👌"
        ) || ((q.starts_with("ok")
            || q.starts_with("okay")
            || q.starts_with("cool")
            || q.starts_with("nice")
            || q.starts_with("got it")
            || q.starts_with("alright")
            || q.starts_with("no worries")
            || q.starts_with("sounds good"))
            && (q.chars().count() <= 48
                || q.contains("no worries")
                || q.contains("bye")
                || q.contains("myself")
                || q.contains("later")
                || q.contains("all good")
                || q.contains("find out"))))
    {
        return true;
    }
    if (q.starts_with("you are ") || q.starts_with("you're ") || q.starts_with("youre "))
        && !q.contains('?')
        && q.chars().count() <= 180
        && (q.contains("working for")
            || q.contains("online")
            || q.contains("assistant")
            || q.contains(" agent")
            || q.contains("bot")
            || q.contains("on various channel"))
    {
        return true;
    }
    if q.contains("wake-up")
        || q.contains("wakeup")
        || q.contains("wake up")
        || ((q.contains("improvement") || q.contains("what shipped") || q.contains("last night"))
            && (q.contains("overnight")
                || q.contains("coding")
                || q.contains("session")
                || q.contains("lately")
                || q.contains("recently")
                || q.contains("improvement loop")
                || q.contains("harness")
                || q.contains("today")
                || q.contains("this morning")
                || q.contains("so far today")
                || q.contains("morning surprise")))
    {
        return true;
    }
    if q.contains("version")
        && (q.contains("you") || q.contains("app") || q.contains("mac-stats") || q.starts_with("what"))
    {
        return true;
    }
    if (q.contains("discord") || q.contains("amvara"))
        && (q.contains("talking")
            || q.contains("channel")
            || q.contains("other agent")
            || q.contains("ok talking")
            || q.contains("cross check"))
    {
        return true;
    }
    if q.contains("redmine")
        && (q.contains("talk to") || q.contains("chat with") || q.contains("message "))
        && !q.contains("ticket")
        && !q.contains("issue")
    {
        return true;
    }
    false
}

fn classify_candidate(
    lane: &str,
    wall_ms: u64,
    tools: &[String],
    question: &str,
    request_id: &str,
) -> Option<RunInsightCandidate> {
    let q = question.to_lowercase();
    let looks_version = q.contains("version")
        && (q.contains("you") || q.contains("app") || q.contains("mac-stats") || q.starts_with("what"));
    if looks_version && lane != "instant" && wall_ms >= 500 {
        return Some(RunInsightCandidate {
            kind: "promote_instant".into(),
            reason: "Version ask should stay on instant lane".into(),
            wall_ms,
            lane: lane.into(),
            question_preview: question.chars().take(80).collect(),
            request_id: request_id.into(),
        });
    }
    let n_up = q.trim_end_matches(['?', '!', '.']).trim();
    let looks_uptime = matches!(
        n_up,
        "uptime"
            | "up time"
            | "how long up"
            | "how long have you been up"
            | "how long are you up"
            | "how long running"
            | "how long have you been running"
            | "process uptime"
            | "app uptime"
    ) || (n_up.contains("uptime") && n_up.chars().count() <= 32 && !n_up.contains("system") && !n_up.contains("machine"))
        || (n_up.starts_with("how long")
            && (n_up.contains("up") || n_up.contains("running"))
            && n_up.chars().count() <= 48
            && !n_up.contains("system")
            && !n_up.contains("machine"));
    if looks_uptime && lane != "instant" && wall_ms >= 500 {
        return Some(RunInsightCandidate {
            kind: "promote_instant".into(),
            reason: "Uptime ask should stay on instant lane".into(),
            wall_ms,
            lane: lane.into(),
            question_preview: question.chars().take(80).collect(),
            request_id: request_id.into(),
        });
    }
    let looks_presence = matches!(
        q.trim_end_matches(['?', '!', '.']).trim(),
        "who are you"
            | "who r you"
            | "what are you"
            | "are you there"
            | "are you online"
            | "you there"
            | "you online"
            | "still there"
            | "still here"
            | "still online"
            | "are you up"
            | "you up"
            | "you around"
            | "you good"
            | "you ok"
            | "you okay"
            | "how are you"
            | "how're you"
            | "how's it going"
            | "hows it going"
            | "how are things"
            | "whats up"
            | "what's up"
            | "anything else"
            | "need anything"
            | "need anything else"
    );
    if looks_presence && lane != "instant" && wall_ms >= 500 {
        return Some(RunInsightCandidate {
            kind: "promote_instant".into(),
            reason: "Presence/who-are-you ask should stay on instant lane".into(),
            wall_ms,
            lane: lane.into(),
            question_preview: question.chars().take(80).collect(),
            request_id: request_id.into(),
        });
    }
    let asks_improvements = q.contains("improvement")
        || q.contains("what shipped")
        || q.contains("what changed")
        || q.contains("what did you ship")
        || q.contains("what did you change");
    let overnight_context = q.contains("last night")
        || q.contains("overnight")
        || q.contains("coding session")
        || q.contains("last night's")
        || q.contains("lately")
        || q.contains("recently")
        || q.contains("improvement loop")
        || q.contains("harness loop")
        || q.contains("overnight harness")
        || q.contains("today")
        || q.contains("this morning")
        || q.contains("so far today")
        || q.contains("morning surprise");
    let product_changelog = (q.contains("changelog")
        || q.contains("enhancement")
        || q.contains("latest change")
        || q.contains("latests change")
        || q.contains("recent change")
        || q.contains("latest version"))
        && (q.contains("mac-stats")
            || q.contains("mac stats")
            || q.contains("your changelog")
            || q.contains("your latest")
            || q.contains("your latests")
            || (q.contains("your") && q.contains("version")));
    if ((asks_improvements && overnight_context) || product_changelog)
        && !q.contains("redmine")
        && !q.contains("ticket")
        && !q.contains("workflow")
        && lane != "instant"
        && wall_ms >= 500
    {
        return Some(RunInsightCandidate {
            kind: "promote_instant".into(),
            reason: "Overnight improvements ask should stay on instant lane".into(),
            wall_ms,
            lane: lane.into(),
            question_preview: question.chars().take(80).collect(),
            request_id: request_id.into(),
        });
    }
    let asks_plan = q.contains("planned")
        || q.contains("what's the plan")
        || q.contains("whats the plan")
        || q.contains("plan for")
        || q.contains("agenda");
    let night_ctx = q.contains("tonight")
        || q.contains("this night")
        || q.contains("this evening")
        || q.contains("for the night");
    if asks_plan
        && night_ctx
        && !q.contains("redmine")
        && !q.contains("ticket")
        && lane != "instant"
        && wall_ms >= 500
    {
        return Some(RunInsightCandidate {
            kind: "promote_instant".into(),
            reason: "Tonight/schedule plan ask should stay on instant lane".into(),
            wall_ms,
            lane: lane.into(),
            question_preview: question.chars().take(80).collect(),
            request_id: request_id.into(),
        });
    }
    let n_cap = q.trim_end_matches(['?', '!', '.']).trim();
    let looks_capabilities = matches!(
        n_cap,
        "what can you do"
            | "what do you do"
            | "what are you able to do"
            | "what are your capabilities"
            | "your capabilities"
            | "capabilities"
            | "help"
            | "commands"
            | "what can you help with"
            | "how can you help"
    ) || (n_cap.starts_with("what can you") && n_cap.chars().count() <= 40)
        || (n_cap.starts_with("how can you help") && n_cap.chars().count() <= 40);
    if looks_capabilities
        && !q.contains("redmine")
        && !q.contains("ticket")
        && lane != "instant"
        && wall_ms >= 500
    {
        return Some(RunInsightCandidate {
            kind: "promote_instant".into(),
            reason: "Capabilities/help ask should stay on instant lane".into(),
            wall_ms,
            lane: lane.into(),
            question_preview: question.chars().take(80).collect(),
            request_id: request_id.into(),
        });
    }
    let about_channels = q.contains("channel");
    let about_other_agents = q.contains("another agent")
        || q.contains("other agent")
        || q.contains("other agents")
        || q.contains("another bot")
        || q.contains("other bot")
        || q.contains("other bots");
    let discordish = q.contains("discord")
        || q.contains("amvara")
        || q.contains("server")
        || q.contains("guild");
    let looks_discord_presence = discordish
        && (q.contains("talking on")
            || q.contains("ok talking")
            || q.contains("okay talking")
            || q.contains("are you online")
            || q.contains("are you connected")
            || (q.contains("cross check") && q.contains("talking")));
    let looks_discord_reach = looks_discord_presence
        || ((about_channels || about_other_agents)
            && (q.contains("can you see")
                || q.contains("do you see")
                || q.contains("see channels")
                || q.contains("talking to")
                || q.contains("talk to another")
                || q.contains("talk to other")
                || q.contains("are you talking")
                || q.contains("may you")
                || q.contains("be talking"))
            && !q.contains("list all")
            && !q.contains("discord_api")
            && !q.contains("post to")
            && !q.contains("send to"));
    if looks_discord_reach && lane != "instant" && wall_ms >= 500 {
        return Some(RunInsightCandidate {
            kind: "promote_instant".into(),
            reason: "Discord reach/channels meta-ask should stay on instant lane".into(),
            wall_ms,
            lane: lane.into(),
            question_preview: question.chars().take(80).collect(),
            request_id: request_id.into(),
        });
    }
    let looks_time = (q.contains("what time") || q.contains("what's the time") || q == "time")
        && !q.contains("timezone");
    let looks_greeting = matches!(
        q.trim_end_matches(['?', '!', '.']).trim(),
        "hi" | "hello" | "hey"
            | "hey there"
            | "yo"
            | "sup"
            | "hola"
            | "good morning"
            | "good afternoon"
            | "good evening"
            | "gm"
            | "thanks"
            | "thank you"
            | "thx"
            | "cheers"
            | "ok"
            | "okay"
            | "cool"
            | "nice"
            | "got it"
            | "no worries"
            | "bye"
    );
    let looks_ack = !q.contains('?')
        && (q.starts_with("ok") || q.starts_with("okay") || q.starts_with("nice") || q.starts_with("got it"))
        && (q.chars().count() <= 48
            || q.contains("no worries")
            || q.contains("myself")
            || q.contains("find out"));
    let looks_identity = !q.contains('?')
        && q.chars().count() <= 180
        && (q.starts_with("you are ") || q.starts_with("you're ") || q.starts_with("youre "))
        && (q.contains("working for")
            || q.contains("online")
            || q.contains("assistant")
            || q.contains(" agent")
            || q.contains("bot")
            || q.contains("on various channel"));
    if (looks_time || looks_greeting || looks_ack || looks_identity)
        && lane != "instant"
        && wall_ms >= 500
    {
        return Some(RunInsightCandidate {
            kind: "promote_instant".into(),
            reason: if looks_time {
                "Time/date ask should stay on instant lane".into()
            } else if looks_identity {
                "Identity/role affirmation should stay on instant lane".into()
            } else if looks_ack {
                "Short ack/sign-off should stay on instant lane".into()
            } else {
                "Greeting/thanks should stay on instant lane".into()
            },
            wall_ms,
            lane: lane.into(),
            question_preview: question.chars().take(80).collect(),
            request_id: request_id.into(),
        });
    }
    if tools.is_empty() && wall_ms >= 8_000 && lane != "instant" {
        return Some(RunInsightCandidate {
            kind: "slow_zero_tool".into(),
            reason: "Slow turn with no tools — candidate for lite/instant".into(),
            wall_ms,
            lane: lane.into(),
            question_preview: question.chars().take(80).collect(),
            request_id: request_id.into(),
        });
    }
    None
}

/// Short Discord/gateway report (Hermes `/insights` lite).
pub fn format_runs_insights_gateway(insights: &RunsInsights) -> String {
    let mut lines = Vec::new();
    if insights.turns == 0 {
        let empty_msg = match insights.window_days {
            Some(d) => format!("No turns in `~/.mac-stats/runs.jsonl` for the last **{d}** days."),
            None => "No turns in `~/.mac-stats/runs.jsonl` yet.".into(),
        };
        lines.push(empty_msg);
    } else {
        let title = match insights.window_days {
            Some(d) => format!("**mac-stats insights** (last **{d}** days · runs.jsonl)"),
            None => "**mac-stats insights** (runs.jsonl)".to_string(),
        };
        lines.push(title);
        lines.push(format!(
            "Turns: **{}** · ok {} · fail {} · p50 **{}** · mean {} · max {}{}",
            insights.turns,
            insights.ok_count,
            insights.fail_count,
            if insights.latency_sample == 0 {
                "n/a".into()
            } else {
                format!("{} ms", insights.p50_ms)
            },
            if insights.latency_sample == 0 {
                "n/a".into()
            } else {
                format!("{}", insights.mean_ms)
            },
            if insights.latency_sample == 0 {
                "n/a".into()
            } else {
                format!("{}", insights.max_ms)
            },
            if insights.latency_sample > 0 && insights.latency_sample < insights.turns {
                format!(
                    " · latency sample {}/{}",
                    insights.latency_sample, insights.turns
                )
            } else {
                String::new()
            }
        ));
    }
    lines.push(crate::discord::format_discord_gateway_insights_line());
    {
        let mut digest = format!(
            "Digest: **{}** open · **{}** stale",
            insights.digest_open_count, insights.digest_stale_count
        );
        if !insights.digest_source.is_empty() {
            digest.push_str(&format!(" · {}", insights.digest_source));
        }
        if !insights.digest_generated_at.is_empty() {
            digest.push_str(&format!(" · {}", insights.digest_generated_at));
        }
        lines.push(digest);
        if !insights.digest_open_hints.is_empty() {
            lines.push(format!(
                "Open hints: {}",
                insights.digest_open_hints.iter().take(3).cloned().collect::<Vec<_>>().join("; ")
            ));
        }
    }
    {
        let snap = crate::scheduler::scheduler_operator_snapshot();
        let mut sched = format!("Schedules: **{}**", snap.total_entries);
        if let Some(secs) = snap.seconds_until_next_fire {
            let when = if secs < 3600 {
                format!("{}m", (secs / 60).max(1))
            } else {
                format!("{}h", (secs + 1800) / 3600)
            };
            let preview = snap
                .next_task_preview
                .as_deref()
                .map(|p| {
                    let t: String = p.chars().take(36).collect();
                    format!(" ({t})")
                })
                .unwrap_or_default();
            sched.push_str(&format!(" · next {when}{preview}"));
        }
        lines.push(sched);
        if let Some(last) = crate::scheduler::list_scheduler_delivery_awareness()
            .into_iter()
            .next()
        {
            let preview: String = last.summary.chars().take(60).collect();
            lines.push(format!("Last delivery: {} · {}", last.utc, preview));
        }
    }
    if !insights.by_lane.is_empty() {
        let lanes = insights
            .by_lane
            .iter()
            .map(|(k, v)| format!("{k}:{v}"))
            .collect::<Vec<_>>()
            .join(" · ");
        lines.push(format!("Lanes: {lanes}"));
    }
    if !insights.by_tool.is_empty() {
        let tools = insights
            .by_tool
            .iter()
            .take(8)
            .map(|(k, v)| format!("{k}×{v}"))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!("Top tools: {tools}"));
    }
    if !insights.slowest.is_empty() {
        lines.push("**Slowest**".into());
        for s in insights.slowest.iter().take(3) {
            let q = if s.question_preview.is_empty() {
                "(empty)"
            } else {
                &s.question_preview
            };
            lines.push(format!("• {} ms · {} · {}", s.wall_ms, s.lane, q));
        }
    }
    if !insights.candidates.is_empty() {
        lines.push("**Candidates**".into());
        for c in insights.candidates.iter().take(4) {
            lines.push(format!(
                "• [{}] {} ms — {} ({})",
                c.kind, c.wall_ms, c.reason, c.question_preview
            ));
        }
    }
    let mut out = lines.join("\n");
    if out.chars().count() > 1800 {
        out = out.chars().take(1790).collect::<String>() + "…";
    }
    out
}

#[derive(Debug, Clone)]
struct FailedRunLine {
    ts: String,
    lane: String,
    wall_ms: u64,
    question_preview: String,
    error: Option<String>,
    request_id: String,
}

#[derive(Debug, Clone)]
struct SlowRunLine {
    ts: String,
    lane: String,
    wall_ms: u64,
    question_preview: String,
    request_id: String,
    ok: bool,
}

/// Recent ok=false turns from runs.jsonl (newest first).
fn collect_failed_runs(limit: usize, days: Option<u32>) -> Vec<FailedRunLine> {
    let path = crate::commands::run_telemetry::runs_jsonl_path();
    if !path.is_file() {
        return Vec::new();
    }
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let window_days = days.map(|d| d.clamp(1, 90));
    let since = window_days.map(|d| chrono::Utc::now() - chrono::Duration::days(d as i64));
    let mut failed = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let v: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let Some(since) = since {
            let ts = v
                .get("ts")
                .and_then(|x| x.as_str())
                .and_then(parse_run_ts);
            match ts {
                Some(t) if t >= since => {}
                Some(_) => continue,
                None => continue,
            }
        }
        let ok = v.get("ok").and_then(|x| x.as_bool()).unwrap_or(true);
        if ok {
            continue;
        }
        failed.push(FailedRunLine {
            ts: v
                .get("ts")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            lane: v
                .get("lane")
                .and_then(|x| x.as_str())
                .unwrap_or("?")
                .to_string(),
            wall_ms: v.get("wall_ms").and_then(|x| x.as_u64()).unwrap_or(0),
            question_preview: v
                .get("question_preview")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            error: v
                .get("error")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string()),
            request_id: v
                .get("request_id")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
        });
    }
    if failed.len() > limit {
        failed = failed.split_off(failed.len() - limit);
    }
    failed.reverse();
    failed
}

/// Recent slow turns (wall_ms ≥ OPS_RUNS_SLOW_MS) from runs.jsonl (newest first).
fn collect_slow_runs(limit: usize, days: Option<u32>) -> Vec<SlowRunLine> {
    let path = crate::commands::run_telemetry::runs_jsonl_path();
    if !path.is_file() {
        return Vec::new();
    }
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let window_days = days.map(|d| d.clamp(1, 90));
    let since = window_days.map(|d| chrono::Utc::now() - chrono::Duration::days(d as i64));
    let mut slow = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let v: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let Some(since) = since {
            let ts = v
                .get("ts")
                .and_then(|x| x.as_str())
                .and_then(parse_run_ts);
            match ts {
                Some(t) if t >= since => {}
                Some(_) => continue,
                None => continue,
            }
        }
        let wall_ms = v.get("wall_ms").and_then(|x| x.as_u64()).unwrap_or(0);
        if wall_ms < OPS_RUNS_SLOW_MS {
            continue;
        }
        slow.push(SlowRunLine {
            ts: v
                .get("ts")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            lane: v
                .get("lane")
                .and_then(|x| x.as_str())
                .unwrap_or("?")
                .to_string(),
            wall_ms,
            question_preview: v
                .get("question_preview")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            request_id: v
                .get("request_id")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            ok: v.get("ok").and_then(|x| x.as_bool()).unwrap_or(true),
        });
    }
    if slow.len() > limit {
        slow = slow.split_off(slow.len() - limit);
    }
    slow.reverse();
    slow
}

/// Zero-LLM failed-turn report (Agent Ops Runs Fail filter parity).
pub fn format_failed_runs_gateway(days: Option<u32>) -> String {
    let failed = collect_failed_runs(12, days);
    let window = match days {
        Some(d) => format!("last **{d}** days"),
        None => "recent".into(),
    };
    if failed.is_empty() {
        return format!(
            "**Failed runs** ({window} · runs.jsonl)\nNo failed turns — all clear."
        );
    }
    let mut lines = vec![format!(
        "**Failed runs** ({window} · runs.jsonl) — **{}** shown",
        failed.len()
    )];
    for r in failed {
        let err = r
            .error
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or("(no error text)");
        let err_short: String = err.chars().take(100).collect();
        let ts_short: String = r.ts.chars().take(19).collect();
        let rid: String = r.request_id.chars().take(8).collect();
        lines.push(format!(
            "• **{ts_short}** · {} · {} ms · `{rid}`\n  {err_short}",
            r.lane, r.wall_ms
        ));
        if !r.question_preview.is_empty() {
            let q: String = r.question_preview.chars().take(72).collect();
            lines.push(format!("  _{q}_"));
        }
    }
    let mut out = lines.join("\n");
    if out.chars().count() > 1800 {
        out = out.chars().take(1790).collect::<String>() + "…";
    }
    out
}

/// Zero-LLM slow-turn report (Agent Ops Runs Slow filter parity).
pub fn format_slow_runs_gateway(days: Option<u32>) -> String {
    let slow = collect_slow_runs(12, days);
    let window = match days {
        Some(d) => format!("last **{d}** days"),
        None => "recent".into(),
    };
    if slow.is_empty() {
        return format!(
            "**Slow runs** ({window} · ≥{} ms · runs.jsonl)\nNo slow turns — all under threshold.",
            OPS_RUNS_SLOW_MS
        );
    }
    let mut lines = vec![format!(
        "**Slow runs** ({window} · ≥{} ms · runs.jsonl) — **{}** shown",
        OPS_RUNS_SLOW_MS,
        slow.len()
    )];
    for r in slow {
        let status = if r.ok { "ok" } else { "fail" };
        let ts_short: String = r.ts.chars().take(19).collect();
        let rid: String = r.request_id.chars().take(8).collect();
        lines.push(format!(
            "• **{ts_short}** · {} · {} ms · `{status}` · `{rid}`",
            r.lane, r.wall_ms
        ));
        if !r.question_preview.is_empty() {
            let q: String = r.question_preview.chars().take(72).collect();
            lines.push(format!("  _{q}_"));
        }
    }
    let mut out = lines.join("\n");
    if out.chars().count() > 1800 {
        out = out.chars().take(1790).collect::<String>() + "…";
    }
    out
}

/// Collect recent runs for a single lane from runs.jsonl (newest first).
fn collect_lane_runs(lane_want: &str, limit: usize, days: Option<u32>) -> Vec<SlowRunLine> {
    let path = crate::commands::run_telemetry::runs_jsonl_path();
    if !path.is_file() {
        return Vec::new();
    }
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let window_days = days.map(|d| d.clamp(1, 90));
    let since = window_days.map(|d| chrono::Utc::now() - chrono::Duration::days(d as i64));
    let mut rows = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let v: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let Some(since) = since {
            let ts = v
                .get("ts")
                .and_then(|x| x.as_str())
                .and_then(parse_run_ts);
            match ts {
                Some(t) if t >= since => {}
                Some(_) => continue,
                None => continue,
            }
        }
        let lane = v
            .get("lane")
            .and_then(|x| x.as_str())
            .unwrap_or("?")
            .to_lowercase();
        if lane != lane_want {
            continue;
        }
        rows.push(SlowRunLine {
            ts: v
                .get("ts")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            lane: v
                .get("lane")
                .and_then(|x| x.as_str())
                .unwrap_or("?")
                .to_string(),
            wall_ms: v.get("wall_ms").and_then(|x| x.as_u64()).unwrap_or(0),
            question_preview: v
                .get("question_preview")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            request_id: v
                .get("request_id")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            ok: v.get("ok").and_then(|x| x.as_bool()).unwrap_or(true),
        });
    }
    if rows.len() > limit {
        rows = rows.split_off(rows.len() - limit);
    }
    rows.reverse();
    rows
}

fn format_lane_runs_gateway(lane_label: &str, lane_want: &str, days: Option<u32>) -> String {
    let rows = collect_lane_runs(lane_want, 12, days);
    let window = match days {
        Some(d) => format!("last **{d}** days"),
        None => "recent".into(),
    };
    if rows.is_empty() {
        return format!(
            "**{lane_label} runs** ({window} · runs.jsonl)\nNo {lane_want}-lane turns yet."
        );
    }
    let mut lines = vec![format!(
        "**{lane_label} runs** ({window} · runs.jsonl) — **{}** shown",
        rows.len()
    )];
    for r in rows {
        let status = if r.ok { "ok" } else { "fail" };
        let ts_short: String = r.ts.chars().take(19).collect();
        let rid: String = r.request_id.chars().take(8).collect();
        lines.push(format!(
            "• **{ts_short}** · {} ms · `{status}` · `{rid}`",
            r.wall_ms
        ));
        if !r.question_preview.is_empty() {
            let q: String = r.question_preview.chars().take(72).collect();
            lines.push(format!("  _{q}_"));
        }
    }
    let mut out = lines.join("\n");
    if out.chars().count() > 1800 {
        out = out.chars().take(1790).collect::<String>() + "…";
    }
    out
}

/// Zero-LLM instant-lane report (Agent Ops Runs Instant filter parity).
pub fn format_instant_runs_gateway(days: Option<u32>) -> String {
    format_lane_runs_gateway("Instant", "instant", days)
}

/// Zero-LLM direct-lane report (Agent Ops Runs Direct filter parity).
pub fn format_direct_runs_gateway(days: Option<u32>) -> String {
    format_lane_runs_gateway("Direct", "direct", days)
}

/// Zero-LLM lite-lane report (Agent Ops Runs Lite filter parity).
pub fn format_lite_runs_gateway(days: Option<u32>) -> String {
    format_lane_runs_gateway("Lite", "lite", days)
}

/// True for `/slow` / `slow runs` — not "why is X slow" or monitor latency asks.
pub fn looks_like_slow_runs_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.contains(" ticket") || n.contains("redmine") || n.contains("http") {
        return false;
    }
    if n.contains("monitor") || n.contains("website") || n.contains("site ") {
        return false;
    }
    if n.contains(" why ") || n.contains("why is") || n.contains("why did") || n.contains("explain")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/slow"
            | "slow runs"
            | "slow run"
            | "slowest runs"
            | "what's slow"
            | "whats slow"
            | "what is slow"
            | "show slow runs"
            | "recent slow runs"
            | "slow turns"
            | "slow"
    ) || n.starts_with("/slow ")
        || (n.starts_with("slow runs ") && parse_insights_days(content).is_some())
}

/// True for `/instant` / `instant runs` — not creative "make it instant" asks.
pub fn looks_like_instant_runs_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.contains(" ticket") || n.contains("redmine") || n.contains("http") {
        return false;
    }
    if n.contains(" why ") || n.contains("why is") || n.contains("why did") || n.contains("explain")
    {
        return false;
    }
    if n.contains("make ") || n.contains("make it") || n.contains("instantly") {
        return false;
    }
    matches!(
        n.as_str(),
        "/instant"
            | "instant runs"
            | "instant run"
            | "instant lane"
            | "instant turns"
            | "show instant runs"
            | "recent instant runs"
            | "instant"
    ) || n.starts_with("/instant ")
        || (n.starts_with("instant runs ") && parse_insights_days(content).is_some())
}

/// True for `/direct` / `direct runs` — not free-form routing asks.
pub fn looks_like_direct_runs_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.contains(" ticket") || n.contains("redmine") || n.contains("http") {
        return false;
    }
    if n.contains(" why ") || n.contains("why is") || n.contains("why did") || n.contains("explain")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/direct"
            | "direct runs"
            | "direct run"
            | "direct lane"
            | "direct turns"
            | "show direct runs"
            | "recent direct runs"
            | "direct"
    ) || n.starts_with("/direct ")
        || (n.starts_with("direct runs ") && parse_insights_days(content).is_some())
}

/// True for `/lite` / `lite runs` — not free-form “make it lite” asks.
pub fn looks_like_lite_runs_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.contains(" ticket") || n.contains("redmine") || n.contains("http") {
        return false;
    }
    if n.contains(" why ") || n.contains("why is") || n.contains("why did") || n.contains("explain")
    {
        return false;
    }
    if n.contains("make ") || n.contains("make it") || n.contains("lightweight") {
        return false;
    }
    matches!(
        n.as_str(),
        "/lite"
            | "lite runs"
            | "lite run"
            | "lite lane"
            | "lite turns"
            | "show lite runs"
            | "recent lite runs"
            | "lite"
    ) || n.starts_with("/lite ")
        || (n.starts_with("lite runs ") && parse_insights_days(content).is_some())
}

/// True for `/failed` / `failed runs` — not "why did X fail".
pub fn looks_like_failed_runs_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.contains(" ticket") || n.contains("redmine") || n.contains("http") {
        return false;
    }
    if n.contains(" why ") || n.contains("why did") || n.contains("explain") {
        return false;
    }
    matches!(
        n.as_str(),
        "/failed"
            | "failed runs"
            | "failed run"
            | "fail runs"
            | "what failed"
            | "any failed"
            | "any failures"
            | "show failures"
            | "recent failures"
            | "failed turns"
            | "error runs"
            | "run errors"
            | "failures"
            | "failed"
            | "what went wrong tonight"
            | "what went wrong today"
    ) || n.starts_with("/failed ")
        || (n.starts_with("failed runs ") && parse_insights_days(content).is_some())
}

/// True for `/insights` / `insights` (Hermes parity) and short NL equivalents.
pub fn looks_like_insights_request(content: &str) -> bool {
    let n = content
        .trim()
        .trim_start_matches('@')
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    let n = n
        .trim_start_matches("werner")
        .trim_start_matches(',')
        .trim()
        .trim_start_matches("please")
        .trim()
        .trim_start_matches("can you")
        .trim()
        .trim_start_matches("could you")
        .trim()
        .trim_start_matches("show me")
        .trim()
        .trim_start_matches("show")
        .trim()
        .trim_end_matches('?')
        .trim();
    // Topic research ("insights on weather") stays with the agent.
    if n.contains(" on ") || n.contains(" about ") || n.contains("weather") || n.contains("http")
    {
        return false;
    }
    matches!(
        n,
        "insights"
            | "/insights"
            | "usage insights"
            | "run insights"
            | "agent insights"
            | "usage analytics"
            | "usage stats"
            | "run stats"
            | "runs report"
            | "latency report"
            | "p50"
            | "p50 report"
    ) || n.starts_with("/insights ")
        || (n.starts_with("insights ") && parse_insights_days(content).is_some())
        || (n.starts_with("usage insights ") && parse_insights_days(content).is_some())
        || (n.starts_with("usage analytics ") && parse_insights_days(content).is_some())
}

fn sanitize_under_dir(path: &str, root: &Path) -> Result<PathBuf, String> {
    let root = root
        .canonicalize()
        .unwrap_or_else(|_| root.to_path_buf());
    let p = PathBuf::from(path);
    let canon = p
        .canonicalize()
        .map_err(|e| format!("Invalid path: {}", e))?;
    if !canon.starts_with(&root) {
        return Err("Path escapes allowed directory".into());
    }
    Ok(canon)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_session_name() {
        let (src, slug) =
            parse_session_filename("session-memory-discord-20260720-181500-weather.md");
        assert_eq!(src, "discord");
        assert!(slug.contains("weather"));
    }

    #[test]
    fn operator_instant_reply_covers_gateway_commands() {
        let status = try_operator_instant_reply("/status").expect("status");
        assert!(status.contains("mac-stats"));
        let insights = try_operator_instant_reply("insights").expect("insights");
        assert!(insights.to_lowercase().contains("insights"));
        let failed = try_operator_instant_reply("/failed").expect("failed");
        assert!(failed.to_lowercase().contains("failed runs"));
        let slow = try_operator_instant_reply("/slow").expect("slow");
        assert!(slow.to_lowercase().contains("slow runs"));
        let instant = try_operator_instant_reply("/instant").expect("instant");
        assert!(instant.to_lowercase().contains("instant runs"));
        let direct = try_operator_instant_reply("/direct").expect("direct");
        assert!(direct.to_lowercase().contains("direct runs"));
        let lite = try_operator_instant_reply("/lite").expect("lite");
        assert!(lite.to_lowercase().contains("lite runs"));
        let agents = try_operator_instant_reply("/agents").expect("agents");
        assert!(agents.to_lowercase().contains("agents"));
        let sessions = try_operator_instant_reply("/sessions").expect("sessions");
        assert!(sessions.to_lowercase().contains("sessions"));
        let knowledge = try_operator_instant_reply("/knowledge").expect("knowledge");
        assert!(knowledge.to_lowercase().contains("knowledge"));
        let schedules = try_operator_instant_reply("list schedules").expect("schedules");
        assert!(schedules.to_lowercase().contains("schedule"));
        let schedules_jobs = try_operator_instant_reply("/schedules jobs").expect("schedules jobs");
        assert!(schedules_jobs.to_lowercase().contains("jobs"));
        let schedules_del =
            try_operator_instant_reply("/schedules deliveries").expect("schedules deliveries");
        assert!(schedules_del.to_lowercase().contains("deliver"));
        let monitors = try_operator_instant_reply("/monitors").expect("monitors");
        assert!(monitors.to_lowercase().contains("monitor"));
        let monitors_down = try_operator_instant_reply("/monitors down").expect("monitors down");
        assert!(monitors_down.to_lowercase().contains("down"));
        assert!(try_operator_instant_reply("status of the redmine ticket").is_none());
        assert!(try_operator_instant_reply("insights on weather").is_none());
        assert!(try_operator_instant_reply("why did the build fail").is_none());
        assert!(try_operator_instant_reply("why is the site slow").is_none());
        assert!(try_operator_instant_reply("make it instant").is_none());
        assert!(try_operator_instant_reply("make it lite").is_none());
        assert!(try_operator_instant_reply("create an agent for weather").is_none());
        assert!(try_operator_instant_reply("resume this session").is_none());
        assert!(try_operator_instant_reply("scrub memory").is_some());
        assert!(try_operator_instant_reply("save this to knowledge").is_none());
        assert!(try_operator_instant_reply("add a monitor for example.com").is_none());
    }

    #[test]
    fn agents_request_detected() {
        assert!(looks_like_agents_request("/agents"));
        assert!(looks_like_agents_request("list agents"));
        assert!(looks_like_agents_request("agents on"));
        assert!(looks_like_agents_request("/agents off"));
        assert!(looks_like_agents_request("enabled agents"));
        assert!(looks_like_agents_request("@Werner agents"));
        assert!(!looks_like_agents_request("create an agent"));
        assert!(!looks_like_agents_request("agent: research weather"));
        assert!(!looks_like_agents_request("why are agents offline"));
        assert_eq!(parse_agents_list_filter("/agents"), AgentsListFilter::All);
        assert_eq!(parse_agents_list_filter("/agents on"), AgentsListFilter::On);
        assert_eq!(
            parse_agents_list_filter("disabled agents"),
            AgentsListFilter::Off
        );
    }

    #[test]
    fn sessions_request_detected() {
        assert!(looks_like_sessions_request("/sessions"));
        assert!(looks_like_sessions_request("list sessions"));
        assert!(looks_like_sessions_request("live sessions"));
        assert!(looks_like_sessions_request("/sessions files"));
        assert!(looks_like_sessions_request("saved sessions"));
        assert!(looks_like_sessions_request("@Werner sessions"));
        assert!(!looks_like_sessions_request("resume this session"));
        assert!(!looks_like_sessions_request("delete session files"));
        assert!(!looks_like_sessions_request("why are sessions empty"));
        assert_eq!(
            parse_sessions_list_filter("/sessions"),
            SessionsListFilter::All
        );
        assert_eq!(
            parse_sessions_list_filter("/sessions live"),
            SessionsListFilter::Live
        );
        assert_eq!(
            parse_sessions_list_filter("session files"),
            SessionsListFilter::Files
        );
    }

    #[test]
    fn knowledge_request_detected() {
        assert!(looks_like_knowledge_request("/knowledge"));
        assert!(looks_like_knowledge_request("list knowledge"));
        assert!(looks_like_knowledge_request("knowledge files"));
        assert!(looks_like_knowledge_request("/knowledge discord"));
        assert!(looks_like_knowledge_request("core knowledge"));
        assert!(looks_like_knowledge_request("@Werner knowledge"));
        assert!(!looks_like_knowledge_request("scrub memory"));
        assert!(!looks_like_knowledge_request("save this to knowledge"));
        assert!(!looks_like_knowledge_request("why is knowledge empty"));
        assert_eq!(
            parse_knowledge_list_filter("/knowledge"),
            KnowledgeListFilter::All
        );
        assert_eq!(
            parse_knowledge_list_filter("/knowledge discord"),
            KnowledgeListFilter::Discord
        );
        assert_eq!(
            parse_knowledge_list_filter("core knowledge"),
            KnowledgeListFilter::Core
        );
    }

    #[test]
    fn instant_runs_request_detected() {
        assert!(looks_like_instant_runs_request("/instant"));
        assert!(looks_like_instant_runs_request("instant runs"));
        assert!(looks_like_instant_runs_request("instant lane"));
        assert!(looks_like_instant_runs_request("@Werner instant runs 3"));
        assert!(looks_like_instant_runs_request("/instant 7"));
        assert!(!looks_like_instant_runs_request("make it instant"));
        assert!(!looks_like_instant_runs_request("why is instant lane broken"));
    }

    #[test]
    fn direct_runs_request_detected() {
        assert!(looks_like_direct_runs_request("/direct"));
        assert!(looks_like_direct_runs_request("direct runs"));
        assert!(looks_like_direct_runs_request("direct lane"));
        assert!(looks_like_direct_runs_request("@Werner direct runs 3"));
        assert!(looks_like_direct_runs_request("/direct 7"));
        assert!(!looks_like_direct_runs_request("why did direct lane fail"));
    }

    #[test]
    fn lite_runs_request_detected() {
        assert!(looks_like_lite_runs_request("/lite"));
        assert!(looks_like_lite_runs_request("lite runs"));
        assert!(looks_like_lite_runs_request("lite lane"));
        assert!(looks_like_lite_runs_request("@Werner lite runs 3"));
        assert!(looks_like_lite_runs_request("/lite 7"));
        assert!(!looks_like_lite_runs_request("make it lite"));
        assert!(!looks_like_lite_runs_request("why is lite lane broken"));
    }

    #[test]
    fn slow_runs_request_detected() {
        assert!(looks_like_slow_runs_request("/slow"));
        assert!(looks_like_slow_runs_request("what's slow"));
        assert!(looks_like_slow_runs_request("slow runs"));
        assert!(looks_like_slow_runs_request("@Werner slow runs 3"));
        assert!(looks_like_slow_runs_request("/slow 7"));
        assert!(!looks_like_slow_runs_request("why is the build slow"));
        assert!(!looks_like_slow_runs_request("slow monitor for example.com"));
    }

    #[test]
    fn failed_runs_request_detected() {
        assert!(looks_like_failed_runs_request("/failed"));
        assert!(looks_like_failed_runs_request("what failed"));
        assert!(looks_like_failed_runs_request("failed runs"));
        assert!(looks_like_failed_runs_request("@Werner failed runs 3"));
        assert!(looks_like_failed_runs_request("/failed 7"));
        assert!(!looks_like_failed_runs_request("why did the deploy fail"));
        assert!(!looks_like_failed_runs_request("explain the failed ticket"));
    }

    #[test]
    fn insights_request_detected() {
        assert!(looks_like_insights_request("/insights"));
        assert!(looks_like_insights_request("insights"));
        assert!(looks_like_insights_request("@Werner insights"));
        assert!(looks_like_insights_request("/insights 7"));
        assert!(looks_like_insights_request("/insights --days 14"));
        assert!(looks_like_insights_request("insights 3"));
        assert!(looks_like_insights_request("show me usage analytics"));
        assert!(looks_like_insights_request("usage stats"));
        assert!(looks_like_insights_request("latency report"));
        assert!(!looks_like_insights_request("any insights on weather?"));
        assert!(!looks_like_insights_request("insights on weather"));
        assert!(!looks_like_insights_request("show insights about Barcelona"));
    }

    #[test]
    fn parse_insights_days_hermes_args() {
        assert_eq!(parse_insights_days("/insights"), None);
        assert_eq!(parse_insights_days("/insights 7"), Some(7));
        assert_eq!(parse_insights_days("/insights --days 14"), Some(14));
        assert_eq!(parse_insights_days("@Werner insights 3"), Some(3));
        assert_eq!(parse_insights_days("/insights 999"), Some(90)); // clamp
        assert_eq!(parse_insights_days("/failed 7"), Some(7));
        assert_eq!(parse_insights_days("/slow 3"), Some(3));
        assert_eq!(parse_insights_days("/instant 7"), Some(7));
        assert_eq!(parse_insights_days("/direct 3"), Some(3));
        assert_eq!(parse_insights_days("/lite 7"), Some(7));
    }

    #[test]
    fn insights_slowest_noise_filters_shipped_patterns() {
        assert!(is_insights_slowest_noise("instant", 400, &[], "ok"));
        assert!(is_insights_slowest_noise(
            "direct",
            17_000,
            &["BRAVE_SEARCH".into()],
            "What´s the wether like in El Masnou right now?"
        ));
        assert!(is_insights_slowest_noise(
            "direct",
            16_000,
            &[],
            "Please cross check if you are ok talking on amvara discord server"
        ));
        assert!(is_insights_slowest_noise(
            "lite",
            44_000,
            &["LIST_SCHEDULES".into(), "TASK_LIST".into()],
            "No improvement loop?"
        ));
        assert!(is_insights_slowest_noise(
            "direct",
            10_000,
            &["TASK_LIST".into()],
            "What's planned for this night?"
        ));
        assert!(is_insights_slowest_noise(
            "direct",
            21_000,
            &["BRAVE_SEARCH".into()],
            "What is the climate today in L Masnou?"
        ));
        assert!(is_insights_slowest_noise(
            "lite",
            8_000,
            &[],
            "ke klima en elmasnau eu"
        ));
        assert!(is_insights_slowest_noise(
            "direct",
            28_000,
            &["SKILL".into()],
            "SKILL: ui-weekly-review — Weekly Agent Ops polish per docs/041_ui_command_center"
        ));
        assert!(is_insights_slowest_noise(
            "instant",
            50,
            &[],
            "/monitors down"
        ));
        assert!(is_insights_slowest_noise(
            "instant",
            80,
            &[],
            "list monitors"
        ));
        assert!(!is_insights_slowest_noise(
            "direct",
            12_000,
            &["REDMINE_API".into()],
            "Review and summarize Redmine ticket: 7736"
        ));
    }

    #[test]
    fn insights_gateway_includes_digest_and_schedules() {
        let insights = RunsInsights {
            turns: 0,
            ok_count: 0,
            fail_count: 0,
            p50_ms: 0,
            mean_ms: 0,
            max_ms: 0,
            latency_sample: 0,
            by_lane: vec![],
            by_tool: vec![],
            candidates: vec![],
            slowest: vec![],
            recent: vec![],
            discord_gateway: String::new(),
            digest_open_count: 0,
            digest_stale_count: 3,
            digest_generated_at: "2026-07-21T05:00:00Z".into(),
            digest_open_hints: vec![],
            digest_source: "python".into(),
            process_uptime_secs: 0,
            window_days: Some(7),
        };
        let report = format_runs_insights_gateway(&insights);
        assert!(report.contains("Digest:"), "{report}");
        assert!(report.contains("Schedules:"), "{report}");
        assert!(report.contains("Discord gateway:"), "{report}");
        assert!(report.contains("last **7** days"), "{report}");
    }

    #[test]
    fn digest_request_detected() {
        assert!(looks_like_digest_request("/digest"));
        assert!(looks_like_digest_request("refresh digest"));
        assert!(looks_like_digest_request("run digester"));
        assert!(looks_like_digest_request("show me digest"));
        assert!(!looks_like_digest_request("digest this long research report please"));
    }

    #[test]
    fn schedules_request_detected() {
        assert!(looks_like_schedules_request("/schedules"));
        assert!(looks_like_schedules_request("/cron list"));
        assert!(looks_like_schedules_request("list schedules"));
        assert!(looks_like_schedules_request("@Werner schedules"));
        assert!(looks_like_schedules_request("upcoming jobs"));
        assert!(looks_like_schedules_request("my cron jobs"));
        assert!(looks_like_schedules_request("/schedules jobs"));
        assert!(looks_like_schedules_request("/schedules deliveries"));
        assert!(looks_like_schedules_request("recent deliveries"));
        assert!(looks_like_schedules_request("list deliveries"));
        assert!(!looks_like_schedules_request("schedule a task for tomorrow"));
        assert!(!looks_like_schedules_request("why are schedules empty"));
        assert_eq!(
            parse_schedules_list_filter("/schedules"),
            SchedulesListFilter::All
        );
        assert_eq!(
            parse_schedules_list_filter("/schedules jobs"),
            SchedulesListFilter::Jobs
        );
        assert_eq!(
            parse_schedules_list_filter("upcoming jobs"),
            SchedulesListFilter::Jobs
        );
        assert_eq!(
            parse_schedules_list_filter("/schedules deliveries"),
            SchedulesListFilter::Deliveries
        );
        assert_eq!(
            parse_schedules_list_filter("recent deliveries"),
            SchedulesListFilter::Deliveries
        );
    }

    #[test]
    fn monitors_request_detected() {
        assert!(looks_like_monitors_request("/monitors"));
        assert!(looks_like_monitors_request("list monitors"));
        assert!(looks_like_monitors_request("@Werner monitors"));
        assert!(looks_like_monitors_request("/monitors up"));
        assert!(looks_like_monitors_request("/monitors down"));
        assert!(looks_like_monitors_request("/monitors slow"));
        assert!(looks_like_monitors_request("down monitors"));
        assert!(looks_like_monitors_request("slow monitors"));
        assert!(looks_like_monitors_request("which sites are down"));
        assert!(looks_like_monitors_request("what's down"));
        assert!(looks_like_monitors_request("monitor list"));
        assert!(!looks_like_monitors_request("add a monitor"));
        assert!(!looks_like_monitors_request("why are monitors down"));
        assert!(!looks_like_monitors_request("check monitor now"));
        assert_eq!(
            parse_monitors_list_filter("/monitors"),
            MonitorsListFilter::All
        );
        assert_eq!(
            parse_monitors_list_filter("/monitors up"),
            MonitorsListFilter::Up
        );
        assert_eq!(
            parse_monitors_list_filter("down monitors"),
            MonitorsListFilter::Down
        );
        assert_eq!(
            parse_monitors_list_filter("slow monitors"),
            MonitorsListFilter::Slow
        );
    }

    #[test]
    fn memory_scrub_request_detected() {
        assert!(looks_like_memory_scrub_request("scrub memory"));
        assert!(looks_like_memory_scrub_request("clean up memory"));
        assert!(looks_like_memory_scrub_request("purge memory"));
        assert!(looks_like_memory_scrub_request("@Werner please scrub memory"));
        assert!(!looks_like_memory_scrub_request(
            "scrub memory and then rewrite my soul.md with a full biography"
        ));
    }

    #[test]
    fn status_request_detected() {
        assert!(looks_like_status_request("/status"));
        assert!(looks_like_status_request("/health"));
        assert!(looks_like_status_request("/version"));
        assert!(looks_like_status_request("bot status"));
        assert!(looks_like_status_request("system health"));
        assert!(looks_like_status_request("what version"));
        assert!(!looks_like_status_request("status of the redmine ticket"));
        assert!(looks_like_status_request("is everything ok"));
        assert!(looks_like_status_request("everything working"));
    }

    #[test]
    fn digest_open_candidates_requests() {
        assert!(looks_like_digest_request("digest open"));
        assert!(looks_like_digest_request("open candidates"));
        assert!(looks_like_digest_request("any open candidates"));
        assert!(!looks_like_digest_request("digest this long research report please"));
    }

    #[test]
    fn ops_help_request_detected() {
        assert!(looks_like_ops_help_request("/ops"));
        assert!(looks_like_ops_help_request("ops"));
        assert!(looks_like_ops_help_request("operator commands"));
        assert!(looks_like_ops_help_request("@Werner /ops"));
        assert!(looks_like_ops_help_request("/help"));
        assert!(looks_like_ops_help_request("help"));
        assert!(looks_like_ops_help_request("what can you do"));
        assert!(looks_like_ops_help_request("command list"));
        assert!(!looks_like_ops_help_request("help me write a cron"));
        assert!(!looks_like_ops_help_request("help with weather"));
    }

    #[test]
    fn ops_help_lists_status() {
        let report = format_ops_help_gateway();
        assert!(report.contains("/status"), "{report}");
        assert!(report.contains("/schedules"), "{report}");
        assert!(report.contains("/schedules jobs"), "{report}");
        assert!(report.contains("/schedules deliveries"), "{report}");
        assert!(report.contains("/digest"), "{report}");
        assert!(report.contains("/slow"), "{report}");
        assert!(report.contains("/instant"), "{report}");
        assert!(report.contains("/lite"), "{report}");
        assert!(report.contains("/direct"), "{report}");
        assert!(report.contains("/agents"), "{report}");
        assert!(report.contains("/sessions"), "{report}");
        assert!(report.contains("/knowledge"), "{report}");
        assert!(report.contains("/monitors"), "{report}");
        assert!(report.contains("/monitors down"), "{report}");
        assert!(report.contains("/help"), "{report}");
        assert!(report.contains("Voice"), "{report}");
    }

    #[test]
    fn agents_gateway_has_counts() {
        let report = format_agents_gateway(AgentsListFilter::All);
        assert!(report.to_lowercase().contains("agents"), "{report}");
        assert!(report.contains("on"), "{report}");
    }

    #[test]
    fn sessions_gateway_has_counts() {
        let report = format_sessions_gateway(SessionsListFilter::All);
        assert!(report.to_lowercase().contains("sessions"), "{report}");
        assert!(
            report.to_lowercase().contains("live") || report.to_lowercase().contains("files"),
            "{report}"
        );
    }

    #[test]
    fn knowledge_gateway_has_counts() {
        let report = format_knowledge_gateway(KnowledgeListFilter::All);
        assert!(report.to_lowercase().contains("knowledge"), "{report}");
        assert!(
            report.to_lowercase().contains("discord") || report.to_lowercase().contains("core"),
            "{report}"
        );
    }

    #[test]
    fn schedules_gateway_has_counts() {
        let report = format_schedules_gateway(SchedulesListFilter::All);
        assert!(report.to_lowercase().contains("schedules"), "{report}");
        assert!(
            report.to_lowercase().contains("jobs") || report.to_lowercase().contains("deliver"),
            "{report}"
        );
        let jobs = format_schedules_gateway(SchedulesListFilter::Jobs);
        assert!(jobs.to_lowercase().contains("jobs"), "{jobs}");
        let dels = format_schedules_gateway(SchedulesListFilter::Deliveries);
        assert!(dels.to_lowercase().contains("deliver"), "{dels}");
    }

    #[test]
    fn monitors_gateway_has_counts() {
        let report = format_monitors_gateway(MonitorsListFilter::All);
        assert!(report.to_lowercase().contains("monitor"), "{report}");
        let down = format_monitors_gateway(MonitorsListFilter::Down);
        assert!(down.to_lowercase().contains("down"), "{down}");
        let slow = format_monitors_gateway(MonitorsListFilter::Slow);
        assert!(slow.to_lowercase().contains("slow"), "{slow}");
    }

    #[test]
    fn status_gateway_mentions_version() {
        let report = format_status_gateway();
        assert!(report.contains("mac-stats v"), "{report}");
        assert!(report.contains("Digest:"), "{report}");
        assert!(report.contains("Schedules:"), "{report}");
    }

    #[test]
    fn rust_native_digest_writes_json() {
        let summary = write_digest_native(7).expect("native digest");
        assert!(digest_json_path().is_file());
        let loaded = load_digest_summary();
        assert_eq!(loaded.source, "rust-native");
        let _ = summary.open_count + summary.stale_count + summary.turns;
    }

    #[test]
    fn write_text_atomic_roundtrip() {
        let dir = std::env::temp_dir().join(format!(
            "mac-stats-atomic-digest-{}",
            std::process::id()
        ));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("latest.json");
        write_text_atomic(&path, "{\"ok\":true}\n").expect("atomic write");
        assert_eq!(fs::read_to_string(&path).unwrap(), "{\"ok\":true}\n");
        write_text_atomic(&path, "{\"ok\":false}\n").expect("overwrite");
        assert_eq!(fs::read_to_string(&path).unwrap(), "{\"ok\":false}\n");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn version_candidate_classified() {
        let c = classify_candidate(
            "lite",
            26_000,
            &[],
            "What version are you?",
            "abc",
        );
        assert!(c.is_some());
        assert_eq!(c.unwrap().kind, "promote_instant");
    }

    #[test]
    fn identity_affirmation_candidate_classified() {
        let c = classify_candidate(
            "direct",
            4_200,
            &[],
            "You are working for Amvara. You are online in Amvara server on various channel.",
            "id-aff",
        );
        assert!(c.is_some());
        let c = c.unwrap();
        assert_eq!(c.kind, "promote_instant");
        assert!(c.reason.contains("Identity"));
    }

    #[test]
    fn capabilities_candidate_classified() {
        let c = classify_candidate("direct", 9_000, &[], "What can you do?", "cap-1");
        assert!(c.is_some());
        let c = c.unwrap();
        assert_eq!(c.kind, "promote_instant");
        assert!(c.reason.contains("Capabilities"));
    }

    #[test]
    fn expanded_presence_candidate_classified() {
        let c = classify_candidate("direct", 3_000, &[], "Need anything else?", "pres-1");
        assert!(c.is_some());
        assert_eq!(c.unwrap().kind, "promote_instant");
    }
}
