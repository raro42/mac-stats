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
        let id = r.id.as_str();
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
                format!("• ✅ `{id}` · {label} · {ms}{slow_mark} · {age}")
            }
            Some(false) => {
                let reason = r
                    .error
                    .as_deref()
                    .map(|e| truncate_preview(e, 40))
                    .filter(|e| !e.is_empty())
                    .unwrap_or_else(|| "DOWN".into());
                format!("• ❌ `{id}` · {label} · {reason} · {age}")
            }
            None => format!("• ⏳ `{id}` · {label} · waiting · {age}"),
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

/// Disk Cleanup scopes/categories filter for `/disk` instant replies (UI parity).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskCleanupListFilter {
    All,
    On,
    Off,
    Reclaim,
    Big,
    Clean,
}

/// Big reclaimable threshold — matches UI `DISK_CLEANUP_BIG_BYTES` (50 MiB).
pub const OPS_DISK_CLEANUP_BIG_BYTES: u64 = 50 * 1024 * 1024;

/// Parse On/Off/Reclaim/Big/Clean from `/disk reclaim`, `cleanup scopes off`, etc. Default All.
pub fn parse_disk_cleanup_list_filter(content: &str) -> DiskCleanupListFilter {
    let n = normalize_operator_command(content);
    if n.ends_with(" off")
        || n == "off"
        || n == "disk off"
        || n == "/disk off"
        || n == "cleanup off"
        || n == "/cleanup off"
        || n == "disabled scopes"
        || n == "scopes off"
        || n == "cleanup scopes off"
        || n == "off scopes"
    {
        return DiskCleanupListFilter::Off;
    }
    if n.ends_with(" on")
        || n == "on"
        || n == "disk on"
        || n == "/disk on"
        || n == "cleanup on"
        || n == "/cleanup on"
        || n == "enabled scopes"
        || n == "scopes on"
        || n == "cleanup scopes on"
        || n == "on scopes"
    {
        return DiskCleanupListFilter::On;
    }
    if n.ends_with(" reclaim")
        || n == "reclaim"
        || n == "disk reclaim"
        || n == "/disk reclaim"
        || n == "cleanup reclaim"
        || n == "/cleanup reclaim"
        || n == "reclaimable"
        || n == "what's reclaimable"
        || n == "whats reclaimable"
        || n == "what is reclaimable"
        || n == "reclaimable space"
        || n == "reclaimable categories"
    {
        return DiskCleanupListFilter::Reclaim;
    }
    if n.ends_with(" big")
        || n == "big"
        || n == "disk big"
        || n == "/disk big"
        || n == "cleanup big"
        || n == "/cleanup big"
        || n == "big reclaim"
        || n == "big categories"
    {
        return DiskCleanupListFilter::Big;
    }
    if n.ends_with(" clean")
        || n == "clean"
        || n == "disk clean"
        || n == "/disk clean"
        || n == "cleanup clean"
        || n == "/cleanup clean"
        || n == "clean categories"
        || n == "already clean"
    {
        return DiskCleanupListFilter::Clean;
    }
    DiskCleanupListFilter::All
}

/// True for `/disk` / `disk cleanup` — Disk Cleanup filter parity; not clean-now / SSD asks.
pub fn looks_like_disk_cleanup_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    if n.contains("clean now")
        || n.contains("run cleanup")
        || n.contains("run disk")
        || n.contains("delete")
        || n.contains("remove")
        || n.contains("empty trash")
        || n.contains("ssd")
        || n.contains("disk usage")
        || n.contains("disk free")
        || n.contains("free space")
        || n.contains("free disk")
        || n.contains("how full")
        || n.contains("percent")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains("why")
        || n.contains("ticket")
        || n.contains("redmine")
        || n.contains("add ")
        || n.contains("create")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/disk"
            | "/cleanup"
            | "cleanup"
            | "disk cleanup"
            | "cleanup status"
            | "disk cleanup status"
            | "list cleanup"
            | "show cleanup"
            | "cleanup scopes"
            | "disk scopes"
            | "/disk on"
            | "disk on"
            | "/cleanup on"
            | "cleanup on"
            | "enabled scopes"
            | "scopes on"
            | "cleanup scopes on"
            | "on scopes"
            | "/disk off"
            | "disk off"
            | "/cleanup off"
            | "cleanup off"
            | "disabled scopes"
            | "scopes off"
            | "cleanup scopes off"
            | "off scopes"
            | "/disk reclaim"
            | "disk reclaim"
            | "/cleanup reclaim"
            | "cleanup reclaim"
            | "reclaim"
            | "reclaimable"
            | "what's reclaimable"
            | "whats reclaimable"
            | "what is reclaimable"
            | "reclaimable space"
            | "reclaimable categories"
            | "/disk big"
            | "disk big"
            | "/cleanup big"
            | "cleanup big"
            | "big reclaim"
            | "big categories"
            | "/disk clean"
            | "disk clean"
            | "/cleanup clean"
            | "cleanup clean"
            | "clean categories"
            | "already clean"
    )
}

/// Zero-LLM Disk Cleanup report (scopes On/Off · categories Reclaim/Big/Clean; shallow scan).
pub fn format_disk_cleanup_gateway(filter: DiskCleanupListFilter) -> String {
    // Shallow preview — no Downloads/Trash (TCC); same as auto-run path.
    let status = crate::commands::disk_cleanup::get_status(false);
    let fmt = crate::commands::disk_cleanup::format_bytes;
    let on_n = status.scopes.iter().filter(|s| s.enabled).count();
    let off_n = status.scopes.len().saturating_sub(on_n);
    let reclaim_n = status
        .categories
        .iter()
        .filter(|c| c.enabled && c.bytes > 0)
        .count();
    let big_n = status
        .categories
        .iter()
        .filter(|c| c.enabled && c.bytes >= OPS_DISK_CLEANUP_BIG_BYTES)
        .count();
    let clean_n = status
        .categories
        .iter()
        .filter(|c| c.enabled && c.bytes == 0)
        .count();
    let reclaim_label = fmt(status.reclaimable_bytes);
    let title = match filter {
        DiskCleanupListFilter::All => format!(
            "**Disk Cleanup** — {reclaim_label} reclaimable · {reclaim_n} reclaim · {big_n} big · scopes {on_n} on · {off_n} off"
        ),
        DiskCleanupListFilter::On => format!("**Disk Cleanup · On** — {on_n} scopes"),
        DiskCleanupListFilter::Off => format!("**Disk Cleanup · Off** — {off_n} scopes"),
        DiskCleanupListFilter::Reclaim => {
            format!("**Disk Cleanup · Reclaim** — {reclaim_n} · {reclaim_label}")
        }
        DiskCleanupListFilter::Big => format!(
            "**Disk Cleanup · Big** — {big_n} (≥{})",
            fmt(OPS_DISK_CLEANUP_BIG_BYTES)
        ),
        DiskCleanupListFilter::Clean => {
            format!("**Disk Cleanup · Clean** — {clean_n} already clean")
        }
    };
    let mut lines = vec![title];
    if filter == DiskCleanupListFilter::All {
        lines.push(format!("Next · {}", status.next_run_label));
        if !status.enabled_scope_summary.is_empty() {
            lines.push(format!("Scopes · {}", status.enabled_scope_summary));
        }
    }

    const MAX_ROWS: usize = 12;

    match filter {
        DiskCleanupListFilter::On | DiskCleanupListFilter::Off => {
            let want_on = filter == DiskCleanupListFilter::On;
            let filtered: Vec<_> = status
                .scopes
                .iter()
                .filter(|s| s.enabled == want_on)
                .collect();
            if filtered.is_empty() {
                lines.push(if want_on {
                    "_No scopes enabled — turn one on under Disk Cleanup._".into()
                } else {
                    "_Every scope is on right now._".into()
                });
            } else {
                for s in filtered.iter().take(MAX_ROWS) {
                    let kind = s.kind.as_str();
                    let path = s
                        .path
                        .as_deref()
                        .map(|p| truncate_preview(p, 36))
                        .filter(|p| !p.is_empty());
                    let mark = if s.enabled { "on" } else { "off" };
                    let extra = path
                        .map(|p| format!(" · `{p}`"))
                        .unwrap_or_default();
                    lines.push(format!(
                        "• `{id}` · {label} · {kind} · {mark}{extra}",
                        id = s.id,
                        label = s.label
                    ));
                }
                if filtered.len() > MAX_ROWS {
                    lines.push(format!("_…+{} more_", filtered.len() - MAX_ROWS));
                }
            }
        }
        DiskCleanupListFilter::All
        | DiskCleanupListFilter::Reclaim
        | DiskCleanupListFilter::Big
        | DiskCleanupListFilter::Clean => {
            let filtered: Vec<_> = status
                .categories
                .iter()
                .filter(|c| c.enabled)
                .filter(|c| match filter {
                    DiskCleanupListFilter::All => true,
                    DiskCleanupListFilter::Reclaim => c.bytes > 0,
                    DiskCleanupListFilter::Big => c.bytes >= OPS_DISK_CLEANUP_BIG_BYTES,
                    DiskCleanupListFilter::Clean => c.bytes == 0,
                    _ => true,
                })
                .collect();
            // Reclaim/Big: largest first; Clean/All: reclaim first then by bytes.
            let mut sorted = filtered;
            sorted.sort_by(|a, b| b.bytes.cmp(&a.bytes).then_with(|| a.label.cmp(&b.label)));
            if sorted.is_empty() {
                let empty = match filter {
                    DiskCleanupListFilter::All => {
                        "_No cleanup categories yet — open Disk Cleanup in the CPU window._"
                    }
                    DiskCleanupListFilter::Reclaim => "_Nothing reclaimable right now._",
                    DiskCleanupListFilter::Big => "_No big reclaimable categories (≥50 MB)._",
                    DiskCleanupListFilter::Clean => "_No clean categories right now._",
                    _ => "_Nothing here._",
                };
                lines.push(empty.into());
            } else {
                for c in sorted.iter().take(MAX_ROWS) {
                    let size = fmt(c.bytes);
                    let hint = truncate_preview(&c.path_hint, 40);
                    let mark = if c.bytes >= OPS_DISK_CLEANUP_BIG_BYTES {
                        " · big"
                    } else if c.bytes > 0 {
                        " · reclaim"
                    } else {
                        " · clean"
                    };
                    lines.push(format!(
                        "• `{id}` · {label} · {size} · {files} files{mark} · `{hint}`",
                        id = c.id,
                        label = c.label,
                        files = c.file_count
                    ));
                }
                if sorted.len() > MAX_ROWS {
                    lines.push(format!("_…+{} more_", sorted.len() - MAX_ROWS));
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

/// Debug Log All · Error · Warn filter for `/logs` instant replies (UI parity).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugLogListFilter {
    All,
    Error,
    Warn,
}

/// Classify a debug.log line — matches `logsLineKind` in cpu.js.
pub fn debug_log_line_kind(line: &str) -> &'static str {
    let lower = line.to_ascii_lowercase();
    if lower.contains(" error ")
        || lower.contains("error:")
        || lower.contains("panic")
    {
        "error"
    } else if lower.contains(" warn ") || lower.contains("warn:") {
        "warn"
    } else {
        "other"
    }
}

/// Parse Error/Warn from `/logs error`, `show warnings`, etc. Default All.
pub fn parse_debug_log_list_filter(content: &str) -> DebugLogListFilter {
    let n = normalize_operator_command(content);
    if n.ends_with(" error")
        || n.ends_with(" errors")
        || n == "error"
        || n == "errors"
        || n == "logs error"
        || n == "/logs error"
        || n == "log error"
        || n == "log errors"
        || n == "debug error"
        || n == "debug errors"
        || n == "error log"
        || n == "error logs"
        || n == "any errors"
        || n == "any error"
        || n == "show errors"
        || n == "list errors"
        || n == "what's wrong"
        || n == "whats wrong"
        || n == "what is wrong"
    {
        return DebugLogListFilter::Error;
    }
    if n.ends_with(" warn")
        || n.ends_with(" warns")
        || n.ends_with(" warning")
        || n.ends_with(" warnings")
        || n == "warn"
        || n == "warns"
        || n == "warning"
        || n == "warnings"
        || n == "logs warn"
        || n == "/logs warn"
        || n == "log warn"
        || n == "log warning"
        || n == "log warnings"
        || n == "debug warn"
        || n == "debug warning"
        || n == "debug warnings"
        || n == "warn log"
        || n == "warn logs"
        || n == "warning log"
        || n == "warning logs"
        || n == "any warnings"
        || n == "any warning"
        || n == "any warns"
        || n == "show warnings"
        || n == "list warnings"
        || n == "show warns"
    {
        return DebugLogListFilter::Warn;
    }
    DebugLogListFilter::All
}

/// True for `/logs` / `debug log` — Error/Warn filter parity; not fix/explain asks.
pub fn looks_like_debug_log_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    if n.contains("why")
        || n.contains("fix")
        || n.contains("explain")
        || n.contains("how to")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains("ticket")
        || n.contains("redmine")
        || n.contains("create")
        || n.contains("delete")
        || n.contains("clear log")
        || n.contains("rotate")
        || n.contains("open in")
        || n.contains("editor")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/logs"
            | "logs"
            | "/log"
            | "log"
            | "debug log"
            | "debug logs"
            | "show logs"
            | "list logs"
            | "log tail"
            | "logs tail"
            | "tail logs"
            | "log status"
            | "logs status"
            | "/logs error"
            | "logs error"
            | "log error"
            | "log errors"
            | "debug error"
            | "debug errors"
            | "error log"
            | "error logs"
            | "any errors"
            | "any error"
            | "show errors"
            | "list errors"
            | "what's wrong"
            | "whats wrong"
            | "what is wrong"
            | "/logs warn"
            | "logs warn"
            | "log warn"
            | "log warning"
            | "log warnings"
            | "debug warn"
            | "debug warning"
            | "debug warnings"
            | "warn log"
            | "warn logs"
            | "warning log"
            | "warning logs"
            | "any warnings"
            | "any warning"
            | "any warns"
            | "show warnings"
            | "list warnings"
            | "show warns"
            | "error"
            | "errors"
            | "warn"
            | "warns"
            | "warning"
            | "warnings"
    )
}

/// Zero-LLM Debug Log report (All · Error · Warn; tail of ~/.mac-stats/debug.log).
pub fn format_debug_log_gateway(filter: DebugLogListFilter) -> String {
    const MAX_ROWS: usize = 12;
    let tail = match crate::commands::logging::read_debug_log(Some(128 * 1024)) {
        Ok(t) => t,
        Err(e) => return format!("**Debug Log** — could not read: {e}"),
    };
    let body = tail.content;
    let mut error_n = 0usize;
    let mut warn_n = 0usize;
    for line in body.lines() {
        match debug_log_line_kind(line) {
            "error" => error_n += 1,
            "warn" => warn_n += 1,
            _ => {}
        }
    }
    let title = match filter {
        DebugLogListFilter::All => {
            format!("**Debug Log** — {error_n} error · {warn_n} warn (tail)")
        }
        DebugLogListFilter::Error => format!("**Debug Log · Error** — {error_n}"),
        DebugLogListFilter::Warn => format!("**Debug Log · Warn** — {warn_n}"),
    };
    let mut lines = vec![title];
    if filter == DebugLogListFilter::All {
        let path_short = truncate_preview(&tail.path, 48);
        lines.push(format!("`{path_short}`"));
    }

    let selected: Vec<String> = match filter {
        DebugLogListFilter::All => {
            let all: Vec<_> = body.lines().map(|s| s.to_string()).collect();
            all.into_iter()
                .rev()
                .take(MAX_ROWS)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect()
        }
        DebugLogListFilter::Error | DebugLogListFilter::Warn => {
            let want = match filter {
                DebugLogListFilter::Error => "error",
                DebugLogListFilter::Warn => "warn",
                DebugLogListFilter::All => unreachable!(),
            };
            let mut out = Vec::new();
            let mut keep_cont = false;
            for line in body.lines() {
                let is_cont = line.starts_with(char::is_whitespace) && !line.trim().is_empty();
                let kind = debug_log_line_kind(line);
                if kind == want {
                    out.push(line.to_string());
                    keep_cont = true;
                } else if keep_cont && is_cont {
                    out.push(line.to_string());
                } else {
                    keep_cont = false;
                }
            }
            // Keep the newest matching blocks (last MAX_ROWS lines of filtered set).
            if out.len() > MAX_ROWS {
                out[out.len() - MAX_ROWS..].to_vec()
            } else {
                out
            }
        }
    };

    if selected.is_empty() {
        let empty = match filter {
            DebugLogListFilter::All => {
                "_Nothing in the log tail yet — check back after the app runs a bit._"
            }
            DebugLogListFilter::Error => "_Nothing here yet — no ERROR lines in this tail._",
            DebugLogListFilter::Warn => "_Nothing here yet — no WARN lines in this tail._",
        };
        lines.push(empty.to_string());
    } else {
        for raw in &selected {
            let preview = truncate_preview(raw.trim_end(), 120);
            let mark = match debug_log_line_kind(raw) {
                "error" => "❌",
                "warn" => "⚠",
                _ => "·",
            };
            lines.push(format!("{mark} `{preview}`"));
        }
        let total_match = match filter {
            DebugLogListFilter::All => body.lines().count(),
            DebugLogListFilter::Error => error_n,
            DebugLogListFilter::Warn => warn_n,
        };
        if total_match > selected.len() {
            lines.push(format!("_…+{} more in tail_", total_match - selected.len()));
        }
    }

    let mut out = lines.join("\n");
    if out.chars().count() > 1800 {
        out = out.chars().take(1790).collect::<String>() + "…";
    }
    out
}

/// Top Processes All · Pinned · Hot filter for `/processes` instant replies (UI parity).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessesListFilter {
    All,
    Pinned,
    Hot,
}

/// Hot thresholds — match `PROCESS_HOT_*` in cpu.js / Top Processes Hot chip.
pub const OPS_PROCESS_HOT_CPU_PCT: f32 = 15.0;
pub const OPS_PROCESS_HOT_GPU_PCT: f32 = 15.0;
pub const OPS_PROCESS_HOT_RAM_BYTES: u64 = 1024 * 1024 * 1024;

pub fn process_row_is_hot(p: &crate::metrics::ProcessUsage) -> bool {
    p.cpu >= OPS_PROCESS_HOT_CPU_PCT
        || p.gpu >= OPS_PROCESS_HOT_GPU_PCT
        || p.memory >= OPS_PROCESS_HOT_RAM_BYTES
}

fn format_process_ram(bytes: u64) -> String {
    const GIB: u64 = 1024 * 1024 * 1024;
    const MIB: u64 = 1024 * 1024;
    if bytes >= GIB {
        format!("{:.1} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{} MiB", bytes / MIB)
    } else if bytes > 0 {
        format!("{} KiB", bytes / 1024)
    } else {
        "—".into()
    }
}

/// True for pin/unpin *actions* (not the Pinned list filter).
fn is_process_pin_action_ask(n: &str) -> bool {
    n.contains("unpin")
        || n.contains("pin this")
        || n.contains("pin that")
        || n.contains("pin the")
        || n == "pin"
        || n.starts_with("pin ")
        || n.contains(" pin ")
}

/// Parse Hot/Pinned from `/processes hot`, `/processes pinned`, etc. Default All.
pub fn parse_processes_list_filter(content: &str) -> ProcessesListFilter {
    let n = normalize_operator_command(content);
    if n.ends_with(" hot")
        || n == "hot"
        || n == "/hot"
        || n == "processes hot"
        || n == "/processes hot"
        || n == "process hot"
        || n == "hot processes"
        || n == "hot process"
        || n == "what's hot"
        || n == "whats hot"
        || n == "what is hot"
        || n == "which processes are hot"
        || n == "which process is hot"
    {
        return ProcessesListFilter::Hot;
    }
    if n.ends_with(" pinned")
        || n == "pinned"
        || n == "/pinned"
        || n == "processes pinned"
        || n == "/processes pinned"
        || n == "process pinned"
        || n == "pinned processes"
        || n == "pinned process"
        || n == "show pinned"
        || n == "list pinned"
        || n == "my pinned"
        || n == "pinned favorites"
        || n == "process favorites"
        || n == "favorite processes"
    {
        return ProcessesListFilter::Pinned;
    }
    ProcessesListFilter::All
}

/// True for `/processes` / `top processes` — Hot/Pinned filter parity; not kill/pin-action asks.
pub fn looks_like_processes_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    if n.contains("why")
        || n.contains("kill")
        || n.contains("force quit")
        || n.contains("force-quit")
        || n.contains("quit ")
        || is_process_pin_action_ask(&n)
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains("ticket")
        || n.contains("redmine")
        || n.contains("how to")
        || n.contains("explain")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/processes"
            | "processes"
            | "/process"
            | "process list"
            | "list processes"
            | "show processes"
            | "top processes"
            | "top process"
            | "process status"
            | "processes status"
            | "/processes hot"
            | "processes hot"
            | "process hot"
            | "hot processes"
            | "hot process"
            | "/hot"
            | "hot"
            | "what's hot"
            | "whats hot"
            | "what is hot"
            | "which processes are hot"
            | "which process is hot"
            | "/processes pinned"
            | "processes pinned"
            | "process pinned"
            | "pinned processes"
            | "pinned process"
            | "/pinned"
            | "pinned"
            | "show pinned"
            | "list pinned"
            | "my pinned"
            | "pinned favorites"
            | "process favorites"
            | "favorite processes"
    )
}

/// Zero-LLM Top Processes report (All · Pinned · Hot; cached list + pinned_processes.json).
pub fn format_processes_gateway(filter: ProcessesListFilter) -> String {
    const MAX_ROWS: usize = 12;
    let details = crate::metrics::get_cpu_details();
    let rows = details.top_processes;
    let hot_n = rows.iter().filter(|p| process_row_is_hot(p)).count();
    let pin_names = crate::metrics::load_pinned_process_names();
    let pin_n = pin_names.len();
    let total = rows.len();
    let title = match filter {
        ProcessesListFilter::All => {
            format!("**Top Processes** — {total} · {hot_n} hot · {pin_n} pinned")
        }
        ProcessesListFilter::Hot => {
            format!(
                "**Top Processes · Hot** — {hot_n} (CPU≥{:.0}% · GPU≥{:.0}% · RAM≥1 GiB)",
                OPS_PROCESS_HOT_CPU_PCT, OPS_PROCESS_HOT_GPU_PCT
            )
        }
        ProcessesListFilter::Pinned => {
            format!(
                "**Top Processes · Pinned** — {pin_n} (max {})",
                crate::metrics::MAX_PINNED_PROCESS_NAMES
            )
        }
    };
    let mut lines = vec![title];

    match filter {
        ProcessesListFilter::All | ProcessesListFilter::Hot => {
            let filtered: Vec<_> = match filter {
                ProcessesListFilter::All => rows.iter().collect(),
                ProcessesListFilter::Hot => rows.iter().filter(|p| process_row_is_hot(p)).collect(),
                ProcessesListFilter::Pinned => unreachable!(),
            };

            if filtered.is_empty() {
                let empty = match filter {
                    ProcessesListFilter::All => {
                        "_Nothing here yet — open the CPU window so Top Processes can fill in._"
                    }
                    ProcessesListFilter::Hot => {
                        "_No process is hot right now (CPU ≥15%, GPU ≥15%, or RAM ≥1 GiB)._"
                    }
                    ProcessesListFilter::Pinned => unreachable!(),
                };
                lines.push(empty.to_string());
            } else {
                for p in filtered.iter().take(MAX_ROWS) {
                    let name = truncate_preview(&p.name, 40);
                    let ram = format_process_ram(p.memory);
                    let hot_mark = if process_row_is_hot(p) { " · hot" } else { "" };
                    let pin_mark = if pin_names.iter().any(|n| n == &p.name) {
                        " · ★"
                    } else {
                        ""
                    };
                    lines.push(format!(
                        "• `{name}` · pid {pid} · CPU {cpu:.0}% · GPU {gpu:.0}% · {ram}{hot_mark}{pin_mark}",
                        pid = p.pid,
                        cpu = p.cpu,
                        gpu = p.gpu,
                    ));
                }
                if filtered.len() > MAX_ROWS {
                    lines.push(format!("_…+{} more_", filtered.len() - MAX_ROWS));
                }
            }
        }
        ProcessesListFilter::Pinned => {
            if pin_names.is_empty() {
                lines.push(
                    "_No pinned favorites yet — star a process in the CPU window (max 6)._"
                        .to_string(),
                );
            } else {
                let live = crate::metrics::get_processes_by_names(pin_names.clone());
                for name in &pin_names {
                    if let Some(p) = live.iter().find(|p| &p.name == name) {
                        let short = truncate_preview(&p.name, 40);
                        let ram = format_process_ram(p.memory);
                        let hot_mark = if process_row_is_hot(p) { " · hot" } else { "" };
                        lines.push(format!(
                            "• ★ `{short}` · pid {pid} · CPU {cpu:.0}% · GPU {gpu:.0}% · {ram}{hot_mark}",
                            pid = p.pid,
                            cpu = p.cpu,
                            gpu = p.gpu,
                        ));
                    } else {
                        let short = truncate_preview(name, 40);
                        lines.push(format!("• ★ `{short}` · _not running_"));
                    }
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

/// CPU rings All · Hot filter for `/rings` instant replies (UI parity).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingsListFilter {
    All,
    Hot,
}

/// Hot thresholds — match `RING_HOT_*` in cpu.js / CPU rings Hot chip.
pub const OPS_RING_HOT_CPU_PCT: f32 = 50.0;
pub const OPS_RING_HOT_GPU_PCT: f32 = 15.0;
pub const OPS_RING_HOT_FREQ_GHZ: f32 = 3.5;
pub const OPS_RING_HOT_TEMP_C: f32 = 70.0;

fn ring_cpu_is_hot(usage: f32) -> bool {
    usage >= OPS_RING_HOT_CPU_PCT
}

fn ring_gpu_is_hot(gpu: f32) -> bool {
    gpu >= OPS_RING_HOT_GPU_PCT
}

fn ring_freq_is_hot(freq_ghz: f32) -> bool {
    freq_ghz >= OPS_RING_HOT_FREQ_GHZ
}

fn ring_temp_is_hot(temp_c: f32) -> bool {
    temp_c >= OPS_RING_HOT_TEMP_C
}

/// Parse Hot from `/rings hot`, `hot rings`, etc. Default All.
pub fn parse_rings_list_filter(content: &str) -> RingsListFilter {
    let n = normalize_operator_command(content);
    if n.ends_with(" hot")
        || n == "rings hot"
        || n == "/rings hot"
        || n == "cpu rings hot"
        || n == "hot rings"
        || n == "hot ring"
        || n == "which rings are hot"
        || n == "which ring is hot"
        || n == "show hot rings"
        || n == "list hot rings"
        || n == "what's hot on rings"
        || n == "whats hot on rings"
    {
        return RingsListFilter::Hot;
    }
    RingsListFilter::All
}

/// True for `/rings` / `cpu rings` — Hot filter parity; not process `/hot` asks.
pub fn looks_like_rings_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    if n.contains("why")
        || n.contains("process")
        || n.contains("kill")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains("ticket")
        || n.contains("redmine")
        || n.contains("how to")
        || n.contains("explain")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/rings"
            | "rings"
            | "cpu rings"
            | "ring gauges"
            | "ring gauge"
            | "metric rings"
            | "list rings"
            | "show rings"
            | "rings status"
            | "/rings hot"
            | "rings hot"
            | "cpu rings hot"
            | "hot rings"
            | "hot ring"
            | "which rings are hot"
            | "which ring is hot"
            | "show hot rings"
            | "list hot rings"
            | "what's hot on rings"
            | "whats hot on rings"
    )
}

/// Zero-LLM CPU rings report (All · Hot; live get_cpu_details; menu-bar amber thresholds).
pub fn format_rings_gateway(filter: RingsListFilter) -> String {
    let d = crate::metrics::get_cpu_details();
    let cpu = d.usage;
    let gpu = d.gpu_usage;
    let freq = d.frequency;
    let temp = d.temperature;

    let rows: [( &str, String, bool); 4] = [
        (
            "CPU",
            format!("{cpu:.0}%"),
            ring_cpu_is_hot(cpu),
        ),
        (
            "GPU",
            format!("{gpu:.0}%"),
            ring_gpu_is_hot(gpu),
        ),
        (
            "Freq",
            if freq > 0.0 {
                format!("{freq:.2} GHz")
            } else {
                "—".into()
            },
            freq > 0.0 && ring_freq_is_hot(freq),
        ),
        (
            "Temp",
            if temp > 0.0 {
                format!("{temp:.0}°C")
            } else {
                "—".into()
            },
            temp > 0.0 && ring_temp_is_hot(temp),
        ),
    ];

    let hot_n = rows.iter().filter(|(_, _, hot)| *hot).count();
    let title = match filter {
        RingsListFilter::All => {
            format!("**CPU rings** — 4 · {hot_n} hot")
        }
        RingsListFilter::Hot => {
            format!(
                "**CPU rings · Hot** — {hot_n} (CPU≥{:.0}% · GPU≥{:.0}% · Freq≥{} GHz · Temp≥{:.0}°C)",
                OPS_RING_HOT_CPU_PCT,
                OPS_RING_HOT_GPU_PCT,
                OPS_RING_HOT_FREQ_GHZ,
                OPS_RING_HOT_TEMP_C
            )
        }
    };
    let mut lines = vec![title];

    let filtered: Vec<_> = match filter {
        RingsListFilter::All => rows.iter().collect(),
        RingsListFilter::Hot => rows.iter().filter(|(_, _, hot)| *hot).collect(),
    };

    if filtered.is_empty() {
        lines.push(match filter {
            RingsListFilter::All => {
                "_Nothing here yet — open the CPU window so rings can fill in._".to_string()
            }
            RingsListFilter::Hot => format!(
                "_No ring is hot right now (CPU ≥{:.0}%, GPU ≥{:.0}%, Freq ≥{} GHz, or Temp ≥{:.0}°C)._",
                OPS_RING_HOT_CPU_PCT,
                OPS_RING_HOT_GPU_PCT,
                OPS_RING_HOT_FREQ_GHZ,
                OPS_RING_HOT_TEMP_C
            ),
        });
    } else {
        for (label, value, hot) in filtered {
            let hot_mark = if *hot { " · hot" } else { "" };
            lines.push(format!("• **{label}** · {value}{hot_mark}"));
        }
    }

    let mut out = lines.join("\n");
    if out.chars().count() > 1800 {
        out = out.chars().take(1790).collect::<String>() + "…";
    }
    out
}

/// Focused CPU ring asks (`/cpu` · `/gpu` · `/freq` · `/temp`) — not full `/rings`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingChipAsk {
    Cpu,
    Gpu,
    Freq,
    Temp,
}

/// Parse `/cpu` · `/gpu` · `/freq` · `/temp` (and short NL). None when not a chip ask.
pub fn parse_ring_chip_ask(content: &str) -> Option<RingChipAsk> {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return None;
    }
    if n.contains("why")
        || n.contains("process")
        || n.contains("kill")
        || n.contains("ring")
        || n.contains("strip")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains("ticket")
        || n.contains("redmine")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("cleanup")
        || n.contains("clean up")
        || n.contains("search")
        || n.contains("weather")
        || n.contains("detail")
        || n.contains("pin")
    {
        return None;
    }
    if matches!(
        n.as_str(),
        "/cpu"
            | "cpu"
            | "cpu usage"
            | "cpu percent"
            | "cpu percentage"
            | "cpu %"
            | "show cpu"
            | "list cpu"
            | "cpu status"
            | "what's the cpu"
            | "whats the cpu"
            | "what is the cpu"
            | "how's the cpu"
            | "hows the cpu"
    ) {
        return Some(RingChipAsk::Cpu);
    }
    if matches!(
        n.as_str(),
        "/gpu"
            | "gpu"
            | "gpu usage"
            | "gpu percent"
            | "gpu percentage"
            | "gpu %"
            | "show gpu"
            | "list gpu"
            | "gpu status"
            | "what's the gpu"
            | "whats the gpu"
            | "what is the gpu"
            | "how's the gpu"
            | "hows the gpu"
    ) {
        return Some(RingChipAsk::Gpu);
    }
    if matches!(
        n.as_str(),
        "/freq"
            | "/frequency"
            | "/ghz"
            | "freq"
            | "frequency"
            | "ghz"
            | "cpu frequency"
            | "cpu freq"
            | "cpu ghz"
            | "show freq"
            | "list freq"
            | "freq status"
            | "what's the frequency"
            | "whats the frequency"
            | "what is the frequency"
            | "what's the freq"
            | "whats the freq"
            | "what is the freq"
            | "how's the frequency"
            | "hows the frequency"
    ) {
        return Some(RingChipAsk::Freq);
    }
    if matches!(
        n.as_str(),
        "/temp"
            | "/temperature"
            | "temp"
            | "temperature"
            | "cpu temp"
            | "cpu temperature"
            | "show temp"
            | "list temp"
            | "temp status"
            | "what's the temp"
            | "whats the temp"
            | "what is the temp"
            | "what's the temperature"
            | "whats the temperature"
            | "what is the temperature"
            | "how's the temp"
            | "hows the temp"
            | "how's the temperature"
            | "hows the temperature"
    ) {
        return Some(RingChipAsk::Temp);
    }
    None
}

/// True for focused CPU · GPU · Freq · Temp ring asks (not full `/rings`).
pub fn looks_like_ring_chip_request(content: &str) -> bool {
    parse_ring_chip_ask(content).is_some()
}

/// Zero-LLM one-ring reply (CPU · GPU · Freq · Temp; live get_cpu_details; menu-bar amber cues).
pub fn format_ring_chip_gateway(ask: RingChipAsk) -> String {
    let d = crate::metrics::get_cpu_details();
    match ask {
        RingChipAsk::Cpu => {
            let hot_mark = if ring_cpu_is_hot(d.usage) {
                " · hot"
            } else {
                ""
            };
            format!("**CPU** · {:.0}%{hot_mark}", d.usage)
        }
        RingChipAsk::Gpu => {
            let hot_mark = if ring_gpu_is_hot(d.gpu_usage) {
                " · hot"
            } else {
                ""
            };
            format!("**GPU** · {:.0}%{hot_mark}", d.gpu_usage)
        }
        RingChipAsk::Freq => {
            if d.frequency <= 0.0 {
                return "**Freq** — _no frequency reading right now (open the CPU window)._"
                    .to_string();
            }
            let hot_mark = if ring_freq_is_hot(d.frequency) {
                " · hot"
            } else {
                ""
            };
            format!("**Freq** · {:.2} GHz{hot_mark}", d.frequency)
        }
        RingChipAsk::Temp => {
            if d.temperature <= 0.0 {
                return "**Temp** — _no temperature reading right now (open the CPU window)._"
                    .to_string();
            }
            let hot_mark = if ring_temp_is_hot(d.temperature) {
                " · hot"
            } else {
                ""
            };
            format!("**Temp** · {:.0}°C{hot_mark}", d.temperature)
        }
    }
}

/// Power strip All · Hot filter for `/strip` instant replies (menu-bar amber parity).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StripListFilter {
    All,
    Hot,
}

/// Strip attention thresholds — match power-strip / menu-bar washes in cpu.js.
pub const OPS_STRIP_BAT_LOW_PCT: f32 = 20.0;
pub const OPS_STRIP_UPTIME_LONG_SECS: u64 = 7 * 24 * 3600;
pub const OPS_STRIP_RAM_HOT_PCT: f32 = 85.0;
pub const OPS_STRIP_SSD_HOT_PCT: f32 = 85.0;

fn format_system_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    if days > 0 {
        format!("{days}d {hours}h")
    } else if hours > 0 {
        format!("{hours}h {mins}m")
    } else {
        format!("{mins}m")
    }
}

fn strip_heat_is_attention(thermal: &str) -> bool {
    matches!(
        thermal.trim(),
        "Fair" | "Serious" | "Critical"
    )
}

/// Parse Hot from `/strip hot`, `hot strip`, etc. Default All.
pub fn parse_strip_list_filter(content: &str) -> StripListFilter {
    let n = normalize_operator_command(content);
    if n.ends_with(" hot")
        || n == "strip hot"
        || n == "/strip hot"
        || n == "power strip hot"
        || n == "powerstrip hot"
        || n == "/power hot"
        || n == "hot strip"
        || n == "hot power strip"
        || n == "which strip is hot"
        || n == "which chips are hot"
        || n == "show hot strip"
        || n == "list hot strip"
        || n == "what's hot on strip"
        || n == "whats hot on strip"
        || n == "what's hot on the strip"
        || n == "whats hot on the strip"
    {
        return StripListFilter::Hot;
    }
    StripListFilter::All
}

/// True for `/strip` / `power strip` — not process `/hot` or `/rings`.
pub fn looks_like_strip_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    if n.contains("why")
        || n.contains("process")
        || n.contains("kill")
        || n.contains("ring")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains("ticket")
        || n.contains("redmine")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("cleanup")
        || n.contains("clean up")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/strip"
            | "strip"
            | "power strip"
            | "powerstrip"
            | "/power"
            | "power"
            | "battery strip"
            | "list strip"
            | "show strip"
            | "strip status"
            | "power status"
            | "/strip hot"
            | "strip hot"
            | "power strip hot"
            | "powerstrip hot"
            | "/power hot"
            | "hot strip"
            | "hot power strip"
            | "which strip is hot"
            | "which chips are hot"
            | "show hot strip"
            | "list hot strip"
            | "what's hot on strip"
            | "whats hot on strip"
            | "what's hot on the strip"
            | "whats hot on the strip"
    )
}

/// Zero-LLM power strip report (All · Hot; live get_cpu_details; menu-bar amber cues).
pub fn format_strip_gateway(filter: StripListFilter) -> String {
    let d = crate::metrics::get_cpu_details();
    let mut rows: Vec<(&str, String, bool)> = Vec::with_capacity(11);

    if d.has_battery && d.battery_level >= 0.0 {
        let bat_hot = d.battery_level <= OPS_STRIP_BAT_LOW_PCT && !d.is_charging;
        let charge = if d.is_charging { " · charging" } else { "" };
        rows.push((
            "Bat",
            format!("{:.0}%{charge}", d.battery_level),
            bat_hot,
        ));
    }

    rows.push((
        "LPM",
        if d.low_power_mode {
            "On".into()
        } else {
            "Off".into()
        },
        d.low_power_mode,
    ));

    let heat = if d.thermal_state.trim().is_empty() {
        "—".to_string()
    } else {
        d.thermal_state.clone()
    };
    rows.push((
        "Heat",
        heat,
        strip_heat_is_attention(&d.thermal_state),
    ));

    rows.push((
        "Up",
        format_system_uptime(d.uptime_secs),
        d.uptime_secs >= OPS_STRIP_UPTIME_LONG_SECS,
    ));

    rows.push((
        "CPU",
        format!("{:.0}%", d.usage),
        ring_cpu_is_hot(d.usage),
    ));
    rows.push((
        "GPU",
        format!("{:.0}%", d.gpu_usage),
        ring_gpu_is_hot(d.gpu_usage),
    ));
    rows.push((
        "Freq",
        if d.frequency > 0.0 {
            format!("{:.2} GHz", d.frequency)
        } else {
            "—".into()
        },
        d.frequency > 0.0 && ring_freq_is_hot(d.frequency),
    ));
    rows.push((
        "Temp",
        if d.temperature > 0.0 {
            format!("{:.0}°C", d.temperature)
        } else {
            "—".into()
        },
        d.temperature > 0.0 && ring_temp_is_hot(d.temperature),
    ));
    rows.push((
        "RAM",
        format!("{:.0}%", d.ram_percent),
        d.ram_percent >= OPS_STRIP_RAM_HOT_PCT,
    ));
    rows.push((
        "SSD",
        format!("{:.0}%", d.disk_percent),
        d.disk_percent >= OPS_STRIP_SSD_HOT_PCT,
    ));

    let hot_n = rows.iter().filter(|(_, _, hot)| *hot).count();
    let title = match filter {
        StripListFilter::All => {
            format!("**Power strip** — {} · {hot_n} hot", rows.len())
        }
        StripListFilter::Hot => format!(
            "**Power strip · Hot** — {hot_n} (Bat≤{:.0}% · Heat Fair+ · Up≥7d · CPU≥{:.0}% · GPU≥{:.0}% · Freq≥{} GHz · Temp≥{:.0}°C · RAM/SSD≥{:.0}%)",
            OPS_STRIP_BAT_LOW_PCT,
            OPS_RING_HOT_CPU_PCT,
            OPS_RING_HOT_GPU_PCT,
            OPS_RING_HOT_FREQ_GHZ,
            OPS_RING_HOT_TEMP_C,
            OPS_STRIP_RAM_HOT_PCT
        ),
    };
    let mut lines = vec![title];

    let filtered: Vec<_> = match filter {
        StripListFilter::All => rows.iter().collect(),
        StripListFilter::Hot => rows.iter().filter(|(_, _, hot)| *hot).collect(),
    };

    if filtered.is_empty() {
        lines.push(match filter {
            StripListFilter::All => {
                "_Nothing here yet — open the CPU window so the strip can fill in._".to_string()
            }
            StripListFilter::Hot => {
                "_No power-strip chip is hot right now (menu-bar amber / attention cues)._"
                    .to_string()
            }
        });
    } else {
        for (label, value, hot) in filtered {
            let hot_mark = if *hot { " · hot" } else { "" };
            lines.push(format!("• **{label}** · {value}{hot_mark}"));
        }
    }

    let mut out = lines.join("\n");
    if out.chars().count() > 1800 {
        out = out.chars().take(1790).collect::<String>() + "…";
    }
    out
}

/// Focused power-strip chip asks (`/battery` · `/heat` · `/lpm` · `/ram` · `/ssd` · `/uptime`) — not full `/strip`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StripChipAsk {
    Battery,
    Heat,
    Lpm,
    Ram,
    Ssd,
    Uptime,
}

/// Parse `/battery` · `/heat` · `/lpm` · `/ram` · `/ssd` · `/uptime` (and short NL). None when not a chip ask.
pub fn parse_strip_chip_ask(content: &str) -> Option<StripChipAsk> {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return None;
    }
    if n.contains("why")
        || n.contains("process")
        || n.contains("kill")
        || n.contains("ring")
        || n.contains("strip")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains("ticket")
        || n.contains("redmine")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("cleanup")
        || n.contains("clean up")
        || n.contains("reclaim")
        || n.contains("search")
        || n.contains("weather")
        || n.contains("detail")
    {
        return None;
    }
    // Battery chip — not "battery strip" (handled by `/strip`).
    if matches!(
        n.as_str(),
        "/battery"
            | "/bat"
            | "battery"
            | "bat"
            | "battery level"
            | "battery percent"
            | "battery percentage"
            | "battery %"
            | "show battery"
            | "list battery"
            | "battery status"
            | "what's the battery"
            | "whats the battery"
            | "what is the battery"
            | "how's the battery"
            | "hows the battery"
            | "battery charge"
            | "charge level"
    ) {
        return Some(StripChipAsk::Battery);
    }
    // Heat / thermal chip — not process `/hot`.
    if matches!(
        n.as_str(),
        "/heat"
            | "/thermal"
            | "heat"
            | "thermal"
            | "thermal state"
            | "thermal pressure"
            | "heat state"
            | "show heat"
            | "list heat"
            | "heat status"
            | "what's the heat"
            | "whats the heat"
            | "what is the heat"
            | "what's the thermal"
            | "whats the thermal"
            | "what is the thermal"
            | "how's the heat"
            | "hows the heat"
    ) {
        return Some(StripChipAsk::Heat);
    }
    // Low Power Mode chip — not bare `/power` (full strip).
    if matches!(
        n.as_str(),
        "/lpm"
            | "lpm"
            | "low power"
            | "low power mode"
            | "low-power mode"
            | "low-power"
            | "show lpm"
            | "list lpm"
            | "lpm status"
            | "is lpm on"
            | "is lpm off"
            | "is low power on"
            | "is low power mode on"
            | "is low power mode off"
            | "low power status"
            | "what's lpm"
            | "whats lpm"
            | "what is lpm"
    ) {
        return Some(StripChipAsk::Lpm);
    }
    // RAM chip — not `/details` (full Load · RAM · Up).
    if matches!(
        n.as_str(),
        "/ram"
            | "ram"
            | "memory"
            | "mem"
            | "/memory"
            | "/mem"
            | "ram percent"
            | "ram percentage"
            | "ram %"
            | "memory percent"
            | "memory %"
            | "show ram"
            | "list ram"
            | "ram status"
            | "what's the ram"
            | "whats the ram"
            | "what is the ram"
            | "how's the ram"
            | "hows the ram"
            | "what's the memory"
            | "whats the memory"
            | "what is the memory"
    ) {
        return Some(StripChipAsk::Ram);
    }
    // SSD chip — not `/disk` Disk Cleanup (cleanup/reclaim rejected above).
    if matches!(
        n.as_str(),
        "/ssd"
            | "ssd"
            | "disk percent"
            | "disk percentage"
            | "disk %"
            | "ssd percent"
            | "ssd %"
            | "disk usage"
            | "ssd usage"
            | "show ssd"
            | "list ssd"
            | "ssd status"
            | "what's the ssd"
            | "whats the ssd"
            | "what is the ssd"
            | "how's the ssd"
            | "hows the ssd"
            | "how full is the disk"
            | "how full is the ssd"
            | "disk free"
            | "free disk"
            | "free space"
    ) {
        return Some(StripChipAsk::Ssd);
    }
    // Uptime chip — not full `/details` / `/strip`.
    if matches!(
        n.as_str(),
        "/uptime"
            | "/up"
            | "uptime"
            | "up time"
            | "system uptime"
            | "show uptime"
            | "list uptime"
            | "uptime status"
            | "what's the uptime"
            | "whats the uptime"
            | "what is the uptime"
            | "how's the uptime"
            | "hows the uptime"
            | "how long up"
            | "how long has it been up"
            | "how long has the mac been up"
    ) {
        return Some(StripChipAsk::Uptime);
    }
    None
}

/// True for focused Bat · Heat · LPM · RAM · SSD · Up chip asks (power-strip parity; not full `/strip`).
pub fn looks_like_strip_chip_request(content: &str) -> bool {
    parse_strip_chip_ask(content).is_some()
}

/// Zero-LLM one-chip reply (Bat · Heat · LPM · RAM · SSD · Up; live get_cpu_details; menu-bar amber cues).
pub fn format_strip_chip_gateway(ask: StripChipAsk) -> String {
    let d = crate::metrics::get_cpu_details();
    match ask {
        StripChipAsk::Battery => {
            if !d.has_battery || d.battery_level < 0.0 {
                return "**Bat** — _no battery reading on this Mac right now._".to_string();
            }
            let bat_hot = d.battery_level <= OPS_STRIP_BAT_LOW_PCT && !d.is_charging;
            let charge = if d.is_charging { " · charging" } else { "" };
            let hot_mark = if bat_hot { " · hot" } else { "" };
            format!(
                "**Bat** · {:.0}%{charge}{hot_mark}",
                d.battery_level
            )
        }
        StripChipAsk::Heat => {
            let heat = if d.thermal_state.trim().is_empty() {
                "—"
            } else {
                d.thermal_state.trim()
            };
            let hot_mark = if strip_heat_is_attention(&d.thermal_state) {
                " · hot"
            } else {
                ""
            };
            format!("**Heat** · {heat}{hot_mark}")
        }
        StripChipAsk::Lpm => {
            let on = d.low_power_mode;
            let value = if on { "On" } else { "Off" };
            let hot_mark = if on { " · hot" } else { "" };
            format!("**LPM** · {value}{hot_mark}")
        }
        StripChipAsk::Ram => {
            let hot_mark = if d.ram_percent >= OPS_STRIP_RAM_HOT_PCT {
                " · hot"
            } else {
                ""
            };
            format!("**RAM** · {:.0}%{hot_mark}", d.ram_percent)
        }
        StripChipAsk::Ssd => {
            let hot_mark = if d.disk_percent >= OPS_STRIP_SSD_HOT_PCT {
                " · hot"
            } else {
                ""
            };
            format!("**SSD** · {:.0}%{hot_mark}", d.disk_percent)
        }
        StripChipAsk::Uptime => {
            let hot_mark = if d.uptime_secs >= OPS_STRIP_UPTIME_LONG_SECS {
                " · long"
            } else {
                ""
            };
            format!(
                "**Up** · {}{hot_mark}",
                format_system_uptime(d.uptime_secs)
            )
        }
    }
}

/// Details All · Hot filter for `/details` instant replies (collapsed glance parity).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailsListFilter {
    All,
    Hot,
}

/// Load attention — match Details collapsed glance `.is-hot` (Load ≥ 4).
pub const OPS_DETAILS_LOAD_HOT: f64 = 4.0;

fn details_load_is_hot(load_1: f64) -> bool {
    load_1 >= OPS_DETAILS_LOAD_HOT
}

fn details_ram_is_hot(ram_pct: f32) -> bool {
    ram_pct >= OPS_STRIP_RAM_HOT_PCT
}

/// Parse Hot from `/details hot`, `hot details`, etc. Default All.
pub fn parse_details_list_filter(content: &str) -> DetailsListFilter {
    let n = normalize_operator_command(content);
    if n.ends_with(" hot")
        || n == "details hot"
        || n == "/details hot"
        || n == "hot details"
        || n == "which details are hot"
        || n == "which detail is hot"
        || n == "show hot details"
        || n == "list hot details"
        || n == "what's hot on details"
        || n == "whats hot on details"
    {
        return DetailsListFilter::Hot;
    }
    DetailsListFilter::All
}

/// True for `/details` / `/load` / load average — not process details or strip/rings.
pub fn looks_like_details_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    if n.contains("why")
        || n.contains("process")
        || n.contains("kill")
        || n.contains("ring")
        || n.contains("strip")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains("ticket")
        || n.contains("redmine")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("more detail")
        || n.contains("full detail")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/details"
            | "details"
            | "system details"
            | "cpu details"
            | "list details"
            | "show details"
            | "details status"
            | "/details hot"
            | "details hot"
            | "hot details"
            | "which details are hot"
            | "which detail is hot"
            | "show hot details"
            | "list hot details"
            | "what's hot on details"
            | "whats hot on details"
            | "/load"
            | "load"
            | "load average"
            | "load avg"
            | "system load"
            | "cpu load"
            | "show load"
            | "list load"
            | "what's the load"
            | "whats the load"
            | "what is the load"
            | "load 1"
            | "1-minute load"
    )
}

/// Zero-LLM Details report (All · Hot; live get_cpu_details; glance Load≥4 · RAM≥85%).
pub fn format_details_gateway(filter: DetailsListFilter) -> String {
    let d = crate::metrics::get_cpu_details();
    let load_hot = details_load_is_hot(d.load_1);
    let ram_hot = details_ram_is_hot(d.ram_percent);
    let up_long = d.uptime_secs >= OPS_STRIP_UPTIME_LONG_SECS;

    let rows: [(&str, String, bool); 3] = [
        ("Load", format!("{:.2}", d.load_1), load_hot),
        ("RAM", format!("{:.0}%", d.ram_percent), ram_hot),
        (
            "Up",
            format_system_uptime(d.uptime_secs),
            // Glance hot is Load/RAM only; long uptime is informational (strip parity mark).
            up_long,
        ),
    ];

    // Hot filter matches Details glance wash (Load≥4 · RAM≥85%), not long Up alone.
    let hot_n = rows
        .iter()
        .filter(|(label, _, hot)| *hot && (*label == "Load" || *label == "RAM"))
        .count();
    let title = match filter {
        DetailsListFilter::All => {
            format!("**Details** — Load · RAM · Up · {hot_n} hot")
        }
        DetailsListFilter::Hot => {
            format!(
                "**Details · Hot** — {hot_n} (Load≥{:.0} · RAM≥{:.0}%)",
                OPS_DETAILS_LOAD_HOT, OPS_STRIP_RAM_HOT_PCT
            )
        }
    };
    let mut lines = vec![title];

    let filtered: Vec<_> = match filter {
        DetailsListFilter::All => rows.iter().collect(),
        DetailsListFilter::Hot => rows
            .iter()
            .filter(|(label, _, hot)| *hot && (*label == "Load" || *label == "RAM"))
            .collect(),
    };

    if filtered.is_empty() {
        lines.push(match filter {
            DetailsListFilter::All => {
                "_Nothing here yet — open the CPU window so Details can fill in._".to_string()
            }
            DetailsListFilter::Hot => format!(
                "_No Details row is hot right now (Load ≥{:.0} or RAM ≥{:.0}%)._",
                OPS_DETAILS_LOAD_HOT, OPS_STRIP_RAM_HOT_PCT
            ),
        });
    } else {
        for (label, value, hot) in filtered {
            let mark = if *label == "Up" && *hot {
                " · long"
            } else if *hot {
                " · hot"
            } else {
                ""
            };
            lines.push(format!("• **{label}** · {value}{mark}"));
        }
    }

    let mut out = lines.join("\n");
    if out.chars().count() > 1800 {
        out = out.chars().take(1790).collect::<String>() + "…";
    }
    out
}

/// Perplexity Search All · Top · Snippet filter for `/perplexity` instant (UI parity).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerplexityListFilter {
    All,
    Top,
    Snippet,
}

/// Top-N hits — match `PERPLEXITY_TOP_N` in cpu.js.
pub const OPS_PERPLEXITY_TOP_N: usize = 3;

fn perplexity_result_has_snippet(snippet: &str) -> bool {
    !snippet.trim().is_empty()
}

/// Parse Top/Snippet from `/perplexity top`, `snippet results`, etc. Default All.
pub fn parse_perplexity_list_filter(content: &str) -> PerplexityListFilter {
    let n = normalize_operator_command(content);
    if n.ends_with(" top")
        || n == "top"
        || n == "/top"
        || n == "perplexity top"
        || n == "/perplexity top"
        || n == "top results"
        || n == "top result"
        || n == "last search top"
        || n == "last perplexity top"
    {
        return PerplexityListFilter::Top;
    }
    if n.ends_with(" snippet")
        || n.ends_with(" snippets")
        || n == "snippet"
        || n == "snippets"
        || n == "/snippet"
        || n == "perplexity snippet"
        || n == "/perplexity snippet"
        || n == "snippet results"
        || n == "snippets results"
        || n == "results with snippets"
        || n == "last search snippet"
        || n == "last perplexity snippet"
    {
        return PerplexityListFilter::Snippet;
    }
    PerplexityListFilter::All
}

/// True for `/perplexity` / `last search` — Top/Snippet filter parity; not new search asks.
pub fn looks_like_perplexity_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    if n.contains("why")
        || n.contains("search for")
        || n.contains("look up")
        || n.contains(" about ")
        || n.contains("ticket")
        || n.contains("redmine")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("run search")
        || n.contains("do a search")
        || n.starts_with("perplexity search ")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/perplexity"
            | "perplexity"
            | "last search"
            | "last perplexity"
            | "perplexity results"
            | "search results"
            | "list perplexity"
            | "show perplexity"
            | "perplexity status"
            | "perplexity search"
            | "/perplexity top"
            | "perplexity top"
            | "top results"
            | "top result"
            | "last search top"
            | "last perplexity top"
            | "/top"
            | "/perplexity snippet"
            | "perplexity snippet"
            | "snippet results"
            | "snippets results"
            | "results with snippets"
            | "last search snippet"
            | "last perplexity snippet"
            | "/snippet"
            | "snippet"
            | "snippets"
    )
}

/// Zero-LLM last Perplexity report (All · Top · Snippet; ~/.mac-stats/perplexity_last.json).
pub fn format_perplexity_gateway(filter: PerplexityListFilter) -> String {
    const MAX_ROWS: usize = 12;
    let Some(cache) = crate::commands::perplexity::load_last_perplexity_search() else {
        return "**Perplexity Search** — _Nothing here yet — run a search in the CPU window or ask me to search first._".to_string();
    };
    let rows = &cache.results;
    let total = rows.len();
    let top_n = total.min(OPS_PERPLEXITY_TOP_N);
    let snippet_n = rows
        .iter()
        .filter(|r| perplexity_result_has_snippet(&r.snippet))
        .count();
    let q = truncate_preview(cache.query.trim(), 48);
    let title = match filter {
        PerplexityListFilter::All => {
            format!("**Perplexity Search** — {total} · {top_n} top · {snippet_n} snippet")
        }
        PerplexityListFilter::Top => {
            format!("**Perplexity · Top** — {top_n} (first {OPS_PERPLEXITY_TOP_N})")
        }
        PerplexityListFilter::Snippet => {
            format!("**Perplexity · Snippet** — {snippet_n}")
        }
    };
    let mut lines = vec![title];
    if !q.is_empty() {
        lines.push(format!("Query · `{q}`"));
    }

    let filtered: Vec<(usize, &crate::commands::perplexity::PerplexitySearchResult)> =
        match filter {
            PerplexityListFilter::All => rows.iter().enumerate().collect(),
            PerplexityListFilter::Top => rows.iter().enumerate().take(OPS_PERPLEXITY_TOP_N).collect(),
            PerplexityListFilter::Snippet => rows
                .iter()
                .enumerate()
                .filter(|(_, r)| perplexity_result_has_snippet(&r.snippet))
                .collect(),
        };

    if filtered.is_empty() {
        let empty = match filter {
            PerplexityListFilter::All => {
                "_Last search returned no results — try another query._"
            }
            PerplexityListFilter::Top => "_Nothing in Top yet — last search had no hits._",
            PerplexityListFilter::Snippet => {
                "_Nothing here yet — no results with preview text in the last search._"
            }
        };
        lines.push(empty.to_string());
    } else {
        for (i, r) in filtered.iter().take(MAX_ROWS) {
            let title_s = truncate_preview(r.title.trim(), 48);
            let url = truncate_preview(r.url.trim(), 56);
            let snip = r.snippet.trim();
            let snip_mark = if perplexity_result_has_snippet(snip) {
                " · snippet"
            } else {
                ""
            };
            let top_mark = if *i < OPS_PERPLEXITY_TOP_N {
                " · top"
            } else {
                ""
            };
            if filter == PerplexityListFilter::Snippet && !snip.is_empty() {
                let preview = truncate_preview(snip, 80);
                lines.push(format!(
                    "• **{title_s}**{top_mark}\n  `{url}`\n  _{preview}_"
                ));
            } else {
                lines.push(format!("• **{title_s}**{top_mark}{snip_mark}\n  `{url}`"));
            }
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

/// True for focused Discord gateway asks (`/discord` · Ready/Offline) — not Knowledge Discord,
/// not free-form “post to Discord…”.
pub fn looks_like_discord_gateway_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    // Knowledge / memory / post / message stay with their own handlers or the agent.
    if n.contains("knowledge")
        || n.contains("memory")
        || n.contains("post")
        || n.contains("send")
        || n.contains("message")
        || n.contains("channel list")
        || n.contains("list channel")
        || n.contains("guild")
        || n.contains("ticket")
        || n.contains("redmine")
        || n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains(" for ")
        || n.contains(" about ")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/discord"
            | "discord"
            | "discord status"
            | "discord gateway"
            | "discord ready"
            | "discord offline"
            | "gateway status"
            | "gateway ready"
            | "bot gateway"
            | "show discord"
            | "list discord"
            | "is discord ready"
            | "is discord online"
            | "is discord connected"
            | "is discord offline"
            | "is the bot ready"
            | "is the bot connected"
            | "is the gateway ready"
            | "is the gateway connected"
            | "discord connection"
            | "discord reconnect"
            | "discord disconnects"
            | "how's discord"
            | "hows discord"
            | "how's the discord"
            | "hows the discord"
            | "how's the gateway"
            | "hows the gateway"
    )
}

/// Zero-LLM Discord Ready / Offline chip (Agent Ops collapsed-glance parity).
pub fn format_discord_gateway_chip() -> String {
    let ready = crate::discord::discord_bot_gateway_ready();
    let token = crate::discord::discord_bot_token_configured();
    let (ready_n, resume_n, disc_n) = crate::discord::discord_gateway_reconnect_stats();
    let stage = crate::discord::discord_last_shard_stage()
        .map(|s| format!("{:?}", s))
        .unwrap_or_else(|| "unknown".into());
    let ready_ago = crate::discord::discord_last_ready_at()
        .map(|t| format!("{}s ago", t.elapsed().as_secs()))
        .unwrap_or_else(|| "never".into());
    let stage_lower = stage.to_lowercase();
    if !token {
        return "**Discord** · Offline · no token in Keychain".to_string();
    }
    if stage_lower == "disconnected" || !ready {
        let mut line = format!("**Discord** · Offline · last Ready {ready_ago}");
        if disc_n > 0 {
            line.push_str(&format!(" · disc×{disc_n}"));
        }
        if !stage.is_empty() && stage != "unknown" {
            line.push_str(&format!(" · stage={stage}"));
        }
        return line;
    }
    let mut line = if resume_n > 0 && disc_n == 0 && stage_lower == "resuming" {
        format!("**Discord** · Resuming · {ready_ago}")
    } else {
        format!("**Discord** · Ready · {ready_ago}")
    };
    if disc_n > 0 {
        line.push_str(&format!(" · disc×{disc_n}"));
    } else if resume_n > 0 && !line.contains("Resuming") {
        line.push_str(&format!(" · res×{resume_n}"));
    }
    line.push_str(&format!(" · stage={stage} · ready×{ready_n}"));
    line
}

/// True for focused Ollama Ready/Offline asks (`/ollama` · menu-bar ✕ / AI Chat glance parity) —
/// not pull/list/chat/API free-form.
pub fn looks_like_ollama_ready_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    // Pull / list / chat / configure stay with agent or OLLAMA_API pre-route.
    if n.contains("pull")
        || n.contains("push")
        || n.contains("list model")
        || n.contains("list models")
        || n.contains("unload")
        || n.contains("load model")
        || n.contains("embed")
        || n.contains("api")
        || n.contains("chat with")
        || n.contains("ask ollama")
        || n.contains("ask the model")
        || n.contains("install")
        || n.contains("download")
        || n.contains("change model")
        || n.contains("set model")
        || n.contains("set url")
        || n.contains("configure")
        || n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains(" ticket")
        || n.contains("redmine")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/ollama"
            | "/llm"
            | "ollama"
            | "llm"
            | "ollama status"
            | "llm status"
            | "ollama ready"
            | "ollama offline"
            | "llm ready"
            | "llm offline"
            | "show ollama"
            | "list ollama"
            | "is ollama ready"
            | "is ollama online"
            | "is ollama connected"
            | "is ollama offline"
            | "is ollama down"
            | "is the llm ready"
            | "is the llm online"
            | "is the llm connected"
            | "is the llm offline"
            | "is the llm down"
            | "ollama connection"
            | "llm connection"
            | "ollama circuit"
            | "how's ollama"
            | "hows ollama"
            | "how's the llm"
            | "hows the llm"
            | "how's ollama doing"
            | "hows ollama doing"
    )
}

fn shorten_ollama_endpoint_for_chip(endpoint: &str) -> String {
    let mut s = endpoint.trim().trim_end_matches('/').to_string();
    for prefix in ["https://", "http://"] {
        if let Some(rest) = s.strip_prefix(prefix) {
            s = rest.to_string();
            break;
        }
    }
    if s.chars().count() > 40 {
        s.chars().take(37).collect::<String>() + "…"
    } else {
        s
    }
}

/// Zero-LLM Ollama Ready / Offline chip (menu-bar Ollama ✕ + AI Chat model-glance parity).
pub fn format_ollama_ready_chip() -> String {
    let cfg = crate::commands::ollama_config::get_ollama_config();
    let circuit_open = crate::ollama::ollama_http_circuit_is_open_for_menu();
    let Some(c) = cfg else {
        return "**Ollama** · Not set · configure URL".to_string();
    };
    let ep = shorten_ollama_endpoint_for_chip(&c.endpoint);
    let model = {
        let m = c.model.trim();
        if m.is_empty() {
            "no model".to_string()
        } else {
            m.to_string()
        }
    };
    if circuit_open {
        return format!("**Ollama** · Offline · circuit open · {model} · {ep}");
    }
    let mut line = format!("**Ollama** · Ready · {model} · {ep}");
    if let Some(backend) = c.detected_backend.as_deref() {
        if !backend.is_empty() && backend != "unknown" {
            line.push_str(&format!(" · {backend}"));
        }
    }
    line
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
    if looks_like_discord_gateway_request(content) {
        return Some(format_discord_gateway_chip());
    }
    if looks_like_ollama_ready_request(content) {
        return Some(format_ollama_ready_chip());
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
    if looks_like_disk_cleanup_request(content) {
        let filter = parse_disk_cleanup_list_filter(content);
        return Some(format_disk_cleanup_gateway(filter));
    }
    if looks_like_debug_log_request(content) {
        let filter = parse_debug_log_list_filter(content);
        return Some(format_debug_log_gateway(filter));
    }
    if looks_like_rings_request(content) {
        let filter = parse_rings_list_filter(content);
        return Some(format_rings_gateway(filter));
    }
    if looks_like_ring_chip_request(content) {
        if let Some(ask) = parse_ring_chip_ask(content) {
            return Some(format_ring_chip_gateway(ask));
        }
    }
    if looks_like_strip_chip_request(content) {
        if let Some(ask) = parse_strip_chip_ask(content) {
            return Some(format_strip_chip_gateway(ask));
        }
    }
    if looks_like_strip_request(content) {
        let filter = parse_strip_list_filter(content);
        return Some(format_strip_gateway(filter));
    }
    if looks_like_details_request(content) {
        let filter = parse_details_list_filter(content);
        return Some(format_details_gateway(filter));
    }
    if looks_like_processes_request(content) {
        let filter = parse_processes_list_filter(content);
        return Some(format_processes_gateway(filter));
    }
    if looks_like_perplexity_request(content) {
        let filter = parse_perplexity_list_filter(content);
        return Some(format_perplexity_gateway(filter));
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
• `/discord` — Discord Ready / Offline (Agent Ops glance; reconnect cues)\n\
• `/ollama` · `/llm` — Ollama Ready / Offline (menu-bar ✕ · AI Chat glance; circuit)\n\
• `/insights` · `/insights 7` — runs.jsonl report (+ optional day window)\n\
• `/failed` · `/failed 7` — recent failed turns from runs.jsonl\n\
• `/slow` · `/slow 7` — recent slow turns (≥{slow_ms} ms wall time)\n\
• `/instant` · `/lite` · `/direct` · `/instant 7` — recent instant-, lite-, or direct-lane turns\n\
• `/agents` · `/agents on` · `/agents off` — Agent Ops On/Off list\n\
• `/sessions` · `/sessions live` · `/sessions files` — Agent Ops Live/Files list\n\
• `/knowledge` · `/knowledge discord` · `/knowledge core` — Agent Ops Knowledge list\n\
• `/schedules` · `/schedules jobs` · `/schedules deliveries` · `/cron list` — Agent Ops Jobs/Deliveries list\n\
• `/monitors` · `/monitors up` · `/monitors down` · `/monitors slow` — External / Monitors list\n\
• `/disk` · `/disk on` · `/disk off` · `/disk reclaim` · `/disk big` · `/disk clean` — Disk Cleanup list\n\
• `/logs` · `/logs error` · `/logs warn` — Debug Log Error/Warn list\n\
• `/processes` · `/processes hot` · `/hot` · `/processes pinned` · `/pinned` — Top Processes Hot/Pinned list\n\
• `/rings` · `/rings hot` — CPU rings All/Hot list (menu-bar amber thresholds)\n\
• `/cpu` · `/gpu` · `/freq` · `/temp` — CPU · GPU · Freq · Temp ring chips\n\
• `/strip` · `/strip hot` · `/power` — power strip All/Hot list (menu-bar amber / attention cues)\n\
• `/battery` · `/bat` · `/heat` · `/thermal` · `/lpm` · `/ram` · `/ssd` · `/uptime` — power-strip Bat · Heat · LPM · RAM · SSD · Up chips\n\
• `/details` · `/details hot` · `/load` — Details Load · RAM · Up (Load≥4 · RAM≥85% hot)\n\
• `/perplexity` · `/perplexity top` · `/perplexity snippet` — last Perplexity Top/Snippet list\n\
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
    // `/disk` On/Off/Reclaim/Big/Clean operator asks (v0.1.710).
    if (q.contains("/disk")
        || q.contains("/cleanup")
        || q.contains("disk cleanup")
        || q.contains("cleanup status")
        || q.contains("list cleanup")
        || q.contains("cleanup scopes")
        || q.contains("reclaimable")
        || q.contains("disk reclaim")
        || q.contains("disk big")
        || q.contains("disk clean")
        || q.contains("cleanup reclaim")
        || q.contains("cleanup on")
        || q.contains("cleanup off")
        || q == "cleanup"
        || q == "reclaim"
        || q == "what's reclaimable"
        || q == "whats reclaimable"
        || q == "what is reclaimable")
        && !q.contains("why")
        && !q.contains("clean now")
        && !q.contains("run cleanup")
        && !q.contains("disk usage")
        && !q.contains("ssd")
        && !q.contains(" ticket")
        && !q.contains("redmine")
    {
        return true;
    }
    // `/logs` Error/Warn operator asks (v0.1.711).
    if (q.contains("/logs")
        || q.contains("/log")
        || q.contains("debug log")
        || q.contains("debug logs")
        || q.contains("show logs")
        || q.contains("list logs")
        || q.contains("log tail")
        || q.contains("tail logs")
        || q.contains("any errors")
        || q.contains("any error")
        || q.contains("any warnings")
        || q.contains("any warning")
        || q.contains("show errors")
        || q.contains("show warnings")
        || q.contains("list errors")
        || q.contains("list warnings")
        || q == "logs"
        || q == "log"
        || q == "error"
        || q == "errors"
        || q == "warn"
        || q == "warns"
        || q == "warning"
        || q == "warnings"
        || q == "what's wrong"
        || q == "whats wrong"
        || q == "what is wrong")
        && !q.contains("why")
        && !q.contains("fix")
        && !q.contains("explain")
        && !q.contains("clear log")
        && !q.contains(" ticket")
        && !q.contains("redmine")
    {
        return true;
    }
    // `/rings` Hot operator asks (v0.1.715).
    if (q.contains("/rings")
        || q.contains("cpu rings")
        || q.contains("ring gauges")
        || q.contains("ring gauge")
        || q.contains("metric rings")
        || q.contains("hot rings")
        || q.contains("rings hot")
        || q.contains("which rings are hot")
        || q.contains("which ring is hot")
        || q.contains("show hot rings")
        || q.contains("list hot rings")
        || q.contains("list rings")
        || q.contains("show rings")
        || q == "rings"
        || q == "what's hot on rings"
        || q == "whats hot on rings")
        && !q.contains("why")
        && !q.contains("process")
        && !q.contains(" ticket")
        && !q.contains("redmine")
    {
        return true;
    }
    // `/cpu` · `/gpu` · `/freq` · `/temp` chip asks (v0.1.720).
    if (q.contains("/cpu")
        || q.contains("/gpu")
        || q.contains("/freq")
        || q.contains("/frequency")
        || q.contains("/ghz")
        || q.contains("/temp")
        || q.contains("/temperature")
        || q.contains("cpu usage")
        || q.contains("cpu percent")
        || q.contains("gpu usage")
        || q.contains("gpu percent")
        || q.contains("cpu frequency")
        || q.contains("cpu temp")
        || q.contains("cpu temperature")
        || q == "cpu"
        || q == "gpu"
        || q == "freq"
        || q == "frequency"
        || q == "ghz"
        || q == "temp"
        || q == "temperature"
        || q == "what's the cpu"
        || q == "whats the cpu"
        || q == "what is the cpu"
        || q == "what's the gpu"
        || q == "whats the gpu"
        || q == "what is the gpu"
        || q == "what's the freq"
        || q == "whats the freq"
        || q == "what is the frequency"
        || q == "what's the temp"
        || q == "whats the temp"
        || q == "what is the temperature")
        && !q.contains("why")
        && !q.contains("process")
        && !q.contains("ring")
        && !q.contains("strip")
        && !q.contains("detail")
        && !q.contains(" ticket")
        && !q.contains("redmine")
    {
        return true;
    }
    // `/strip` Hot operator asks (v0.1.716).
    if (q.contains("/strip")
        || q.contains("/power")
        || q.contains("power strip")
        || q.contains("powerstrip")
        || q.contains("battery strip")
        || q.contains("hot strip")
        || q.contains("strip hot")
        || q.contains("which strip is hot")
        || q.contains("which chips are hot")
        || q.contains("show hot strip")
        || q.contains("list hot strip")
        || q.contains("list strip")
        || q.contains("show strip")
        || q == "strip"
        || q == "power"
        || q == "what's hot on strip"
        || q == "whats hot on strip"
        || q == "what's hot on the strip"
        || q == "whats hot on the strip")
        && !q.contains("why")
        && !q.contains("process")
        && !q.contains("ring")
        && !q.contains("cleanup")
        && !q.contains("clean up")
        && !q.contains(" ticket")
        && !q.contains("redmine")
    {
        return true;
    }
    // `/battery` · `/heat` · `/lpm` · `/ram` · `/ssd` · `/uptime` chip asks (v0.1.719–721).
    if (q.contains("/battery")
        || q.contains("/bat")
        || q.contains("/heat")
        || q.contains("/thermal")
        || q.contains("/lpm")
        || q.contains("/ram")
        || q.contains("/ssd")
        || q.contains("/uptime")
        || q.contains("/up")
        || q.contains("/memory")
        || q.contains("/mem")
        || q.contains("battery level")
        || q.contains("battery percent")
        || q.contains("thermal state")
        || q.contains("thermal pressure")
        || q.contains("low power mode")
        || q.contains("low-power mode")
        || q.contains("ram percent")
        || q.contains("memory percent")
        || q.contains("disk usage")
        || q.contains("ssd usage")
        || q.contains("system uptime")
        || q == "battery"
        || q == "bat"
        || q == "heat"
        || q == "thermal"
        || q == "lpm"
        || q == "low power"
        || q == "ram"
        || q == "memory"
        || q == "mem"
        || q == "ssd"
        || q == "uptime"
        || q == "what's the battery"
        || q == "whats the battery"
        || q == "what is the battery"
        || q == "what's the heat"
        || q == "whats the heat"
        || q == "what is the heat"
        || q == "what's the ram"
        || q == "whats the ram"
        || q == "what is the ram"
        || q == "what's the ssd"
        || q == "whats the ssd"
        || q == "what is the ssd"
        || q == "what's the uptime"
        || q == "whats the uptime"
        || q == "what is the uptime"
        || q == "is lpm on"
        || q == "is low power mode on")
        && !q.contains("why")
        && !q.contains("process")
        && !q.contains("strip")
        && !q.contains("cleanup")
        && !q.contains("detail")
        && !q.contains(" ticket")
        && !q.contains("redmine")
    {
        return true;
    }
    // `/discord` gateway Ready/Offline chip asks (v0.1.722).
    if (q.contains("/discord")
        || q == "discord"
        || q.contains("discord status")
        || q.contains("discord gateway")
        || q.contains("discord ready")
        || q.contains("discord offline")
        || q.contains("gateway status")
        || q.contains("gateway ready")
        || q.contains("bot gateway")
        || q.contains("is discord ready")
        || q.contains("is discord online")
        || q.contains("is discord connected")
        || q.contains("is discord offline")
        || q.contains("is the bot ready")
        || q.contains("is the bot connected")
        || q.contains("is the gateway ready")
        || q.contains("is the gateway connected")
        || q.contains("discord connection")
        || q.contains("discord reconnect")
        || q.contains("discord disconnects")
        || q == "how's discord"
        || q == "hows discord"
        || q == "how's the discord"
        || q == "hows the discord"
        || q == "how's the gateway"
        || q == "hows the gateway")
        && !q.contains("knowledge")
        && !q.contains("memory")
        && !q.contains("post")
        && !q.contains("send")
        && !q.contains("message")
        && !q.contains("why")
        && !q.contains(" ticket")
        && !q.contains("redmine")
    {
        return true;
    }
    // `/ollama` · `/llm` Ready/Offline chip asks (v0.1.723).
    if (q.contains("/ollama")
        || q.contains("/llm")
        || q == "ollama"
        || q == "llm"
        || q.contains("ollama status")
        || q.contains("llm status")
        || q.contains("ollama ready")
        || q.contains("ollama offline")
        || q.contains("llm ready")
        || q.contains("llm offline")
        || q.contains("is ollama ready")
        || q.contains("is ollama online")
        || q.contains("is ollama connected")
        || q.contains("is ollama offline")
        || q.contains("is ollama down")
        || q.contains("is the llm ready")
        || q.contains("is the llm online")
        || q.contains("is the llm connected")
        || q.contains("is the llm offline")
        || q.contains("is the llm down")
        || q.contains("ollama connection")
        || q.contains("llm connection")
        || q.contains("ollama circuit")
        || q == "how's ollama"
        || q == "hows ollama"
        || q == "how's the llm"
        || q == "hows the llm"
        || q == "how's ollama doing"
        || q == "hows ollama doing")
        && !q.contains("pull")
        && !q.contains("list model")
        && !q.contains("chat with")
        && !q.contains("ask ollama")
        && !q.contains("install")
        && !q.contains("why")
        && !q.contains(" ticket")
        && !q.contains("redmine")
    {
        return true;
    }
    // `/details` · `/load` operator asks (v0.1.718).
    if (q.contains("/details")
        || q.contains("/load")
        || q.contains("system details")
        || q.contains("cpu details")
        || q.contains("load average")
        || q.contains("load avg")
        || q.contains("system load")
        || q.contains("cpu load")
        || q.contains("hot details")
        || q.contains("details hot")
        || q.contains("which details are hot")
        || q.contains("which detail is hot")
        || q.contains("show hot details")
        || q.contains("list hot details")
        || q.contains("list details")
        || q.contains("show details")
        || q.contains("show load")
        || q.contains("list load")
        || q == "details"
        || q == "load"
        || q == "what's the load"
        || q == "whats the load"
        || q == "what is the load"
        || q == "what's hot on details"
        || q == "whats hot on details")
        && !q.contains("why")
        && !q.contains("process")
        && !q.contains("ring")
        && !q.contains("strip")
        && !q.contains("more detail")
        && !q.contains("full detail")
        && !q.contains(" ticket")
        && !q.contains("redmine")
    {
        return true;
    }
    // `/processes` Hot/Pinned operator asks (v0.1.712 / v0.1.714).
    if (q.contains("/processes")
        || q.contains("/process")
        || q.contains("/hot")
        || q.contains("/pinned")
        || q.contains("top processes")
        || q.contains("top process")
        || q.contains("list processes")
        || q.contains("show processes")
        || q.contains("hot processes")
        || q.contains("hot process")
        || q.contains("processes hot")
        || q.contains("pinned processes")
        || q.contains("processes pinned")
        || q.contains("which processes are hot")
        || q.contains("which process is hot")
        || q.contains("show pinned")
        || q.contains("list pinned")
        || q.contains("my pinned")
        || q.contains("favorite processes")
        || q == "processes"
        || q == "process list"
        || q == "hot"
        || q == "pinned"
        || q == "what's hot"
        || q == "whats hot"
        || q == "what is hot")
        && !q.contains("why")
        && !q.contains("kill")
        && !q.contains("force quit")
        && !q.contains("pin this")
        && !q.contains("unpin")
        && !q.contains(" ticket")
        && !q.contains("redmine")
    {
        return true;
    }
    // `/perplexity` Top/Snippet operator asks (v0.1.713).
    if (q.contains("/perplexity")
        || q.contains("last search")
        || q.contains("last perplexity")
        || q.contains("perplexity results")
        || q.contains("search results")
        || q.contains("top results")
        || q.contains("snippet results")
        || q.contains("results with snippets")
        || q == "perplexity"
        || q == "perplexity search"
        || q == "perplexity top"
        || q == "perplexity snippet"
        || q == "/top"
        || q == "/snippet"
        || q == "snippet"
        || q == "snippets")
        && !q.contains("why")
        && !q.contains("search for")
        && !q.contains("look up")
        && !q.starts_with("perplexity search ")
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
        let discord = try_operator_instant_reply("/discord").expect("discord");
        assert!(
            discord.to_lowercase().contains("discord"),
            "{discord}"
        );
        assert!(try_operator_instant_reply("is discord ready").is_some());
        assert!(try_operator_instant_reply("/knowledge discord").is_some()); // knowledge, not chip
        assert!(try_operator_instant_reply("post to discord").is_none());
        let ollama = try_operator_instant_reply("/ollama").expect("ollama");
        assert!(ollama.to_lowercase().contains("ollama"), "{ollama}");
        assert!(try_operator_instant_reply("is ollama ready").is_some());
        assert!(try_operator_instant_reply("/llm").is_some());
        assert!(try_operator_instant_reply("pull llama3").is_none());
        assert!(try_operator_instant_reply("chat with ollama about weather").is_none());
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
        let disk = try_operator_instant_reply("/disk").expect("disk");
        assert!(disk.to_lowercase().contains("disk cleanup") || disk.to_lowercase().contains("reclaim"));
        let disk_reclaim = try_operator_instant_reply("/disk reclaim").expect("disk reclaim");
        assert!(disk_reclaim.to_lowercase().contains("reclaim"));
        let logs = try_operator_instant_reply("/logs").expect("logs");
        assert!(logs.to_lowercase().contains("debug log"), "{logs}");
        let logs_err = try_operator_instant_reply("/logs error").expect("logs error");
        assert!(logs_err.to_lowercase().contains("error"), "{logs_err}");
        let processes = try_operator_instant_reply("/processes").expect("processes");
        assert!(
            processes.to_lowercase().contains("top processes"),
            "{processes}"
        );
        let processes_hot = try_operator_instant_reply("/processes hot").expect("processes hot");
        assert!(processes_hot.to_lowercase().contains("hot"), "{processes_hot}");
        let processes_pinned =
            try_operator_instant_reply("/processes pinned").expect("processes pinned");
        assert!(
            processes_pinned.to_lowercase().contains("pinned"),
            "{processes_pinned}"
        );
        let rings = try_operator_instant_reply("/rings").expect("rings");
        assert!(rings.to_lowercase().contains("cpu rings"), "{rings}");
        let rings_hot = try_operator_instant_reply("/rings hot").expect("rings hot");
        assert!(
            rings_hot.to_lowercase().contains("hot") || rings_hot.to_lowercase().contains("ring"),
            "{rings_hot}"
        );
        let cpu = try_operator_instant_reply("/cpu").expect("cpu");
        assert!(cpu.to_lowercase().contains("cpu"), "{cpu}");
        let gpu = try_operator_instant_reply("/gpu").expect("gpu");
        assert!(gpu.to_lowercase().contains("gpu"), "{gpu}");
        let freq = try_operator_instant_reply("/freq").expect("freq");
        assert!(freq.to_lowercase().contains("freq"), "{freq}");
        let temp = try_operator_instant_reply("/temp").expect("temp");
        assert!(temp.to_lowercase().contains("temp"), "{temp}");
        let strip = try_operator_instant_reply("/strip").expect("strip");
        assert!(
            strip.to_lowercase().contains("power strip"),
            "{strip}"
        );
        let strip_hot = try_operator_instant_reply("/strip hot").expect("strip hot");
        assert!(
            strip_hot.to_lowercase().contains("hot") || strip_hot.to_lowercase().contains("strip"),
            "{strip_hot}"
        );
        let bat = try_operator_instant_reply("/battery").expect("battery");
        assert!(bat.to_lowercase().contains("bat"), "{bat}");
        let heat = try_operator_instant_reply("/heat").expect("heat");
        assert!(heat.to_lowercase().contains("heat"), "{heat}");
        let lpm = try_operator_instant_reply("/lpm").expect("lpm");
        assert!(lpm.to_lowercase().contains("lpm"), "{lpm}");
        let ram = try_operator_instant_reply("/ram").expect("ram");
        assert!(ram.to_lowercase().contains("ram"), "{ram}");
        let ssd = try_operator_instant_reply("/ssd").expect("ssd");
        assert!(ssd.to_lowercase().contains("ssd"), "{ssd}");
        let uptime = try_operator_instant_reply("/uptime").expect("uptime");
        assert!(
            uptime.to_lowercase().contains("up"),
            "{uptime}"
        );
        let details = try_operator_instant_reply("/details").expect("details");
        assert!(
            details.to_lowercase().contains("details"),
            "{details}"
        );
        let details_hot = try_operator_instant_reply("/details hot").expect("details hot");
        assert!(
            details_hot.to_lowercase().contains("hot")
                || details_hot.to_lowercase().contains("details"),
            "{details_hot}"
        );
        let load = try_operator_instant_reply("/load").expect("load");
        assert!(
            load.to_lowercase().contains("details") || load.to_lowercase().contains("load"),
            "{load}"
        );
        let perplexity = try_operator_instant_reply("/perplexity").expect("perplexity");
        assert!(
            perplexity.to_lowercase().contains("perplexity"),
            "{perplexity}"
        );
        let perplexity_top = try_operator_instant_reply("/perplexity top").expect("perplexity top");
        assert!(
            perplexity_top.to_lowercase().contains("top")
                || perplexity_top.to_lowercase().contains("perplexity"),
            "{perplexity_top}"
        );
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
        assert!(try_operator_instant_reply("clean now").is_none());
        assert!(try_operator_instant_reply("disk cleanup").is_some());
        assert!(try_operator_instant_reply("disk usage").is_some()); // /ssd chip
        assert!(try_operator_instant_reply("why is there an error").is_none());
        assert!(try_operator_instant_reply("fix the error").is_none());
        assert!(try_operator_instant_reply("explain the warning").is_none());
        assert!(try_operator_instant_reply("kill that process").is_none());
        assert!(try_operator_instant_reply("force quit chrome").is_none());
        assert!(try_operator_instant_reply("pin this process").is_none());
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
    fn disk_cleanup_request_detected() {
        assert!(looks_like_disk_cleanup_request("/disk"));
        assert!(looks_like_disk_cleanup_request("disk cleanup"));
        assert!(looks_like_disk_cleanup_request("@Werner /cleanup"));
        assert!(looks_like_disk_cleanup_request("/disk on"));
        assert!(looks_like_disk_cleanup_request("/disk off"));
        assert!(looks_like_disk_cleanup_request("/disk reclaim"));
        assert!(looks_like_disk_cleanup_request("/disk big"));
        assert!(looks_like_disk_cleanup_request("/disk clean"));
        assert!(looks_like_disk_cleanup_request("what's reclaimable"));
        assert!(looks_like_disk_cleanup_request("enabled scopes"));
        assert!(looks_like_disk_cleanup_request("cleanup scopes"));
        assert!(!looks_like_disk_cleanup_request("clean now"));
        assert!(!looks_like_disk_cleanup_request("disk usage"));
        assert!(!looks_like_disk_cleanup_request("run cleanup"));
        assert!(!looks_like_disk_cleanup_request("why is cleanup slow"));
        assert_eq!(
            parse_disk_cleanup_list_filter("/disk"),
            DiskCleanupListFilter::All
        );
        assert_eq!(
            parse_disk_cleanup_list_filter("/disk on"),
            DiskCleanupListFilter::On
        );
        assert_eq!(
            parse_disk_cleanup_list_filter("disabled scopes"),
            DiskCleanupListFilter::Off
        );
        assert_eq!(
            parse_disk_cleanup_list_filter("what's reclaimable"),
            DiskCleanupListFilter::Reclaim
        );
        assert_eq!(
            parse_disk_cleanup_list_filter("/disk big"),
            DiskCleanupListFilter::Big
        );
        assert_eq!(
            parse_disk_cleanup_list_filter("clean categories"),
            DiskCleanupListFilter::Clean
        );
    }

    #[test]
    fn debug_log_request_and_filter() {
        assert!(looks_like_debug_log_request("/logs"));
        assert!(looks_like_debug_log_request("debug log"));
        assert!(looks_like_debug_log_request("@Werner /logs"));
        assert!(looks_like_debug_log_request("/logs error"));
        assert!(looks_like_debug_log_request("/logs warn"));
        assert!(looks_like_debug_log_request("any errors"));
        assert!(looks_like_debug_log_request("show warnings"));
        assert!(looks_like_debug_log_request("what's wrong"));
        assert!(!looks_like_debug_log_request("why is there an error"));
        assert!(!looks_like_debug_log_request("fix the error"));
        assert!(!looks_like_debug_log_request("explain the warning"));
        assert!(!looks_like_debug_log_request("clear log"));
        assert_eq!(
            parse_debug_log_list_filter("/logs"),
            DebugLogListFilter::All
        );
        assert_eq!(
            parse_debug_log_list_filter("/logs error"),
            DebugLogListFilter::Error
        );
        assert_eq!(
            parse_debug_log_list_filter("any errors"),
            DebugLogListFilter::Error
        );
        assert_eq!(
            parse_debug_log_list_filter("/logs warn"),
            DebugLogListFilter::Warn
        );
        assert_eq!(
            parse_debug_log_list_filter("show warnings"),
            DebugLogListFilter::Warn
        );
        assert_eq!(debug_log_line_kind("2026-08-29 ERROR boom"), "error");
        assert_eq!(debug_log_line_kind("2026-08-29 WARN soft"), "warn");
        assert_eq!(debug_log_line_kind("2026-08-29 INFO ok"), "other");
        assert_eq!(debug_log_line_kind("thread panicked at src"), "error");
    }

    #[test]
    fn debug_log_gateway_has_counts() {
        let report = format_debug_log_gateway(DebugLogListFilter::All);
        assert!(report.to_lowercase().contains("debug log"), "{report}");
        let err = format_debug_log_gateway(DebugLogListFilter::Error);
        assert!(err.to_lowercase().contains("error"), "{err}");
        let warn = format_debug_log_gateway(DebugLogListFilter::Warn);
        assert!(warn.to_lowercase().contains("warn"), "{warn}");
    }

    #[test]
    fn processes_request_and_filter() {
        assert!(looks_like_processes_request("/processes"));
        assert!(looks_like_processes_request("top processes"));
        assert!(looks_like_processes_request("@Werner /processes"));
        assert!(looks_like_processes_request("/processes hot"));
        assert!(looks_like_processes_request("/hot"));
        assert!(looks_like_processes_request("what's hot"));
        assert!(looks_like_processes_request("hot processes"));
        assert!(looks_like_processes_request("which processes are hot"));
        assert!(looks_like_processes_request("/processes pinned"));
        assert!(looks_like_processes_request("/pinned"));
        assert!(looks_like_processes_request("pinned processes"));
        assert!(looks_like_processes_request("show pinned"));
        assert!(!looks_like_processes_request("kill that process"));
        assert!(!looks_like_processes_request("force quit chrome"));
        assert!(!looks_like_processes_request("pin this process"));
        assert!(!looks_like_processes_request("unpin chrome"));
        assert!(!looks_like_processes_request("why is chrome hot"));
        assert_eq!(
            parse_processes_list_filter("/processes"),
            ProcessesListFilter::All
        );
        assert_eq!(
            parse_processes_list_filter("/processes hot"),
            ProcessesListFilter::Hot
        );
        assert_eq!(
            parse_processes_list_filter("what's hot"),
            ProcessesListFilter::Hot
        );
        assert_eq!(
            parse_processes_list_filter("/hot"),
            ProcessesListFilter::Hot
        );
        assert_eq!(
            parse_processes_list_filter("/processes pinned"),
            ProcessesListFilter::Pinned
        );
        assert_eq!(
            parse_processes_list_filter("pinned processes"),
            ProcessesListFilter::Pinned
        );
        assert_eq!(
            parse_processes_list_filter("/pinned"),
            ProcessesListFilter::Pinned
        );
        assert!(process_row_is_hot(&crate::metrics::ProcessUsage {
            name: "hot".into(),
            cpu: 20.0,
            gpu: 0.0,
            pid: 1,
            memory: 0,
        }));
        assert!(process_row_is_hot(&crate::metrics::ProcessUsage {
            name: "gpu".into(),
            cpu: 0.0,
            gpu: 16.0,
            pid: 2,
            memory: 0,
        }));
        assert!(process_row_is_hot(&crate::metrics::ProcessUsage {
            name: "ram".into(),
            cpu: 0.0,
            gpu: 0.0,
            pid: 3,
            memory: OPS_PROCESS_HOT_RAM_BYTES,
        }));
        assert!(!process_row_is_hot(&crate::metrics::ProcessUsage {
            name: "cool".into(),
            cpu: 1.0,
            gpu: 0.0,
            pid: 4,
            memory: 1024,
        }));
    }

    #[test]
    fn processes_gateway_has_title() {
        let report = format_processes_gateway(ProcessesListFilter::All);
        assert!(report.to_lowercase().contains("top processes"), "{report}");
        let hot = format_processes_gateway(ProcessesListFilter::Hot);
        assert!(hot.to_lowercase().contains("hot"), "{hot}");
        let pinned = format_processes_gateway(ProcessesListFilter::Pinned);
        assert!(pinned.to_lowercase().contains("pinned"), "{pinned}");
    }

    #[test]
    fn rings_request_and_filter() {
        assert!(looks_like_rings_request("/rings"));
        assert!(looks_like_rings_request("cpu rings"));
        assert!(looks_like_rings_request("@Werner /rings"));
        assert!(looks_like_rings_request("/rings hot"));
        assert!(looks_like_rings_request("hot rings"));
        assert!(looks_like_rings_request("which rings are hot"));
        assert!(looks_like_rings_request("show hot rings"));
        assert!(!looks_like_rings_request("/hot"));
        assert!(!looks_like_rings_request("what's hot"));
        assert!(!looks_like_rings_request("hot processes"));
        assert!(!looks_like_rings_request("why are rings hot"));
        assert_eq!(parse_rings_list_filter("/rings"), RingsListFilter::All);
        assert_eq!(parse_rings_list_filter("/rings hot"), RingsListFilter::Hot);
        assert_eq!(parse_rings_list_filter("hot rings"), RingsListFilter::Hot);
        assert_eq!(
            parse_rings_list_filter("which rings are hot"),
            RingsListFilter::Hot
        );
        assert!(ring_cpu_is_hot(OPS_RING_HOT_CPU_PCT));
        assert!(ring_gpu_is_hot(OPS_RING_HOT_GPU_PCT));
        assert!(ring_freq_is_hot(OPS_RING_HOT_FREQ_GHZ));
        assert!(ring_temp_is_hot(OPS_RING_HOT_TEMP_C));
        assert!(!ring_cpu_is_hot(OPS_RING_HOT_CPU_PCT - 1.0));
    }

    #[test]
    fn rings_gateway_has_title() {
        let report = format_rings_gateway(RingsListFilter::All);
        assert!(report.to_lowercase().contains("cpu rings"), "{report}");
        let hot = format_rings_gateway(RingsListFilter::Hot);
        assert!(hot.to_lowercase().contains("hot"), "{hot}");
    }

    #[test]
    fn ring_chip_request_and_format() {
        assert_eq!(parse_ring_chip_ask("/cpu"), Some(RingChipAsk::Cpu));
        assert_eq!(
            parse_ring_chip_ask("what's the cpu"),
            Some(RingChipAsk::Cpu)
        );
        assert_eq!(parse_ring_chip_ask("cpu usage"), Some(RingChipAsk::Cpu));
        assert_eq!(parse_ring_chip_ask("/gpu"), Some(RingChipAsk::Gpu));
        assert_eq!(
            parse_ring_chip_ask("what's the gpu"),
            Some(RingChipAsk::Gpu)
        );
        assert_eq!(parse_ring_chip_ask("/freq"), Some(RingChipAsk::Freq));
        assert_eq!(parse_ring_chip_ask("/ghz"), Some(RingChipAsk::Freq));
        assert_eq!(
            parse_ring_chip_ask("cpu frequency"),
            Some(RingChipAsk::Freq)
        );
        assert_eq!(parse_ring_chip_ask("/temp"), Some(RingChipAsk::Temp));
        assert_eq!(
            parse_ring_chip_ask("what's the temperature"),
            Some(RingChipAsk::Temp)
        );
        assert!(parse_ring_chip_ask("/rings").is_none());
        assert!(parse_ring_chip_ask("cpu rings").is_none());
        assert!(parse_ring_chip_ask("hot rings").is_none());
        assert!(parse_ring_chip_ask("why is the cpu hot").is_none());
        assert!(parse_ring_chip_ask("cpu details").is_none());
        assert!(looks_like_ring_chip_request("/temperature"));
        let cpu = format_ring_chip_gateway(RingChipAsk::Cpu);
        assert!(cpu.to_lowercase().contains("cpu"), "{cpu}");
        let gpu = format_ring_chip_gateway(RingChipAsk::Gpu);
        assert!(gpu.to_lowercase().contains("gpu"), "{gpu}");
        let freq = format_ring_chip_gateway(RingChipAsk::Freq);
        assert!(freq.to_lowercase().contains("freq"), "{freq}");
        let temp = format_ring_chip_gateway(RingChipAsk::Temp);
        assert!(temp.to_lowercase().contains("temp"), "{temp}");
    }

    #[test]
    fn strip_request_and_filter() {
        assert!(looks_like_strip_request("/strip"));
        assert!(looks_like_strip_request("power strip"));
        assert!(looks_like_strip_request("@Werner /strip"));
        assert!(looks_like_strip_request("/strip hot"));
        assert!(looks_like_strip_request("hot strip"));
        assert!(looks_like_strip_request("which strip is hot"));
        assert!(looks_like_strip_request("/power"));
        assert!(!looks_like_strip_request("/hot"));
        assert!(!looks_like_strip_request("what's hot"));
        assert!(!looks_like_strip_request("/rings"));
        assert!(!looks_like_strip_request("hot rings"));
        assert!(!looks_like_strip_request("why is the strip hot"));
        assert!(!looks_like_strip_request("disk cleanup"));
        assert_eq!(parse_strip_list_filter("/strip"), StripListFilter::All);
        assert_eq!(parse_strip_list_filter("/strip hot"), StripListFilter::Hot);
        assert_eq!(parse_strip_list_filter("hot strip"), StripListFilter::Hot);
        assert_eq!(
            parse_strip_list_filter("what's hot on the strip"),
            StripListFilter::Hot
        );
        assert!(strip_heat_is_attention("Fair"));
        assert!(strip_heat_is_attention("Serious"));
        assert!(!strip_heat_is_attention("Nominal"));
    }

    #[test]
    fn strip_chip_request_and_format() {
        assert_eq!(
            parse_strip_chip_ask("/battery"),
            Some(StripChipAsk::Battery)
        );
        assert_eq!(
            parse_strip_chip_ask("what's the battery"),
            Some(StripChipAsk::Battery)
        );
        assert_eq!(parse_strip_chip_ask("/bat"), Some(StripChipAsk::Battery));
        assert_eq!(parse_strip_chip_ask("/heat"), Some(StripChipAsk::Heat));
        assert_eq!(
            parse_strip_chip_ask("thermal state"),
            Some(StripChipAsk::Heat)
        );
        assert_eq!(
            parse_strip_chip_ask("what's the heat"),
            Some(StripChipAsk::Heat)
        );
        assert_eq!(parse_strip_chip_ask("/lpm"), Some(StripChipAsk::Lpm));
        assert_eq!(
            parse_strip_chip_ask("low power mode"),
            Some(StripChipAsk::Lpm)
        );
        assert_eq!(
            parse_strip_chip_ask("is lpm on"),
            Some(StripChipAsk::Lpm)
        );
        assert_eq!(parse_strip_chip_ask("/ram"), Some(StripChipAsk::Ram));
        assert_eq!(
            parse_strip_chip_ask("what's the ram"),
            Some(StripChipAsk::Ram)
        );
        assert_eq!(parse_strip_chip_ask("memory"), Some(StripChipAsk::Ram));
        assert_eq!(parse_strip_chip_ask("/ssd"), Some(StripChipAsk::Ssd));
        assert_eq!(
            parse_strip_chip_ask("disk usage"),
            Some(StripChipAsk::Ssd)
        );
        assert_eq!(
            parse_strip_chip_ask("/uptime"),
            Some(StripChipAsk::Uptime)
        );
        assert_eq!(
            parse_strip_chip_ask("system uptime"),
            Some(StripChipAsk::Uptime)
        );
        assert_eq!(parse_strip_chip_ask("/up"), Some(StripChipAsk::Uptime));
        assert!(parse_strip_chip_ask("battery strip").is_none());
        assert!(parse_strip_chip_ask("/strip").is_none());
        assert!(parse_strip_chip_ask("/power").is_none());
        assert!(parse_strip_chip_ask("/disk").is_none());
        assert!(parse_strip_chip_ask("disk cleanup").is_none());
        assert!(parse_strip_chip_ask("/details").is_none());
        assert!(parse_strip_chip_ask("what's hot").is_none());
        assert!(parse_strip_chip_ask("why is the battery low").is_none());
        assert!(looks_like_strip_chip_request("/thermal"));
        let heat = format_strip_chip_gateway(StripChipAsk::Heat);
        assert!(heat.to_lowercase().contains("heat"), "{heat}");
        let lpm = format_strip_chip_gateway(StripChipAsk::Lpm);
        assert!(lpm.to_lowercase().contains("lpm"), "{lpm}");
        let bat = format_strip_chip_gateway(StripChipAsk::Battery);
        assert!(bat.to_lowercase().contains("bat"), "{bat}");
        let ram = format_strip_chip_gateway(StripChipAsk::Ram);
        assert!(ram.to_lowercase().contains("ram"), "{ram}");
        let ssd = format_strip_chip_gateway(StripChipAsk::Ssd);
        assert!(ssd.to_lowercase().contains("ssd"), "{ssd}");
        let up = format_strip_chip_gateway(StripChipAsk::Uptime);
        assert!(up.to_lowercase().contains("up"), "{up}");
    }

    #[test]
    fn strip_gateway_has_title() {
        let report = format_strip_gateway(StripListFilter::All);
        assert!(report.to_lowercase().contains("power strip"), "{report}");
        let hot = format_strip_gateway(StripListFilter::Hot);
        assert!(hot.to_lowercase().contains("hot"), "{hot}");
    }

    #[test]
    fn details_request_and_filter() {
        assert!(looks_like_details_request("/details"));
        assert!(looks_like_details_request("system details"));
        assert!(looks_like_details_request("@Werner /details"));
        assert!(looks_like_details_request("/details hot"));
        assert!(looks_like_details_request("hot details"));
        assert!(looks_like_details_request("/load"));
        assert!(looks_like_details_request("load average"));
        assert!(looks_like_details_request("what's the load"));
        assert!(!looks_like_details_request("/hot"));
        assert!(!looks_like_details_request("what's hot"));
        assert!(!looks_like_details_request("/strip"));
        assert!(!looks_like_details_request("/rings"));
        assert!(!looks_like_details_request("process details"));
        assert!(!looks_like_details_request("more details about weather"));
        assert!(!looks_like_details_request("why is the load high"));
        assert_eq!(
            parse_details_list_filter("/details"),
            DetailsListFilter::All
        );
        assert_eq!(
            parse_details_list_filter("/details hot"),
            DetailsListFilter::Hot
        );
        assert_eq!(
            parse_details_list_filter("hot details"),
            DetailsListFilter::Hot
        );
        assert!(details_load_is_hot(OPS_DETAILS_LOAD_HOT));
        assert!(!details_load_is_hot(OPS_DETAILS_LOAD_HOT - 0.1));
        assert!(details_ram_is_hot(OPS_STRIP_RAM_HOT_PCT));
        assert!(!details_ram_is_hot(OPS_STRIP_RAM_HOT_PCT - 1.0));
    }

    #[test]
    fn details_gateway_has_title() {
        let report = format_details_gateway(DetailsListFilter::All);
        assert!(report.to_lowercase().contains("details"), "{report}");
        assert!(report.to_lowercase().contains("load"), "{report}");
        let hot = format_details_gateway(DetailsListFilter::Hot);
        assert!(hot.to_lowercase().contains("hot"), "{hot}");
    }

    #[test]
    fn perplexity_request_and_filter() {
        assert!(looks_like_perplexity_request("/perplexity"));
        assert!(looks_like_perplexity_request("last search"));
        assert!(looks_like_perplexity_request("@Werner /perplexity"));
        assert!(looks_like_perplexity_request("/perplexity top"));
        assert!(looks_like_perplexity_request("top results"));
        assert!(looks_like_perplexity_request("/perplexity snippet"));
        assert!(looks_like_perplexity_request("snippet results"));
        assert!(looks_like_perplexity_request("results with snippets"));
        assert!(!looks_like_perplexity_request("search for barcelona"));
        assert!(!looks_like_perplexity_request("perplexity search weather"));
        assert!(!looks_like_perplexity_request("look up the news"));
        assert!(!looks_like_perplexity_request("why is search slow"));
        assert_eq!(
            parse_perplexity_list_filter("/perplexity"),
            PerplexityListFilter::All
        );
        assert_eq!(
            parse_perplexity_list_filter("/perplexity top"),
            PerplexityListFilter::Top
        );
        assert_eq!(
            parse_perplexity_list_filter("top results"),
            PerplexityListFilter::Top
        );
        assert_eq!(
            parse_perplexity_list_filter("/perplexity snippet"),
            PerplexityListFilter::Snippet
        );
        assert_eq!(
            parse_perplexity_list_filter("snippet results"),
            PerplexityListFilter::Snippet
        );
    }

    #[test]
    fn perplexity_gateway_has_title() {
        let report = format_perplexity_gateway(PerplexityListFilter::All);
        assert!(report.to_lowercase().contains("perplexity"), "{report}");
        let top = format_perplexity_gateway(PerplexityListFilter::Top);
        assert!(top.to_lowercase().contains("top") || top.to_lowercase().contains("perplexity"), "{top}");
        let snip = format_perplexity_gateway(PerplexityListFilter::Snippet);
        assert!(
            snip.to_lowercase().contains("snippet") || snip.to_lowercase().contains("perplexity"),
            "{snip}"
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
    fn discord_gateway_request_detected() {
        assert!(looks_like_discord_gateway_request("/discord"));
        assert!(looks_like_discord_gateway_request("discord"));
        assert!(looks_like_discord_gateway_request("discord status"));
        assert!(looks_like_discord_gateway_request("is discord ready"));
        assert!(looks_like_discord_gateway_request("gateway status"));
        assert!(looks_like_discord_gateway_request("how's discord"));
        assert!(!looks_like_discord_gateway_request("/knowledge discord"));
        assert!(!looks_like_discord_gateway_request("discord knowledge"));
        assert!(!looks_like_discord_gateway_request("post to discord"));
        assert!(!looks_like_discord_gateway_request("send a discord message"));
        assert!(!looks_like_discord_gateway_request("why is discord offline"));
        let chip = format_discord_gateway_chip();
        assert!(chip.to_lowercase().contains("discord"), "{chip}");
    }

    #[test]
    fn ollama_ready_request_detected() {
        assert!(looks_like_ollama_ready_request("/ollama"));
        assert!(looks_like_ollama_ready_request("/llm"));
        assert!(looks_like_ollama_ready_request("ollama"));
        assert!(looks_like_ollama_ready_request("ollama status"));
        assert!(looks_like_ollama_ready_request("is ollama ready"));
        assert!(looks_like_ollama_ready_request("how's ollama"));
        assert!(!looks_like_ollama_ready_request("pull llama3"));
        assert!(!looks_like_ollama_ready_request("list models"));
        assert!(!looks_like_ollama_ready_request("chat with ollama about weather"));
        assert!(!looks_like_ollama_ready_request("why is ollama offline"));
        assert!(!looks_like_ollama_ready_request("install ollama"));
        let chip = format_ollama_ready_chip();
        assert!(chip.to_lowercase().contains("ollama"), "{chip}");
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
        assert!(report.contains("/discord"), "{report}");
        assert!(report.contains("/ollama"), "{report}");
        assert!(report.contains("/llm"), "{report}");
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
        assert!(report.contains("/disk"), "{report}");
        assert!(report.contains("/disk reclaim"), "{report}");
        assert!(report.contains("/logs"), "{report}");
        assert!(report.contains("/logs error"), "{report}");
        assert!(report.contains("/processes"), "{report}");
        assert!(report.contains("/processes hot"), "{report}");
        assert!(report.contains("/processes pinned"), "{report}");
        assert!(report.contains("/rings"), "{report}");
        assert!(report.contains("/rings hot"), "{report}");
        assert!(report.contains("/cpu"), "{report}");
        assert!(report.contains("/gpu"), "{report}");
        assert!(report.contains("/freq"), "{report}");
        assert!(report.contains("/temp"), "{report}");
        assert!(report.contains("/strip"), "{report}");
        assert!(report.contains("/strip hot"), "{report}");
        assert!(report.contains("/battery"), "{report}");
        assert!(report.contains("/heat"), "{report}");
        assert!(report.contains("/lpm"), "{report}");
        assert!(report.contains("/ram"), "{report}");
        assert!(report.contains("/ssd"), "{report}");
        assert!(report.contains("/uptime"), "{report}");
        assert!(report.contains("/details"), "{report}");
        assert!(report.contains("/details hot"), "{report}");
        assert!(report.contains("/load"), "{report}");
        assert!(report.contains("/hot"), "{report}");
        assert!(report.contains("/pinned"), "{report}");
        assert!(report.contains("/perplexity"), "{report}");
        assert!(report.contains("/perplexity top"), "{report}");
        assert!(report.contains("/perplexity snippet"), "{report}");
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
    fn disk_cleanup_gateway_has_counts() {
        let report = format_disk_cleanup_gateway(DiskCleanupListFilter::All);
        assert!(
            report.to_lowercase().contains("disk cleanup"),
            "{report}"
        );
        let reclaim = format_disk_cleanup_gateway(DiskCleanupListFilter::Reclaim);
        assert!(reclaim.to_lowercase().contains("reclaim"), "{reclaim}");
        let on = format_disk_cleanup_gateway(DiskCleanupListFilter::On);
        assert!(on.to_lowercase().contains("on"), "{on}");
        let big = format_disk_cleanup_gateway(DiskCleanupListFilter::Big);
        assert!(big.to_lowercase().contains("big"), "{big}");
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
