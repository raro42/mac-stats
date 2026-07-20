//! Dashboard "Agent Ops" APIs — agents overview helpers, sessions, memory, runs.
//! OpenClaw-shaped ops surface over ~/.mac-stats data (Hermes mental model).

use crate::config::Config;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize)]
pub struct LiveSessionSummary {
    pub source: String,
    pub session_id: u64,
    pub message_count: usize,
    pub last_activity: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionFileSummary {
    pub name: String,
    pub path: String,
    pub source_hint: String,
    pub slug: String,
    pub modified_ms: u64,
    pub size_bytes: u64,
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
    pub by_lane: Vec<(String, usize)>,
    pub by_tool: Vec<(String, usize)>,
    pub candidates: Vec<RunInsightCandidate>,
    pub slowest: Vec<RunTurnSummary>,
    pub recent: Vec<RunTurnSummary>,
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
        })
        .collect();
    rows.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));
    rows
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
        rows.push(SessionFileSummary {
            name,
            path: path.display().to_string(),
            source_hint,
            slug,
            modified_ms: file_mtime_ms(&meta),
            size_bytes: meta.len(),
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
    let path = crate::commands::run_telemetry::runs_jsonl_path();
    let lim = limit.clamp(1, 200) as usize;
    let empty = RunsInsights {
        turns: 0,
        ok_count: 0,
        fail_count: 0,
        p50_ms: 0,
        mean_ms: 0,
        max_ms: 0,
        by_lane: vec![],
        by_tool: vec![],
        candidates: vec![],
        slowest: vec![],
        recent: vec![],
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
        recent.push(RunTurnSummary {
            ts: v
                .get("ts")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            lane,
            wall_ms: wall,
            tools,
            question_preview,
            ok,
            request_id,
        });
    }

    let turns = walls.len();
    let max_ms = walls.iter().copied().max().unwrap_or(0);
    let mean_ms = if turns == 0 {
        0
    } else {
        walls.iter().sum::<u64>() / turns as u64
    };
    let mut sorted = walls.clone();
    sorted.sort_unstable();
    let p50_ms = if sorted.is_empty() {
        0
    } else {
        sorted[sorted.len() / 2]
    };

    let mut slowest = recent.clone();
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
        by_lane,
        by_tool,
        candidates,
        slowest,
        recent,
    }
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
    if insights.turns == 0 {
        return "No turns in `~/.mac-stats/runs.jsonl` yet.".into();
    }
    let mut lines = vec![
        "**mac-stats insights** (runs.jsonl)".to_string(),
        format!(
            "Turns: **{}** · ok {} · fail {} · p50 **{}** ms · mean {} · max {}",
            insights.turns,
            insights.ok_count,
            insights.fail_count,
            insights.p50_ms,
            insights.mean_ms,
            insights.max_ms
        ),
        crate::discord::format_discord_gateway_insights_line(),
    ];
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

/// True for `/insights` / `insights` (Hermes parity).
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
        .trim();
    matches!(
        n,
        "insights"
            | "/insights"
            | "show insights"
            | "usage insights"
            | "run insights"
            | "agent insights"
    ) || n.starts_with("/insights ")
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
    fn insights_request_detected() {
        assert!(looks_like_insights_request("/insights"));
        assert!(looks_like_insights_request("insights"));
        assert!(looks_like_insights_request("@Werner insights"));
        assert!(!looks_like_insights_request("any insights on weather?"));
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
}
