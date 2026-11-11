// included by gateway.rs — insights
/// Digester Slowest parity: exclude shipped instant noise from insights p50/slowest.
pub(super) fn is_insights_slowest_noise(lane: &str, wall_ms: u64, tools: &[String], question: &str) -> bool {
    // Instant lane turns are digester Slowest noise (any wall time).
    if lane == "instant" {
        return true;
    }
    let q = question.to_lowercase();
    if q.trim_start().starts_with("skill:") {
        return true;
    }
    let has_search_tool = tools.iter().any(|t| {
        let u = t.to_uppercase();
        u.contains("BRAVE") || u.contains("PERPLEXITY")
    });
    if q.contains("climate") || q.contains("clima") || q.contains("klima") {
        return true;
    }
    if (q.contains("weather") || q.contains("wether") || q.contains("masnou")) && has_search_tool {
        return true;
    }
    // Non-instant lanes that still asked an operator slash/path glance.
    if looks_like_keep_percentile_request(question)
        || looks_like_ops_help_request(question)
        || looks_like_status_request(question)
        || looks_like_debug_log_path_request(question)
        || looks_like_debug_log_size_request(question)
        || looks_like_debug_log_age_request(question)
        || looks_like_results_tsv_path_request(question)
        || looks_like_results_tsv_size_request(question)
        || looks_like_results_tsv_age_request(question)
        || looks_like_results_tsv_pace_request(question)
        || looks_like_results_tsv_count_request(question)
    {
        return true;
    }
    false
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
            "Turns: **{}** · ok {} · {} · p50 **{}** · mean {} · max {}{}",
            insights.turns,
            insights.ok_count,
            fail_status_bit(insights.fail_count),
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
            "Digest: {} · {}",
            digest_open_status_bit(insights.digest_open_count),
            digest_stale_status_bit(insights.digest_stale_count)
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
    // Empty sections match the CPU window calm copy. Zero turns stay “No turns…”.
    if !insights.by_lane.is_empty() {
        let lanes = insights
            .by_lane
            .iter()
            .map(|(k, v)| format!("{k}:{v}"))
            .collect::<Vec<_>>()
            .join(" · ");
        lines.push(format!("Lanes: {lanes}"));
    } else if insights.turns > 0 {
        lines.push("Lanes: **no lanes yet**".into());
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
    } else if insights.turns > 0 {
        lines.push("Top tools: **no tools yet**".into());
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
    } else if insights.turns > 0 {
        lines.push("Slowest: **nothing slow**".into());
    }
    if !insights.candidates.is_empty() {
        lines.push("**Candidates**".into());
        for c in insights.candidates.iter().take(4) {
            lines.push(format!(
                "• [{}] {} ms — {} ({})",
                c.kind, c.wall_ms, c.reason, c.question_preview
            ));
        }
    } else if insights.turns > 0 {
        lines.push("Candidates: **nothing open**".into());
    }
    let mut out = lines.join("\n");
    if out.chars().count() > 1800 {
        out = out.chars().take(1790).collect::<String>() + "…";
    }
    out
}


/// Recent ok=false turns from runs.jsonl (newest first).
pub(super) fn collect_failed_runs(limit: usize, days: Option<u32>) -> Vec<FailedRunLine> {
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
pub(super) fn collect_slow_runs(limit: usize, days: Option<u32>) -> Vec<SlowRunLine> {
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
/// View/see/show me/open/list-the NL (v0.1.1044). Exact `open slow` / `open the slow` only.
/// After normalize strips `show me` / `show`, prefer `the slow` so `show me the slow` still matches.
pub fn looks_like_slow_runs_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 64 {
        return false;
    }
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
    if n.contains("path") || n.contains("where") || n.contains("size") || contains_age_token(&n) {
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
            | "the slow"
            | "view slow"
            | "view the slow"
            | "see slow"
            | "see the slow"
            | "show slow"
            | "show the slow"
            | "show me slow"
            | "show me the slow"
            | "open slow"
            | "open the slow"
            | "list slow"
            | "list the slow"
            | "view slow runs"
            | "see slow runs"
            | "open slow runs"
            | "list slow runs"
            | "list the slow runs"
            | "show me the slow runs"
    ) || n.starts_with("/slow ")
        || (n.starts_with("slow runs ") && parse_insights_days(content).is_some())
        || (n.starts_with("view slow ") && parse_insights_days(content).is_some())
        || (n.starts_with("see slow ") && parse_insights_days(content).is_some())
        || (n.starts_with("open slow ") && parse_insights_days(content).is_some())
        || (n.starts_with("list slow ") && parse_insights_days(content).is_some())
        || (n.starts_with("list the slow ") && parse_insights_days(content).is_some())
}


/// True for `/instant` / `instant runs` — not creative "make it instant" asks.
/// View/see/show me/open/list-the NL (v0.1.1044). Exact `open instant` / `open the instant` only.
pub fn looks_like_instant_runs_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 64 {
        return false;
    }
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
    if n.contains("path") || n.contains("where") || n.contains("size") || contains_age_token(&n) {
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
            | "the instant"
            | "view instant"
            | "view the instant"
            | "see instant"
            | "see the instant"
            | "show instant"
            | "show the instant"
            | "show me instant"
            | "show me the instant"
            | "open instant"
            | "open the instant"
            | "list instant"
            | "list the instant"
            | "view instant runs"
            | "see instant runs"
            | "open instant runs"
            | "list instant runs"
            | "list the instant runs"
            | "show me the instant runs"
            | "view instant lane"
            | "see instant lane"
            | "open instant lane"
            | "list the instant lane"
    ) || n.starts_with("/instant ")
        || (n.starts_with("instant runs ") && parse_insights_days(content).is_some())
        || (n.starts_with("view instant ") && parse_insights_days(content).is_some())
        || (n.starts_with("see instant ") && parse_insights_days(content).is_some())
        || (n.starts_with("open instant ") && parse_insights_days(content).is_some())
        || (n.starts_with("list instant ") && parse_insights_days(content).is_some())
        || (n.starts_with("list the instant ") && parse_insights_days(content).is_some())
}


/// True for `/direct` / `direct runs` — not free-form routing asks.
/// View/see/show me/open/list-the NL (v0.1.1044). Exact `open direct` / `open the direct` only.
pub fn looks_like_direct_runs_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 64 {
        return false;
    }
    if n.contains(" ticket") || n.contains("redmine") || n.contains("http") {
        return false;
    }
    if n.contains(" why ") || n.contains("why is") || n.contains("why did") || n.contains("explain")
    {
        return false;
    }
    if n.contains("path") || n.contains("where") || n.contains("size") || contains_age_token(&n) {
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
            | "the direct"
            | "view direct"
            | "view the direct"
            | "see direct"
            | "see the direct"
            | "show direct"
            | "show the direct"
            | "show me direct"
            | "show me the direct"
            | "open direct"
            | "open the direct"
            | "list direct"
            | "list the direct"
            | "view direct runs"
            | "see direct runs"
            | "open direct runs"
            | "list direct runs"
            | "list the direct runs"
            | "show me the direct runs"
            | "view direct lane"
            | "see direct lane"
            | "open direct lane"
            | "list the direct lane"
    ) || n.starts_with("/direct ")
        || (n.starts_with("direct runs ") && parse_insights_days(content).is_some())
        || (n.starts_with("view direct ") && parse_insights_days(content).is_some())
        || (n.starts_with("see direct ") && parse_insights_days(content).is_some())
        || (n.starts_with("open direct ") && parse_insights_days(content).is_some())
        || (n.starts_with("list direct ") && parse_insights_days(content).is_some())
        || (n.starts_with("list the direct ") && parse_insights_days(content).is_some())
}


/// True for `/lite` / `lite runs` — not free-form “make it lite” asks.
/// View/see/show me/open/list-the NL (v0.1.1044). Exact `open lite` / `open the lite` only.
pub fn looks_like_lite_runs_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 64 {
        return false;
    }
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
    if n.contains("path") || n.contains("where") || n.contains("size") || contains_age_token(&n) {
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
            | "the lite"
            | "view lite"
            | "view the lite"
            | "see lite"
            | "see the lite"
            | "show lite"
            | "show the lite"
            | "show me lite"
            | "show me the lite"
            | "open lite"
            | "open the lite"
            | "list lite"
            | "list the lite"
            | "view lite runs"
            | "see lite runs"
            | "open lite runs"
            | "list lite runs"
            | "list the lite runs"
            | "show me the lite runs"
            | "view lite lane"
            | "see lite lane"
            | "open lite lane"
            | "list the lite lane"
    ) || n.starts_with("/lite ")
        || (n.starts_with("lite runs ") && parse_insights_days(content).is_some())
        || (n.starts_with("view lite ") && parse_insights_days(content).is_some())
        || (n.starts_with("see lite ") && parse_insights_days(content).is_some())
        || (n.starts_with("open lite ") && parse_insights_days(content).is_some())
        || (n.starts_with("list lite ") && parse_insights_days(content).is_some())
        || (n.starts_with("list the lite ") && parse_insights_days(content).is_some())
}


/// True for `/failed` / `failed runs` — not "why did X fail".
/// View/see/show me/open/list-the NL (v0.1.1044). Exact `open failed` / `open the failed` only.
/// After normalize strips `show me` / `show`, prefer `the failed` so `show me the failed` still matches.
pub fn looks_like_failed_runs_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 64 {
        return false;
    }
    if n.contains(" ticket") || n.contains("redmine") || n.contains("http") {
        return false;
    }
    if n.contains(" why ") || n.contains("why did") || n.contains("explain") {
        return false;
    }
    if n.contains("path") || n.contains("where") || n.contains("size") || contains_age_token(&n) {
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
            | "the failed"
            | "the failures"
            | "what went wrong tonight"
            | "what went wrong today"
            | "view failed"
            | "view the failed"
            | "see failed"
            | "see the failed"
            | "show failed"
            | "show the failed"
            | "show me failed"
            | "show me the failed"
            | "open failed"
            | "open the failed"
            | "list failed"
            | "list the failed"
            | "view failures"
            | "see failures"
            | "open failures"
            | "list failures"
            | "list the failures"
            | "show me the failures"
            | "view failed runs"
            | "see failed runs"
            | "open failed runs"
            | "list failed runs"
            | "list the failed runs"
            | "show me the failed runs"
    ) || n.starts_with("/failed ")
        || (n.starts_with("failed runs ") && parse_insights_days(content).is_some())
        || (n.starts_with("view failed ") && parse_insights_days(content).is_some())
        || (n.starts_with("see failed ") && parse_insights_days(content).is_some())
        || (n.starts_with("open failed ") && parse_insights_days(content).is_some())
        || (n.starts_with("list failed ") && parse_insights_days(content).is_some())
        || (n.starts_with("list the failed ") && parse_insights_days(content).is_some())
        || (n.starts_with("view failures ") && parse_insights_days(content).is_some())
        || (n.starts_with("open failures ") && parse_insights_days(content).is_some())
}


/// True for `/insights` / `insights` (Hermes parity) and short NL equivalents.
/// View/see/show me/open/list-the NL (v0.1.1039). Exact `open insights` / `open the insights`
/// only. After normalize strips `show me` / `show`, prefer `the insights` so
/// `show me the insights` still matches. Topic research (“insights on …”) stays with the agent.
pub fn looks_like_insights_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 64 {
        return false;
    }
    // Topic research ("insights on weather") stays with the agent.
    if n.contains(" on ")
        || n.contains(" about ")
        || n.contains("weather")
        || n.contains("http")
        || n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("ticket")
        || n.contains("redmine")
        || n.contains("path")
        || n.contains("where")
        || n.contains("size")
        || contains_age_token(&n)
    {
        return false;
    }
    matches!(
        n.as_str(),
        "insights"
            | "the insights"
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
            | "view insights"
            | "view the insights"
            | "see insights"
            | "see the insights"
            | "show insights"
            | "show the insights"
            | "show me insights"
            | "show me the insights"
            | "open insights"
            | "open the insights"
            | "list insights"
            | "list the insights"
            | "view usage insights"
            | "see usage insights"
            | "open usage insights"
            | "list usage insights"
            | "view usage analytics"
            | "see usage analytics"
            | "open usage analytics"
            | "list usage analytics"
    ) || n.starts_with("/insights ")
        || (n.starts_with("insights ") && parse_insights_days(content).is_some())
        || (n.starts_with("usage insights ") && parse_insights_days(content).is_some())
        || (n.starts_with("usage analytics ") && parse_insights_days(content).is_some())
        || (n.starts_with("view insights ") && parse_insights_days(content).is_some())
        || (n.starts_with("see insights ") && parse_insights_days(content).is_some())
        || (n.starts_with("open insights ") && parse_insights_days(content).is_some())
        || (n.starts_with("list insights ") && parse_insights_days(content).is_some())
}

