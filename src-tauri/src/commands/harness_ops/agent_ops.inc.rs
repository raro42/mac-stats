// included by gateway.rs — agent_ops
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
    /// Digester stale / already-shipped hints (ignored by overnight fuel).
    pub digest_stale_hints: Vec<String>,
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


pub(super) fn file_mtime_ms(meta: &fs::Metadata) -> u64 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}


pub fn parse_session_filename(name: &str) -> (String, String) {
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
        .trim()
        .trim_start_matches("can you")
        .trim()
        .trim_start_matches("could you")
        .trim()
        .trim_start_matches("show me")
        .trim()
        .trim_start_matches("show")
        .trim()
        .trim_start_matches("view")
        .trim()
        .trim_start_matches("see")
        .trim()
        .trim_start_matches("open")
        .trim()
        .trim_start_matches("list")
        .trim()
        .trim_start_matches("the")
        .trim()
        .trim_end_matches('?')
        .trim();
    let rest = if let Some(r) = n.strip_prefix("/insights") {
        r.trim()
    } else if let Some(r) = n.strip_prefix("insights") {
        r.trim()
    } else if let Some(r) = n.strip_prefix("usage insights") {
        r.trim()
    } else if let Some(r) = n.strip_prefix("usage analytics") {
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
    } else if let Some(r) = n.strip_prefix("failed") {
        r.trim()
    } else if let Some(r) = n.strip_prefix("failures") {
        r.trim()
    } else if let Some(r) = n.strip_prefix("slow") {
        r.trim()
    } else if let Some(r) = n.strip_prefix("instant") {
        r.trim()
    } else if let Some(r) = n.strip_prefix("direct") {
        r.trim()
    } else if let Some(r) = n.strip_prefix("lite") {
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
        digest_stale_hints: digest.stale_hints.clone(),
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
        digest_stale_hints: digest.stale_hints,
        digest_source: digest.source,
        process_uptime_secs: crate::state::process_uptime_secs(),
        window_days,
    }
}


pub(super) fn digest_json_path() -> PathBuf {
    if let Ok(p) = std::env::var("MAC_STATS_DIGEST_JSON") {
        let t = p.trim();
        if !t.is_empty() {
            return PathBuf::from(t);
        }
    }
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


pub(super) fn digest_md_path() -> PathBuf {
    let json = digest_json_path();
    if json.file_name().and_then(|n| n.to_str()) == Some("latest.json") {
        return json.with_extension("md");
    }
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
            .join(".mac-stats")
            .join("improvements")
            .join("latest.md")
    } else {
        std::env::temp_dir()
            .join("mac-stats-improvements")
            .join("latest.md")
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
pub(super) fn digest_script_candidates() -> Vec<PathBuf> {
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


pub fn parse_run_ts(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
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


pub(super) fn shipped_cutoffs() -> (
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


pub(super) fn is_stale_shipped_candidate(
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


pub(super) fn hint_for_run(rec: &serde_json::Value) -> Option<String> {
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
pub(super) fn write_digest_native(days: i64) -> Result<DigestSummary, String> {
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

    // Build summary from this write — do not re-read shared latest.json (python
    // digester / parallel tests can race and flip source to "python").
    Ok(DigestSummary {
        open_count: open.len(),
        stale_count: stale.len(),
        turns,
        generated_at: generated,
        open_hints: open
            .iter()
            .filter_map(|item| {
                item.get("hint")
                    .and_then(|h| h.as_str())
                    .map(|s| s.to_string())
            })
            .take(5)
            .collect(),
        stale_hints: stale
            .iter()
            .filter_map(|item| {
                item.get("hint")
                    .and_then(|h| h.as_str())
                    .map(|s| s.to_string())
            })
            .take(5)
            .collect(),
        path: json_path.display().to_string(),
        source: "rust-native".to_string(),
    })
}


/// True for read-only digest open/candidate asks — cached summary only, no digester spawn.
/// View/see/show me/open/list-the NL (v0.1.1046). Exact `open digest` / `open the digest`
/// only for open. After normalize strips `show me` / `show`, prefer `the digest` so
/// `show me the digest` stays read-only (bare `show me digest` → `digest` still refreshes).
/// Does not steal `/digest` refresh, path/size/age, or digester spawn.
pub fn looks_like_digest_open_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 64 || n.contains(" this ") || n.contains("research") {
        return false;
    }
    // Refresh / path / size / age stay on their lanes (keyword-only; no nested looks_like_*).
    if n.contains("refresh")
        || n.contains("rerun")
        || n.contains("rescan")
        || n.contains("recompute")
        || n.contains("run digester")
        || n.contains("run digest")
        || n.contains("update digest")
        || n.contains("update digester")
        || n == "digest"
        || n == "/digest"
        || n.contains("size")
        || n.contains("big")
        || n.contains("large")
        || n.contains("bytes")
        || contains_age_token(&n)
        || n.contains("how old")
        || n.contains("stale")
        || n.contains("when")
        || n.contains("path")
        || n.contains("where")
        || n.contains("why")
        || n.contains("explain")
        || n.contains("fix")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "digest open"
            | "open digest"
            | "open the digest"
            | "the digest"
            | "view digest"
            | "view the digest"
            | "see digest"
            | "see the digest"
            | "show the digest"
            | "show me the digest"
            | "list digest"
            | "list the digest"
            | "open candidates"
            | "digest candidates"
            | "open digest hints"
            | "any open candidates"
            | "show open candidates"
            | "view open candidates"
            | "see open candidates"
            | "list open candidates"
            | "list the open candidates"
            | "show me the open candidates"
            | "the open candidates"
            | "view digest open"
            | "see digest open"
            | "list digest open"
            | "view digest candidates"
            | "see digest candidates"
            | "list digest candidates"
            | "list the digest candidates"
            | "view open digest"
            | "see open digest"
            | "list open digest"
    )
}


/// True for `/digest` / `run digest` operator asks that re-run the digester.
/// View/see/list-the stay on open (read-only). Refresh phrases + bare `digest` / `/digest`
/// (and `show me digest` after normalize → `digest`) still re-run the digester.
pub fn looks_like_digest_refresh_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 64 || n.contains(" this ") || n.contains("research") {
        return false;
    }
    if looks_like_digest_open_request(content) {
        return false;
    }
    // Do not steal read-only size/age (keyword-only; no nested looks_like_*).
    if n.contains("size")
        || n.contains("big")
        || n.contains("large")
        || n.contains("bytes")
        || contains_age_token(&n)
        || n.contains("how old")
        || n.contains("stale")
        || n.contains("when")
        || n.contains("path")
        || n.contains("where")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "digest"
            | "/digest"
            | "run digest"
            | "run the digest"
            | "refresh digest"
            | "refresh the digest"
            | "agent digest"
            | "run digester"
            | "run the digester"
            | "refresh digester"
            | "refresh the digester"
            | "update digest"
            | "update the digest"
            | "update digester"
            | "rerun digest"
            | "rerun the digest"
            | "rerun digester"
            | "rescan digest"
            | "rescan the digest"
            | "rescan digester"
            | "scan digester"
            | "recompute digest"
    )
}


/// True for any digest operator ask (refresh or read-only open).
pub fn looks_like_digest_request(content: &str) -> bool {
    looks_like_digest_open_request(content) || looks_like_digest_refresh_request(content)
}


/// Digest open words for status lines. Zero is calm (CPU window parity), not a problem count.
/// Inventory asks (“how many open”) still return a number.
pub(super) fn digest_open_status_bit(n: usize) -> String {
    if n == 0 {
        "**queue clear**".to_string()
    } else if n == 1 {
        "**1** open".to_string()
    } else {
        format!("**{n}** open")
    }
}


/// Digest stale words for status lines. Zero is calm (CPU window parity).
pub(super) fn digest_stale_status_bit(n: usize) -> String {
    if n == 0 {
        "**nothing stale**".to_string()
    } else if n == 1 {
        "**1** stale".to_string()
    } else {
        format!("**{n}** stale")
    }
}


/// Zero-LLM digest open snapshot from cached `latest.json` (no Python digester).
pub fn format_digest_open_gateway() -> String {
    let summary = load_digest_summary();
    if summary.open_count == 0 {
        // Keep calm "queue clear" and still say open candidates (operator / test parity).
        return format!(
            "**Digest:** {} · **0** open candidates · `/digest` for a fresh scan.",
            digest_open_status_bit(0)
        );
    }
    let mut reply = format!(
        "**Digest:** **{}** open candidate(s)",
        summary.open_count
    );
    if !summary.open_hints.is_empty() {
        reply.push_str("\n**Open:** ");
        reply.push_str(&summary.open_hints.join("; "));
    }
    if !summary.generated_at.is_empty() {
        let age = age_from_rfc3339(&summary.generated_at);
        reply.push_str(&format!(
            "\n_Cached {age} ago · `/digest` to refresh._"
        ));
    }
    reply
}


/// Zero-LLM digest age from cached `latest.json` (no Python digester).
pub fn format_digest_age_gateway() -> String {
    let summary = load_digest_summary();
    if summary.generated_at.is_empty() {
        return "**Digest:** no cached digest yet · run `/digest` first.".to_string();
    }
    let age = age_from_rfc3339(&summary.generated_at);
    let mut reply = format!(
        "**Digest:** cached **{age}** ago · {} · {}",
        digest_open_status_bit(summary.open_count),
        digest_stale_status_bit(summary.stale_count),
    );
    if summary.turns > 0 {
        reply.push_str(&format!(" · **{}** turns (7d window)", summary.turns));
    }
    reply.push_str("\n_`/digest` to refresh._");
    reply
}


/// Zero-LLM digest `latest.md` file size (stat only; no digester spawn / open dump).
pub fn format_digest_md_size_gateway() -> String {
    let path = digest_md_path();
    if !path.exists() {
        return "**Digest.md:** no `latest.md` yet · run `/digest` first · `digest size` for `latest.json`."
            .to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Digest.md:** empty `latest.md` · `/digest` to refresh · `digest size` for `latest.json`."
                .to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Digest.md:** **{label}** on disk (`latest.md`) · `digest.md age` for mtime · `digest size` for `latest.json` · `digest age` for cache age · `/digest` to refresh."
            )
        }
        Err(e) => format!("**Digest.md** — could not stat `latest.md`: {e}"),
    }
}


/// Zero-LLM digest `latest.md` age from file mtime (stat only; no digester spawn / open dump).
pub fn format_digest_md_age_gateway() -> String {
    let path = digest_md_path();
    if !path.exists() {
        return "**Digest.md:** no `latest.md` yet · run `/digest` first · `digest age` for cache age."
            .to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Digest.md:** last write **{age}** ago (`latest.md`) · `digest.md size` for on-disk bytes · `digest age` for cache age · `/digest` to refresh."
            )
        }
        Err(e) => format!("**Digest.md** — could not stat `latest.md`: {e}"),
    }
}


/// Zero-LLM digest `latest.json` file size (stat only; no digester spawn / open dump).
pub fn format_digest_size_gateway() -> String {
    let path = digest_json_path();
    if !path.exists() {
        return "**Digest:** no `latest.json` yet · run `/digest` first · `digest age` after a refresh."
            .to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Digest:** empty `latest.json` · `/digest` to refresh · `digest age` for cache age."
                .to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Digest:** **{label}** on disk (`latest.json`) · `digest age` for cache age · `digest open` for candidates · `/digest` to refresh."
            )
        }
        Err(e) => format!("**Digest** — could not stat `latest.json`: {e}"),
    }
}


/// Instant digest age reply (read-only cache; no digester spawn).
pub fn try_digest_age_instant_reply(content: &str) -> Option<String> {
    if looks_like_digest_age_request(content) {
        Some(format_digest_age_gateway())
    } else {
        None
    }
}


/// Instant digest reply: read-only size/open/age use cache; refresh re-runs digester.
pub fn try_digest_instant_reply(content: &str) -> Option<String> {
    // digest.md size before latest.json size (stat only; no digester spawn).
    if looks_like_digest_md_size_request(content) {
        return Some(format_digest_md_size_gateway());
    }
    // digest.md age before cache digest age (mtime; no digester spawn).
    if looks_like_digest_md_age_request(content) {
        return Some(format_digest_md_age_gateway());
    }
    // Size before age/open (stat only; no digester spawn).
    if looks_like_digest_size_request(content) {
        return Some(format_digest_size_gateway());
    }
    if looks_like_digest_age_request(content) {
        return Some(format_digest_age_gateway());
    }
    if looks_like_digest_open_request(content) {
        return Some(format_digest_open_gateway());
    }
    if !looks_like_digest_refresh_request(content) {
        return None;
    }
    let line = refresh_agent_digest();
    let summary = load_digest_summary();
    let mut reply = line;
    if summary.open_count > 0 {
        reply.push_str("\n**Open:** ");
        reply.push_str(&summary.open_hints.join("; "));
    }
    Some(reply)
}


/// True when the phrase asks how many digest open candidates, not a verb or open snapshot.
/// “Open digest” / “digest open” stay the read-only snapshot. “Delete open candidates” stays
/// with the agent.
pub(super) fn looks_like_digest_open_inventory_count(n: &str) -> bool {
    n.contains("how many")
        || n.contains("number of")
        || n.contains("open count")
        || n.contains("digest count")
        || n.contains("candidate count")
        || n.contains("count open")
        || n.contains("count the open")
        || n.contains("count of open")
        || n.contains("count candidate")
        || n.contains("count the candidate")
        || n.contains("count of candidate")
}


pub(super) fn age_from_rfc3339(ts: &str) -> String {
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

