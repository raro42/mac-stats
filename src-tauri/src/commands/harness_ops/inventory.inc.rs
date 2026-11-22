// included by gateway.rs — inventory
/// Agent Ops Runs Slow filter parity (wall time ≥ this ms).
pub const OPS_RUNS_SLOW_MS: u64 = 2000;


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


/// True for short “next schedule / next job” asks — Agent Ops health Next schedule parity; not full list.
pub fn looks_like_next_schedule_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 56 {
        return false;
    }
    if n.starts_with("schedule a")
        || n.starts_with("schedule me")
        || n.contains(" for tomorrow")
        || n.contains("create")
        || n.contains("add ")
        || n.contains("remove")
        || n.contains("delete")
        || n.contains(" about ")
        || n.contains("ticket")
        || n.contains("redmine")
        || n.contains("tonight")
        || n.contains("this night")
        || n.contains("this evening")
        || n.contains("plan for")
        || n.contains("planned")
        || n.contains("agenda")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "next schedule"
            | "/next schedule"
            | "next scheduled"
            | "next scheduled job"
            | "next scheduled task"
            | "next job"
            | "next cron"
            | "next cron job"
            | "when is the next schedule"
            | "when is the next job"
            | "when's the next schedule"
            | "when's the next job"
            | "what's the next schedule"
            | "what is the next schedule"
            | "what's the next job"
            | "what is the next job"
            | "whats the next schedule"
            | "whats the next job"
    )
}


/// True for short “last delivery” asks — Agent Ops health Last delivery parity; not full list.
pub fn looks_like_last_delivery_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 56 {
        return false;
    }
    if n.contains("deliveries")
        || n.contains("list ")
        || n.contains("show ")
        || n.contains("recent ")
        || n.starts_with("schedule")
        || n.contains("create")
        || n.contains("why")
        || n.contains(" about ")
        || n.contains("failed")
        || n.contains("explain")
        || n.contains("ticket")
        || n.contains("redmine")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "last delivery"
            | "/last delivery"
            | "when was the last delivery"
            | "when's the last delivery"
            | "when is the last delivery"
            | "what's the last delivery"
            | "what is the last delivery"
            | "whats the last delivery"
    )
}


/// True for short “how many schedules/jobs/deliveries” count asks — not full list or next fire.
/// Inventory only: “how many schedules”, “job count”, “count the deliveries”.
/// Verb phrases (“delete these schedules”, “clear the jobs”, “export deliveries”) stay with the agent.
pub fn looks_like_schedule_count_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    if n.contains("create")
        || n.contains("add ")
        || n.contains("why")
        || n.contains("list")
        || n.contains("show ")
        || n.contains(" about ")
        || n.contains("ticket")
        || n.contains("redmine")
        || n.contains("next ")
        || n.contains("when ")
        || n.contains("delete")
        || n.contains("clear")
        || n.contains("remove")
        || n.contains("export")
        || n.contains("pause")
        || n.contains("disable")
        || n.contains("enable")
        || n.contains("edit")
        || n.contains("update")
        || n.contains("cancel")
        || n.contains("fire ")
        || n.starts_with("run ")
        || n.contains(" run ")
    {
        return false;
    }
    if looks_like_next_schedule_request(content) || looks_like_last_delivery_request(content) {
        return false;
    }
    // Inventory only — bare "schedules" / verb+noun stay off this lane.
    looks_like_noun_inventory_count(&n, "schedule")
        || looks_like_noun_inventory_count(&n, "job")
        || looks_like_noun_inventory_count(&n, "cron")
        || looks_like_noun_inventory_count(&n, "delivery")
        || ((n.contains("how many") || n.contains("number of"))
            && (n.contains("scheduled") || n.contains("deliveries")))
}


/// Which operator inventory a short count ask targets (not full lists).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorCountKind {
    Agents,
    Monitors,
    Tasks,
    Sessions,
    Skills,
    Plugins,
    Knowledge,
    DigestOpen,
}


/// True when the phrase asks how many of `stem` (e.g. "agent"), not a verb on one item.
pub(super) fn looks_like_noun_inventory_count(n: &str, stem: &str) -> bool {
    if !n.contains(stem) {
        return false;
    }
    n.contains("how many")
        || n.contains("number of")
        || n.contains(&format!("{stem} count"))
        || n.contains(&format!("{stem}s count"))
        || n.contains(&format!("count {stem}"))
        || n.contains(&format!("count the {stem}"))
        || n.contains(&format!("count of {stem}"))
}


/// True when the phrase asks how many sessions, not a verb on one session.
pub(super) fn looks_like_session_inventory_count(n: &str) -> bool {
    looks_like_noun_inventory_count(n, "session")
}


/// True when the phrase asks how many knowledge/memory files, not a verb on one note.
pub(super) fn looks_like_knowledge_inventory_count(n: &str) -> bool {
    looks_like_noun_inventory_count(n, "knowledge")
        || looks_like_noun_inventory_count(n, "memories")
        || looks_like_noun_inventory_count(n, "memory")
}


/// Parse count-only operator asks — Agent Ops card parity; not list/create asks.
pub fn parse_operator_count_kind(content: &str) -> Option<OperatorCountKind> {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return None;
    }
    if looks_like_schedule_count_request(content) {
        return None;
    }
    if n.contains("create")
        || n.contains("add ")
        || n.contains("why")
        || n.contains("list")
        || n.contains("show ")
        || n.contains(" about ")
        || n.contains("ticket")
        || n.contains("redmine")
        || n.contains("next ")
        || n.contains("when ")
        || n.contains("last ")
        || n.contains("cursor_agent")
        || n.contains("cursor-agent")
        || n.contains("fix ")
        || n.contains(':')
        || n.chars().any(|c| c.is_ascii_digit())
    {
        return None;
    }
    // Digest open inventory only — "open digest" stays the snapshot; verbs stay with the agent.
    if n.contains("open candidate")
        || n.contains("digest open")
        || n.contains("open digest")
        || n == "how many open"
        || n == "open count"
        || n == "digest count"
        || n == "candidate count"
        || ((n.contains("digest") || n.contains("candidate"))
            && (n.contains("how many") || n.contains("number of") || n.contains("count")))
    {
        if looks_like_digest_open_inventory_count(&n) {
            return Some(OperatorCountKind::DigestOpen);
        }
        return None;
    }
    // Inventory only — "delete this agent" and other verbs stay with the agent.
    if n.contains("agent") {
        if looks_like_noun_inventory_count(&n, "agent") {
            return Some(OperatorCountKind::Agents);
        }
        return None;
    }
    if n.contains("monitor") || (n.contains("site") && n.contains("how many")) {
        if n.contains("site") && n.contains("how many") {
            return Some(OperatorCountKind::Monitors);
        }
        if looks_like_noun_inventory_count(&n, "monitor") {
            return Some(OperatorCountKind::Monitors);
        }
        return None;
    }
    if n.contains("task") {
        if looks_like_noun_inventory_count(&n, "task") {
            return Some(OperatorCountKind::Tasks);
        }
        return None;
    }
    // Session inventory only. "Edit this session" and other verbs stay with the agent.
    // "Open sessions" and "saved sessions" stay the list (checked before this count).
    if n.contains("session") {
        if looks_like_session_inventory_count(&n) {
            return Some(OperatorCountKind::Sessions);
        }
        return None;
    }
    if n.contains("skill") {
        if looks_like_noun_inventory_count(&n, "skill") {
            return Some(OperatorCountKind::Skills);
        }
        return None;
    }
    if n.contains("plugin") {
        if looks_like_noun_inventory_count(&n, "plugin") {
            return Some(OperatorCountKind::Plugins);
        }
        return None;
    }
    if n.contains("knowledge") || n.contains("memories") || n.contains("memory file") {
        if looks_like_knowledge_inventory_count(&n) {
            return Some(OperatorCountKind::Knowledge);
        }
        return None;
    }
    None
}


/// True for short “how many agents/monitors/tasks…” count asks.
pub fn looks_like_operator_count_request(content: &str) -> bool {
    parse_operator_count_kind(content).is_some()
}


/// Zero-LLM operator inventory counts (Agent Ops overview parity; not full lists).
pub fn format_operator_count_gateway(kind: OperatorCountKind) -> String {
    match kind {
        OperatorCountKind::Agents => {
            let agents = crate::agents::load_all_agents();
            let on_n = agents.iter().filter(|a| a.enabled).count();
            let off_n = agents.len().saturating_sub(on_n);
            format!(
                "**Agents:** **{on_n}** on · **{off_n}** off (**{total}** total) · Agent Ops → Agents for the list.",
                total = agents.len()
            )
        }
        OperatorCountKind::Monitors => {
            let rows = crate::commands::monitors::list_monitors_for_ops();
            let up_n = rows.iter().filter(|r| r.is_up == Some(true)).count();
            let down_n = rows.iter().filter(|r| r.is_up == Some(false)).count();
            let slow_n = rows.iter().filter(|r| monitor_row_is_slow(r)).count();
            format!(
                "**Monitors:** **{total}** total · **{up_n}** up · **{down_n}** down · **{slow_n}** slow · External / Monitors for the list.",
                total = rows.len()
            )
        }
        OperatorCountKind::Tasks => match crate::task::count_tasks_by_status() {
            Ok((open, wip, paused, finished, unsuccessful)) => {
                let active = open + wip;
                let total = active + paused + finished + unsuccessful;
                format!(
                    "**Tasks:** **{active}** active (open **{open}** · wip **{wip}**) · **{total}** total · `/tasks` or Agent Ops for the list."
                )
            }
            Err(e) => format!("**Tasks:** unavailable ({e})"),
        },
        OperatorCountKind::Sessions => {
            let live_n = list_live_sessions().len();
            let files_n = list_session_files(Some(20)).map(|f| f.len()).unwrap_or(0);
            format!(
                "**Sessions:** **{live_n}** live · **{files_n}** files · Agent Ops → Sessions for the list."
            )
        }
        OperatorCountKind::Skills => {
            let n = crate::skills::load_skills().len();
            format!(
                "**Skills:** **{n}** installed · `/skills` for the catalog."
            )
        }
        OperatorCountKind::Plugins => {
            let plugins = crate::commands::plugins::list_registered_plugins();
            let on_n = plugins.iter().filter(|p| p.enabled).count();
            let off_n = plugins.len().saturating_sub(on_n);
            format!(
                "**Plugins:** **{on_n}** on · **{off_n}** off (**{total}** total) · `/plugins` for the list.",
                total = plugins.len()
            )
        }
        OperatorCountKind::Knowledge => {
            let files = list_memory_files().unwrap_or_default();
            let discord_n = files
                .iter()
                .filter(|f| knowledge_row_is_discord(&f.kind))
                .count();
            let core_n = files
                .iter()
                .filter(|f| knowledge_row_is_core(&f.kind))
                .count();
            format!(
                "**Knowledge:** **{discord_n}** Discord · **{core_n}** Core (**{total}** files) · Agent Ops → Knowledge for the list.",
                total = files.len()
            )
        }
        OperatorCountKind::DigestOpen => {
            let summary = load_digest_summary();
            if summary.open_count == 0 {
                "**Digest:** **0** open candidates · run `/digest` for a fresh scan.".to_string()
            } else {
                format!(
                    "**Digest:** **{}** open candidate(s) · `/digest` or Agent Ops → Runs for hints.",
                    summary.open_count
                )
            }
        }
    }
}


/// Which runs.jsonl count a short ask targets (not full lists or `/insights`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunsCountKind {
    Total,
    Failed,
    Slow,
    Instant,
    Direct,
    Lite,
}


/// Counts from runs.jsonl for zero-LLM operator count replies.
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct RunsCountSnapshot {
    total: usize,
    ok: usize,
    fail: usize,
    slow: usize,
    instant: usize,
    direct: usize,
    lite: usize,
}


pub(super) fn snapshot_runs_counts(days: Option<u32>) -> RunsCountSnapshot {
    let path = crate::commands::run_telemetry::runs_jsonl_path();
    let mut snap = RunsCountSnapshot::default();
    if !path.is_file() {
        return snap;
    }
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return snap,
    };
    let window_days = days.map(|d| d.clamp(1, 90));
    let since = window_days.map(|d| chrono::Utc::now() - chrono::Duration::days(d as i64));
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
        snap.total += 1;
        let ok = v.get("ok").and_then(|x| x.as_bool()).unwrap_or(true);
        if ok {
            snap.ok += 1;
        } else {
            snap.fail += 1;
        }
        let wall = v.get("wall_ms").and_then(|x| x.as_u64()).unwrap_or(0);
        if wall >= OPS_RUNS_SLOW_MS {
            snap.slow += 1;
        }
        let lane = v
            .get("lane")
            .and_then(|x| x.as_str())
            .unwrap_or("?")
            .to_lowercase();
        match lane.as_str() {
            "instant" => snap.instant += 1,
            "direct" => snap.direct += 1,
            "lite" => snap.lite += 1,
            _ => {}
        }
    }
    snap
}


/// Parse count-only runs asks — Agent Ops Runs card parity; not list/report asks.
/// Inventory only: “how many runs”, “run count”, “number of failed runs”.
/// Verb phrases (“delete these runs”, “clear the runs”, “export runs”) stay with the agent.
pub fn parse_runs_count_kind(content: &str) -> Option<RunsCountKind> {
    let n = normalize_operator_command(content);
    if n.chars().count() > 52 {
        return None;
    }
    if looks_like_schedule_count_request(content) || looks_like_operator_count_request(content) {
        return None;
    }
    if looks_like_debug_log_count_request(content) {
        return None;
    }
    if n.contains("log") || n.contains("debug") {
        return None;
    }
    if n.contains("list")
        || n.contains("show ")
        || n.contains("recent ")
        || n.contains("why")
        || n.contains("explain")
        || n.contains(" about ")
        || n.contains("ticket")
        || n.contains("redmine")
        || n.contains("report")
        || n.contains("p50")
        || n.contains("insights")
    {
        return None;
    }
    if looks_like_failed_runs_request(content)
        || looks_like_slow_runs_request(content)
        || looks_like_instant_runs_request(content)
        || looks_like_direct_runs_request(content)
        || looks_like_lite_runs_request(content)
        || looks_like_insights_request(content)
    {
        return None;
    }
    // Inventory phrasing only — bare "runs" / "failed runs" / verb+runs stay off this lane.
    let is_count = n.contains("how many") || n.contains("count") || n.contains("number of");
    if !is_count {
        return None;
    }
    if n.contains("fail") || n.contains("error") {
        return Some(RunsCountKind::Failed);
    }
    if n.contains("slow") {
        return Some(RunsCountKind::Slow);
    }
    if n.contains("instant") {
        return Some(RunsCountKind::Instant);
    }
    if n.contains("direct") {
        return Some(RunsCountKind::Direct);
    }
    if n.contains("lite") {
        return Some(RunsCountKind::Lite);
    }
    if n.contains("run") {
        return Some(RunsCountKind::Total);
    }
    None
}


/// True for short “how many runs / failed / slow…” count asks.
pub fn looks_like_runs_count_request(content: &str) -> bool {
    parse_runs_count_kind(content).is_some()
}


/// Zero-LLM runs.jsonl counts (Agent Ops Runs overview parity; not full lists).
pub fn format_runs_count_gateway(kind: RunsCountKind) -> String {
    let snap = snapshot_runs_counts(None);
    match kind {
        RunsCountKind::Total => {
            if snap.total == 0 {
                return "**Runs:** no turns in `runs.jsonl` yet · `/insights` after the first chat turn."
                    .to_string();
            }
            format!(
                "**Runs:** **{total}** turns · ok **{ok}** · {fail} · instant **{instant}** · direct **{direct}** · slow **{slow}** (≥{} ms) · `/insights` or Agent Ops → Runs.",
                OPS_RUNS_SLOW_MS,
                total = snap.total,
                ok = snap.ok,
                fail = fail_status_bit(snap.fail),
                instant = snap.instant,
                direct = snap.direct,
                slow = snap.slow,
            )
        }
        RunsCountKind::Failed => {
            if snap.fail == 0 {
                "**Failed runs:** **0** · all clear · `/failed` or Agent Ops → Runs (Fail filter)."
                    .to_string()
            } else {
                format!(
                    "**Failed runs:** **{}** of **{}** turns · `/failed` or Agent Ops → Runs (Fail filter).",
                    snap.fail, snap.total
                )
            }
        }
        RunsCountKind::Slow => format!(
            "**Slow runs:** **{}** of **{}** turns (≥{} ms) · `/slow` or Agent Ops → Runs (Slow filter).",
            snap.slow, snap.total, OPS_RUNS_SLOW_MS
        ),
        RunsCountKind::Instant => format!(
            "**Instant runs:** **{}** of **{}** turns · `/instant` or Agent Ops → Runs (Instant filter).",
            snap.instant, snap.total
        ),
        RunsCountKind::Direct => format!(
            "**Direct runs:** **{}** of **{}** turns · `/direct` or Agent Ops → Runs (Direct filter).",
            snap.direct, snap.total
        ),
        RunsCountKind::Lite => format!(
            "**Lite runs:** **{}** of **{}** turns · `/lite` or Agent Ops → Runs (Lite filter).",
            snap.lite, snap.total
        ),
    }
}


/// Zero-LLM schedule or delivery count (Agent Ops Schedules card parity).
pub fn format_schedule_count_gateway(content: &str) -> String {
    let n = normalize_operator_command(content);
    if n.contains("delivery") || n.contains("deliveries") {
        let n = crate::scheduler::list_scheduler_delivery_awareness().len();
        return format!(
            "**Deliveries:** **{n}** recorded · Agent Ops → Schedules for the list."
        );
    }
    let snap = crate::scheduler::scheduler_operator_snapshot();
    format!(
        "**Schedules:** **{n}** job(s) loaded · Agent Ops → Schedules for the full list.",
        n = snap.total_entries
    )
}


/// Zero-LLM last scheduler delivery (Agent Ops health Last delivery card parity).
pub fn format_last_delivery_gateway() -> String {
    let deliveries = crate::scheduler::list_scheduler_delivery_awareness();
    if deliveries.is_empty() {
        return "**Deliveries:** nothing recorded yet — fired jobs that post to Discord show up here."
            .to_string();
    }
    let last = &deliveries[0];
    let age = age_from_rfc3339(&last.utc);
    let preview: String = last.summary.chars().take(72).collect();
    let sid = last
        .schedule_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("—");
    let total = deliveries.len();
    format!(
        "**Last delivery** · **{utc}** ({age} ago) — `{sid}` · ch {channel}\n\n{preview}\n\n**{total}** recorded · Agent Ops → Schedules for the full list.",
        utc = last.utc,
        channel = last.channel_id
    )
}


/// Zero-LLM next schedule fire (Agent Ops health Next schedule card parity).
pub fn format_next_schedule_gateway() -> String {
    let snap = crate::scheduler::scheduler_operator_snapshot();
    if snap.total_entries == 0 {
        return "**Schedules:** no jobs loaded — add one in Agent Ops → Schedules or via Discord `SCHEDULE` tools."
            .to_string();
    }
    match (
        snap.next_run_at.as_deref(),
        snap.next_task_preview.as_deref(),
        snap.seconds_until_next_fire,
    ) {
        (Some(at), Some(task), Some(secs)) => {
            let when = if secs < 3600 {
                format!("{}m", (secs / 60).max(1))
            } else {
                format!("{}h", (secs + 1800) / 3600)
            };
            let preview: String = task.chars().take(72).collect();
            format!(
                "**Next schedule** · **{at}** (~{when}) — {preview}\n\n**{n}** job(s) loaded · Agent Ops → Schedules for the full list.",
                n = snap.total_entries
            )
        }
        (Some(at), Some(task), None) => {
            let preview: String = task.chars().take(72).collect();
            format!(
                "**Next schedule** · **{at}** — {preview}\n\n**{n}** job(s) loaded · Agent Ops → Schedules for the full list.",
                n = snap.total_entries
            )
        }
        _ => format!(
            "**Schedules:** **{}** job(s) loaded but no upcoming fire computed yet.",
            snap.total_entries
        ),
    }
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
            | "list the schedules"
            | "list schedule"
            | "list the schedule"
            | "show schedules"
            | "show the schedules"
            | "show me schedules"
            | "show me the schedules"
            | "show schedule"
            | "show the schedule"
            | "show me schedule"
            | "show me the schedule"
            | "view schedules"
            | "view the schedules"
            | "view schedule"
            | "view the schedule"
            | "see schedules"
            | "see the schedules"
            | "see schedule"
            | "see the schedule"
            | "open schedules"
            | "open the schedules"
            | "open schedule"
            | "open the schedule"
            | "my schedules"
            | "the schedules"
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


pub(super) fn monitor_row_is_slow(r: &crate::commands::monitors::OpsMonitorRow) -> bool {
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
    // Size/age/path-only asks go to monitors.json instant (does not steal `/monitors` list).
    if looks_like_monitors_size_request(content)
        || looks_like_monitors_age_request(content)
        || looks_like_monitors_path_request(content)
    {
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
            | "list the monitors"
            | "list monitor"
            | "list the monitor"
            | "show monitors"
            | "show the monitors"
            | "show me monitors"
            | "show me the monitors"
            | "show monitor"
            | "show the monitor"
            | "show me monitor"
            | "show me the monitor"
            | "view monitors"
            | "view the monitors"
            | "view monitor"
            | "view the monitor"
            | "see monitors"
            | "see the monitors"
            | "see monitor"
            | "see the monitor"
            | "open monitors"
            | "open the monitors"
            | "open monitor"
            | "open the monitor"
            | "my monitors"
            | "all monitors"
            | "the monitors"
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
    // Path-only asks go to disk_cleanup.json instant (inline — do not call path detector;
    // path → pinned → quarantine → this fn would recurse).
    if n.contains("disk_cleanup.json")
        || n.contains("disk_cleanup")
        || n.contains("disk-cleanup")
        || n.contains("cleanup file")
        || n.contains("cleanup config")
        || (n.contains("disk cleanup")
            && (n.contains("path")
                || n.contains("where")
                || n.contains("location")
                || n.contains("folder")
                || n.contains("directory")
                || n.contains("dir")
                || n.contains("file")
                || n.contains("json")))
    {
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
        || n.contains("how much disk")
        || n.contains("how much storage")
        || n.contains("disk used")
        || n.contains("storage used")
        || n.contains("disk full")
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
            | "list the cleanup"
            | "list disk"
            | "list the disk"
            | "list disk cleanup"
            | "list the disk cleanup"
            | "show cleanup"
            | "show the cleanup"
            | "show me cleanup"
            | "show me the cleanup"
            | "show disk"
            | "show the disk"
            | "show me disk"
            | "show me the disk"
            | "show disk cleanup"
            | "show the disk cleanup"
            | "show me disk cleanup"
            | "show me the disk cleanup"
            | "view cleanup"
            | "view the cleanup"
            | "view disk"
            | "view the disk"
            | "view disk cleanup"
            | "view the disk cleanup"
            | "see cleanup"
            | "see the cleanup"
            | "see disk"
            | "see the disk"
            | "see disk cleanup"
            | "see the disk cleanup"
            | "open cleanup"
            | "open the cleanup"
            | "open disk"
            | "open the disk"
            | "open disk cleanup"
            | "open the disk cleanup"
            | "the disk cleanup"
            | "my disk cleanup"
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
            | "show the logs"
            | "show me the logs"
            | "show me logs"
            | "the logs"
            | "the log"
            | "list logs"
            | "list the logs"
            | "view logs"
            | "view the logs"
            | "view log"
            | "view the log"
            | "see logs"
            | "see the logs"
            | "see log"
            | "see the log"
            | "open logs"
            | "open the logs"
            | "open log"
            | "open the log"
            | "review logs"
            | "review log"
            | "review the logs"
            | "review the log"
            | "check logs"
            | "check log"
            | "check the logs"
            | "check the log"
            | "look at logs"
            | "look at the logs"
            | "look at log"
            | "look at the log"
            | "read logs"
            | "read the logs"
            | "read log"
            | "read the log"
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


/// Error/warn counts from the debug.log tail (128 KiB window; UI parity).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugLogCountSnapshot {
    pub errors: usize,
    pub warns: usize,
}


/// Count ERROR/WARN lines in the debug.log tail — shared by list + count instant lanes.
pub fn snapshot_debug_log_counts() -> Result<DebugLogCountSnapshot, String> {
    let tail = crate::commands::logging::read_debug_log(Some(128 * 1024))
        .map_err(|e| e.to_string())?;
    let mut errors = 0usize;
    let mut warns = 0usize;
    for line in tail.content.lines() {
        match debug_log_line_kind(line) {
            "error" => errors += 1,
            "warn" => warns += 1,
            _ => {}
        }
    }
    Ok(DebugLogCountSnapshot { errors, warns })
}


/// Which debug.log count slice to return — count-only asks; not `/logs` list parity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugLogCountKind {
    Error,
    Warn,
    Both,
}


/// Parse count-only debug.log asks — Debug Log glance parity; not list/report asks.
/// Inventory only: “how many log errors”, “error count”, “count the warnings”.
/// Verb phrases (“delete these errors”, “clear the log”, “export warnings”) stay with the agent.
pub fn parse_debug_log_count_kind(content: &str) -> Option<DebugLogCountKind> {
    let n = normalize_operator_command(content);
    if n.chars().count() > 56 {
        return None;
    }
    if n.contains("list")
        || n.contains("show ")
        || n.contains("tail")
        || n.contains("why")
        || n.contains("fix")
        || n.contains("explain")
        || n.contains("clear")
        || n.contains("delete")
        || n.contains("remove")
        || n.contains("export")
        || n.contains("rotate")
        || n.contains("open in")
        || n.contains("editor")
        || n.contains(" ticket")
        || n.contains("redmine")
    {
        return None;
    }
    if looks_like_debug_log_request(content) && !n.contains("how many") && !n.contains("count") {
        return None;
    }
    // Inventory phrasing only — bare "… errors" / verb+errors stay off this lane.
    let is_count = n.contains("how many") || n.contains("count") || n.contains("number of");
    if !is_count {
        return None;
    }
    let log_ctx = n.contains("log") || n.contains("debug");
    let wants_error = n.contains("error") || n.contains("panic");
    let wants_warn = n.contains("warn") || n.contains("warning");
    if wants_error && wants_warn {
        return Some(DebugLogCountKind::Both);
    }
    if wants_error {
        return Some(DebugLogCountKind::Error);
    }
    if wants_warn {
        return Some(DebugLogCountKind::Warn);
    }
    if log_ctx {
        return Some(DebugLogCountKind::Both);
    }
    None
}


/// True for short “how many log errors / warn count…” asks.
/// Inventory only — verb phrases stay with the agent.
pub fn looks_like_debug_log_count_request(content: &str) -> bool {
    parse_debug_log_count_kind(content).is_some()
}


/// Zero-LLM debug.log error/warn counts (tail window; no line dump).
pub fn format_debug_log_count_gateway(kind: DebugLogCountKind) -> String {
    let snap = match snapshot_debug_log_counts() {
        Ok(s) => s,
        Err(e) => return format!("**Debug Log** — could not read: {e}"),
    };
    match kind {
        DebugLogCountKind::Error => {
            if snap.errors == 0 {
                "**Log errors:** **0** in this tail · all clear · `/logs error` for lines."
                    .to_string()
            } else {
                format!(
                    "**Log errors:** **{}** in this tail · `/logs error` or Debug Log → Error filter.",
                    snap.errors
                )
            }
        }
        DebugLogCountKind::Warn => {
            if snap.warns == 0 {
                "**Log warnings:** **0** in this tail · all clear · `/logs warn` for lines."
                    .to_string()
            } else {
                format!(
                    "**Log warnings:** **{}** in this tail · `/logs warn` or Debug Log → Warn filter.",
                    snap.warns
                )
            }
        }
        DebugLogCountKind::Both => format!(
            "**Debug Log:** **{}** error · **{}** warn in this tail · `/logs` for lines · Agent Ops glance parity.",
            snap.errors, snap.warns
        ),
    }
}


/// Total debug.log file size on disk (metadata only; no tail read).
pub fn snapshot_debug_log_file_bytes() -> Result<u64, String> {
    let path = crate::config::Config::log_file_path();
    if !path.exists() {
        return Ok(0);
    }
    std::fs::metadata(&path)
        .map(|m| m.len())
        .map_err(|e| format!("Failed to stat log: {e}"))
}


/// Last debug.log modification time (metadata only; no tail read).
pub fn snapshot_debug_log_modified_ms() -> Result<Option<u64>, String> {
    let path = crate::config::Config::log_file_path();
    if !path.exists() {
        return Ok(None);
    }
    let modified = std::fs::metadata(&path)
        .map_err(|e| format!("Failed to stat log: {e}"))?
        .modified()
        .map_err(|e| format!("Failed to read log mtime: {e}"))?;
    let ms = modified
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map_err(|e| format!("Invalid log mtime: {e}"))?
        .as_millis() as u64;
    Ok(Some(ms))
}


pub(super) fn format_one_plist_size_line(label: &str, path: &std::path::Path) -> String {
    let display = path.display().to_string();
    if !path.exists() {
        return format!("**{label}:** not installed · `{display}`");
    }
    match std::fs::metadata(path).map(|m| m.len()) {
        Ok(0) => format!("**{label}:** empty · `{display}`"),
        Ok(bytes) => {
            let size = crate::commands::disk_cleanup::format_bytes(bytes);
            format!("**{label}:** **{size}** · `{display}`")
        }
        Err(e) => format!("**{label}:** could not stat `{display}`: {e}"),
    }
}


pub(super) fn format_one_plist_age_line(label: &str, path: &std::path::Path) -> String {
    let display = path.display().to_string();
    if !path.exists() {
        return format!("**{label}:** not installed · `{display}`");
    }
    match std::fs::metadata(path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!("**{label}:** last write **{age}** ago · `{display}`")
        }
        Err(e) => format!("**{label}:** could not stat `{display}`: {e}"),
    }
}


pub(super) fn format_keep_rate_pct(part: u64, total: u64) -> String {
    if total == 0 {
        "n/a".to_string()
    } else {
        let pct = ((part as f64) * 100.0 / (total as f64)).round() as u64;
        format!("{pct}%")
    }
}


pub(super) fn format_skew_x100(v: i64) -> String {
    let sign = if v < 0 { "-" } else { "+" };
    let abs = v.abs();
    let whole = abs / 100;
    let frac = abs % 100;
    format!("{sign}{whole}.{frac:02}")
}


pub(super) fn format_kurt_x100(v: i64) -> String {
    let sign = if v < 0 { "-" } else { "+" };
    let abs = v.abs();
    let whole = abs / 100;
    let frac = abs % 100;
    format!("{sign}{whole}.{frac:02}")
}


pub(super) fn format_gini_x1000(v: i64) -> String {
    let whole = v / 1000;
    let frac = (v % 1000).abs();
    format!("{whole}.{frac:03}")
}


/// Bucket order for stable histogram display.
pub(super) const KEEP_GAP_HIST_ORDER: &[&str] = &["<5m", "5–10m", "10–20m", "20–30m", "30–60m", "≥60m"];


pub(super) fn format_entropy_x100(v: i64) -> String {
    let whole = v / 100;
    let frac = v.abs() % 100;
    format!("{whole}.{frac:02}")
}


/// Sum file bytes under `dir` (recursive). Caps at `max_entries` visited files+dirs.
pub(super) fn dir_total_bytes(dir: &std::path::Path, max_entries: usize) -> Result<(u64, usize), String> {
    if !dir.exists() {
        return Err("missing".to_string());
    }
    let mut total = 0u64;
    let mut files = 0usize;
    let mut visited = 0usize;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(cur) = stack.pop() {
        let rd = match std::fs::read_dir(&cur) {
            Ok(rd) => rd,
            Err(e) => return Err(e.to_string()),
        };
        for ent in rd.flatten() {
            visited = visited.saturating_add(1);
            if visited > max_entries {
                return Ok((total, files));
            }
            let Ok(meta) = ent.metadata() else {
                continue;
            };
            if meta.is_dir() {
                stack.push(ent.path());
            } else if meta.is_file() {
                total = total.saturating_add(meta.len());
                files = files.saturating_add(1);
            }
        }
    }
    Ok((total, files))
}


/// Newest file mtime under `dir` (recursive). Caps at `max_entries` visited files+dirs.
/// Returns `(newest_ms, file_count)`. Empty dir → `(None, 0)`. Missing → `Err("missing")`.
pub(super) fn dir_newest_mtime_ms(
    dir: &std::path::Path,
    max_entries: usize,
) -> Result<(Option<u64>, usize), String> {
    if !dir.exists() {
        return Err("missing".to_string());
    }
    let mut newest_ms: Option<u64> = None;
    let mut files = 0usize;
    let mut visited = 0usize;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(cur) = stack.pop() {
        let rd = match std::fs::read_dir(&cur) {
            Ok(rd) => rd,
            Err(e) => return Err(e.to_string()),
        };
        for ent in rd.flatten() {
            visited = visited.saturating_add(1);
            if visited > max_entries {
                return Ok((newest_ms, files));
            }
            let Ok(meta) = ent.metadata() else {
                continue;
            };
            if meta.is_dir() {
                stack.push(ent.path());
            } else if meta.is_file() {
                files = files.saturating_add(1);
                if let Ok(modified) = meta.modified() {
                    let ms = modified
                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0);
                    newest_ms = Some(match newest_ms {
                        Some(prev) => prev.max(ms),
                        None => ms,
                    });
                }
            }
        }
    }
    Ok((newest_ms, files))
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


pub(super) fn format_process_ram(bytes: u64) -> String {
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
pub(super) fn is_process_pin_action_ask(n: &str) -> bool {
    n.contains("unpin")
        || n.contains("pin this")
        || n.contains("pin that")
        || n.contains("pin the")
        || n == "pin"
        || n.starts_with("pin ")
        || n.contains(" pin ")
}


/// Parse Hot/Pinned from `/processes hot`, `/processes pinned`, etc. Default All.
/// View/see/show me/open/list-the NL (v0.1.1043). Exact `open hot` / `open pinned` only —
/// not `view hot rings` / `view hot strip` / `view hot details` (those stay on rings/strip/details)
/// and not pinned path/size/age.
pub fn parse_processes_list_filter(content: &str) -> ProcessesListFilter {
    let n = normalize_operator_command(content);
    if n.ends_with(" hot")
        || n == "hot"
        || n == "/hot"
        || n == "the hot"
        || n == "processes hot"
        || n == "/processes hot"
        || n == "process hot"
        || n == "hot processes"
        || n == "hot process"
        || n == "the hot processes"
        || n == "the hot process"
        || n == "what's hot"
        || n == "whats hot"
        || n == "what is hot"
        || n == "which processes are hot"
        || n == "which process is hot"
        || n == "view hot"
        || n == "view the hot"
        || n == "see hot"
        || n == "see the hot"
        || n == "show hot"
        || n == "show the hot"
        || n == "show me hot"
        || n == "show me the hot"
        || n == "open hot"
        || n == "open the hot"
        || n == "list hot"
        || n == "list the hot"
        || n == "view hot processes"
        || n == "view the hot processes"
        || n == "see hot processes"
        || n == "see the hot processes"
        || n == "show hot processes"
        || n == "show the hot processes"
        || n == "show me hot processes"
        || n == "show me the hot processes"
        || n == "open hot processes"
        || n == "open the hot processes"
        || n == "list hot processes"
        || n == "list the hot processes"
        || n == "view processes hot"
        || n == "see processes hot"
        || n == "open processes hot"
        || n == "list processes hot"
    {
        // Do not steal rings/strip/details Hot (`view hot rings` ends with rings, not hot).
        if !(n.contains("ring") || n.contains("strip") || n.contains("detail")) {
            return ProcessesListFilter::Hot;
        }
    }
    if n.ends_with(" pinned")
        || n == "pinned"
        || n == "/pinned"
        || n == "the pinned"
        || n == "processes pinned"
        || n == "/processes pinned"
        || n == "process pinned"
        || n == "pinned processes"
        || n == "pinned process"
        || n == "the pinned processes"
        || n == "the pinned process"
        || n == "show pinned"
        || n == "show the pinned"
        || n == "show me pinned"
        || n == "show me the pinned"
        || n == "list pinned"
        || n == "list the pinned"
        || n == "my pinned"
        || n == "pinned favorites"
        || n == "process favorites"
        || n == "favorite processes"
        || n == "view pinned"
        || n == "view the pinned"
        || n == "see pinned"
        || n == "see the pinned"
        || n == "open pinned"
        || n == "open the pinned"
        || n == "view pinned processes"
        || n == "view the pinned processes"
        || n == "see pinned processes"
        || n == "see the pinned processes"
        || n == "show pinned processes"
        || n == "show the pinned processes"
        || n == "show me pinned processes"
        || n == "show me the pinned processes"
        || n == "open pinned processes"
        || n == "open the pinned processes"
        || n == "list pinned processes"
        || n == "list the pinned processes"
        || n == "view processes pinned"
        || n == "see processes pinned"
        || n == "open processes pinned"
        || n == "list processes pinned"
    {
        if !(n.contains(" path")
            || n.ends_with(" path")
            || n.contains(" size")
            || n.ends_with(" size")
            || n.contains(" age")
            || n.ends_with(" age"))
        {
            return ProcessesListFilter::Pinned;
        }
    }
    ProcessesListFilter::All
}


/// True for `/processes` / `top processes` — Hot/Pinned filter parity; not kill/pin-action asks.
/// View/see/show me/open/list-the Hot·Pinned NL (v0.1.1043). Exact `open hot` / `open pinned`
/// only — not rings/strip/details Hot, not pinned path/size/age.
pub fn looks_like_processes_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    // Path-only asks go to pinned_processes.json instant (does not steal `/pinned` list).
    if looks_like_pinned_processes_size_request(content)
        || looks_like_pinned_processes_age_request(content)
    {
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
        // Rings / strip / details own `view hot rings` / `view hot strip` / `view hot details`.
        || n.contains("ring")
        || n.contains("strip")
        || n.contains("detail")
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
            | "list the processes"
            | "list process"
            | "list the process"
            | "show processes"
            | "show the processes"
            | "show me processes"
            | "show me the processes"
            | "show process"
            | "show the process"
            | "show me process"
            | "show me the process"
            | "view processes"
            | "view the processes"
            | "view process"
            | "view the process"
            | "see processes"
            | "see the processes"
            | "see process"
            | "see the process"
            | "open processes"
            | "open the processes"
            | "open process"
            | "open the process"
            | "open process list"
            | "open the process list"
            | "top processes"
            | "top process"
            | "the processes"
            | "the process list"
            | "process status"
            | "processes status"
            | "/processes hot"
            | "processes hot"
            | "process hot"
            | "hot processes"
            | "hot process"
            | "the hot processes"
            | "the hot process"
            | "/hot"
            | "hot"
            | "the hot"
            | "what's hot"
            | "whats hot"
            | "what is hot"
            | "which processes are hot"
            | "which process is hot"
            | "view hot"
            | "view the hot"
            | "see hot"
            | "see the hot"
            | "show hot"
            | "show the hot"
            | "show me hot"
            | "show me the hot"
            | "open hot"
            | "open the hot"
            | "list hot"
            | "list the hot"
            | "view hot processes"
            | "view the hot processes"
            | "see hot processes"
            | "see the hot processes"
            | "show hot processes"
            | "show the hot processes"
            | "show me hot processes"
            | "show me the hot processes"
            | "open hot processes"
            | "open the hot processes"
            | "list hot processes"
            | "list the hot processes"
            | "view processes hot"
            | "see processes hot"
            | "open processes hot"
            | "list processes hot"
            | "/processes pinned"
            | "processes pinned"
            | "process pinned"
            | "pinned processes"
            | "pinned process"
            | "the pinned processes"
            | "the pinned process"
            | "/pinned"
            | "pinned"
            | "the pinned"
            | "show pinned"
            | "show the pinned"
            | "show me pinned"
            | "show me the pinned"
            | "list pinned"
            | "list the pinned"
            | "my pinned"
            | "pinned favorites"
            | "process favorites"
            | "favorite processes"
            | "view pinned"
            | "view the pinned"
            | "see pinned"
            | "see the pinned"
            | "open pinned"
            | "open the pinned"
            | "view pinned processes"
            | "view the pinned processes"
            | "see pinned processes"
            | "see the pinned processes"
            | "show pinned processes"
            | "show the pinned processes"
            | "show me pinned processes"
            | "show me the pinned processes"
            | "open pinned processes"
            | "open the pinned processes"
            | "list pinned processes"
            | "list the pinned processes"
            | "view processes pinned"
            | "see processes pinned"
            | "open processes pinned"
            | "list processes pinned"
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


/// One process using the most CPU — not the full Top Processes list.
/// "Why" stays with the agent. `/processes` and `hot processes` stay the list.
/// "How much cpu is used" stays the CPU ring.
pub fn looks_like_top_cpu_process_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 56 {
        return false;
    }
    if n.contains("why")
        || n.contains("how much")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("kill")
        || n.contains("hot")
        || n.contains("gpu")
        || n.contains("ram")
        || n.contains("memory")
        || n.contains("list")
        || n.contains("processes")
        || n.contains("pin")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "what's using the most cpu"
            | "whats using the most cpu"
            | "what is using the most cpu"
            | "who's using the most cpu"
            | "whos using the most cpu"
            | "who is using the most cpu"
            | "which process is using the most cpu"
            | "which process uses the most cpu"
            | "what's the top cpu process"
            | "whats the top cpu process"
            | "what is the top cpu process"
            | "top cpu process"
            | "what's eating the cpu"
            | "whats eating the cpu"
            | "what is eating the cpu"
            | "what's hogging the cpu"
            | "whats hogging the cpu"
            | "which app is using the most cpu"
            | "which app uses the most cpu"
    )
}


/// Zero-LLM name of the process with the highest CPU % (cached Top Processes).
pub fn format_top_cpu_process_gateway() -> String {
    let details = crate::metrics::get_cpu_details();
    let best = details.top_processes.iter().max_by(|a, b| {
        a.cpu
            .partial_cmp(&b.cpu)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    match best {
        Some(p) if !p.name.trim().is_empty() => {
            let name = truncate_preview(&p.name, 40);
            let ram = format_process_ram(p.memory);
            let hot_mark = if process_row_is_hot(p) { " · hot" } else { "" };
            format!(
                "**Top CPU** · `{name}` · pid {pid} · {cpu:.0}% · {ram}{hot_mark}",
                pid = p.pid,
                cpu = p.cpu,
            )
        }
        _ => "**Top CPU** — _nothing here yet — open the CPU window so Top Processes can fill in._"
            .to_string(),
    }
}


/// One process using the most RAM — not the full Top Processes list.
/// "Why" stays with the agent. `/processes` and `hot processes` stay the list.
/// "How much ram is used" stays the RAM percent chip. "How big is memory" stays installed RAM.
pub fn looks_like_top_ram_process_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 56 {
        return false;
    }
    if n.contains("why")
        || n.contains("how much")
        || n.contains("how big")
        || n.contains("how many")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("kill")
        || n.contains("hot")
        || n.contains("cpu")
        || n.contains("gpu")
        || n.contains("list")
        || n.contains("processes")
        || n.contains("pin")
        || n.contains("notes")
        || n.contains("folder")
        || n.contains("memory.md")
        || n.contains("path")
        || n.contains("size")
        || contains_age_token(&n)
        || n.contains("used")
        || n.contains("installed")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "what's using the most ram"
            | "whats using the most ram"
            | "what is using the most ram"
            | "who's using the most ram"
            | "whos using the most ram"
            | "who is using the most ram"
            | "which process is using the most ram"
            | "which process uses the most ram"
            | "what's the top ram process"
            | "whats the top ram process"
            | "what is the top ram process"
            | "top ram process"
            | "what's eating the ram"
            | "whats eating the ram"
            | "what is eating the ram"
            | "what's hogging the ram"
            | "whats hogging the ram"
            | "which app is using the most ram"
            | "which app uses the most ram"
            | "what's using the most memory"
            | "whats using the most memory"
            | "what is using the most memory"
            | "who's using the most memory"
            | "whos using the most memory"
            | "who is using the most memory"
            | "which process is using the most memory"
            | "which process uses the most memory"
            | "what's the top memory process"
            | "whats the top memory process"
            | "what is the top memory process"
            | "top memory process"
            | "what's eating the memory"
            | "whats eating the memory"
            | "what is eating the memory"
            | "what's hogging the memory"
            | "whats hogging the memory"
            | "which app is using the most memory"
            | "which app uses the most memory"
    )
}


/// Zero-LLM name of the process with the highest resident RAM (cached Top Processes).
pub fn format_top_ram_process_gateway() -> String {
    let details = crate::metrics::get_cpu_details();
    let best = details
        .top_processes
        .iter()
        .filter(|p| p.memory > 0 && !p.name.trim().is_empty())
        .max_by_key(|p| p.memory);
    match best {
        Some(p) => {
            let name = truncate_preview(&p.name, 40);
            let ram = format_process_ram(p.memory);
            let hot_mark = if process_row_is_hot(p) { " · hot" } else { "" };
            format!(
                "**Top RAM** · `{name}` · pid {pid} · {ram} · {cpu:.0}%{hot_mark}",
                pid = p.pid,
                cpu = p.cpu,
            )
        }
        None => {
            "**Top RAM** — _nothing here yet — open the CPU window so Top Processes can fill in._"
                .to_string()
        }
    }
}


/// One process using the most GPU — not the full Top Processes list.
/// "Why" stays with the agent. `/processes` and `hot processes` stay the list.
/// "How much gpu is used" and "is the gpu hot" stay the GPU ring.
pub fn looks_like_top_gpu_process_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 56 {
        return false;
    }
    if n.contains("why")
        || n.contains("how much")
        || n.contains("how hot")
        || n.contains("how fast")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("kill")
        || n.contains("hot")
        || n.contains("cpu")
        || n.contains("ram")
        || n.contains("memory")
        || n.contains("list")
        || n.contains("processes")
        || n.contains("pin")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "what's using the most gpu"
            | "whats using the most gpu"
            | "what is using the most gpu"
            | "who's using the most gpu"
            | "whos using the most gpu"
            | "who is using the most gpu"
            | "which process is using the most gpu"
            | "which process uses the most gpu"
            | "what's the top gpu process"
            | "whats the top gpu process"
            | "what is the top gpu process"
            | "top gpu process"
            | "what's eating the gpu"
            | "whats eating the gpu"
            | "what is eating the gpu"
            | "what's hogging the gpu"
            | "whats hogging the gpu"
            | "which app is using the most gpu"
            | "which app uses the most gpu"
    )
}


/// Zero-LLM name of the process with the highest estimated GPU % (cached Top Processes).
pub fn format_top_gpu_process_gateway() -> String {
    let details = crate::metrics::get_cpu_details();
    let best = details
        .top_processes
        .iter()
        .filter(|p| p.gpu > 0.0 && !p.name.trim().is_empty())
        .max_by(|a, b| {
            a.gpu
                .partial_cmp(&b.gpu)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    match best {
        Some(p) => {
            let name = truncate_preview(&p.name, 40);
            let ram = format_process_ram(p.memory);
            let hot_mark = if process_row_is_hot(p) { " · hot" } else { "" };
            format!(
                "**Top GPU** · `{name}` · pid {pid} · GPU {gpu:.0}% · CPU {cpu:.0}% · {ram}{hot_mark}",
                pid = p.pid,
                cpu = p.cpu,
                gpu = p.gpu,
            )
        }
        None if details.top_processes.is_empty() => {
            "**Top GPU** — _nothing here yet — open the CPU window so Top Processes can fill in._"
                .to_string()
        }
        None => "**Top GPU** — _no process is using the GPU right now._".to_string(),
    }
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

pub const OPS_RING_HOT_RAM_PCT: f32 = 85.0;

pub const OPS_RING_HOT_FREQ_GHZ: f32 = 3.5;

pub const OPS_RING_HOT_TEMP_C: f32 = 70.0;


pub(super) fn ring_cpu_is_hot(usage: f32) -> bool {
    usage >= OPS_RING_HOT_CPU_PCT
}


pub(super) fn ring_gpu_is_hot(gpu: f32) -> bool {
    gpu >= OPS_RING_HOT_GPU_PCT
}


pub(super) fn ring_ram_is_hot(ram: f32) -> bool {
    ram >= OPS_RING_HOT_RAM_PCT
}


pub(super) fn ring_freq_is_hot(freq_ghz: f32) -> bool {
    freq_ghz >= OPS_RING_HOT_FREQ_GHZ
}


pub(super) fn ring_temp_is_hot(temp_c: f32) -> bool {
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
        || n == "view hot rings"
        || n == "see hot rings"
        || n == "open hot rings"
        || n == "list the hot rings"
        || n == "what's hot on rings"
        || n == "whats hot on rings"
    {
        return RingsListFilter::Hot;
    }
    RingsListFilter::All
}


/// True for `/rings` / `cpu rings` — Hot filter parity; not process `/hot` asks.
/// View/see/show me/open/list-the NL (v0.1.1040). Exact `open rings` / `open the rings`
/// only. After normalize strips `show me` / `show`, prefer `the rings` so
/// `show me the rings` still matches.
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
        || n.contains("path")
        || n.contains("where")
        || n.contains("size")
        || contains_age_token(&n)
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/rings"
            | "rings"
            | "the rings"
            | "cpu rings"
            | "the cpu rings"
            | "ring gauges"
            | "ring gauge"
            | "metric rings"
            | "list rings"
            | "list the rings"
            | "show rings"
            | "show the rings"
            | "show me rings"
            | "show me the rings"
            | "view rings"
            | "view the rings"
            | "see rings"
            | "see the rings"
            | "open rings"
            | "open the rings"
            | "view cpu rings"
            | "see cpu rings"
            | "open cpu rings"
            | "list cpu rings"
            | "list the cpu rings"
            | "rings status"
            | "/rings hot"
            | "rings hot"
            | "cpu rings hot"
            | "hot rings"
            | "hot ring"
            | "the hot rings"
            | "which rings are hot"
            | "which ring is hot"
            | "show hot rings"
            | "list hot rings"
            | "view hot rings"
            | "see hot rings"
            | "open hot rings"
            | "list the hot rings"
            | "view rings hot"
            | "see rings hot"
            | "open rings hot"
            | "list rings hot"
            | "what's hot on rings"
            | "whats hot on rings"
    )
}


/// Zero-LLM CPU rings report (All · Hot; live get_cpu_details; menu-bar amber thresholds).
pub fn format_rings_gateway(filter: RingsListFilter) -> String {
    let d = crate::metrics::get_cpu_details();
    let cpu = d.usage;
    let gpu = d.gpu_usage;
    let ram = d.ram_percent;
    let freq = d.frequency;
    let temp = d.temperature;

    let rows: [(&str, String, bool); 5] = [
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
            "RAM",
            format!("{ram:.0}%"),
            ring_ram_is_hot(ram),
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
            format!("**CPU rings** — 5 · {hot_n} hot")
        }
        RingsListFilter::Hot => {
            format!(
                "**CPU rings · Hot** — {hot_n} (CPU≥{:.0}% · GPU≥{:.0}% · RAM≥{:.0}% · Freq≥{} GHz · Temp≥{:.0}°C)",
                OPS_RING_HOT_CPU_PCT,
                OPS_RING_HOT_GPU_PCT,
                OPS_RING_HOT_RAM_PCT,
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
                "_No ring is hot right now (CPU ≥{:.0}%, GPU ≥{:.0}%, RAM ≥{:.0}%, Freq ≥{} GHz, or Temp ≥{:.0}°C)._",
                OPS_RING_HOT_CPU_PCT,
                OPS_RING_HOT_GPU_PCT,
                OPS_RING_HOT_RAM_PCT,
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


/// Focused CPU ring asks (`/cpu` · `/gpu` · `/ram-ring` · `/freq` · `/temp`) — not full `/rings`.
/// RAM ring (v0.1.2915) uses `ram ring` / `/ram-ring` so strip `/ram` stays the power-strip chip.
/// CPU · GPU · Freq · Temp `* ring` / `/*-ring` (v0.1.2928) — parity with `/ram-ring`; plural
/// `cpu rings` / `view cpu rings` stay the full `/rings` panel (router checks rings first).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingChipAsk {
    Cpu,
    Gpu,
    Ram,
    Freq,
    Temp,
}


/// Parse `/cpu` · `/gpu` · `/ram-ring` · `/freq` · `/temp` (and short NL). None when not a chip ask.
/// View/see/show me/open/list-the NL (v0.1.1041). Exact `open cpu` / `open the cpu` only
/// (same for gpu / freq / temp). After normalize strips `show me` / `show`, prefer `the cpu`
/// etc. so `show me the cpu` still matches. Does not steal `/rings`, `/details`, or `cpu window`.
/// RAM ring phrases must include `ring` / `gauge` so strip `/ram` / "how much ram" stay strip.
/// CPU · GPU · Freq · Temp ring/gauge aliases run before the generic `ring` veto (v0.1.2928).
pub fn parse_ring_chip_ask(content: &str) -> Option<RingChipAsk> {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return None;
    }
    // RAM ring before the generic `ring` veto (strip `/ram` stays StripChipAsk::Ram).
    if matches!(
        n.as_str(),
        "/ram-ring"
            | "/ramring"
            | "ram ring"
            | "the ram ring"
            | "ram gauge"
            | "the ram gauge"
            | "memory ring"
            | "the memory ring"
            | "memory gauge"
            | "the memory gauge"
            | "show ram ring"
            | "show the ram ring"
            | "show me ram ring"
            | "show me the ram ring"
            | "list ram ring"
            | "list the ram ring"
            | "view ram ring"
            | "view the ram ring"
            | "see ram ring"
            | "see the ram ring"
            | "open ram ring"
            | "open the ram ring"
            | "ram ring status"
            | "how's the ram ring"
            | "hows the ram ring"
            | "what's the ram ring"
            | "whats the ram ring"
            | "what is the ram ring"
            | "is the ram ring hot"
            | "is ram ring hot"
            | "is the ram ring high"
            | "is ram ring high"
    ) {
        return Some(RingChipAsk::Ram);
    }
    // CPU · GPU · Freq · Temp ring/gauge before the generic `ring` veto (RAM parity).
    // Plural `cpu rings` stays `looks_like_rings_request` (router runs that first).
    if matches!(
        n.as_str(),
        "/cpu-ring"
            | "/cpuring"
            | "cpu ring"
            | "the cpu ring"
            | "cpu gauge"
            | "the cpu gauge"
            | "show cpu ring"
            | "show the cpu ring"
            | "show me cpu ring"
            | "show me the cpu ring"
            | "list cpu ring"
            | "list the cpu ring"
            | "view cpu ring"
            | "view the cpu ring"
            | "see cpu ring"
            | "see the cpu ring"
            | "open cpu ring"
            | "open the cpu ring"
            | "cpu ring status"
            | "how's the cpu ring"
            | "hows the cpu ring"
            | "what's the cpu ring"
            | "whats the cpu ring"
            | "what is the cpu ring"
            | "is the cpu ring hot"
            | "is cpu ring hot"
            | "is the cpu ring high"
            | "is cpu ring high"
    ) {
        return Some(RingChipAsk::Cpu);
    }
    if matches!(
        n.as_str(),
        "/gpu-ring"
            | "/gpuring"
            | "gpu ring"
            | "the gpu ring"
            | "gpu gauge"
            | "the gpu gauge"
            | "show gpu ring"
            | "show the gpu ring"
            | "show me gpu ring"
            | "show me the gpu ring"
            | "list gpu ring"
            | "list the gpu ring"
            | "view gpu ring"
            | "view the gpu ring"
            | "see gpu ring"
            | "see the gpu ring"
            | "open gpu ring"
            | "open the gpu ring"
            | "gpu ring status"
            | "how's the gpu ring"
            | "hows the gpu ring"
            | "what's the gpu ring"
            | "whats the gpu ring"
            | "what is the gpu ring"
            | "is the gpu ring hot"
            | "is gpu ring hot"
            | "is the gpu ring high"
            | "is gpu ring high"
    ) {
        return Some(RingChipAsk::Gpu);
    }
    if matches!(
        n.as_str(),
        "/freq-ring"
            | "/frequency-ring"
            | "/freqring"
            | "freq ring"
            | "the freq ring"
            | "frequency ring"
            | "the frequency ring"
            | "freq gauge"
            | "the freq gauge"
            | "frequency gauge"
            | "the frequency gauge"
            | "show freq ring"
            | "show the freq ring"
            | "show me freq ring"
            | "show me the freq ring"
            | "show frequency ring"
            | "show the frequency ring"
            | "show me frequency ring"
            | "show me the frequency ring"
            | "list freq ring"
            | "list the freq ring"
            | "list frequency ring"
            | "list the frequency ring"
            | "view freq ring"
            | "view the freq ring"
            | "view frequency ring"
            | "view the frequency ring"
            | "see freq ring"
            | "see the freq ring"
            | "see frequency ring"
            | "see the frequency ring"
            | "open freq ring"
            | "open the freq ring"
            | "open frequency ring"
            | "open the frequency ring"
            | "freq ring status"
            | "how's the freq ring"
            | "hows the freq ring"
            | "what's the freq ring"
            | "whats the freq ring"
            | "what is the freq ring"
            | "is the freq ring hot"
            | "is freq ring hot"
            | "is the freq ring high"
            | "is freq ring high"
    ) {
        return Some(RingChipAsk::Freq);
    }
    if matches!(
        n.as_str(),
        "/temp-ring"
            | "/temperature-ring"
            | "/tempring"
            | "temp ring"
            | "the temp ring"
            | "temperature ring"
            | "the temperature ring"
            | "temp gauge"
            | "the temp gauge"
            | "temperature gauge"
            | "the temperature gauge"
            | "show temp ring"
            | "show the temp ring"
            | "show me temp ring"
            | "show me the temp ring"
            | "show temperature ring"
            | "show the temperature ring"
            | "show me temperature ring"
            | "show me the temperature ring"
            | "list temp ring"
            | "list the temp ring"
            | "list temperature ring"
            | "list the temperature ring"
            | "view temp ring"
            | "view the temp ring"
            | "view temperature ring"
            | "view the temperature ring"
            | "see temp ring"
            | "see the temp ring"
            | "see temperature ring"
            | "see the temperature ring"
            | "open temp ring"
            | "open the temp ring"
            | "open temperature ring"
            | "open the temperature ring"
            | "temp ring status"
            | "how's the temp ring"
            | "hows the temp ring"
            | "what's the temp ring"
            | "whats the temp ring"
            | "what is the temp ring"
            | "is the temp ring hot"
            | "is temp ring hot"
            | "is the temp ring high"
            | "is temp ring high"
    ) {
        return Some(RingChipAsk::Temp);
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
        || n.contains("window")
        || n.contains(" path")
        || n.ends_with(" path")
        || n.contains(" size")
        || n.ends_with(" size")
        || n.contains(" age")
        || n.ends_with(" age")
        || n.starts_with("where ")
    {
        return None;
    }
    if matches!(
        n.as_str(),
        "/cpu"
            | "cpu"
            | "the cpu"
            | "cpu usage"
            | "cpu percent"
            | "cpu percentage"
            | "cpu %"
            | "show cpu"
            | "show the cpu"
            | "show me cpu"
            | "show me the cpu"
            | "list cpu"
            | "list the cpu"
            | "view cpu"
            | "view the cpu"
            | "see cpu"
            | "see the cpu"
            | "open cpu"
            | "open the cpu"
            | "cpu status"
            | "what's the cpu"
            | "whats the cpu"
            | "what is the cpu"
            | "how's the cpu"
            | "hows the cpu"
            | "how much cpu"
            | "how much cpu is used"
            | "how much cpu used"
            | "how much cpu do i have"
            | "cpu used"
            | "is the cpu high"
            | "is cpu high"
            | "is the cpu busy"
            | "is cpu busy"
            | "how busy is the cpu"
            | "how busy is cpu"
    ) {
        return Some(RingChipAsk::Cpu);
    }
    if matches!(
        n.as_str(),
        "/gpu"
            | "gpu"
            | "the gpu"
            | "gpu usage"
            | "gpu percent"
            | "gpu percentage"
            | "gpu %"
            | "show gpu"
            | "show the gpu"
            | "show me gpu"
            | "show me the gpu"
            | "list gpu"
            | "list the gpu"
            | "view gpu"
            | "view the gpu"
            | "see gpu"
            | "see the gpu"
            | "open gpu"
            | "open the gpu"
            | "gpu status"
            | "what's the gpu"
            | "whats the gpu"
            | "what is the gpu"
            | "how's the gpu"
            | "hows the gpu"
            | "is the gpu hot"
            | "is gpu hot"
            | "how hot is the gpu"
            | "how hot is gpu"
            | "how much gpu"
            | "how much gpu is used"
            | "how much gpu used"
            | "how much gpu do i have"
            | "gpu used"
            | "is the gpu high"
            | "is gpu high"
            | "is the gpu busy"
            | "is gpu busy"
            | "how busy is the gpu"
            | "how busy is gpu"
    ) {
        return Some(RingChipAsk::Gpu);
    }
    if matches!(
        n.as_str(),
        "/freq"
            | "/frequency"
            | "/ghz"
            | "freq"
            | "the freq"
            | "frequency"
            | "the frequency"
            | "ghz"
            | "the ghz"
            | "cpu frequency"
            | "cpu freq"
            | "cpu ghz"
            | "show freq"
            | "show the freq"
            | "show me freq"
            | "show me the freq"
            | "show frequency"
            | "show the frequency"
            | "show me frequency"
            | "show me the frequency"
            | "list freq"
            | "list the freq"
            | "list frequency"
            | "list the frequency"
            | "view freq"
            | "view the freq"
            | "view frequency"
            | "view the frequency"
            | "see freq"
            | "see the freq"
            | "see frequency"
            | "see the frequency"
            | "open freq"
            | "open the freq"
            | "open frequency"
            | "open the frequency"
            | "open ghz"
            | "open the ghz"
            | "freq status"
            | "what's the frequency"
            | "whats the frequency"
            | "what is the frequency"
            | "what's the freq"
            | "whats the freq"
            | "what is the freq"
            | "how's the frequency"
            | "hows the frequency"
            | "how fast"
            | "how fast is the cpu"
            | "how fast is cpu"
            | "clock speed"
            | "the clock speed"
            | "cpu clock"
            | "cpu clock speed"
            | "what's the clock speed"
            | "whats the clock speed"
            | "what is the clock speed"
            | "how's the clock speed"
            | "hows the clock speed"
            | "is the frequency high"
            | "is frequency high"
            | "is the freq high"
            | "is freq high"
            | "is the clock high"
            | "is clock high"
    ) {
        return Some(RingChipAsk::Freq);
    }
    if matches!(
        n.as_str(),
        "/temp"
            | "/temperature"
            | "temp"
            | "the temp"
            | "temperature"
            | "the temperature"
            | "cpu temp"
            | "cpu temperature"
            | "show temp"
            | "show the temp"
            | "show me temp"
            | "show me the temp"
            | "show temperature"
            | "show the temperature"
            | "show me temperature"
            | "show me the temperature"
            | "list temp"
            | "list the temp"
            | "list temperature"
            | "list the temperature"
            | "view temp"
            | "view the temp"
            | "view temperature"
            | "view the temperature"
            | "see temp"
            | "see the temp"
            | "see temperature"
            | "see the temperature"
            | "open temp"
            | "open the temp"
            | "open temperature"
            | "open the temperature"
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
            | "how hot"
            | "how hot is it"
            | "how hot is the cpu"
            | "how hot is cpu"
            | "is the cpu hot"
            | "is cpu hot"
            | "is it hot"
            | "is the temp hot"
            | "is temp hot"
    ) {
        return Some(RingChipAsk::Temp);
    }
    None
}


/// True for focused CPU · GPU · RAM · Freq · Temp ring asks (not full `/rings`).
pub fn looks_like_ring_chip_request(content: &str) -> bool {
    parse_ring_chip_ask(content).is_some()
}


/// Zero-LLM one-ring reply (CPU · GPU · RAM · Freq · Temp; live get_cpu_details; menu-bar amber cues).
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
        RingChipAsk::Ram => {
            let hot_mark = if ring_ram_is_hot(d.ram_percent) {
                " · hot"
            } else {
                ""
            };
            format!("**RAM** · {:.0}%{hot_mark}", d.ram_percent)
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


/// P-core or E-core clock — not the combined Freq ring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreClusterChip {
    Performance,
    Efficiency,
}


/// Focused P-core / E-core clock asks. "Why" stays with the agent.
/// "How fast is the CPU" stays the Freq ring. "How fast is the GPU" stays with the agent.
/// Slash + view/see/open/list NL (v0.1.2926) — parity with Load `/load-5m` · `view load 5`.
pub fn parse_core_cluster_freq(content: &str) -> Option<CoreClusterChip> {
    let n = normalize_operator_command(content);
    if n.chars().count() > 56 {
        return None;
    }
    if n.contains("why")
        || n.contains("process")
        || n.contains("gpu")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains("how to")
        || n.contains("explain")
    {
        return None;
    }
    if matches!(
        n.as_str(),
        "/p-core"
            | "/p-cores"
            | "/pcore"
            | "p core"
            | "p-core"
            | "p cores"
            | "p-cores"
            | "the p core"
            | "the p cores"
            | "p core frequency"
            | "p-core frequency"
            | "the p core frequency"
            | "p core freq"
            | "p-core freq"
            | "p core clock"
            | "p-core clock"
            | "p core speed"
            | "performance core"
            | "performance cores"
            | "the performance cores"
            | "performance core frequency"
            | "performance core clock"
            | "view p core"
            | "view the p core"
            | "view p cores"
            | "view the p cores"
            | "view p-core"
            | "view p-cores"
            | "view p core frequency"
            | "view the p core frequency"
            | "view p-core frequency"
            | "view performance cores"
            | "view the performance cores"
            | "see p core"
            | "see the p core"
            | "see p cores"
            | "see the p cores"
            | "see p-core"
            | "see p-cores"
            | "see p core frequency"
            | "see the p core frequency"
            | "see p-core frequency"
            | "open p core"
            | "open the p core"
            | "open p cores"
            | "open the p cores"
            | "open p-core"
            | "open p-cores"
            | "open p core frequency"
            | "open the p core frequency"
            | "open performance cores"
            | "list p core"
            | "list the p core"
            | "list p cores"
            | "list the p cores"
            | "list p-core"
            | "list the p-cores"
            | "list p core frequency"
            | "list the p core frequency"
            | "how fast are the p cores"
            | "how fast is the p core"
            | "how fast are the performance cores"
            | "what's the p core frequency"
            | "whats the p core frequency"
            | "what is the p core frequency"
            | "is the p core high"
            | "is p core high"
            | "is the p core frequency high"
    ) {
        return Some(CoreClusterChip::Performance);
    }
    if matches!(
        n.as_str(),
        "/e-core"
            | "/e-cores"
            | "/ecore"
            | "e core"
            | "e-core"
            | "e cores"
            | "e-cores"
            | "the e core"
            | "the e cores"
            | "e core frequency"
            | "e-core frequency"
            | "the e core frequency"
            | "e core freq"
            | "e-core freq"
            | "e core clock"
            | "e-core clock"
            | "e core speed"
            | "efficiency core"
            | "efficiency cores"
            | "the efficiency cores"
            | "efficiency core frequency"
            | "efficiency core clock"
            | "view e core"
            | "view the e core"
            | "view e cores"
            | "view the e cores"
            | "view e-core"
            | "view e-cores"
            | "view e core frequency"
            | "view the e core frequency"
            | "view e-core frequency"
            | "view efficiency cores"
            | "view the efficiency cores"
            | "see e core"
            | "see the e core"
            | "see e cores"
            | "see the e cores"
            | "see e-core"
            | "see e-cores"
            | "see e core frequency"
            | "see the e core frequency"
            | "see e-core frequency"
            | "open e core"
            | "open the e core"
            | "open e cores"
            | "open the e cores"
            | "open e-core"
            | "open e-cores"
            | "open e core frequency"
            | "open the e core frequency"
            | "open efficiency cores"
            | "list e core"
            | "list the e core"
            | "list e cores"
            | "list the e cores"
            | "list e-core"
            | "list the e-cores"
            | "list e core frequency"
            | "list the e core frequency"
            | "how fast are the e cores"
            | "how fast is the e core"
            | "how fast are the efficiency cores"
            | "what's the e core frequency"
            | "whats the e core frequency"
            | "what is the e core frequency"
            | "is the e core high"
            | "is e core high"
            | "is the e core frequency high"
    ) {
        return Some(CoreClusterChip::Efficiency);
    }
    None
}


/// True for a focused P-core or E-core clock ask (not the combined Freq ring).
pub fn looks_like_core_cluster_freq_request(content: &str) -> bool {
    parse_core_cluster_freq(content).is_some()
}


/// Zero-LLM P-core / E-core reply (same Freq≥3.5 GHz hot mark as the Freq ring).
pub fn format_core_cluster_freq_gateway(cluster: CoreClusterChip) -> String {
    let d = crate::metrics::get_cpu_details();
    let (label, value) = match cluster {
        CoreClusterChip::Performance => ("P-core", d.p_core_frequency),
        CoreClusterChip::Efficiency => ("E-core", d.e_core_frequency),
    };
    if value <= 0.0 {
        return format!(
            "**{label}** — _no frequency reading right now (open the CPU window)._"
        );
    }
    let hot_mark = if ring_freq_is_hot(value) { " · hot" } else { "" };
    format!("**{label}** · {value:.2} GHz{hot_mark}")
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

/// Combined CPU+GPU watts — Power chip amber (same 20 W as the CPU window).
pub const OPS_STRIP_POWER_HOT_W: f32 = 20.0;


pub(super) fn format_system_uptime(secs: u64) -> String {
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


/// mac-stats process clock — not the Mac Up chip.
/// "Why" stays with the agent. "What's the uptime" / bare `/uptime` stay system uptime.
/// Slash + view/open NL (`/app-uptime` · `/process-uptime` · `view app uptime`) match Bat
/// `/battery` · watts `/watts` · chip `/chip` · RAM `/ram-size` shape.
pub fn looks_like_app_uptime_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 56 {
        return false;
    }
    if n.contains("why")
        || n.contains("system")
        || n.contains("machine")
        || n.contains("the mac")
        || n.contains("mac been")
        || n.contains("session")
        || n.contains("restart")
        || n.contains("kill")
        || n.contains("how to")
        || n.contains("explain")
        // Mac Up chip owns bare `/uptime` · `uptime` · `view uptime` · `what's the uptime`.
        || n == "/uptime"
        || n == "uptime"
        || n == "up time"
        || n == "the uptime"
        || n == "view uptime"
        || n == "view the uptime"
        || n == "see uptime"
        || n == "see the uptime"
        || n == "open uptime"
        || n == "open the uptime"
        || n == "list uptime"
        || n == "list the uptime"
        || n == "what's the uptime"
        || n == "whats the uptime"
        || n == "what is the uptime"
        || n == "how's the uptime"
        || n == "hows the uptime"
        || n == "uptime status"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/app-uptime"
            | "/appuptime"
            | "/process-uptime"
            | "/processuptime"
            | "/bot-uptime"
            | "/botuptime"
            | "/mac-stats-uptime"
            | "/macstats-uptime"
            | "/macstatsuptime"
            | "/your-uptime"
            | "/youruptime"
            | "how long have you been up"
            | "how long are you up"
            | "how long running"
            | "how long have you been running"
            | "how long has the app been up"
            | "how long has the app been running"
            | "how long has mac-stats been up"
            | "how long has mac-stats been running"
            | "process uptime"
            | "app uptime"
            | "mac-stats uptime"
            | "bot uptime"
            | "your uptime"
            // View/see/show me/open/list (bare view uptime stays Mac Up chip).
            | "view app uptime"
            | "view the app uptime"
            | "view process uptime"
            | "view the process uptime"
            | "view bot uptime"
            | "view the bot uptime"
            | "view mac-stats uptime"
            | "view the mac-stats uptime"
            | "view your uptime"
            | "see app uptime"
            | "see the app uptime"
            | "see process uptime"
            | "see the process uptime"
            | "see bot uptime"
            | "see mac-stats uptime"
            | "open app uptime"
            | "open the app uptime"
            | "open process uptime"
            | "open the process uptime"
            | "open bot uptime"
            | "open mac-stats uptime"
            | "list app uptime"
            | "list the app uptime"
            | "list process uptime"
            | "list the process uptime"
            | "list bot uptime"
            | "list mac-stats uptime"
    )
}


/// Zero-LLM mac-stats process uptime (not the Mac Up chip).
pub fn format_app_uptime_gateway() -> String {
    format!("**App up** · {}", crate::state::format_process_uptime())
}


pub(super) fn strip_heat_is_attention(thermal: &str) -> bool {
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
        || n == "view hot strip"
        || n == "see hot strip"
        || n == "open hot strip"
        || n == "list the hot strip"
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
/// View/see/show me/open/list-the NL (v0.1.1040). Exact `open strip` / `open the strip`
/// only. After normalize strips `show me` / `show`, prefer `the strip` so
/// `show me the strip` still matches.
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
        || n.contains("path")
        || n.contains("where")
        || n.contains("size")
        || contains_age_token(&n)
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/strip"
            | "strip"
            | "the strip"
            | "power strip"
            | "the power strip"
            | "powerstrip"
            | "/power"
            | "power"
            | "the power"
            | "battery strip"
            | "list strip"
            | "list the strip"
            | "show strip"
            | "show the strip"
            | "show me strip"
            | "show me the strip"
            | "view strip"
            | "view the strip"
            | "see strip"
            | "see the strip"
            | "open strip"
            | "open the strip"
            | "view power"
            | "view the power"
            | "see power"
            | "see the power"
            | "open power"
            | "open the power"
            | "list power"
            | "list the power"
            | "view power strip"
            | "see power strip"
            | "open power strip"
            | "list power strip"
            | "list the power strip"
            | "strip status"
            | "power status"
            | "/strip hot"
            | "strip hot"
            | "power strip hot"
            | "powerstrip hot"
            | "/power hot"
            | "hot strip"
            | "the hot strip"
            | "hot power strip"
            | "which strip is hot"
            | "which chips are hot"
            | "show hot strip"
            | "list hot strip"
            | "view hot strip"
            | "see hot strip"
            | "open hot strip"
            | "list the hot strip"
            | "view strip hot"
            | "see strip hot"
            | "open strip hot"
            | "list strip hot"
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


/// Focused power-strip chip asks (`/battery` · `/heat` · `/lpm` · `/ram` · `/ssd` · `/storage` · `/uptime` · `/watts`) — not full `/strip`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StripChipAsk {
    Battery,
    Heat,
    Lpm,
    Ram,
    Ssd,
    Uptime,
    /// Combined CPU+GPU watts (`/watts` · `/power-draw`). Not `/power` (full strip) and not Low Power Mode.
    Power,
}


/// Parse `/battery` · `/heat` · `/lpm` · `/ram` · `/ssd` · `/storage` · `/uptime` · `/watts` (and short NL). None when not a chip ask.
/// View/see/show me/open/list-the NL (v0.1.1042). Exact `open battery` / `open the battery` only
/// (same for heat / lpm / ram / ssd / uptime / watts / power draw). After normalize strips `show me` / `show`, prefer
/// `the battery` etc. so `show me the battery` still matches. Does not steal `/strip`, `/power`, `/disk`
/// Disk Cleanup (`open disk`), `/details`, or path/size/age.
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
        || n.contains(" path")
        || n.ends_with(" path")
        || n.contains(" size")
        || n.ends_with(" size")
        || n.contains(" age")
        || n.ends_with(" age")
        || n.starts_with("where ")
        // Disk Cleanup owns bare `open disk` / `view disk` (not SSD %).
        || n == "open disk"
        || n == "open the disk"
        || n == "view disk"
        || n == "view the disk"
        || n == "see disk"
        || n == "see the disk"
        || n == "list disk"
        || n == "list the disk"
    {
        return None;
    }
    // Battery chip — not "battery strip" (handled by `/strip`).
    if matches!(
        n.as_str(),
        "/battery"
            | "/bat"
            | "battery"
            | "the battery"
            | "bat"
            | "the bat"
            | "battery level"
            | "battery percent"
            | "battery percentage"
            | "battery %"
            | "show battery"
            | "show the battery"
            | "show me battery"
            | "show me the battery"
            | "list battery"
            | "list the battery"
            | "list bat"
            | "list the bat"
            | "view battery"
            | "view the battery"
            | "view bat"
            | "view the bat"
            | "see battery"
            | "see the battery"
            | "see bat"
            | "see the bat"
            | "open battery"
            | "open the battery"
            | "open bat"
            | "open the bat"
            | "battery status"
            | "what's the battery"
            | "whats the battery"
            | "what is the battery"
            | "how's the battery"
            | "hows the battery"
            | "battery charge"
            | "charge level"
            | "how much battery"
            | "how much battery left"
            | "how much battery is left"
            | "how much battery do i have"
            | "how much charge"
            | "how much charge is left"
            | "how much charge left"
            | "battery left"
            | "battery remaining"
            | "charge left"
            | "charge remaining"
            | "is the battery low"
            | "is battery low"
            | "low battery"
            | "is it charging"
            | "is the battery charging"
            | "is battery charging"
            | "is the mac charging"
            | "am i charging"
            | "is it plugged in"
            | "is the mac plugged in"
            | "is it on charge"
    ) {
        return Some(StripChipAsk::Battery);
    }
    // Heat / thermal chip — not process `/hot`.
    if matches!(
        n.as_str(),
        "/heat"
            | "/thermal"
            | "heat"
            | "the heat"
            | "thermal"
            | "the thermal"
            | "thermal state"
            | "thermal pressure"
            | "heat state"
            | "show heat"
            | "show the heat"
            | "show me heat"
            | "show me the heat"
            | "show thermal"
            | "show the thermal"
            | "show me thermal"
            | "show me the thermal"
            | "list heat"
            | "list the heat"
            | "list thermal"
            | "list the thermal"
            | "view heat"
            | "view the heat"
            | "view thermal"
            | "view the thermal"
            | "see heat"
            | "see the heat"
            | "see thermal"
            | "see the thermal"
            | "open heat"
            | "open the heat"
            | "open thermal"
            | "open the thermal"
            | "heat status"
            | "what's the heat"
            | "whats the heat"
            | "what is the heat"
            | "what's the thermal"
            | "whats the thermal"
            | "what is the thermal"
            | "how's the heat"
            | "hows the heat"
            | "how's the thermal"
            | "hows the thermal"
            | "how's the thermal state"
            | "hows the thermal state"
            | "is the heat high"
            | "is heat high"
            | "is the thermal high"
            | "is thermal high"
            | "is the thermal state high"
            | "is thermal state high"
            | "is it throttling"
            | "is the mac throttling"
            | "is the cpu throttling"
    ) {
        return Some(StripChipAsk::Heat);
    }
    // Low Power Mode chip — not bare `/power` (full strip).
    if matches!(
        n.as_str(),
        "/lpm"
            | "lpm"
            | "the lpm"
            | "low power"
            | "low power mode"
            | "the low power mode"
            | "low-power mode"
            | "low-power"
            | "show lpm"
            | "show the lpm"
            | "show me lpm"
            | "show me the lpm"
            | "show low power"
            | "show low power mode"
            | "show me low power"
            | "show me low power mode"
            | "show me the low power mode"
            | "list lpm"
            | "list the lpm"
            | "list low power"
            | "list low power mode"
            | "list the low power mode"
            | "view lpm"
            | "view the lpm"
            | "view low power"
            | "view low power mode"
            | "view the low power mode"
            | "see lpm"
            | "see the lpm"
            | "see low power"
            | "see low power mode"
            | "see the low power mode"
            | "open lpm"
            | "open the lpm"
            | "open low power"
            | "open low power mode"
            | "open the low power mode"
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
            | "the ram"
            | "memory"
            | "the memory"
            | "mem"
            | "the mem"
            | "/memory"
            | "/mem"
            | "ram percent"
            | "ram percentage"
            | "ram %"
            | "memory percent"
            | "memory %"
            | "show ram"
            | "show the ram"
            | "show me ram"
            | "show me the ram"
            | "show memory"
            | "show the memory"
            | "show me memory"
            | "show me the memory"
            | "list ram"
            | "list the ram"
            | "list memory"
            | "list the memory"
            | "view ram"
            | "view the ram"
            | "view memory"
            | "view the memory"
            | "see ram"
            | "see the ram"
            | "see memory"
            | "see the memory"
            | "open ram"
            | "open the ram"
            | "open memory"
            | "open the memory"
            | "ram status"
            | "what's the ram"
            | "whats the ram"
            | "what is the ram"
            | "how's the ram"
            | "hows the ram"
            | "what's the memory"
            | "whats the memory"
            | "what is the memory"
            | "ram usage"
            | "memory usage"
            | "how much ram"
            | "how much ram is used"
            | "how much ram used"
            | "how much ram do i have"
            | "ram used"
            | "how much memory"
            | "how much memory is used"
            | "how much memory used"
            | "how much memory do i have"
            | "memory used"
            | "is the ram high"
            | "is ram high"
            | "is the memory high"
            | "is memory high"
    ) {
        return Some(StripChipAsk::Ram);
    }
    // SSD chip — not `/disk` Disk Cleanup (cleanup/reclaim / open disk rejected above).
    // Slash + view/open NL (`/storage` · `view storage`) match Bat `/battery` · watts `/watts` shape;
    // bare `open disk` / `view disk` stay Disk Cleanup (rejected above).
    if matches!(
        n.as_str(),
        "/ssd"
            | "/storage"
            | "ssd"
            | "the ssd"
            | "storage"
            | "the storage"
            | "disk percent"
            | "disk percentage"
            | "disk %"
            | "ssd percent"
            | "ssd %"
            | "disk usage"
            | "ssd usage"
            | "storage usage"
            | "show ssd"
            | "show the ssd"
            | "show me ssd"
            | "show me the ssd"
            | "show storage"
            | "show the storage"
            | "show me storage"
            | "show me the storage"
            | "list ssd"
            | "list the ssd"
            | "list storage"
            | "list the storage"
            | "view ssd"
            | "view the ssd"
            | "view storage"
            | "view the storage"
            | "see ssd"
            | "see the ssd"
            | "see storage"
            | "see the storage"
            | "open ssd"
            | "open the ssd"
            | "open storage"
            | "open the storage"
            | "ssd status"
            | "storage status"
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
            | "how much ssd"
            | "how much ssd is used"
            | "how much ssd used"
            | "how much ssd do i have"
            | "ssd used"
            | "how much disk"
            | "how much disk is used"
            | "how much disk used"
            | "how much disk do i have"
            | "disk used"
            | "how much storage"
            | "how much storage is used"
            | "how much storage used"
            | "how much storage do i have"
            | "storage used"
            | "is the disk full"
            | "is disk full"
            | "is the ssd full"
            | "is ssd full"
            | "is the storage full"
            | "is storage full"
            | "is the disk high"
            | "is disk high"
            | "is the ssd high"
            | "is ssd high"
    ) {
        return Some(StripChipAsk::Ssd);
    }
    // Uptime chip — not full `/details` / `/strip`.
    if matches!(
        n.as_str(),
        "/uptime"
            | "/up"
            | "uptime"
            | "the uptime"
            | "up time"
            | "system uptime"
            | "show uptime"
            | "show the uptime"
            | "show me uptime"
            | "show me the uptime"
            | "list uptime"
            | "list the uptime"
            | "view uptime"
            | "view the uptime"
            | "see uptime"
            | "see the uptime"
            | "open uptime"
            | "open the uptime"
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
    // Power watts — not `/power` / bare `power` / `view power` (full strip) and not Low Power Mode.
    // Slash + view/open NL (`/watts` · `/power-draw`) match Bat `/battery` shape; bare `/power` stays strip.
    if matches!(
        n.as_str(),
        "/watts"
            | "/power-draw"
            | "/powerdraw"
            | "how much power"
            | "how much power is used"
            | "how much power used"
            | "power draw"
            | "the power draw"
            | "power watts"
            | "the power watts"
            | "how many watts"
            | "what's the power draw"
            | "whats the power draw"
            | "what is the power draw"
            | "how's the power draw"
            | "hows the power draw"
            | "is the power high"
            | "is power high"
            | "is the power draw high"
            | "is power draw high"
            | "watts"
            | "the watts"
            | "cpu power"
            | "gpu power"
            | "show watts"
            | "show the watts"
            | "show me watts"
            | "show me the watts"
            | "show power draw"
            | "show the power draw"
            | "show me power draw"
            | "show me the power draw"
            | "list watts"
            | "list the watts"
            | "list power draw"
            | "list the power draw"
            | "view watts"
            | "view the watts"
            | "view power draw"
            | "view the power draw"
            | "see watts"
            | "see the watts"
            | "see power draw"
            | "see the power draw"
            | "open watts"
            | "open the watts"
            | "open power draw"
            | "open the power draw"
    ) {
        return Some(StripChipAsk::Power);
    }
    None
}


/// True for focused Bat · Heat · LPM · RAM · SSD · Up · Power-watts chip asks (power-strip parity; not full `/strip`).
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
        StripChipAsk::Power => {
            let total = d.cpu_power + d.gpu_power;
            if total <= 0.0 {
                return "**Power** — _no power reading right now (open the CPU window)._"
                    .to_string();
            }
            let hot_mark = if total >= OPS_STRIP_POWER_HOT_W {
                " · hot"
            } else {
                ""
            };
            format!(
                "**Power** · {:.1} W{hot_mark} (CPU {:.1} · GPU {:.1})",
                total, d.cpu_power, d.gpu_power
            )
        }
    }
}


/// Installed RAM size — not the used-% chip, not notes / memory.md disk size.
/// "Why" stays with the agent. "How much RAM is used" stays the RAM percent chip.
/// Slash + view/open NL (`/ram-size` · `/ram-capacity` · `view ram size`) match Bat `/battery` ·
/// watts `/watts` · SSD `/free` shape; bare `/ram` / `how much ram` / `view ram` stay the % chip.
pub fn looks_like_ram_capacity_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 56 {
        return false;
    }
    if n.contains("why")
        || n.contains("path")
        || n.contains(" age")
        || n.ends_with(" age")
        || n.contains("folder")
        || n.contains("notes")
        || n.contains("memory.md")
        || n.contains("memory-md")
        || n.contains("discord")
        || n.contains("session")
        || n.contains("curated")
        || n.contains("process")
        || n.contains("file")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("used")
        || n.contains("usage")
        || n.contains("percent")
        || n.contains('%')
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/ram-size"
            | "/ramsize"
            | "/ram-capacity"
            | "/ramcapacity"
            | "/memory-size"
            | "/memorysize"
            | "/memory-capacity"
            | "/memorycapacity"
            | "/total-ram"
            | "/totalram"
            | "/installed-ram"
            | "/installedram"
            | "memory size"
            | "the memory size"
            | "how big is memory"
            | "how big is the memory"
            | "how large is memory"
            | "how large is the memory"
            | "memory bytes"
            | "how big is ram"
            | "how big is the ram"
            | "how large is ram"
            | "how large is the ram"
            | "ram size"
            | "the ram size"
            | "total ram"
            | "the total ram"
            | "total memory"
            | "the total memory"
            | "ram capacity"
            | "memory capacity"
            | "how many gb of ram"
            | "how many gb of memory"
            | "how many gigabytes of ram"
            | "how many gigabytes of memory"
            | "how much ram does this mac have"
            | "how much memory does this mac have"
            | "what's the ram size"
            | "whats the ram size"
            | "what is the ram size"
            | "what's the memory size"
            | "whats the memory size"
            | "what is the memory size"
            | "installed ram"
            | "installed memory"
            | "how much ram installed"
            | "how much memory installed"
            // View/see/show me/open/list (show me → ram size stays capacity; bare view ram stays %).
            | "view ram size"
            | "view the ram size"
            | "view ram capacity"
            | "view the ram capacity"
            | "view memory size"
            | "view the memory size"
            | "view memory capacity"
            | "view the memory capacity"
            | "view total ram"
            | "view the total ram"
            | "view installed ram"
            | "view the installed ram"
            | "see ram size"
            | "see the ram size"
            | "see ram capacity"
            | "see memory size"
            | "see memory capacity"
            | "see total ram"
            | "see installed ram"
            | "open ram size"
            | "open the ram size"
            | "open ram capacity"
            | "open the ram capacity"
            | "open memory size"
            | "open the memory size"
            | "open memory capacity"
            | "open the memory capacity"
            | "open total ram"
            | "open installed ram"
            | "list ram size"
            | "list the ram size"
            | "list ram capacity"
            | "list memory size"
            | "list memory capacity"
            | "list total ram"
            | "list installed ram"
    )
}


/// Zero-LLM installed RAM (total bytes + used). Percent chip still owns "how much ram is used".
pub fn format_ram_capacity_gateway() -> String {
    let d = crate::metrics::get_cpu_details();
    if d.ram_total_bytes == 0 {
        return "**RAM** — _no memory size reading right now._".to_string();
    }
    let total = crate::commands::disk_cleanup::format_bytes(d.ram_total_bytes);
    let used = crate::commands::disk_cleanup::format_bytes(d.ram_used_bytes);
    format!("**RAM** · {total} installed · {used} used")
}


/// Apple chip name — not the CPU ring, not the Ollama model.
/// "Why" stays with the agent. "What's the CPU" / bare `/cpu` / `view cpu` stay the usage ring.
/// Slash + view/open NL (`/chip` · `/processor` · `/cpu-name` · `view chip`) match Bat `/battery` ·
/// watts `/watts` · SSD `/free` · RAM `/ram-size` shape; `/model` stays Ollama.
pub fn looks_like_chip_identity_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 56 {
        return false;
    }
    if n.contains("why")
        || n.contains("hot")
        || n.contains("model")
        || n.contains("ollama")
        || n.contains("freq")
        || n.contains("clock")
        || n.contains("ghz")
        || n.contains("fast")
        || n.contains("usage")
        || n.contains("percent")
        || n.contains("high")
        || n.contains("busy")
        || n.contains("path")
        || n.contains("size")
        || contains_age_token(&n)
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("gpu")
        // CPU usage ring owns bare `/cpu` · `view cpu` · `what's the cpu` (not chip name).
        || n == "/cpu"
        || n == "cpu"
        || n == "the cpu"
        || n == "view cpu"
        || n == "view the cpu"
        || n == "see cpu"
        || n == "see the cpu"
        || n == "open cpu"
        || n == "open the cpu"
        || n == "list cpu"
        || n == "list the cpu"
        || n == "what's the cpu"
        || n == "whats the cpu"
        || n == "what is the cpu"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/chip"
            | "/chip-name"
            | "/chipname"
            | "/processor"
            | "/cpu-name"
            | "/cpuname"
            | "/apple-chip"
            | "/applechip"
            | "what chip"
            | "what chip is this"
            | "what chip do i have"
            | "what chip does this mac have"
            | "which chip"
            | "which chip is this"
            | "which chip do i have"
            | "which chip does this mac have"
            | "what's the chip"
            | "whats the chip"
            | "what is the chip"
            | "what's my chip"
            | "whats my chip"
            | "what is my chip"
            | "mac chip"
            | "the mac chip"
            | "chip name"
            | "the chip name"
            | "what processor"
            | "what processor is this"
            | "what processor do i have"
            | "which processor"
            | "which processor is this"
            | "what's the processor"
            | "whats the processor"
            | "what is the processor"
            | "processor name"
            | "cpu name"
            | "the cpu name"
            | "what cpu do i have"
            | "what cpu is this"
            | "which cpu do i have"
            | "apple chip"
            | "the apple chip"
            // View/see/show me/open/list (bare view cpu stays CPU ring).
            | "view chip"
            | "view the chip"
            | "view chip name"
            | "view the chip name"
            | "view processor"
            | "view the processor"
            | "view processor name"
            | "view the processor name"
            | "view cpu name"
            | "view the cpu name"
            | "view apple chip"
            | "view the apple chip"
            | "view mac chip"
            | "view the mac chip"
            | "see chip"
            | "see the chip"
            | "see chip name"
            | "see the chip name"
            | "see processor"
            | "see the processor"
            | "see cpu name"
            | "see the cpu name"
            | "see apple chip"
            | "see the apple chip"
            | "open chip"
            | "open the chip"
            | "open chip name"
            | "open the chip name"
            | "open processor"
            | "open the processor"
            | "open cpu name"
            | "open the cpu name"
            | "open apple chip"
            | "open the apple chip"
            | "list chip"
            | "list the chip"
            | "list chip name"
            | "list the chip name"
            | "list processor"
            | "list the processor"
            | "list cpu name"
            | "list the cpu name"
            | "list apple chip"
            | "list the apple chip"
    )
}


/// Zero-LLM chip name from cached `chip_info` (e.g. `Apple M3 · 16 cores`).
pub fn format_chip_identity_gateway() -> String {
    let d = crate::metrics::get_cpu_details();
    let chip = d.chip_info.trim();
    if chip.is_empty() {
        return "**Chip** — _no chip name right now._".to_string();
    }
    format!("**Chip** · {chip}")
}


/// macOS ProductName · ProductVersion · BuildVersion — not app `/version` / status health.
/// Slash + view/open NL (`/macos` · `/os-version` · `view macos` · `macos version`) match chip
/// `/chip` · RAM `/ram-size` shape; bare `/version` · `view version` · `what version` stay app.
pub fn looks_like_macos_version_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 56 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || n.contains("changelog")
        || n.contains("release")
        || n.contains("bump")
        || n.contains("ship")
        || n.contains("app version")
        || n.contains("mac-stats version")
        || n.contains("bot version")
        // App `/version` · bare version stays status health (routed earlier; keep explicit veto).
        || n == "/version"
        || n == "version"
        || n == "the version"
        || n == "what version"
        || n == "which version"
        || n == "view version"
        || n == "view the version"
        || n == "see version"
        || n == "see the version"
        || n == "open version"
        || n == "open the version"
        || n == "list version"
        || n == "list the version"
        || n == "show version"
        || n == "show the version"
        || n == "show me version"
        || n == "show me the version"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/macos"
            | "/mac-os"
            | "/os"
            | "/os-version"
            | "/osversion"
            | "/macos-version"
            | "/macosversion"
            | "/mac-os-version"
            | "/system-version"
            | "/systemversion"
            | "/sw-vers"
            | "/swvers"
            | "/build"
            | "/build-version"
            | "/buildversion"
            | "macos"
            | "the macos"
            | "mac os"
            | "the mac os"
            | "os version"
            | "the os version"
            | "macos version"
            | "the macos version"
            | "mac os version"
            | "the mac os version"
            | "system version"
            | "the system version"
            | "what macos"
            | "what macos is this"
            | "what macos do i have"
            | "what os"
            | "what os is this"
            | "what os do i have"
            | "which macos"
            | "which macos is this"
            | "which os"
            | "which os is this"
            | "what's the macos"
            | "whats the macos"
            | "what is the macos"
            | "what's the os"
            | "whats the os"
            | "what is the os"
            | "what's the os version"
            | "whats the os version"
            | "what is the os version"
            | "what's the macos version"
            | "whats the macos version"
            | "what is the macos version"
            | "what's the system version"
            | "whats the system version"
            | "what is the system version"
            | "build version"
            | "the build version"
            | "sw vers"
            | "sw_vers"
            // View/see/show me/open/list (bare view version stays app `/version`).
            | "view macos"
            | "view the macos"
            | "view macos version"
            | "view the macos version"
            | "view os"
            | "view the os"
            | "view os version"
            | "view the os version"
            | "view system version"
            | "view the system version"
            | "view mac os"
            | "view the mac os"
            | "view mac os version"
            | "view the mac os version"
            | "view build"
            | "view the build"
            | "view build version"
            | "view the build version"
            | "see macos"
            | "see the macos"
            | "see macos version"
            | "see the macos version"
            | "see os"
            | "see the os"
            | "see os version"
            | "see the os version"
            | "see system version"
            | "see the system version"
            | "open macos"
            | "open the macos"
            | "open macos version"
            | "open the macos version"
            | "open os"
            | "open the os"
            | "open os version"
            | "open the os version"
            | "open system version"
            | "open the system version"
            | "list macos"
            | "list the macos"
            | "list macos version"
            | "list the macos version"
            | "list os"
            | "list the os"
            | "list os version"
            | "list the os version"
            | "list system version"
            | "list the system version"
            | "show macos"
            | "show the macos"
            | "show me macos"
            | "show me the macos"
            | "show macos version"
            | "show the macos version"
            | "show me macos version"
            | "show me the macos version"
            | "show os version"
            | "show the os version"
            | "show me os version"
            | "show me the os version"
            | "show system version"
            | "show the system version"
            | "show me system version"
            | "show me the system version"
    )
}


/// Zero-LLM macOS version from cached `sw_vers` (e.g. `macOS 15.1 (24B83)`).
pub fn format_macos_version_gateway() -> String {
    let v = crate::metrics::get_macos_version();
    let v = v.trim();
    if v.is_empty() {
        return "**macOS** — _no OS version right now._".to_string();
    }
    format!("**macOS** · {v}")
}


/// ComputerName · LocalHostName — not monitor/website host, not bare "host".
/// Slash + view/open NL (`/hostname` · `/computer-name` · `view hostname`) match chip
/// `/chip` · macOS `/macos` shape.
pub fn looks_like_hostname_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 56 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("monitor")
        || n.contains("website")
        || n.contains("url")
        || n.contains("http")
        || n.contains("dns")
        || n.contains("ssrf")
        || n.contains("allowlist")
        || n.contains("localhost")
        || n.contains("remote")
        || n.contains("ssh")
        // Bare "host" is too ambiguous (monitors / URLs / agent host talk).
        || n == "/host"
        || n == "host"
        || n == "the host"
        || n == "view host"
        || n == "view the host"
        || n == "see host"
        || n == "see the host"
        || n == "open host"
        || n == "open the host"
        || n == "list host"
        || n == "list the host"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/hostname"
            | "/host-name"
            | "/computer-name"
            | "/computername"
            | "/machine-name"
            | "/machinename"
            | "/local-hostname"
            | "/localhostname"
            | "hostname"
            | "the hostname"
            | "host name"
            | "the host name"
            | "computer name"
            | "the computer name"
            | "machine name"
            | "the machine name"
            | "local hostname"
            | "the local hostname"
            | "local host name"
            | "the local host name"
            | "what hostname"
            | "what hostname is this"
            | "what hostname do i have"
            | "what's the hostname"
            | "whats the hostname"
            | "what is the hostname"
            | "what's my hostname"
            | "whats my hostname"
            | "what is my hostname"
            | "which hostname"
            | "which hostname is this"
            | "what computer name"
            | "what computer name is this"
            | "what's the computer name"
            | "whats the computer name"
            | "what is the computer name"
            | "what's my computer name"
            | "whats my computer name"
            | "what is my computer name"
            | "what machine name"
            | "what's the machine name"
            | "whats the machine name"
            | "what is the machine name"
            | "my hostname"
            | "my computer name"
            | "my machine name"
            // View/see/show me/open/list (bare view host stays vetoed).
            | "view hostname"
            | "view the hostname"
            | "view host name"
            | "view the host name"
            | "view computer name"
            | "view the computer name"
            | "view machine name"
            | "view the machine name"
            | "view local hostname"
            | "view the local hostname"
            | "see hostname"
            | "see the hostname"
            | "see computer name"
            | "see the computer name"
            | "see machine name"
            | "see the machine name"
            | "open hostname"
            | "open the hostname"
            | "open computer name"
            | "open the computer name"
            | "open machine name"
            | "open the machine name"
            | "list hostname"
            | "list the hostname"
            | "list computer name"
            | "list the computer name"
            | "list machine name"
            | "list the machine name"
            | "show hostname"
            | "show the hostname"
            | "show me hostname"
            | "show me the hostname"
            | "show computer name"
            | "show the computer name"
            | "show me computer name"
            | "show me the computer name"
            | "show machine name"
            | "show the machine name"
            | "show me machine name"
            | "show me the machine name"
    )
}


/// Zero-LLM hostname from cached ComputerName · LocalHostName (e.g. `Ralf’s MacBook (ralfs-macbook)`).
pub fn format_hostname_gateway() -> String {
    let v = crate::metrics::get_hostname();
    let v = v.trim();
    if v.is_empty() {
        return "**Host** — _no hostname right now._".to_string();
    }
    format!("**Host** · {v}")
}


/// Mac marketing name · machine model — not Apple chip (`/chip`), not Ollama (`/model`).
/// Slash + view/open NL (`/mac-model` · `/hardware-model` · `view mac model` · `what mac is this`)
/// match chip `/chip` · hostname `/hostname` shape; bare `/model` · `which model` · `model name` stay Ollama.
pub fn looks_like_mac_model_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 56 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("ollama")
        || n.contains("llm")
        || n.contains("chip")
        || n.contains("processor")
        || n.contains("path")
        || n.contains("size")
        || contains_age_token(&n)
        // Ollama owns bare `/model` · `which model` · `model name` · `what model`.
        || n == "/model"
        || n == "model"
        || n == "the model"
        || n == "which model"
        || n == "what model"
        || n == "current model"
        || n == "your model"
        || n == "model name"
        || n == "the model name"
        || n == "view model"
        || n == "view the model"
        || n == "see model"
        || n == "see the model"
        || n == "open model"
        || n == "open the model"
        || n == "list model"
        || n == "list the model"
        || n == "what model are you"
        || n == "which model are you"
        || n == "what model are you using"
        || n == "which model are you using"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/mac-model"
            | "/macmodel"
            | "/mac-model-name"
            | "/hardware-model"
            | "/hardwaremodel"
            | "/machine-model"
            | "/machinemodel"
            | "/model-id"
            | "/modelid"
            | "/hw-model"
            | "/hwmodel"
            | "mac model"
            | "the mac model"
            | "mac model name"
            | "the mac model name"
            | "hardware model"
            | "the hardware model"
            | "machine model"
            | "the machine model"
            | "model id"
            | "the model id"
            | "hw model"
            | "the hw model"
            | "what mac"
            | "what mac is this"
            | "what mac do i have"
            | "which mac"
            | "which mac is this"
            | "which mac do i have"
            | "what's my mac"
            | "whats my mac"
            | "what is my mac"
            | "what macbook"
            | "what macbook is this"
            | "what macbook do i have"
            | "which macbook"
            | "which macbook is this"
            | "what's my macbook"
            | "whats my macbook"
            | "what is my macbook"
            | "what kind of mac"
            | "what kind of mac is this"
            | "what type of mac"
            | "what type of mac is this"
            // View/see/show me/open/list (bare view model stays Ollama).
            | "view mac model"
            | "view the mac model"
            | "view hardware model"
            | "view the hardware model"
            | "view machine model"
            | "view the machine model"
            | "view model id"
            | "view the model id"
            | "view hw model"
            | "view the hw model"
            | "see mac model"
            | "see the mac model"
            | "see hardware model"
            | "see the hardware model"
            | "see machine model"
            | "see the machine model"
            | "open mac model"
            | "open the mac model"
            | "open hardware model"
            | "open the hardware model"
            | "open machine model"
            | "open the machine model"
            | "list mac model"
            | "list the mac model"
            | "list hardware model"
            | "list the hardware model"
            | "list machine model"
            | "list the machine model"
            | "show mac model"
            | "show the mac model"
            | "show me mac model"
            | "show me the mac model"
            | "show hardware model"
            | "show the hardware model"
            | "show me hardware model"
            | "show me the hardware model"
            | "show machine model"
            | "show the machine model"
            | "show me machine model"
            | "show me the machine model"
    )
}


/// Zero-LLM Mac model from cached `system_profiler` / `sysctl hw.model` (e.g. `MacBook Pro · Mac16,1`).
pub fn format_mac_model_gateway() -> String {
    let v = crate::metrics::get_mac_model();
    let v = v.trim();
    if v.is_empty() {
        return "**Mac** — _no Mac model right now._".to_string();
    }
    format!("**Mac** · {v}")
}



/// Login username — not Discord `user-info.json`, not bare "user" (too ambiguous).
/// Slash + view/open NL (`/username` · `/whoami` · `view username`) match hostname `/hostname` ·
/// chip `/chip` shape; `user info path` / size / age stay the Discord map file.
pub fn looks_like_username_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 56 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("info")
        || n.contains("json")
        || n.contains("discord")
        || n.contains("display")
        || n.contains("monitor")
        || n.contains("website")
        || n.contains("url")
        || n.contains("http")
        || n.contains("agent")
        || n.contains("chat")
        || n.contains("filter")
        // Bare "user" is too ambiguous (Discord / chat You / user-info file).
        || n == "/user"
        || n == "user"
        || n == "the user"
        || n == "view user"
        || n == "view the user"
        || n == "see user"
        || n == "see the user"
        || n == "open user"
        || n == "open the user"
        || n == "list user"
        || n == "list the user"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/username"
            | "/user-name"
            | "/whoami"
            | "/who-am-i"
            | "/login-name"
            | "/loginname"
            | "/account-name"
            | "/accountname"
            | "username"
            | "the username"
            | "user name"
            | "the user name"
            | "login name"
            | "the login name"
            | "account name"
            | "the account name"
            | "whoami"
            | "who am i"
            | "who am i logged in as"
            | "what username"
            | "what username is this"
            | "what username do i have"
            | "what's the username"
            | "whats the username"
            | "what is the username"
            | "what's my username"
            | "whats my username"
            | "what is my username"
            | "what's my user name"
            | "whats my user name"
            | "what is my user name"
            | "what's my login name"
            | "whats my login name"
            | "what is my login name"
            | "which username"
            | "which username is this"
            | "my username"
            | "my user name"
            | "my login name"
            | "my account name"
            // View/see/show me/open/list (bare view user / bare login stay out).
            | "view username"
            | "view the username"
            | "view user name"
            | "view the user name"
            | "view whoami"
            | "view login name"
            | "view the login name"
            | "view account name"
            | "view the account name"
            | "see username"
            | "see the username"
            | "see login name"
            | "see the login name"
            | "see whoami"
            | "open username"
            | "open the username"
            | "open whoami"
            | "open login name"
            | "open the login name"
            | "open account name"
            | "open the account name"
            | "list username"
            | "list the username"
            | "list login name"
            | "list the login name"
            | "list whoami"
            | "show username"
            | "show the username"
            | "show me username"
            | "show me the username"
            | "show login name"
            | "show the login name"
            | "show me login name"
            | "show me the login name"
            | "show whoami"
            | "show me whoami"
    )
}


/// Zero-LLM login username from cached `$USER` / `id -un` / `whoami`.
pub fn format_username_gateway() -> String {
    let v = crate::metrics::get_username();
    let v = v.trim();
    if v.is_empty() {
        return "**User** — _no login name right now._".to_string();
    }
    format!("**User** · `{v}`")
}


/// Hardware serial number — not Mac model, not chip, not UUID.
/// Slash + view/open NL (`/serial` · `/serial-number` · `view serial`) match username `/username` ·
/// Mac model `/mac-model` shape; bare `sn` alone stays with the agent (too short).
pub fn looks_like_serial_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 56 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("serializ")
        || n.contains("serialis")
        || n.contains("console")
        || n.contains("port")
        || n.contains("usb")
        || n.contains("cable")
        || n.contains("monitor")
        || n.contains("display")
        || n.contains("uuid")
        || n.contains("json")
        || n.contains("agent")
        || n.contains("chat")
        // Bare "sn" is too short / ambiguous.
        || n == "sn"
        || n == "the sn"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/serial"
            | "/serial-number"
            | "/serialnumber"
            | "/hw-serial"
            | "/hardware-serial"
            | "/device-serial"
            | "/sn"
            | "serial"
            | "the serial"
            | "serial number"
            | "the serial number"
            | "hardware serial"
            | "the hardware serial"
            | "device serial"
            | "the device serial"
            | "hw serial"
            | "the hw serial"
            | "what serial"
            | "what serial is this"
            | "what serial number"
            | "what serial number is this"
            | "what's the serial"
            | "whats the serial"
            | "what is the serial"
            | "what's the serial number"
            | "whats the serial number"
            | "what is the serial number"
            | "what's my serial"
            | "whats my serial"
            | "what is my serial"
            | "what's my serial number"
            | "whats my serial number"
            | "what is my serial number"
            | "which serial"
            | "which serial is this"
            | "which serial number"
            | "my serial"
            | "my serial number"
            // View/see/show me/open/list.
            | "view serial"
            | "view the serial"
            | "view serial number"
            | "view the serial number"
            | "view hardware serial"
            | "view the hardware serial"
            | "see serial"
            | "see the serial"
            | "see serial number"
            | "see the serial number"
            | "open serial"
            | "open the serial"
            | "open serial number"
            | "open the serial number"
            | "open hardware serial"
            | "list serial"
            | "list the serial"
            | "list serial number"
            | "list the serial number"
            | "show serial"
            | "show the serial"
            | "show me serial"
            | "show me the serial"
            | "show serial number"
            | "show the serial number"
            | "show me serial number"
            | "show me the serial number"
    )
}


/// Zero-LLM hardware serial from cached `system_profiler` / `ioreg` / DMI.
pub fn format_serial_gateway() -> String {
    let v = crate::metrics::get_serial_number();
    let v = v.trim();
    if v.is_empty() {
        return "**Serial** — _no serial number right now._".to_string();
    }
    format!("**Serial** · `{v}`")
}


/// Local timezone — not wall-clock “what time”, not uptime / timers.
/// Slash + view/open NL (`/timezone` · `/tz` · `view timezone`) match username `/username` ·
/// hostname `/hostname` shape; bare `/time` / `what time` stay with the agent.
pub fn looks_like_timezone_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 56 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("uptime")
        || n.contains("timer")
        || n.contains("timeout")
        || n.contains("timestamp")
        || n.contains("schedule")
        || n.contains("cron")
        || n.contains("deadline")
        || n.contains("eta")
        || n.contains("meeting")
        || n.contains("calendar")
        || n.contains("convert")
        || n.contains("utc")
        || n.contains("gmt")
        || n.contains("epoch")
        || n.contains("unix")
        || n.contains("json")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        // Wall clock stays with the agent (not this lane).
        || n == "/time"
        || n == "time"
        || n == "the time"
        || n == "what time"
        || n == "what's the time"
        || n == "whats the time"
        || n == "what is the time"
        || n == "what's the time now"
        || n == "whats the time now"
        || n == "what time is it"
        || n == "current time"
        || n == "the current time"
        || n == "local time"
        || n == "the local time"
        || n == "view time"
        || n == "view the time"
        || n == "see time"
        || n == "see the time"
        || n == "open time"
        || n == "open the time"
        || n == "list time"
        || n == "list the time"
        || n == "show time"
        || n == "show the time"
        || n == "show me time"
        || n == "show me the time"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/timezone"
            | "/time-zone"
            | "/time_zone"
            | "/tz"
            | "/local-timezone"
            | "/localtimezone"
            | "/local-tz"
            | "/localtz"
            | "timezone"
            | "the timezone"
            | "time zone"
            | "the time zone"
            | "tz"
            | "the tz"
            | "local timezone"
            | "the local timezone"
            | "local time zone"
            | "the local time zone"
            | "local tz"
            | "the local tz"
            | "what timezone"
            | "what timezone is this"
            | "what timezone am i in"
            | "what timezone do i have"
            | "what's the timezone"
            | "whats the timezone"
            | "what is the timezone"
            | "what's my timezone"
            | "whats my timezone"
            | "what is my timezone"
            | "what time zone"
            | "what time zone is this"
            | "what time zone am i in"
            | "what's the time zone"
            | "whats the time zone"
            | "what is the time zone"
            | "what's my time zone"
            | "whats my time zone"
            | "what is my time zone"
            | "which timezone"
            | "which timezone is this"
            | "which time zone"
            | "which time zone is this"
            | "my timezone"
            | "my time zone"
            | "my tz"
            // View/see/show me/open/list (bare view time stays out).
            | "view timezone"
            | "view the timezone"
            | "view time zone"
            | "view the time zone"
            | "view tz"
            | "view the tz"
            | "view local timezone"
            | "view the local timezone"
            | "see timezone"
            | "see the timezone"
            | "see time zone"
            | "see the time zone"
            | "see tz"
            | "open timezone"
            | "open the timezone"
            | "open time zone"
            | "open the time zone"
            | "open tz"
            | "open the tz"
            | "list timezone"
            | "list the timezone"
            | "list time zone"
            | "list the time zone"
            | "list tz"
            | "show timezone"
            | "show the timezone"
            | "show me timezone"
            | "show me the timezone"
            | "show time zone"
            | "show the time zone"
            | "show me time zone"
            | "show me the time zone"
            | "show tz"
            | "show me tz"
    )
}


/// Zero-LLM local timezone from `/etc/localtime` + chrono Local.
pub fn format_timezone_gateway() -> String {
    let v = crate::metrics::get_timezone();
    let v = v.trim();
    if v.is_empty() {
        return "**Timezone** — _no timezone right now._".to_string();
    }
    format!("**Timezone** · {v}")
}


/// Machine architecture — not Apple chip name, not Architect theme, not software design.
/// Slash + view/open NL (`/arch` · `/architecture` · `view arch`) match timezone `/tz` ·
/// serial `/serial` shape; bare `architect` / theme Architect stay out; `/chip` stays chip.
pub fn looks_like_architecture_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 56 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("software")
        || n.contains("design")
        || n.contains("pattern")
        || n.contains("review")
        || n.contains("theme")
        || n.contains("json")
        || n.contains("chip")
        || n.contains("processor")
        || n.contains("core")
        || n.contains("m1")
        || n.contains("m2")
        || n.contains("m3")
        || n.contains("m4")
        // Theme / product name "Architect" (not "architecture").
        || (n.contains("architect") && !n.contains("architecture"))
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/arch"
            | "/architecture"
            | "/uname-m"
            | "/uname_m"
            | "/machine-arch"
            | "/machinearch"
            | "/cpu-arch"
            | "/cpuarch"
            | "/hw-arch"
            | "/hwarch"
            | "arch"
            | "the arch"
            | "architecture"
            | "the architecture"
            | "cpu architecture"
            | "the cpu architecture"
            | "machine architecture"
            | "the machine architecture"
            | "machine arch"
            | "the machine arch"
            | "cpu arch"
            | "the cpu arch"
            | "hw arch"
            | "the hw arch"
            | "hardware architecture"
            | "the hardware architecture"
            | "uname -m"
            | "uname m"
            | "what arch"
            | "what arch is this"
            | "what architecture"
            | "what architecture is this"
            | "what's the arch"
            | "whats the arch"
            | "what is the arch"
            | "what's the architecture"
            | "whats the architecture"
            | "what is the architecture"
            | "what's my arch"
            | "whats my arch"
            | "what is my arch"
            | "what's my architecture"
            | "whats my architecture"
            | "what is my architecture"
            | "which arch"
            | "which arch is this"
            | "which architecture"
            | "which architecture is this"
            | "my arch"
            | "my architecture"
            | "am i on arm"
            | "am i on arm64"
            | "am i on aarch64"
            | "am i on x86"
            | "am i on x86_64"
            | "am i on intel"
            // View/see/show me/open/list.
            | "view arch"
            | "view the arch"
            | "view architecture"
            | "view the architecture"
            | "view cpu architecture"
            | "view machine arch"
            | "see arch"
            | "see the arch"
            | "see architecture"
            | "see the architecture"
            | "open arch"
            | "open the arch"
            | "open architecture"
            | "open the architecture"
            | "open cpu architecture"
            | "list arch"
            | "list the arch"
            | "list architecture"
            | "list the architecture"
            | "show arch"
            | "show the arch"
            | "show me arch"
            | "show me the arch"
            | "show architecture"
            | "show the architecture"
            | "show me architecture"
            | "show me the architecture"
    )
}


/// Zero-LLM machine arch from cached `uname -m` / consts::ARCH.
pub fn format_architecture_gateway() -> String {
    let v = crate::metrics::get_architecture();
    let v = v.trim();
    if v.is_empty() {
        return "**Arch** — _no architecture right now._".to_string();
    }
    format!("**Arch** · `{v}`")
}


/// System locale / language — not timezone, not programming language, not LLM “language model”.
/// Slash + view/open NL (`/locale` · `/language` · `view locale`) match timezone `/tz` ·
/// arch `/arch` shape; bare `lang` stays with the agent.
pub fn looks_like_locale_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 56 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("programming")
        || n.contains("code")
        || n.contains("script")
        || n.contains("model")
        || n.contains("ollama")
        || n.contains("llm")
        || n.contains("timezone")
        || n.contains("time zone")
        || n == "/tz"
        || n == "tz"
        || n == "the tz"
        // Bare short token — too ambiguous (programming / chat slang).
        || n == "lang"
        || n == "the lang"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/locale"
            | "/apple-locale"
            | "/applelocale"
            | "/system-locale"
            | "/systemlocale"
            | "/lang"
            | "/language"
            | "/system-language"
            | "/systemlanguage"
            | "/apple-language"
            | "/applelanguage"
            | "locale"
            | "the locale"
            | "apple locale"
            | "the apple locale"
            | "system locale"
            | "the system locale"
            | "language"
            | "the language"
            | "system language"
            | "the system language"
            | "apple language"
            | "the apple language"
            | "what locale"
            | "what locale is this"
            | "what language"
            | "what language is this"
            | "what language am i using"
            | "what's the locale"
            | "whats the locale"
            | "what is the locale"
            | "what's my locale"
            | "whats my locale"
            | "what is my locale"
            | "what's the language"
            | "whats the language"
            | "what is the language"
            | "what's my language"
            | "whats my language"
            | "what is my language"
            | "which locale"
            | "which locale is this"
            | "which language"
            | "which language is this"
            | "my locale"
            | "my language"
            | "my system language"
            // View/see/show me/open/list.
            | "view locale"
            | "view the locale"
            | "view language"
            | "view the language"
            | "view system locale"
            | "view apple locale"
            | "see locale"
            | "see the locale"
            | "see language"
            | "see the language"
            | "open locale"
            | "open the locale"
            | "open language"
            | "open the language"
            | "open system locale"
            | "list locale"
            | "list the locale"
            | "list language"
            | "list the language"
            | "show locale"
            | "show the locale"
            | "show me locale"
            | "show me the locale"
            | "show language"
            | "show the language"
            | "show me language"
            | "show me the language"
    )
}


/// Zero-LLM system locale from cached AppleLocale / LANG.
pub fn format_locale_gateway() -> String {
    let v = crate::metrics::get_locale();
    let v = v.trim();
    if v.is_empty() {
        return "**Locale** — _no locale right now._".to_string();
    }
    format!("**Locale** · `{v}`")
}
/// Kernel release — not machine arch (`uname -m`), not macOS ProductVersion, not full `uname`.
/// Slash + view/open NL (`/kernel` · `/uname-r` · `view kernel`) match arch `/arch` ·
/// locale `/locale` shape; bare `uname` / `/uname` stay with the agent; `/uname-m` stays arch.
pub fn looks_like_kernel_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 56 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("panic")
        || n.contains("module")
        || n.contains("driver")
        || n.contains("compile")
        || n.contains("build")
        || n.contains("exploit")
        || n.contains("cve")
        || n.contains("patch")
        || n.contains("update")
        || n.contains("macos")
        || n.contains("os version")
        || n.contains("system version")
        // Arch lane owns uname -m / machine arch.
        || n.contains("uname-m")
        || n.contains("uname_m")
        || n.contains("uname -m")
        || n.contains("architecture")
        || n.contains("machine arch")
        || n.contains("cpu arch")
        || n.contains("hw arch")
        // Bare / short uname dump stays with the agent.
        || n == "uname"
        || n == "the uname"
        || n == "/uname"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/kernel"
            | "/kernel-release"
            | "/kernelrelease"
            | "/kernel-version"
            | "/kernelversion"
            | "/uname-r"
            | "/uname_r"
            | "/kern"
            | "kernel"
            | "the kernel"
            | "kernel release"
            | "the kernel release"
            | "kernel version"
            | "the kernel version"
            | "kernel rev"
            | "the kernel rev"
            | "uname -r"
            | "uname r"
            | "what kernel"
            | "what kernel is this"
            | "what kernel release"
            | "what kernel version"
            | "what's the kernel"
            | "whats the kernel"
            | "what is the kernel"
            | "what's my kernel"
            | "whats my kernel"
            | "what is my kernel"
            | "what's the kernel release"
            | "whats the kernel release"
            | "what is the kernel release"
            | "what's the kernel version"
            | "whats the kernel version"
            | "what is the kernel version"
            | "which kernel"
            | "which kernel is this"
            | "my kernel"
            | "my kernel release"
            | "my kernel version"
            // View/see/show me/open/list.
            | "view kernel"
            | "view the kernel"
            | "view kernel release"
            | "view the kernel release"
            | "view kernel version"
            | "view the kernel version"
            | "see kernel"
            | "see the kernel"
            | "see kernel release"
            | "see the kernel release"
            | "open kernel"
            | "open the kernel"
            | "open kernel release"
            | "open the kernel release"
            | "open kernel version"
            | "list kernel"
            | "list the kernel"
            | "list kernel release"
            | "list the kernel release"
            | "show kernel"
            | "show the kernel"
            | "show me kernel"
            | "show me the kernel"
            | "show kernel release"
            | "show the kernel release"
            | "show me kernel release"
            | "show me the kernel release"
            | "show kernel version"
            | "show the kernel version"
            | "show me kernel version"
            | "show me the kernel version"
    )
}


/// Zero-LLM kernel release from cached `uname -r`.
pub fn format_kernel_gateway() -> String {
    let v = crate::metrics::get_kernel_release();
    let v = v.trim();
    if v.is_empty() {
        return "**Kernel** — _no kernel release right now._".to_string();
    }
    format!("**Kernel** · `{v}`")
}


/// CPU core count — not chip name, not P/E cluster clocks, not CPU usage ring.
/// Slash + view/open NL (`/cores` · `/ncpu` · `view cores`) match RAM `/ram-size` ·
/// locale `/locale` shape; `/p-core` · `/e-core` stay cluster clocks; `/chip` stays name;
/// bare `core` stays with the agent.
pub fn looks_like_cpu_cores_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 56 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("freq")
        || n.contains("clock")
        || n.contains("ghz")
        || n.contains("fast")
        || n.contains("usage")
        || n.contains("percent")
        || n.contains('%')
        || n.contains("hot")
        || n.contains("high")
        || n.contains("busy")
        || n.contains("gpu")
        || n.contains("process")
        // P/E cluster clock lanes own these (do not use bare "e core" — matches "the core").
        || n.contains("p-core")
        || n.contains("p core")
        || n.contains("e-core")
        || n.contains("performance")
        || n.contains("efficiency")
        || n.contains("/pcore")
        || n.contains("/ecore")
        || n.starts_with("e core")
        || n.starts_with("e-cores")
        || n.contains(" e core")
        || n.contains(" e-core")
        || n == "/e-cores"
        || n == "/ecore"
        // Chip name owns /chip; CPU ring owns bare /cpu · view cpu.
        || n.contains("chip")
        || n == "/cpu"
        || n == "cpu"
        || n == "the cpu"
        || n == "view cpu"
        || n == "view the cpu"
        || n == "see cpu"
        || n == "see the cpu"
        || n == "open cpu"
        || n == "open the cpu"
        || n == "list cpu"
        || n == "list the cpu"
        // Bare short token — too ambiguous.
        || n == "core"
        || n == "the core"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/cores"
            | "/core-count"
            | "/corecount"
            | "/cpu-cores"
            | "/cpucores"
            | "/cpu-count"
            | "/cpucount"
            | "/ncpu"
            | "/logical-cpus"
            | "/logicalcpus"
            | "/logical-cores"
            | "/logicalcores"
            | "/physical-cpus"
            | "/physicalcpus"
            | "/hw-ncpu"
            | "/hwncpu"
            | "cores"
            | "the cores"
            | "cpu cores"
            | "the cpu cores"
            | "core count"
            | "the core count"
            | "cpu count"
            | "the cpu count"
            | "cpu core count"
            | "the cpu core count"
            | "logical cpus"
            | "the logical cpus"
            | "logical cores"
            | "the logical cores"
            | "physical cpus"
            | "the physical cpus"
            | "physical cores"
            | "the physical cores"
            | "ncpu"
            | "the ncpu"
            | "how many cores"
            | "how many cpu cores"
            | "how many logical cores"
            | "how many physical cores"
            | "how many cpus"
            | "number of cores"
            | "number of cpu cores"
            | "what's the core count"
            | "whats the core count"
            | "what is the core count"
            | "what's my core count"
            | "whats my core count"
            | "what is my core count"
            | "what's the cpu count"
            | "whats the cpu count"
            | "what is the cpu count"
            | "how many cores do i have"
            | "how many cores does this mac have"
            | "how many cores does this have"
            // View/see/show me/open/list.
            | "view cores"
            | "view the cores"
            | "view core count"
            | "view the core count"
            | "view cpu cores"
            | "view the cpu cores"
            | "view cpu count"
            | "view ncpu"
            | "see cores"
            | "see the cores"
            | "see core count"
            | "see the core count"
            | "see cpu cores"
            | "see cpu count"
            | "open cores"
            | "open the cores"
            | "open core count"
            | "open the core count"
            | "open cpu cores"
            | "open cpu count"
            | "open ncpu"
            | "list cores"
            | "list the cores"
            | "list core count"
            | "list the core count"
            | "list cpu cores"
            | "list cpu count"
            | "show cores"
            | "show the cores"
            | "show me cores"
            | "show me the cores"
            | "show core count"
            | "show the core count"
            | "show me core count"
            | "show me the core count"
            | "show cpu cores"
            | "show me cpu cores"
            | "show cpu count"
            | "show me cpu count"
    )
}


/// Zero-LLM CPU core count from cached profiler / sysctl.
pub fn format_cpu_cores_gateway() -> String {
    let v = crate::metrics::get_cpu_cores();
    let v = v.trim();
    if v.is_empty() {
        return "**Cores** — _no core count right now._".to_string();
    }
    format!("**Cores** · {v}")
}


/// Last boot wall-clock — not Mac Up duration, not app process uptime.
/// Slash + view/open NL (`/boot` · `/boot-time` · `view boot time`) match cores `/cores` ·
/// kernel `/kernel` shape; bare `/uptime` / `view uptime` / `what's the uptime` stay Mac Up;
/// `/app-uptime` stays process uptime; boot disk / boot volume stay with the agent.
pub fn looks_like_boot_time_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 64 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("disk")
        || n.contains("volume")
        || n.contains("camp")
        || n.contains("safe boot")
        || n.contains("recovery")
        || n.contains("firmware")
        || n.contains("efi")
        || n.contains("loader")
        || n.contains("partition")
        || n.contains("app uptime")
        || n.contains("process uptime")
        || n.contains("bot uptime")
        || n.contains("mac-stats uptime")
        // Mac Up duration chip owns bare uptime phrases.
        || n == "/uptime"
        || n == "/up"
        || n == "uptime"
        || n == "the uptime"
        || n == "up time"
        || n == "system uptime"
        || n == "view uptime"
        || n == "view the uptime"
        || n == "see uptime"
        || n == "see the uptime"
        || n == "open uptime"
        || n == "open the uptime"
        || n == "list uptime"
        || n == "list the uptime"
        || n == "what's the uptime"
        || n == "whats the uptime"
        || n == "what is the uptime"
        || n == "how's the uptime"
        || n == "hows the uptime"
        || n == "how long up"
        || n == "how long has it been up"
        || n == "how long has the mac been up"
        // Bare "boot" alone is ambiguous (boot disk / recovery).
        || n == "boot"
        || n == "the boot"
        || n == "reboot"
        || n == "the reboot"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/boot"
            | "/boot-time"
            | "/boottime"
            | "/boot-timestamp"
            | "/boottimestamp"
            | "/last-boot"
            | "/lastboot"
            | "/last-reboot"
            | "/lastreboot"
            | "/reboot-time"
            | "/reboottime"
            | "boot time"
            | "the boot time"
            | "boot timestamp"
            | "the boot timestamp"
            | "last boot"
            | "the last boot"
            | "last reboot"
            | "the last reboot"
            | "last boot time"
            | "the last boot time"
            | "last reboot time"
            | "the last reboot time"
            | "reboot time"
            | "the reboot time"
            | "when did this boot"
            | "when did it boot"
            | "when did the mac boot"
            | "when did this mac boot"
            | "when was the last boot"
            | "when was the last reboot"
            | "when was last boot"
            | "when was last reboot"
            | "what's the boot time"
            | "whats the boot time"
            | "what is the boot time"
            | "what's my boot time"
            | "whats my boot time"
            | "what is my boot time"
            | "what's the last boot"
            | "whats the last boot"
            | "what is the last boot"
            | "what's the last reboot"
            | "whats the last reboot"
            | "what is the last reboot"
            | "when did we last reboot"
            | "when did i last reboot"
            // View/see/show me/open/list.
            | "view boot"
            | "view the boot"
            | "view boot time"
            | "view the boot time"
            | "view last boot"
            | "view the last boot"
            | "view last reboot"
            | "view the last reboot"
            | "view reboot time"
            | "see boot"
            | "see the boot"
            | "see boot time"
            | "see the boot time"
            | "see last boot"
            | "see the last boot"
            | "see last reboot"
            | "open boot"
            | "open the boot"
            | "open boot time"
            | "open the boot time"
            | "open last boot"
            | "open the last boot"
            | "open last reboot"
            | "open reboot time"
            | "list boot"
            | "list the boot"
            | "list boot time"
            | "list the boot time"
            | "list last boot"
            | "list last reboot"
            | "show boot"
            | "show the boot"
            | "show me boot"
            | "show me the boot"
            | "show boot time"
            | "show the boot time"
            | "show me boot time"
            | "show me the boot time"
            | "show last boot"
            | "show the last boot"
            | "show me last boot"
            | "show me the last boot"
            | "show last reboot"
            | "show the last reboot"
            | "show me last reboot"
            | "show me the last reboot"
            | "show reboot time"
            | "show the reboot time"
            | "show me reboot time"
            | "show me the reboot time"
    )
}


/// Zero-LLM last boot wall-clock from uptime + local clock.
pub fn format_boot_time_gateway() -> String {
    let v = crate::metrics::get_boot_time();
    let v = v.trim();
    if v.is_empty() {
        return "**Boot** — _no boot time right now._".to_string();
    }
    format!("**Boot** · `{v}`")
}


/// Primary local LAN IPv4 — not public/WAN IP, not hostname.
/// Slash + view/open NL (`/ip` · `/local-ip` · `view local ip`) match boot `/boot` ·
/// hostname `/hostname` shape; bare `ip` / `what's my ip` / public·wan·external stay with the agent.
pub fn looks_like_local_ip_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 56 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("public")
        || n.contains("external")
        || n.contains("wan")
        || n.contains("internet")
        || n.contains("website")
        || n.contains("dns")
        || n.contains("gateway")
        || n.contains("router")
        || n.contains("vpn")
        || n.contains("hostname")
        || n.contains("computer name")
        || n.contains("machine name")
        // Bare short / classic public-IP asks stay with the agent.
        || n == "ip"
        || n == "the ip"
        || n == "my ip"
        || n == "what's my ip"
        || n == "whats my ip"
        || n == "what is my ip"
        || n == "what's the ip"
        || n == "whats the ip"
        || n == "what is the ip"
        || n == "what is my ip address"
        || n == "what's my ip address"
        || n == "whats my ip address"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/ip"
            | "/local-ip"
            | "/localip"
            | "/lan-ip"
            | "/lanip"
            | "/ipv4"
            | "/primary-ip"
            | "/primaryip"
            | "/en0-ip"
            | "/en0ip"
            | "/en0"
            | "local ip"
            | "the local ip"
            | "local ipv4"
            | "the local ipv4"
            | "lan ip"
            | "the lan ip"
            | "lan address"
            | "the lan address"
            | "local address"
            | "the local address"
            | "primary ip"
            | "the primary ip"
            | "ipv4"
            | "the ipv4"
            | "en0 ip"
            | "the en0 ip"
            | "what's my local ip"
            | "whats my local ip"
            | "what is my local ip"
            | "what's the local ip"
            | "whats the local ip"
            | "what is the local ip"
            | "what's my lan ip"
            | "whats my lan ip"
            | "what is my lan ip"
            | "what's my ipv4"
            | "whats my ipv4"
            | "what is my ipv4"
            | "what's the lan ip"
            | "whats the lan ip"
            | "what is the lan ip"
            | "my local ip"
            | "my lan ip"
            | "my ipv4"
            // View/see/show me/open/list.
            | "view ip"
            | "view the ip"
            | "view local ip"
            | "view the local ip"
            | "view lan ip"
            | "view the lan ip"
            | "view ipv4"
            | "view the ipv4"
            | "view primary ip"
            | "see ip"
            | "see the ip"
            | "see local ip"
            | "see the local ip"
            | "see lan ip"
            | "see ipv4"
            | "open ip"
            | "open the ip"
            | "open local ip"
            | "open the local ip"
            | "open lan ip"
            | "open ipv4"
            | "list ip"
            | "list the ip"
            | "list local ip"
            | "list the local ip"
            | "list lan ip"
            | "list ipv4"
            | "show ip"
            | "show the ip"
            | "show me ip"
            | "show me the ip"
            | "show local ip"
            | "show the local ip"
            | "show me local ip"
            | "show me the local ip"
            | "show lan ip"
            | "show the lan ip"
            | "show me lan ip"
            | "show me the lan ip"
            | "show ipv4"
            | "show the ipv4"
            | "show me ipv4"
            | "show me the ipv4"
    )
}


/// Zero-LLM primary local LAN IPv4 from default-route iface + ipconfig.
pub fn format_local_ip_gateway() -> String {
    let v = crate::metrics::get_local_ipv4();
    let v = v.trim();
    if v.is_empty() {
        return "**IP** — _no local IPv4 right now._".to_string();
    }
    format!("**IP** · `{v}`")
}


/// Default IPv4 gateway / router — not Discord bot gateway, not local LAN IP.
/// Slash + view/open NL (`/gateway` · `/router` · `view default gateway`) match `/ip` shape;
/// Discord `/discord` · `gateway status` · `gateway ready` stay Discord; bare `gateway` /
/// bare `router` stay with the agent; `/ip` stays LAN address.
pub fn looks_like_default_gateway_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 56 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("discord")
        || n.contains("bot")
        || n.contains("shard")
        || n.contains("ready")
        || n.contains("offline")
        || n.contains("connected")
        || n.contains("status")
        || n.contains("public")
        || n.contains("external")
        || n.contains("wan")
        || n.contains("internet")
        || n.contains("dns")
        || n.contains("vpn")
        || n.contains("hostname")
        || n.contains("local ip")
        || n.contains("lan ip")
        // Bare short / Discord-adjacent asks stay with the agent (or Discord chip).
        || n == "gateway"
        || n == "the gateway"
        || n == "router"
        || n == "the router"
        || n == "gw"
        || n == "the gw"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/gateway"
            | "/default-gateway"
            | "/defaultgateway"
            | "/gw"
            | "/router"
            | "/router-ip"
            | "/routerip"
            | "/gateway-ip"
            | "/gatewayip"
            | "/default-route"
            | "/defaultroute"
            | "default gateway"
            | "the default gateway"
            | "default route"
            | "the default route"
            | "gateway ip"
            | "the gateway ip"
            | "gateway address"
            | "the gateway address"
            | "router ip"
            | "the router ip"
            | "router address"
            | "the router address"
            | "router gateway"
            | "the router gateway"
            | "what's my gateway"
            | "whats my gateway"
            | "what is my gateway"
            | "what's the gateway"
            | "whats the gateway"
            | "what is the gateway"
            | "what's my default gateway"
            | "whats my default gateway"
            | "what is my default gateway"
            | "what's the default gateway"
            | "whats the default gateway"
            | "what is the default gateway"
            | "what's my router"
            | "whats my router"
            | "what is my router"
            | "what's the router ip"
            | "whats the router ip"
            | "what is the router ip"
            | "my default gateway"
            | "my gateway ip"
            | "my router ip"
            // View/see/show me/open/list.
            | "view gateway"
            | "view the gateway"
            | "view default gateway"
            | "view the default gateway"
            | "view gateway ip"
            | "view the gateway ip"
            | "view router"
            | "view the router"
            | "view router ip"
            | "view the router ip"
            | "view default route"
            | "see gateway"
            | "see the gateway"
            | "see default gateway"
            | "see the default gateway"
            | "see gateway ip"
            | "see router"
            | "see router ip"
            | "open gateway"
            | "open the gateway"
            | "open default gateway"
            | "open the default gateway"
            | "open gateway ip"
            | "open router"
            | "open the router"
            | "open router ip"
            | "list gateway"
            | "list the gateway"
            | "list default gateway"
            | "list the default gateway"
            | "list gateway ip"
            | "list router"
            | "list router ip"
            | "show gateway"
            | "show the gateway"
            | "show me gateway"
            | "show me the gateway"
            | "show default gateway"
            | "show the default gateway"
            | "show me default gateway"
            | "show me the default gateway"
            | "show gateway ip"
            | "show the gateway ip"
            | "show me gateway ip"
            | "show me the gateway ip"
            | "show router"
            | "show the router"
            | "show me router"
            | "show me the router"
            | "show router ip"
            | "show the router ip"
            | "show me router ip"
            | "show me the router ip"
    )
}


/// Zero-LLM default IPv4 gateway from `route -n get default`.
pub fn format_default_gateway_report() -> String {
    let v = crate::metrics::get_default_gateway();
    let v = v.trim();
    if v.is_empty() {
        return "**Gateway** — _no default gateway right now._".to_string();
    }
    format!("**Gateway** · `{v}`")
}


/// DNS nameservers — not default gateway, not local LAN IP, not dig/lookup.
/// Slash + view/open NL (`/dns` · `/nameserver` · `view dns servers`) match `/gateway` ·
/// `/ip` shape; bare `dns` / lookup·resolve·dig·flush stay with the agent; `/gateway` stays
/// default route; `/ip` stays LAN address.
pub fn looks_like_dns_servers_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 56 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("lookup")
        || n.contains("nslookup")
        || n.contains("dig")
        || n.contains("flush")
        || n.contains("cache")
        || n.contains("query")
        || n.contains("record")
        || n.contains("mx")
        || n.contains("public")
        || n.contains("external")
        || n.contains("wan")
        || n.contains("internet")
        || n.contains("vpn")
        || n.contains("hostname")
        || n.contains("gateway")
        || n.contains("router")
        || n.contains("local ip")
        || n.contains("lan ip")
        // "resolve" as a verb — do not veto `/resolver` · `/resolvers`.
        || n == "resolve"
        || n.starts_with("resolve ")
        || n.ends_with(" resolve")
        || n.contains(" resolve ")
        || n.contains("dns resolve")
        || n.contains("resolve dns")
        // Bare short / tooling asks stay with the agent.
        || n == "dns"
        || n == "the dns"
        || n == "nameserver"
        || n == "the nameserver"
        || n == "nameservers"
        || n == "the nameservers"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/dns"
            | "/dns-servers"
            | "/dnsservers"
            | "/dns-server"
            | "/dnsserver"
            | "/nameserver"
            | "/nameservers"
            | "/ns"
            | "/dns-ns"
            | "/resolver"
            | "/resolvers"
            | "dns servers"
            | "the dns servers"
            | "dns server"
            | "the dns server"
            | "dns addresses"
            | "the dns addresses"
            | "dns address"
            | "the dns address"
            | "name servers"
            | "the name servers"
            | "name server"
            | "the name server"
            | "my dns servers"
            | "my dns server"
            | "my nameservers"
            | "my nameserver"
            | "what's my dns"
            | "whats my dns"
            | "what is my dns"
            | "what's the dns"
            | "whats the dns"
            | "what is the dns"
            | "what's my dns server"
            | "whats my dns server"
            | "what is my dns server"
            | "what's my dns servers"
            | "whats my dns servers"
            | "what is my dns servers"
            | "what's the dns server"
            | "whats the dns server"
            | "what is the dns server"
            | "what's the dns servers"
            | "whats the dns servers"
            | "what is the dns servers"
            | "what's my nameserver"
            | "whats my nameserver"
            | "what is my nameserver"
            | "what's my nameservers"
            | "whats my nameservers"
            | "what is my nameservers"
            | "which dns"
            | "which dns server"
            | "which dns servers"
            | "which nameserver"
            | "which nameservers"
            // View/see/show me/open/list.
            | "view dns"
            | "view the dns"
            | "view dns servers"
            | "view the dns servers"
            | "view dns server"
            | "view the dns server"
            | "view nameserver"
            | "view the nameserver"
            | "view nameservers"
            | "view the nameservers"
            | "see dns"
            | "see the dns"
            | "see dns servers"
            | "see the dns servers"
            | "see dns server"
            | "see nameserver"
            | "see nameservers"
            | "open dns"
            | "open the dns"
            | "open dns servers"
            | "open the dns servers"
            | "open dns server"
            | "open nameserver"
            | "open nameservers"
            | "list dns"
            | "list the dns"
            | "list dns servers"
            | "list the dns servers"
            | "list dns server"
            | "list nameserver"
            | "list nameservers"
            | "show dns"
            | "show the dns"
            | "show me dns"
            | "show me the dns"
            | "show dns servers"
            | "show the dns servers"
            | "show me dns servers"
            | "show me the dns servers"
            | "show dns server"
            | "show the dns server"
            | "show me dns server"
            | "show me the dns server"
            | "show nameserver"
            | "show the nameserver"
            | "show me nameserver"
            | "show me the nameserver"
            | "show nameservers"
            | "show the nameservers"
            | "show me nameservers"
            | "show me the nameservers"
    )
}


/// Zero-LLM DNS nameservers from the first `scutil --dns` resolver.
pub fn format_dns_servers_report() -> String {
    let v = crate::metrics::get_dns_servers();
    let v = v.trim();
    if v.is_empty() {
        return "**DNS** — _no nameservers right now._".to_string();
    }
    format!("**DNS** · `{v}`")
}


/// SSD / disk total capacity — not the used-% chip, not free-bytes, not Disk Cleanup.
/// "Why" stays with the agent. "How much disk is used" / bare `/ssd` stay the percent chip.
/// Slash + view/open NL (`/ssd-size` · `/disk-size` · `view ssd size`) match RAM `/ram-size` ·
/// Bat `/battery` shape; bare `view disk` / `open disk` stay Disk Cleanup; `/free` stays free-bytes.
pub fn looks_like_disk_capacity_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 56 {
        return false;
    }
    if n.contains("why")
        || n.contains("path")
        || n.contains(" age")
        || n.ends_with(" age")
        || n.contains("cleanup")
        || n.contains("quarantine")
        || n.contains("reclaim")
        || n.contains("free")
        || n.contains("left")
        || n.contains("used")
        || n.contains("usage")
        || n.contains("percent")
        || n.contains('%')
        || n.contains("full")
        || n.contains("hot")
        || n.contains("json")
        || n.contains("notes")
        || n.contains("memory")
        || n.contains("process")
        || n.contains("folder")
        || n.contains("file")
        || n.contains("how to")
        || n.contains("explain")
        // SSD % chip owns bare `/ssd` · `/storage` · `view ssd` · `how much storage`.
        || n == "/ssd"
        || n == "/storage"
        || n == "ssd"
        || n == "storage"
        || n == "the ssd"
        || n == "the storage"
        || n == "disk"
        || n == "the disk"
        || n == "view ssd"
        || n == "view the ssd"
        || n == "view storage"
        || n == "view the storage"
        || n == "see ssd"
        || n == "see the ssd"
        || n == "see storage"
        || n == "see the storage"
        || n == "open ssd"
        || n == "open the ssd"
        || n == "open storage"
        || n == "open the storage"
        || n == "list ssd"
        || n == "list the ssd"
        || n == "list storage"
        || n == "list the storage"
        || n == "how much ssd"
        || n == "how much storage"
        || n == "how much disk"
        // Disk Cleanup owns bare `view disk` / `open disk`.
        || n == "view disk"
        || n == "view the disk"
        || n == "see disk"
        || n == "see the disk"
        || n == "open disk"
        || n == "open the disk"
        || n == "list disk"
        || n == "list the disk"
        || n == "/disk"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/ssd-size"
            | "/ssdsize"
            | "/disk-size"
            | "/disksize"
            | "/storage-size"
            | "/storagesize"
            | "/ssd-capacity"
            | "/ssdcapacity"
            | "/disk-capacity"
            | "/diskcapacity"
            | "/storage-capacity"
            | "/storagecapacity"
            | "/total-ssd"
            | "/totalssd"
            | "/total-disk"
            | "/totaldisk"
            | "/total-storage"
            | "/totalstorage"
            | "ssd size"
            | "the ssd size"
            | "disk size"
            | "the disk size"
            | "storage size"
            | "the storage size"
            | "ssd capacity"
            | "the ssd capacity"
            | "disk capacity"
            | "the disk capacity"
            | "storage capacity"
            | "the storage capacity"
            | "total ssd"
            | "the total ssd"
            | "total disk"
            | "the total disk"
            | "total storage"
            | "the total storage"
            | "how big is ssd"
            | "how big is the ssd"
            | "how big is disk"
            | "how big is the disk"
            | "how big is storage"
            | "how big is the storage"
            | "how large is ssd"
            | "how large is the ssd"
            | "how large is disk"
            | "how large is the disk"
            | "how large is storage"
            | "how large is the storage"
            | "how many gb of ssd"
            | "how many gb of disk"
            | "how many gb of storage"
            | "how many gigabytes of ssd"
            | "how many gigabytes of disk"
            | "how many gigabytes of storage"
            | "how much ssd does this mac have"
            | "how much disk does this mac have"
            | "how much storage does this mac have"
            | "what's the ssd size"
            | "whats the ssd size"
            | "what is the ssd size"
            | "what's the disk size"
            | "whats the disk size"
            | "what is the disk size"
            | "what's the storage size"
            | "whats the storage size"
            | "what is the storage size"
            // View/see/show me/open/list (bare view ssd / view disk stay % / Cleanup).
            | "view ssd size"
            | "view the ssd size"
            | "view ssd capacity"
            | "view the ssd capacity"
            | "view disk size"
            | "view the disk size"
            | "view disk capacity"
            | "view the disk capacity"
            | "view storage size"
            | "view the storage size"
            | "view storage capacity"
            | "view the storage capacity"
            | "view total ssd"
            | "view the total ssd"
            | "view total disk"
            | "view the total disk"
            | "view total storage"
            | "view the total storage"
            | "see ssd size"
            | "see the ssd size"
            | "see ssd capacity"
            | "see disk size"
            | "see disk capacity"
            | "see storage size"
            | "see storage capacity"
            | "see total ssd"
            | "see total disk"
            | "see total storage"
            | "open ssd size"
            | "open the ssd size"
            | "open ssd capacity"
            | "open the ssd capacity"
            | "open disk size"
            | "open the disk size"
            | "open disk capacity"
            | "open the disk capacity"
            | "open storage size"
            | "open the storage size"
            | "open storage capacity"
            | "open the storage capacity"
            | "open total ssd"
            | "open total disk"
            | "open total storage"
            | "list ssd size"
            | "list the ssd size"
            | "list ssd capacity"
            | "list disk size"
            | "list disk capacity"
            | "list storage size"
            | "list storage capacity"
            | "list total ssd"
            | "list total disk"
            | "list total storage"
    )
}


/// Zero-LLM SSD capacity (total bytes + used). Percent chip still owns "how much ssd".
pub fn format_disk_capacity_gateway() -> String {
    match crate::metrics::get_system_disk_bytes() {
        Some((total, available)) if total > 0 => {
            let used_bytes = total.saturating_sub(available);
            let tot = crate::commands::disk_cleanup::format_bytes(total);
            let used = crate::commands::disk_cleanup::format_bytes(used_bytes);
            format!("**SSD** · {tot} capacity · {used} used")
        }
        _ => "**SSD** — _no disk size reading right now._".to_string(),
    }
}


/// Free disk bytes — not the used-% chip, not Disk Cleanup, not a config file size.
/// "Why" stays with the agent. "How much disk is used" stays the SSD percent chip.
/// Bare "free space" / "disk free" stay the percent chip.
/// Slash + view/open NL (`/free` · `/ssd-free` · `view free space`) match Bat `/battery` ·
/// watts `/watts` · SSD `/storage` shape; bare `view disk` / `open disk` stay Disk Cleanup.
pub fn looks_like_disk_free_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 56 {
        return false;
    }
    if n.contains("why")
        || n.contains("used")
        || n.contains("usage")
        || n.contains("percent")
        || n.contains('%')
        || n.contains("full")
        || n.contains("hot")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.contains("path")
        || n.contains(" age")
        || n.ends_with(" age")
        || n.contains(" size")
        || n.ends_with(" size")
        || n.contains("json")
        || n.contains("notes")
        || n.contains("memory")
        || n.contains("process")
        || n.contains("how to")
        || n.contains("explain")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/free"
            | "/ssd-free"
            | "/ssdfree"
            | "/disk-free"
            | "/diskfree"
            | "/free-space"
            | "/freespace"
            | "/space-left"
            | "/spaceleft"
            | "how much free space"
            | "how much space is left"
            | "how much space left"
            | "how much disk is free"
            | "how much disk space is left"
            | "how much disk space left"
            | "how much storage is free"
            | "how much storage is left"
            | "how much storage left"
            | "storage left"
            | "disk space left"
            | "free disk space"
            | "how many gb free"
            | "how many gb are free"
            | "how many gigabytes free"
            | "how many gb of free space"
            | "how much ssd is free"
            | "how much ssd free"
            | "space left on disk"
            | "space left on the disk"
            // View/see/show me/open/list (show me → free space stays SSD %; keep view/open prefixes).
            | "view free space"
            | "view the free space"
            | "view free disk space"
            | "view disk free"
            | "view the disk free"
            | "view ssd free"
            | "view the ssd free"
            | "view storage free"
            | "view the storage free"
            | "view space left"
            | "view the space left"
            | "see free space"
            | "see the free space"
            | "see free disk space"
            | "see disk free"
            | "see ssd free"
            | "see storage free"
            | "see space left"
            | "open free space"
            | "open the free space"
            | "open free disk space"
            | "open disk free"
            | "open ssd free"
            | "open storage free"
            | "open space left"
            | "list free space"
            | "list the free space"
            | "list free disk space"
            | "list disk free"
            | "list ssd free"
            | "list storage free"
            | "list space left"
            | "list the space left"
            // After normalize strips `show` / `show me`: `show me how much free space` →
            // `how much free space` (above). Bare `show free space` → `free space` stays SSD %.
    )
}


/// Zero-LLM free disk bytes (same volume as the SSD % chip).
pub fn format_disk_free_gateway() -> String {
    match crate::metrics::get_system_disk_bytes() {
        Some((total, available)) if total > 0 => {
            let free = crate::commands::disk_cleanup::format_bytes(available);
            let tot = crate::commands::disk_cleanup::format_bytes(total);
            format!("**SSD** · {free} free · {tot} total")
        }
        _ => "**SSD** — _no free-space reading right now._".to_string(),
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


pub(super) fn details_load_is_hot(load_1: f64) -> bool {
    load_1 >= OPS_DETAILS_LOAD_HOT
}


pub(super) fn details_ram_is_hot(ram_pct: f32) -> bool {
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
        || n == "view hot details"
        || n == "see hot details"
        || n == "open hot details"
        || n == "list the hot details"
        || n == "what's hot on details"
        || n == "whats hot on details"
    {
        return DetailsListFilter::Hot;
    }
    DetailsListFilter::All
}


/// Focused Load 1m asks — not full `/details` / `/load` (those stay the three-row panel).
/// Parity with `load 5` / `load 15` chips: `load 1` / `1-minute load` / `/load-1m` stay here.
/// Slash + view/see/show me/open/list-the NL (v0.1.2927) — parity with Load 5m `/load-5m` · `view load 5`.
/// Exact `open load 1` / `open the 1 minute load` only (not bare `open load` → Details).
/// "Why is the load high" stays with the agent.
pub fn looks_like_load_chip_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    if n.contains("why")
        || n.contains("process")
        || n.contains("download")
        || n.contains("payload")
        || n.contains("reload")
        || n.contains("upload")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains("how to")
        || n.contains("explain")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "is the load high"
            | "is load high"
            | "how's the load"
            | "hows the load"
            | "how high is the load"
            | "how high is load"
            | "how loaded"
            | "how loaded is it"
            | "how loaded is the mac"
            | "how loaded is the system"
            | "is the system loaded"
            | "is the mac loaded"
            | "load high"
            // Explicit 1m window — same shape as load 5 / load 15 (not full `/load`).
            | "/load-1m"
            | "/load-1"
            | "/load1"
            | "/load1m"
            | "load 1"
            | "load 1m"
            | "1m load"
            | "1 minute load"
            | "1-minute load"
            | "one minute load"
            | "the 1 minute load"
            | "what's the 1 minute load"
            | "whats the 1 minute load"
            | "what is the 1 minute load"
            | "how's the 1 minute load"
            | "hows the 1 minute load"
            | "is the 1 minute load high"
            | "is 1 minute load high"
            | "view load 1"
            | "view load 1m"
            | "view the 1 minute load"
            | "view 1 minute load"
            | "view 1m load"
            | "see load 1"
            | "see load 1m"
            | "see the 1 minute load"
            | "see 1 minute load"
            | "see 1m load"
            | "show load 1"
            | "show load 1m"
            | "show the 1 minute load"
            | "show 1 minute load"
            | "show me load 1"
            | "show me load 1m"
            | "show me the 1 minute load"
            | "show me 1 minute load"
            | "open load 1"
            | "open load 1m"
            | "open the 1 minute load"
            | "open 1 minute load"
            | "open 1m load"
            | "list load 1"
            | "list load 1m"
            | "list the 1 minute load"
            | "list 1 minute load"
            | "list the 1m load"
    )
}


/// Zero-LLM Load 1m reply (Details glance hot at Load≥4; not the full Details panel).
pub fn format_load_chip_gateway() -> String {
    let d = crate::metrics::get_cpu_details();
    let hot_mark = if details_load_is_hot(d.load_1) {
        " · hot"
    } else {
        ""
    };
    format!("**Load** · {:.2}{hot_mark}", d.load_1)
}


/// 5-minute or 15-minute load — not the 1-minute chip and not full `/load`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadWindowChip {
    Five,
    Fifteen,
}


/// Focused 5m / 15m load asks. "Why" stays with the agent. `/load` stays the full panel.
/// Slash + view/see/show me/open/list-the NL (v0.1.2924) — parity with Load 1m `/load-1m`.
/// Exact `open load 5` / `open the 5 minute load` only (not bare `open load` → Details).
pub fn parse_load_window_chip(content: &str) -> Option<LoadWindowChip> {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return None;
    }
    if n.contains("why")
        || n.contains("process")
        || n.contains("download")
        || n.contains("payload")
        || n.contains("reload")
        || n.contains("upload")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains("how to")
        || n.contains("explain")
    {
        return None;
    }
    if matches!(
        n.as_str(),
        "/load-5m"
            | "/load-5"
            | "/load5"
            | "/load5m"
            | "5 minute load"
            | "5-minute load"
            | "five minute load"
            | "load 5"
            | "load 5m"
            | "5m load"
            | "the 5 minute load"
            | "what's the 5 minute load"
            | "whats the 5 minute load"
            | "what is the 5 minute load"
            | "how's the 5 minute load"
            | "hows the 5 minute load"
            | "is the 5 minute load high"
            | "is 5 minute load high"
            | "view load 5"
            | "view load 5m"
            | "view the 5 minute load"
            | "view 5 minute load"
            | "view 5m load"
            | "see load 5"
            | "see load 5m"
            | "see the 5 minute load"
            | "see 5 minute load"
            | "see 5m load"
            | "show load 5"
            | "show load 5m"
            | "show the 5 minute load"
            | "show 5 minute load"
            | "show me load 5"
            | "show me load 5m"
            | "show me the 5 minute load"
            | "show me 5 minute load"
            | "open load 5"
            | "open load 5m"
            | "open the 5 minute load"
            | "open 5 minute load"
            | "open 5m load"
            | "list load 5"
            | "list load 5m"
            | "list the 5 minute load"
            | "list 5 minute load"
            | "list the 5m load"
    ) {
        return Some(LoadWindowChip::Five);
    }
    if matches!(
        n.as_str(),
        "/load-15m"
            | "/load-15"
            | "/load15"
            | "/load15m"
            | "15 minute load"
            | "15-minute load"
            | "fifteen minute load"
            | "load 15"
            | "load 15m"
            | "15m load"
            | "the 15 minute load"
            | "what's the 15 minute load"
            | "whats the 15 minute load"
            | "what is the 15 minute load"
            | "how's the 15 minute load"
            | "hows the 15 minute load"
            | "is the 15 minute load high"
            | "is 15 minute load high"
            | "view load 15"
            | "view load 15m"
            | "view the 15 minute load"
            | "view 15 minute load"
            | "view 15m load"
            | "see load 15"
            | "see load 15m"
            | "see the 15 minute load"
            | "see 15 minute load"
            | "see 15m load"
            | "show load 15"
            | "show load 15m"
            | "show the 15 minute load"
            | "show 15 minute load"
            | "show me load 15"
            | "show me load 15m"
            | "show me the 15 minute load"
            | "show me 15 minute load"
            | "open load 15"
            | "open load 15m"
            | "open the 15 minute load"
            | "open 15 minute load"
            | "open 15m load"
            | "list load 15"
            | "list load 15m"
            | "list the 15 minute load"
            | "list 15 minute load"
            | "list the 15m load"
    ) {
        return Some(LoadWindowChip::Fifteen);
    }
    None
}


/// True for a focused 5m or 15m load ask (not `/load`, not the 1-minute chip).
pub fn looks_like_load_window_chip_request(content: &str) -> bool {
    parse_load_window_chip(content).is_some()
}


/// Zero-LLM Load 5m / 15m reply (same Load≥4 hot mark as the 1-minute chip).
pub fn format_load_window_chip_gateway(window: LoadWindowChip) -> String {
    let d = crate::metrics::get_cpu_details();
    let (label, value) = match window {
        LoadWindowChip::Five => ("5m", d.load_5),
        LoadWindowChip::Fifteen => ("15m", d.load_15),
    };
    let hot_mark = if details_load_is_hot(value) {
        " · hot"
    } else {
        ""
    };
    format!("**Load {label}** · {value:.2}{hot_mark}")
}


/// True for `/details` / `/load` / load average — not process details or strip/rings.
/// View/see/show me/open/list-the NL (v0.1.1040). Exact `open details` / `open the details`
/// only. After normalize strips `show me` / `show`, prefer `the details` / `the load` so
/// `show me the details` still matches.
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
        || n.contains("path")
        || n.contains("where")
        || n.contains("size")
        || contains_age_token(&n)
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/details"
            | "details"
            | "the details"
            | "system details"
            | "the system details"
            | "cpu details"
            | "list details"
            | "list the details"
            | "show details"
            | "show the details"
            | "show me details"
            | "show me the details"
            | "view details"
            | "view the details"
            | "see details"
            | "see the details"
            | "open details"
            | "open the details"
            | "view system details"
            | "see system details"
            | "open system details"
            | "list system details"
            | "details status"
            | "/details hot"
            | "details hot"
            | "hot details"
            | "the hot details"
            | "which details are hot"
            | "which detail is hot"
            | "show hot details"
            | "list hot details"
            | "view hot details"
            | "see hot details"
            | "open hot details"
            | "list the hot details"
            | "view details hot"
            | "see details hot"
            | "open details hot"
            | "list details hot"
            | "what's hot on details"
            | "whats hot on details"
            | "/load"
            | "load"
            | "the load"
            | "load average"
            | "load avg"
            | "system load"
            | "cpu load"
            | "show load"
            | "show the load"
            | "list load"
            | "list the load"
            | "view load"
            | "view the load"
            | "see load"
            | "see the load"
            | "open load"
            | "open the load"
            | "what's the load"
            | "whats the load"
            | "what is the load"
            // `load 1` / `1-minute load` / `/load-1m` → Load 1m chip (not full Details).
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


pub(super) fn perplexity_result_has_snippet(snippet: &str) -> bool {
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
        || n == "view top results"
        || n == "see top results"
        || n == "open top results"
        || n == "list top results"
        || n == "list the top results"
        || n == "show me the top results"
        || n == "the top results"
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
        || n == "view snippet results"
        || n == "see snippet results"
        || n == "open snippet results"
        || n == "list snippet results"
        || n == "list the snippet results"
        || n == "show me the snippet results"
        || n == "the snippet results"
    {
        return PerplexityListFilter::Snippet;
    }
    PerplexityListFilter::All
}


/// True for `/perplexity` / `last search` — Top/Snippet filter parity; not new search asks.
/// View/see/show me/open/list-the NL (v0.1.1045). Exact `open perplexity` / `open the perplexity`
/// only. After normalize strips `show me` / `show`, prefer `the perplexity` so
/// `show me the perplexity` still matches. Does not steal `/perplexity key`, path/size/age, or live search.
pub fn looks_like_perplexity_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 64 {
        return false;
    }
    // Path/size/age asks go to perplexity_last.json instant (inline — avoid calling path/size/age detectors).
    if n.contains("perplexity_last.json")
        || n.contains("perplexity_last")
        || n.contains("perplexity-last")
        || n.contains("last search file")
        || n.contains("last search cache")
        || n.contains("perplexity cache")
        || n.contains("perplexity results file")
        || n.contains("perplexity results path")
        || n.contains("perplexity last size")
        || n.contains("perplexity last age")
        || (n.contains("perplexity")
            && (n.contains("path")
                || n.contains("where")
                || n.contains("location")
                || n.contains("folder")
                || n.contains("directory")
                || n.contains("dir")
                || n.contains("file")
                || n.contains("json")
                || n.contains("size")
                || n.contains("big")
                || n.contains("large")
                || n.contains("bytes")
                || contains_age_token(&n)
                || n.contains("old")
                || n.contains("stale")
                || n.contains("when")
                || n.contains("updated")
                || n.contains("modified")))
        || (n.contains("last search")
            && (n.contains("path")
                || n.contains("where")
                || n.contains("file")
                || n.contains("json")
                || n.contains("cache")
                || n.contains("size")
                || n.contains("big")
                || n.contains("large")
                || contains_age_token(&n)
                || n.contains("old")
                || n.contains("stale")))
    {
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
        || n.contains("perplexity key")
        || n.contains("perplexity status")
        || n.contains("is perplexity ready")
        || n.contains("is perplexity configured")
        || n.contains("how's perplexity")
        || n.contains("hows perplexity")
        || n.starts_with("perplexity search ")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/perplexity"
            | "perplexity"
            | "the perplexity"
            | "last search"
            | "the last search"
            | "last perplexity"
            | "the last perplexity"
            | "perplexity results"
            | "search results"
            | "list perplexity"
            | "list the perplexity"
            | "show perplexity"
            | "show the perplexity"
            | "show me perplexity"
            | "show me the perplexity"
            | "view perplexity"
            | "view the perplexity"
            | "see perplexity"
            | "see the perplexity"
            | "open perplexity"
            | "open the perplexity"
            | "view last search"
            | "see last search"
            | "open last search"
            | "list last search"
            | "list the last search"
            | "show me the last search"
            | "view last perplexity"
            | "see last perplexity"
            | "open last perplexity"
            | "list the last perplexity"
            | "view perplexity results"
            | "see perplexity results"
            | "open perplexity results"
            | "list the perplexity results"
            | "show me the perplexity results"
            | "view search results"
            | "see search results"
            | "open search results"
            | "list the search results"
            | "perplexity search"
            | "/perplexity top"
            | "perplexity top"
            | "view perplexity top"
            | "see perplexity top"
            | "open perplexity top"
            | "list the perplexity top"
            | "top results"
            | "top result"
            | "the top results"
            | "view top results"
            | "see top results"
            | "open top results"
            | "list top results"
            | "list the top results"
            | "show me the top results"
            | "last search top"
            | "last perplexity top"
            | "/top"
            | "/perplexity snippet"
            | "perplexity snippet"
            | "view perplexity snippet"
            | "see perplexity snippet"
            | "open perplexity snippet"
            | "list the perplexity snippet"
            | "snippet results"
            | "snippets results"
            | "the snippet results"
            | "view snippet results"
            | "see snippet results"
            | "open snippet results"
            | "list snippet results"
            | "list the snippet results"
            | "show me the snippet results"
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
/// View/see/show me/open/list-the NL (v0.1.1024). Exact `open agents` only — not `open agent …`.
pub fn looks_like_agents_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
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
        || n.contains("path")
        || n.contains("where")
        || n.contains("size")
        || n.contains("how big")
        || n.contains("how old")
        || n.ends_with(" age")
        || n.contains(" age ")
        || n.contains("agents age")
        || n.contains("agent age")
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
            | "list the agents"
            | "list agent"
            | "list the agent"
            | "show agents"
            | "show the agents"
            | "show me agents"
            | "show me the agents"
            | "show agent"
            | "show the agent"
            | "show me agent"
            | "show me the agent"
            | "view agents"
            | "view the agents"
            | "view agent"
            | "view the agent"
            | "see agents"
            | "see the agents"
            | "see agent"
            | "see the agent"
            | "open agents"
            | "open the agents"
            | "my agents"
            | "which agents"
            | "what agents"
            | "all agents"
            | "the agents"
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


/// True for `/skills` / `list skills` — Hermes skills_list catalog; not SKILL: / SKILL_VIEW / manage.
/// View/see/show me/open/list-the NL (v0.1.1025). Exact `open skills` only — not `open skill …`.
pub fn looks_like_skills_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    // Tool invocations and manage/run asks stay with the agent / tool loop.
    if n.contains("skill:")
        || n.contains("skill=")
        || n.contains("skills_list")
        || n.contains("skill_view")
        || n.contains("skill_manage")
        || n.starts_with("skill ")
        || n.contains("create")
        || n.contains("add ")
        || n.contains("edit")
        || n.contains("delete")
        || n.contains("remove")
        || n.contains("write")
        || n.contains("patch")
        || n.contains("run skill")
        || n.contains("invoke")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains("why")
        || n.contains("ticket")
        || n.contains("redmine")
        || n.contains("path")
        || n.contains("folder")
        || n.contains("directory")
        || n.contains("where")
        || n.contains("location")
        || n.contains("size")
        || n.contains("how big")
        || n.contains("how old")
        || n.ends_with(" age")
        || n.contains(" age ")
        || n.contains("skills age")
        || n.contains("skill age")
        || n.chars().any(|c| c.is_ascii_digit())
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/skills"
            | "skills"
            | "list skills"
            | "list the skills"
            | "list skill"
            | "list the skill"
            | "show skills"
            | "show the skills"
            | "show me skills"
            | "show me the skills"
            | "show skill"
            | "show the skill"
            | "show me skill"
            | "show me the skill"
            | "view skills"
            | "view the skills"
            | "view skill"
            | "view the skill"
            | "see skills"
            | "see the skills"
            | "see skill"
            | "see the skill"
            | "open skills"
            | "open the skills"
            | "my skills"
            | "which skills"
            | "what skills"
            | "all skills"
            | "the skills"
            | "skill list"
            | "skills list"
            | "skills catalog"
            | "skill catalog"
            | "installed skills"
            | "available skills"
            | "skills installed"
            | "skills available"
    )
}


/// Zero-LLM skills catalog (Hermes skills_list / SKILLS_LIST parity).
pub fn format_skills_gateway() -> String {
    let skills = crate::skills::load_skills();
    let title = if skills.is_empty() {
        "**Skills** — 0 installed".to_string()
    } else {
        format!("**Skills** — {} installed", skills.len())
    };
    let mut lines = vec![title];
    if skills.is_empty() {
        lines.push(
            "_None yet — add `skill-<n>-<topic>.md` under `~/.mac-stats/agents/skills/`._"
                .to_string(),
        );
    } else {
        for s in &skills {
            let desc = s
                .content
                .lines()
                .map(|l| l.trim())
                .find(|l| !l.is_empty())
                .unwrap_or("(no description)")
                .chars()
                .take(100)
                .collect::<String>();
            lines.push(format!("• `{}-{}` — {}", s.number, s.topic, desc));
        }
        lines.push(
            "_Run: `SKILL: <n|topic>` · View: `SKILL_VIEW: <n|topic>`_".to_string(),
        );
    }
    let mut out = lines.join("\n");
    if out.chars().count() > 1800 {
        out = out.chars().take(1790).collect::<String>() + "…";
    }
    out
}


/// Task list filter for `/tasks` instant — Active (open+WIP) or All statuses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TasksListFilter {
    Active,
    All,
}


/// Parse Active/All from `/tasks all`, `all tasks`, etc. Default Active.
pub fn parse_tasks_list_filter(content: &str) -> TasksListFilter {
    let n = normalize_operator_command(content);
    if n.ends_with(" all")
        || n == "all tasks"
        || n == "tasks all"
        || n == "/tasks all"
        || n == "list all tasks"
        || n == "all task"
        || n.contains(" every status")
        || n.contains("by status")
    {
        return TasksListFilter::All;
    }
    TasksListFilter::Active
}


/// True for `/tasks` / `list tasks` — TASK_LIST catalog; not create/show/status/append.
/// View/see/show me/open/list-the NL (v0.1.1026). Exact `open tasks` / `open the tasks` only —
/// not `open task …` (singular stays with the agent). Path/size/age lanes stay separate.
pub fn looks_like_tasks_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    if n.contains("task:")
        || n.contains("task_")
        || n.contains("create")
        || n.contains("append")
        || n.contains("assign")
        || n.contains("status")
        || n.contains("close")
        || n.contains("finish")
        || n.contains("delete")
        || n.contains("remove")
        || n.contains("sleep")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains("why")
        || n.contains("ticket")
        || n.contains("redmine")
        || n.contains("schedule")
        || n.contains("path")
        || n.contains("folder")
        || n.contains("directory")
        || n.contains("where")
        || n.contains("location")
        || n.contains("size")
        || n.contains("how big")
        || n.contains("how old")
        || n.ends_with(" age")
        || n.contains(" age ")
        || n.contains("tasks age")
        || n.contains("task age")
        || n.chars().any(|c| c.is_ascii_digit())
    {
        return false;
    }
    // "show task <id>" becomes "task <id>" after normalize — already rejected via digits.
    // Bare "task" alone is too vague (could be create); require list-ish wording or /tasks.
    matches!(
        n.as_str(),
        "/tasks"
            | "/tasks all"
            | "/tasks open"
            | "/tasks active"
            | "tasks"
            | "list tasks"
            | "list the tasks"
            | "list task"
            | "list the task"
            | "show tasks"
            | "show the tasks"
            | "show me tasks"
            | "show me the tasks"
            | "show task"
            | "show the task"
            | "show me task"
            | "show me the task"
            | "view tasks"
            | "view the tasks"
            | "view task"
            | "view the task"
            | "see tasks"
            | "see the tasks"
            | "see task"
            | "see the task"
            | "open tasks"
            | "open the tasks"
            | "my tasks"
            | "which tasks"
            | "what tasks"
            | "all tasks"
            | "active tasks"
            | "wip tasks"
            | "task list"
            | "tasks list"
            | "tasks catalog"
            | "list open tasks"
            | "list all tasks"
            | "list my tasks"
            | "tasks all"
            | "tasks open"
            | "tasks active"
            | "the tasks"
    )
}


/// Zero-LLM task list (TASK_LIST parity; Active = open+WIP, All = by status).
pub fn format_tasks_gateway(filter: TasksListFilter) -> String {
    let (title, body) = match filter {
        TasksListFilter::All => (
            "**All tasks**".to_string(),
            crate::task::format_list_all_tasks().unwrap_or_else(|e| format!("(unavailable: {e})")),
        ),
        TasksListFilter::Active => (
            "**Active tasks** (open · WIP)".to_string(),
            crate::task::format_list_open_and_wip_tasks()
                .unwrap_or_else(|e| format!("(unavailable: {e})")),
        ),
    };
    let mut lines = vec![title, String::new(), body];
    lines.push(String::new());
    lines.push(
        "_Create: `TASK_CREATE:` · Show: `TASK_SHOW: <id>` · `/tasks all` for every status_".to_string(),
    );
    let mut out = lines.join("\n");
    if out.chars().count() > 1800 {
        out = out.chars().take(1790).collect::<String>() + "…";
    }
    out
}


/// Plugins All · On · Off filter for `/plugins` instant replies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginsListFilter {
    All,
    On,
    Off,
}


/// Parse On/Off from `/plugins on`, `enabled plugins`, etc. Default All.
pub fn parse_plugins_list_filter(content: &str) -> PluginsListFilter {
    let n = normalize_operator_command(content);
    if n.ends_with(" on")
        || n.ends_with(" enabled")
        || n == "enabled plugins"
        || n == "on plugins"
        || n == "plugins on"
        || n == "/plugins on"
    {
        return PluginsListFilter::On;
    }
    if n.ends_with(" off")
        || n.ends_with(" disabled")
        || n == "disabled plugins"
        || n == "off plugins"
        || n == "plugins off"
        || n == "/plugins off"
    {
        return PluginsListFilter::Off;
    }
    PluginsListFilter::All
}


/// True for `/plugins` / `list plugins` — registered script plugins; not run/add/remove.
/// View/see/show me/open/list-the NL (v0.1.1027). Exact `open plugins` / `open the plugins` only —
/// not `open plugin …` (singular stays with the agent). Path/size/age lanes stay separate.
pub fn looks_like_plugins_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    if n.contains("plugin:")
        || n.contains("run plugin")
        || n.contains("execute")
        || n.contains("create")
        || n.contains("add ")
        || n.contains("edit")
        || n.contains("delete")
        || n.contains("remove")
        || n.contains("install")
        || n.contains("write")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains("why")
        || n.contains("ticket")
        || n.contains("redmine")
        || n.contains("tauri")
        || n.contains("path")
        || n.contains("folder")
        || n.contains("directory")
        || n.contains("where")
        || n.contains("location")
        || n.contains("size")
        || n.contains("how big")
        || n.contains("how old")
        || n.ends_with(" age")
        || n.contains(" age ")
        || n.contains("plugins age")
        || n.contains("plugin age")
        || n.chars().any(|c| c.is_ascii_digit())
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/plugins"
            | "plugins"
            | "list plugins"
            | "list the plugins"
            | "list plugin"
            | "list the plugin"
            | "show plugins"
            | "show the plugins"
            | "show me plugins"
            | "show me the plugins"
            | "show plugin"
            | "show the plugin"
            | "show me plugin"
            | "show me the plugin"
            | "view plugins"
            | "view the plugins"
            | "view plugin"
            | "view the plugin"
            | "see plugins"
            | "see the plugins"
            | "see plugin"
            | "see the plugin"
            | "open plugins"
            | "open the plugins"
            | "my plugins"
            | "which plugins"
            | "what plugins"
            | "all plugins"
            | "the plugins"
            | "plugin list"
            | "plugins list"
            | "plugins catalog"
            | "installed plugins"
            | "available plugins"
            | "plugins installed"
            | "plugins available"
            | "/plugins on"
            | "plugins on"
            | "enabled plugins"
            | "on plugins"
            | "plugins enabled"
            | "/plugins off"
            | "plugins off"
            | "disabled plugins"
            | "off plugins"
            | "plugins disabled"
    )
}


/// Zero-LLM plugins catalog (registered script plugins; On/Off filter; no script run).
pub fn format_plugins_gateway(filter: PluginsListFilter) -> String {
    let mut plugins = crate::commands::plugins::list_registered_plugins();
    plugins.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then_with(|| a.id.cmp(&b.id))
    });
    let on_n = plugins.iter().filter(|p| p.enabled).count();
    let off_n = plugins.len().saturating_sub(on_n);
    let title = match filter {
        PluginsListFilter::All => format!(
            "**Plugins** — {on_n} on · {off_n} off ({total} total)",
            total = plugins.len()
        ),
        PluginsListFilter::On => format!("**Plugins · On** — {on_n}"),
        PluginsListFilter::Off => format!("**Plugins · Off** — {off_n}"),
    };
    let mut lines = vec![title];
    fn plugin_row(p: &crate::plugins::Plugin) -> String {
        let path = p.script_path.display();
        let interval = p.schedule_interval_secs;
        format!("• {} · `{id}` · every {interval}s · `{path}`", p.name, id = p.id)
    }
    match filter {
        PluginsListFilter::All => {
            if plugins.is_empty() {
                lines.push(
                    "_None registered yet — add a script plugin via Settings / `add_plugin` (no script run from this list)._"
                        .to_string(),
                );
            } else {
                if on_n > 0 {
                    lines.push("**On**".to_string());
                    for p in plugins.iter().filter(|p| p.enabled) {
                        lines.push(plugin_row(p));
                    }
                }
                if off_n > 0 {
                    lines.push("**Off**".to_string());
                    for p in plugins.iter().filter(|p| !p.enabled) {
                        lines.push(plugin_row(p));
                    }
                }
            }
        }
        PluginsListFilter::On => {
            let ons: Vec<_> = plugins.iter().filter(|p| p.enabled).collect();
            if ons.is_empty() {
                lines.push("_None on right now._".to_string());
            } else {
                for p in ons {
                    lines.push(plugin_row(p));
                }
            }
        }
        PluginsListFilter::Off => {
            let offs: Vec<_> = plugins.iter().filter(|p| !p.enabled).collect();
            if offs.is_empty() {
                lines.push("_None off right now._".to_string());
            } else {
                for p in offs {
                    lines.push(plugin_row(p));
                }
            }
        }
    }
    lines.push(String::new());
    lines.push(
        "_List only — run/add/remove stay with the agent / Settings (no script execute)._"
            .to_string(),
    );
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
/// View/see/show me/open/list-the NL (v0.1.1022). Exact `open sessions` only — not `open session …`.
pub fn looks_like_sessions_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
        return false;
    }
    if n.contains("create")
        || n.contains("add ")
        || n.contains("edit")
        || n.contains("delete")
        || n.contains("remove")
        || n.contains("resume")
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
            | "list the sessions"
            | "list session"
            | "list the session"
            | "show sessions"
            | "show the sessions"
            | "show me sessions"
            | "show me the sessions"
            | "show session"
            | "show the session"
            | "show me session"
            | "show me the session"
            | "view sessions"
            | "view the sessions"
            | "view session"
            | "view the session"
            | "see sessions"
            | "see the sessions"
            | "see session"
            | "see the session"
            | "open sessions"
            | "open the sessions"
            | "my sessions"
            | "which sessions"
            | "what sessions"
            | "all sessions"
            | "the sessions"
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


pub(super) fn truncate_preview(s: &str, max: usize) -> String {
    let t = s.trim().replace('\n', " ");
    if t.chars().count() <= max {
        return t;
    }
    t.chars().take(max.saturating_sub(1)).collect::<String>() + "…"
}


pub(super) fn age_from_ms(ms: u64) -> String {
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


pub(super) fn knowledge_row_is_discord(kind: &str) -> bool {
    kind.eq_ignore_ascii_case("discord")
}


pub(super) fn knowledge_row_is_core(kind: &str) -> bool {
    matches!(
        kind.to_ascii_lowercase().as_str(),
        "soul" | "global" | "main"
    )
}


/// True for `/knowledge` / `list knowledge` — Agent Ops Discord/Core parity; not scrub/edit asks.
/// View/see/show me/open/list-the NL (v0.1.1023).
pub fn looks_like_knowledge_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 48 {
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
        || n.contains("path")
        || n.contains("where")
        || n.contains("size")
        || n.contains("how big")
        || n.contains("how old")
        || n.contains(" age")
        || n.ends_with(" age")
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/knowledge"
            | "knowledge"
            | "list knowledge"
            | "list the knowledge"
            | "list knowledge files"
            | "list the knowledge files"
            | "show knowledge"
            | "show the knowledge"
            | "show me knowledge"
            | "show me the knowledge"
            | "show knowledge files"
            | "show the knowledge files"
            | "show me knowledge files"
            | "show me the knowledge files"
            | "view knowledge"
            | "view the knowledge"
            | "view knowledge files"
            | "view the knowledge files"
            | "see knowledge"
            | "see the knowledge"
            | "see knowledge files"
            | "see the knowledge files"
            | "open knowledge"
            | "open the knowledge"
            | "open knowledge files"
            | "open the knowledge files"
            | "my knowledge"
            | "which knowledge"
            | "what knowledge"
            | "all knowledge"
            | "the knowledge"
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


pub(super) fn shorten_ollama_endpoint_for_chip(endpoint: &str) -> String {
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


pub(super) fn shorten_redmine_url_for_chip(url: &str) -> String {
    let mut s = url.trim().trim_end_matches('/').to_string();
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


/// Shorten a vault path for the Ori Ready chip (`$HOME` → `~`).
pub(super) fn shorten_ori_vault_for_chip(path: &std::path::Path) -> String {
    let raw = path.display().to_string();
    let home_short = if let Ok(home) = std::env::var("HOME") {
        if raw.starts_with(&home) {
            format!("~{}", &raw[home.len()..])
        } else {
            raw.clone()
        }
    } else {
        raw.clone()
    };
    if home_short.chars().count() > 40 {
        let mut s: String = home_short.chars().take(37).collect();
        s.push('…');
        s
    } else {
        home_short
    }
}


pub(super) fn alert_ready_reject_noise(n: &str) -> bool {
    n.contains("send ")
        || n.contains("post ")
        || n.starts_with("post ")
        || n.contains("message ")
        || n.contains("notify ")
        || n.contains("alert me")
        || n.contains("fire ")
        || n.contains("trigger")
        || n.contains("create")
        || n.contains("update")
        || n.contains("delete")
        || n.contains("remove")
        || n.contains("register")
        || n.contains("talk to")
        || n.contains("chat with")
        || n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains(" for ")
        || n.contains(" about ")
        || n.contains(" of ")
        || n.chars().any(|c| c.is_ascii_digit())
}


pub(super) fn classify_candidate(
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


#[derive(Debug, Clone)]
pub(super) struct FailedRunLine {
    ts: String,
    lane: String,
    wall_ms: u64,
    question_preview: String,
    error: Option<String>,
    request_id: String,
}


#[derive(Debug, Clone)]
pub(super) struct SlowRunLine {
    ts: String,
    lane: String,
    wall_ms: u64,
    question_preview: String,
    request_id: String,
    ok: bool,
}


/// Collect recent runs for a single lane from runs.jsonl (newest first).
pub(super) fn collect_lane_runs(lane_want: &str, limit: usize, days: Option<u32>) -> Vec<SlowRunLine> {
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


pub(super) fn format_lane_runs_gateway(lane_label: &str, lane_want: &str, days: Option<u32>) -> String {
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


pub(super) fn sanitize_under_dir(path: &str, root: &Path) -> Result<PathBuf, String> {
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
