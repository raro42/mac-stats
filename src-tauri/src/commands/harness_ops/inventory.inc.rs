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
        || n.contains("cycle")
        || n.contains("health")
        || n.contains("condition")
        || n.contains("max capacity")
        || n.contains("maximum capacity")
        // Battery time-to-empty / full owns these (not Bat %).
        || n.contains("time remaining")
        || n.contains("time to empty")
        || n.contains("time to full")
        || n.contains("until empty")
        || n.contains("until full")
        || n.contains("battery time")
        || n.contains("bat time")
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
/// `/app-uptime` stays process uptime; `/boot-volume` · `/startup-disk` stay boot volume name.
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


/// Current Wi-Fi SSID — not password, not settings, not LAN IP / gateway / DNS.
/// Slash + view/open NL (`/wifi` · `/ssid` · `view wifi`) match `/dns` · `/gateway` ·
/// `/ip` shape; bare `wifi` / `ssid` / password·settings·enable·disable stay with the agent;
/// `/ip` stays LAN address; `/dns` stays nameservers; `/gateway` stays default route.
pub fn looks_like_wifi_ssid_request(content: &str) -> bool {
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
        || n.contains("password")
        || n.contains("passwd")
        || n.contains("passphrase")
        || n.contains("settings")
        || n.contains("prefer")
        || n.contains("enable")
        || n.contains("disable")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("connect")
        || n.contains("disconnect")
        || n.contains("join")
        || n.contains("forget")
        || n.contains("speed")
        || n.contains("signal")
        || n.contains("channel")
        || n.contains("bluetooth")
        || n.contains("hotspot")
        || n.contains("gateway")
        || n.contains("router")
        || n.contains("dns")
        || n.contains("local ip")
        || n.contains("lan ip")
        // Bare short / settings asks stay with the agent.
        || n == "wifi"
        || n == "the wifi"
        || n == "wi-fi"
        || n == "the wi-fi"
        || n == "ssid"
        || n == "the ssid"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/wifi"
            | "/wi-fi"
            | "/ssid"
            | "/wifi-ssid"
            | "/wifissid"
            | "/wifi-network"
            | "/wifinetwork"
            | "/airport"
            | "/wlan"
            | "wifi network"
            | "the wifi network"
            | "wi-fi network"
            | "the wi-fi network"
            | "wifi name"
            | "the wifi name"
            | "wi-fi name"
            | "the wi-fi name"
            | "wifi ssid"
            | "the wifi ssid"
            | "wi-fi ssid"
            | "the wi-fi ssid"
            | "current wifi"
            | "the current wifi"
            | "current wi-fi"
            | "the current wi-fi"
            | "current ssid"
            | "the current ssid"
            | "my wifi"
            | "my wifi network"
            | "my wifi name"
            | "my ssid"
            | "what's my wifi"
            | "whats my wifi"
            | "what is my wifi"
            | "what's the wifi"
            | "whats the wifi"
            | "what is the wifi"
            | "what's my wifi network"
            | "whats my wifi network"
            | "what is my wifi network"
            | "what's my wifi name"
            | "whats my wifi name"
            | "what is my wifi name"
            | "what's my ssid"
            | "whats my ssid"
            | "what is my ssid"
            | "what's the ssid"
            | "whats the ssid"
            | "what is the ssid"
            | "which wifi"
            | "which wifi network"
            | "which ssid"
            | "what wifi am i on"
            | "what wifi network am i on"
            | "what network am i on"
            // View/see/show me/open/list.
            | "view wifi"
            | "view the wifi"
            | "view wifi network"
            | "view the wifi network"
            | "view wifi name"
            | "view the wifi name"
            | "view wifi ssid"
            | "view the wifi ssid"
            | "view ssid"
            | "view the ssid"
            | "view wi-fi"
            | "view the wi-fi"
            | "see wifi"
            | "see the wifi"
            | "see wifi network"
            | "see the wifi network"
            | "see wifi name"
            | "see ssid"
            | "see the ssid"
            | "open wifi"
            | "open the wifi"
            | "open wifi network"
            | "open the wifi network"
            | "open wifi name"
            | "open ssid"
            | "open the ssid"
            | "list wifi"
            | "list the wifi"
            | "list wifi network"
            | "list the wifi network"
            | "list wifi name"
            | "list ssid"
            | "list the ssid"
            | "show wifi"
            | "show the wifi"
            | "show me wifi"
            | "show me the wifi"
            | "show wifi network"
            | "show the wifi network"
            | "show me wifi network"
            | "show me the wifi network"
            | "show wifi name"
            | "show the wifi name"
            | "show me wifi name"
            | "show me the wifi name"
            | "show ssid"
            | "show the ssid"
            | "show me ssid"
            | "show me the ssid"
    )
}


/// Zero-LLM current Wi-Fi SSID from `networksetup -getairportnetwork`.
pub fn format_wifi_ssid_report() -> String {
    let v = crate::metrics::get_wifi_ssid();
    let v = v.trim();
    if v.is_empty() {
        return "**Wi-Fi** — _no SSID right now._".to_string();
    }
    format!("**Wi-Fi** · `{v}`")
}


/// Primary interface MAC / ethernet address — not Mac model, not serial, not LAN IP.
/// Slash + view/open NL (`/mac-address` · `/ether` · `view mac address`) match `/dns` ·
/// `/ip` shape; bare `mac` / `ether` / `address` stay with the agent; `/mac-model` stays
/// marketing name · machine model; `/serial` stays hardware serial; `/ip` stays LAN address.
pub fn looks_like_mac_address_request(content: &str) -> bool {
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
        || n.contains("model")
        || n.contains("serial")
        || n.contains("chip")
        || n.contains("processor")
        || n.contains("macbook")
        || n.contains("gateway")
        || n.contains("router")
        || n.contains("dns")
        || n.contains("nameserver")
        || n.contains("local ip")
        || n.contains("lan ip")
        || n.contains("ipv4")
        || n.contains("public")
        || n.contains("external")
        || n.contains("wan")
        || n.contains("wifi")
        || n.contains("ssid")
        || n.contains("wlan")
        || n.contains("airport")
        || n.contains("bluetooth")
        || n.contains("spoof")
        || n.contains("change")
        || n.contains("set ")
        // Bare short / ambiguous asks stay with the agent.
        || n == "mac"
        || n == "the mac"
        || n == "ether"
        || n == "the ether"
        || n == "ethernet"
        || n == "the ethernet"
        || n == "address"
        || n == "the address"
        // "what's my mac" / "what mac is this" stay Mac model.
        || n == "what's my mac"
        || n == "whats my mac"
        || n == "what is my mac"
        || n == "what mac"
        || n == "what mac is this"
        || n == "which mac"
        || n == "which mac is this"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/mac-address"
            | "/macaddress"
            | "/mac-addr"
            | "/macaddr"
            | "/ether"
            | "/ethernet"
            | "/ethernet-address"
            | "/ethernetaddress"
            | "/hw-address"
            | "/hwaddress"
            | "/hardware-address"
            | "/hardwareaddress"
            | "/link-address"
            | "/linkaddress"
            | "/en0-mac"
            | "/en0mac"
            | "mac address"
            | "the mac address"
            | "mac addr"
            | "the mac addr"
            | "ethernet address"
            | "the ethernet address"
            | "ether address"
            | "the ether address"
            | "hardware address"
            | "the hardware address"
            | "hw address"
            | "the hw address"
            | "hardware mac"
            | "the hardware mac"
            | "link address"
            | "the link address"
            | "my mac address"
            | "my ethernet address"
            | "my ether address"
            | "what's my mac address"
            | "whats my mac address"
            | "what is my mac address"
            | "what's the mac address"
            | "whats the mac address"
            | "what is the mac address"
            | "what's my ethernet address"
            | "whats my ethernet address"
            | "what is my ethernet address"
            | "what's my ether address"
            | "whats my ether address"
            | "what is my ether address"
            | "which mac address"
            | "which ethernet address"
            // View/see/show me/open/list.
            | "view mac address"
            | "view the mac address"
            | "view ethernet address"
            | "view the ethernet address"
            | "view ether"
            | "view the ether"
            | "view ethernet"
            | "view the ethernet"
            | "see mac address"
            | "see the mac address"
            | "see ethernet address"
            | "see the ethernet address"
            | "see ether"
            | "see ethernet"
            | "open mac address"
            | "open the mac address"
            | "open ethernet address"
            | "open the ethernet address"
            | "open ether"
            | "open ethernet"
            | "list mac address"
            | "list the mac address"
            | "list ethernet address"
            | "list the ethernet address"
            | "list ether"
            | "list ethernet"
            | "show mac address"
            | "show the mac address"
            | "show me mac address"
            | "show me the mac address"
            | "show ethernet address"
            | "show the ethernet address"
            | "show me ethernet address"
            | "show me the ethernet address"
            | "show ether"
            | "show the ether"
            | "show me ether"
            | "show me the ether"
            | "show ethernet"
            | "show the ethernet"
            | "show me ethernet"
            | "show me the ethernet"
    )
}


/// Zero-LLM primary interface MAC from `route` default iface + `ifconfig ether`.
pub fn format_mac_address_report() -> String {
    let v = crate::metrics::get_primary_mac_address();
    let v = v.trim();
    if v.is_empty() {
        return "**MAC** — _no ethernet address right now._".to_string();
    }
    format!("**MAC** · `{v}`")
}


/// Primary interface MTU — not MAC, not LAN IP, not Wi-Fi SSID.
/// Slash + view/open NL (`/mtu` · `/if-mtu` · `view mtu`) match `/mac-address` · `/ip` shape;
/// bare `mtu` stays with the agent; `/mac-address` stays ethernet address; `/ip` stays LAN;
/// `/wifi` stays SSID.
pub fn looks_like_mtu_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 56 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("json")
        || n.contains("mac address")
        || n.contains("ethernet")
        || n.contains("ether")
        || n.contains("serial")
        || n.contains("gateway")
        || n.contains("router")
        || n.contains("dns")
        || n.contains("nameserver")
        || n.contains("local ip")
        || n.contains("lan ip")
        || n.contains("ipv4")
        || n.contains("public")
        || n.contains("external")
        || n.contains("wan")
        || n.contains("wifi")
        || n.contains("ssid")
        || n.contains("wlan")
        || n.contains("airport")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("jumbo")
        || n.contains("fragment")
        // Resource-stat size nouns (vault/config/notes) stay out; "mtu size" is allowed below.
        || (n.contains("size") && !n.contains("mtu"))
        // Bare short / ambiguous asks stay with the agent.
        || n == "mtu"
        || n == "the mtu"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/mtu"
            | "/if-mtu"
            | "/ifmtu"
            | "/iface-mtu"
            | "/ifacemtu"
            | "/interface-mtu"
            | "/interfacemtu"
            | "/link-mtu"
            | "/linkmtu"
            | "/en0-mtu"
            | "/en0mtu"
            | "/network-mtu"
            | "/networkmtu"
            | "mtu size"
            | "the mtu size"
            | "interface mtu"
            | "the interface mtu"
            | "iface mtu"
            | "the iface mtu"
            | "link mtu"
            | "the link mtu"
            | "network mtu"
            | "the network mtu"
            | "my mtu"
            | "my interface mtu"
            | "what's my mtu"
            | "whats my mtu"
            | "what is my mtu"
            | "what's the mtu"
            | "whats the mtu"
            | "what is the mtu"
            | "what's my interface mtu"
            | "whats my interface mtu"
            | "what is my interface mtu"
            | "which mtu"
            | "which interface mtu"
            // View/see/show me/open/list.
            | "view mtu"
            | "view the mtu"
            | "view interface mtu"
            | "view the interface mtu"
            | "view iface mtu"
            | "view link mtu"
            | "see mtu"
            | "see the mtu"
            | "see interface mtu"
            | "see the interface mtu"
            | "open mtu"
            | "open the mtu"
            | "open interface mtu"
            | "open the interface mtu"
            | "list mtu"
            | "list the mtu"
            | "list interface mtu"
            | "list the interface mtu"
            | "show mtu"
            | "show the mtu"
            | "show me mtu"
            | "show me the mtu"
            | "show interface mtu"
            | "show the interface mtu"
            | "show me interface mtu"
            | "show me the interface mtu"
    )
}


/// Zero-LLM primary interface MTU from `route` default iface + `ifconfig`.
pub fn format_mtu_report() -> String {
    let v = crate::metrics::get_primary_mtu();
    let v = v.trim();
    if v.is_empty() {
        return "**MTU** — _no interface MTU right now._".to_string();
    }
    format!("**MTU** · `{v}`")
}


/// Primary IPv4 netmask / subnet mask — not LAN IP, not gateway, not DNS, not MAC.
/// Slash + view/open NL (`/netmask` · `/subnet` · `view netmask`) match `/mac-address` ·
/// `/ip` shape; bare `netmask` / `subnet` / `mask` stay with the agent; `/ip` stays LAN;
/// `/mac-address` stays ethernet; `/wifi` stays SSID; `/mtu` stays MTU.
pub fn looks_like_netmask_request(content: &str) -> bool {
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
        || n.contains("gateway")
        || n.contains("router")
        || n.contains("dns")
        || n.contains("nameserver")
        || n.contains("mac address")
        || n.contains("ethernet")
        || n.contains("ether")
        || n.contains("wifi")
        || n.contains("mtu")
        || n.contains("ssid")
        || n.contains("wlan")
        || n.contains("airport")
        || n.contains("broadcast")
        || n.contains("bcast")
        || n.contains("link status")
        || n.contains("link-status")
        || n.contains("link state")
        || n.contains("link-speed")
        || n.contains("link speed")
        || n.contains("link media")
        || n.contains("network speed")
        || n.contains("network media")
        || n.contains("public")
        || n.contains("external")
        || n.contains("wan")
        || n.contains("vpn")
        || n.contains("calculator")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("spoof")
        // Bare short / ambiguous asks stay with the agent.
        || n == "netmask"
        || n == "the netmask"
        || n == "subnet"
        || n == "the subnet"
        || n == "mask"
        || n == "the mask"
        || n == "cidr"
        || n == "the cidr"
        || n == "prefix"
        || n == "the prefix"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/netmask"
            | "/subnet"
            | "/subnet-mask"
            | "/subnetmask"
            | "/network-mask"
            | "/networkmask"
            | "/ip-netmask"
            | "/ipnetmask"
            | "/lan-netmask"
            | "/lannetmask"
            | "/cidr"
            | "/prefix-len"
            | "/prefixlen"
            | "/prefix-length"
            | "/prefixlength"
            | "subnet mask"
            | "the subnet mask"
            | "network mask"
            | "the network mask"
            | "ip netmask"
            | "the ip netmask"
            | "lan netmask"
            | "the lan netmask"
            | "my netmask"
            | "my subnet mask"
            | "my network mask"
            | "what's my netmask"
            | "whats my netmask"
            | "what is my netmask"
            | "what's the netmask"
            | "whats the netmask"
            | "what is the netmask"
            | "what's my subnet mask"
            | "whats my subnet mask"
            | "what is my subnet mask"
            | "what's the subnet mask"
            | "whats the subnet mask"
            | "what is the subnet mask"
            | "what's my subnet"
            | "whats my subnet"
            | "what is my subnet"
            | "what's the subnet"
            | "whats the subnet"
            | "what is the subnet"
            | "which netmask"
            | "which subnet mask"
            | "what cidr"
            | "what's my cidr"
            | "whats my cidr"
            | "what is my cidr"
            // View/see/show me/open/list.
            | "view netmask"
            | "view the netmask"
            | "view subnet mask"
            | "view the subnet mask"
            | "view subnet"
            | "view the subnet"
            | "view network mask"
            | "view the network mask"
            | "view cidr"
            | "view the cidr"
            | "see netmask"
            | "see the netmask"
            | "see subnet mask"
            | "see the subnet mask"
            | "see subnet"
            | "see the subnet"
            | "see cidr"
            | "open netmask"
            | "open the netmask"
            | "open subnet mask"
            | "open the subnet mask"
            | "open subnet"
            | "open the subnet"
            | "open cidr"
            | "list netmask"
            | "list the netmask"
            | "list subnet mask"
            | "list the subnet mask"
            | "list subnet"
            | "list the subnet"
            | "list cidr"
            | "show netmask"
            | "show the netmask"
            | "show me netmask"
            | "show me the netmask"
            | "show subnet mask"
            | "show the subnet mask"
            | "show me subnet mask"
            | "show me the subnet mask"
            | "show subnet"
            | "show the subnet"
            | "show me subnet"
            | "show me the subnet"
            | "show cidr"
            | "show the cidr"
            | "show me cidr"
            | "show me the cidr"
    )
}


/// Zero-LLM primary interface IPv4 netmask from `route` default iface + `ifconfig`.
pub fn format_netmask_report() -> String {
    let v = crate::metrics::get_primary_netmask();
    let v = v.trim();
    if v.is_empty() {
        return "**Netmask** — _no subnet mask right now._".to_string();
    }
    format!("**Netmask** · `{v}`")
}

/// Primary / default-route interface name — not LAN IP, not MAC, not MTU, not netmask.
/// Slash + view/open NL (`/iface` · `/ifname` · `view interface`) match `/ip` · `/mtu` shape;
/// bare `iface` / `interface` stay with the agent; `/iface-mtu` · `/interface-mtu` stay MTU;
/// `/en0` stays LAN IP.
pub fn looks_like_primary_iface_request(content: &str) -> bool {
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
        || n.contains("mtu")
        || n.contains("netmask")
        || n.contains("subnet")
        || n.contains("cidr")
        || n.contains("broadcast")
        || n.contains("bcast")
        || n.contains("status")
        || n.contains("speed")
        || n.contains("media")
        || n.contains("link-")
        || n.contains("mac")
        || n.contains("ether")
        || n.contains("address")
        || n.contains("wifi")
        || n.contains("ssid")
        || n.contains("wlan")
        || n.contains("airport")
        || n.contains("gateway")
        || n.contains("router")
        || n.contains("dns")
        || n.contains("nameserver")
        || n.contains("ip")
        || n.contains("vpn")
        || n.contains("user")
        || n.contains("ui")
        || n.contains("api")
        || n.contains("interfaces")
        || n.contains("list all")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("rename")
        || n.contains("create")
        || n.contains("delete")
        || n.contains("add ")
        // Bare short / ambiguous asks stay with the agent.
        || n == "iface"
        || n == "the iface"
        || n == "interface"
        || n == "the interface"
        || n == "ifname"
        || n == "the ifname"
        || n == "nic"
        || n == "the nic"
        || n == "netif"
        || n == "the netif"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/iface"
            | "/ifname"
            | "/if-name"
            | "/interface"
            | "/interface-name"
            | "/interfacename"
            | "/default-iface"
            | "/defaultiface"
            | "/default-interface"
            | "/defaultinterface"
            | "/primary-iface"
            | "/primaryiface"
            | "/primary-interface"
            | "/primaryinterface"
            | "/netif"
            | "/nic"
            | "default interface"
            | "the default interface"
            | "default iface"
            | "the default iface"
            | "primary interface"
            | "the primary interface"
            | "primary iface"
            | "the primary iface"
            | "network interface"
            | "the network interface"
            | "network iface"
            | "the network iface"
            | "interface name"
            | "the interface name"
            | "iface name"
            | "the iface name"
            | "my interface"
            | "my iface"
            | "my network interface"
            | "what's my interface"
            | "whats my interface"
            | "what is my interface"
            | "what's the interface"
            | "whats the interface"
            | "what is the interface"
            | "what's my iface"
            | "whats my iface"
            | "what is my iface"
            | "what's the iface"
            | "whats the iface"
            | "what is the iface"
            | "what's my default interface"
            | "whats my default interface"
            | "what is my default interface"
            | "what's the default interface"
            | "whats the default interface"
            | "what is the default interface"
            | "what's my primary interface"
            | "whats my primary interface"
            | "what is my primary interface"
            | "which interface"
            | "which iface"
            | "which network interface"
            | "what interface am i on"
            | "what iface am i on"
            // View/see/show me/open/list.
            | "view interface"
            | "view the interface"
            | "view iface"
            | "view the iface"
            | "view default interface"
            | "view the default interface"
            | "view primary interface"
            | "view the primary interface"
            | "view interface name"
            | "view the interface name"
            | "see interface"
            | "see the interface"
            | "see iface"
            | "see the iface"
            | "see default interface"
            | "see the default interface"
            | "open interface"
            | "open the interface"
            | "open iface"
            | "open the iface"
            | "open default interface"
            | "open the default interface"
            | "list interface"
            | "list the interface"
            | "list iface"
            | "list the iface"
            | "list default interface"
            | "list the default interface"
            | "show interface"
            | "show the interface"
            | "show me interface"
            | "show me the interface"
            | "show iface"
            | "show the iface"
            | "show me iface"
            | "show me the iface"
            | "show default interface"
            | "show the default interface"
            | "show me default interface"
            | "show me the default interface"
            | "show primary interface"
            | "show the primary interface"
            | "show me primary interface"
            | "show me the primary interface"
    )
}

/// Zero-LLM primary / default-route interface name from `route -n get default`.
pub fn format_primary_iface_report() -> String {
    let v = crate::metrics::get_primary_iface();
    let v = v.trim();
    if v.is_empty() {
        return "**Iface** — _no default interface right now._".to_string();
    }
    format!("**Iface** · `{v}`")
}

/// Primary IPv4 broadcast address — not LAN IP, not netmask, not gateway, not iface name.
/// Slash + view/open NL (`/broadcast` · `/bcast` · `view broadcast`) match `/netmask` · `/ip` shape;
/// bare `broadcast` / `bcast` stay with the agent; `/ip` stays LAN; `/netmask` stays subnet mask;
/// `/iface` stays interface name; `/gateway` stays default route.
pub fn looks_like_broadcast_request(content: &str) -> bool {
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
        || n.contains("gateway")
        || n.contains("router")
        || n.contains("dns")
        || n.contains("nameserver")
        || n.contains("mac address")
        || n.contains("ethernet")
        || n.contains("ether")
        || n.contains("wifi")
        || n.contains("mtu")
        || n.contains("ssid")
        || n.contains("wlan")
        || n.contains("airport")
        || n.contains("netmask")
        || n.contains("subnet")
        || n.contains("cidr")
        || n.contains("iface")
        || n.contains("ifname")
        || n.contains("link status")
        || n.contains("link-status")
        || n.contains("link state")
        || n.contains("link-speed")
        || n.contains("link speed")
        || n.contains("link media")
        || n.contains("network speed")
        || n.contains("network media")
        || n.contains("public")
        || n.contains("external")
        || n.contains("wan")
        || n.contains("vpn")
        || n.contains("typing")
        || n.contains("discord")
        || n.contains("channel")
        || n.contains("message")
        || n.contains("send")
        || n.contains("storm")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("spoof")
        // Bare short / ambiguous asks stay with the agent.
        || n == "broadcast"
        || n == "the broadcast"
        || n == "bcast"
        || n == "the bcast"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/broadcast"
            | "/bcast"
            | "/ip-broadcast"
            | "/ipbroadcast"
            | "/lan-broadcast"
            | "/lanbroadcast"
            | "/broadcast-addr"
            | "/broadcastaddr"
            | "/broadcast-address"
            | "/broadcastaddress"
            | "/ipv4-broadcast"
            | "/ipv4broadcast"
            | "broadcast address"
            | "the broadcast address"
            | "ip broadcast"
            | "the ip broadcast"
            | "lan broadcast"
            | "the lan broadcast"
            | "ipv4 broadcast"
            | "the ipv4 broadcast"
            | "my broadcast"
            | "my broadcast address"
            | "my ip broadcast"
            | "what's my broadcast"
            | "whats my broadcast"
            | "what is my broadcast"
            | "what's the broadcast"
            | "whats the broadcast"
            | "what is the broadcast"
            | "what's my broadcast address"
            | "whats my broadcast address"
            | "what is my broadcast address"
            | "what's the broadcast address"
            | "whats the broadcast address"
            | "what is the broadcast address"
            | "which broadcast"
            | "which broadcast address"
            // View/see/show me/open/list.
            | "view broadcast"
            | "view the broadcast"
            | "view broadcast address"
            | "view the broadcast address"
            | "view ip broadcast"
            | "view the ip broadcast"
            | "view lan broadcast"
            | "view the lan broadcast"
            | "see broadcast"
            | "see the broadcast"
            | "see broadcast address"
            | "see the broadcast address"
            | "open broadcast"
            | "open the broadcast"
            | "open broadcast address"
            | "open the broadcast address"
            | "list broadcast"
            | "list the broadcast"
            | "list broadcast address"
            | "list the broadcast address"
            | "show broadcast"
            | "show the broadcast"
            | "show me broadcast"
            | "show me the broadcast"
            | "show broadcast address"
            | "show the broadcast address"
            | "show me broadcast address"
            | "show me the broadcast address"
    )
}

/// Zero-LLM primary interface IPv4 broadcast from `route` default iface + `ifconfig`.
pub fn format_broadcast_report() -> String {
    let v = crate::metrics::get_primary_broadcast();
    let v = v.trim();
    if v.is_empty() {
        return "**Broadcast** — _no broadcast address right now._".to_string();
    }
    format!("**Broadcast** · `{v}`")
}

/// Primary interface link status · media / speed — not MTU, not MAC, not iface name, not broadcast.
/// Slash + view/open NL (`/link-status` · `/media` · `/link-speed` · `view link status`) match
/// `/broadcast` · `/mtu` shape; bare `status` / `media` / `speed` / `link` stay with the agent;
/// `/link-mtu` stays MTU; `/link-address` stays MAC; `/iface` stays interface name.
pub fn looks_like_link_status_request(content: &str) -> bool {
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
        || n.contains("mtu")
        || n.contains("link-mtu")
        || n.contains("link-address")
        || n.contains("link address")
        || n.contains("mac address")
        || n.contains("ethernet")
        || n.contains("ether")
        || n.contains("broadcast")
        || n.contains("bcast")
        || n.contains("netmask")
        || n.contains("subnet")
        || n.contains("cidr")
        || n.contains("gateway")
        || n.contains("router")
        || n.contains("dns")
        || n.contains("nameserver")
        || n.contains("wifi")
        || n.contains("ssid")
        || n.contains("wlan")
        || n.contains("airport")
        || n.contains("discord")
        || n.contains("channel")
        || n.contains("message")
        || n.contains("typing")
        || n.contains("send")
        || n.contains("storm")
        || n.contains("median")
        || n.contains("keep")
        || n.contains("discard")
        || n.contains("public")
        || n.contains("external")
        || n.contains("wan")
        || n.contains("vpn")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("spoof")
        // Bare short / ambiguous asks stay with the agent (multi-word NL matches below).
        || n == "status"
        || n == "the status"
        || n == "link"
        || n == "the link"
        || n == "media"
        || n == "the media"
        || n == "speed"
        || n == "the speed"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/link-status"
            | "/linkstatus"
            | "/link-state"
            | "/linkstate"
            | "/if-status"
            | "/ifstatus"
            | "/iface-status"
            | "/ifacestatus"
            | "/interface-status"
            | "/interfacestatus"
            | "/if-state"
            | "/ifstate"
            | "/media"
            | "/link-media"
            | "/linkmedia"
            | "/network-media"
            | "/networkmedia"
            | "/link-speed"
            | "/linkspeed"
            | "/if-speed"
            | "/ifspeed"
            | "/iface-speed"
            | "/ifacespeed"
            | "/interface-speed"
            | "/interfacespeed"
            | "/network-speed"
            | "/networkspeed"
            | "link status"
            | "the link status"
            | "interface status"
            | "the interface status"
            | "iface status"
            | "the iface status"
            | "link state"
            | "the link state"
            | "interface state"
            | "the interface state"
            | "link media"
            | "the link media"
            | "network media"
            | "the network media"
            | "link speed"
            | "the link speed"
            | "interface speed"
            | "the interface speed"
            | "network speed"
            | "the network speed"
            | "my link status"
            | "my interface status"
            | "my link speed"
            | "my network speed"
            | "what's my link status"
            | "whats my link status"
            | "what is my link status"
            | "what's the link status"
            | "whats the link status"
            | "what is the link status"
            | "what's my link speed"
            | "whats my link speed"
            | "what is my link speed"
            | "what's the link speed"
            | "whats the link speed"
            | "what is the link speed"
            | "what's my network speed"
            | "whats my network speed"
            | "what is my network speed"
            | "which link status"
            | "which link speed"
            | "is the link up"
            | "is my link up"
            // View/see/show me/open/list.
            | "view link status"
            | "view the link status"
            | "view interface status"
            | "view the interface status"
            | "view link state"
            | "view the link state"
            | "view link media"
            | "view the link media"
            | "view link speed"
            | "view the link speed"
            | "view network speed"
            | "view the network speed"
            | "view media"
            | "view the media"
            | "see link status"
            | "see the link status"
            | "see link speed"
            | "see the link speed"
            | "open link status"
            | "open the link status"
            | "open link speed"
            | "open the link speed"
            | "open media"
            | "open the media"
            | "list link status"
            | "list the link status"
            | "list link speed"
            | "list the link speed"
            | "show link status"
            | "show the link status"
            | "show me link status"
            | "show me the link status"
            | "show link speed"
            | "show the link speed"
            | "show me link speed"
            | "show me the link speed"
            | "show media"
            | "show the media"
            | "show me media"
            | "show me the media"
    )
}

/// Zero-LLM primary interface link status · media from `route` default iface + `ifconfig`.
pub fn format_link_status_report() -> String {
    let v = crate::metrics::get_primary_link_status();
    let v = v.trim();
    if v.is_empty() {
        return "**Link** — _no link status right now._".to_string();
    }
    format!("**Link** · `{v}`")
}

/// Battery cycle count · health · max capacity — not the Bat % chip, not charging / left / low.
/// Slash + view/open NL (`/cycles` · `/battery-cycles` · `view cycle count`) match `/battery` · `/chip` shape;
/// bare `cycles` / `health` stay with the agent; `/battery` · `view battery` · `how much battery` stay Bat %.
pub fn looks_like_battery_cycles_request(content: &str) -> bool {
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
        || n.contains("charging")
        || n.contains("plugged")
        || n.contains("percent")
        || n.contains('%')
        || n.contains("left")
        || n.contains("remaining")
        || n.contains("low")
        || n.contains("strip")
        || n.contains("watts")
        || n.contains("power")
        || n.contains("charger")
        || n.contains("adapter")
        || n.contains("heat")
        || n.contains("thermal")
        || n.contains("cpu")
        || n.contains("gpu")
        || n.contains("ram")
        || n.contains("disk")
        || n.contains("ssd")
        || n.contains("keep")
        || n.contains("discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        // Bare short / ambiguous / Bat-% asks stay with the agent or `/battery`.
        || n == "cycles"
        || n == "the cycles"
        || n == "health"
        || n == "the health"
        || n == "condition"
        || n == "the condition"
        || n == "capacity"
        || n == "the capacity"
        || n == "battery"
        || n == "the battery"
        || n == "/battery"
        || n == "/bat"
        || n == "view battery"
        || n == "view the battery"
        || n == "open battery"
        || n == "open the battery"
        || n == "how much battery"
        || n == "battery status"
        || n == "battery charge"
        || n == "battery level"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/cycles"
            | "/cycle-count"
            | "/cyclecount"
            | "/battery-cycles"
            | "/batterycycles"
            | "/battery-cycle-count"
            | "/batterycyclecount"
            | "/bat-cycles"
            | "/batcycles"
            | "/battery-health"
            | "/batteryhealth"
            | "/battery-condition"
            | "/batterycondition"
            | "/max-capacity"
            | "/maxcapacity"
            | "/battery-max-capacity"
            | "/batterymaxcapacity"
            | "cycle count"
            | "the cycle count"
            | "battery cycles"
            | "the battery cycles"
            | "battery cycle count"
            | "the battery cycle count"
            | "battery health"
            | "the battery health"
            | "battery condition"
            | "the battery condition"
            | "max capacity"
            | "the max capacity"
            | "battery max capacity"
            | "the battery max capacity"
            | "maximum capacity"
            | "the maximum capacity"
            | "my cycle count"
            | "my battery cycles"
            | "what's my cycle count"
            | "whats my cycle count"
            | "what is my cycle count"
            | "what's the cycle count"
            | "whats the cycle count"
            | "what is the cycle count"
            | "what's my battery health"
            | "whats my battery health"
            | "what is my battery health"
            | "what's the battery health"
            | "whats the battery health"
            | "what is the battery health"
            | "which cycle count"
            | "how many cycles"
            | "how many battery cycles"
            | "how many cycles on the battery"
            // View/see/show me/open/list.
            | "view cycles"
            | "view the cycles"
            | "view cycle count"
            | "view the cycle count"
            | "view battery cycles"
            | "view the battery cycles"
            | "view battery health"
            | "view the battery health"
            | "view battery condition"
            | "view the battery condition"
            | "view max capacity"
            | "view the max capacity"
            | "see cycles"
            | "see the cycles"
            | "see cycle count"
            | "see the cycle count"
            | "see battery cycles"
            | "see the battery cycles"
            | "see battery health"
            | "see the battery health"
            | "open cycles"
            | "open the cycles"
            | "open cycle count"
            | "open the cycle count"
            | "open battery cycles"
            | "open the battery cycles"
            | "open battery health"
            | "open the battery health"
            | "list cycles"
            | "list the cycles"
            | "list cycle count"
            | "list the cycle count"
            | "list battery cycles"
            | "list the battery cycles"
            | "show cycles"
            | "show the cycles"
            | "show me cycles"
            | "show me the cycles"
            | "show cycle count"
            | "show the cycle count"
            | "show me cycle count"
            | "show me the cycle count"
            | "show battery cycles"
            | "show the battery cycles"
            | "show me battery cycles"
            | "show me the battery cycles"
            | "show battery health"
            | "show the battery health"
            | "show me battery health"
            | "show me the battery health"
    )
}

/// Zero-LLM battery cycle count · health · max capacity from `system_profiler SPPowerDataType`.
pub fn format_battery_cycles_report() -> String {
    let v = crate::metrics::get_battery_cycles();
    let v = v.trim();
    if v.is_empty() {
        return "**Cycles** — _no battery cycle reading on this Mac right now._".to_string();
    }
    format!("**Cycles** · `{v}`")
}

/// AC charger · adapter wattage — not CPU+GPU `/watts`, not Bat % / `is it charging`.
/// Slash + view/open NL (`/charger` · `/adapter` · `view charger`) match `/cycles` · `/watts` shape;
/// bare `charger` / `adapter` stay with the agent; `/watts` · `how many watts` stay system power draw.
pub fn looks_like_ac_charger_request(content: &str) -> bool {
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
        || n.contains("cycle")
        || n.contains("health")
        || n.contains("condition")
        || n.contains("max capacity")
        || n.contains("percent")
        || n.contains('%')
        || n.contains("left")
        || n.contains("remaining")
        || n.contains("low")
        || n.contains("strip")
        || n.contains("heat")
        || n.contains("thermal")
        || n.contains("cpu")
        || n.contains("gpu")
        || n.contains("ram")
        || n.contains("disk")
        || n.contains("ssd")
        || n.contains("keep")
        || n.contains("discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        // System power draw / full strip / Bat % stay elsewhere.
        || n == "/watts"
        || n == "/power-draw"
        || n == "/power"
        || n == "watts"
        || n == "the watts"
        || n == "power draw"
        || n == "the power draw"
        || n == "how many watts"
        || n == "how much power"
        || n == "view watts"
        || n == "view the watts"
        || n == "open watts"
        || n == "open the watts"
        || n == "view power"
        || n == "view the power"
        || n == "open power"
        || n == "open the power"
        || n == "power"
        || n == "the power"
        || n == "/battery"
        || n == "/bat"
        || n == "battery"
        || n == "the battery"
        || n == "view battery"
        || n == "view the battery"
        || n == "open battery"
        || n == "open the battery"
        || n == "how much battery"
        || n == "is it charging"
        || n == "is the battery charging"
        || n == "is it plugged in"
        || n == "is the battery plugged in"
        || n == "battery charge"
        || n == "battery level"
        || n == "battery status"
        // Bare short / ambiguous stay with the agent.
        || n == "charger"
        || n == "the charger"
        || n == "adapter"
        || n == "the adapter"
        || n == "ac"
        || n == "the ac"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/charger"
            | "/ac-charger"
            | "/accharger"
            | "/adapter"
            | "/power-adapter"
            | "/poweradapter"
            | "/ac-adapter"
            | "/acadapter"
            | "/charger-watts"
            | "/chargerwatts"
            | "/ac-watts"
            | "/acwatts"
            | "/adapter-watts"
            | "/adapterwatts"
            | "/charger-wattage"
            | "/adapter-wattage"
            | "charger watts"
            | "the charger watts"
            | "adapter watts"
            | "the adapter watts"
            | "ac charger"
            | "the ac charger"
            | "ac adapter"
            | "the ac adapter"
            | "power adapter"
            | "the power adapter"
            | "charger wattage"
            | "the charger wattage"
            | "adapter wattage"
            | "the adapter wattage"
            | "my charger"
            | "my adapter"
            | "my ac charger"
            | "my power adapter"
            | "what's my charger"
            | "whats my charger"
            | "what is my charger"
            | "what's my adapter"
            | "whats my adapter"
            | "what is my adapter"
            | "what's my charger watts"
            | "whats my charger watts"
            | "what is my charger watts"
            | "what's my adapter watts"
            | "whats my adapter watts"
            | "what is my adapter watts"
            | "what's the charger"
            | "whats the charger"
            | "what is the charger"
            | "what's the adapter"
            | "whats the adapter"
            | "what is the adapter"
            | "which charger"
            | "which adapter"
            | "how many charger watts"
            | "how many adapter watts"
            // View/see/show me/open/list.
            | "view charger"
            | "view the charger"
            | "view adapter"
            | "view the adapter"
            | "view ac charger"
            | "view the ac charger"
            | "view power adapter"
            | "view the power adapter"
            | "view charger watts"
            | "view the charger watts"
            | "view adapter watts"
            | "view the adapter watts"
            | "see charger"
            | "see the charger"
            | "see adapter"
            | "see the adapter"
            | "see charger watts"
            | "see the charger watts"
            | "open charger"
            | "open the charger"
            | "open adapter"
            | "open the adapter"
            | "open ac charger"
            | "open the ac charger"
            | "open charger watts"
            | "open the charger watts"
            | "list charger"
            | "list the charger"
            | "list adapter"
            | "list the adapter"
            | "list charger watts"
            | "list the charger watts"
            | "show charger"
            | "show the charger"
            | "show me charger"
            | "show me the charger"
            | "show adapter"
            | "show the adapter"
            | "show me adapter"
            | "show me the adapter"
            | "show charger watts"
            | "show the charger watts"
            | "show me charger watts"
            | "show me the charger watts"
            | "show adapter watts"
            | "show the adapter watts"
            | "show me adapter watts"
            | "show me the adapter watts"
    )
}

/// Zero-LLM AC charger · adapter wattage from `system_profiler SPPowerDataType`.
pub fn format_ac_charger_report() -> String {
    let v = crate::metrics::get_ac_charger();
    let v = v.trim();
    if v.is_empty() {
        return "**Charger** — _no AC charger reading on this Mac right now._".to_string();
    }
    format!("**Charger** · `{v}`")
}

/// Battery time-to-empty / time-to-full (not Bat % · not cycles).
/// Slash + view/open NL (`/battery-time` · `/time-remaining` · `view battery time`) match `/cycles` shape;
/// `battery left` / `battery remaining` / `/battery` stay Bat %; bare `time` / `remaining` stay with the agent.
pub fn looks_like_battery_time_request(content: &str) -> bool {
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
        || n.contains("cycle")
        || n.contains("health")
        || n.contains("condition")
        || n.contains("capacity")
        || n.contains("percent")
        || n.contains('%')
        || n.contains("charging")
        || n.contains("plugged")
        || n.contains("low")
        || n.contains("strip")
        || n.contains("watts")
        || n.contains("power")
        || n.contains("heat")
        || n.contains("thermal")
        || n.contains("cpu")
        || n.contains("gpu")
        || n.contains("ram")
        || n.contains("disk")
        || n.contains("ssd")
        || n.contains("keep")
        || n.contains("discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        // Bare / Bat-% asks stay with the agent or `/battery`.
        || n == "time"
        || n == "the time"
        || n == "remaining"
        || n == "the remaining"
        || n == "battery"
        || n == "the battery"
        || n == "/battery"
        || n == "/bat"
        || n == "view battery"
        || n == "view the battery"
        || n == "open battery"
        || n == "open the battery"
        || n == "how much battery"
        || n == "battery left"
        || n == "battery remaining"
        || n == "charge left"
        || n == "charge remaining"
        || n == "battery status"
        || n == "battery charge"
        || n == "battery level"
        || n == "how much battery left"
        || n == "how much battery is left"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/battery-time"
            | "/batterytime"
            | "/bat-time"
            | "/battime"
            | "/time-remaining"
            | "/timeremaining"
            | "/time-to-empty"
            | "/timetoempty"
            | "/time-to-full"
            | "/timetofull"
            | "/eta-empty"
            | "/etaempty"
            | "/eta-full"
            | "/etafull"
            | "battery time"
            | "the battery time"
            | "battery time remaining"
            | "the battery time remaining"
            | "time remaining"
            | "the time remaining"
            | "time to empty"
            | "the time to empty"
            | "time to full"
            | "the time to full"
            | "until empty"
            | "until full"
            | "how long until empty"
            | "how long until full"
            | "how long until the battery dies"
            | "how long until the battery is empty"
            | "how long until the battery is full"
            | "how long left on the battery"
            | "how long left on battery"
            | "what's my battery time"
            | "whats my battery time"
            | "what is my battery time"
            | "what's the battery time"
            | "whats the battery time"
            | "what is the battery time"
            | "what's my time remaining"
            | "whats my time remaining"
            | "what is my time remaining"
            | "what's the time remaining"
            | "whats the time remaining"
            | "what is the time remaining"
            | "which battery time"
            // View/see/show me/open/list.
            | "view battery time"
            | "view the battery time"
            | "view time remaining"
            | "view the time remaining"
            | "view time to empty"
            | "view the time to empty"
            | "view time to full"
            | "view the time to full"
            | "see battery time"
            | "see the battery time"
            | "see time remaining"
            | "see the time remaining"
            | "open battery time"
            | "open the battery time"
            | "open time remaining"
            | "open the time remaining"
            | "list battery time"
            | "list the battery time"
            | "list time remaining"
            | "list the time remaining"
            | "show battery time"
            | "show the battery time"
            | "show me battery time"
            | "show me the battery time"
            | "show time remaining"
            | "show the time remaining"
            | "show me time remaining"
            | "show me the time remaining"
            | "show time to empty"
            | "show the time to empty"
            | "show me time to empty"
            | "show me the time to empty"
            | "show time to full"
            | "show the time to full"
            | "show me time to full"
            | "show me the time to full"
    )
}

/// Zero-LLM battery time-to-empty / time-to-full from the `battery` crate.
pub fn format_battery_time_report() -> String {
    let v = crate::metrics::get_battery_time_remaining();
    let v = v.trim();
    if v.is_empty() {
        return "**Battery time** — _no time-to-empty / time-to-full estimate right now._".to_string();
    }
    format!("**Battery time** · `{v}`")
}

/// Connected displays · resolutions — not GPU ring `/gpu`, not screenshot folder lanes,
/// not website Monitors. Slash + view/open NL (`/display` · `/resolution` · `view display`);
/// bare `display` / `resolution` / `screen` stay with the agent; `view monitors` stays Monitors.
pub fn looks_like_display_resolution_request(content: &str) -> bool {
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
        || n.contains("screenshot")
        || n.contains("browser")
        || n.contains("bright")
        || n.contains("sleep")
        || n.contains("mirror")
        || n.contains("theme")
        || n.contains("window")
        || n.contains("cpu")
        || n.contains("gpu")
        || n.contains("ram")
        || n.contains("disk")
        || n.contains("ssd")
        || n.contains("battery")
        || n.contains("charger")
        || n.contains("adapter")
        || n.contains("keep")
        || n.contains("discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        // Screen Sharing / VNC / Remote Login / Screen Lock stay elsewhere (not display resolution).
        || n.contains("screen sharing")
        || n.contains("screensharing")
        || n.contains("screen-sharing")
        || n.contains("screen share")
        || n.contains("screen lock")
        || n.contains("screen-lock")
        || n.contains("screenlock")
        || n.contains("lock screen")
        || n.contains("lock-screen")
        || n.contains("lockscreen")
        || n.contains("vnc")
        || n.contains("remote login")
        || n.contains("remotelogin")
        || n == "/ssh"
        || n == "/sshd"
        || n == "/remote-login"
        || n == "/screen-lock"
        || n == "/lock-screen"
        || n == "/screenlock"
        || n == "/lockscreen"
        // Website monitors / GPU ring / screenshots stay elsewhere.
        || n.contains("monitor")
        || n == "/gpu"
        || n == "/gpu-ring"
        || n == "gpu"
        || n == "the gpu"
        || n == "view gpu"
        || n == "view the gpu"
        || n == "open gpu"
        || n == "open the gpu"
        // Bare short / ambiguous stay with the agent.
        || n == "display"
        || n == "the display"
        || n == "displays"
        || n == "the displays"
        || n == "resolution"
        || n == "the resolution"
        || n == "screen"
        || n == "the screen"
        || n == "screens"
        || n == "the screens"
        || n == "res"
        || n == "the res"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/display"
            | "/displays"
            | "/resolution"
            | "/screen-resolution"
            | "/screenresolution"
            | "/display-resolution"
            | "/displayresolution"
            | "/display-res"
            | "/displayres"
            | "/screen-res"
            | "/screenres"
            | "/pixel-resolution"
            | "/pixelresolution"
            | "/main-display"
            | "/maindisplay"
            | "/screen"
            | "/screens"
            | "display resolution"
            | "the display resolution"
            | "screen resolution"
            | "the screen resolution"
            | "pixel resolution"
            | "the pixel resolution"
            | "main display"
            | "the main display"
            | "my display"
            | "my displays"
            | "my resolution"
            | "my screen resolution"
            | "my display resolution"
            | "what's my display"
            | "whats my display"
            | "what is my display"
            | "what's my displays"
            | "whats my displays"
            | "what is my displays"
            | "what's my resolution"
            | "whats my resolution"
            | "what is my resolution"
            | "what's my screen resolution"
            | "whats my screen resolution"
            | "what is my screen resolution"
            | "what's my display resolution"
            | "whats my display resolution"
            | "what is my display resolution"
            | "what's the display"
            | "whats the display"
            | "what is the display"
            | "what's the resolution"
            | "whats the resolution"
            | "what is the resolution"
            | "what's the screen resolution"
            | "whats the screen resolution"
            | "what is the screen resolution"
            | "which display"
            | "which resolution"
            | "which screen"
            // View/see/show me/open/list.
            | "view display"
            | "view the display"
            | "view displays"
            | "view the displays"
            | "view resolution"
            | "view the resolution"
            | "view screen resolution"
            | "view the screen resolution"
            | "view display resolution"
            | "view the display resolution"
            | "view screen"
            | "view the screen"
            | "view screens"
            | "view the screens"
            | "view main display"
            | "view the main display"
            | "see display"
            | "see the display"
            | "see displays"
            | "see the displays"
            | "see resolution"
            | "see the resolution"
            | "see screen resolution"
            | "see the screen resolution"
            | "open display"
            | "open the display"
            | "open displays"
            | "open the displays"
            | "open resolution"
            | "open the resolution"
            | "open screen resolution"
            | "open the screen resolution"
            | "open display resolution"
            | "open the display resolution"
            | "list display"
            | "list the display"
            | "list displays"
            | "list the displays"
            | "list resolution"
            | "list the resolution"
            | "list screen resolution"
            | "list the screen resolution"
            | "show display"
            | "show the display"
            | "show me display"
            | "show me the display"
            | "show displays"
            | "show the displays"
            | "show me displays"
            | "show me the displays"
            | "show resolution"
            | "show the resolution"
            | "show me resolution"
            | "show me the resolution"
            | "show screen resolution"
            | "show the screen resolution"
            | "show me screen resolution"
            | "show me the screen resolution"
            | "show display resolution"
            | "show the display resolution"
            | "show me display resolution"
            | "show me the display resolution"
    )
}

/// Zero-LLM connected display names · resolutions from `system_profiler SPDisplaysDataType`.
pub fn format_display_resolution_report() -> String {
    let v = crate::metrics::get_display_resolution();
    let v = v.trim();
    if v.is_empty() {
        return "**Display** — _no display reading on this Mac right now._".to_string();
    }
    format!("**Display** · `{v}`")
}

/// GPU chipset · cores · Metal — not GPU usage ring `/gpu`, not CPU `/chip`, not `/display`.
/// Slash + view/open NL (`/gpu-name` · `/gpu-model` · `view gpu name`); bare `gpu` /
/// `view gpu` / `what's the gpu` stay the usage ring; `/chip` stays Apple CPU chip.
pub fn looks_like_gpu_identity_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 56 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("usage")
        || n.contains("percent")
        || n.contains("high")
        || n.contains("busy")
        || n.contains("hot")
        || n.contains("ring")
        || n.contains("gauge")
        || n.contains("path")
        || n.contains("size")
        || contains_age_token(&n)
        || n.contains("screenshot")
        || n.contains("display")
        || n.contains("resolution")
        || n.contains("screen")
        || n.contains("monitor")
        || n.contains("process")
        || n.contains("temp")
        || n.contains("freq")
        || n.contains("clock")
        || n.contains("watt")
        || n.contains("power")
        || n.contains("keep")
        || n.contains("discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        // GPU usage ring owns bare `/gpu` · `view gpu` · `what's the gpu`.
        || n == "/gpu"
        || n == "/gpu-ring"
        || n == "/gpuring"
        || n == "gpu"
        || n == "the gpu"
        || n == "view gpu"
        || n == "view the gpu"
        || n == "see gpu"
        || n == "see the gpu"
        || n == "open gpu"
        || n == "open the gpu"
        || n == "list gpu"
        || n == "list the gpu"
        || n == "show gpu"
        || n == "show the gpu"
        || n == "show me gpu"
        || n == "show me the gpu"
        || n == "what's the gpu"
        || n == "whats the gpu"
        || n == "what is the gpu"
        || n == "how's the gpu"
        || n == "hows the gpu"
        || n == "how much gpu"
        || n == "how much gpu do i have"
        // CPU chip stays `/chip`.
        || n == "/chip"
        || n == "chip"
        || n == "the chip"
        || n == "view chip"
        || n == "view the chip"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/gpu-name"
            | "/gpuname"
            | "/gpu-model"
            | "/gpumodel"
            | "/gpu-chip"
            | "/gpuchip"
            | "/gpu-chipset"
            | "/gpuchipset"
            | "/chipset"
            | "/gpu-cores"
            | "/gpucores"
            | "/metal"
            | "/metal-support"
            | "/metalsupport"
            | "gpu name"
            | "the gpu name"
            | "gpu model"
            | "the gpu model"
            | "gpu chip"
            | "the gpu chip"
            | "gpu chipset"
            | "the gpu chipset"
            | "chipset"
            | "the chipset"
            | "gpu cores"
            | "the gpu cores"
            | "metal support"
            | "the metal support"
            | "what gpu"
            | "what gpu is this"
            | "what gpu do i have"
            | "what gpu does this mac have"
            | "which gpu"
            | "which gpu is this"
            | "which gpu do i have"
            | "which gpu does this mac have"
            | "what's my gpu"
            | "whats my gpu"
            | "what is my gpu"
            | "what's my gpu name"
            | "whats my gpu name"
            | "what is my gpu name"
            | "what's my gpu model"
            | "whats my gpu model"
            | "what is my gpu model"
            | "what's the gpu name"
            | "whats the gpu name"
            | "what is the gpu name"
            | "what's the gpu model"
            | "whats the gpu model"
            | "what is the gpu model"
            | "what's the chipset"
            | "whats the chipset"
            | "what is the chipset"
            | "what's my chipset"
            | "whats my chipset"
            | "what is my chipset"
            | "how many gpu cores"
            | "how many gpu cores do i have"
            // View/see/show me/open/list (bare view gpu stays usage ring).
            | "view gpu name"
            | "view the gpu name"
            | "view gpu model"
            | "view the gpu model"
            | "view gpu chip"
            | "view the gpu chip"
            | "view gpu chipset"
            | "view the gpu chipset"
            | "view chipset"
            | "view the chipset"
            | "view gpu cores"
            | "view the gpu cores"
            | "view metal"
            | "view the metal"
            | "view metal support"
            | "view the metal support"
            | "see gpu name"
            | "see the gpu name"
            | "see gpu model"
            | "see the gpu model"
            | "see chipset"
            | "see the chipset"
            | "see gpu cores"
            | "see the gpu cores"
            | "open gpu name"
            | "open the gpu name"
            | "open gpu model"
            | "open the gpu model"
            | "open chipset"
            | "open the chipset"
            | "open gpu cores"
            | "open the gpu cores"
            | "list gpu name"
            | "list the gpu name"
            | "list gpu model"
            | "list the gpu model"
            | "list chipset"
            | "list the chipset"
            | "list gpu cores"
            | "list the gpu cores"
            | "show gpu name"
            | "show the gpu name"
            | "show me gpu name"
            | "show me the gpu name"
            | "show gpu model"
            | "show the gpu model"
            | "show me gpu model"
            | "show me the gpu model"
            | "show chipset"
            | "show the chipset"
            | "show me chipset"
            | "show me the chipset"
            | "show gpu cores"
            | "show the gpu cores"
            | "show me gpu cores"
            | "show me the gpu cores"
    )
}

/// Zero-LLM GPU chipset · cores · Metal from `system_profiler SPDisplaysDataType`.
pub fn format_gpu_identity_report() -> String {
    let v = crate::metrics::get_gpu_identity();
    let v = v.trim();
    if v.is_empty() {
        return "**GPU** — _no GPU name on this Mac right now._".to_string();
    }
    format!("**GPU** · `{v}`")
}

/// VPN connection status — not LAN `/ip`, not Wi-Fi `/wifi`, not primary `/link-status`.
/// Slash + view/open NL (`/vpn` · `/vpn-status` · `view vpn`) match `/wifi` · `/link-status` shape;
/// bare `vpn` stays with the agent; Signal `/signal` stays alerts.
pub fn looks_like_vpn_status_request(content: &str) -> bool {
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
        || n.contains("password")
        || n.contains("credential")
        || n.contains("keychain")
        || n.contains("install")
        || n.contains("enable")
        || n.contains("disable")
        || n.contains("connect to")
        || n.contains("disconnect from")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("keep")
        || n.contains("discard")
        // Signal alerts / LAN / Wi-Fi stay elsewhere.
        || n == "/signal"
        || n == "signal"
        || n == "the signal"
        || n.contains("wifi")
        || n.contains("wi-fi")
        || n.contains("ssid")
        || n.contains("airport")
        || n.contains("wlan")
        || n == "/ip"
        || n == "/local-ip"
        || n == "/link-status"
        || n == "/media"
        || n == "/link-speed"
        // Bare short / ambiguous stay with the agent.
        || n == "vpn"
        || n == "the vpn"
        || n == "tunnel"
        || n == "the tunnel"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/vpn"
            | "/vpn-status"
            | "/vpnstatus"
            | "/vpn-state"
            | "/vpnstate"
            | "/vpn-connection"
            | "/vpnconnection"
            | "/is-vpn"
            | "/isvpn"
            | "/on-vpn"
            | "/onvpn"
            | "vpn status"
            | "the vpn status"
            | "vpn state"
            | "the vpn state"
            | "vpn connection"
            | "the vpn connection"
            | "my vpn"
            | "my vpn status"
            | "am i on vpn"
            | "am i on a vpn"
            | "am i connected to vpn"
            | "am i connected to a vpn"
            | "is vpn connected"
            | "is the vpn connected"
            | "is vpn up"
            | "is the vpn up"
            | "is vpn on"
            | "is the vpn on"
            | "what's my vpn"
            | "whats my vpn"
            | "what is my vpn"
            | "what's my vpn status"
            | "whats my vpn status"
            | "what is my vpn status"
            | "what's the vpn"
            | "whats the vpn"
            | "what is the vpn"
            | "what's the vpn status"
            | "whats the vpn status"
            | "what is the vpn status"
            | "which vpn"
            | "which vpn am i on"
            // View/see/show me/open/list.
            | "view vpn"
            | "view the vpn"
            | "view vpn status"
            | "view the vpn status"
            | "see vpn"
            | "see the vpn"
            | "see vpn status"
            | "see the vpn status"
            | "open vpn"
            | "open the vpn"
            | "open vpn status"
            | "open the vpn status"
            | "list vpn"
            | "list the vpn"
            | "list vpn status"
            | "list the vpn status"
            | "show vpn"
            | "show the vpn"
            | "show me vpn"
            | "show me the vpn"
            | "show vpn status"
            | "show the vpn status"
            | "show me vpn status"
            | "show me the vpn status"
    )
}

/// Zero-LLM VPN status from `scutil --nc list` (+ utun fallback).
pub fn format_vpn_status_report() -> String {
    let v = crate::metrics::get_vpn_status();
    let v = v.trim();
    if v.is_empty() {
        return "**VPN** — _no VPN reading on this Mac right now._".to_string();
    }
    format!("**VPN** · `{v}`")
}

/// Hardware UUID (platform UUID) — not serial, not provisioning UDID, not generate-uuid.
/// Slash + view/open NL (`/uuid` · `/hw-uuid` · `view uuid`) match `/serial` · `/mac-model` shape;
/// bare `uuid` stays with the agent; `/serial` stays hardware serial; `/udid` stays with the agent.
pub fn looks_like_hardware_uuid_request(content: &str) -> bool {
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
        || n.contains("generate")
        || n.contains("create")
        || n.contains("random")
        || n.contains("session")
        || n.contains("discord")
        || n.contains("snowflake")
        || n.contains("provision")
        || n.contains("udid")
        || n.contains("serial")
        || n.contains("keep")
        || n.contains("discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        // Serial / bare short stay elsewhere.
        || n == "/serial"
        || n == "/sn"
        || n == "serial"
        || n == "the serial"
        || n == "view serial"
        || n == "open serial"
        || n == "/udid"
        || n == "udid"
        || n == "the udid"
        || n == "uuid"
        || n == "the uuid"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/uuid"
            | "/hw-uuid"
            | "/hwuuid"
            | "/hardware-uuid"
            | "/hardwareuuid"
            | "/platform-uuid"
            | "/platformuuid"
            | "/device-uuid"
            | "/deviceuuid"
            | "/mac-uuid"
            | "/macuuid"
            | "hardware uuid"
            | "the hardware uuid"
            | "platform uuid"
            | "the platform uuid"
            | "device uuid"
            | "the device uuid"
            | "mac uuid"
            | "the mac uuid"
            | "hw uuid"
            | "the hw uuid"
            | "my uuid"
            | "my hardware uuid"
            | "my platform uuid"
            | "what uuid"
            | "what uuid is this"
            | "what hardware uuid"
            | "what hardware uuid is this"
            | "what's my uuid"
            | "whats my uuid"
            | "what is my uuid"
            | "what's my hardware uuid"
            | "whats my hardware uuid"
            | "what is my hardware uuid"
            | "what's the uuid"
            | "whats the uuid"
            | "what is the uuid"
            | "what's the hardware uuid"
            | "whats the hardware uuid"
            | "what is the hardware uuid"
            | "what's the platform uuid"
            | "whats the platform uuid"
            | "what is the platform uuid"
            | "which uuid"
            | "which hardware uuid"
            // View/see/show me/open/list.
            | "view uuid"
            | "view the uuid"
            | "view hardware uuid"
            | "view the hardware uuid"
            | "view platform uuid"
            | "view the platform uuid"
            | "see uuid"
            | "see the uuid"
            | "see hardware uuid"
            | "see the hardware uuid"
            | "open uuid"
            | "open the uuid"
            | "open hardware uuid"
            | "open the hardware uuid"
            | "list uuid"
            | "list the uuid"
            | "list hardware uuid"
            | "list the hardware uuid"
            | "list platform uuid"
            | "list the platform uuid"
            | "show uuid"
            | "show the uuid"
            | "show me uuid"
            | "show me the uuid"
            | "show hardware uuid"
            | "show the hardware uuid"
            | "show me hardware uuid"
            | "show me the hardware uuid"
    )
}

/// Zero-LLM hardware UUID from `system_profiler` / `ioreg` IOPlatformUUID.
pub fn format_hardware_uuid_report() -> String {
    let v = crate::metrics::get_hardware_uuid();
    let v = v.trim();
    if v.is_empty() {
        return "**UUID** — _no hardware UUID on this Mac right now._".to_string();
    }
    format!("**UUID** · `{v}`")
}

/// Boot volume / startup disk name — not last boot wall-clock, not SSD % / free / Disk Cleanup.
/// Slash + view/open NL (`/boot-volume` · `/startup-disk` · `view boot volume`) match `/boot` ·
/// `/uuid` shape; bare `boot` / `volume` / `disk` stay with the agent; `/boot` stays boot time;
/// `view disk` stays Disk Cleanup; `/ssd` · `/free` stay capacity · free.
pub fn looks_like_boot_volume_request(content: &str) -> bool {
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
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.contains("free")
        || n.contains("capacity")
        || n.contains("percent")
        || n.contains('%')
        || n.contains("used")
        || n.contains("usage")
        || n.contains("uuid")
        || n.contains("serial")
        || n.contains("time")
        || n.contains("reboot")
        || n.contains("uptime")
        || n.contains("recovery")
        || n.contains("safe boot")
        || n.contains("firmware")
        || n.contains("efi")
        || n.contains("partition")
        || n.contains("format")
        || n.contains("erase")
        || n.contains("keep")
        || n.contains("discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        // Last boot wall-clock / Mac Up / Disk Cleanup / capacity stay elsewhere.
        || n == "/boot"
        || n == "/boot-time"
        || n == "/boottime"
        || n == "/last-boot"
        || n == "/last-reboot"
        || n == "/reboot-time"
        || n == "/uptime"
        || n == "/up"
        || n == "boot"
        || n == "the boot"
        || n == "volume"
        || n == "the volume"
        || n == "disk"
        || n == "the disk"
        || n == "view disk"
        || n == "open disk"
        || n == "view boot"
        || n == "open boot"
        || n == "see boot"
        || n == "list boot"
        || n == "/ssd"
        || n == "/storage"
        || n == "/free"
        || n == "/ssd-size"
        || n == "/disk-size"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/boot-volume"
            | "/bootvolume"
            | "/boot-vol"
            | "/bootvol"
            | "/boot-disk"
            | "/bootdisk"
            | "/startup-disk"
            | "/startupdisk"
            | "/startup-volume"
            | "/startupvolume"
            | "/startup-vol"
            | "/startupvol"
            | "/volume-name"
            | "/volumename"
            | "/boot-volume-name"
            | "/bootvolumename"
            | "boot volume"
            | "the boot volume"
            | "boot disk"
            | "the boot disk"
            | "startup disk"
            | "the startup disk"
            | "startup volume"
            | "the startup volume"
            | "volume name"
            | "the volume name"
            | "boot volume name"
            | "the boot volume name"
            | "my boot volume"
            | "my boot disk"
            | "my startup disk"
            | "my startup volume"
            | "what boot volume"
            | "what boot disk"
            | "what startup disk"
            | "what's my boot volume"
            | "whats my boot volume"
            | "what is my boot volume"
            | "what's my boot disk"
            | "whats my boot disk"
            | "what is my boot disk"
            | "what's my startup disk"
            | "whats my startup disk"
            | "what is my startup disk"
            | "what's the boot volume"
            | "whats the boot volume"
            | "what is the boot volume"
            | "what's the startup disk"
            | "whats the startup disk"
            | "what is the startup disk"
            | "which boot volume"
            | "which startup disk"
            | "how's the boot volume"
            | "hows the boot volume"
            // View/see/show me/open/list.
            | "view boot volume"
            | "view the boot volume"
            | "view boot disk"
            | "view the boot disk"
            | "view startup disk"
            | "view the startup disk"
            | "view startup volume"
            | "view the startup volume"
            | "view volume name"
            | "view the volume name"
            | "see boot volume"
            | "see the boot volume"
            | "see boot disk"
            | "see the boot disk"
            | "see startup disk"
            | "see the startup disk"
            | "open boot volume"
            | "open the boot volume"
            | "open boot disk"
            | "open the boot disk"
            | "open startup disk"
            | "open the startup disk"
            | "list boot volume"
            | "list the boot volume"
            | "list boot disk"
            | "list the boot disk"
            | "list startup disk"
            | "list the startup disk"
            | "show boot volume"
            | "show the boot volume"
            | "show me boot volume"
            | "show me the boot volume"
            | "show boot disk"
            | "show the boot disk"
            | "show me boot disk"
            | "show me the boot disk"
            | "show startup disk"
            | "show the startup disk"
            | "show me startup disk"
            | "show me the startup disk"
    )
}

/// Zero-LLM boot volume / startup disk name from `diskutil info /` (Linux: findmnt/lsblk LABEL).
pub fn format_boot_volume_report() -> String {
    let v = crate::metrics::get_boot_volume();
    let v = v.trim();
    if v.is_empty() {
        return "**Boot volume** — _no boot volume name right now._".to_string();
    }
    format!("**Boot volume** · `{v}`")
}

/// FileVault full-disk encryption status — not boot volume, not SSD % / free, not Disk Cleanup.
/// Slash + view/open NL (`/filevault` · `/fv` · `view filevault`) match `/boot-volume` · `/uuid`
/// shape; bare `filevault` / `fv` / `encryption` stay with the agent; `/boot-volume` stays volume
/// name; `view disk` stays Disk Cleanup; `/ssd` · `/free` stay capacity · free.
pub fn looks_like_filevault_request(content: &str) -> bool {
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
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.contains("enable")
        || n.contains("disable")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("unlock")
        || n.contains("password")
        || n.contains("recovery key")
        || n.contains("keep")
        || n.contains("discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("sip")
        || n.contains("csrutil")
        || n.contains("gatekeeper")
        || n.contains("file sharing")
        || n.contains("filesharing")
        || n.contains("file-sharing")
        || n.contains("file share")
        || n == "/file-sharing"
        || n == "/filesharing"
        || n == "/smb"
        || n == "/smbd"
        // Boot volume / Disk Cleanup / capacity stay elsewhere.
        || n == "/boot-volume"
        || n == "/startup-disk"
        || n == "/boot"
        || n == "view disk"
        || n == "open disk"
        || n == "/ssd"
        || n == "/storage"
        || n == "/free"
        || n == "/ssd-size"
        || n == "/disk-size"
        // Bare short / ambiguous stay with the agent.
        || n == "filevault"
        || n == "the filevault"
        || n == "fv"
        || n == "the fv"
        || n == "encryption"
        || n == "the encryption"
        || n == "disk encryption"
        || n == "the disk encryption"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/filevault"
            | "/file-vault"
            | "/filevault-status"
            | "/filevaultstatus"
            | "/fv"
            | "/fv-status"
            | "/fvstatus"
            | "/fde"
            | "/fde-status"
            | "/fdestatus"
            | "/full-disk-encryption"
            | "/fulldiskencryption"
            | "/disk-encryption"
            | "/diskencryption"
            | "/encryption-status"
            | "/encryptionstatus"
            | "filevault status"
            | "the filevault status"
            | "file vault status"
            | "the file vault status"
            | "fv status"
            | "the fv status"
            | "fde status"
            | "the fde status"
            | "encryption status"
            | "the encryption status"
            | "disk encryption status"
            | "the disk encryption status"
            | "full disk encryption"
            | "the full disk encryption"
            | "full disk encryption status"
            | "the full disk encryption status"
            | "my filevault"
            | "my filevault status"
            | "my encryption"
            | "my encryption status"
            | "is filevault on"
            | "is filevault off"
            | "is filevault enabled"
            | "is filevault disabled"
            | "is file vault on"
            | "is file vault enabled"
            | "is encryption on"
            | "is disk encryption on"
            | "is disk encryption enabled"
            | "is fde on"
            | "is fde enabled"
            | "what's my filevault"
            | "whats my filevault"
            | "what is my filevault"
            | "what's my filevault status"
            | "whats my filevault status"
            | "what is my filevault status"
            | "what's the filevault"
            | "whats the filevault"
            | "what is the filevault"
            | "what's the filevault status"
            | "whats the filevault status"
            | "what is the filevault status"
            | "what's my encryption"
            | "whats my encryption"
            | "what is my encryption"
            | "what's the encryption status"
            | "whats the encryption status"
            | "what is the encryption status"
            | "which filevault"
            | "how's filevault"
            | "hows filevault"
            | "how's the filevault"
            | "hows the filevault"
            // View/see/show me/open/list.
            | "view filevault"
            | "view the filevault"
            | "view filevault status"
            | "view the filevault status"
            | "view file vault"
            | "view the file vault"
            | "view encryption"
            | "view the encryption"
            | "view encryption status"
            | "view the encryption status"
            | "view disk encryption"
            | "view the disk encryption"
            | "view fde"
            | "view the fde"
            | "see filevault"
            | "see the filevault"
            | "see filevault status"
            | "see the filevault status"
            | "see encryption"
            | "see the encryption"
            | "open filevault"
            | "open the filevault"
            | "open filevault status"
            | "open the filevault status"
            | "open encryption"
            | "open the encryption"
            | "list filevault"
            | "list the filevault"
            | "list filevault status"
            | "list the filevault status"
            | "list encryption"
            | "list the encryption"
            | "show filevault"
            | "show the filevault"
            | "show me filevault"
            | "show me the filevault"
            | "show filevault status"
            | "show the filevault status"
            | "show me filevault status"
            | "show me the filevault status"
            | "show encryption"
            | "show the encryption"
            | "show me encryption"
            | "show me the encryption"
    )
}

/// Zero-LLM FileVault status from `fdesetup status` (macOS-only; empty elsewhere).
pub fn format_filevault_report() -> String {
    let v = crate::metrics::get_filevault_status();
    let v = v.trim();
    if v.is_empty() {
        return "**FileVault** — _no FileVault reading on this Mac right now._".to_string();
    }
    format!("**FileVault** · `{v}`")
}

/// System Integrity Protection (SIP) — not FileVault, not Gatekeeper, not boot volume.
/// Slash + view/open NL (`/sip` · `/csrutil` · `view sip`) match `/filevault` · `/uuid` shape;
/// bare `sip` / `csrutil` stay with the agent; `/filevault` stays FileVault; Gatekeeper stays out.
pub fn looks_like_sip_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        // Action verbs (leave status asks like "is sip enabled").
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("unlock")
        || n.contains("password")
        || n.contains("recovery key")
        || n.contains("keep")
        || n.contains("discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("gatekeeper")
        || n.contains("spctl")
        // FileVault / boot volume / Disk Cleanup stay elsewhere.
        || n == "/filevault"
        || n == "/fv"
        || n == "/fde"
        || n == "/boot-volume"
        || n == "/startup-disk"
        || n == "view disk"
        || n == "open disk"
        || n == "/ssd"
        || n == "/storage"
        || n == "/free"
        // Bare short / ambiguous stay with the agent.
        || n == "sip"
        || n == "the sip"
        || n == "csrutil"
        || n == "the csrutil"
        || n == "system integrity"
        || n == "the system integrity"
        || n == "system integrity protection"
        || n == "the system integrity protection"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/sip"
            | "/sip-status"
            | "/sipstatus"
            | "/csrutil"
            | "/csrutil-status"
            | "/csrutilstatus"
            | "/system-integrity"
            | "/systemintegrity"
            | "/system-integrity-protection"
            | "/systemintegrityprotection"
            | "/sip-protection"
            | "/sipprotection"
            | "sip status"
            | "the sip status"
            | "csrutil status"
            | "the csrutil status"
            | "system integrity status"
            | "the system integrity status"
            | "system integrity protection status"
            | "the system integrity protection status"
            | "my sip"
            | "my sip status"
            | "my system integrity"
            | "my system integrity protection"
            | "is sip on"
            | "is sip off"
            | "is sip enabled"
            | "is sip disabled"
            | "is system integrity on"
            | "is system integrity enabled"
            | "is system integrity protection on"
            | "is system integrity protection enabled"
            | "what's my sip"
            | "whats my sip"
            | "what is my sip"
            | "what's my sip status"
            | "whats my sip status"
            | "what is my sip status"
            | "what's the sip"
            | "whats the sip"
            | "what is the sip"
            | "what's the sip status"
            | "whats the sip status"
            | "what is the sip status"
            | "what's my system integrity"
            | "whats my system integrity"
            | "what is my system integrity"
            | "what's the system integrity status"
            | "whats the system integrity status"
            | "what is the system integrity status"
            | "which sip"
            | "how's sip"
            | "hows sip"
            | "how's the sip"
            | "hows the sip"
            // View/see/show me/open/list.
            | "view sip"
            | "view the sip"
            | "view sip status"
            | "view the sip status"
            | "view csrutil"
            | "view the csrutil"
            | "view csrutil status"
            | "view system integrity"
            | "view the system integrity"
            | "view system integrity protection"
            | "view the system integrity protection"
            | "view system integrity status"
            | "view the system integrity status"
            | "see sip"
            | "see the sip"
            | "see sip status"
            | "see the sip status"
            | "see system integrity"
            | "see the system integrity"
            | "open sip"
            | "open the sip"
            | "open sip status"
            | "open the sip status"
            | "open csrutil"
            | "open the csrutil"
            | "open system integrity"
            | "open the system integrity"
            | "list sip"
            | "list the sip"
            | "list sip status"
            | "list the sip status"
            | "list system integrity"
            | "list the system integrity"
            | "show sip"
            | "show the sip"
            | "show me sip"
            | "show me the sip"
            | "show sip status"
            | "show the sip status"
            | "show me sip status"
            | "show me the sip status"
            | "show system integrity"
            | "show the system integrity"
            | "show me system integrity"
            | "show me the system integrity"
    )
}

/// Zero-LLM SIP status from `csrutil status` (macOS-only; empty elsewhere).
pub fn format_sip_report() -> String {
    let v = crate::metrics::get_sip_status();
    let v = v.trim();
    if v.is_empty() {
        return "**SIP** — _no System Integrity Protection reading on this Mac right now._".to_string();
    }
    format!("**SIP** · `{v}`")
}

/// Gatekeeper assessment status — not SIP, not FileVault, not boot volume.
/// Slash + view/open NL (`/gatekeeper` · `/spctl` · `view gatekeeper`) match `/sip` · `/filevault` shape;
/// bare `gatekeeper` / `spctl` stay with the agent; `/sip` stays SIP; `/filevault` stays FileVault.
pub fn looks_like_gatekeeper_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        // Action verbs (leave status asks like "is gatekeeper enabled").
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("assess ")
        || n.contains("notariz")
        || n.contains("quarantine")
        || n.contains("password")
        // Token keep/discard only — do not substring-match inside "gatekeeper".
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("system integrity")
        || n.contains("firewall")
        || n == "/sip"
        || n == "/csrutil"
        || n == "/filevault"
        || n == "/fv"
        || n == "/fde"
        || n == "/firewall"
        || n == "/fw"
        || n == "/afw"
        || n == "/boot-volume"
        || n == "/startup-disk"
        || n == "view disk"
        || n == "open disk"
        || n == "/ssd"
        || n == "/storage"
        || n == "/free"
        // Bare short / ambiguous stay with the agent.
        || n == "gatekeeper"
        || n == "the gatekeeper"
        || n == "gk"
        || n == "the gk"
        || n == "spctl"
        || n == "the spctl"
        || n == "assessments"
        || n == "the assessments"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/gatekeeper"
            | "/gatekeeper-status"
            | "/gatekeeperstatus"
            | "/gk"
            | "/gk-status"
            | "/gkstatus"
            | "/spctl"
            | "/spctl-status"
            | "/spctlstatus"
            | "/assessments"
            | "/assessments-status"
            | "/assessmentsstatus"
            | "gatekeeper status"
            | "the gatekeeper status"
            | "gk status"
            | "the gk status"
            | "spctl status"
            | "the spctl status"
            | "assessments status"
            | "the assessments status"
            | "my gatekeeper"
            | "my gatekeeper status"
            | "my gk"
            | "my gk status"
            | "is gatekeeper on"
            | "is gatekeeper off"
            | "is gatekeeper enabled"
            | "is gatekeeper disabled"
            | "is gk on"
            | "is gk enabled"
            | "is spctl on"
            | "is spctl enabled"
            | "are assessments enabled"
            | "are assessments disabled"
            | "what's my gatekeeper"
            | "whats my gatekeeper"
            | "what is my gatekeeper"
            | "what's my gatekeeper status"
            | "whats my gatekeeper status"
            | "what is my gatekeeper status"
            | "what's the gatekeeper"
            | "whats the gatekeeper"
            | "what is the gatekeeper"
            | "what's the gatekeeper status"
            | "whats the gatekeeper status"
            | "what is the gatekeeper status"
            | "what's my gk"
            | "whats my gk"
            | "what is my gk"
            | "what's the gk status"
            | "whats the gk status"
            | "what is the gk status"
            | "which gatekeeper"
            | "how's gatekeeper"
            | "hows gatekeeper"
            | "how's the gatekeeper"
            | "hows the gatekeeper"
            // View/see/show me/open/list.
            | "view gatekeeper"
            | "view the gatekeeper"
            | "view gatekeeper status"
            | "view the gatekeeper status"
            | "view gk"
            | "view the gk"
            | "view gk status"
            | "view spctl"
            | "view the spctl"
            | "view spctl status"
            | "view assessments"
            | "view the assessments"
            | "view assessments status"
            | "see gatekeeper"
            | "see the gatekeeper"
            | "see gatekeeper status"
            | "see the gatekeeper status"
            | "see gk"
            | "see the gk"
            | "see spctl"
            | "see the spctl"
            | "open gatekeeper"
            | "open the gatekeeper"
            | "open gatekeeper status"
            | "open the gatekeeper status"
            | "open gk"
            | "open the gk"
            | "open spctl"
            | "open the spctl"
            | "list gatekeeper"
            | "list the gatekeeper"
            | "list gatekeeper status"
            | "list the gatekeeper status"
            | "list gk"
            | "list the gk"
            | "list spctl"
            | "list the spctl"
            | "show gatekeeper"
            | "show the gatekeeper"
            | "show me gatekeeper"
            | "show me the gatekeeper"
            | "show gatekeeper status"
            | "show the gatekeeper status"
            | "show me gatekeeper status"
            | "show me the gatekeeper status"
            | "show gk"
            | "show the gk"
            | "show me gk"
            | "show me the gk"
            | "show spctl"
            | "show the spctl"
            | "show me spctl"
            | "show me the spctl"
    )
}

/// Zero-LLM Gatekeeper status from `spctl --status` (macOS-only; empty elsewhere).
pub fn format_gatekeeper_report() -> String {
    let v = crate::metrics::get_gatekeeper_status();
    let v = v.trim();
    if v.is_empty() {
        return "**Gatekeeper** — _no Gatekeeper reading on this Mac right now._".to_string();
    }
    format!("**Gatekeeper** · `{v}`")
}

/// Application Firewall global state — not Gatekeeper, not SIP, not FileVault, not stealth, not logging,
/// not allow-signed, not block-all.
/// Slash + view/open NL (`/firewall` · `/afw` · `view firewall`) match `/gatekeeper` · `/sip` shape;
/// bare `firewall` / `afw` stay with the agent; `/gatekeeper` stays Gatekeeper; `/sip` stays SIP;
/// `/stealth` · `/firewall-stealth` stay stealth; `/firewall-logging` · `/fw-logging` stay logging;
/// `/allow-signed` · `/fw-allow-signed` stay allow-signed; `/block-all` · `/fw-block-all` stay block-all.
pub fn looks_like_firewall_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        // Action verbs (leave status asks like "is firewall enabled").
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("allow ")
        || n.contains("block ")
        || n.contains("password")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("gatekeeper")
        || n.contains("spctl")
        || n.contains("system integrity")
        || n.contains("sip")
        || n.contains("csrutil")
        || n.contains("stealth")
        || n.contains("logging")
        // Gatekeeper / SIP / FileVault / stealth / logging / boot volume / Disk Cleanup stay elsewhere.
        || n == "/gatekeeper"
        || n == "/gk"
        || n == "/spctl"
        || n == "/sip"
        || n == "/csrutil"
        || n == "/filevault"
        || n == "/fv"
        || n == "/fde"
        || n == "/stealth"
        || n == "/firewall-stealth"
        || n == "/firewall-logging"
        || n == "/fw-logging"
        || n == "/afw-logging"
        || n == "/allow-signed"
        || n == "/allowsigned"
        || n == "/fw-allow-signed"
        || n == "/afw-allow-signed"
        || n == "/firewall-allow-signed"
        || n == "/block-all"
        || n == "/blockall"
        || n == "/fw-block-all"
        || n == "/afw-block-all"
        || n == "/firewall-block-all"
        || n == "/getblockall"
        || n == "/listapps"
        || n == "/list-apps"
        || n == "/firewall-apps"
        || n == "/fw-apps"
        || n == "/afw-apps"
        || n == "/firewall-listapps"
        || n == "/socketfilterfw-listapps"
        || n == "/mac-firewall-apps"
        || n.contains("firewall apps")
        || n.contains("fw apps")
        || n.contains("afw apps")
        || n.contains("listapps")
        || n == "/boot-volume"
        || n == "/startup-disk"
        || n == "/remote-login"
        || n == "/ssh"
        || n == "/sshd"
        || n == "view disk"
        || n == "open disk"
        || n == "/ssd"
        || n == "/storage"
        || n == "/free"
        // Bare short / ambiguous stay with the agent.
        || n == "firewall"
        || n == "the firewall"
        || n == "afw"
        || n == "the afw"
        || n == "application firewall"
        || n == "the application firewall"
        || n == "app firewall"
        || n == "the app firewall"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/firewall"
            | "/firewall-status"
            | "/firewallstatus"
            | "/fw"
            | "/fw-status"
            | "/fwstatus"
            | "/afw"
            | "/afw-status"
            | "/afwstatus"
            | "/app-firewall"
            | "/appfirewall"
            | "/application-firewall"
            | "/applicationfirewall"
            | "/socketfilterfw"
            | "/os-firewall"
            | "/macos-firewall"
            | "firewall status"
            | "the firewall status"
            | "afw status"
            | "the afw status"
            | "fw status"
            | "the fw status"
            | "application firewall status"
            | "the application firewall status"
            | "app firewall status"
            | "the app firewall status"
            | "my firewall"
            | "my firewall status"
            | "my application firewall"
            | "my app firewall"
            | "is firewall on"
            | "is firewall off"
            | "is firewall enabled"
            | "is firewall disabled"
            | "is the firewall on"
            | "is the firewall enabled"
            | "is application firewall on"
            | "is application firewall enabled"
            | "is app firewall on"
            | "is app firewall enabled"
            | "what's my firewall"
            | "whats my firewall"
            | "what is my firewall"
            | "what's my firewall status"
            | "whats my firewall status"
            | "what is my firewall status"
            | "what's the firewall"
            | "whats the firewall"
            | "what is the firewall"
            | "what's the firewall status"
            | "whats the firewall status"
            | "what is the firewall status"
            | "what's my application firewall"
            | "whats my application firewall"
            | "what is my application firewall"
            | "which firewall"
            | "how's firewall"
            | "hows firewall"
            | "how's the firewall"
            | "hows the firewall"
            // View/see/show me/open/list.
            | "view firewall"
            | "view the firewall"
            | "view firewall status"
            | "view the firewall status"
            | "view afw"
            | "view the afw"
            | "view application firewall"
            | "view the application firewall"
            | "view app firewall"
            | "view the app firewall"
            | "see firewall"
            | "see the firewall"
            | "see firewall status"
            | "see the firewall status"
            | "see application firewall"
            | "see the application firewall"
            | "open firewall"
            | "open the firewall"
            | "open firewall status"
            | "open the firewall status"
            | "open afw"
            | "open the afw"
            | "open application firewall"
            | "open the application firewall"
            | "list firewall"
            | "list the firewall"
            | "list firewall status"
            | "list the firewall status"
            | "list application firewall"
            | "list the application firewall"
            | "show firewall"
            | "show the firewall"
            | "show me firewall"
            | "show me the firewall"
            | "show firewall status"
            | "show the firewall status"
            | "show me firewall status"
            | "show me the firewall status"
            | "show application firewall"
            | "show the application firewall"
            | "show me application firewall"
            | "show me the application firewall"
    )
}

/// Zero-LLM Application Firewall status from `socketfilterfw --getglobalstate` (macOS-only; empty elsewhere).
pub fn format_firewall_report() -> String {
    let v = crate::metrics::get_firewall_status();
    let v = v.trim();
    if v.is_empty() {
        return "**Firewall** — _no Application Firewall reading on this Mac right now._".to_string();
    }
    format!("**Firewall** · `{v}`")
}

/// Application Firewall stealth mode — not global Firewall `/firewall`, not logging, not allow-signed,
/// not Gatekeeper, not SIP.
/// Slash + view/open NL (`/stealth` · `/firewall-stealth` · `view stealth`) match `/firewall` shape;
/// bare `stealth` stays with the agent; `/firewall` stays global state; `/firewall-logging` stays logging;
/// `/allow-signed` stays allow-signed.
pub fn looks_like_firewall_stealth_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("allow ")
        || n.contains("block ")
        || n.contains("password")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("gatekeeper")
        || n.contains("spctl")
        || n.contains("system integrity")
        || n.contains("sip")
        || n.contains("csrutil")
        || n.contains("logging")
        // Global Firewall / logging / Gatekeeper / SIP / FileVault stay elsewhere.
        || n == "/firewall"
        || n == "/firewall-status"
        || n == "/fw"
        || n == "/fw-status"
        || n == "/afw"
        || n == "/afw-status"
        || n == "/app-firewall"
        || n == "/application-firewall"
        || n == "/socketfilterfw"
        || n == "/firewall-logging"
        || n == "/fw-logging"
        || n == "/afw-logging"
        || n == "/allow-signed"
        || n == "/allowsigned"
        || n == "/fw-allow-signed"
        || n == "/afw-allow-signed"
        || n == "/firewall-allow-signed"
        || n == "/block-all"
        || n == "/blockall"
        || n == "/fw-block-all"
        || n == "/afw-block-all"
        || n == "/firewall-block-all"
        || n == "/getblockall"
        || n == "/listapps"
        || n == "/list-apps"
        || n == "/firewall-apps"
        || n == "/fw-apps"
        || n == "/afw-apps"
        || n == "/firewall-listapps"
        || n == "/socketfilterfw-listapps"
        || n == "/mac-firewall-apps"
        || n.contains("firewall apps")
        || n.contains("fw apps")
        || n.contains("afw apps")
        || n.contains("listapps")
        || n == "/gatekeeper"
        || n == "/gk"
        || n == "/spctl"
        || n == "/sip"
        || n == "/csrutil"
        || n == "/filevault"
        || n == "/fv"
        || n == "/fde"
        || n == "/remote-login"
        || n == "/ssh"
        || n == "/sshd"
        // Bare short / ambiguous stay with the agent.
        || n == "stealth"
        || n == "the stealth"
        || n == "stealth mode"
        || n == "the stealth mode"
        || n == "firewall stealth"
        || n == "the firewall stealth"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/stealth"
            | "/stealth-mode"
            | "/stealthmode"
            | "/stealth-status"
            | "/stealthstatus"
            | "/firewall-stealth"
            | "/firewallstealth"
            | "/fw-stealth"
            | "/fwstealth"
            | "/afw-stealth"
            | "/afwstealth"
            | "/app-firewall-stealth"
            | "/application-firewall-stealth"
            | "/socketfilterfw-stealth"
            | "stealth status"
            | "the stealth status"
            | "stealth mode status"
            | "the stealth mode status"
            | "firewall stealth status"
            | "the firewall stealth status"
            | "fw stealth status"
            | "afw stealth status"
            | "my stealth"
            | "my stealth status"
            | "my stealth mode"
            | "my firewall stealth"
            | "is stealth on"
            | "is stealth off"
            | "is stealth enabled"
            | "is stealth disabled"
            | "is the stealth on"
            | "is the stealth enabled"
            | "is stealth mode on"
            | "is stealth mode enabled"
            | "is firewall stealth on"
            | "is firewall stealth enabled"
            | "what's my stealth"
            | "whats my stealth"
            | "what is my stealth"
            | "what's my stealth status"
            | "whats my stealth status"
            | "what is my stealth status"
            | "what's my stealth mode"
            | "whats my stealth mode"
            | "what is my stealth mode"
            | "what's the stealth"
            | "whats the stealth"
            | "what is the stealth"
            | "what's the stealth status"
            | "whats the stealth status"
            | "what is the stealth status"
            | "what's the stealth mode"
            | "whats the stealth mode"
            | "what is the stealth mode"
            | "how's stealth"
            | "hows stealth"
            | "how's the stealth"
            | "hows the stealth"
            | "how's stealth mode"
            | "hows stealth mode"
            // View/see/show me/open/list.
            | "view stealth"
            | "view the stealth"
            | "view stealth status"
            | "view the stealth status"
            | "view stealth mode"
            | "view the stealth mode"
            | "view firewall stealth"
            | "view the firewall stealth"
            | "see stealth"
            | "see the stealth"
            | "see stealth status"
            | "see the stealth status"
            | "see stealth mode"
            | "see the stealth mode"
            | "see firewall stealth"
            | "see the firewall stealth"
            | "open stealth"
            | "open the stealth"
            | "open stealth status"
            | "open the stealth status"
            | "open stealth mode"
            | "open the stealth mode"
            | "open firewall stealth"
            | "open the firewall stealth"
            | "list stealth"
            | "list the stealth"
            | "list stealth status"
            | "list the stealth status"
            | "list stealth mode"
            | "list the stealth mode"
            | "list firewall stealth"
            | "list the firewall stealth"
            | "show stealth"
            | "show the stealth"
            | "show me stealth"
            | "show me the stealth"
            | "show stealth status"
            | "show the stealth status"
            | "show me stealth status"
            | "show me the stealth status"
            | "show stealth mode"
            | "show the stealth mode"
            | "show me stealth mode"
            | "show me the stealth mode"
            | "show firewall stealth"
            | "show the firewall stealth"
            | "show me firewall stealth"
            | "show me the firewall stealth"
    )
}

/// Zero-LLM Firewall stealth status from `socketfilterfw --getstealthmode` (macOS-only; empty elsewhere).
pub fn format_firewall_stealth_report() -> String {
    let v = crate::metrics::get_firewall_stealth_status();
    let v = v.trim();
    if v.is_empty() {
        return "**Stealth** — _no Firewall stealth reading on this Mac right now._".to_string();
    }
    format!("**Stealth** · `{v}`")
}

/// Application Firewall logging — not global Firewall `/firewall`, not stealth `/stealth`,
/// not allow-signed `/allow-signed`.
/// Slash + view/open NL (`/firewall-logging` · `/fw-logging` · `view firewall logging`) match
/// `/stealth` · `/firewall` shape; bare `logging` / `firewall logging` stay with the agent;
/// `/firewall` stays global; `/stealth` stays stealth; `/allow-signed` stays allow-signed;
/// `/logs` stays debug log.
pub fn looks_like_firewall_logging_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("allow ")
        || n.contains("block ")
        || n.contains("password")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("gatekeeper")
        || n.contains("spctl")
        || n.contains("system integrity")
        || n.contains("sip")
        || n.contains("csrutil")
        || n.contains("stealth")
        || n.contains("debug")
        // Global Firewall / stealth / Gatekeeper / SIP / FileVault / debug `/logs` stay elsewhere.
        || n == "/firewall"
        || n == "/firewall-status"
        || n == "/fw"
        || n == "/fw-status"
        || n == "/afw"
        || n == "/afw-status"
        || n == "/app-firewall"
        || n == "/application-firewall"
        || n == "/socketfilterfw"
        || n == "/stealth"
        || n == "/stealth-mode"
        || n == "/firewall-stealth"
        || n == "/fw-stealth"
        || n == "/afw-stealth"
        || n == "/gatekeeper"
        || n == "/gk"
        || n == "/spctl"
        || n == "/sip"
        || n == "/csrutil"
        || n == "/filevault"
        || n == "/fv"
        || n == "/fde"
        || n == "/logs"
        || n == "/log"
        || n == "/debug-log"
        || n == "/allow-signed"
        || n == "/allowsigned"
        || n == "/fw-allow-signed"
        || n == "/afw-allow-signed"
        || n == "/firewall-allow-signed"
        || n == "/block-all"
        || n == "/blockall"
        || n == "/fw-block-all"
        || n == "/afw-block-all"
        || n == "/firewall-block-all"
        || n == "/getblockall"
        || n == "/listapps"
        || n == "/list-apps"
        || n == "/firewall-apps"
        || n == "/fw-apps"
        || n == "/afw-apps"
        || n == "/firewall-listapps"
        || n == "/socketfilterfw-listapps"
        || n == "/mac-firewall-apps"
        || n.contains("firewall apps")
        || n.contains("fw apps")
        || n.contains("afw apps")
        || n.contains("listapps")
        || n == "/remote-login"
        || n == "/ssh"
        || n == "/sshd"
        // Bare short / ambiguous stay with the agent.
        || n == "logging"
        || n == "the logging"
        || n == "firewall logging"
        || n == "the firewall logging"
        || n == "logging mode"
        || n == "the logging mode"
        || n == "log"
        || n == "the log"
        || n == "logs"
        || n == "the logs"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/firewall-logging"
            | "/firewalllogging"
            | "/firewall-logging-status"
            | "/firewall-log"
            | "/firewalllog"
            | "/fw-logging"
            | "/fwlogging"
            | "/fw-logging-status"
            | "/fw-log"
            | "/fwlog"
            | "/afw-logging"
            | "/afwlogging"
            | "/afw-logging-status"
            | "/afw-log"
            | "/afwlog"
            | "/app-firewall-logging"
            | "/application-firewall-logging"
            | "/socketfilterfw-logging"
            | "/logging-mode"
            | "/loggingmode"
            | "/logging-opt"
            | "/loggingopt"
            | "/firewall-logging-mode"
            | "/firewall-logging-opt"
            | "/mac-firewall-logging"
            | "/mac-fw-logging"
            | "firewall logging status"
            | "the firewall logging status"
            | "firewall logging mode"
            | "the firewall logging mode"
            | "fw logging status"
            | "the fw logging status"
            | "afw logging status"
            | "the afw logging status"
            | "logging mode status"
            | "the logging mode status"
            | "my firewall logging"
            | "my firewall logging status"
            | "my fw logging"
            | "my logging mode"
            | "is firewall logging on"
            | "is firewall logging off"
            | "is firewall logging enabled"
            | "is firewall logging disabled"
            | "is the firewall logging on"
            | "is the firewall logging enabled"
            | "is fw logging on"
            | "is fw logging enabled"
            | "is afw logging on"
            | "is afw logging enabled"
            | "is logging mode on"
            | "is logging mode enabled"
            | "what's my firewall logging"
            | "whats my firewall logging"
            | "what is my firewall logging"
            | "what's my firewall logging status"
            | "whats my firewall logging status"
            | "what is my firewall logging status"
            | "what's my logging mode"
            | "whats my logging mode"
            | "what is my logging mode"
            | "what's the firewall logging"
            | "whats the firewall logging"
            | "what is the firewall logging"
            | "what's the firewall logging status"
            | "whats the firewall logging status"
            | "what is the firewall logging status"
            | "what's the logging mode"
            | "whats the logging mode"
            | "what is the logging mode"
            | "how's firewall logging"
            | "hows firewall logging"
            | "how's the firewall logging"
            | "hows the firewall logging"
            // View/see/show me/open/list.
            | "view firewall logging"
            | "view the firewall logging"
            | "view firewall logging status"
            | "view the firewall logging status"
            | "view fw logging"
            | "view the fw logging"
            | "view afw logging"
            | "view the afw logging"
            | "view logging mode"
            | "view the logging mode"
            | "see firewall logging"
            | "see the firewall logging"
            | "see firewall logging status"
            | "see the firewall logging status"
            | "see fw logging"
            | "see the fw logging"
            | "see logging mode"
            | "see the logging mode"
            | "open firewall logging"
            | "open the firewall logging"
            | "open firewall logging status"
            | "open the firewall logging status"
            | "open fw logging"
            | "open the fw logging"
            | "open logging mode"
            | "open the logging mode"
            | "list firewall logging"
            | "list the firewall logging"
            | "list firewall logging status"
            | "list the firewall logging status"
            | "list fw logging"
            | "list the fw logging"
            | "list logging mode"
            | "list the logging mode"
            | "show firewall logging"
            | "show the firewall logging"
            | "show me firewall logging"
            | "show me the firewall logging"
            | "show firewall logging status"
            | "show the firewall logging status"
            | "show me firewall logging status"
            | "show me the firewall logging status"
            | "show logging mode"
            | "show the logging mode"
            | "show me logging mode"
            | "show me the logging mode"
    )
}

/// Zero-LLM Firewall logging status from `socketfilterfw --getloggingmode` (+ `--getloggingopt`).
pub fn format_firewall_logging_report() -> String {
    let v = crate::metrics::get_firewall_logging_status();
    let v = v.trim();
    if v.is_empty() {
        return "**Firewall logging** — _no Firewall logging reading on this Mac right now._"
            .to_string();
    }
    format!("**Firewall logging** · `{v}`")
}

/// Application Firewall allow-signed — not global Firewall `/firewall`, not stealth `/stealth`,
/// not logging `/firewall-logging`, not block-all `/block-all`.
/// Slash + view/open NL (`/allow-signed` · `/fw-allow-signed` · `view allow signed`) match
/// `/firewall-logging` · `/stealth` shape; bare `allow signed` / `signed software` stay with the agent;
/// `/firewall` stays global; `/stealth` stays stealth; `/firewall-logging` stays logging;
/// `/block-all` stays block-all.
pub fn looks_like_firewall_allow_signed_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("block ")
        || n.contains("password")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("gatekeeper")
        || n.contains("spctl")
        || n.contains("system integrity")
        || n.contains("sip")
        || n.contains("csrutil")
        || n.contains("stealth")
        || n.contains("logging")
        // Global Firewall / stealth / logging / Gatekeeper / SIP / FileVault stay elsewhere.
        || n == "/firewall"
        || n == "/firewall-status"
        || n == "/fw"
        || n == "/fw-status"
        || n == "/afw"
        || n == "/afw-status"
        || n == "/app-firewall"
        || n == "/application-firewall"
        || n == "/socketfilterfw"
        || n == "/stealth"
        || n == "/stealth-mode"
        || n == "/firewall-stealth"
        || n == "/fw-stealth"
        || n == "/afw-stealth"
        || n == "/firewall-logging"
        || n == "/fw-logging"
        || n == "/afw-logging"
        || n == "/logging-mode"
        || n == "/logging-opt"
        || n == "/block-all"
        || n == "/blockall"
        || n == "/fw-block-all"
        || n == "/afw-block-all"
        || n == "/firewall-block-all"
        || n == "/getblockall"
        || n == "/listapps"
        || n == "/list-apps"
        || n == "/firewall-apps"
        || n == "/fw-apps"
        || n == "/afw-apps"
        || n == "/firewall-listapps"
        || n == "/socketfilterfw-listapps"
        || n == "/mac-firewall-apps"
        || n.contains("firewall apps")
        || n.contains("fw apps")
        || n.contains("afw apps")
        || n.contains("listapps")
        || n == "/gatekeeper"
        || n == "/gk"
        || n == "/spctl"
        || n == "/sip"
        || n == "/csrutil"
        || n == "/filevault"
        || n == "/fv"
        || n == "/fde"
        || n == "/logs"
        || n == "/log"
        || n == "/debug-log"
        || n == "/remote-login"
        || n == "/ssh"
        || n == "/sshd"
        // Bare short / ambiguous stay with the agent.
        || n == "allow signed"
        || n == "the allow signed"
        || n == "signed"
        || n == "the signed"
        || n == "signed software"
        || n == "the signed software"
        || n == "signed apps"
        || n == "the signed apps"
        || n == "allow signed software"
        || n == "the allow signed software"
        || n == "allow signed apps"
        || n == "the allow signed apps"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/allow-signed"
            | "/allowsigned"
            | "/allow-signed-status"
            | "/allowsignedstatus"
            | "/allow-signed-apps"
            | "/allowsignedapps"
            | "/allow-signed-software"
            | "/allowsignedsoftware"
            | "/allow-signed-built-in"
            | "/allow-signed-downloaded"
            | "/fw-allow-signed"
            | "/fwallowsigned"
            | "/fw-allow-signed-status"
            | "/afw-allow-signed"
            | "/afwallowsigned"
            | "/afw-allow-signed-status"
            | "/firewall-allow-signed"
            | "/firewallallowsigned"
            | "/firewall-allow-signed-status"
            | "/app-firewall-allow-signed"
            | "/application-firewall-allow-signed"
            | "/socketfilterfw-allow-signed"
            | "/getallowsigned"
            | "/mac-allow-signed"
            | "/mac-fw-allow-signed"
            | "allow signed status"
            | "the allow signed status"
            | "allow signed apps status"
            | "the allow signed apps status"
            | "allow signed software status"
            | "the allow signed software status"
            | "fw allow signed status"
            | "the fw allow signed status"
            | "afw allow signed status"
            | "the afw allow signed status"
            | "firewall allow signed status"
            | "the firewall allow signed status"
            | "my allow signed"
            | "my allow signed status"
            | "my fw allow signed"
            | "is allow signed on"
            | "is allow signed off"
            | "is allow signed enabled"
            | "is allow signed disabled"
            | "is the allow signed on"
            | "is the allow signed enabled"
            | "is fw allow signed on"
            | "is fw allow signed enabled"
            | "is afw allow signed on"
            | "is afw allow signed enabled"
            | "is signed software allowed"
            | "is signed apps allowed"
            | "what's my allow signed"
            | "whats my allow signed"
            | "what is my allow signed"
            | "what's my allow signed status"
            | "whats my allow signed status"
            | "what is my allow signed status"
            | "what's the allow signed"
            | "whats the allow signed"
            | "what is the allow signed"
            | "what's the allow signed status"
            | "whats the allow signed status"
            | "what is the allow signed status"
            | "how's allow signed"
            | "hows allow signed"
            | "how's the allow signed"
            | "hows the allow signed"
            // View/see/show me/open/list.
            | "view allow signed"
            | "view the allow signed"
            | "view allow signed status"
            | "view the allow signed status"
            | "view allow signed apps"
            | "view the allow signed apps"
            | "view allow signed software"
            | "view the allow signed software"
            | "view fw allow signed"
            | "view the fw allow signed"
            | "view afw allow signed"
            | "view the afw allow signed"
            | "view firewall allow signed"
            | "view the firewall allow signed"
            | "see allow signed"
            | "see the allow signed"
            | "see allow signed status"
            | "see the allow signed status"
            | "see allow signed apps"
            | "see the allow signed apps"
            | "see fw allow signed"
            | "see the fw allow signed"
            | "open allow signed"
            | "open the allow signed"
            | "open allow signed status"
            | "open the allow signed status"
            | "open allow signed apps"
            | "open the allow signed apps"
            | "open fw allow signed"
            | "open the fw allow signed"
            | "list allow signed"
            | "list the allow signed"
            | "list allow signed status"
            | "list the allow signed status"
            | "list allow signed apps"
            | "list the allow signed apps"
            | "list fw allow signed"
            | "list the fw allow signed"
            | "show allow signed"
            | "show the allow signed"
            | "show me allow signed"
            | "show me the allow signed"
            | "show allow signed status"
            | "show the allow signed status"
            | "show me allow signed status"
            | "show me the allow signed status"
            | "show allow signed apps"
            | "show the allow signed apps"
            | "show me allow signed apps"
            | "show me the allow signed apps"
    )
}

/// Zero-LLM Firewall allow-signed status from `socketfilterfw --getallowsigned`.
pub fn format_firewall_allow_signed_report() -> String {
    let v = crate::metrics::get_firewall_allow_signed_status();
    let v = v.trim();
    if v.is_empty() {
        return "**Allow signed** — _no Firewall allow-signed reading on this Mac right now._"
            .to_string();
    }
    format!("**Allow signed** · `{v}`")
}

/// Application Firewall block-all — not global Firewall `/firewall`, not stealth `/stealth`,
/// not logging `/firewall-logging`, not allow-signed `/allow-signed`.
/// Slash + view/open NL (`/block-all` · `/fw-block-all` · `view block all`) match
/// `/allow-signed` · `/stealth` shape; bare `block all` / `blockall` stay with the agent;
/// `/firewall` stays global; `/stealth` stays stealth; `/firewall-logging` stays logging;
/// `/allow-signed` stays allow-signed.
pub fn looks_like_firewall_block_all_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("allow ")
        || n.contains("password")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("gatekeeper")
        || n.contains("spctl")
        || n.contains("system integrity")
        || n.contains("sip")
        || n.contains("csrutil")
        || n.contains("stealth")
        || n.contains("logging")
        || n.contains("allow signed")
        || n.contains("signed software")
        || n.contains("signed apps")
        // Global Firewall / stealth / logging / allow-signed / Gatekeeper / SIP / FileVault stay elsewhere.
        || n == "/firewall"
        || n == "/firewall-status"
        || n == "/fw"
        || n == "/fw-status"
        || n == "/afw"
        || n == "/afw-status"
        || n == "/app-firewall"
        || n == "/application-firewall"
        || n == "/socketfilterfw"
        || n == "/stealth"
        || n == "/stealth-mode"
        || n == "/firewall-stealth"
        || n == "/fw-stealth"
        || n == "/afw-stealth"
        || n == "/firewall-logging"
        || n == "/fw-logging"
        || n == "/afw-logging"
        || n == "/logging-mode"
        || n == "/logging-opt"
        || n == "/allow-signed"
        || n == "/allowsigned"
        || n == "/fw-allow-signed"
        || n == "/afw-allow-signed"
        || n == "/firewall-allow-signed"
        || n == "/getallowsigned"
        || n == "/listapps"
        || n == "/list-apps"
        || n == "/firewall-apps"
        || n == "/fw-apps"
        || n == "/afw-apps"
        || n == "/firewall-listapps"
        || n == "/socketfilterfw-listapps"
        || n == "/mac-firewall-apps"
        || n.contains("firewall apps")
        || n.contains("fw apps")
        || n.contains("afw apps")
        || n.contains("listapps")
        || n == "/gatekeeper"
        || n == "/gk"
        || n == "/spctl"
        || n == "/sip"
        || n == "/csrutil"
        || n == "/filevault"
        || n == "/fv"
        || n == "/fde"
        || n == "/logs"
        || n == "/log"
        || n == "/debug-log"
        || n == "/remote-login"
        || n == "/ssh"
        || n == "/sshd"
        // Bare short / ambiguous stay with the agent.
        || n == "block all"
        || n == "the block all"
        || n == "blockall"
        || n == "the blockall"
        || n == "block all incoming"
        || n == "the block all incoming"
        || n == "block all connections"
        || n == "the block all connections"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/block-all"
            | "/blockall"
            | "/block-all-status"
            | "/blockallstatus"
            | "/block-all-state"
            | "/blockallstate"
            | "/block-all-incoming"
            | "/blockallincoming"
            | "/firewall-block-all"
            | "/firewallblockall"
            | "/firewall-block-all-status"
            | "/fw-block-all"
            | "/fwblockall"
            | "/fw-block-all-status"
            | "/afw-block-all"
            | "/afwblockall"
            | "/afw-block-all-status"
            | "/app-firewall-block-all"
            | "/application-firewall-block-all"
            | "/socketfilterfw-block-all"
            | "/getblockall"
            | "/mac-block-all"
            | "/mac-fw-block-all"
            | "block all status"
            | "the block all status"
            | "block all state"
            | "the block all state"
            | "block all incoming status"
            | "the block all incoming status"
            | "fw block all status"
            | "the fw block all status"
            | "afw block all status"
            | "the afw block all status"
            | "firewall block all status"
            | "the firewall block all status"
            | "my block all"
            | "my block all status"
            | "my fw block all"
            | "is block all on"
            | "is block all off"
            | "is block all enabled"
            | "is block all disabled"
            | "is the block all on"
            | "is the block all enabled"
            | "is fw block all on"
            | "is fw block all enabled"
            | "is afw block all on"
            | "is afw block all enabled"
            | "is block all incoming on"
            | "is block all incoming enabled"
            | "what's my block all"
            | "whats my block all"
            | "what is my block all"
            | "what's my block all status"
            | "whats my block all status"
            | "what is my block all status"
            | "what's the block all"
            | "whats the block all"
            | "what is the block all"
            | "what's the block all status"
            | "whats the block all status"
            | "what is the block all status"
            | "how's block all"
            | "hows block all"
            | "how's the block all"
            | "hows the block all"
            // View/see/show me/open/list.
            | "view block all"
            | "view the block all"
            | "view block all status"
            | "view the block all status"
            | "view block all incoming"
            | "view the block all incoming"
            | "view fw block all"
            | "view the fw block all"
            | "view afw block all"
            | "view the afw block all"
            | "view firewall block all"
            | "view the firewall block all"
            | "see block all"
            | "see the block all"
            | "see block all status"
            | "see the block all status"
            | "see fw block all"
            | "see the fw block all"
            | "open block all"
            | "open the block all"
            | "open block all status"
            | "open the block all status"
            | "open fw block all"
            | "open the fw block all"
            | "list block all"
            | "list the block all"
            | "list block all status"
            | "list the block all status"
            | "list fw block all"
            | "list the fw block all"
            | "show block all"
            | "show the block all"
            | "show me block all"
            | "show me the block all"
            | "show block all status"
            | "show the block all status"
            | "show me block all status"
            | "show me the block all status"
            | "show fw block all"
            | "show the fw block all"
            | "show me fw block all"
            | "show me the fw block all"
    )
}

/// Zero-LLM Firewall block-all status from `socketfilterfw --getblockall`.
pub fn format_firewall_block_all_report() -> String {
    let v = crate::metrics::get_firewall_block_all_status();
    let v = v.trim();
    if v.is_empty() {
        return "**Block all** — _no Firewall block-all reading on this Mac right now._".to_string();
    }
    format!("**Block all** · `{v}`")
}

/// Application Firewall listed apps — not global Firewall `/firewall`, not stealth `/stealth`,
/// not logging `/firewall-logging`, not allow-signed `/allow-signed`, not block-all `/block-all`.
/// Slash + view/open NL (`/listapps` · `/firewall-apps` · `view firewall apps`) match `/block-all` shape;
/// bare `list apps` / `firewall apps` / `apps` stay with the agent; `list allow signed apps` stays allow-signed;
/// `list app firewall` stays global Firewall.
pub fn looks_like_firewall_listapps_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("password")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("gatekeeper")
        || n.contains("spctl")
        || n.contains("system integrity")
        || n.contains("sip")
        || n.contains("csrutil")
        || n.contains("stealth")
        || n.contains("logging")
        || n.contains("allow signed")
        || n.contains("signed software")
        || n.contains("signed apps")
        || n.contains("block all")
        || n.contains("blockall")
        // Global Firewall / stealth / logging / allow-signed / block-all / Gatekeeper / SIP / FileVault stay elsewhere.
        || n == "/firewall"
        || n == "/firewall-status"
        || n == "/fw"
        || n == "/fw-status"
        || n == "/afw"
        || n == "/afw-status"
        || n == "/app-firewall"
        || n == "/application-firewall"
        || n == "/socketfilterfw"
        || n == "/stealth"
        || n == "/stealth-mode"
        || n == "/firewall-stealth"
        || n == "/fw-stealth"
        || n == "/afw-stealth"
        || n == "/firewall-logging"
        || n == "/fw-logging"
        || n == "/afw-logging"
        || n == "/logging-mode"
        || n == "/logging-opt"
        || n == "/allow-signed"
        || n == "/allowsigned"
        || n == "/fw-allow-signed"
        || n == "/afw-allow-signed"
        || n == "/firewall-allow-signed"
        || n == "/getallowsigned"
        || n == "/block-all"
        || n == "/blockall"
        || n == "/fw-block-all"
        || n == "/afw-block-all"
        || n == "/firewall-block-all"
        || n == "/getblockall"
        || n == "/gatekeeper"
        || n == "/gk"
        || n == "/spctl"
        || n == "/sip"
        || n == "/csrutil"
        || n == "/filevault"
        || n == "/fv"
        || n == "/fde"
        || n == "/logs"
        || n == "/log"
        || n == "/debug-log"
        || n == "/remote-login"
        || n == "/ssh"
        || n == "/sshd"
        || n == "list app firewall"
        || n == "list the app firewall"
        || n == "list application firewall"
        || n == "list the application firewall"
        // Bare short / ambiguous stay with the agent.
        || n == "list apps"
        || n == "the list apps"
        || n == "listapps"
        || n == "the listapps"
        || n == "apps"
        || n == "the apps"
        || n == "firewall apps"
        || n == "the firewall apps"
        || n == "fw apps"
        || n == "the fw apps"
        || n == "afw apps"
        || n == "the afw apps"
        || n == "app firewall apps"
        || n == "the app firewall apps"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/listapps"
            | "/list-apps"
            | "/listapps-status"
            | "/list-apps-status"
            | "/firewall-apps"
            | "/firewallapps"
            | "/firewall-apps-status"
            | "/firewall-listapps"
            | "/firewalllistapps"
            | "/fw-apps"
            | "/fwapps"
            | "/fw-apps-status"
            | "/fw-listapps"
            | "/afw-apps"
            | "/afwapps"
            | "/afw-apps-status"
            | "/afw-listapps"
            | "/app-firewall-apps"
            | "/application-firewall-apps"
            | "/socketfilterfw-listapps"
            | "/socketfilterfw-apps"
            | "/mac-firewall-apps"
            | "/mac-fw-apps"
            | "/mac-listapps"
            | "firewall apps status"
            | "the firewall apps status"
            | "fw apps status"
            | "the fw apps status"
            | "afw apps status"
            | "the afw apps status"
            | "listapps status"
            | "the listapps status"
            | "my firewall apps"
            | "my firewall apps status"
            | "my fw apps"
            | "my listapps"
            | "what's my firewall apps"
            | "whats my firewall apps"
            | "what is my firewall apps"
            | "what's my firewall apps status"
            | "whats my firewall apps status"
            | "what is my firewall apps status"
            | "what's the firewall apps"
            | "whats the firewall apps"
            | "what is the firewall apps"
            | "what's the firewall apps status"
            | "whats the firewall apps status"
            | "what is the firewall apps status"
            | "how's firewall apps"
            | "hows firewall apps"
            | "how's the firewall apps"
            | "hows the firewall apps"
            // View/see/show me/open/list.
            | "view firewall apps"
            | "view the firewall apps"
            | "view firewall apps status"
            | "view the firewall apps status"
            | "view fw apps"
            | "view the fw apps"
            | "view afw apps"
            | "view the afw apps"
            | "view listapps"
            | "view the listapps"
            | "view app firewall apps"
            | "view the app firewall apps"
            | "see firewall apps"
            | "see the firewall apps"
            | "see firewall apps status"
            | "see the firewall apps status"
            | "see fw apps"
            | "see the fw apps"
            | "open firewall apps"
            | "open the firewall apps"
            | "open firewall apps status"
            | "open the firewall apps status"
            | "open fw apps"
            | "open the fw apps"
            | "open listapps"
            | "open the listapps"
            | "list firewall apps"
            | "list the firewall apps"
            | "list firewall apps status"
            | "list the firewall apps status"
            | "list fw apps"
            | "list the fw apps"
            | "list afw apps"
            | "list the afw apps"
            | "list app firewall apps"
            | "list the app firewall apps"
            | "list application firewall apps"
            | "list the application firewall apps"
            | "show firewall apps"
            | "show the firewall apps"
            | "show me firewall apps"
            | "show me the firewall apps"
            | "show firewall apps status"
            | "show the firewall apps status"
            | "show me firewall apps status"
            | "show me the firewall apps status"
            | "show fw apps"
            | "show the fw apps"
            | "show me fw apps"
            | "show me the fw apps"
    )
}

/// Zero-LLM Firewall listed apps from `socketfilterfw --listapps`.
pub fn format_firewall_listapps_report() -> String {
    let v = crate::metrics::get_firewall_listapps_status();
    let v = v.trim();
    if v.is_empty() {
        return "**Firewall apps** — _no Application Firewall app list on this Mac right now._"
            .to_string();
    }
    format!("**Firewall apps** · {v}")
}

/// Time Machine session + destination — not Disk Cleanup, not SSD % / free, not boot volume.
/// Slash + view/open NL (`/timemachine` · `/tm` · `view time machine`) match `/firewall` · `/sip` shape;
/// bare `backup` / `tm` / `time machine` stay with the agent; `view disk` stays Disk Cleanup.
pub fn looks_like_time_machine_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.contains("thin")
        || n.contains("snapshot")
        || n.contains("prune")
        // Action verbs (leave status asks like "is time machine running").
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("start ")
        || n.contains("stop ")
        || n.contains("exclude")
        || n.contains("password")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("firewall")
        || n.contains("stealth")
        || n.contains("gatekeeper")
        || n.contains("filevault")
        || n.contains("system integrity")
        || n == "/firewall"
        || n == "/fw"
        || n == "/afw"
        || n == "/stealth"
        || n == "/gatekeeper"
        || n == "/sip"
        || n == "/filevault"
        || n == "/boot-volume"
        || n == "/startup-disk"
        || n == "/remote-login"
        || n == "/remotelogin"
        || n == "/ssh"
        || n == "/sshd"
        || n == "view disk"
        || n == "open disk"
        || n == "/ssd"
        || n == "/storage"
        || n == "/free"
        || n == "/tasks"
        // Bare short / ambiguous stay with the agent.
        || n == "tm"
        || n == "the tm"
        || n == "backup"
        || n == "the backup"
        || n == "backups"
        || n == "time machine"
        || n == "the time machine"
        || n == "timemachine"
        || n == "the timemachine"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/timemachine"
            | "/time-machine"
            | "/timemachine-status"
            | "/time-machine-status"
            | "/tm"
            | "/tm-status"
            | "/tmstatus"
            | "/tmutil"
            | "/tmutil-status"
            | "/backup-status"
            | "/backupstatus"
            | "/tm-backup"
            | "/tmbackup"
            | "/time-machine-backup"
            | "time machine status"
            | "the time machine status"
            | "timemachine status"
            | "the timemachine status"
            | "tm status"
            | "the tm status"
            | "backup status"
            | "the backup status"
            | "my time machine"
            | "my time machine status"
            | "my timemachine"
            | "my backup status"
            | "is time machine on"
            | "is time machine off"
            | "is time machine running"
            | "is time machine idle"
            | "is timemachine on"
            | "is timemachine running"
            | "is tm running"
            | "is backup running"
            | "what's my time machine"
            | "whats my time machine"
            | "what is my time machine"
            | "what's my time machine status"
            | "whats my time machine status"
            | "what is my time machine status"
            | "what's the time machine"
            | "whats the time machine"
            | "what is the time machine"
            | "what's the time machine status"
            | "whats the time machine status"
            | "what is the time machine status"
            | "what's my backup status"
            | "whats my backup status"
            | "what is my backup status"
            | "how's time machine"
            | "hows time machine"
            | "how's the time machine"
            | "hows the time machine"
            // View/see/show me/open/list.
            | "view time machine"
            | "view the time machine"
            | "view time machine status"
            | "view the time machine status"
            | "view timemachine"
            | "view the timemachine"
            | "view tm"
            | "view the tm"
            | "view tm status"
            | "view backup status"
            | "view the backup status"
            | "see time machine"
            | "see the time machine"
            | "see time machine status"
            | "see the time machine status"
            | "see backup status"
            | "see the backup status"
            | "open time machine"
            | "open the time machine"
            | "open time machine status"
            | "open the time machine status"
            | "open tm"
            | "open the tm"
            | "open tm status"
            | "open the tm status"
            | "open backup status"
            | "open the backup status"
            | "list time machine"
            | "list the time machine"
            | "list time machine status"
            | "list the time machine status"
            | "list backup status"
            | "list the backup status"
            | "show time machine"
            | "show the time machine"
            | "show me time machine"
            | "show me the time machine"
            | "show time machine status"
            | "show the time machine status"
            | "show me time machine status"
            | "show me the time machine status"
            | "show backup status"
            | "show the backup status"
            | "show me backup status"
            | "show me the backup status"
    )
}

/// Zero-LLM Time Machine status from `tmutil status` + `destinationinfo` (macOS-only; empty elsewhere).
pub fn format_time_machine_report() -> String {
    let v = crate::metrics::get_time_machine_status();
    let v = v.trim();
    if v.is_empty() {
        return "**Time Machine** — _no Time Machine reading on this Mac right now._".to_string();
    }
    format!("**Time Machine** · `{v}`")
}

/// Remote Login (SSH) — not Firewall, not stealth, not VPN, not Time Machine.
/// Slash + view/open NL (`/remote-login` · `/ssh` · `view remote login`) match `/firewall` · `/sip` shape;
/// bare `ssh` / `sshd` / `remote login` stay with the agent; `/vpn` stays VPN; `/firewall` stays Firewall.
pub fn looks_like_remote_login_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        // Action verbs (leave status asks like "is remote login on").
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("password")
        || n.contains("key ")
        || n.contains(" keys")
        || n.contains("tunnel")
        || n.contains("scp")
        || n.contains("sftp")
        || n.contains("connect")
        || n.contains("screen sharing")
        || n.contains("screensharing")
        || n.contains("screen-sharing")
        || n.contains("screen share")
        || n.contains("vnc")
        || n.contains("ard")
        || n.contains("remote management")
        || n.contains("remote desktop")
        || n.contains("remote apple events")
        || n.contains("remote-apple-events")
        || n.contains("remoteappleevents")
        || n.contains("apple events")
        || n.contains("apple-events")
        || n.contains("eppc")
        || n.contains("file sharing")
        || n.contains("filesharing")
        || n.contains("file-sharing")
        || n.contains("file share")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("firewall")
        || n.contains("stealth")
        || n.contains("gatekeeper")
        || n.contains("filevault")
        || n.contains("system integrity")
        || n.contains("time machine")
        || n.contains("timemachine")
        || n.contains("vpn")
        || n == "/firewall"
        || n == "/fw"
        || n == "/afw"
        || n == "/stealth"
        || n == "/gatekeeper"
        || n == "/sip"
        || n == "/filevault"
        || n == "/timemachine"
        || n == "/tm"
        || n == "/vpn"
        || n == "/ip"
        || n == "/wifi"
        || n == "/tasks"
        || n == "/screen-sharing"
        || n == "/screensharing"
        || n == "/screen-share"
        || n == "/vnc"
        || n == "/vnc-status"
        || n == "/file-sharing"
        || n == "/filesharing"
        || n == "/smb"
        || n == "/smbd"
        || n == "/remote-apple-events"
        || n == "/remoteappleevents"
        || n == "/apple-events"
        || n == "/rae"
        || n == "/eppc"
        || n == "/aeserver"
        || n == "/screen"
        || n == "/screens"
        // Bare short / ambiguous stay with the agent.
        || n == "ssh"
        || n == "the ssh"
        || n == "sshd"
        || n == "the sshd"
        || n == "remote login"
        || n == "the remote login"
        || n == "remote-login"
        || n == "the remote-login"
        || n == "remotelogin"
        || n == "the remotelogin"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/remote-login"
            | "/remotelogin"
            | "/remote-login-status"
            | "/remotelogin-status"
            | "/remote-login-state"
            | "/ssh"
            | "/sshd"
            | "/ssh-status"
            | "/sshd-status"
            | "/ssh-login"
            | "/sshlogin"
            | "/remote-ssh"
            | "/remotessh"
            | "/ssh-remote"
            | "/sshd-remote"
            | "remote login status"
            | "the remote login status"
            | "remote-login status"
            | "the remote-login status"
            | "remotelogin status"
            | "the remotelogin status"
            | "ssh status"
            | "the ssh status"
            | "sshd status"
            | "the sshd status"
            | "ssh login status"
            | "the ssh login status"
            | "my remote login"
            | "my remote login status"
            | "my ssh"
            | "my ssh status"
            | "my sshd"
            | "my sshd status"
            | "is remote login on"
            | "is remote login off"
            | "is remote login enabled"
            | "is remote login disabled"
            | "is remotelogin on"
            | "is remotelogin enabled"
            | "is ssh on"
            | "is ssh off"
            | "is ssh enabled"
            | "is ssh disabled"
            | "is sshd on"
            | "is sshd enabled"
            | "is ssh login on"
            | "is ssh login enabled"
            | "what's my remote login"
            | "whats my remote login"
            | "what is my remote login"
            | "what's my remote login status"
            | "whats my remote login status"
            | "what is my remote login status"
            | "what's the remote login"
            | "whats the remote login"
            | "what is the remote login"
            | "what's the remote login status"
            | "whats the remote login status"
            | "what is the remote login status"
            | "what's my ssh"
            | "whats my ssh"
            | "what is my ssh"
            | "what's my ssh status"
            | "whats my ssh status"
            | "what is my ssh status"
            | "what's the ssh status"
            | "whats the ssh status"
            | "what is the ssh status"
            | "how's remote login"
            | "hows remote login"
            | "how's the remote login"
            | "hows the remote login"
            // View/see/show me/open/list.
            | "view remote login"
            | "view the remote login"
            | "view remote login status"
            | "view the remote login status"
            | "view remotelogin"
            | "view the remotelogin"
            | "view ssh"
            | "view the ssh"
            | "view ssh status"
            | "view the ssh status"
            | "view sshd"
            | "view the sshd"
            | "view sshd status"
            | "view the sshd status"
            | "view ssh login"
            | "view the ssh login"
            | "see remote login"
            | "see the remote login"
            | "see remote login status"
            | "see the remote login status"
            | "see ssh"
            | "see the ssh"
            | "see ssh status"
            | "see the ssh status"
            | "open remote login"
            | "open the remote login"
            | "open remote login status"
            | "open the remote login status"
            | "open ssh"
            | "open the ssh"
            | "open ssh status"
            | "open the ssh status"
            | "open sshd"
            | "open the sshd"
            | "open sshd status"
            | "open the sshd status"
            | "list remote login"
            | "list the remote login"
            | "list remote login status"
            | "list the remote login status"
            | "list ssh"
            | "list the ssh"
            | "list ssh status"
            | "list the ssh status"
            | "show remote login"
            | "show the remote login"
            | "show me remote login"
            | "show me the remote login"
            | "show remote login status"
            | "show the remote login status"
            | "show me remote login status"
            | "show me the remote login status"
            | "show ssh"
            | "show the ssh"
            | "show me ssh"
            | "show me the ssh"
            | "show ssh status"
            | "show the ssh status"
            | "show me ssh status"
            | "show me the ssh status"
    )
}

/// Zero-LLM Remote Login (SSH) status from `systemsetup -getremotelogin` (macOS-only; empty elsewhere).
pub fn format_remote_login_report() -> String {
    let v = crate::metrics::get_remote_login_status();
    let v = v.trim();
    if v.is_empty() {
        return "**Remote Login** — _no Remote Login reading on this Mac right now._".to_string();
    }
    format!("**Remote Login** · `{v}`")
}

/// Screen Sharing / VNC — Sharing pref enablement + daemon idle/active.
/// "Why" / enable/disable stay with the agent. Slash + view/open NL (`/screen-sharing` · `/vnc`).
/// Bare `screen sharing` / `vnc` stay with the agent; `/remote-login` · `/ssh` stay Remote Login;
/// `/screen` · `/screens` stay display resolution.
pub fn looks_like_screen_sharing_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("password")
        || n.contains("screenshot")
        || n.contains("resolution")
        || n.contains("display")
        || n.contains("connect")
        || n.contains("tunnel")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("firewall")
        || n.contains("stealth")
        || n.contains("gatekeeper")
        || n.contains("filevault")
        || n.contains("system integrity")
        || n.contains("time machine")
        || n.contains("timemachine")
        || n.contains("vpn")
        || n.contains("remote login")
        || n.contains("remotelogin")
        || n.contains("remote-login")
        || n == "/firewall"
        || n == "/fw"
        || n == "/afw"
        || n == "/stealth"
        || n == "/gatekeeper"
        || n == "/sip"
        || n == "/filevault"
        || n == "/timemachine"
        || n == "/tm"
        || n == "/vpn"
        || n == "/remote-login"
        || n == "/remotelogin"
        || n == "/ssh"
        || n == "/sshd"
        || n == "/ssh-status"
        || n == "/sshd-status"
        || n == "/file-sharing"
        || n == "/filesharing"
        || n == "/smb"
        || n == "/smbd"
        || n == "/ip"
        || n == "/wifi"
        || n == "/tasks"
        || n == "/screen"
        || n == "/screens"
        || n == "/display"
        || n == "/displays"
        || n == "/screen-lock"
        || n == "/screenlock"
        || n == "/lock-screen"
        || n == "/lockscreen"
        || n.contains("screen lock")
        || n.contains("screen-lock")
        || n.contains("screenlock")
        || n.contains("lock screen")
        || n.contains("lock-screen")
        || n.contains("lockscreen")
        || n.contains("file sharing")
        || n.contains("filesharing")
        || n.contains("file-sharing")
        || n == "smb"
        || n == "smbd"
        // Bare short / ambiguous stay with the agent.
        || n == "vnc"
        || n == "the vnc"
        || n == "screen sharing"
        || n == "the screen sharing"
        || n == "screensharing"
        || n == "the screensharing"
        || n == "screen-sharing"
        || n == "the screen-sharing"
        || n == "screen share"
        || n == "the screen share"
        || n == "ard"
        || n == "the ard"
        || n == "remote management"
        || n == "the remote management"
        || n == "remote desktop"
        || n == "the remote desktop"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/screen-sharing"
            | "/screensharing"
            | "/screen-sharing-status"
            | "/screensharing-status"
            | "/screen-share"
            | "/screenshare"
            | "/screen-share-status"
            | "/vnc"
            | "/vnc-status"
            | "/vnc-state"
            | "/vnc-sharing"
            | "/mac-vnc"
            | "/sharing-screen"
            | "screen sharing status"
            | "the screen sharing status"
            | "screensharing status"
            | "the screensharing status"
            | "screen-sharing status"
            | "the screen-sharing status"
            | "screen share status"
            | "the screen share status"
            | "vnc status"
            | "the vnc status"
            | "my screen sharing"
            | "my screen sharing status"
            | "my screensharing"
            | "my vnc"
            | "my vnc status"
            | "is screen sharing on"
            | "is screen sharing off"
            | "is screen sharing enabled"
            | "is screen sharing disabled"
            | "is screensharing on"
            | "is screensharing enabled"
            | "is screen-sharing on"
            | "is screen-sharing enabled"
            | "is vnc on"
            | "is vnc off"
            | "is vnc enabled"
            | "is vnc disabled"
            | "what's my screen sharing"
            | "whats my screen sharing"
            | "what is my screen sharing"
            | "what's my screen sharing status"
            | "whats my screen sharing status"
            | "what is my screen sharing status"
            | "what's the screen sharing"
            | "whats the screen sharing"
            | "what is the screen sharing"
            | "what's the screen sharing status"
            | "whats the screen sharing status"
            | "what is the screen sharing status"
            | "what's my vnc"
            | "whats my vnc"
            | "what is my vnc"
            | "what's my vnc status"
            | "whats my vnc status"
            | "what is my vnc status"
            | "what's the vnc status"
            | "whats the vnc status"
            | "what is the vnc status"
            | "how's screen sharing"
            | "hows screen sharing"
            | "how's the screen sharing"
            | "hows the screen sharing"
            // View/see/show me/open/list.
            | "view screen sharing"
            | "view the screen sharing"
            | "view screen sharing status"
            | "view the screen sharing status"
            | "view screensharing"
            | "view the screensharing"
            | "view screen-sharing"
            | "view the screen-sharing"
            | "view screen share"
            | "view the screen share"
            | "view vnc"
            | "view the vnc"
            | "view vnc status"
            | "view the vnc status"
            | "see screen sharing"
            | "see the screen sharing"
            | "see screen sharing status"
            | "see the screen sharing status"
            | "see vnc"
            | "see the vnc"
            | "see vnc status"
            | "see the vnc status"
            | "open screen sharing"
            | "open the screen sharing"
            | "open screen sharing status"
            | "open the screen sharing status"
            | "open screensharing"
            | "open the screensharing"
            | "open vnc"
            | "open the vnc"
            | "open vnc status"
            | "open the vnc status"
            | "list screen sharing"
            | "list the screen sharing"
            | "list screen sharing status"
            | "list the screen sharing status"
            | "list vnc"
            | "list the vnc"
            | "list vnc status"
            | "list the vnc status"
            | "show screen sharing"
            | "show the screen sharing"
            | "show me screen sharing"
            | "show me the screen sharing"
            | "show screen sharing status"
            | "show the screen sharing status"
            | "show me screen sharing status"
            | "show me the screen sharing status"
            | "show vnc"
            | "show the vnc"
            | "show me vnc"
            | "show me the vnc"
            | "show vnc status"
            | "show the vnc status"
            | "show me vnc status"
            | "show me the vnc status"
    )
}

/// Zero-LLM Screen Sharing status from `launchctl` (macOS-only; empty elsewhere).
pub fn format_screen_sharing_report() -> String {
    let v = crate::metrics::get_screen_sharing_status();
    let v = v.trim();
    if v.is_empty() {
        return "**Screen Sharing** — _no Screen Sharing reading on this Mac right now._".to_string();
    }
    format!("**Screen Sharing** · `{v}`")
}

/// File Sharing / SMB — Sharing pref enablement + daemon idle/active.
/// "Why" / enable/disable stay with the agent. Slash + view/open NL (`/file-sharing` · `/smb`).
/// Bare `file sharing` / `smb` / `smbd` stay with the agent; `/screen-sharing` · `/vnc` stay Screen Sharing;
/// `/remote-login` · `/ssh` stay Remote Login; `/filevault` stays FileVault.
pub fn looks_like_file_sharing_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("password")
        || n.contains("screenshot")
        || n.contains("resolution")
        || n.contains("display")
        || n.contains("connect")
        || n.contains("tunnel")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("firewall")
        || n.contains("stealth")
        || n.contains("gatekeeper")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("file-vault")
        || n.contains("system integrity")
        || n.contains("time machine")
        || n.contains("timemachine")
        || n.contains("vpn")
        || n.contains("remote login")
        || n.contains("remotelogin")
        || n.contains("remote-login")
        || n.contains("screen sharing")
        || n.contains("screensharing")
        || n.contains("screen-sharing")
        || n.contains("screen share")
        || n == "/firewall"
        || n == "/fw"
        || n == "/afw"
        || n == "/stealth"
        || n == "/gatekeeper"
        || n == "/sip"
        || n == "/filevault"
        || n == "/file-vault"
        || n == "/timemachine"
        || n == "/tm"
        || n == "/vpn"
        || n == "/remote-login"
        || n == "/remotelogin"
        || n == "/ssh"
        || n == "/sshd"
        || n == "/ssh-status"
        || n == "/sshd-status"
        || n == "/screen-sharing"
        || n == "/screensharing"
        || n == "/screen-share"
        || n == "/vnc"
        || n == "/vnc-status"
        || n == "/mac-vnc"
        || n == "/printer-sharing"
        || n == "/printersharing"
        || n == "/printer-share"
        || n == "/share-printers"
        || n == "/cups-share"
        || n == "/cups-sharing"
        || n.contains("printer sharing")
        || n.contains("printersharing")
        || n.contains("printer-sharing")
        || n.contains("share printers")
        || n.contains("cups share")
        || n.contains("cups sharing")
        || n.contains("remote management")
        || n.contains("remotemanagement")
        || n.contains("remote-management")
        || n.contains("remote desktop")
        || n.contains("apple remote desktop")
        || n.contains("smb guest")
        || n.contains("smb-guest")
        || n.contains("smbguest")
        || n.contains("guest access")
        || n.contains("guest-access")
        || n == "/remote-management"
        || n == "/remotemanagement"
        || n == "/remote-mgmt"
        || n == "/ard"
        || n == "/ard-status"
        || n == "/ard-state"
        || n == "/ard-agent"
        || n == "/apple-remote-desktop"
        || n == "/mac-ard"
        || n == "/smb-guest"
        || n == "/smbguest"
        || n == "/smb-guest-access"
        || n == "/guest-access"
        || n == "/guestaccess"
        || n == "/mac-smb-guest"
        || n == "/ip"
        || n == "/wifi"
        || n == "/tasks"
        || n == "/screen"
        || n == "/screens"
        || n == "/display"
        || n == "/displays"
        // Bare short / ambiguous stay with the agent.
        || n == "smb"
        || n == "the smb"
        || n == "smbd"
        || n == "the smbd"
        || n == "file sharing"
        || n == "the file sharing"
        || n == "filesharing"
        || n == "the filesharing"
        || n == "file-sharing"
        || n == "the file-sharing"
        || n == "file share"
        || n == "the file share"
        || n == "afp"
        || n == "the afp"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/file-sharing"
            | "/filesharing"
            | "/file-sharing-status"
            | "/filesharing-status"
            | "/file-share"
            | "/fileshare"
            | "/file-share-status"
            | "/smb"
            | "/smb-status"
            | "/smb-state"
            | "/smbd"
            | "/smbd-status"
            | "/mac-smb"
            | "/mac-smbd"
            | "/sharing-files"
            | "file sharing status"
            | "the file sharing status"
            | "filesharing status"
            | "the filesharing status"
            | "file-sharing status"
            | "the file-sharing status"
            | "file share status"
            | "the file share status"
            | "smb status"
            | "the smb status"
            | "smbd status"
            | "the smbd status"
            | "my file sharing"
            | "my file sharing status"
            | "my filesharing"
            | "my smb"
            | "my smb status"
            | "is file sharing on"
            | "is file sharing off"
            | "is file sharing enabled"
            | "is file sharing disabled"
            | "is filesharing on"
            | "is filesharing enabled"
            | "is file-sharing on"
            | "is file-sharing enabled"
            | "is smb on"
            | "is smb off"
            | "is smb enabled"
            | "is smb disabled"
            | "is smbd on"
            | "is smbd enabled"
            | "what's my file sharing"
            | "whats my file sharing"
            | "what is my file sharing"
            | "what's my file sharing status"
            | "whats my file sharing status"
            | "what is my file sharing status"
            | "what's the file sharing"
            | "whats the file sharing"
            | "what is the file sharing"
            | "what's the file sharing status"
            | "whats the file sharing status"
            | "what is the file sharing status"
            | "what's my smb"
            | "whats my smb"
            | "what is my smb"
            | "what's my smb status"
            | "whats my smb status"
            | "what is my smb status"
            | "what's the smb status"
            | "whats the smb status"
            | "what is the smb status"
            | "how's file sharing"
            | "hows file sharing"
            | "how's the file sharing"
            | "hows the file sharing"
            // View/see/show me/open/list.
            | "view file sharing"
            | "view the file sharing"
            | "view file sharing status"
            | "view the file sharing status"
            | "view filesharing"
            | "view the filesharing"
            | "view file-sharing"
            | "view the file-sharing"
            | "view file share"
            | "view the file share"
            | "view smb"
            | "view the smb"
            | "view smb status"
            | "view the smb status"
            | "view smbd"
            | "view the smbd"
            | "see file sharing"
            | "see the file sharing"
            | "see file sharing status"
            | "see the file sharing status"
            | "see smb"
            | "see the smb"
            | "see smb status"
            | "see the smb status"
            | "open file sharing"
            | "open the file sharing"
            | "open file sharing status"
            | "open the file sharing status"
            | "open filesharing"
            | "open the filesharing"
            | "open smb"
            | "open the smb"
            | "open smb status"
            | "open the smb status"
            | "list file sharing"
            | "list the file sharing"
            | "list file sharing status"
            | "list the file sharing status"
            | "list smb"
            | "list the smb"
            | "list smb status"
            | "list the smb status"
            | "show file sharing"
            | "show the file sharing"
            | "show me file sharing"
            | "show me the file sharing"
            | "show file sharing status"
            | "show the file sharing status"
            | "show me file sharing status"
            | "show me the file sharing status"
            | "show smb"
            | "show the smb"
            | "show me smb"
            | "show me the smb"
            | "show smb status"
            | "show the smb status"
            | "show me smb status"
            | "show me the smb status"
    )
}

/// Zero-LLM File Sharing status from `launchctl` (macOS-only; empty elsewhere).
pub fn format_file_sharing_report() -> String {
    let v = crate::metrics::get_file_sharing_status();
    let v = v.trim();
    if v.is_empty() {
        return "**File Sharing** — _no File Sharing reading on this Mac right now._".to_string();
    }
    format!("**File Sharing** · `{v}`")
}

/// Printer Sharing — CUPS `_share_printers` from `cupsctl`.
/// "Why" / enable/disable stay with the agent. Slash + view/open NL (`/printer-sharing` · `/share-printers`).
/// Bare `printer` / `printers` / `cups` stay with the agent; `/file-sharing` · `/smb` stay File Sharing;
/// `/screen-sharing` · `/vnc` stay Screen Sharing; `/remote-login` · `/ssh` stay Remote Login.
pub fn looks_like_printer_sharing_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("password")
        || n.contains("screenshot")
        || n.contains("resolution")
        || n.contains("display")
        || n.contains("connect")
        || n.contains("tunnel")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("firewall")
        || n.contains("stealth")
        || n.contains("gatekeeper")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("file-vault")
        || n.contains("system integrity")
        || n.contains("time machine")
        || n.contains("timemachine")
        || n.contains("vpn")
        || n.contains("remote login")
        || n.contains("remotelogin")
        || n.contains("remote-login")
        || n.contains("screen sharing")
        || n.contains("screensharing")
        || n.contains("screen-sharing")
        || n.contains("screen share")
        || n.contains("file sharing")
        || n.contains("filesharing")
        || n.contains("file-sharing")
        || n.contains("file share")
        || n.contains("remote management")
        || n.contains("remote desktop")
        || n.contains("queue")
        || n.contains("job")
        || n.contains("spool")
        || n == "/firewall"
        || n == "/fw"
        || n == "/afw"
        || n == "/stealth"
        || n == "/gatekeeper"
        || n == "/sip"
        || n == "/filevault"
        || n == "/file-vault"
        || n == "/timemachine"
        || n == "/tm"
        || n == "/vpn"
        || n == "/remote-login"
        || n == "/remotelogin"
        || n == "/ssh"
        || n == "/sshd"
        || n == "/ssh-status"
        || n == "/sshd-status"
        || n == "/screen-sharing"
        || n == "/screensharing"
        || n == "/screen-share"
        || n == "/vnc"
        || n == "/vnc-status"
        || n == "/mac-vnc"
        || n == "/file-sharing"
        || n == "/filesharing"
        || n == "/file-share"
        || n == "/smb"
        || n == "/smb-status"
        || n == "/smbd"
        || n == "/mac-smb"
        || n == "/remote-management"
        || n == "/remotemanagement"
        || n == "/remote-mgmt"
        || n == "/ard"
        || n == "/ard-status"
        || n == "/ard-state"
        || n == "/ard-agent"
        || n == "/apple-remote-desktop"
        || n == "/mac-ard"
        || n == "/content-caching"
        || n == "/contentcaching"
        || n == "/content-cache"
        || n == "/contentcache"
        || n == "/asset-cache"
        || n == "/assetcache"
        || n == "/asset-caching"
        || n == "/mac-content-caching"
        || n == "/ip"
        || n == "/wifi"
        || n == "/tasks"
        || n == "/screen"
        || n == "/screens"
        || n == "/display"
        || n == "/displays"
        // Bare short / ambiguous stay with the agent.
        || n == "printer"
        || n == "the printer"
        || n == "printers"
        || n == "the printers"
        || n == "cups"
        || n == "the cups"
        || n == "print"
        || n == "the print"
        || n == "printer sharing"
        || n == "the printer sharing"
        || n == "printersharing"
        || n == "the printersharing"
        || n == "printer-sharing"
        || n == "the printer-sharing"
        || n == "printer share"
        || n == "the printer share"
        || n == "share printers"
        || n == "the share printers"
        || n == "ard"
        || n == "the ard"
        || n == "remote management"
        || n == "the remote management"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/printer-sharing"
            | "/printersharing"
            | "/printer-sharing-status"
            | "/printersharing-status"
            | "/printer-share"
            | "/printershare"
            | "/printer-share-status"
            | "/share-printers"
            | "/shareprinters"
            | "/share-printers-status"
            | "/cups-share"
            | "/cups-sharing"
            | "/cups-share-status"
            | "/cups-sharing-status"
            | "/mac-printer-sharing"
            | "/sharing-printers"
            | "printer sharing status"
            | "the printer sharing status"
            | "printersharing status"
            | "the printersharing status"
            | "printer-sharing status"
            | "the printer-sharing status"
            | "printer share status"
            | "the printer share status"
            | "share printers status"
            | "the share printers status"
            | "cups share status"
            | "the cups share status"
            | "cups sharing status"
            | "the cups sharing status"
            | "my printer sharing"
            | "my printer sharing status"
            | "my printersharing"
            | "my cups share"
            | "my cups sharing"
            | "is printer sharing on"
            | "is printer sharing off"
            | "is printer sharing enabled"
            | "is printer sharing disabled"
            | "is printersharing on"
            | "is printersharing enabled"
            | "is printer-sharing on"
            | "is printer-sharing enabled"
            | "is share printers on"
            | "is share printers enabled"
            | "is cups share on"
            | "is cups share enabled"
            | "is cups sharing on"
            | "is cups sharing enabled"
            | "what's my printer sharing"
            | "whats my printer sharing"
            | "what is my printer sharing"
            | "what's my printer sharing status"
            | "whats my printer sharing status"
            | "what is my printer sharing status"
            | "what's the printer sharing"
            | "whats the printer sharing"
            | "what is the printer sharing"
            | "what's the printer sharing status"
            | "whats the printer sharing status"
            | "what is the printer sharing status"
            | "how's printer sharing"
            | "hows printer sharing"
            | "how's the printer sharing"
            | "hows the printer sharing"
            // View/see/show me/open/list.
            | "view printer sharing"
            | "view the printer sharing"
            | "view printer sharing status"
            | "view the printer sharing status"
            | "view printersharing"
            | "view the printersharing"
            | "view printer-sharing"
            | "view the printer-sharing"
            | "view printer share"
            | "view the printer share"
            | "view share printers"
            | "view the share printers"
            | "view cups share"
            | "view the cups share"
            | "view cups sharing"
            | "view the cups sharing"
            | "view cups share status"
            | "view the cups share status"
            | "see printer sharing"
            | "see the printer sharing"
            | "see printer sharing status"
            | "see the printer sharing status"
            | "see share printers"
            | "see the share printers"
            | "see cups share"
            | "see the cups share"
            | "see cups sharing"
            | "see the cups sharing"
            | "open printer sharing"
            | "open the printer sharing"
            | "open printer sharing status"
            | "open the printer sharing status"
            | "open printersharing"
            | "open the printersharing"
            | "open share printers"
            | "open the share printers"
            | "open cups share"
            | "open the cups share"
            | "open cups sharing"
            | "open the cups sharing"
            | "open cups share status"
            | "open the cups share status"
            | "list printer sharing"
            | "list the printer sharing"
            | "list printer sharing status"
            | "list the printer sharing status"
            | "list share printers"
            | "list the share printers"
            | "list cups share"
            | "list the cups share"
            | "list cups sharing"
            | "list the cups sharing"
            | "show printer sharing"
            | "show the printer sharing"
            | "show me printer sharing"
            | "show me the printer sharing"
            | "show printer sharing status"
            | "show the printer sharing status"
            | "show me printer sharing status"
            | "show me the printer sharing status"
            | "show share printers"
            | "show the share printers"
            | "show me share printers"
            | "show me the share printers"
            | "show cups share"
            | "show the cups share"
            | "show me cups share"
            | "show me the cups share"
            | "show cups sharing"
            | "show the cups sharing"
            | "show me cups sharing"
            | "show me the cups sharing"
    )
}

/// Zero-LLM Printer Sharing status from `cupsctl` (macOS-only; empty elsewhere).
pub fn format_printer_sharing_report() -> String {
    let v = crate::metrics::get_printer_sharing_status();
    let v = v.trim();
    if v.is_empty() {
        return "**Printer Sharing** — _no Printer Sharing reading on this Mac right now._".to_string();
    }
    format!("**Printer Sharing** · `{v}`")
}

/// Remote Management (ARD) — launchd marker + ARDAgent.
/// "Why" / enable/disable stay with the agent. Slash + view/open NL (`/remote-management` · `/ard`).
/// Bare `ard` / `remote management` / `remote desktop` stay with the agent; `/screen-sharing` · `/vnc`
/// stay Screen Sharing; `/remote-login` · `/ssh` stay Remote Login; `/file-sharing` · `/smb` stay File Sharing;
/// `/printer-sharing` stays Printer Sharing; `/content-caching` · `/asset-cache` stay Content Caching.
pub fn looks_like_remote_management_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("password")
        || n.contains("screenshot")
        || n.contains("resolution")
        || n.contains("display")
        || n.contains("connect")
        || n.contains("tunnel")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("firewall")
        || n.contains("stealth")
        || n.contains("gatekeeper")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("file-vault")
        || n.contains("system integrity")
        || n.contains("time machine")
        || n.contains("timemachine")
        || n.contains("vpn")
        || n.contains("remote login")
        || n.contains("remotelogin")
        || n.contains("remote-login")
        || n.contains("screen sharing")
        || n.contains("screensharing")
        || n.contains("screen-sharing")
        || n.contains("screen share")
        || n.contains("file sharing")
        || n.contains("filesharing")
        || n.contains("file-sharing")
        || n.contains("file share")
        || n.contains("printer sharing")
        || n.contains("printersharing")
        || n.contains("printer-sharing")
        || n.contains("share printers")
        || n.contains("cups share")
        || n.contains("cups sharing")
        || n.contains("content caching")
        || n.contains("content-caching")
        || n.contains("content cache")
        || n.contains("asset cache")
        || n.contains("asset-cache")
        || n.contains("assetcache")
        || n.contains("remote apple events")
        || n.contains("remote-apple-events")
        || n.contains("remoteappleevents")
        || n.contains("apple events")
        || n.contains("apple-events")
        || n.contains("eppc")
        || n == "/firewall"
        || n == "/fw"
        || n == "/afw"
        || n == "/stealth"
        || n == "/gatekeeper"
        || n == "/sip"
        || n == "/filevault"
        || n == "/file-vault"
        || n == "/timemachine"
        || n == "/tm"
        || n == "/vpn"
        || n == "/remote-login"
        || n == "/remotelogin"
        || n == "/ssh"
        || n == "/sshd"
        || n == "/ssh-status"
        || n == "/sshd-status"
        || n == "/screen-sharing"
        || n == "/screensharing"
        || n == "/screen-share"
        || n == "/vnc"
        || n == "/vnc-status"
        || n == "/mac-vnc"
        || n == "/file-sharing"
        || n == "/filesharing"
        || n == "/file-share"
        || n == "/smb"
        || n == "/smb-status"
        || n == "/smbd"
        || n == "/mac-smb"
        || n == "/printer-sharing"
        || n == "/printersharing"
        || n == "/printer-share"
        || n == "/share-printers"
        || n == "/cups-share"
        || n == "/cups-sharing"
        || n == "/content-caching"
        || n == "/contentcaching"
        || n == "/content-cache"
        || n == "/contentcache"
        || n == "/asset-cache"
        || n == "/assetcache"
        || n == "/asset-caching"
        || n == "/mac-content-caching"
        || n == "/remote-apple-events"
        || n == "/remoteappleevents"
        || n == "/apple-events"
        || n == "/rae"
        || n == "/eppc"
        || n == "/aeserver"
        || n == "/ip"
        || n == "/wifi"
        || n == "/tasks"
        || n == "/screen"
        || n == "/screens"
        || n == "/display"
        || n == "/displays"
        // Bare short / ambiguous stay with the agent.
        || n == "ard"
        || n == "the ard"
        || n == "remote management"
        || n == "the remote management"
        || n == "remotemanagement"
        || n == "the remotemanagement"
        || n == "remote-management"
        || n == "the remote-management"
        || n == "remote desktop"
        || n == "the remote desktop"
        || n == "apple remote desktop"
        || n == "the apple remote desktop"
        || n == "ard agent"
        || n == "the ard agent"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/remote-management"
            | "/remotemanagement"
            | "/remote-management-status"
            | "/remotemanagement-status"
            | "/remote-mgmt"
            | "/remotemgmt"
            | "/remote-mgmt-status"
            | "/ard"
            | "/ard-status"
            | "/ard-state"
            | "/ard-agent"
            | "/ardagent"
            | "/apple-remote-desktop"
            | "/apple-remote-desktop-status"
            | "/mac-ard"
            | "/mac-remote-management"
            | "remote management status"
            | "the remote management status"
            | "remotemanagement status"
            | "the remotemanagement status"
            | "remote-management status"
            | "the remote-management status"
            | "remote mgmt status"
            | "the remote mgmt status"
            | "ard status"
            | "the ard status"
            | "ard state"
            | "the ard state"
            | "apple remote desktop status"
            | "the apple remote desktop status"
            | "my remote management"
            | "my remote management status"
            | "my remotemanagement"
            | "my ard"
            | "my ard status"
            | "my apple remote desktop"
            | "is remote management on"
            | "is remote management off"
            | "is remote management enabled"
            | "is remote management disabled"
            | "is remotemanagement on"
            | "is remotemanagement enabled"
            | "is remote-management on"
            | "is remote-management enabled"
            | "is ard on"
            | "is ard off"
            | "is ard enabled"
            | "is ard disabled"
            | "is apple remote desktop on"
            | "is apple remote desktop enabled"
            | "what's my remote management"
            | "whats my remote management"
            | "what is my remote management"
            | "what's my remote management status"
            | "whats my remote management status"
            | "what is my remote management status"
            | "what's my ard"
            | "whats my ard"
            | "what is my ard"
            | "what's my ard status"
            | "whats my ard status"
            | "what is my ard status"
            | "what's the remote management"
            | "whats the remote management"
            | "what is the remote management"
            | "what's the remote management status"
            | "whats the remote management status"
            | "what is the remote management status"
            | "what's the ard status"
            | "whats the ard status"
            | "what is the ard status"
            | "how's remote management"
            | "hows remote management"
            | "how's the remote management"
            | "hows the remote management"
            | "how's ard"
            | "hows ard"
            // View/see/show me/open/list.
            | "view remote management"
            | "view the remote management"
            | "view remote management status"
            | "view the remote management status"
            | "view remotemanagement"
            | "view the remotemanagement"
            | "view remote-management"
            | "view the remote-management"
            | "view remote mgmt"
            | "view the remote mgmt"
            | "view ard"
            | "view the ard"
            | "view ard status"
            | "view the ard status"
            | "view ard state"
            | "view the ard state"
            | "view apple remote desktop"
            | "view the apple remote desktop"
            | "view apple remote desktop status"
            | "view the apple remote desktop status"
            | "see remote management"
            | "see the remote management"
            | "see remote management status"
            | "see the remote management status"
            | "see ard"
            | "see the ard"
            | "see ard status"
            | "see the ard status"
            | "see apple remote desktop"
            | "see the apple remote desktop"
            | "open remote management"
            | "open the remote management"
            | "open remote management status"
            | "open the remote management status"
            | "open remotemanagement"
            | "open the remotemanagement"
            | "open ard"
            | "open the ard"
            | "open ard status"
            | "open the ard status"
            | "open apple remote desktop"
            | "open the apple remote desktop"
            | "list remote management"
            | "list the remote management"
            | "list remote management status"
            | "list the remote management status"
            | "list ard"
            | "list the ard"
            | "list ard status"
            | "list the ard status"
            | "list apple remote desktop"
            | "list the apple remote desktop"
            | "show remote management"
            | "show the remote management"
            | "show me remote management"
            | "show me the remote management"
            | "show remote management status"
            | "show the remote management status"
            | "show me remote management status"
            | "show me the remote management status"
            | "show ard"
            | "show the ard"
            | "show me ard"
            | "show me the ard"
            | "show ard status"
            | "show the ard status"
            | "show me ard status"
            | "show me the ard status"
            | "show apple remote desktop"
            | "show the apple remote desktop"
            | "show me apple remote desktop"
            | "show me the apple remote desktop"
    )
}

/// Zero-LLM Remote Management status from launchd marker + ARDAgent (macOS-only; empty elsewhere).
pub fn format_remote_management_report() -> String {
    let v = crate::metrics::get_remote_management_status();
    let v = v.trim();
    if v.is_empty() {
        return "**Remote Management** — _no Remote Management reading on this Mac right now._".to_string();
    }
    format!("**Remote Management** · `{v}`")
}

/// Content Caching (AssetCache) — AssetCacheManagerUtil / prefs + AssetCache process.
/// "Why" / enable/disable stay with the agent. Slash + view/open NL (`/content-caching` · `/asset-cache`).
/// Bare `caching` / `content caching` / `asset cache` stay with the agent; `/remote-management` · `/ard`
/// stay Remote Management; `/file-sharing` · `/smb` stay File Sharing; `/printer-sharing` stays Printer Sharing;
/// `/screen-sharing` · `/vnc` stay Screen Sharing; `/remote-login` · `/ssh` stay Remote Login.
pub fn looks_like_content_caching_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("password")
        || n.contains("screenshot")
        || n.contains("resolution")
        || n.contains("display")
        || n.contains("connect")
        || n.contains("tunnel")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        // Word-boundary "set " only — bare `contains("set ")` false-positives on "asset ".
        || n.starts_with("set ")
        || n.contains(" set ")
        || n.contains("reset")
        || n.contains("firewall")
        || n.contains("stealth")
        || n.contains("gatekeeper")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("file-vault")
        || n.contains("system integrity")
        || n.contains("time machine")
        || n.contains("timemachine")
        || n.contains("vpn")
        || n.contains("remote login")
        || n.contains("remotelogin")
        || n.contains("remote-login")
        || n.contains("screen sharing")
        || n.contains("screensharing")
        || n.contains("screen-sharing")
        || n.contains("screen share")
        || n.contains("file sharing")
        || n.contains("filesharing")
        || n.contains("file-sharing")
        || n.contains("file share")
        || n.contains("printer sharing")
        || n.contains("printersharing")
        || n.contains("printer-sharing")
        || n.contains("share printers")
        || n.contains("cups share")
        || n.contains("cups sharing")
        || n.contains("remote management")
        || n.contains("remotemanagement")
        || n.contains("remote-management")
        || n.contains("remote desktop")
        || n.contains("apple remote desktop")
        || n == "/firewall"
        || n == "/fw"
        || n == "/afw"
        || n == "/stealth"
        || n == "/gatekeeper"
        || n == "/sip"
        || n == "/filevault"
        || n == "/file-vault"
        || n == "/timemachine"
        || n == "/tm"
        || n == "/vpn"
        || n == "/remote-login"
        || n == "/remotelogin"
        || n == "/ssh"
        || n == "/sshd"
        || n == "/ssh-status"
        || n == "/sshd-status"
        || n == "/screen-sharing"
        || n == "/screensharing"
        || n == "/screen-share"
        || n == "/vnc"
        || n == "/vnc-status"
        || n == "/mac-vnc"
        || n == "/file-sharing"
        || n == "/filesharing"
        || n == "/file-share"
        || n == "/smb"
        || n == "/smb-status"
        || n == "/smbd"
        || n == "/mac-smb"
        || n == "/printer-sharing"
        || n == "/printersharing"
        || n == "/printer-share"
        || n == "/share-printers"
        || n == "/cups-share"
        || n == "/cups-sharing"
        || n == "/remote-management"
        || n == "/remotemanagement"
        || n == "/remote-mgmt"
        || n == "/ard"
        || n == "/ard-status"
        || n == "/ard-state"
        || n == "/ard-agent"
        || n == "/apple-remote-desktop"
        || n == "/mac-ard"
        || n.contains("internet sharing")
        || n.contains("internetsharing")
        || n.contains("internet-sharing")
        || n.contains("network sharing")
        || n.contains("networksharing")
        || n.contains("network-sharing")
        || n.contains("bluetooth sharing")
        || n.contains("bluetoothsharing")
        || n.contains("bluetooth-sharing")
        || n.contains("bluetooth share")
        || n.contains("bt sharing")
        || n.contains("bt-sharing")
        || n.contains("file exchange")
        || n.contains("file-exchange")
        || n.contains("obex")
        || n == "/internet-sharing"
        || n == "/internetsharing"
        || n == "/internet-share"
        || n == "/network-sharing"
        || n == "/ics"
        || n == "/ics-status"
        || n == "/nat-sharing"
        || n == "/mac-internet-sharing"
        || n == "/bluetooth-sharing"
        || n == "/bluetoothsharing"
        || n == "/bluetooth-share"
        || n == "/bt-sharing"
        || n == "/bt-share"
        || n == "/bts"
        || n == "/bts-status"
        || n == "/obex"
        || n == "/obex-sharing"
        || n == "/file-exchange"
        || n == "/bluetooth-file-exchange"
        || n == "/mac-bluetooth-sharing"
        || n == "/ip"
        || n == "/wifi"
        || n == "/tasks"
        || n == "/screen"
        || n == "/screens"
        || n == "/display"
        || n == "/displays"
        || n == "/cache"
        || n == "/caches"
        // Bare short / ambiguous stay with the agent.
        || n == "caching"
        || n == "the caching"
        || n == "content caching"
        || n == "the content caching"
        || n == "contentcaching"
        || n == "the contentcaching"
        || n == "content-caching"
        || n == "the content-caching"
        || n == "content cache"
        || n == "the content cache"
        || n == "asset cache"
        || n == "the asset cache"
        || n == "assetcache"
        || n == "the assetcache"
        || n == "asset-cache"
        || n == "the asset-cache"
        || n == "asset caching"
        || n == "the asset caching"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/content-caching"
            | "/contentcaching"
            | "/content-caching-status"
            | "/contentcaching-status"
            | "/content-cache"
            | "/contentcache"
            | "/content-cache-status"
            | "/contentcache-status"
            | "/asset-cache"
            | "/assetcache"
            | "/asset-cache-status"
            | "/assetcache-status"
            | "/asset-caching"
            | "/assetcaching"
            | "/mac-content-caching"
            | "/mac-asset-cache"
            | "content caching status"
            | "the content caching status"
            | "contentcaching status"
            | "the contentcaching status"
            | "content-caching status"
            | "the content-caching status"
            | "content cache status"
            | "the content cache status"
            | "asset cache status"
            | "the asset cache status"
            | "assetcache status"
            | "the assetcache status"
            | "asset-cache status"
            | "the asset-cache status"
            | "asset caching status"
            | "the asset caching status"
            | "my content caching"
            | "my content caching status"
            | "my contentcaching"
            | "my content cache"
            | "my asset cache"
            | "my asset cache status"
            | "is content caching on"
            | "is content caching off"
            | "is content caching enabled"
            | "is content caching disabled"
            | "is contentcaching on"
            | "is contentcaching enabled"
            | "is content-caching on"
            | "is content-caching enabled"
            | "is content cache on"
            | "is content cache enabled"
            | "is asset cache on"
            | "is asset cache off"
            | "is asset cache enabled"
            | "is asset cache disabled"
            | "is assetcache on"
            | "is assetcache enabled"
            | "is asset-cache on"
            | "is asset-cache enabled"
            | "is asset caching on"
            | "is asset caching enabled"
            | "what's my content caching"
            | "whats my content caching"
            | "what is my content caching"
            | "what's my content caching status"
            | "whats my content caching status"
            | "what is my content caching status"
            | "what's my asset cache"
            | "whats my asset cache"
            | "what is my asset cache"
            | "what's my asset cache status"
            | "whats my asset cache status"
            | "what is my asset cache status"
            | "what's the content caching"
            | "whats the content caching"
            | "what is the content caching"
            | "what's the content caching status"
            | "whats the content caching status"
            | "what is the content caching status"
            | "what's the asset cache"
            | "whats the asset cache"
            | "what is the asset cache"
            | "what's the asset cache status"
            | "whats the asset cache status"
            | "what is the asset cache status"
            | "how's content caching"
            | "hows content caching"
            | "how's the content caching"
            | "hows the content caching"
            | "how's asset cache"
            | "hows asset cache"
            // View/see/show me/open/list.
            | "view content caching"
            | "view the content caching"
            | "view content caching status"
            | "view the content caching status"
            | "view contentcaching"
            | "view the contentcaching"
            | "view content-caching"
            | "view the content-caching"
            | "view content cache"
            | "view the content cache"
            | "view asset cache"
            | "view the asset cache"
            | "view asset cache status"
            | "view the asset cache status"
            | "view assetcache"
            | "view the assetcache"
            | "view asset-cache"
            | "view the asset-cache"
            | "view asset caching"
            | "view the asset caching"
            | "see content caching"
            | "see the content caching"
            | "see content caching status"
            | "see the content caching status"
            | "see asset cache"
            | "see the asset cache"
            | "see asset cache status"
            | "see the asset cache status"
            | "open content caching"
            | "open the content caching"
            | "open content caching status"
            | "open the content caching status"
            | "open contentcaching"
            | "open the contentcaching"
            | "open asset cache"
            | "open the asset cache"
            | "open asset cache status"
            | "open the asset cache status"
            | "open assetcache"
            | "open the assetcache"
            | "list content caching"
            | "list the content caching"
            | "list content caching status"
            | "list the content caching status"
            | "list asset cache"
            | "list the asset cache"
            | "list asset cache status"
            | "list the asset cache status"
            | "show content caching"
            | "show the content caching"
            | "show me content caching"
            | "show me the content caching"
            | "show content caching status"
            | "show the content caching status"
            | "show me content caching status"
            | "show me the content caching status"
            | "show asset cache"
            | "show the asset cache"
            | "show me asset cache"
            | "show me the asset cache"
            | "show asset cache status"
            | "show the asset cache status"
            | "show me asset cache status"
            | "show me the asset cache status"
    )
}

/// Zero-LLM Content Caching status from AssetCacheManagerUtil / prefs + AssetCache (macOS-only; empty elsewhere).
pub fn format_content_caching_report() -> String {
    let v = crate::metrics::get_content_caching_status();
    let v = v.trim();
    if v.is_empty() {
        return "**Content Caching** — _no Content Caching reading on this Mac right now._".to_string();
    }
    format!("**Content Caching** · `{v}`")
}

/// Internet Sharing / ICS / NAT — plist `:NAT:Enabled` + `com.apple.NetworkSharing` daemon.
/// "Why" / enable/disable stay with the agent. Slash + view/open NL (`/internet-sharing` · `/ics`).
/// Bare `internet` / `nat` / `ics` / `internet sharing` stay with the agent; `/vpn` stays VPN;
/// `/content-caching` · `/asset-cache` stay Content Caching; `/file-sharing` · `/smb` stay File Sharing;
/// `/remote-management` · `/ard` stay Remote Management; `/ip` stays LAN.
pub fn looks_like_internet_sharing_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("password")
        || n.contains("screenshot")
        || n.contains("resolution")
        || n.contains("display")
        || n.contains("connect")
        || n.contains("tunnel")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("firewall")
        || n.contains("stealth")
        || n.contains("gatekeeper")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("file-vault")
        || n.contains("system integrity")
        || n.contains("time machine")
        || n.contains("timemachine")
        || n.contains("vpn")
        || n.contains("remote login")
        || n.contains("remotelogin")
        || n.contains("remote-login")
        || n.contains("screen sharing")
        || n.contains("screensharing")
        || n.contains("screen-sharing")
        || n.contains("screen share")
        || n.contains("file sharing")
        || n.contains("filesharing")
        || n.contains("file-sharing")
        || n.contains("file share")
        || n.contains("printer sharing")
        || n.contains("printersharing")
        || n.contains("printer-sharing")
        || n.contains("share printers")
        || n.contains("cups share")
        || n.contains("cups sharing")
        || n.contains("remote management")
        || n.contains("remotemanagement")
        || n.contains("remote-management")
        || n.contains("remote desktop")
        || n.contains("apple remote desktop")
        || n.contains("content caching")
        || n.contains("contentcaching")
        || n.contains("content-caching")
        || n.contains("content cache")
        || n.contains("asset cache")
        || n.contains("assetcache")
        || n.contains("asset-cache")
        || n.contains("caching server")
        || n.contains("bluetooth")
        || n == "/firewall"
        || n == "/fw"
        || n == "/afw"
        || n == "/stealth"
        || n == "/gatekeeper"
        || n == "/sip"
        || n == "/filevault"
        || n == "/file-vault"
        || n == "/timemachine"
        || n == "/tm"
        || n == "/vpn"
        || n == "/remote-login"
        || n == "/remotelogin"
        || n == "/ssh"
        || n == "/sshd"
        || n == "/ssh-status"
        || n == "/sshd-status"
        || n == "/screen-sharing"
        || n == "/screensharing"
        || n == "/screen-share"
        || n == "/vnc"
        || n == "/vnc-status"
        || n == "/mac-vnc"
        || n == "/file-sharing"
        || n == "/filesharing"
        || n == "/file-share"
        || n == "/smb"
        || n == "/smb-status"
        || n == "/smbd"
        || n == "/mac-smb"
        || n == "/printer-sharing"
        || n == "/printersharing"
        || n == "/printer-share"
        || n == "/share-printers"
        || n == "/cups-share"
        || n == "/cups-sharing"
        || n == "/remote-management"
        || n == "/remotemanagement"
        || n == "/remote-mgmt"
        || n == "/ard"
        || n == "/ard-status"
        || n == "/ard-state"
        || n == "/ard-agent"
        || n == "/apple-remote-desktop"
        || n == "/mac-ard"
        || n == "/content-caching"
        || n == "/contentcaching"
        || n == "/content-cache"
        || n == "/asset-cache"
        || n == "/assetcache"
        || n == "/caching-server"
        || n == "/mac-content-caching"
        || n == "/ip"
        || n == "/wifi"
        || n == "/gateway"
        || n == "/tasks"
        || n == "/screen"
        || n == "/screens"
        || n == "/display"
        || n == "/displays"
        // Bare short / ambiguous stay with the agent.
        || n == "internet"
        || n == "the internet"
        || n == "nat"
        || n == "the nat"
        || n == "ics"
        || n == "the ics"
        || n == "internet sharing"
        || n == "the internet sharing"
        || n == "internetsharing"
        || n == "the internetsharing"
        || n == "internet-sharing"
        || n == "the internet-sharing"
        || n == "network sharing"
        || n == "the network sharing"
        || n == "networksharing"
        || n == "the networksharing"
        || n == "network-sharing"
        || n == "the network-sharing"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/internet-sharing"
            | "/internetsharing"
            | "/internet-sharing-status"
            | "/internetsharing-status"
            | "/internet-share"
            | "/internetshare"
            | "/network-sharing"
            | "/networksharing"
            | "/network-sharing-status"
            | "/ics"
            | "/ics-status"
            | "/ics-state"
            | "/nat-sharing"
            | "/nat-share"
            | "/mac-internet-sharing"
            | "/mac-ics"
            | "internet sharing status"
            | "the internet sharing status"
            | "internetsharing status"
            | "the internetsharing status"
            | "internet-sharing status"
            | "the internet-sharing status"
            | "internet share status"
            | "the internet share status"
            | "network sharing status"
            | "the network sharing status"
            | "network-sharing status"
            | "the network-sharing status"
            | "ics status"
            | "the ics status"
            | "ics state"
            | "the ics state"
            | "nat sharing status"
            | "the nat sharing status"
            | "my internet sharing"
            | "my internet sharing status"
            | "my internetsharing"
            | "my ics"
            | "my ics status"
            | "is internet sharing on"
            | "is internet sharing off"
            | "is internet sharing enabled"
            | "is internet sharing disabled"
            | "is internetsharing on"
            | "is internetsharing enabled"
            | "is internet-sharing on"
            | "is internet-sharing enabled"
            | "is network sharing on"
            | "is network sharing enabled"
            | "is ics on"
            | "is ics off"
            | "is ics enabled"
            | "is ics disabled"
            | "what's my internet sharing"
            | "whats my internet sharing"
            | "what is my internet sharing"
            | "what's my internet sharing status"
            | "whats my internet sharing status"
            | "what is my internet sharing status"
            | "what's my ics"
            | "whats my ics"
            | "what is my ics"
            | "what's my ics status"
            | "whats my ics status"
            | "what is my ics status"
            | "what's the internet sharing"
            | "whats the internet sharing"
            | "what is the internet sharing"
            | "what's the internet sharing status"
            | "whats the internet sharing status"
            | "what is the internet sharing status"
            | "what's the ics status"
            | "whats the ics status"
            | "what is the ics status"
            | "how's internet sharing"
            | "hows internet sharing"
            | "how's the internet sharing"
            | "hows the internet sharing"
            | "how's ics"
            | "hows ics"
            // View/see/show me/open/list.
            | "view internet sharing"
            | "view the internet sharing"
            | "view internet sharing status"
            | "view the internet sharing status"
            | "view internetsharing"
            | "view the internetsharing"
            | "view internet-sharing"
            | "view the internet-sharing"
            | "view internet share"
            | "view the internet share"
            | "view network sharing"
            | "view the network sharing"
            | "view network-sharing"
            | "view the network-sharing"
            | "view ics"
            | "view the ics"
            | "view ics status"
            | "view the ics status"
            | "view ics state"
            | "view the ics state"
            | "view nat sharing"
            | "view the nat sharing"
            | "see internet sharing"
            | "see the internet sharing"
            | "see internet sharing status"
            | "see the internet sharing status"
            | "see network sharing"
            | "see the network sharing"
            | "see ics"
            | "see the ics"
            | "see ics status"
            | "see the ics status"
            | "open internet sharing"
            | "open the internet sharing"
            | "open internet sharing status"
            | "open the internet sharing status"
            | "open internetsharing"
            | "open the internetsharing"
            | "open network sharing"
            | "open the network sharing"
            | "open ics"
            | "open the ics"
            | "open ics status"
            | "open the ics status"
            | "list internet sharing"
            | "list the internet sharing"
            | "list internet sharing status"
            | "list the internet sharing status"
            | "list network sharing"
            | "list the network sharing"
            | "list ics"
            | "list the ics"
            | "list ics status"
            | "list the ics status"
            | "show internet sharing"
            | "show the internet sharing"
            | "show me internet sharing"
            | "show me the internet sharing"
            | "show internet sharing status"
            | "show the internet sharing status"
            | "show me internet sharing status"
            | "show me the internet sharing status"
            | "show network sharing"
            | "show the network sharing"
            | "show me network sharing"
            | "show me the network sharing"
            | "show ics"
            | "show the ics"
            | "show me ics"
            | "show me the ics"
            | "show ics status"
            | "show the ics status"
            | "show me ics status"
            | "show me the ics status"
    )
}

/// Zero-LLM Internet Sharing status from NAT plist + NetworkSharing daemon (macOS-only; empty elsewhere).
pub fn format_internet_sharing_report() -> String {
    let v = crate::metrics::get_internet_sharing_status();
    let v = v.trim();
    if v.is_empty() {
        return "**Internet Sharing** — _no Internet Sharing reading on this Mac right now._".to_string();
    }
    format!("**Internet Sharing** · `{v}`")
}

/// Bluetooth Sharing / OBEX file exchange — Bluetooth prefs + BluetoothFileExchange.
/// "Why" / enable/disable stay with the agent. Slash + view/open NL (`/bluetooth-sharing` · `/bt-sharing`).
/// Bare `bluetooth` / `bluetooth sharing` / `obex` / `bts` stay with the agent; `/internet-sharing` · `/ics`
/// stay Internet Sharing; `/file-sharing` · `/smb` stay File Sharing; `/wifi` stays SSID;
/// `/content-caching` · `/asset-cache` stay Content Caching.
pub fn looks_like_bluetooth_sharing_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("password")
        || n.contains("screenshot")
        || n.contains("resolution")
        || n.contains("display")
        || n.contains("connect")
        || n.contains("tunnel")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        // Word-boundary "change" only — bare `contains("change")` false-positives on "exchange".
        || n == "change"
        || n.starts_with("change ")
        || n.contains(" change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("firewall")
        || n.contains("stealth")
        || n.contains("gatekeeper")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("file-vault")
        || n.contains("system integrity")
        || n.contains("time machine")
        || n.contains("timemachine")
        || n.contains("vpn")
        || n.contains("remote login")
        || n.contains("remotelogin")
        || n.contains("remote-login")
        || n.contains("screen sharing")
        || n.contains("screensharing")
        || n.contains("screen-sharing")
        || n.contains("screen share")
        || n.contains("file sharing")
        || n.contains("filesharing")
        || n.contains("file-sharing")
        || n.contains("file share")
        || n.contains("printer sharing")
        || n.contains("printersharing")
        || n.contains("printer-sharing")
        || n.contains("share printers")
        || n.contains("cups share")
        || n.contains("cups sharing")
        || n.contains("remote management")
        || n.contains("remotemanagement")
        || n.contains("remote-management")
        || n.contains("remote desktop")
        || n.contains("apple remote desktop")
        || n.contains("content caching")
        || n.contains("contentcaching")
        || n.contains("content-caching")
        || n.contains("content cache")
        || n.contains("asset cache")
        || n.contains("assetcache")
        || n.contains("asset-cache")
        || n.contains("caching server")
        || n.contains("internet sharing")
        || n.contains("internetsharing")
        || n.contains("internet-sharing")
        || n.contains("network sharing")
        || n.contains("networksharing")
        || n.contains("network-sharing")
        || n.contains("media sharing")
        || n.contains("mediasharing")
        || n.contains("media-sharing")
        || n.contains("home sharing")
        || n.contains("homesharing")
        || n.contains("home-sharing")
        || n.contains("itunes sharing")
        || n == "/media-sharing"
        || n == "/mediasharing"
        || n == "/home-sharing"
        || n == "/homesharing"
        || n == "/itunes-sharing"
        || n.contains("wifi")
        || n.contains("wi-fi")
        || n.contains("ssid")
        || n == "/firewall"
        || n == "/fw"
        || n == "/afw"
        || n == "/stealth"
        || n == "/gatekeeper"
        || n == "/sip"
        || n == "/filevault"
        || n == "/file-vault"
        || n == "/timemachine"
        || n == "/tm"
        || n == "/vpn"
        || n == "/remote-login"
        || n == "/remotelogin"
        || n == "/ssh"
        || n == "/sshd"
        || n == "/ssh-status"
        || n == "/sshd-status"
        || n == "/screen-sharing"
        || n == "/screensharing"
        || n == "/screen-share"
        || n == "/vnc"
        || n == "/vnc-status"
        || n == "/mac-vnc"
        || n == "/file-sharing"
        || n == "/filesharing"
        || n == "/file-share"
        || n == "/smb"
        || n == "/smb-status"
        || n == "/smbd"
        || n == "/mac-smb"
        || n == "/printer-sharing"
        || n == "/printersharing"
        || n == "/printer-share"
        || n == "/share-printers"
        || n == "/cups-share"
        || n == "/cups-sharing"
        || n == "/remote-management"
        || n == "/remotemanagement"
        || n == "/remote-mgmt"
        || n == "/ard"
        || n == "/ard-status"
        || n == "/ard-state"
        || n == "/ard-agent"
        || n == "/apple-remote-desktop"
        || n == "/mac-ard"
        || n == "/content-caching"
        || n == "/contentcaching"
        || n == "/content-cache"
        || n == "/asset-cache"
        || n == "/assetcache"
        || n == "/caching-server"
        || n == "/mac-content-caching"
        || n == "/internet-sharing"
        || n == "/internetsharing"
        || n == "/internet-share"
        || n == "/network-sharing"
        || n == "/ics"
        || n == "/ics-status"
        || n == "/nat-sharing"
        || n == "/mac-internet-sharing"
        || n == "/mac-ics"
        || n == "/ip"
        || n == "/wifi"
        || n == "/wi-fi"
        || n == "/ssid"
        || n == "/gateway"
        || n == "/tasks"
        || n == "/screen"
        || n == "/screens"
        || n == "/display"
        || n == "/displays"
        // Bare short / ambiguous stay with the agent.
        || n == "bluetooth"
        || n == "the bluetooth"
        || n == "bluetooth sharing"
        || n == "the bluetooth sharing"
        || n == "bluetoothsharing"
        || n == "the bluetoothsharing"
        || n == "bluetooth-sharing"
        || n == "the bluetooth-sharing"
        || n == "bluetooth share"
        || n == "the bluetooth share"
        || n == "bt sharing"
        || n == "the bt sharing"
        || n == "bt-sharing"
        || n == "the bt-sharing"
        || n == "bts"
        || n == "the bts"
        || n == "obex"
        || n == "the obex"
        || n == "file exchange"
        || n == "the file exchange"
        || n == "file-exchange"
        || n == "the file-exchange"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/bluetooth-sharing"
            | "/bluetoothsharing"
            | "/bluetooth-sharing-status"
            | "/bluetoothsharing-status"
            | "/bluetooth-share"
            | "/bluetoothshare"
            | "/bt-sharing"
            | "/btsharing"
            | "/bt-sharing-status"
            | "/bt-share"
            | "/btshare"
            | "/bts"
            | "/bts-status"
            | "/bts-state"
            | "/obex"
            | "/obex-status"
            | "/obex-sharing"
            | "/obex-share"
            | "/file-exchange"
            | "/fileexchange"
            | "/bluetooth-file-exchange"
            | "/bluetoothfileexchange"
            | "/mac-bluetooth-sharing"
            | "/mac-bt-sharing"
            | "bluetooth sharing status"
            | "the bluetooth sharing status"
            | "bluetoothsharing status"
            | "the bluetoothsharing status"
            | "bluetooth-sharing status"
            | "the bluetooth-sharing status"
            | "bluetooth share status"
            | "the bluetooth share status"
            | "bt sharing status"
            | "the bt sharing status"
            | "bt-sharing status"
            | "the bt-sharing status"
            | "bts status"
            | "the bts status"
            | "bts state"
            | "the bts state"
            | "obex status"
            | "the obex status"
            | "obex sharing status"
            | "the obex sharing status"
            | "file exchange status"
            | "the file exchange status"
            | "my bluetooth sharing"
            | "my bluetooth sharing status"
            | "my bluetoothsharing"
            | "my bt sharing"
            | "my bt-sharing"
            | "my bts"
            | "my bts status"
            | "is bluetooth sharing on"
            | "is bluetooth sharing off"
            | "is bluetooth sharing enabled"
            | "is bluetooth sharing disabled"
            | "is bluetoothsharing on"
            | "is bluetoothsharing enabled"
            | "is bluetooth-sharing on"
            | "is bluetooth-sharing enabled"
            | "is bt sharing on"
            | "is bt sharing enabled"
            | "is bt-sharing on"
            | "is bt-sharing enabled"
            | "is bts on"
            | "is bts off"
            | "is bts enabled"
            | "is bts disabled"
            | "is obex on"
            | "is obex enabled"
            | "is file exchange on"
            | "is file exchange enabled"
            | "what's my bluetooth sharing"
            | "whats my bluetooth sharing"
            | "what is my bluetooth sharing"
            | "what's my bluetooth sharing status"
            | "whats my bluetooth sharing status"
            | "what is my bluetooth sharing status"
            | "what's my bts"
            | "whats my bts"
            | "what is my bts"
            | "what's my bts status"
            | "whats my bts status"
            | "what is my bts status"
            | "what's the bluetooth sharing"
            | "whats the bluetooth sharing"
            | "what is the bluetooth sharing"
            | "what's the bluetooth sharing status"
            | "whats the bluetooth sharing status"
            | "what is the bluetooth sharing status"
            | "what's the bts status"
            | "whats the bts status"
            | "what is the bts status"
            | "how's bluetooth sharing"
            | "hows bluetooth sharing"
            | "how's the bluetooth sharing"
            | "hows the bluetooth sharing"
            | "how's bts"
            | "hows bts"
            // View/see/show me/open/list.
            | "view bluetooth sharing"
            | "view the bluetooth sharing"
            | "view bluetooth sharing status"
            | "view the bluetooth sharing status"
            | "view bluetoothsharing"
            | "view the bluetoothsharing"
            | "view bluetooth-sharing"
            | "view the bluetooth-sharing"
            | "view bluetooth share"
            | "view the bluetooth share"
            | "view bt sharing"
            | "view the bt sharing"
            | "view bt-sharing"
            | "view the bt-sharing"
            | "view bts"
            | "view the bts"
            | "view bts status"
            | "view the bts status"
            | "view obex"
            | "view the obex"
            | "view obex status"
            | "view the obex status"
            | "view file exchange"
            | "view the file exchange"
            | "see bluetooth sharing"
            | "see the bluetooth sharing"
            | "see bluetooth sharing status"
            | "see the bluetooth sharing status"
            | "see bt sharing"
            | "see the bt sharing"
            | "see bts"
            | "see the bts"
            | "see bts status"
            | "see the bts status"
            | "see obex"
            | "see the obex"
            | "open bluetooth sharing"
            | "open the bluetooth sharing"
            | "open bluetooth sharing status"
            | "open the bluetooth sharing status"
            | "open bluetoothsharing"
            | "open the bluetoothsharing"
            | "open bt sharing"
            | "open the bt sharing"
            | "open bts"
            | "open the bts"
            | "open bts status"
            | "open the bts status"
            | "open obex"
            | "open the obex"
            | "list bluetooth sharing"
            | "list the bluetooth sharing"
            | "list bluetooth sharing status"
            | "list the bluetooth sharing status"
            | "list bt sharing"
            | "list the bt sharing"
            | "list bts"
            | "list the bts"
            | "list bts status"
            | "list the bts status"
            | "show bluetooth sharing"
            | "show the bluetooth sharing"
            | "show me bluetooth sharing"
            | "show me the bluetooth sharing"
            | "show bluetooth sharing status"
            | "show the bluetooth sharing status"
            | "show me bluetooth sharing status"
            | "show me the bluetooth sharing status"
            | "show bt sharing"
            | "show the bt sharing"
            | "show me bt sharing"
            | "show me the bt sharing"
            | "show bts"
            | "show the bts"
            | "show me bts"
            | "show me the bts"
            | "show bts status"
            | "show the bts status"
            | "show me bts status"
            | "show me the bts status"
    )
}

/// Zero-LLM Bluetooth Sharing status from Bluetooth prefs + BluetoothFileExchange (macOS-only; empty elsewhere).
pub fn format_bluetooth_sharing_report() -> String {
    let v = crate::metrics::get_bluetooth_sharing_status();
    let v = v.trim();
    if v.is_empty() {
        return "**Bluetooth Sharing** — _no Bluetooth Sharing reading on this Mac right now._".to_string();
    }
    format!("**Bluetooth Sharing** · `{v}`")
}


/// Media Sharing / Home Sharing — `com.apple.amp.mediasharingd` prefs.
/// "Why" / enable/disable stay with the agent. Slash + view/open NL (`/media-sharing` · `/home-sharing`).
/// Bare `media` / `media sharing` / `home sharing` stay with the agent; `/bluetooth-sharing` · `/bts`
/// stay Bluetooth Sharing; `/file-sharing` · `/smb` stay File Sharing; `/internet-sharing` · `/ics`
/// stay Internet Sharing; `/wifi` stays SSID.
pub fn looks_like_media_sharing_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("password")
        || n.contains("screenshot")
        || n.contains("resolution")
        || n.contains("display")
        || n.contains("connect")
        || n.contains("tunnel")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("firewall")
        || n.contains("stealth")
        || n.contains("gatekeeper")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("file-vault")
        || n.contains("system integrity")
        || n.contains("time machine")
        || n.contains("timemachine")
        || n.contains("vpn")
        || n.contains("remote login")
        || n.contains("remotelogin")
        || n.contains("remote-login")
        || n.contains("screen sharing")
        || n.contains("screensharing")
        || n.contains("screen-sharing")
        || n.contains("screen share")
        || n.contains("file sharing")
        || n.contains("filesharing")
        || n.contains("file-sharing")
        || n.contains("file share")
        || n.contains("printer sharing")
        || n.contains("printersharing")
        || n.contains("printer-sharing")
        || n.contains("share printers")
        || n.contains("cups share")
        || n.contains("cups sharing")
        || n.contains("remote management")
        || n.contains("remotemanagement")
        || n.contains("remote-management")
        || n.contains("remote desktop")
        || n.contains("apple remote desktop")
        || n.contains("content caching")
        || n.contains("contentcaching")
        || n.contains("content-caching")
        || n.contains("content cache")
        || n.contains("asset cache")
        || n.contains("assetcache")
        || n.contains("asset-cache")
        || n.contains("caching server")
        || n.contains("internet sharing")
        || n.contains("internetsharing")
        || n.contains("internet-sharing")
        || n.contains("network sharing")
        || n.contains("networksharing")
        || n.contains("network-sharing")
        || n.contains("bluetooth sharing")
        || n.contains("bluetoothsharing")
        || n.contains("bluetooth-sharing")
        || n.contains("bluetooth share")
        || n.contains("file exchange")
        || n.contains("obex")
        || n.contains("remote apple events")
        || n.contains("remote-apple-events")
        || n.contains("remoteappleevents")
        || n.contains("apple events")
        || n.contains("apple-events")
        || n.contains("eppc")
        || n.contains("airplay")
        || n.contains("air play")
        || n.contains("airplay-receiver")
        || n.contains("wifi")
        || n.contains("wi-fi")
        || n.contains("ssid")
        || n == "/firewall"
        || n == "/fw"
        || n == "/afw"
        || n == "/stealth"
        || n == "/gatekeeper"
        || n == "/sip"
        || n == "/filevault"
        || n == "/file-vault"
        || n == "/timemachine"
        || n == "/tm"
        || n == "/vpn"
        || n == "/remote-login"
        || n == "/remotelogin"
        || n == "/ssh"
        || n == "/sshd"
        || n == "/screen-sharing"
        || n == "/screensharing"
        || n == "/vnc"
        || n == "/file-sharing"
        || n == "/filesharing"
        || n == "/smb"
        || n == "/smb-status"
        || n == "/smbd"
        || n == "/printer-sharing"
        || n == "/cups-share"
        || n == "/remote-management"
        || n == "/ard"
        || n == "/content-caching"
        || n == "/asset-cache"
        || n == "/internet-sharing"
        || n == "/ics"
        || n == "/nat-sharing"
        || n == "/bluetooth-sharing"
        || n == "/bluetoothsharing"
        || n == "/bt-sharing"
        || n == "/bts"
        || n == "/obex"
        || n == "/file-exchange"
        || n == "/remote-apple-events"
        || n == "/remoteappleevents"
        || n == "/apple-events"
        || n == "/rae"
        || n == "/eppc"
        || n == "/aeserver"
        || n == "/ip"
        || n == "/wifi"
        || n == "/gateway"
        || n == "/tasks"
        || n == "/screen"
        || n == "/screens"
        || n == "/display"
        || n == "/displays"
        // Bare short / ambiguous stay with the agent.
        || n == "media"
        || n == "the media"
        || n == "media sharing"
        || n == "the media sharing"
        || n == "mediasharing"
        || n == "the mediasharing"
        || n == "media-sharing"
        || n == "the media-sharing"
        || n == "home sharing"
        || n == "the home sharing"
        || n == "homesharing"
        || n == "the homesharing"
        || n == "home-sharing"
        || n == "the home-sharing"
        || n == "itunes sharing"
        || n == "the itunes sharing"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/media-sharing"
            | "/mediasharing"
            | "/media-sharing-status"
            | "/mediasharing-status"
            | "/media-share"
            | "/mediashare"
            | "/home-sharing"
            | "/homesharing"
            | "/home-sharing-status"
            | "/home-share"
            | "/homeshare"
            | "/itunes-sharing"
            | "/itunes-share"
            | "/amp-sharing"
            | "/amp-media-sharing"
            | "/mac-media-sharing"
            | "/mac-home-sharing"
            | "media sharing status"
            | "the media sharing status"
            | "mediasharing status"
            | "the mediasharing status"
            | "media-sharing status"
            | "the media-sharing status"
            | "home sharing status"
            | "the home sharing status"
            | "homesharing status"
            | "the homesharing status"
            | "home-sharing status"
            | "the home-sharing status"
            | "itunes sharing status"
            | "the itunes sharing status"
            | "my media sharing"
            | "my media sharing status"
            | "my home sharing"
            | "my home sharing status"
            | "is media sharing on"
            | "is media sharing off"
            | "is media sharing enabled"
            | "is media sharing disabled"
            | "is mediasharing on"
            | "is mediasharing enabled"
            | "is media-sharing on"
            | "is media-sharing enabled"
            | "is home sharing on"
            | "is home sharing off"
            | "is home sharing enabled"
            | "is home sharing disabled"
            | "is homesharing on"
            | "is homesharing enabled"
            | "is home-sharing on"
            | "is home-sharing enabled"
            | "what's my media sharing"
            | "whats my media sharing"
            | "what is my media sharing"
            | "what's my media sharing status"
            | "whats my media sharing status"
            | "what is my media sharing status"
            | "what's my home sharing"
            | "whats my home sharing"
            | "what is my home sharing"
            | "what's my home sharing status"
            | "whats my home sharing status"
            | "what is my home sharing status"
            | "what's the media sharing"
            | "whats the media sharing"
            | "what is the media sharing"
            | "what's the media sharing status"
            | "whats the media sharing status"
            | "what is the media sharing status"
            | "what's the home sharing"
            | "whats the home sharing"
            | "what is the home sharing"
            | "what's the home sharing status"
            | "whats the home sharing status"
            | "what is the home sharing status"
            | "how's media sharing"
            | "hows media sharing"
            | "how's the media sharing"
            | "hows the media sharing"
            | "how's home sharing"
            | "hows home sharing"
            // View/see/show me/open/list.
            | "view media sharing"
            | "view the media sharing"
            | "view media sharing status"
            | "view the media sharing status"
            | "view mediasharing"
            | "view the mediasharing"
            | "view media-sharing"
            | "view the media-sharing"
            | "view media share"
            | "view the media share"
            | "view home sharing"
            | "view the home sharing"
            | "view home sharing status"
            | "view the home sharing status"
            | "view homesharing"
            | "view the homesharing"
            | "view home-sharing"
            | "view the home-sharing"
            | "view itunes sharing"
            | "view the itunes sharing"
            | "see media sharing"
            | "see the media sharing"
            | "see media sharing status"
            | "see the media sharing status"
            | "see home sharing"
            | "see the home sharing"
            | "see home sharing status"
            | "see the home sharing status"
            | "open media sharing"
            | "open the media sharing"
            | "open media sharing status"
            | "open the media sharing status"
            | "open mediasharing"
            | "open the mediasharing"
            | "open home sharing"
            | "open the home sharing"
            | "open home sharing status"
            | "open the home sharing status"
            | "list media sharing"
            | "list the media sharing"
            | "list media sharing status"
            | "list the media sharing status"
            | "list home sharing"
            | "list the home sharing"
            | "list home sharing status"
            | "list the home sharing status"
            | "show media sharing"
            | "show the media sharing"
            | "show me media sharing"
            | "show me the media sharing"
            | "show media sharing status"
            | "show the media sharing status"
            | "show me media sharing status"
            | "show me the media sharing status"
            | "show home sharing"
            | "show the home sharing"
            | "show me home sharing"
            | "show me the home sharing"
            | "show home sharing status"
            | "show the home sharing status"
            | "show me home sharing status"
            | "show me the home sharing status"
    )
}

/// Zero-LLM Media Sharing status from mediasharingd prefs (macOS-only; empty elsewhere).
pub fn format_media_sharing_report() -> String {
    let v = crate::metrics::get_media_sharing_status();
    let v = v.trim();
    if v.is_empty() {
        return "**Media Sharing** — _no Media Sharing reading on this Mac right now._".to_string();
    }
    format!("**Media Sharing** · `{v}`")
}

/// Remote Apple Events — `systemsetup -getremoteappleevents`.
/// "Why" / enable/disable stay with the agent. Slash + view/open NL (`/remote-apple-events` · `/rae` · `/eppc`).
/// Bare `apple events` / `remote apple events` / `eppc` / `rae` stay with the agent; `/remote-login` · `/ssh`
/// stay Remote Login; `/remote-management` · `/ard` stay Remote Management; `/screen-sharing` · `/vnc`
/// stay Screen Sharing; `/media-sharing` · `/home-sharing` stay Media Sharing.
pub fn looks_like_remote_apple_events_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("password")
        || n.contains("screenshot")
        || n.contains("resolution")
        || n.contains("display")
        || n.contains("connect")
        || n.contains("tunnel")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("firewall")
        || n.contains("stealth")
        || n.contains("gatekeeper")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("file-vault")
        || n.contains("system integrity")
        || n.contains("time machine")
        || n.contains("timemachine")
        || n.contains("vpn")
        || n.contains("remote login")
        || n.contains("remotelogin")
        || n.contains("remote-login")
        || n.contains("screen sharing")
        || n.contains("screensharing")
        || n.contains("screen-sharing")
        || n.contains("screen share")
        || n.contains("file sharing")
        || n.contains("filesharing")
        || n.contains("file-sharing")
        || n.contains("file share")
        || n.contains("printer sharing")
        || n.contains("printersharing")
        || n.contains("printer-sharing")
        || n.contains("share printers")
        || n.contains("cups share")
        || n.contains("cups sharing")
        || n.contains("remote management")
        || n.contains("remotemanagement")
        || n.contains("remote-management")
        || n.contains("remote desktop")
        || n.contains("apple remote desktop")
        || n.contains("content caching")
        || n.contains("contentcaching")
        || n.contains("content-caching")
        || n.contains("content cache")
        || n.contains("asset cache")
        || n.contains("assetcache")
        || n.contains("asset-cache")
        || n.contains("internet sharing")
        || n.contains("internetsharing")
        || n.contains("internet-sharing")
        || n.contains("bluetooth sharing")
        || n.contains("bluetoothsharing")
        || n.contains("bluetooth-sharing")
        || n.contains("media sharing")
        || n.contains("mediasharing")
        || n.contains("media-sharing")
        || n.contains("home sharing")
        || n.contains("homesharing")
        || n.contains("home-sharing")
        || n.contains("airplay")
        || n.contains("air play")
        || n.contains("airplay-receiver")
        || n.contains("airplayreceiver")
        || n.contains("wifi")
        || n.contains("wi-fi")
        || n.contains("ssid")
        || n == "/firewall"
        || n == "/fw"
        || n == "/afw"
        || n == "/stealth"
        || n == "/gatekeeper"
        || n == "/sip"
        || n == "/filevault"
        || n == "/file-vault"
        || n == "/timemachine"
        || n == "/tm"
        || n == "/vpn"
        || n == "/remote-login"
        || n == "/remotelogin"
        || n == "/ssh"
        || n == "/sshd"
        || n == "/screen-sharing"
        || n == "/screensharing"
        || n == "/vnc"
        || n == "/file-sharing"
        || n == "/filesharing"
        || n == "/smb"
        || n == "/smb-status"
        || n == "/smbd"
        || n == "/printer-sharing"
        || n == "/cups-share"
        || n == "/remote-management"
        || n == "/ard"
        || n == "/content-caching"
        || n == "/asset-cache"
        || n == "/internet-sharing"
        || n == "/ics"
        || n == "/bluetooth-sharing"
        || n == "/bts"
        || n == "/obex"
        || n == "/media-sharing"
        || n == "/home-sharing"
        || n == "/airplay"
        || n == "/airplay-receiver"
        || n == "/airplayreceiver"
        || n == "/dvd-sharing"
        || n == "/cd-sharing"
        || n == "/ods"
        || n == "/odsagent"
        || n == "/ip"
        || n == "/wifi"
        || n == "/tasks"
        || n == "/screen"
        || n == "/screens"
        || n == "/display"
        || n == "/displays"
        || n.contains("dvd sharing")
        || n.contains("dvdsharing")
        || n.contains("dvd-sharing")
        || n.contains("cd sharing")
        || n.contains("cdsharing")
        || n.contains("cd-sharing")
        || n.contains("optical sharing")
        // Bare short / ambiguous stay with the agent.
        || n == "rae"
        || n == "the rae"
        || n == "eppc"
        || n == "the eppc"
        || n == "apple events"
        || n == "the apple events"
        || n == "apple event"
        || n == "the apple event"
        || n == "remote apple events"
        || n == "the remote apple events"
        || n == "remote-apple-events"
        || n == "the remote-apple-events"
        || n == "remoteappleevents"
        || n == "the remoteappleevents"
        || n == "aeserver"
        || n == "the aeserver"
        || n == "ae server"
        || n == "the ae server"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/remote-apple-events"
            | "/remoteappleevents"
            | "/remote-apple-events-status"
            | "/remoteappleevents-status"
            | "/remote-apple-event"
            | "/remoteappleevent"
            | "/apple-events"
            | "/appleevents"
            | "/apple-events-status"
            | "/appleevents-status"
            | "/apple-event"
            | "/appleevent"
            | "/rae"
            | "/rae-status"
            | "/rae-state"
            | "/eppc"
            | "/eppc-status"
            | "/eppc-state"
            | "/aeserver"
            | "/ae-server"
            | "/mac-rae"
            | "/mac-eppc"
            | "/mac-remote-apple-events"
            | "/mac-apple-events"
            | "remote apple events status"
            | "the remote apple events status"
            | "remote-apple-events status"
            | "the remote-apple-events status"
            | "remoteappleevents status"
            | "the remoteappleevents status"
            | "apple events status"
            | "the apple events status"
            | "apple-events status"
            | "the apple-events status"
            | "rae status"
            | "the rae status"
            | "rae state"
            | "the rae state"
            | "eppc status"
            | "the eppc status"
            | "eppc state"
            | "the eppc state"
            | "my remote apple events"
            | "my remote apple events status"
            | "my apple events"
            | "my apple events status"
            | "my rae"
            | "my rae status"
            | "my eppc"
            | "my eppc status"
            | "is remote apple events on"
            | "is remote apple events off"
            | "is remote apple events enabled"
            | "is remote apple events disabled"
            | "is remote-apple-events on"
            | "is remote-apple-events enabled"
            | "is remoteappleevents on"
            | "is remoteappleevents enabled"
            | "is apple events on"
            | "is apple events off"
            | "is apple events enabled"
            | "is apple events disabled"
            | "is apple-events on"
            | "is apple-events enabled"
            | "is rae on"
            | "is rae off"
            | "is rae enabled"
            | "is rae disabled"
            | "is eppc on"
            | "is eppc off"
            | "is eppc enabled"
            | "is eppc disabled"
            | "what's my remote apple events"
            | "whats my remote apple events"
            | "what is my remote apple events"
            | "what's my remote apple events status"
            | "whats my remote apple events status"
            | "what is my remote apple events status"
            | "what's the remote apple events"
            | "whats the remote apple events"
            | "what is the remote apple events"
            | "what's the remote apple events status"
            | "whats the remote apple events status"
            | "what is the remote apple events status"
            | "what's my apple events"
            | "whats my apple events"
            | "what is my apple events"
            | "what's my apple events status"
            | "whats my apple events status"
            | "what is my apple events status"
            | "what's the apple events status"
            | "whats the apple events status"
            | "what is the apple events status"
            | "what's my rae"
            | "whats my rae"
            | "what is my rae"
            | "what's my rae status"
            | "whats my rae status"
            | "what is my rae status"
            | "what's the rae status"
            | "whats the rae status"
            | "what is the rae status"
            | "what's my eppc"
            | "whats my eppc"
            | "what is my eppc"
            | "what's my eppc status"
            | "whats my eppc status"
            | "what is my eppc status"
            | "how's remote apple events"
            | "hows remote apple events"
            | "how's the remote apple events"
            | "hows the remote apple events"
            | "how's apple events"
            | "hows apple events"
            // View/see/show me/open/list.
            | "view remote apple events"
            | "view the remote apple events"
            | "view remote apple events status"
            | "view the remote apple events status"
            | "view remote-apple-events"
            | "view the remote-apple-events"
            | "view remoteappleevents"
            | "view the remoteappleevents"
            | "view apple events"
            | "view the apple events"
            | "view apple events status"
            | "view the apple events status"
            | "view apple-events"
            | "view the apple-events"
            | "view rae"
            | "view the rae"
            | "view rae status"
            | "view the rae status"
            | "view eppc"
            | "view the eppc"
            | "view eppc status"
            | "view the eppc status"
            | "see remote apple events"
            | "see the remote apple events"
            | "see remote apple events status"
            | "see the remote apple events status"
            | "see apple events"
            | "see the apple events"
            | "see apple events status"
            | "see the apple events status"
            | "see rae"
            | "see the rae"
            | "see rae status"
            | "see the rae status"
            | "see eppc"
            | "see the eppc"
            | "see eppc status"
            | "see the eppc status"
            | "open remote apple events"
            | "open the remote apple events"
            | "open remote apple events status"
            | "open the remote apple events status"
            | "open apple events"
            | "open the apple events"
            | "open apple events status"
            | "open the apple events status"
            | "open rae"
            | "open the rae"
            | "open rae status"
            | "open the rae status"
            | "open eppc"
            | "open the eppc"
            | "open eppc status"
            | "open the eppc status"
            | "list remote apple events"
            | "list the remote apple events"
            | "list remote apple events status"
            | "list the remote apple events status"
            | "list apple events"
            | "list the apple events"
            | "list apple events status"
            | "list the apple events status"
            | "list rae"
            | "list the rae"
            | "list rae status"
            | "list the rae status"
            | "list eppc"
            | "list the eppc"
            | "list eppc status"
            | "list the eppc status"
            | "show remote apple events"
            | "show the remote apple events"
            | "show me remote apple events"
            | "show me the remote apple events"
            | "show remote apple events status"
            | "show the remote apple events status"
            | "show me remote apple events status"
            | "show me the remote apple events status"
            | "show apple events"
            | "show the apple events"
            | "show me apple events"
            | "show me the apple events"
            | "show apple events status"
            | "show the apple events status"
            | "show me apple events status"
            | "show me the apple events status"
            | "show rae"
            | "show the rae"
            | "show me rae"
            | "show me the rae"
            | "show rae status"
            | "show the rae status"
            | "show me rae status"
            | "show me the rae status"
            | "show eppc"
            | "show the eppc"
            | "show me eppc"
            | "show me the eppc"
            | "show eppc status"
            | "show the eppc status"
            | "show me eppc status"
            | "show me the eppc status"
    )
}

/// Zero-LLM Remote Apple Events status from `systemsetup -getremoteappleevents` (macOS-only; empty elsewhere).
pub fn format_remote_apple_events_report() -> String {
    let v = crate::metrics::get_remote_apple_events_status();
    let v = v.trim();
    if v.is_empty() {
        return "**Remote Apple Events** — _no Remote Apple Events reading on this Mac right now._"
            .to_string();
    }
    format!("**Remote Apple Events** · `{v}`")
}

/// AirPlay Receiver — Control Center host prefs + ports 5000/7000.
/// "Why" / enable/disable stay with the agent. Slash + view/open NL (`/airplay` · `/airplay-receiver`).
/// Bare `airplay` / `airplay receiver` stay with the agent; `/media-sharing` · `/home-sharing` stay Media
/// Sharing; `/remote-apple-events` · `/rae` · `/eppc` stay Remote Apple Events; `/screen-sharing` · `/vnc`
/// stay Screen Sharing; `/display` stays connected displays.
pub fn looks_like_airplay_receiver_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("password")
        || n.contains("screenshot")
        || n.contains("resolution")
        || n.contains("connect")
        || n.contains("tunnel")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("firewall")
        || n.contains("stealth")
        || n.contains("gatekeeper")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("file-vault")
        || n.contains("system integrity")
        || n.contains("time machine")
        || n.contains("timemachine")
        || n.contains("vpn")
        || n.contains("remote login")
        || n.contains("remotelogin")
        || n.contains("remote-login")
        || n.contains("screen sharing")
        || n.contains("screensharing")
        || n.contains("screen-sharing")
        || n.contains("screen share")
        || n.contains("file sharing")
        || n.contains("filesharing")
        || n.contains("file-sharing")
        || n.contains("file share")
        || n.contains("printer sharing")
        || n.contains("printersharing")
        || n.contains("printer-sharing")
        || n.contains("share printers")
        || n.contains("cups share")
        || n.contains("cups sharing")
        || n.contains("remote management")
        || n.contains("remotemanagement")
        || n.contains("remote-management")
        || n.contains("remote desktop")
        || n.contains("apple remote desktop")
        || n.contains("remote apple events")
        || n.contains("remote-apple-events")
        || n.contains("remoteappleevents")
        || n.contains("apple events")
        || n.contains("apple-events")
        || n.contains("content caching")
        || n.contains("contentcaching")
        || n.contains("content-caching")
        || n.contains("content cache")
        || n.contains("asset cache")
        || n.contains("assetcache")
        || n.contains("asset-cache")
        || n.contains("internet sharing")
        || n.contains("internetsharing")
        || n.contains("internet-sharing")
        || n.contains("bluetooth sharing")
        || n.contains("bluetoothsharing")
        || n.contains("bluetooth-sharing")
        || n.contains("media sharing")
        || n.contains("mediasharing")
        || n.contains("media-sharing")
        || n.contains("home sharing")
        || n.contains("homesharing")
        || n.contains("home-sharing")
        || n.contains("wifi")
        || n.contains("wi-fi")
        || n.contains("ssid")
        || n == "/firewall"
        || n == "/fw"
        || n == "/afw"
        || n == "/stealth"
        || n == "/gatekeeper"
        || n == "/sip"
        || n == "/filevault"
        || n == "/file-vault"
        || n == "/timemachine"
        || n == "/tm"
        || n == "/vpn"
        || n == "/remote-login"
        || n == "/remotelogin"
        || n == "/ssh"
        || n == "/sshd"
        || n == "/screen-sharing"
        || n == "/screensharing"
        || n == "/vnc"
        || n == "/file-sharing"
        || n == "/filesharing"
        || n == "/smb"
        || n == "/smb-status"
        || n == "/smbd"
        || n == "/printer-sharing"
        || n == "/cups-share"
        || n == "/remote-management"
        || n == "/ard"
        || n == "/content-caching"
        || n == "/asset-cache"
        || n == "/internet-sharing"
        || n == "/ics"
        || n == "/bluetooth-sharing"
        || n == "/bts"
        || n == "/obex"
        || n == "/media-sharing"
        || n == "/home-sharing"
        || n == "/remote-apple-events"
        || n == "/rae"
        || n == "/eppc"
        || n == "/apple-events"
        || n == "/dvd-sharing"
        || n == "/cd-sharing"
        || n == "/ods"
        || n == "/odsagent"
        || n == "/airdrop"
        || n == "/air-drop"
        || n == "/mac-airdrop"
        || n.contains("airdrop")
        || n.contains("air drop")
        || n.contains("air-drop")
        || n == "/ip"
        || n == "/wifi"
        || n == "/tasks"
        || n == "/screen"
        || n == "/screens"
        || n == "/display"
        || n == "/displays"
        || n.contains("dvd sharing")
        || n.contains("dvdsharing")
        || n.contains("dvd-sharing")
        || n.contains("cd sharing")
        || n.contains("cdsharing")
        || n.contains("cd-sharing")
        || n.contains("dvd or cd")
        || n.contains("dvd-cd")
        || n.contains("optical sharing")
        || n.contains("optical disk")
        || n == "airplay"
        || n == "the airplay"
        || n == "air play"
        || n == "the air play"
        || n == "airplay receiver"
        || n == "the airplay receiver"
        || n == "air play receiver"
        || n == "the air play receiver"
        || n == "airplay-receiver"
        || n == "the airplay-receiver"
        || n == "receiver"
        || n == "the receiver"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/airplay"
            | "/airplay-status"
            | "/airplay-state"
            | "/airplay-receiver"
            | "/airplayreceiver"
            | "/airplay-receiver-status"
            | "/airplayreceiver-status"
            | "/airplay-recv"
            | "/airplay-rx"
            | "/ap-receiver"
            | "/mac-airplay"
            | "/mac-airplay-receiver"
            | "/airplay-receiver-mode"
            | "airplay status"
            | "the airplay status"
            | "airplay state"
            | "the airplay state"
            | "airplay receiver status"
            | "the airplay receiver status"
            | "airplay-receiver status"
            | "the airplay-receiver status"
            | "airplayreceiver status"
            | "the airplayreceiver status"
            | "my airplay"
            | "my airplay status"
            | "my airplay receiver"
            | "my airplay receiver status"
            | "is airplay on"
            | "is airplay off"
            | "is airplay enabled"
            | "is airplay disabled"
            | "is airplay receiver on"
            | "is airplay receiver off"
            | "is airplay receiver enabled"
            | "is airplay receiver disabled"
            | "is airplay-receiver on"
            | "is airplay-receiver enabled"
            | "is airplayreceiver on"
            | "is airplayreceiver enabled"
            | "what's my airplay"
            | "whats my airplay"
            | "what is my airplay"
            | "what's my airplay status"
            | "whats my airplay status"
            | "what is my airplay status"
            | "what's my airplay receiver"
            | "whats my airplay receiver"
            | "what is my airplay receiver"
            | "what's my airplay receiver status"
            | "whats my airplay receiver status"
            | "what is my airplay receiver status"
            | "what's the airplay"
            | "whats the airplay"
            | "what is the airplay"
            | "what's the airplay status"
            | "whats the airplay status"
            | "what is the airplay status"
            | "what's the airplay receiver"
            | "whats the airplay receiver"
            | "what is the airplay receiver"
            | "what's the airplay receiver status"
            | "whats the airplay receiver status"
            | "what is the airplay receiver status"
            | "how's airplay"
            | "hows airplay"
            | "how's the airplay"
            | "hows the airplay"
            | "how's airplay receiver"
            | "hows airplay receiver"
            | "view airplay"
            | "view the airplay"
            | "view airplay status"
            | "view the airplay status"
            | "view airplay receiver"
            | "view the airplay receiver"
            | "view airplay receiver status"
            | "view the airplay receiver status"
            | "view airplay-receiver"
            | "view the airplay-receiver"
            | "view airplayreceiver"
            | "view the airplayreceiver"
            | "see airplay"
            | "see the airplay"
            | "see airplay status"
            | "see the airplay status"
            | "see airplay receiver"
            | "see the airplay receiver"
            | "see airplay receiver status"
            | "see the airplay receiver status"
            | "open airplay"
            | "open the airplay"
            | "open airplay status"
            | "open the airplay status"
            | "open airplay receiver"
            | "open the airplay receiver"
            | "open airplay receiver status"
            | "open the airplay receiver status"
            | "list airplay"
            | "list the airplay"
            | "list airplay status"
            | "list the airplay status"
            | "list airplay receiver"
            | "list the airplay receiver"
            | "list airplay receiver status"
            | "list the airplay receiver status"
            | "show airplay"
            | "show the airplay"
            | "show me airplay"
            | "show me the airplay"
            | "show airplay status"
            | "show the airplay status"
            | "show me airplay status"
            | "show me the airplay status"
            | "show airplay receiver"
            | "show the airplay receiver"
            | "show me airplay receiver"
            | "show me the airplay receiver"
            | "show airplay receiver status"
            | "show the airplay receiver status"
            | "show me airplay receiver status"
            | "show me the airplay receiver status"
    )
}

/// Zero-LLM AirPlay Receiver status from Control Center prefs + listen ports (macOS-only; empty elsewhere).
pub fn format_airplay_receiver_report() -> String {
    let v = crate::metrics::get_airplay_receiver_status();
    let v = v.trim();
    if v.is_empty() {
        return "**AirPlay Receiver** — _no AirPlay Receiver reading on this Mac right now._".to_string();
    }
    format!("**AirPlay Receiver** · `{v}`")
}

/// DVD or CD Sharing (ODSAgent) — Sharing pane optical-disk share, not Media/AirPlay/File Sharing.
/// Slash + view/open NL (`/dvd-sharing` · `/cd-sharing` · `/ods` · `view dvd sharing`).
/// Bare `dvd` / `cd` / `dvd sharing` / `cd sharing` / `ods` stay with the agent.
pub fn looks_like_dvd_cd_sharing_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("password")
        || n.contains("screenshot")
        || n.contains("resolution")
        || n.contains("connect")
        || n.contains("tunnel")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("firewall")
        || n.contains("stealth")
        || n.contains("gatekeeper")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("file-vault")
        || n.contains("system integrity")
        || n.contains("time machine")
        || n.contains("timemachine")
        || n.contains("vpn")
        || n.contains("remote login")
        || n.contains("remotelogin")
        || n.contains("remote-login")
        || n.contains("screen sharing")
        || n.contains("screensharing")
        || n.contains("screen-sharing")
        || n.contains("screen share")
        || n.contains("file sharing")
        || n.contains("filesharing")
        || n.contains("file-sharing")
        || n.contains("file share")
        || n.contains("printer sharing")
        || n.contains("printersharing")
        || n.contains("printer-sharing")
        || n.contains("share printers")
        || n.contains("cups share")
        || n.contains("cups sharing")
        || n.contains("remote management")
        || n.contains("remotemanagement")
        || n.contains("remote-management")
        || n.contains("remote desktop")
        || n.contains("apple remote desktop")
        || n.contains("remote apple events")
        || n.contains("remote-apple-events")
        || n.contains("remoteappleevents")
        || n.contains("apple events")
        || n.contains("apple-events")
        || n.contains("content caching")
        || n.contains("contentcaching")
        || n.contains("content-caching")
        || n.contains("content cache")
        || n.contains("asset cache")
        || n.contains("assetcache")
        || n.contains("asset-cache")
        || n.contains("internet sharing")
        || n.contains("internetsharing")
        || n.contains("internet-sharing")
        || n.contains("bluetooth sharing")
        || n.contains("bluetoothsharing")
        || n.contains("bluetooth-sharing")
        || n.contains("media sharing")
        || n.contains("mediasharing")
        || n.contains("media-sharing")
        || n.contains("home sharing")
        || n.contains("homesharing")
        || n.contains("home-sharing")
        || n.contains("airplay")
        || n.contains("air play")
        || n.contains("airplay-receiver")
        || n.contains("wifi")
        || n.contains("wi-fi")
        || n.contains("ssid")
        || n == "/firewall"
        || n == "/fw"
        || n == "/afw"
        || n == "/stealth"
        || n == "/gatekeeper"
        || n == "/sip"
        || n == "/filevault"
        || n == "/file-vault"
        || n == "/timemachine"
        || n == "/tm"
        || n == "/vpn"
        || n == "/remote-login"
        || n == "/remotelogin"
        || n == "/ssh"
        || n == "/sshd"
        || n == "/screen-sharing"
        || n == "/screensharing"
        || n == "/vnc"
        || n == "/file-sharing"
        || n == "/filesharing"
        || n == "/smb"
        || n == "/smb-status"
        || n == "/smbd"
        || n == "/printer-sharing"
        || n == "/cups-share"
        || n == "/remote-management"
        || n == "/ard"
        || n == "/content-caching"
        || n == "/asset-cache"
        || n == "/internet-sharing"
        || n == "/ics"
        || n == "/bluetooth-sharing"
        || n == "/bts"
        || n == "/obex"
        || n == "/media-sharing"
        || n == "/home-sharing"
        || n == "/remote-apple-events"
        || n == "/rae"
        || n == "/eppc"
        || n == "/apple-events"
        || n == "/airplay"
        || n == "/airplay-receiver"
        || n == "/airdrop"
        || n == "/air-drop"
        || n == "/mac-airdrop"
        || n.contains("airdrop")
        || n.contains("air drop")
        || n.contains("air-drop")
        || n == "/ip"
        || n == "/wifi"
        || n == "/tasks"
        || n == "/screen"
        || n == "/screens"
        || n == "/display"
        || n == "/displays"
        // Bare phrases stay with the agent (incl. bare `cd` / `dvd` / `ods`).
        // Do NOT use contains() for our own tokens — that would kill "dvd sharing status".
        || n == "dvd"
        || n == "the dvd"
        || n == "cd"
        || n == "the cd"
        || n == "ods"
        || n == "the ods"
        || n == "dvd sharing"
        || n == "the dvd sharing"
        || n == "cd sharing"
        || n == "the cd sharing"
        || n == "dvd or cd sharing"
        || n == "the dvd or cd sharing"
        || n == "dvd-cd sharing"
        || n == "the dvd-cd sharing"
        || n == "optical"
        || n == "the optical"
        || n == "optical sharing"
        || n == "the optical sharing"
        || n == "optical disk"
        || n == "the optical disk"
        || n == "optical disk sharing"
        || n == "the optical disk sharing"
        || n == "odsagent"
        || n == "the odsagent"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/dvd-sharing"
            | "/dvdsharing"
            | "/dvd-sharing-status"
            | "/dvdsharing-status"
            | "/dvd-share"
            | "/dvdshare"
            | "/cd-sharing"
            | "/cdsharing"
            | "/cd-sharing-status"
            | "/cdsharing-status"
            | "/cd-share"
            | "/cdshare"
            | "/dvd-cd-sharing"
            | "/dvdcdsharing"
            | "/dvd-cd-share"
            | "/dvd-or-cd-sharing"
            | "/dvdorcdsharing"
            | "/optical-sharing"
            | "/opticalsharing"
            | "/optical-disk-sharing"
            | "/ods"
            | "/ods-status"
            | "/ods-state"
            | "/odsagent"
            | "/ods-agent"
            | "/mac-dvd-sharing"
            | "/mac-cd-sharing"
            | "/mac-ods"
            | "/mac-optical-sharing"
            | "dvd sharing status"
            | "the dvd sharing status"
            | "dvd-sharing status"
            | "the dvd-sharing status"
            | "dvdsharing status"
            | "the dvdsharing status"
            | "cd sharing status"
            | "the cd sharing status"
            | "cd-sharing status"
            | "the cd-sharing status"
            | "cdsharing status"
            | "the cdsharing status"
            | "dvd or cd sharing status"
            | "the dvd or cd sharing status"
            | "dvd-cd sharing status"
            | "the dvd-cd sharing status"
            | "optical sharing status"
            | "the optical sharing status"
            | "optical disk sharing status"
            | "the optical disk sharing status"
            | "ods status"
            | "the ods status"
            | "ods state"
            | "the ods state"
            | "odsagent status"
            | "the odsagent status"
            | "my dvd sharing"
            | "my dvd sharing status"
            | "my cd sharing"
            | "my cd sharing status"
            | "my ods"
            | "my ods status"
            | "is dvd sharing on"
            | "is dvd sharing off"
            | "is dvd sharing enabled"
            | "is dvd sharing disabled"
            | "is cd sharing on"
            | "is cd sharing off"
            | "is cd sharing enabled"
            | "is cd sharing disabled"
            | "is dvd-cd sharing on"
            | "is dvd-cd sharing enabled"
            | "is dvd or cd sharing on"
            | "is dvd or cd sharing enabled"
            | "is optical sharing on"
            | "is optical sharing enabled"
            | "is ods on"
            | "is ods off"
            | "is ods enabled"
            | "is ods disabled"
            | "is odsagent on"
            | "is odsagent enabled"
            | "what's my dvd sharing"
            | "whats my dvd sharing"
            | "what is my dvd sharing"
            | "what's my dvd sharing status"
            | "whats my dvd sharing status"
            | "what is my dvd sharing status"
            | "what's my cd sharing"
            | "whats my cd sharing"
            | "what is my cd sharing"
            | "what's my cd sharing status"
            | "whats my cd sharing status"
            | "what is my cd sharing status"
            | "what's my ods"
            | "whats my ods"
            | "what is my ods"
            | "what's my ods status"
            | "whats my ods status"
            | "what is my ods status"
            | "what's the dvd sharing"
            | "whats the dvd sharing"
            | "what is the dvd sharing"
            | "what's the dvd sharing status"
            | "whats the dvd sharing status"
            | "what is the dvd sharing status"
            | "what's the cd sharing"
            | "whats the cd sharing"
            | "what is the cd sharing"
            | "what's the cd sharing status"
            | "whats the cd sharing status"
            | "what is the cd sharing status"
            | "what's the ods"
            | "whats the ods"
            | "what is the ods"
            | "what's the ods status"
            | "whats the ods status"
            | "what is the ods status"
            | "how's dvd sharing"
            | "hows dvd sharing"
            | "how's the dvd sharing"
            | "hows the dvd sharing"
            | "how's cd sharing"
            | "hows cd sharing"
            | "how's ods"
            | "hows ods"
            | "view dvd sharing"
            | "view the dvd sharing"
            | "view dvd sharing status"
            | "view the dvd sharing status"
            | "view dvd-sharing"
            | "view the dvd-sharing"
            | "view dvdsharing"
            | "view the dvdsharing"
            | "view cd sharing"
            | "view the cd sharing"
            | "view cd sharing status"
            | "view the cd sharing status"
            | "view cd-sharing"
            | "view the cd-sharing"
            | "view cdsharing"
            | "view the cdsharing"
            | "view dvd or cd sharing"
            | "view the dvd or cd sharing"
            | "view dvd-cd sharing"
            | "view the dvd-cd sharing"
            | "view optical sharing"
            | "view the optical sharing"
            | "view optical disk sharing"
            | "view the optical disk sharing"
            | "view ods"
            | "view the ods"
            | "view ods status"
            | "view the ods status"
            | "view odsagent"
            | "view the odsagent"
            | "see dvd sharing"
            | "see the dvd sharing"
            | "see dvd sharing status"
            | "see the dvd sharing status"
            | "see cd sharing"
            | "see the cd sharing"
            | "see cd sharing status"
            | "see the cd sharing status"
            | "see ods"
            | "see the ods"
            | "see ods status"
            | "see the ods status"
            | "open dvd sharing"
            | "open the dvd sharing"
            | "open dvd sharing status"
            | "open the dvd sharing status"
            | "open cd sharing"
            | "open the cd sharing"
            | "open cd sharing status"
            | "open the cd sharing status"
            | "open ods"
            | "open the ods"
            | "open ods status"
            | "open the ods status"
            | "list dvd sharing"
            | "list the dvd sharing"
            | "list dvd sharing status"
            | "list the dvd sharing status"
            | "list cd sharing"
            | "list the cd sharing"
            | "list cd sharing status"
            | "list the cd sharing status"
            | "list ods"
            | "list the ods"
            | "list ods status"
            | "list the ods status"
            | "show dvd sharing"
            | "show the dvd sharing"
            | "show me dvd sharing"
            | "show me the dvd sharing"
            | "show dvd sharing status"
            | "show the dvd sharing status"
            | "show me dvd sharing status"
            | "show me the dvd sharing status"
            | "show cd sharing"
            | "show the cd sharing"
            | "show me cd sharing"
            | "show me the cd sharing"
            | "show cd sharing status"
            | "show the cd sharing status"
            | "show me cd sharing status"
            | "show me the cd sharing status"
            | "show ods"
            | "show the ods"
            | "show me ods"
            | "show me the ods"
            | "show ods status"
            | "show the ods status"
            | "show me ods status"
            | "show me the ods status"
    )
}

/// Zero-LLM DVD or CD Sharing status from ODSAgent launchd (macOS-only; empty elsewhere).
pub fn format_dvd_cd_sharing_report() -> String {
    let v = crate::metrics::get_dvd_cd_sharing_status();
    let v = v.trim();
    if v.is_empty() {
        return "**DVD or CD Sharing** — _no DVD/CD Sharing reading on this Mac right now._".to_string();
    }
    format!("**DVD or CD Sharing** · `{v}`")
}

/// AirDrop — sharingd DiscoverableMode (Off / Contacts Only / Everyone).
/// Slash + view/open NL (`/airdrop` · `/air-drop` · `view airdrop`).
/// Bare `airdrop` / `air drop` stay with the agent; `/airplay` stays AirPlay Receiver;
/// `/bluetooth-sharing` · `/bts` stay Bluetooth Sharing; `/media-sharing` stays Media Sharing;
/// `/file-sharing` · `/smb` stay File Sharing.
pub fn looks_like_airdrop_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("password")
        || n.contains("screenshot")
        || n.contains("resolution")
        || n.contains("connect")
        || n.contains("tunnel")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("firewall")
        || n.contains("stealth")
        || n.contains("gatekeeper")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("file-vault")
        || n.contains("system integrity")
        || n.contains("time machine")
        || n.contains("timemachine")
        || n.contains("vpn")
        || n.contains("remote login")
        || n.contains("remotelogin")
        || n.contains("remote-login")
        || n.contains("screen sharing")
        || n.contains("screensharing")
        || n.contains("screen-sharing")
        || n.contains("screen share")
        || n.contains("file sharing")
        || n.contains("filesharing")
        || n.contains("file-sharing")
        || n.contains("file share")
        || n.contains("printer sharing")
        || n.contains("printersharing")
        || n.contains("printer-sharing")
        || n.contains("share printers")
        || n.contains("cups share")
        || n.contains("cups sharing")
        || n.contains("remote management")
        || n.contains("remotemanagement")
        || n.contains("remote-management")
        || n.contains("remote desktop")
        || n.contains("apple remote desktop")
        || n.contains("remote apple events")
        || n.contains("remote-apple-events")
        || n.contains("remoteappleevents")
        || n.contains("apple events")
        || n.contains("apple-events")
        || n.contains("content caching")
        || n.contains("contentcaching")
        || n.contains("content-caching")
        || n.contains("content cache")
        || n.contains("asset cache")
        || n.contains("assetcache")
        || n.contains("asset-cache")
        || n.contains("internet sharing")
        || n.contains("internetsharing")
        || n.contains("internet-sharing")
        || n.contains("bluetooth sharing")
        || n.contains("bluetoothsharing")
        || n.contains("bluetooth-sharing")
        || n.contains("media sharing")
        || n.contains("mediasharing")
        || n.contains("media-sharing")
        || n.contains("home sharing")
        || n.contains("homesharing")
        || n.contains("home-sharing")
        || n.contains("airplay")
        || n.contains("air play")
        || n.contains("airplay-receiver")
        || n.contains("dvd sharing")
        || n.contains("dvd-sharing")
        || n.contains("cd sharing")
        || n.contains("cd-sharing")
        || n.contains("optical sharing")
        || n.contains("wifi")
        || n.contains("wi-fi")
        || n.contains("ssid")
        || n == "/firewall"
        || n == "/fw"
        || n == "/afw"
        || n == "/stealth"
        || n == "/gatekeeper"
        || n == "/sip"
        || n == "/filevault"
        || n == "/file-vault"
        || n == "/timemachine"
        || n == "/tm"
        || n == "/vpn"
        || n == "/remote-login"
        || n == "/remotelogin"
        || n == "/ssh"
        || n == "/sshd"
        || n == "/screen-sharing"
        || n == "/screensharing"
        || n == "/vnc"
        || n == "/file-sharing"
        || n == "/filesharing"
        || n == "/smb"
        || n == "/smb-status"
        || n == "/smbd"
        || n == "/printer-sharing"
        || n == "/cups-share"
        || n == "/remote-management"
        || n == "/ard"
        || n == "/content-caching"
        || n == "/asset-cache"
        || n == "/internet-sharing"
        || n == "/ics"
        || n == "/bluetooth-sharing"
        || n == "/bts"
        || n == "/obex"
        || n == "/media-sharing"
        || n == "/home-sharing"
        || n == "/remote-apple-events"
        || n == "/rae"
        || n == "/eppc"
        || n == "/apple-events"
        || n == "/airplay"
        || n == "/airplay-receiver"
        || n == "/dvd-sharing"
        || n == "/cd-sharing"
        || n == "/ods"
        || n == "/odsagent"
        || n == "/ip"
        || n == "/wifi"
        || n == "/tasks"
        || n == "/screen"
        || n == "/screens"
        || n == "/display"
        || n == "/displays"
        // Bare phrases stay with the agent.
        || n == "airdrop"
        || n == "the airdrop"
        || n == "air drop"
        || n == "the air drop"
        || n == "air-drop"
        || n == "the air-drop"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/airdrop"
            | "/air-drop"
            | "/airdrop-status"
            | "/air-drop-status"
            | "/airdrop-state"
            | "/air-drop-state"
            | "/airdrop-mode"
            | "/air-drop-mode"
            | "/mac-airdrop"
            | "/mac-air-drop"
            | "airdrop status"
            | "the airdrop status"
            | "air-drop status"
            | "the air-drop status"
            | "air drop status"
            | "the air drop status"
            | "airdrop state"
            | "the airdrop state"
            | "airdrop mode"
            | "the airdrop mode"
            | "air drop mode"
            | "the air drop mode"
            | "my airdrop"
            | "my airdrop status"
            | "my air drop"
            | "my air drop status"
            | "is airdrop on"
            | "is airdrop off"
            | "is airdrop enabled"
            | "is airdrop disabled"
            | "is air drop on"
            | "is air drop off"
            | "is air drop enabled"
            | "is air drop disabled"
            | "is air-drop on"
            | "is air-drop enabled"
            | "what's my airdrop"
            | "whats my airdrop"
            | "what is my airdrop"
            | "what's my airdrop status"
            | "whats my airdrop status"
            | "what is my airdrop status"
            | "what's my air drop"
            | "whats my air drop"
            | "what is my air drop"
            | "what's my air drop status"
            | "whats my air drop status"
            | "what is my air drop status"
            | "what's the airdrop"
            | "whats the airdrop"
            | "what is the airdrop"
            | "what's the airdrop status"
            | "whats the airdrop status"
            | "what is the airdrop status"
            | "what's the air drop"
            | "whats the air drop"
            | "what is the air drop"
            | "what's the air drop status"
            | "whats the air drop status"
            | "what is the air drop status"
            | "how's airdrop"
            | "hows airdrop"
            | "how's the airdrop"
            | "hows the airdrop"
            | "how's air drop"
            | "hows air drop"
            | "view airdrop"
            | "view the airdrop"
            | "view airdrop status"
            | "view the airdrop status"
            | "view air-drop"
            | "view the air-drop"
            | "view air drop"
            | "view the air drop"
            | "view air drop status"
            | "view the air drop status"
            | "see airdrop"
            | "see the airdrop"
            | "see airdrop status"
            | "see the airdrop status"
            | "see air drop"
            | "see the air drop"
            | "see air drop status"
            | "see the air drop status"
            | "open airdrop"
            | "open the airdrop"
            | "open airdrop status"
            | "open the airdrop status"
            | "open air drop"
            | "open the air drop"
            | "open air drop status"
            | "open the air drop status"
            | "list airdrop"
            | "list the airdrop"
            | "list airdrop status"
            | "list the airdrop status"
            | "list air drop"
            | "list the air drop"
            | "list air drop status"
            | "list the air drop status"
    )
}

/// Zero-LLM AirDrop status from sharingd DiscoverableMode (macOS-only; empty elsewhere).
pub fn format_airdrop_report() -> String {
    let v = crate::metrics::get_airdrop_status();
    let v = v.trim();
    if v.is_empty() {
        return "**AirDrop** — _no AirDrop reading on this Mac right now._".to_string();
    }
    format!("**AirDrop** · `{v}`")
}

/// Guest Account — loginwindow GuestEnabled (System Settings → Users & Groups).
/// Slash + view/open NL (`/guest` · `/guest-account` · `view guest account`).
/// Bare `guest` / `guest account` stay with the agent; `/username` · `/whoami` stay login name;
/// `/filevault` stays FileVault; `/sip` stays SIP; `/airdrop` stays AirDrop.
pub fn looks_like_guest_account_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("password")
        || n.contains("screenshot")
        || n.contains("resolution")
        || n.contains("connect")
        || n.contains("tunnel")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("create")
        || n.contains("delete")
        || n.contains("remove")
        || n.contains("firewall")
        || n.contains("stealth")
        || n.contains("gatekeeper")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("file-vault")
        || n.contains("system integrity")
        || n.contains("time machine")
        || n.contains("timemachine")
        || n.contains("vpn")
        || n.contains("remote login")
        || n.contains("remotelogin")
        || n.contains("remote-login")
        || n.contains("screen sharing")
        || n.contains("screensharing")
        || n.contains("screen-sharing")
        || n.contains("file sharing")
        || n.contains("filesharing")
        || n.contains("file-sharing")
        || n.contains("smb guest")
        || n.contains("smb-guest")
        || n.contains("smbguest")
        || n.contains("guest access")
        || n.contains("guest-access")
        || n.contains("airdrop")
        || n.contains("air drop")
        || n.contains("air-drop")
        || n.contains("airplay")
        || n.contains("username")
        || n.contains("user name")
        || n.contains("whoami")
        || n.contains("who am i")
        || n == "/firewall"
        || n == "/fw"
        || n == "/afw"
        || n == "/stealth"
        || n == "/gatekeeper"
        || n == "/sip"
        || n == "/filevault"
        || n == "/file-vault"
        || n == "/timemachine"
        || n == "/tm"
        || n == "/vpn"
        || n == "/remote-login"
        || n == "/ssh"
        || n == "/screen-sharing"
        || n == "/vnc"
        || n == "/file-sharing"
        || n == "/smb"
        || n == "/smb-guest"
        || n == "/smbguest"
        || n == "/guest-access"
        || n == "/guestaccess"
        || n == "/airdrop"
        || n == "/air-drop"
        || n == "/airplay"
        || n == "/username"
        || n == "/whoami"
        || n == "/ip"
        || n == "/wifi"
        || n == "/tasks"
        || n == "/screen"
        || n == "/screens"
        || n == "/display"
        || n == "/displays"
        // Bare phrases stay with the agent.
        || n == "guest"
        || n == "the guest"
        || n == "guest account"
        || n == "the guest account"
        || n == "guest user"
        || n == "the guest user"
        || n == "guest login"
        || n == "the guest login"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/guest"
            | "/guest-account"
            | "/guestaccount"
            | "/guest-status"
            | "/guest-account-status"
            | "/guestaccountstatus"
            | "/guest-state"
            | "/guest-account-state"
            | "/guest-user"
            | "/guestuser"
            | "/guest-login"
            | "/guestlogin"
            | "/mac-guest"
            | "/mac-guest-account"
            | "/mac-guest-user"
            | "guest status"
            | "the guest status"
            | "guest account status"
            | "the guest account status"
            | "guest user status"
            | "the guest user status"
            | "guest login status"
            | "the guest login status"
            | "guest state"
            | "the guest state"
            | "guest account state"
            | "the guest account state"
            | "my guest"
            | "my guest status"
            | "my guest account"
            | "my guest account status"
            | "my guest user"
            | "my guest user status"
            | "is guest on"
            | "is guest off"
            | "is guest enabled"
            | "is guest disabled"
            | "is guest account on"
            | "is guest account off"
            | "is guest account enabled"
            | "is guest account disabled"
            | "is guest user on"
            | "is guest user off"
            | "is guest user enabled"
            | "is guest user disabled"
            | "is guest login on"
            | "is guest login enabled"
            | "what's my guest"
            | "whats my guest"
            | "what is my guest"
            | "what's my guest status"
            | "whats my guest status"
            | "what is my guest status"
            | "what's my guest account"
            | "whats my guest account"
            | "what is my guest account"
            | "what's my guest account status"
            | "whats my guest account status"
            | "what is my guest account status"
            | "what's the guest"
            | "whats the guest"
            | "what is the guest"
            | "what's the guest status"
            | "whats the guest status"
            | "what is the guest status"
            | "what's the guest account"
            | "whats the guest account"
            | "what is the guest account"
            | "what's the guest account status"
            | "whats the guest account status"
            | "what is the guest account status"
            | "how's guest"
            | "hows guest"
            | "how's the guest"
            | "hows the guest"
            | "how's guest account"
            | "hows guest account"
            | "how's the guest account"
            | "hows the guest account"
            | "view guest"
            | "view the guest"
            | "view guest status"
            | "view the guest status"
            | "view guest account"
            | "view the guest account"
            | "view guest account status"
            | "view the guest account status"
            | "view guest user"
            | "view the guest user"
            | "view guest user status"
            | "view the guest user status"
            | "view guest login"
            | "view the guest login"
            | "see guest"
            | "see the guest"
            | "see guest status"
            | "see the guest status"
            | "see guest account"
            | "see the guest account"
            | "see guest account status"
            | "see the guest account status"
            | "see guest user"
            | "see the guest user"
            | "open guest"
            | "open the guest"
            | "open guest status"
            | "open the guest status"
            | "open guest account"
            | "open the guest account"
            | "open guest account status"
            | "open the guest account status"
            | "open guest user"
            | "open the guest user"
            | "list guest"
            | "list the guest"
            | "list guest status"
            | "list the guest status"
            | "list guest account"
            | "list the guest account"
            | "list guest account status"
            | "list the guest account status"
            | "list guest user"
            | "list the guest user"
    )
}

/// Zero-LLM Guest Account status from loginwindow GuestEnabled (macOS-only; empty elsewhere).
pub fn format_guest_account_report() -> String {
    let v = crate::metrics::get_guest_account_status();
    let v = v.trim();
    if v.is_empty() {
        return "**Guest Account** — _no Guest Account reading on this Mac right now._".to_string();
    }
    format!("**Guest Account** · `{v}`")
}

/// Automatic Login — loginwindow autoLoginUser (System Settings → Users & Groups).
/// Slash + view/open NL (`/autologin` · `/auto-login` · `view automatic login`).
/// Bare `autologin` / `automatic login` stay with the agent; `/guest` · `/guest-login` stay Guest Account;
/// `/username` · `/whoami` stay login name; `/remote-login` · `/ssh` stay Remote Login;
/// `/filevault` stays FileVault.
pub fn looks_like_autologin_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("password")
        || n.contains("screenshot")
        || n.contains("resolution")
        || n.contains("connect")
        || n.contains("tunnel")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("create")
        || n.contains("delete")
        || n.contains("remove")
        || n.contains("firewall")
        || n.contains("stealth")
        || n.contains("gatekeeper")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("file-vault")
        || n.contains("system integrity")
        || n.contains("time machine")
        || n.contains("timemachine")
        || n.contains("vpn")
        || n.contains("remote login")
        || n.contains("remotelogin")
        || n.contains("remote-login")
        || n.contains("screen sharing")
        || n.contains("screensharing")
        || n.contains("screen-sharing")
        || n.contains("file sharing")
        || n.contains("filesharing")
        || n.contains("file-sharing")
        || n.contains("airdrop")
        || n.contains("air drop")
        || n.contains("air-drop")
        || n.contains("airplay")
        || n.contains("guest account")
        || n.contains("guest user")
        || n.contains("guest-account")
        || n.contains("guest-user")
        || n.contains("guest-login")
        || n.contains("guestlogin")
        || n.contains("automatic time")
        || n.contains("automatic-time")
        || n.contains("screen lock")
        || n.contains("screen-lock")
        || n.contains("screenlock")
        || n.contains("lock screen")
        || n.contains("lock-screen")
        || n.contains("lockscreen")
        || n.contains("username")
        || n.contains("user name")
        || n.contains("whoami")
        || n.contains("who am i")
        || n == "/firewall"
        || n == "/fw"
        || n == "/afw"
        || n == "/stealth"
        || n == "/gatekeeper"
        || n == "/sip"
        || n == "/filevault"
        || n == "/file-vault"
        || n == "/timemachine"
        || n == "/tm"
        || n == "/vpn"
        || n == "/remote-login"
        || n == "/ssh"
        || n == "/screen-sharing"
        || n == "/vnc"
        || n == "/file-sharing"
        || n == "/smb"
        || n == "/airdrop"
        || n == "/air-drop"
        || n == "/airplay"
        || n == "/guest"
        || n == "/guest-account"
        || n == "/guestaccount"
        || n == "/guest-login"
        || n == "/guestlogin"
        || n == "/guest-user"
        || n == "/mac-guest"
        || n == "/screen-lock"
        || n == "/screenlock"
        || n == "/lock-screen"
        || n == "/lockscreen"
        || n == "/username"
        || n == "/whoami"
        || n == "/ip"
        || n == "/wifi"
        || n == "/tasks"
        || n == "/screen"
        || n == "/screens"
        || n == "/display"
        || n == "/displays"
        // Bare phrases stay with the agent.
        || n == "autologin"
        || n == "the autologin"
        || n == "auto login"
        || n == "the auto login"
        || n == "automatic login"
        || n == "the automatic login"
        || n == "auto-login"
        || n == "the auto-login"
        || n == "login"
        || n == "the login"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/autologin"
            | "/auto-login"
            | "/automatic-login"
            | "/automaticlogin"
            | "/autologin-status"
            | "/auto-login-status"
            | "/automatic-login-status"
            | "/autologinstatus"
            | "/autologin-state"
            | "/auto-login-state"
            | "/automatic-login-state"
            | "/mac-autologin"
            | "/mac-auto-login"
            | "/mac-automatic-login"
            | "autologin status"
            | "the autologin status"
            | "auto login status"
            | "the auto login status"
            | "automatic login status"
            | "the automatic login status"
            | "auto-login status"
            | "the auto-login status"
            | "autologin state"
            | "the autologin state"
            | "auto login state"
            | "the auto login state"
            | "automatic login state"
            | "the automatic login state"
            | "my autologin"
            | "my autologin status"
            | "my auto login"
            | "my auto login status"
            | "my automatic login"
            | "my automatic login status"
            | "is autologin on"
            | "is autologin off"
            | "is autologin enabled"
            | "is autologin disabled"
            | "is auto login on"
            | "is auto login off"
            | "is auto login enabled"
            | "is auto login disabled"
            | "is automatic login on"
            | "is automatic login off"
            | "is automatic login enabled"
            | "is automatic login disabled"
            | "is auto-login on"
            | "is auto-login enabled"
            | "what's my autologin"
            | "whats my autologin"
            | "what is my autologin"
            | "what's my autologin status"
            | "whats my autologin status"
            | "what is my autologin status"
            | "what's my auto login"
            | "whats my auto login"
            | "what is my auto login"
            | "what's my auto login status"
            | "whats my auto login status"
            | "what is my auto login status"
            | "what's my automatic login"
            | "whats my automatic login"
            | "what is my automatic login"
            | "what's my automatic login status"
            | "whats my automatic login status"
            | "what is my automatic login status"
            | "what's the autologin"
            | "whats the autologin"
            | "what is the autologin"
            | "what's the autologin status"
            | "whats the autologin status"
            | "what is the autologin status"
            | "what's the auto login"
            | "whats the auto login"
            | "what is the auto login"
            | "what's the auto login status"
            | "whats the auto login status"
            | "what is the auto login status"
            | "what's the automatic login"
            | "whats the automatic login"
            | "what is the automatic login"
            | "what's the automatic login status"
            | "whats the automatic login status"
            | "what is the automatic login status"
            | "how's autologin"
            | "hows autologin"
            | "how's the autologin"
            | "hows the autologin"
            | "how's automatic login"
            | "hows automatic login"
            | "how's the automatic login"
            | "hows the automatic login"
            | "view autologin"
            | "view the autologin"
            | "view autologin status"
            | "view the autologin status"
            | "view auto login"
            | "view the auto login"
            | "view auto login status"
            | "view the auto login status"
            | "view automatic login"
            | "view the automatic login"
            | "view automatic login status"
            | "view the automatic login status"
            | "view auto-login"
            | "view the auto-login"
            | "see autologin"
            | "see the autologin"
            | "see auto login"
            | "see the auto login"
            | "see automatic login"
            | "see the automatic login"
            | "see automatic login status"
            | "see the automatic login status"
            | "open autologin"
            | "open the autologin"
            | "open autologin status"
            | "open the autologin status"
            | "open auto login"
            | "open the auto login"
            | "open automatic login"
            | "open the automatic login"
            | "open automatic login status"
            | "open the automatic login status"
            | "list autologin"
            | "list the autologin"
            | "list autologin status"
            | "list the autologin status"
            | "list auto login"
            | "list the auto login"
            | "list automatic login"
            | "list the automatic login"
            | "list automatic login status"
            | "list the automatic login status"
    )
}

/// Zero-LLM Automatic Login status from loginwindow autoLoginUser (macOS-only; empty elsewhere).
pub fn format_autologin_report() -> String {
    let v = crate::metrics::get_autologin_status();
    let v = v.trim();
    if v.is_empty() {
        return "**Automatic Login** — _no Automatic Login reading on this Mac right now._".to_string();
    }
    format!("**Automatic Login** · `{v}`")
}

/// Screen Lock — screensaver askForPassword (+ delay) (System Settings → Lock Screen).
/// Slash + view/open NL (`/screen-lock` · `/lock-screen` · `view screen lock`).
/// Bare `screen lock` / `lock screen` / `lock` stay with the agent; `/screen-sharing` · `/vnc` stay
/// Screen Sharing; `/screen` · `/display` stay display resolution; `/autologin` stays Automatic Login.
pub fn looks_like_screen_lock_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("screenshot")
        || n.contains("resolution")
        || n.contains("connect")
        || n.contains("tunnel")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("create")
        || n.contains("delete")
        || n.contains("remove")
        || n.contains("firewall")
        || n.contains("stealth")
        || n.contains("gatekeeper")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("file-vault")
        || n.contains("system integrity")
        || n.contains("time machine")
        || n.contains("timemachine")
        || n.contains("vpn")
        || n.contains("remote login")
        || n.contains("remotelogin")
        || n.contains("remote-login")
        || n.contains("screen sharing")
        || n.contains("screensharing")
        || n.contains("screen-sharing")
        || n.contains("screen share")
        || n.contains("file sharing")
        || n.contains("filesharing")
        || n.contains("file-sharing")
        || n.contains("airdrop")
        || n.contains("air drop")
        || n.contains("air-drop")
        || n.contains("airplay")
        || n.contains("guest account")
        || n.contains("guest user")
        || n.contains("smb guest")
        || n.contains("smb-guest")
        || n.contains("smbguest")
        || n.contains("guest access")
        || n.contains("guest-access")
        || n.contains("automatic login")
        || n.contains("auto login")
        || n.contains("autologin")
        || n.contains("auto-login")
        || n.contains("username")
        || n.contains("whoami")
        || n == "/firewall"
        || n == "/fw"
        || n == "/afw"
        || n == "/stealth"
        || n == "/gatekeeper"
        || n == "/sip"
        || n == "/filevault"
        || n == "/file-vault"
        || n == "/timemachine"
        || n == "/tm"
        || n == "/vpn"
        || n == "/remote-login"
        || n == "/ssh"
        || n == "/screen-sharing"
        || n == "/screensharing"
        || n == "/vnc"
        || n == "/file-sharing"
        || n == "/smb"
        || n == "/smb-guest"
        || n == "/smbguest"
        || n == "/guest-access"
        || n == "/guestaccess"
        || n == "/airdrop"
        || n == "/air-drop"
        || n == "/airplay"
        || n == "/guest"
        || n == "/guest-account"
        || n == "/autologin"
        || n == "/auto-login"
        || n == "/automatic-login"
        || n == "/screensaver"
        || n == "/screen-saver"
        || n == "/screensaver-idle"
        || n == "/idle-time"
        || n == "/idletime"
        || n == "/mac-screensaver"
        || n == "/mac-idle-time"
        || n == "/username"
        || n == "/whoami"
        || n == "/ip"
        || n == "/wifi"
        || n == "/tasks"
        || n == "/screen"
        || n == "/screens"
        || n == "/display"
        || n == "/displays"
        // Bare phrases stay with the agent.
        || n == "screen lock"
        || n == "the screen lock"
        || n == "screen-lock"
        || n == "the screen-lock"
        || n == "screenlock"
        || n == "the screenlock"
        || n == "lock screen"
        || n == "the lock screen"
        || n == "lock-screen"
        || n == "the lock-screen"
        || n == "lockscreen"
        || n == "the lockscreen"
        || n == "lock"
        || n == "the lock"
        || n == "screensaver"
        || n == "the screensaver"
        || n == "screen saver"
        || n == "the screen saver"
        || n == "idle time"
        || n == "the idle time"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/screen-lock"
            | "/screenlock"
            | "/lock-screen"
            | "/lockscreen"
            | "/screen-lock-status"
            | "/screenlock-status"
            | "/lock-screen-status"
            | "/lockscreen-status"
            | "/screen-lock-state"
            | "/lock-screen-state"
            | "/mac-screen-lock"
            | "/mac-lock-screen"
            | "/mac-screenlock"
            | "/mac-lockscreen"
            | "screen lock status"
            | "the screen lock status"
            | "screen-lock status"
            | "the screen-lock status"
            | "screenlock status"
            | "the screenlock status"
            | "lock screen status"
            | "the lock screen status"
            | "lock-screen status"
            | "the lock-screen status"
            | "lockscreen status"
            | "the lockscreen status"
            | "screen lock state"
            | "the screen lock state"
            | "lock screen state"
            | "the lock screen state"
            | "my screen lock"
            | "my screen lock status"
            | "my lock screen"
            | "my lock screen status"
            | "is screen lock on"
            | "is screen lock off"
            | "is screen lock enabled"
            | "is screen lock disabled"
            | "is lock screen on"
            | "is lock screen off"
            | "is lock screen enabled"
            | "is lock screen disabled"
            | "is screen-lock on"
            | "is screen-lock enabled"
            | "is lock-screen on"
            | "is lock-screen enabled"
            | "what's my screen lock"
            | "whats my screen lock"
            | "what is my screen lock"
            | "what's my screen lock status"
            | "whats my screen lock status"
            | "what is my screen lock status"
            | "what's my lock screen"
            | "whats my lock screen"
            | "what is my lock screen"
            | "what's my lock screen status"
            | "whats my lock screen status"
            | "what is my lock screen status"
            | "what's the screen lock"
            | "whats the screen lock"
            | "what is the screen lock"
            | "what's the screen lock status"
            | "whats the screen lock status"
            | "what is the screen lock status"
            | "what's the lock screen"
            | "whats the lock screen"
            | "what is the lock screen"
            | "what's the lock screen status"
            | "whats the lock screen status"
            | "what is the lock screen status"
            | "how's screen lock"
            | "hows screen lock"
            | "how's the screen lock"
            | "hows the screen lock"
            | "how's lock screen"
            | "hows lock screen"
            | "how's the lock screen"
            | "hows the lock screen"
            | "view screen lock"
            | "view the screen lock"
            | "view screen lock status"
            | "view the screen lock status"
            | "view screen-lock"
            | "view the screen-lock"
            | "view lock screen"
            | "view the lock screen"
            | "view lock screen status"
            | "view the lock screen status"
            | "view lock-screen"
            | "view the lock-screen"
            | "see screen lock"
            | "see the screen lock"
            | "see screen lock status"
            | "see the screen lock status"
            | "see lock screen"
            | "see the lock screen"
            | "see lock screen status"
            | "see the lock screen status"
            | "open screen lock"
            | "open the screen lock"
            | "open screen lock status"
            | "open the screen lock status"
            | "open lock screen"
            | "open the lock screen"
            | "open lock screen status"
            | "open the lock screen status"
            | "list screen lock"
            | "list the screen lock"
            | "list screen lock status"
            | "list the screen lock status"
            | "list lock screen"
            | "list the lock screen"
            | "list lock screen status"
            | "list the lock screen status"
    )
}

/// Zero-LLM Screen Lock status from screensaver askForPassword (macOS-only; empty elsewhere).
pub fn format_screen_lock_report() -> String {
    let v = crate::metrics::get_screen_lock_status();
    let v = v.trim();
    if v.is_empty() {
        return "**Screen Lock** — _no Screen Lock reading on this Mac right now._".to_string();
    }
    format!("**Screen Lock** · `{v}`")
}

/// SMB Guest Access — `sysadminctl -smbGuestAccess` (+ smb.server AllowGuestAccess).
/// Slash + view/open NL (`/smb-guest` · `/guest-access` · `view smb guest`).
/// Bare `smb guest` / `guest access` stay with the agent; `/smb` · `/file-sharing` stay File Sharing;
/// `/guest` · `/guest-account` stay Guest Account; `/screen-lock` stays Screen Lock.
pub fn looks_like_smb_guest_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("password")
        || n.contains("screenshot")
        || n.contains("resolution")
        || n.contains("connect")
        || n.contains("tunnel")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("create")
        || n.contains("delete")
        || n.contains("remove")
        || n.contains("firewall")
        || n.contains("stealth")
        || n.contains("gatekeeper")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("file-vault")
        || n.contains("system integrity")
        || n.contains("time machine")
        || n.contains("timemachine")
        || n.contains("vpn")
        || n.contains("remote login")
        || n.contains("remotelogin")
        || n.contains("remote-login")
        || n.contains("screen sharing")
        || n.contains("screensharing")
        || n.contains("screen-sharing")
        || n.contains("screen lock")
        || n.contains("screen-lock")
        || n.contains("screenlock")
        || n.contains("lock screen")
        || n.contains("airdrop")
        || n.contains("air drop")
        || n.contains("air-drop")
        || n.contains("airplay")
        || n.contains("automatic login")
        || n.contains("auto login")
        || n.contains("autologin")
        || n.contains("auto-login")
        || n.contains("username")
        || n.contains("whoami")
        || n.contains("afp guest")
        || n.contains("afp-guest")
        || n == "/firewall"
        || n == "/fw"
        || n == "/afw"
        || n == "/stealth"
        || n == "/gatekeeper"
        || n == "/sip"
        || n == "/filevault"
        || n == "/file-vault"
        || n == "/timemachine"
        || n == "/tm"
        || n == "/vpn"
        || n == "/remote-login"
        || n == "/ssh"
        || n == "/screen-sharing"
        || n == "/vnc"
        || n == "/file-sharing"
        || n == "/filesharing"
        || n == "/smb"
        || n == "/smb-status"
        || n == "/smbd"
        || n == "/airdrop"
        || n == "/air-drop"
        || n == "/airplay"
        || n == "/guest"
        || n == "/guest-account"
        || n == "/guest-user"
        || n == "/guest-login"
        || n == "/mac-guest"
        || n == "/autologin"
        || n == "/auto-login"
        || n == "/automatic-login"
        || n == "/screen-lock"
        || n == "/lock-screen"
        || n == "/screensaver"
        || n == "/screen-saver"
        || n == "/screensaver-idle"
        || n == "/idle-time"
        || n == "/idletime"
        || n == "/mac-screensaver"
        || n == "/mac-idle-time"
        || n == "/username"
        || n == "/whoami"
        || n == "/ip"
        || n == "/wifi"
        || n == "/tasks"
        || n == "/screen"
        || n == "/screens"
        || n == "/display"
        || n == "/displays"
        // Bare phrases stay with the agent.
        || n == "smb guest"
        || n == "the smb guest"
        || n == "smb-guest"
        || n == "the smb-guest"
        || n == "smbguest"
        || n == "the smbguest"
        || n == "guest access"
        || n == "the guest access"
        || n == "smb guest access"
        || n == "the smb guest access"
        || n == "guest-access"
        || n == "the guest-access"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/smb-guest"
            | "/smbguest"
            | "/smb-guest-access"
            | "/smbguestaccess"
            | "/smb-guest-status"
            | "/smbguest-status"
            | "/smb-guest-state"
            | "/smbguest-state"
            | "/guest-access"
            | "/guestaccess"
            | "/guest-access-status"
            | "/guestaccess-status"
            | "/smb-guest-access-status"
            | "/mac-smb-guest"
            | "/mac-smbguest"
            | "/mac-guest-access"
            | "/mac-smb-guest-access"
            | "smb guest status"
            | "the smb guest status"
            | "smb-guest status"
            | "the smb-guest status"
            | "smbguest status"
            | "the smbguest status"
            | "smb guest access status"
            | "the smb guest access status"
            | "smb-guest-access status"
            | "the smb-guest-access status"
            | "guest access status"
            | "the guest access status"
            | "guest-access status"
            | "the guest-access status"
            | "smb guest state"
            | "the smb guest state"
            | "guest access state"
            | "the guest access state"
            | "my smb guest"
            | "my smb guest status"
            | "my smb guest access"
            | "my smb guest access status"
            | "my guest access"
            | "my guest access status"
            | "is smb guest on"
            | "is smb guest off"
            | "is smb guest enabled"
            | "is smb guest disabled"
            | "is smb guest access on"
            | "is smb guest access off"
            | "is smb guest access enabled"
            | "is smb guest access disabled"
            | "is guest access on"
            | "is guest access off"
            | "is guest access enabled"
            | "is guest access disabled"
            | "is smb-guest on"
            | "is smb-guest enabled"
            | "is guest-access on"
            | "is guest-access enabled"
            | "what's my smb guest"
            | "whats my smb guest"
            | "what is my smb guest"
            | "what's my smb guest status"
            | "whats my smb guest status"
            | "what is my smb guest status"
            | "what's my smb guest access"
            | "whats my smb guest access"
            | "what is my smb guest access"
            | "what's my smb guest access status"
            | "whats my smb guest access status"
            | "what is my smb guest access status"
            | "what's my guest access"
            | "whats my guest access"
            | "what is my guest access"
            | "what's my guest access status"
            | "whats my guest access status"
            | "what is my guest access status"
            | "what's the smb guest"
            | "whats the smb guest"
            | "what is the smb guest"
            | "what's the smb guest status"
            | "whats the smb guest status"
            | "what is the smb guest status"
            | "what's the smb guest access"
            | "whats the smb guest access"
            | "what is the smb guest access"
            | "what's the smb guest access status"
            | "whats the smb guest access status"
            | "what is the smb guest access status"
            | "what's the guest access"
            | "whats the guest access"
            | "what is the guest access"
            | "what's the guest access status"
            | "whats the guest access status"
            | "what is the guest access status"
            | "how's smb guest"
            | "hows smb guest"
            | "how's the smb guest"
            | "hows the smb guest"
            | "how's smb guest access"
            | "hows smb guest access"
            | "how's guest access"
            | "hows guest access"
            | "view smb guest"
            | "view the smb guest"
            | "view smb guest status"
            | "view the smb guest status"
            | "view smb guest access"
            | "view the smb guest access"
            | "view smb guest access status"
            | "view the smb guest access status"
            | "view smb-guest"
            | "view the smb-guest"
            | "view guest access"
            | "view the guest access"
            | "view guest access status"
            | "view the guest access status"
            | "view guest-access"
            | "view the guest-access"
            | "see smb guest"
            | "see the smb guest"
            | "see smb guest status"
            | "see the smb guest status"
            | "see smb guest access"
            | "see the smb guest access"
            | "see guest access"
            | "see the guest access"
            | "see guest access status"
            | "see the guest access status"
            | "open smb guest"
            | "open the smb guest"
            | "open smb guest status"
            | "open the smb guest status"
            | "open smb guest access"
            | "open the smb guest access"
            | "open smb-guest"
            | "open the smb-guest"
            | "open guest access"
            | "open the guest access"
            | "open guest access status"
            | "open the guest access status"
            | "list smb guest"
            | "list the smb guest"
            | "list smb guest status"
            | "list the smb guest status"
            | "list smb guest access"
            | "list the smb guest access"
            | "list guest access"
            | "list the guest access"
            | "list guest access status"
            | "list the guest access status"
    )
}

/// Zero-LLM SMB Guest Access status from sysadminctl / smb.server prefs (macOS-only; empty elsewhere).
pub fn format_smb_guest_report() -> String {
    let v = crate::metrics::get_smb_guest_status();
    let v = v.trim();
    if v.is_empty() {
        return "**SMB Guest** — _no SMB guest access reading on this Mac right now._".to_string();
    }
    format!("**SMB Guest** · `{v}`")
}

/// Screensaver idle — `com.apple.screensaver` `idleTime` (seconds; 0 = Never).
/// Slash + view/open NL (`/screensaver` · `/screensaver-idle` · `/idle-time` · `view screensaver`).
/// Bare `screensaver` / `screen saver` / `idle time` stay with the agent; `/screen-lock` · `/lock-screen`
/// stay Screen Lock; `/screen-sharing` · `/vnc` stay Screen Sharing; `/screen` · `/display` stay display.
pub fn looks_like_screensaver_idle_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("password")
        || n.contains("ask for password")
        || n.contains("screenshot")
        || n.contains("resolution")
        || n.contains("connect")
        || n.contains("tunnel")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("create")
        || n.contains("delete")
        || n.contains("remove")
        || n.contains("firewall")
        || n.contains("stealth")
        || n.contains("gatekeeper")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("file-vault")
        || n.contains("system integrity")
        || n.contains("time machine")
        || n.contains("timemachine")
        || n.contains("vpn")
        || n.contains("remote login")
        || n.contains("remotelogin")
        || n.contains("remote-login")
        || n.contains("screen sharing")
        || n.contains("screensharing")
        || n.contains("screen-sharing")
        || n.contains("screen lock")
        || n.contains("screen-lock")
        || n.contains("screenlock")
        || n.contains("lock screen")
        || n.contains("lock-screen")
        || n.contains("lockscreen")
        || n.contains("airdrop")
        || n.contains("air drop")
        || n.contains("air-drop")
        || n.contains("airplay")
        || n.contains("automatic login")
        || n.contains("auto login")
        || n.contains("autologin")
        || n.contains("auto-login")
        || n.contains("smb guest")
        || n.contains("guest access")
        || n.contains("guest account")
        || n == "/firewall"
        || n == "/fw"
        || n == "/afw"
        || n == "/stealth"
        || n == "/gatekeeper"
        || n == "/sip"
        || n == "/filevault"
        || n == "/file-vault"
        || n == "/timemachine"
        || n == "/tm"
        || n == "/vpn"
        || n == "/remote-login"
        || n == "/ssh"
        || n == "/screen-sharing"
        || n == "/vnc"
        || n == "/file-sharing"
        || n == "/smb"
        || n == "/smb-guest"
        || n == "/guest-access"
        || n == "/guest"
        || n == "/guest-account"
        || n == "/airdrop"
        || n == "/air-drop"
        || n == "/airplay"
        || n == "/autologin"
        || n == "/auto-login"
        || n == "/automatic-login"
        || n == "/screen-lock"
        || n == "/lock-screen"
        || n == "/screenlock"
        || n == "/lockscreen"
        || n == "/mac-screen-lock"
        || n == "/mac-lock-screen"
        || n == "/ip"
        || n == "/wifi"
        || n == "/tasks"
        || n == "/screen"
        || n == "/screens"
        || n == "/display"
        || n == "/displays"
        || n.contains("displaysleep")
        || n.contains("display sleep")
        || n.contains("display-sleep")
        || n.contains("screen sleep")
        || n.contains("screen-sleep")
        || n.contains("disksleep")
        || n.contains("disk sleep")
        || n == "/displaysleep"
        || n == "/display-sleep"
        || n == "/screen-sleep"
        || n == "/powernap"
        || n == "/power-nap"
        || n == "/womp"
        // Bare phrases stay with the agent.
        || n == "screensaver"
        || n == "the screensaver"
        || n == "screen saver"
        || n == "the screen saver"
        || n == "screensaver idle"
        || n == "the screensaver idle"
        || n == "screen saver idle"
        || n == "the screen saver idle"
        || n == "idle time"
        || n == "the idle time"
        || n == "idle"
        || n == "the idle"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/screensaver"
            | "/screen-saver"
            | "/screensaver-idle"
            | "/screensaveridle"
            | "/screen-saver-idle"
            | "/screensaver-idle-status"
            | "/screensaver-idle-state"
            | "/screensaver-status"
            | "/screensaver-state"
            | "/screen-saver-status"
            | "/idle-time"
            | "/idletime"
            | "/idle-time-status"
            | "/screensaver-time"
            | "/screen-saver-time"
            | "/mac-screensaver"
            | "/mac-screen-saver"
            | "/mac-screensaver-idle"
            | "/mac-idle-time"
            | "screensaver status"
            | "the screensaver status"
            | "screen saver status"
            | "the screen saver status"
            | "screensaver idle status"
            | "the screensaver idle status"
            | "screen saver idle status"
            | "the screen saver idle status"
            | "idle time status"
            | "the idle time status"
            | "screensaver state"
            | "the screensaver state"
            | "idle time state"
            | "the idle time state"
            | "my screensaver"
            | "my screensaver status"
            | "my screensaver idle"
            | "my screensaver idle status"
            | "my screen saver"
            | "my screen saver status"
            | "my idle time"
            | "my idle time status"
            | "is screensaver on"
            | "is screensaver off"
            | "is screensaver enabled"
            | "is screensaver disabled"
            | "is screen saver on"
            | "is screen saver off"
            | "is screen saver enabled"
            | "is screen saver disabled"
            | "is screensaver idle on"
            | "is screensaver idle enabled"
            | "what's my screensaver"
            | "whats my screensaver"
            | "what is my screensaver"
            | "what's my screensaver status"
            | "whats my screensaver status"
            | "what is my screensaver status"
            | "what's my screensaver idle"
            | "whats my screensaver idle"
            | "what is my screensaver idle"
            | "what's my screensaver idle status"
            | "whats my screensaver idle status"
            | "what is my screensaver idle status"
            | "what's my screen saver"
            | "whats my screen saver"
            | "what is my screen saver"
            | "what's my screen saver status"
            | "whats my screen saver status"
            | "what is my screen saver status"
            | "what's my idle time"
            | "whats my idle time"
            | "what is my idle time"
            | "what's my idle time status"
            | "whats my idle time status"
            | "what is my idle time status"
            | "what's the screensaver"
            | "whats the screensaver"
            | "what is the screensaver"
            | "what's the screensaver status"
            | "whats the screensaver status"
            | "what is the screensaver status"
            | "what's the screensaver idle"
            | "whats the screensaver idle"
            | "what is the screensaver idle"
            | "what's the screensaver idle status"
            | "whats the screensaver idle status"
            | "what is the screensaver idle status"
            | "what's the screen saver"
            | "whats the screen saver"
            | "what is the screen saver"
            | "what's the screen saver status"
            | "whats the screen saver status"
            | "what is the screen saver status"
            | "what's the idle time"
            | "whats the idle time"
            | "what is the idle time"
            | "what's the idle time status"
            | "whats the idle time status"
            | "what is the idle time status"
            | "view screensaver"
            | "view the screensaver"
            | "view screensaver status"
            | "view the screensaver status"
            | "view screensaver idle"
            | "view the screensaver idle"
            | "view screensaver idle status"
            | "view the screensaver idle status"
            | "view screen saver"
            | "view the screen saver"
            | "view screen saver status"
            | "view the screen saver status"
            | "view idle time"
            | "view the idle time"
            | "view idle time status"
            | "view the idle time status"
            | "see screensaver"
            | "see the screensaver"
            | "see screensaver status"
            | "see the screensaver status"
            | "see screensaver idle"
            | "see screen saver"
            | "see idle time"
            | "show me the screensaver"
            | "show me the screensaver status"
            | "show me the screensaver idle"
            | "show me the screensaver idle status"
            | "show me the screen saver"
            | "show me the screen saver status"
            | "show me the idle time"
            | "show me the idle time status"
            | "open screensaver"
            | "open the screensaver"
            | "open screensaver status"
            | "open the screensaver status"
            | "open screensaver idle"
            | "open the screensaver idle"
            | "open screen saver"
            | "open the screen saver"
            | "open idle time"
            | "open the idle time"
            | "list screensaver"
            | "list the screensaver"
            | "list screensaver status"
            | "list the screensaver status"
            | "list screensaver idle"
            | "list the screensaver idle"
            | "list screen saver"
            | "list the screen saver"
            | "list idle time"
            | "list the idle time"
            | "list idle time status"
            | "list the idle time status"
    )
}

/// Zero-LLM Screensaver idle status from screensaver idleTime (macOS-only; empty elsewhere).
pub fn format_screensaver_idle_report() -> String {
    let v = crate::metrics::get_screensaver_idle_status();
    let v = v.trim();
    if v.is_empty() {
        return "**Screensaver** — _no Screensaver idle reading on this Mac right now._".to_string();
    }
    format!("**Screensaver** · `{v}`")
}

/// Power Nap — `pmset` Battery/AC `powernap` (+ active fallback).
/// Slash + view/open NL (`/powernap` · `/power-nap` · `view power nap`).
/// Bare `powernap` / `power nap` stay with the agent; `/lpm` stays Low Power Mode;
/// `/power` stays the strip; `/watts` stays draw; `/screensaver` stays Screensaver idle;
/// Wake for network `/womp` · `/wake-for-network` stays its own lane.
pub fn looks_like_power_nap_request(content: &str) -> bool {
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
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("password")
        || n.contains("screenshot")
        || n.contains("resolution")
        || n.contains("connect")
        || n.contains("tunnel")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("create")
        || n.contains("delete")
        || n.contains("remove")
        || n.contains("low power")
        || n.contains("low-power")
        || n.contains("lowpower")
        || n.contains("wake for")
        || n.contains("wake-for")
        || n.contains("wake on")
        || n.contains("wake-on")
        || n.contains("womp")
        || n.contains("displaysleep")
        || n.contains("display sleep")
        || n.contains("display-sleep")
        || n.contains("screen sleep")
        || n.contains("screen-sleep")
        || n.contains("disksleep")
        || n.contains("disk sleep")
        || n.contains("screensaver")
        || n.contains("screen saver")
        || n.contains("screen-saver")
        || n.contains("idle time")
        || n.contains("idle-time")
        || n.contains("screen lock")
        || n.contains("screen-lock")
        || n.contains("lock screen")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("battery time")
        || n.contains("time remaining")
        || n.contains("cycle")
        || n.contains("charger")
        || n.contains("adapter")
        || n.contains("watt")
        || n == "/lpm"
        || n == "/power"
        || n == "/watts"
        || n == "/power-draw"
        || n == "/battery"
        || n == "/bat"
        || n == "/battery-time"
        || n == "/cycles"
        || n == "/charger"
        || n == "/adapter"
        || n == "/displaysleep"
        || n == "/display-sleep"
        || n == "/screen-sleep"
        || n == "/screensaver"
        || n == "/screen-saver"
        || n == "/idle-time"
        || n == "/screen-lock"
        || n == "/lock-screen"
        || n == "/filevault"
        || n == "/tasks"
        || n == "/uptime"
        // Bare phrases stay with the agent.
        || n == "powernap"
        || n == "the powernap"
        || n == "power nap"
        || n == "the power nap"
        || n == "power-nap"
        || n == "the power-nap"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/powernap"
            | "/power-nap"
            | "/powernap-status"
            | "/power-nap-status"
            | "/powernap-state"
            | "/power-nap-state"
            | "/mac-powernap"
            | "/mac-power-nap"
            | "/mac-powernap-status"
            | "/mac-power-nap-status"
            | "powernap status"
            | "the powernap status"
            | "power nap status"
            | "the power nap status"
            | "power-nap status"
            | "the power-nap status"
            | "powernap state"
            | "the powernap state"
            | "power nap state"
            | "the power nap state"
            | "my powernap"
            | "my powernap status"
            | "my power nap"
            | "my power nap status"
            | "is powernap on"
            | "is powernap off"
            | "is powernap enabled"
            | "is powernap disabled"
            | "is power nap on"
            | "is power nap off"
            | "is power nap enabled"
            | "is power nap disabled"
            | "is power-nap on"
            | "is power-nap enabled"
            | "what's my powernap"
            | "whats my powernap"
            | "what is my powernap"
            | "what's my powernap status"
            | "whats my powernap status"
            | "what is my powernap status"
            | "what's my power nap"
            | "whats my power nap"
            | "what is my power nap"
            | "what's my power nap status"
            | "whats my power nap status"
            | "what is my power nap status"
            | "what's the powernap"
            | "whats the powernap"
            | "what is the powernap"
            | "what's the powernap status"
            | "whats the powernap status"
            | "what is the powernap status"
            | "what's the power nap"
            | "whats the power nap"
            | "what is the power nap"
            | "what's the power nap status"
            | "whats the power nap status"
            | "what is the power nap status"
            | "how's powernap"
            | "hows powernap"
            | "how's the powernap"
            | "hows the powernap"
            | "how's power nap"
            | "hows power nap"
            | "how's the power nap"
            | "hows the power nap"
            | "view powernap"
            | "view the powernap"
            | "view powernap status"
            | "view the powernap status"
            | "view power nap"
            | "view the power nap"
            | "view power nap status"
            | "view the power nap status"
            | "view power-nap"
            | "view the power-nap"
            | "see powernap"
            | "see the powernap"
            | "see powernap status"
            | "see the powernap status"
            | "see power nap"
            | "see the power nap"
            | "see power nap status"
            | "see the power nap status"
            | "open powernap"
            | "open the powernap"
            | "open powernap status"
            | "open the powernap status"
            | "open power nap"
            | "open the power nap"
            | "open power nap status"
            | "open the power nap status"
            | "open power-nap"
            | "open the power-nap"
            | "list powernap"
            | "list the powernap"
            | "list powernap status"
            | "list the powernap status"
            | "list power nap"
            | "list the power nap"
            | "list power nap status"
            | "list the power nap status"
    )
}

/// Zero-LLM Power Nap status from pmset Battery/AC (macOS-only; empty elsewhere).
pub fn format_power_nap_report() -> String {
    let v = crate::metrics::get_power_nap_status();
    let v = v.trim();
    if v.is_empty() {
        return "**Power Nap** — _no Power Nap reading on this Mac right now._".to_string();
    }
    format!("**Power Nap** · `{v}`")
}

/// Wake for network access — `pmset` Battery/AC `womp` (+ active fallback).
/// Slash + view/open NL (`/womp` · `/wake-for-network` · `view wake for network`).
/// Bare `womp` / `wake for network` / `wake on lan` / `wol` stay with the agent;
/// `/powernap` stays Power Nap; `/lpm` stays Low Power Mode; `/power` stays the strip;
/// `/wifi` stays SSID; `/vpn` stays VPN.
pub fn looks_like_wake_for_network_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("password")
        || n.contains("screenshot")
        || n.contains("resolution")
        || n.contains("connect")
        || n.contains("tunnel")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("create")
        || n.contains("delete")
        || n.contains("remove")
        || n.contains("low power")
        || n.contains("low-power")
        || n.contains("lowpower")
        || n.contains("powernap")
        || n.contains("power nap")
        || n.contains("power-nap")
        || n.contains("displaysleep")
        || n.contains("display sleep")
        || n.contains("display-sleep")
        || n.contains("screen sleep")
        || n.contains("screen-sleep")
        || n.contains("disksleep")
        || n.contains("disk sleep")
        || n.contains("screensaver")
        || n.contains("screen saver")
        || n.contains("screen-saver")
        || n.contains("idle time")
        || n.contains("idle-time")
        || n.contains("screen lock")
        || n.contains("screen-lock")
        || n.contains("lock screen")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("battery time")
        || n.contains("time remaining")
        || n.contains("cycle")
        || n.contains("charger")
        || n.contains("adapter")
        || n.contains("watt")
        || n.contains("networkoversleep")
        || n.contains("ttyskeepawake")
        || n.contains("tcpkeepalive")
        || n.contains("proximitywake")
        || n.contains("proximity wake")
        || n.contains("proximity-wake")
        || n == "/lpm"
        || n == "/power"
        || n == "/watts"
        || n == "/power-draw"
        || n == "/battery"
        || n == "/bat"
        || n == "/battery-time"
        || n == "/cycles"
        || n == "/charger"
        || n == "/adapter"
        || n == "/powernap"
        || n == "/power-nap"
        || n == "/displaysleep"
        || n == "/display-sleep"
        || n == "/screen-sleep"
        || n == "/screensaver"
        || n == "/screen-saver"
        || n == "/idle-time"
        || n == "/screen-lock"
        || n == "/lock-screen"
        || n == "/filevault"
        || n == "/proximitywake"
        || n == "/proximity-wake"
        || n == "/wifi"
        || n == "/ssid"
        || n == "/vpn"
        || n == "/tasks"
        || n == "/uptime"
        // Bare phrases stay with the agent.
        || n == "womp"
        || n == "the womp"
        || n == "wol"
        || n == "the wol"
        || n == "wake on lan"
        || n == "the wake on lan"
        || n == "wake-on-lan"
        || n == "the wake-on-lan"
        || n == "wake on network"
        || n == "the wake on network"
        || n == "wake-on-network"
        || n == "the wake-on-network"
        || n == "wake for network"
        || n == "the wake for network"
        || n == "wake-for-network"
        || n == "the wake-for-network"
        || n == "wake for network access"
        || n == "the wake for network access"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/womp"
            | "/womp-status"
            | "/womp-state"
            | "/wake-for-network"
            | "/wake-for-network-status"
            | "/wake-for-network-state"
            | "/wake-on-lan"
            | "/wake-on-lan-status"
            | "/wake-on-lan-state"
            | "/wake-on-network"
            | "/wake-on-network-status"
            | "/wol"
            | "/wol-status"
            | "/wol-state"
            | "/mac-womp"
            | "/mac-womp-status"
            | "/mac-wake-for-network"
            | "/mac-wake-on-lan"
            | "/mac-wol"
            | "womp status"
            | "the womp status"
            | "womp state"
            | "the womp state"
            | "my womp"
            | "my womp status"
            | "is womp on"
            | "is womp off"
            | "is womp enabled"
            | "is womp disabled"
            | "what's my womp"
            | "whats my womp"
            | "what is my womp"
            | "what's my womp status"
            | "whats my womp status"
            | "what is my womp status"
            | "what's the womp"
            | "whats the womp"
            | "what is the womp"
            | "what's the womp status"
            | "whats the womp status"
            | "what is the womp status"
            | "how's womp"
            | "hows womp"
            | "how's the womp"
            | "hows the womp"
            | "view womp"
            | "view the womp"
            | "view womp status"
            | "view the womp status"
            | "see womp"
            | "see the womp"
            | "see womp status"
            | "see the womp status"
            | "open womp"
            | "open the womp"
            | "open womp status"
            | "open the womp status"
            | "list womp"
            | "list the womp"
            | "list womp status"
            | "list the womp status"
            | "wake for network status"
            | "the wake for network status"
            | "wake-for-network status"
            | "the wake-for-network status"
            | "wake for network state"
            | "the wake for network state"
            | "wake for network access status"
            | "the wake for network access status"
            | "my wake for network"
            | "my wake for network status"
            | "my wake-for-network"
            | "is wake for network on"
            | "is wake for network off"
            | "is wake for network enabled"
            | "is wake for network disabled"
            | "is wake-for-network on"
            | "is wake-for-network enabled"
            | "is wake for network access on"
            | "is wake for network access enabled"
            | "what's my wake for network"
            | "whats my wake for network"
            | "what is my wake for network"
            | "what's my wake for network status"
            | "whats my wake for network status"
            | "what is my wake for network status"
            | "what's the wake for network"
            | "whats the wake for network"
            | "what is the wake for network"
            | "what's the wake for network status"
            | "whats the wake for network status"
            | "what is the wake for network status"
            | "how's wake for network"
            | "hows wake for network"
            | "how's the wake for network"
            | "hows the wake for network"
            | "view wake for network"
            | "view the wake for network"
            | "view wake for network status"
            | "view the wake for network status"
            | "view wake-for-network"
            | "view the wake-for-network"
            | "view wake for network access"
            | "view the wake for network access"
            | "see wake for network"
            | "see the wake for network"
            | "see wake for network status"
            | "see the wake for network status"
            | "see wake-for-network"
            | "see the wake-for-network"
            | "open wake for network"
            | "open the wake for network"
            | "open wake for network status"
            | "open the wake for network status"
            | "open wake-for-network"
            | "open the wake-for-network"
            | "list wake for network"
            | "list the wake for network"
            | "list wake for network status"
            | "list the wake for network status"
            | "list wake-for-network"
            | "list the wake-for-network"
            | "wake on lan status"
            | "the wake on lan status"
            | "wake-on-lan status"
            | "the wake-on-lan status"
            | "wake on lan state"
            | "the wake on lan state"
            | "my wake on lan"
            | "my wake on lan status"
            | "is wake on lan on"
            | "is wake on lan off"
            | "is wake on lan enabled"
            | "is wake on lan disabled"
            | "is wake-on-lan on"
            | "is wake-on-lan enabled"
            | "what's my wake on lan"
            | "whats my wake on lan"
            | "what is my wake on lan"
            | "what's my wake on lan status"
            | "whats my wake on lan status"
            | "what is my wake on lan status"
            | "what's the wake on lan"
            | "whats the wake on lan"
            | "what is the wake on lan"
            | "what's the wake on lan status"
            | "whats the wake on lan status"
            | "what is the wake on lan status"
            | "view wake on lan"
            | "view the wake on lan"
            | "view wake on lan status"
            | "view the wake on lan status"
            | "view wake-on-lan"
            | "view the wake-on-lan"
            | "see wake on lan"
            | "see the wake on lan"
            | "see wake on lan status"
            | "see the wake on lan status"
            | "open wake on lan"
            | "open the wake on lan"
            | "open wake on lan status"
            | "open the wake on lan status"
            | "open wake-on-lan"
            | "open the wake-on-lan"
            | "list wake on lan"
            | "list the wake on lan"
            | "list wake on lan status"
            | "list the wake on lan status"
            | "wol status"
            | "the wol status"
            | "wol state"
            | "the wol state"
            | "my wol"
            | "my wol status"
            | "is wol on"
            | "is wol off"
            | "is wol enabled"
            | "is wol disabled"
            | "what's my wol"
            | "whats my wol"
            | "what is my wol"
            | "what's my wol status"
            | "whats my wol status"
            | "what is my wol status"
            | "what's the wol"
            | "whats the wol"
            | "what is the wol"
            | "what's the wol status"
            | "whats the wol status"
            | "what is the wol status"
            | "view wol"
            | "view the wol"
            | "view wol status"
            | "view the wol status"
            | "see wol"
            | "see the wol"
            | "see wol status"
            | "see the wol status"
            | "open wol"
            | "open the wol"
            | "open wol status"
            | "open the wol status"
            | "list wol"
            | "list the wol"
            | "list wol status"
            | "list the wol status"
            | "wake on network status"
            | "the wake on network status"
            | "wake-on-network status"
            | "the wake-on-network status"
            | "is wake on network on"
            | "is wake on network enabled"
            | "is wake-on-network on"
            | "is wake-on-network enabled"
            | "view wake on network"
            | "view the wake on network"
            | "view wake-on-network"
            | "view the wake-on-network"
            | "see wake on network"
            | "see the wake on network"
            | "open wake on network"
            | "open the wake on network"
            | "open wake-on-network"
            | "open the wake-on-network"
            | "list wake on network"
            | "list the wake on network"
            | "list wake-on-network"
            | "list the wake-on-network"
    )
}

/// Zero-LLM Wake for network status from pmset Battery/AC `womp` (macOS-only; empty elsewhere).
pub fn format_wake_for_network_report() -> String {
    let v = crate::metrics::get_wake_for_network_status();
    let v = v.trim();
    if v.is_empty() {
        return "**Wake for network** — _no Wake for network reading on this Mac right now._"
            .to_string();
    }
    format!("**Wake for network** · `{v}`")
}

/// Display Sleep — `pmset` Battery/AC `displaysleep` minutes (+ active fallback; 0 = Never).
/// Slash + view/open NL (`/displaysleep` · `/display-sleep` · `view display sleep`).
/// Bare `displaysleep` / `display sleep` / `screen sleep` stay with the agent;
/// `/screensaver` · `/idle-time` stay Screensaver idle; `/display` · `/screen` stay resolution;
/// `/powernap` stays Power Nap; `/womp` stays Wake for network; disksleep stays out.
pub fn looks_like_displaysleep_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("password")
        || n.contains("screenshot")
        || n.contains("resolution")
        || n.contains("connect")
        || n.contains("tunnel")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("create")
        || n.contains("delete")
        || n.contains("remove")
        || n.contains("disk sleep")
        || n.contains("disk-sleep")
        || n.contains("disksleep")
        || n.contains("system sleep")
        || n.contains("system-sleep")
        || n.contains("systemsleep")
        || n.contains("computer sleep")
        || n.contains("computer-sleep")
        || n.contains("computersleep")
        || n.contains("low power")
        || n.contains("low-power")
        || n.contains("lowpower")
        || n.contains("powernap")
        || n.contains("power nap")
        || n.contains("power-nap")
        || n.contains("wake for")
        || n.contains("wake-for")
        || n.contains("wake on")
        || n.contains("wake-on")
        || n.contains("womp")
        || n.contains("proximity")
        || n.contains("screensaver")
        || n.contains("screen saver")
        || n.contains("screen-saver")
        || n.contains("idle time")
        || n.contains("idle-time")
        || n.contains("screen lock")
        || n.contains("screen-lock")
        || n.contains("lock screen")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("battery time")
        || n.contains("time remaining")
        || n.contains("cycle")
        || n.contains("charger")
        || n.contains("adapter")
        || n.contains("watt")
        || n == "/lpm"
        || n == "/power"
        || n == "/watts"
        || n == "/power-draw"
        || n == "/battery"
        || n == "/bat"
        || n == "/battery-time"
        || n == "/cycles"
        || n == "/charger"
        || n == "/adapter"
        || n == "/powernap"
        || n == "/power-nap"
        || n == "/womp"
        || n == "/wake-for-network"
        || n == "/wake-on-lan"
        || n == "/wol"
        || n == "/systemsleep"
        || n == "/system-sleep"
        || n == "/computer-sleep"
        || n == "/computersleep"
        || n == "/sleep"
        || n == "/screensaver"
        || n == "/screen-saver"
        || n == "/idle-time"
        || n == "/screen-lock"
        || n == "/lock-screen"
        || n == "/filevault"
        || n == "/display"
        || n == "/displays"
        || n == "/screen"
        || n == "/screens"
        || n == "/resolution"
        || n == "/wifi"
        || n == "/ssid"
        || n == "/vpn"
        || n == "/tasks"
        || n == "/uptime"
        // Bare phrases stay with the agent.
        || n == "displaysleep"
        || n == "the displaysleep"
        || n == "display sleep"
        || n == "the display sleep"
        || n == "display-sleep"
        || n == "the display-sleep"
        || n == "screen sleep"
        || n == "the screen sleep"
        || n == "screen-sleep"
        || n == "the screen-sleep"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/displaysleep"
            | "/display-sleep"
            | "/displaysleep-status"
            | "/display-sleep-status"
            | "/displaysleep-state"
            | "/display-sleep-state"
            | "/displaysleep-timer"
            | "/display-sleep-timer"
            | "/screen-sleep"
            | "/screensleep"
            | "/screen-sleep-status"
            | "/screen-sleep-timer"
            | "/mac-displaysleep"
            | "/mac-display-sleep"
            | "/mac-displaysleep-status"
            | "/mac-display-sleep-status"
            | "/mac-screen-sleep"
            | "displaysleep status"
            | "the displaysleep status"
            | "display sleep status"
            | "the display sleep status"
            | "display-sleep status"
            | "the display-sleep status"
            | "displaysleep state"
            | "the displaysleep state"
            | "display sleep state"
            | "the display sleep state"
            | "displaysleep timer"
            | "the displaysleep timer"
            | "display sleep timer"
            | "the display sleep timer"
            | "my displaysleep"
            | "my displaysleep status"
            | "my display sleep"
            | "my display sleep status"
            | "is displaysleep on"
            | "is displaysleep off"
            | "is displaysleep enabled"
            | "is displaysleep disabled"
            | "is display sleep on"
            | "is display sleep off"
            | "is display sleep enabled"
            | "is display sleep disabled"
            | "is display-sleep on"
            | "is display-sleep enabled"
            | "what's my displaysleep"
            | "whats my displaysleep"
            | "what is my displaysleep"
            | "what's my displaysleep status"
            | "whats my displaysleep status"
            | "what is my displaysleep status"
            | "what's my display sleep"
            | "whats my display sleep"
            | "what is my display sleep"
            | "what's my display sleep status"
            | "whats my display sleep status"
            | "what is my display sleep status"
            | "what's the displaysleep"
            | "whats the displaysleep"
            | "what is the displaysleep"
            | "what's the displaysleep status"
            | "whats the displaysleep status"
            | "what is the displaysleep status"
            | "what's the display sleep"
            | "whats the display sleep"
            | "what is the display sleep"
            | "what's the display sleep status"
            | "whats the display sleep status"
            | "what is the display sleep status"
            | "how's displaysleep"
            | "hows displaysleep"
            | "how's the displaysleep"
            | "hows the displaysleep"
            | "how's display sleep"
            | "hows display sleep"
            | "how's the display sleep"
            | "hows the display sleep"
            | "view displaysleep"
            | "view the displaysleep"
            | "view displaysleep status"
            | "view the displaysleep status"
            | "view display sleep"
            | "view the display sleep"
            | "view display sleep status"
            | "view the display sleep status"
            | "view display-sleep"
            | "view the display-sleep"
            | "see displaysleep"
            | "see the displaysleep"
            | "see displaysleep status"
            | "see the displaysleep status"
            | "see display sleep"
            | "see the display sleep"
            | "see display sleep status"
            | "see the display sleep status"
            | "open displaysleep"
            | "open the displaysleep"
            | "open displaysleep status"
            | "open the displaysleep status"
            | "open display sleep"
            | "open the display sleep"
            | "open display sleep status"
            | "open the display sleep status"
            | "open display-sleep"
            | "open the display-sleep"
            | "list displaysleep"
            | "list the displaysleep"
            | "list displaysleep status"
            | "list the displaysleep status"
            | "list display sleep"
            | "list the display sleep"
            | "list display sleep status"
            | "list the display sleep status"
            | "screen sleep status"
            | "the screen sleep status"
            | "screen-sleep status"
            | "the screen-sleep status"
            | "my screen sleep"
            | "my screen sleep status"
            | "is screen sleep on"
            | "is screen sleep off"
            | "is screen sleep enabled"
            | "is screen-sleep on"
            | "is screen-sleep enabled"
            | "what's my screen sleep"
            | "whats my screen sleep"
            | "what is my screen sleep"
            | "what's the screen sleep"
            | "whats the screen sleep"
            | "what is the screen sleep"
            | "what's the screen sleep status"
            | "whats the screen sleep status"
            | "what is the screen sleep status"
            | "view screen sleep"
            | "view the screen sleep"
            | "view screen sleep status"
            | "view the screen sleep status"
            | "view screen-sleep"
            | "view the screen-sleep"
            | "see screen sleep"
            | "see the screen sleep"
            | "see screen sleep status"
            | "see the screen sleep status"
            | "open screen sleep"
            | "open the screen sleep"
            | "open screen sleep status"
            | "open the screen sleep status"
            | "open screen-sleep"
            | "open the screen-sleep"
            | "list screen sleep"
            | "list the screen sleep"
            | "list screen sleep status"
            | "list the screen sleep status"
            | "list screen-sleep"
            | "list the screen-sleep"
    )
}

/// Zero-LLM Display Sleep status from pmset Battery/AC `displaysleep` (macOS-only; empty elsewhere).
pub fn format_displaysleep_report() -> String {
    let v = crate::metrics::get_displaysleep_status();
    let v = v.trim();
    if v.is_empty() {
        return "**Display Sleep** — _no Display Sleep reading on this Mac right now._".to_string();
    }
    format!("**Display Sleep** · `{v}`")
}

/// Disk Sleep — `pmset` Battery/AC `disksleep` minutes (+ active fallback; 0 = Never).
/// Slash + view/open NL (`/disksleep` · `/disk-sleep` · `view disk sleep`).
/// Bare `disksleep` / `disk sleep` / `hard disk sleep` stay with the agent;
/// `/displaysleep` · `/display-sleep` stay Display Sleep; `/screensaver` stays Screensaver;
/// Disk Cleanup / `/ssd` / `/free` stay put; system `sleep` stays out.
pub fn looks_like_disksleep_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.contains("quarantine")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("password")
        || n.contains("screenshot")
        || n.contains("resolution")
        || n.contains("connect")
        || n.contains("tunnel")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("create")
        || n.contains("delete")
        || n.contains("remove")
        || n.contains("display sleep")
        || n.contains("display-sleep")
        || n.contains("displaysleep")
        || n.contains("screen sleep")
        || n.contains("screen-sleep")
        || n.contains("screensleep")
        || n.contains("system sleep")
        || n.contains("system-sleep")
        || n.contains("systemsleep")
        || n.contains("computer sleep")
        || n.contains("computer-sleep")
        || n.contains("computersleep")
        || n.contains("proximity")
        || n.contains("low power")
        || n.contains("low-power")
        || n.contains("lowpower")
        || n.contains("powernap")
        || n.contains("power nap")
        || n.contains("power-nap")
        || n.contains("wake for")
        || n.contains("wake-for")
        || n.contains("wake on")
        || n.contains("wake-on")
        || n.contains("womp")
        || n.contains("screensaver")
        || n.contains("screen saver")
        || n.contains("screen-saver")
        || n.contains("idle time")
        || n.contains("idle-time")
        || n.contains("screen lock")
        || n.contains("screen-lock")
        || n.contains("lock screen")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("battery time")
        || n.contains("time remaining")
        || n.contains("cycle")
        || n.contains("charger")
        || n.contains("adapter")
        || n.contains("watt")
        || n.contains("percent")
        || n.contains('%')
        || n.contains("free")
        || n.contains("used")
        || n.contains("capacity")
        || n == "/lpm"
        || n == "/power"
        || n == "/watts"
        || n == "/power-draw"
        || n == "/battery"
        || n == "/bat"
        || n == "/battery-time"
        || n == "/cycles"
        || n == "/charger"
        || n == "/adapter"
        || n == "/powernap"
        || n == "/power-nap"
        || n == "/womp"
        || n == "/wake-for-network"
        || n == "/wake-on-lan"
        || n == "/wol"
        || n == "/systemsleep"
        || n == "/system-sleep"
        || n == "/computer-sleep"
        || n == "/computersleep"
        || n == "/sleep"
        || n == "/displaysleep"
        || n == "/display-sleep"
        || n == "/screen-sleep"
        || n == "/screensaver"
        || n == "/screen-saver"
        || n == "/idle-time"
        || n == "/screen-lock"
        || n == "/lock-screen"
        || n == "/filevault"
        || n == "/display"
        || n == "/displays"
        || n == "/screen"
        || n == "/screens"
        || n == "/resolution"
        || n == "/ssd"
        || n == "/disk"
        || n == "/free"
        || n == "/ssd-size"
        || n == "/disk-size"
        || n == "/wifi"
        || n == "/ssid"
        || n == "/vpn"
        || n == "/tasks"
        || n == "/uptime"
        // Bare phrases stay with the agent.
        || n == "disksleep"
        || n == "the disksleep"
        || n == "disk sleep"
        || n == "the disk sleep"
        || n == "disk-sleep"
        || n == "the disk-sleep"
        || n == "hard disk sleep"
        || n == "the hard disk sleep"
        || n == "hard-disk-sleep"
        || n == "the hard-disk-sleep"
        || n == "hdd sleep"
        || n == "the hdd sleep"
        || n == "hdd-sleep"
        || n == "the hdd-sleep"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/disksleep"
            | "/disk-sleep"
            | "/disksleep-status"
            | "/disk-sleep-status"
            | "/disksleep-state"
            | "/disk-sleep-state"
            | "/disksleep-timer"
            | "/disk-sleep-timer"
            | "/hard-disk-sleep"
            | "/harddisksleep"
            | "/hard-disk-sleep-status"
            | "/hdd-sleep"
            | "/hddsleep"
            | "/hdd-sleep-status"
            | "/mac-disksleep"
            | "/mac-disk-sleep"
            | "/mac-disksleep-status"
            | "/mac-disk-sleep-status"
            | "/mac-hard-disk-sleep"
            | "/mac-hdd-sleep"
            | "disksleep status"
            | "the disksleep status"
            | "disk sleep status"
            | "the disk sleep status"
            | "disk-sleep status"
            | "the disk-sleep status"
            | "disksleep state"
            | "the disksleep state"
            | "disk sleep state"
            | "the disk sleep state"
            | "disksleep timer"
            | "the disksleep timer"
            | "disk sleep timer"
            | "the disk sleep timer"
            | "my disksleep"
            | "my disksleep status"
            | "my disk sleep"
            | "my disk sleep status"
            | "is disksleep on"
            | "is disksleep off"
            | "is disksleep enabled"
            | "is disksleep disabled"
            | "is disk sleep on"
            | "is disk sleep off"
            | "is disk sleep enabled"
            | "is disk sleep disabled"
            | "is disk-sleep on"
            | "is disk-sleep enabled"
            | "what's my disksleep"
            | "whats my disksleep"
            | "what is my disksleep"
            | "what's my disksleep status"
            | "whats my disksleep status"
            | "what is my disksleep status"
            | "what's my disk sleep"
            | "whats my disk sleep"
            | "what is my disk sleep"
            | "what's my disk sleep status"
            | "whats my disk sleep status"
            | "what is my disk sleep status"
            | "what's the disksleep"
            | "whats the disksleep"
            | "what is the disksleep"
            | "what's the disksleep status"
            | "whats the disksleep status"
            | "what is the disksleep status"
            | "what's the disk sleep"
            | "whats the disk sleep"
            | "what is the disk sleep"
            | "what's the disk sleep status"
            | "whats the disk sleep status"
            | "what is the disk sleep status"
            | "how's disksleep"
            | "hows disksleep"
            | "how's the disksleep"
            | "hows the disksleep"
            | "how's disk sleep"
            | "hows disk sleep"
            | "how's the disk sleep"
            | "hows the disk sleep"
            | "view disksleep"
            | "view the disksleep"
            | "view disksleep status"
            | "view the disksleep status"
            | "view disk sleep"
            | "view the disk sleep"
            | "view disk sleep status"
            | "view the disk sleep status"
            | "view disk-sleep"
            | "view the disk-sleep"
            | "see disksleep"
            | "see the disksleep"
            | "see disksleep status"
            | "see the disksleep status"
            | "see disk sleep"
            | "see the disk sleep"
            | "see disk sleep status"
            | "see the disk sleep status"
            | "open disksleep"
            | "open the disksleep"
            | "open disksleep status"
            | "open the disksleep status"
            | "open disk sleep"
            | "open the disk sleep"
            | "open disk sleep status"
            | "open the disk sleep status"
            | "open disk-sleep"
            | "open the disk-sleep"
            | "list disksleep"
            | "list the disksleep"
            | "list disksleep status"
            | "list the disksleep status"
            | "list disk sleep"
            | "list the disk sleep"
            | "list disk sleep status"
            | "list the disk sleep status"
            | "hard disk sleep status"
            | "the hard disk sleep status"
            | "hard-disk-sleep status"
            | "the hard-disk-sleep status"
            | "my hard disk sleep"
            | "my hard disk sleep status"
            | "is hard disk sleep on"
            | "is hard disk sleep off"
            | "is hard disk sleep enabled"
            | "is hard-disk-sleep on"
            | "is hard-disk-sleep enabled"
            | "what's my hard disk sleep"
            | "whats my hard disk sleep"
            | "what is my hard disk sleep"
            | "what's the hard disk sleep"
            | "whats the hard disk sleep"
            | "what is the hard disk sleep"
            | "what's the hard disk sleep status"
            | "whats the hard disk sleep status"
            | "what is the hard disk sleep status"
            | "view hard disk sleep"
            | "view the hard disk sleep"
            | "view hard disk sleep status"
            | "view the hard disk sleep status"
            | "view hard-disk-sleep"
            | "view the hard-disk-sleep"
            | "see hard disk sleep"
            | "see the hard disk sleep"
            | "open hard disk sleep"
            | "open the hard disk sleep"
            | "open hard-disk-sleep"
            | "open the hard-disk-sleep"
            | "list hard disk sleep"
            | "list the hard disk sleep"
            | "list hard-disk-sleep"
            | "list the hard-disk-sleep"
            | "hdd sleep status"
            | "the hdd sleep status"
            | "hdd-sleep status"
            | "the hdd-sleep status"
            | "my hdd sleep"
            | "is hdd sleep on"
            | "is hdd sleep enabled"
            | "is hdd-sleep on"
            | "is hdd-sleep enabled"
            | "what's my hdd sleep"
            | "whats my hdd sleep"
            | "what's the hdd sleep"
            | "whats the hdd sleep"
            | "view hdd sleep"
            | "view the hdd sleep"
            | "view hdd-sleep"
            | "view the hdd-sleep"
            | "see hdd sleep"
            | "see the hdd sleep"
            | "open hdd sleep"
            | "open the hdd sleep"
            | "open hdd-sleep"
            | "open the hdd-sleep"
            | "list hdd sleep"
            | "list the hdd sleep"
            | "list hdd-sleep"
            | "list the hdd-sleep"
    )
}

/// Zero-LLM Disk Sleep status from pmset Battery/AC `disksleep` (macOS-only; empty elsewhere).
pub fn format_disksleep_report() -> String {
    let v = crate::metrics::get_disksleep_status();
    let v = v.trim();
    if v.is_empty() {
        return "**Disk Sleep** — _no Disk Sleep reading on this Mac right now._".to_string();
    }
    format!("**Disk Sleep** · `{v}`")
}

/// Proximity Wake — `pmset` Battery/AC `proximitywake` (+ active fallback).
/// Slash + view/open NL (`/proximitywake` · `/proximity-wake` · `view proximity wake`).
/// Bare `proximitywake` / `proximity wake` stay with the agent;
/// `/womp` stays Wake for network; `/powernap` stays Power Nap; `/displaysleep` /
/// `/disksleep` stay timers; system sleep / ttyskeepawake stay out.
pub fn looks_like_proximitywake_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("password")
        || n.contains("screenshot")
        || n.contains("resolution")
        || n.contains("connect")
        || n.contains("tunnel")
        || n.split_whitespace().any(|w| w == "keep" || w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("create")
        || n.contains("delete")
        || n.contains("remove")
        || n.contains("low power")
        || n.contains("low-power")
        || n.contains("lowpower")
        || n.contains("powernap")
        || n.contains("power nap")
        || n.contains("power-nap")
        || n.contains("wake for network")
        || n.contains("wake-for-network")
        || n.contains("wake on lan")
        || n.contains("wake-on-lan")
        || n.contains("wake on network")
        || n.contains("womp")
        || n.contains("displaysleep")
        || n.contains("display sleep")
        || n.contains("display-sleep")
        || n.contains("screen sleep")
        || n.contains("screen-sleep")
        || n.contains("disksleep")
        || n.contains("disk sleep")
        || n.contains("disk-sleep")
        || n.contains("hard disk sleep")
        || n.contains("hdd sleep")
        || n.contains("system sleep")
        || n.contains("system-sleep")
        || n.contains("systemsleep")
        || n.contains("computer sleep")
        || n.contains("computer-sleep")
        || n.contains("computersleep")
        || n.contains("ttyskeepawake")
        || n.contains("ttys keep awake")
        || n.contains("networkoversleep")
        || n.contains("network over sleep")
        || n.contains("network-over-sleep")
        || n.contains("tcpkeepalive")
        || n.contains("tcp keep alive")
        || n.contains("tcp-keep-alive")
        || n.contains("screensaver")
        || n.contains("screen saver")
        || n.contains("screen-saver")
        || n.contains("idle time")
        || n.contains("idle-time")
        || n.contains("screen lock")
        || n.contains("screen-lock")
        || n.contains("lock screen")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("battery time")
        || n.contains("time remaining")
        || n.contains("cycle")
        || n.contains("charger")
        || n.contains("adapter")
        || n.contains("watt")
        || n == "/lpm"
        || n == "/power"
        || n == "/watts"
        || n == "/power-draw"
        || n == "/battery"
        || n == "/bat"
        || n == "/battery-time"
        || n == "/cycles"
        || n == "/charger"
        || n == "/adapter"
        || n == "/powernap"
        || n == "/power-nap"
        || n == "/systemsleep"
        || n == "/system-sleep"
        || n == "/computer-sleep"
        || n == "/computersleep"
        || n == "/sleep"
        || n == "/womp"
        || n == "/wake-for-network"
        || n == "/wake-on-lan"
        || n == "/wol"
        || n == "/displaysleep"
        || n == "/display-sleep"
        || n == "/screen-sleep"
        || n == "/disksleep"
        || n == "/disk-sleep"
        || n == "/hard-disk-sleep"
        || n == "/hdd-sleep"
        || n == "/screensaver"
        || n == "/screen-saver"
        || n == "/idle-time"
        || n == "/screen-lock"
        || n == "/lock-screen"
        || n == "/filevault"
        || n == "/wifi"
        || n == "/ssid"
        || n == "/vpn"
        || n == "/tasks"
        || n == "/uptime"
        // Bare phrases stay with the agent.
        || n == "proximitywake"
        || n == "the proximitywake"
        || n == "proximity wake"
        || n == "the proximity wake"
        || n == "proximity-wake"
        || n == "the proximity-wake"
        || n == "proximity wake from devices"
        || n == "the proximity wake from devices"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/proximitywake"
            | "/proximity-wake"
            | "/proximitywake-status"
            | "/proximity-wake-status"
            | "/proximitywake-state"
            | "/proximity-wake-state"
            | "/mac-proximitywake"
            | "/mac-proximity-wake"
            | "/mac-proximitywake-status"
            | "/mac-proximity-wake-status"
            | "proximitywake status"
            | "the proximitywake status"
            | "proximity wake status"
            | "the proximity wake status"
            | "proximity-wake status"
            | "the proximity-wake status"
            | "proximitywake state"
            | "the proximitywake state"
            | "proximity wake state"
            | "the proximity wake state"
            | "my proximitywake"
            | "my proximitywake status"
            | "my proximity wake"
            | "my proximity wake status"
            | "is proximitywake on"
            | "is proximitywake off"
            | "is proximitywake enabled"
            | "is proximitywake disabled"
            | "is proximity wake on"
            | "is proximity wake off"
            | "is proximity wake enabled"
            | "is proximity wake disabled"
            | "is proximity-wake on"
            | "is proximity-wake enabled"
            | "what's my proximitywake"
            | "whats my proximitywake"
            | "what is my proximitywake"
            | "what's my proximitywake status"
            | "whats my proximitywake status"
            | "what is my proximitywake status"
            | "what's my proximity wake"
            | "whats my proximity wake"
            | "what is my proximity wake"
            | "what's my proximity wake status"
            | "whats my proximity wake status"
            | "what is my proximity wake status"
            | "what's the proximitywake"
            | "whats the proximitywake"
            | "what is the proximitywake"
            | "what's the proximitywake status"
            | "whats the proximitywake status"
            | "what is the proximitywake status"
            | "what's the proximity wake"
            | "whats the proximity wake"
            | "what is the proximity wake"
            | "what's the proximity wake status"
            | "whats the proximity wake status"
            | "what is the proximity wake status"
            | "how's proximitywake"
            | "hows proximitywake"
            | "how's the proximitywake"
            | "hows the proximitywake"
            | "how's proximity wake"
            | "hows proximity wake"
            | "how's the proximity wake"
            | "hows the proximity wake"
            | "view proximitywake"
            | "view the proximitywake"
            | "view proximitywake status"
            | "view the proximitywake status"
            | "view proximity wake"
            | "view the proximity wake"
            | "view proximity wake status"
            | "view the proximity wake status"
            | "view proximity-wake"
            | "view the proximity-wake"
            | "see proximitywake"
            | "see the proximitywake"
            | "see proximitywake status"
            | "see the proximitywake status"
            | "see proximity wake"
            | "see the proximity wake"
            | "see proximity wake status"
            | "see the proximity wake status"
            | "open proximitywake"
            | "open the proximitywake"
            | "open proximitywake status"
            | "open the proximitywake status"
            | "open proximity wake"
            | "open the proximity wake"
            | "open proximity wake status"
            | "open the proximity wake status"
            | "open proximity-wake"
            | "open the proximity-wake"
            | "list proximitywake"
            | "list the proximitywake"
            | "list proximitywake status"
            | "list the proximitywake status"
            | "list proximity wake"
            | "list the proximity wake"
            | "list proximity wake status"
            | "list the proximity wake status"
            | "list proximity-wake"
            | "list the proximity-wake"
    )
}

/// Zero-LLM Proximity Wake status from pmset Battery/AC `proximitywake` (macOS-only; empty elsewhere).
pub fn format_proximitywake_report() -> String {
    let v = crate::metrics::get_proximitywake_status();
    let v = v.trim();
    if v.is_empty() {
        return "**Proximity Wake** — _no Proximity Wake reading on this Mac right now._"
            .to_string();
    }
    format!("**Proximity Wake** · `{v}`")
}

/// TTY keep awake — `pmset` Battery/AC `ttyskeepawake` (+ active fallback).
/// Slash + view/open NL (`/ttyskeepawake` · `/tty-keep-awake` · `view tty keep awake`).
/// Bare `ttyskeepawake` / `tty keep awake` / `ttys keep awake` stay with the agent;
/// `/proximitywake` stays Proximity Wake; `/womp` stays Wake for network;
/// `/powernap` stays Power Nap; system sleep stays out.
pub fn looks_like_ttyskeepawake_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("password")
        || n.contains("screenshot")
        || n.contains("resolution")
        || n.contains("connect")
        || n.contains("tunnel")
        || n.split_whitespace().any(|w| w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("create")
        || n.contains("delete")
        || n.contains("remove")
        || n.contains("low power")
        || n.contains("low-power")
        || n.contains("lowpower")
        || n.contains("powernap")
        || n.contains("power nap")
        || n.contains("power-nap")
        || n.contains("wake for network")
        || n.contains("wake-for-network")
        || n.contains("wake on lan")
        || n.contains("wake-on-lan")
        || n.contains("wake on network")
        || n.contains("womp")
        || n.contains("proximitywake")
        || n.contains("proximity wake")
        || n.contains("proximity-wake")
        || n.contains("displaysleep")
        || n.contains("display sleep")
        || n.contains("display-sleep")
        || n.contains("screen sleep")
        || n.contains("screen-sleep")
        || n.contains("disksleep")
        || n.contains("disk sleep")
        || n.contains("disk-sleep")
        || n.contains("hard disk sleep")
        || n.contains("hdd sleep")
        || n.contains("system sleep")
        || n.contains("system-sleep")
        || n.contains("systemsleep")
        || n.contains("computer sleep")
        || n.contains("computer-sleep")
        || n.contains("computersleep")
        || n.contains("networkoversleep")
        || n.contains("network over sleep")
        || n.contains("network-over-sleep")
        || n.contains("tcpkeepalive")
        || n.contains("tcp keep alive")
        || n.contains("tcp-keep-alive")
        || n.contains("screensaver")
        || n.contains("screen saver")
        || n.contains("screen-saver")
        || n.contains("idle time")
        || n.contains("idle-time")
        || n.contains("screen lock")
        || n.contains("screen-lock")
        || n.contains("lock screen")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("battery time")
        || n.contains("time remaining")
        || n.contains("cycle")
        || n.contains("charger")
        || n.contains("adapter")
        || n.contains("watt")
        || n == "/lpm"
        || n == "/power"
        || n == "/watts"
        || n == "/power-draw"
        || n == "/battery"
        || n == "/bat"
        || n == "/battery-time"
        || n == "/cycles"
        || n == "/charger"
        || n == "/adapter"
        || n == "/powernap"
        || n == "/power-nap"
        || n == "/womp"
        || n == "/wake-for-network"
        || n == "/wake-on-lan"
        || n == "/wol"
        || n == "/proximitywake"
        || n == "/proximity-wake"
        || n == "/displaysleep"
        || n == "/display-sleep"
        || n == "/screen-sleep"
        || n == "/disksleep"
        || n == "/disk-sleep"
        || n == "/hard-disk-sleep"
        || n == "/hdd-sleep"
        || n == "/systemsleep"
        || n == "/system-sleep"
        || n == "/computer-sleep"
        || n == "/computersleep"
        || n == "/sleep"
        || n == "/screensaver"
        || n == "/screen-saver"
        || n == "/idle-time"
        || n == "/screen-lock"
        || n == "/lock-screen"
        || n == "/filevault"
        || n == "/wifi"
        || n == "/ssid"
        || n == "/vpn"
        || n == "/tasks"
        || n == "/uptime"
        // Bare phrases stay with the agent.
        || n == "ttyskeepawake"
        || n == "the ttyskeepawake"
        || n == "tty keep awake"
        || n == "the tty keep awake"
        || n == "tty-keep-awake"
        || n == "the tty-keep-awake"
        || n == "ttys keep awake"
        || n == "the ttys keep awake"
        || n == "ttys-keep-awake"
        || n == "the ttys-keep-awake"
        || n == "ttykeepawake"
        || n == "the ttykeepawake"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/ttyskeepawake"
            | "/ttys-keep-awake"
            | "/tty-keep-awake"
            | "/ttykeepawake"
            | "/ttyskeepawake-status"
            | "/ttys-keep-awake-status"
            | "/tty-keep-awake-status"
            | "/ttyskeepawake-state"
            | "/ttys-keep-awake-state"
            | "/tty-keep-awake-state"
            | "/mac-ttyskeepawake"
            | "/mac-ttys-keep-awake"
            | "/mac-tty-keep-awake"
            | "/mac-ttyskeepawake-status"
            | "/mac-tty-keep-awake-status"
            | "ttyskeepawake status"
            | "the ttyskeepawake status"
            | "tty keep awake status"
            | "the tty keep awake status"
            | "tty-keep-awake status"
            | "the tty-keep-awake status"
            | "ttys keep awake status"
            | "the ttys keep awake status"
            | "ttys-keep-awake status"
            | "the ttys-keep-awake status"
            | "ttyskeepawake state"
            | "the ttyskeepawake state"
            | "tty keep awake state"
            | "the tty keep awake state"
            | "my ttyskeepawake"
            | "my ttyskeepawake status"
            | "my tty keep awake"
            | "my tty keep awake status"
            | "my ttys keep awake"
            | "my ttys keep awake status"
            | "is ttyskeepawake on"
            | "is ttyskeepawake off"
            | "is ttyskeepawake enabled"
            | "is ttyskeepawake disabled"
            | "is tty keep awake on"
            | "is tty keep awake off"
            | "is tty keep awake enabled"
            | "is tty keep awake disabled"
            | "is ttys keep awake on"
            | "is ttys keep awake off"
            | "is ttys keep awake enabled"
            | "is ttys keep awake disabled"
            | "is tty-keep-awake on"
            | "is tty-keep-awake enabled"
            | "is ttys-keep-awake on"
            | "is ttys-keep-awake enabled"
            | "what's my ttyskeepawake"
            | "whats my ttyskeepawake"
            | "what is my ttyskeepawake"
            | "what's my ttyskeepawake status"
            | "whats my ttyskeepawake status"
            | "what is my ttyskeepawake status"
            | "what's my tty keep awake"
            | "whats my tty keep awake"
            | "what is my tty keep awake"
            | "what's my tty keep awake status"
            | "whats my tty keep awake status"
            | "what is my tty keep awake status"
            | "what's my ttys keep awake"
            | "whats my ttys keep awake"
            | "what is my ttys keep awake"
            | "what's my ttys keep awake status"
            | "whats my ttys keep awake status"
            | "what is my ttys keep awake status"
            | "what's the ttyskeepawake"
            | "whats the ttyskeepawake"
            | "what is the ttyskeepawake"
            | "what's the ttyskeepawake status"
            | "whats the ttyskeepawake status"
            | "what is the ttyskeepawake status"
            | "what's the tty keep awake"
            | "whats the tty keep awake"
            | "what is the tty keep awake"
            | "what's the tty keep awake status"
            | "whats the tty keep awake status"
            | "what is the tty keep awake status"
            | "what's the ttys keep awake"
            | "whats the ttys keep awake"
            | "what is the ttys keep awake"
            | "what's the ttys keep awake status"
            | "whats the ttys keep awake status"
            | "what is the ttys keep awake status"
            | "how's ttyskeepawake"
            | "hows ttyskeepawake"
            | "how's the ttyskeepawake"
            | "hows the ttyskeepawake"
            | "how's tty keep awake"
            | "hows tty keep awake"
            | "how's the tty keep awake"
            | "hows the tty keep awake"
            | "how's ttys keep awake"
            | "hows ttys keep awake"
            | "how's the ttys keep awake"
            | "hows the ttys keep awake"
            | "view ttyskeepawake"
            | "view the ttyskeepawake"
            | "view ttyskeepawake status"
            | "view the ttyskeepawake status"
            | "view tty keep awake"
            | "view the tty keep awake"
            | "view tty keep awake status"
            | "view the tty keep awake status"
            | "view tty-keep-awake"
            | "view the tty-keep-awake"
            | "view ttys keep awake"
            | "view the ttys keep awake"
            | "view ttys-keep-awake"
            | "view the ttys-keep-awake"
            | "see ttyskeepawake"
            | "see the ttyskeepawake"
            | "see ttyskeepawake status"
            | "see the ttyskeepawake status"
            | "see tty keep awake"
            | "see the tty keep awake"
            | "see tty keep awake status"
            | "see the tty keep awake status"
            | "see ttys keep awake"
            | "see the ttys keep awake"
            | "open ttyskeepawake"
            | "open the ttyskeepawake"
            | "open ttyskeepawake status"
            | "open the ttyskeepawake status"
            | "open tty keep awake"
            | "open the tty keep awake"
            | "open tty keep awake status"
            | "open the tty keep awake status"
            | "open tty-keep-awake"
            | "open the tty-keep-awake"
            | "open ttys keep awake"
            | "open the ttys keep awake"
            | "open ttys-keep-awake"
            | "open the ttys-keep-awake"
            | "list ttyskeepawake"
            | "list the ttyskeepawake"
            | "list ttyskeepawake status"
            | "list the ttyskeepawake status"
            | "list tty keep awake"
            | "list the tty keep awake"
            | "list tty keep awake status"
            | "list the tty keep awake status"
            | "list tty-keep-awake"
            | "list the tty-keep-awake"
            | "list ttys keep awake"
            | "list the ttys keep awake"
            | "list ttys-keep-awake"
            | "list the ttys-keep-awake"
    )
}

/// Zero-LLM TTY keep awake status from pmset Battery/AC `ttyskeepawake` (macOS-only; empty elsewhere).
pub fn format_ttyskeepawake_report() -> String {
    let v = crate::metrics::get_ttyskeepawake_status();
    let v = v.trim();
    if v.is_empty() {
        return "**TTY keep awake** — _no TTY keep awake reading on this Mac right now._"
            .to_string();
    }
    format!("**TTY keep awake** · `{v}`")
}

/// TCP Keep Alive — `pmset` Battery/AC `tcpkeepalive` (+ active fallback).
/// Slash + view/open NL (`/tcpkeepalive` · `/tcp-keep-alive` · `view tcp keep alive`).
/// Bare `tcpkeepalive` / `tcp keep alive` stay with the agent;
/// `/ttyskeepawake` stays TTY keep awake; `/womp` stays Wake for network;
/// `/systemsleep` stays System Sleep; networkoversleep stays out.
pub fn looks_like_tcpkeepalive_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("password")
        || n.contains("screenshot")
        || n.contains("resolution")
        || n.contains("connect")
        || n.contains("tunnel")
        || n.split_whitespace().any(|w| w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("create")
        || n.contains("delete")
        || n.contains("remove")
        || n.contains("low power")
        || n.contains("low-power")
        || n.contains("lowpower")
        || n.contains("powernap")
        || n.contains("power nap")
        || n.contains("power-nap")
        || n.contains("wake for network")
        || n.contains("wake-for-network")
        || n.contains("wake on lan")
        || n.contains("wake-on-lan")
        || n.contains("wake on network")
        || n.contains("womp")
        || n.contains("proximitywake")
        || n.contains("proximity wake")
        || n.contains("proximity-wake")
        || n.contains("displaysleep")
        || n.contains("display sleep")
        || n.contains("display-sleep")
        || n.contains("screen sleep")
        || n.contains("screen-sleep")
        || n.contains("disksleep")
        || n.contains("disk sleep")
        || n.contains("disk-sleep")
        || n.contains("hard disk sleep")
        || n.contains("hdd sleep")
        || n.contains("system sleep")
        || n.contains("system-sleep")
        || n.contains("systemsleep")
        || n.contains("computer sleep")
        || n.contains("computer-sleep")
        || n.contains("computersleep")
        || n.contains("ttyskeepawake")
        || n.contains("tty keep awake")
        || n.contains("ttys keep awake")
        || n.contains("tty-keep-awake")
        || n.contains("ttys-keep-awake")
        || n.contains("ttykeepawake")
        || n.contains("networkoversleep")
        || n.contains("network over sleep")
        || n.contains("network-over-sleep")
        || n.contains("hibernate")
        || n.contains("screensaver")
        || n.contains("screen saver")
        || n.contains("screen-saver")
        || n.contains("idle time")
        || n.contains("idle-time")
        || n.contains("screen lock")
        || n.contains("screen-lock")
        || n.contains("lock screen")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("battery time")
        || n.contains("time remaining")
        || n.contains("cycle")
        || n.contains("charger")
        || n.contains("adapter")
        || n.contains("watt")
        || n == "/lpm"
        || n == "/power"
        || n == "/watts"
        || n == "/power-draw"
        || n == "/battery"
        || n == "/bat"
        || n == "/battery-time"
        || n == "/cycles"
        || n == "/charger"
        || n == "/adapter"
        || n == "/powernap"
        || n == "/power-nap"
        || n == "/womp"
        || n == "/wake-for-network"
        || n == "/wake-on-lan"
        || n == "/wol"
        || n == "/proximitywake"
        || n == "/proximity-wake"
        || n == "/displaysleep"
        || n == "/display-sleep"
        || n == "/screen-sleep"
        || n == "/disksleep"
        || n == "/disk-sleep"
        || n == "/hard-disk-sleep"
        || n == "/hdd-sleep"
        || n == "/systemsleep"
        || n == "/system-sleep"
        || n == "/computer-sleep"
        || n == "/computersleep"
        || n == "/sleep"
        || n == "/ttyskeepawake"
        || n == "/tty-keep-awake"
        || n == "/ttys-keep-awake"
        || n == "/ttykeepawake"
        || n == "/screensaver"
        || n == "/screen-saver"
        || n == "/idle-time"
        || n == "/screen-lock"
        || n == "/lock-screen"
        || n == "/filevault"
        || n == "/wifi"
        || n == "/ssid"
        || n == "/vpn"
        || n == "/tasks"
        || n == "/uptime"
        // Bare phrases stay with the agent.
        || n == "tcpkeepalive"
        || n == "the tcpkeepalive"
        || n == "tcp keep alive"
        || n == "the tcp keep alive"
        || n == "tcp-keep-alive"
        || n == "the tcp-keep-alive"
        || n == "tcp keepalive"
        || n == "the tcp keepalive"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/tcpkeepalive"
            | "/tcp-keep-alive"
            | "/tcpkeepalive-status"
            | "/tcp-keep-alive-status"
            | "/tcpkeepalive-state"
            | "/tcp-keep-alive-state"
            | "/tcp-keepalive"
            | "/tcp-keepalive-status"
            | "/mac-tcpkeepalive"
            | "/mac-tcp-keep-alive"
            | "/mac-tcpkeepalive-status"
            | "/mac-tcp-keep-alive-status"
            | "tcpkeepalive status"
            | "the tcpkeepalive status"
            | "tcp keep alive status"
            | "the tcp keep alive status"
            | "tcp-keep-alive status"
            | "the tcp-keep-alive status"
            | "tcp keepalive status"
            | "the tcp keepalive status"
            | "tcpkeepalive state"
            | "the tcpkeepalive state"
            | "tcp keep alive state"
            | "the tcp keep alive state"
            | "my tcpkeepalive"
            | "my tcpkeepalive status"
            | "my tcp keep alive"
            | "my tcp keep alive status"
            | "is tcpkeepalive on"
            | "is tcpkeepalive off"
            | "is tcpkeepalive enabled"
            | "is tcpkeepalive disabled"
            | "is tcp keep alive on"
            | "is tcp keep alive off"
            | "is tcp keep alive enabled"
            | "is tcp keep alive disabled"
            | "is tcp-keep-alive on"
            | "is tcp-keep-alive enabled"
            | "what's my tcpkeepalive"
            | "whats my tcpkeepalive"
            | "what is my tcpkeepalive"
            | "what's my tcpkeepalive status"
            | "whats my tcpkeepalive status"
            | "what is my tcpkeepalive status"
            | "what's my tcp keep alive"
            | "whats my tcp keep alive"
            | "what is my tcp keep alive"
            | "what's my tcp keep alive status"
            | "whats my tcp keep alive status"
            | "what is my tcp keep alive status"
            | "what's the tcpkeepalive"
            | "whats the tcpkeepalive"
            | "what is the tcpkeepalive"
            | "what's the tcpkeepalive status"
            | "whats the tcpkeepalive status"
            | "what is the tcpkeepalive status"
            | "what's the tcp keep alive"
            | "whats the tcp keep alive"
            | "what is the tcp keep alive"
            | "what's the tcp keep alive status"
            | "whats the tcp keep alive status"
            | "what is the tcp keep alive status"
            | "how's tcpkeepalive"
            | "hows tcpkeepalive"
            | "how's the tcpkeepalive"
            | "hows the tcpkeepalive"
            | "how's tcp keep alive"
            | "hows tcp keep alive"
            | "how's the tcp keep alive"
            | "hows the tcp keep alive"
            | "view tcpkeepalive"
            | "view the tcpkeepalive"
            | "view tcpkeepalive status"
            | "view the tcpkeepalive status"
            | "view tcp keep alive"
            | "view the tcp keep alive"
            | "view tcp keep alive status"
            | "view the tcp keep alive status"
            | "view tcp-keep-alive"
            | "view the tcp-keep-alive"
            | "see tcpkeepalive"
            | "see the tcpkeepalive"
            | "see tcpkeepalive status"
            | "see the tcpkeepalive status"
            | "see tcp keep alive"
            | "see the tcp keep alive"
            | "see tcp keep alive status"
            | "see the tcp keep alive status"
            | "open tcpkeepalive"
            | "open the tcpkeepalive"
            | "open tcpkeepalive status"
            | "open the tcpkeepalive status"
            | "open tcp keep alive"
            | "open the tcp keep alive"
            | "open tcp keep alive status"
            | "open the tcp keep alive status"
            | "open tcp-keep-alive"
            | "open the tcp-keep-alive"
            | "list tcpkeepalive"
            | "list the tcpkeepalive"
            | "list tcpkeepalive status"
            | "list the tcpkeepalive status"
            | "list tcp keep alive"
            | "list the tcp keep alive"
            | "list tcp keep alive status"
            | "list the tcp keep alive status"
            | "list tcp-keep-alive"
            | "list the tcp-keep-alive"
    )
}

/// Zero-LLM TCP Keep Alive status from pmset Battery/AC `tcpkeepalive` (macOS-only; empty elsewhere).
pub fn format_tcpkeepalive_report() -> String {
    let v = crate::metrics::get_tcpkeepalive_status();
    let v = v.trim();
    if v.is_empty() {
        return "**TCP Keep Alive** — _no TCP Keep Alive reading on this Mac right now._"
            .to_string();
    }
    format!("**TCP Keep Alive** · `{v}`")
}

/// Network over sleep — `pmset` Battery/AC `networkoversleep` (+ active fallback).
/// Slash + view/open NL (`/networkoversleep` · `/network-over-sleep` · `view network over sleep`).
/// Bare `networkoversleep` / `network over sleep` stay with the agent;
/// `/tcpkeepalive` stays TCP Keep Alive; `/womp` stays Wake for network;
/// `/systemsleep` stays System Sleep; hibernatemode stays out.
pub fn looks_like_networkoversleep_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("password")
        || n.contains("screenshot")
        || n.contains("resolution")
        || n.contains("connect")
        || n.contains("tunnel")
        || n.split_whitespace().any(|w| w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("create")
        || n.contains("delete")
        || n.contains("remove")
        || n.contains("low power")
        || n.contains("low-power")
        || n.contains("lowpower")
        || n.contains("powernap")
        || n.contains("power nap")
        || n.contains("power-nap")
        || n.contains("wake for network")
        || n.contains("wake-for-network")
        || n.contains("wake on lan")
        || n.contains("wake-on-lan")
        || n.contains("wake on network")
        || n.contains("womp")
        || n.contains("proximitywake")
        || n.contains("proximity wake")
        || n.contains("proximity-wake")
        || n.contains("displaysleep")
        || n.contains("display sleep")
        || n.contains("display-sleep")
        || n.contains("screen sleep")
        || n.contains("screen-sleep")
        || n.contains("disksleep")
        || n.contains("disk sleep")
        || n.contains("disk-sleep")
        || n.contains("hard disk sleep")
        || n.contains("hdd sleep")
        || n.contains("system sleep")
        || n.contains("system-sleep")
        || n.contains("systemsleep")
        || n.contains("computer sleep")
        || n.contains("computer-sleep")
        || n.contains("computersleep")
        || n.contains("ttyskeepawake")
        || n.contains("tty keep awake")
        || n.contains("ttys keep awake")
        || n.contains("tty-keep-awake")
        || n.contains("ttys-keep-awake")
        || n.contains("ttykeepawake")
        || n.contains("tcpkeepalive")
        || n.contains("tcp keep alive")
        || n.contains("tcp-keep-alive")
        || n.contains("tcp keepalive")
        || n.contains("hibernate")
        || n.contains("hibernatemode")
        || n.contains("screensaver")
        || n.contains("screen saver")
        || n.contains("screen-saver")
        || n.contains("idle time")
        || n.contains("idle-time")
        || n.contains("screen lock")
        || n.contains("screen-lock")
        || n.contains("lock screen")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("battery time")
        || n.contains("time remaining")
        || n.contains("cycle")
        || n.contains("charger")
        || n.contains("adapter")
        || n.contains("watt")
        || n == "/lpm"
        || n == "/power"
        || n == "/watts"
        || n == "/power-draw"
        || n == "/battery"
        || n == "/bat"
        || n == "/battery-time"
        || n == "/cycles"
        || n == "/charger"
        || n == "/adapter"
        || n == "/powernap"
        || n == "/power-nap"
        || n == "/womp"
        || n == "/wake-for-network"
        || n == "/wake-on-lan"
        || n == "/wol"
        || n == "/proximitywake"
        || n == "/proximity-wake"
        || n == "/displaysleep"
        || n == "/display-sleep"
        || n == "/screen-sleep"
        || n == "/disksleep"
        || n == "/disk-sleep"
        || n == "/hard-disk-sleep"
        || n == "/hdd-sleep"
        || n == "/systemsleep"
        || n == "/system-sleep"
        || n == "/computer-sleep"
        || n == "/computersleep"
        || n == "/sleep"
        || n == "/ttyskeepawake"
        || n == "/tty-keep-awake"
        || n == "/ttys-keep-awake"
        || n == "/ttykeepawake"
        || n == "/tcpkeepalive"
        || n == "/tcp-keep-alive"
        || n == "/tcp-keepalive"
        || n == "/screensaver"
        || n == "/screen-saver"
        || n == "/idle-time"
        || n == "/screen-lock"
        || n == "/lock-screen"
        || n == "/filevault"
        || n == "/wifi"
        || n == "/ssid"
        || n == "/vpn"
        || n == "/tasks"
        || n == "/uptime"
        // Bare phrases stay with the agent.
        || n == "networkoversleep"
        || n == "the networkoversleep"
        || n == "network over sleep"
        || n == "the network over sleep"
        || n == "network-over-sleep"
        || n == "the network-over-sleep"
        || n == "network oversleep"
        || n == "the network oversleep"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/networkoversleep"
            | "/network-over-sleep"
            | "/networkoversleep-status"
            | "/network-over-sleep-status"
            | "/networkoversleep-state"
            | "/network-over-sleep-state"
            | "/network-oversleep"
            | "/network-oversleep-status"
            | "/mac-networkoversleep"
            | "/mac-network-over-sleep"
            | "/mac-networkoversleep-status"
            | "/mac-network-over-sleep-status"
            | "networkoversleep status"
            | "the networkoversleep status"
            | "network over sleep status"
            | "the network over sleep status"
            | "network-over-sleep status"
            | "the network-over-sleep status"
            | "network oversleep status"
            | "the network oversleep status"
            | "networkoversleep state"
            | "the networkoversleep state"
            | "network over sleep state"
            | "the network over sleep state"
            | "my networkoversleep"
            | "my networkoversleep status"
            | "my network over sleep"
            | "my network over sleep status"
            | "is networkoversleep on"
            | "is networkoversleep off"
            | "is networkoversleep enabled"
            | "is networkoversleep disabled"
            | "is network over sleep on"
            | "is network over sleep off"
            | "is network over sleep enabled"
            | "is network over sleep disabled"
            | "is network-over-sleep on"
            | "is network-over-sleep enabled"
            | "what's my networkoversleep"
            | "whats my networkoversleep"
            | "what is my networkoversleep"
            | "what's my networkoversleep status"
            | "whats my networkoversleep status"
            | "what is my networkoversleep status"
            | "what's my network over sleep"
            | "whats my network over sleep"
            | "what is my network over sleep"
            | "what's my network over sleep status"
            | "whats my network over sleep status"
            | "what is my network over sleep status"
            | "what's the networkoversleep"
            | "whats the networkoversleep"
            | "what is the networkoversleep"
            | "what's the networkoversleep status"
            | "whats the networkoversleep status"
            | "what is the networkoversleep status"
            | "what's the network over sleep"
            | "whats the network over sleep"
            | "what is the network over sleep"
            | "what's the network over sleep status"
            | "whats the network over sleep status"
            | "what is the network over sleep status"
            | "how's networkoversleep"
            | "hows networkoversleep"
            | "how's the networkoversleep"
            | "hows the networkoversleep"
            | "how's network over sleep"
            | "hows network over sleep"
            | "how's the network over sleep"
            | "hows the network over sleep"
            | "view networkoversleep"
            | "view the networkoversleep"
            | "view networkoversleep status"
            | "view the networkoversleep status"
            | "view network over sleep"
            | "view the network over sleep"
            | "view network over sleep status"
            | "view the network over sleep status"
            | "view network-over-sleep"
            | "view the network-over-sleep"
            | "see networkoversleep"
            | "see the networkoversleep"
            | "see networkoversleep status"
            | "see the networkoversleep status"
            | "see network over sleep"
            | "see the network over sleep"
            | "see network over sleep status"
            | "see the network over sleep status"
            | "open networkoversleep"
            | "open the networkoversleep"
            | "open networkoversleep status"
            | "open the networkoversleep status"
            | "open network over sleep"
            | "open the network over sleep"
            | "open network over sleep status"
            | "open the network over sleep status"
            | "open network-over-sleep"
            | "open the network-over-sleep"
            | "list networkoversleep"
            | "list the networkoversleep"
            | "list networkoversleep status"
            | "list the networkoversleep status"
            | "list network over sleep"
            | "list the network over sleep"
            | "list network over sleep status"
            | "list the network over sleep status"
            | "list network-over-sleep"
            | "list the network-over-sleep"
    )
}

/// Zero-LLM Network over sleep status from pmset Battery/AC `networkoversleep` (macOS-only; empty elsewhere).
pub fn format_networkoversleep_report() -> String {
    let v = crate::metrics::get_networkoversleep_status();
    let v = v.trim();
    if v.is_empty() {
        return "**Network over sleep** — _no Network over sleep reading on this Mac right now._"
            .to_string();
    }
    format!("**Network over sleep** · `{v}`")
}

/// System Sleep — `pmset` Battery/AC `sleep` minutes (+ active fallback; 0 = Never).
/// Slash + view/open NL (`/systemsleep` · `/system-sleep` · `/computer-sleep` · `view system sleep`).
/// Bare `systemsleep` / `system sleep` / `computer sleep` / `sleep` stay with the agent;
/// `/displaysleep` stays Display Sleep; `/disksleep` stays Disk Sleep;
/// `/screensaver` stays Screensaver; `/ttyskeepawake` stays TTY keep awake.
pub fn looks_like_systemsleep_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if n.contains("why")
        || n.contains("how to")
        || n.contains("explain")
        || n.contains("path")
        || contains_age_token(&n)
        || n.contains("size")
        || n.contains("json")
        || n.contains("cleanup")
        || n.contains("reclaim")
        || n.starts_with("enable ")
        || n.starts_with("disable ")
        || n.contains("turn on")
        || n.contains("turn off")
        || n.contains("password")
        || n.contains("screenshot")
        || n.contains("resolution")
        || n.contains("connect")
        || n.contains("tunnel")
        || n.split_whitespace().any(|w| w == "discard")
        || n.contains("change")
        || n.contains("set ")
        || n.contains("reset")
        || n.contains("create")
        || n.contains("delete")
        || n.contains("remove")
        || n.contains("low power")
        || n.contains("low-power")
        || n.contains("lowpower")
        || n.contains("powernap")
        || n.contains("power nap")
        || n.contains("power-nap")
        || n.contains("wake for network")
        || n.contains("wake-for-network")
        || n.contains("wake on lan")
        || n.contains("wake-on-lan")
        || n.contains("wake on network")
        || n.contains("womp")
        || n.contains("proximitywake")
        || n.contains("proximity wake")
        || n.contains("proximity-wake")
        || n.contains("displaysleep")
        || n.contains("display sleep")
        || n.contains("display-sleep")
        || n.contains("screen sleep")
        || n.contains("screen-sleep")
        || n.contains("disksleep")
        || n.contains("disk sleep")
        || n.contains("disk-sleep")
        || n.contains("hard disk sleep")
        || n.contains("hdd sleep")
        || n.contains("ttyskeepawake")
        || n.contains("tty keep awake")
        || n.contains("ttys keep awake")
        || n.contains("tty-keep-awake")
        || n.contains("ttys-keep-awake")
        || n.contains("networkoversleep")
        || n.contains("network over sleep")
        || n.contains("network-over-sleep")
        || n.contains("tcpkeepalive")
        || n.contains("tcp keep alive")
        || n.contains("tcp-keep-alive")
        || n.contains("screensaver")
        || n.contains("screen saver")
        || n.contains("screen-saver")
        || n.contains("idle time")
        || n.contains("idle-time")
        || n.contains("screen lock")
        || n.contains("screen-lock")
        || n.contains("lock screen")
        || n.contains("filevault")
        || n.contains("file vault")
        || n.contains("battery time")
        || n.contains("time remaining")
        || n.contains("cycle")
        || n.contains("charger")
        || n.contains("adapter")
        || n.contains("watt")
        || n == "/lpm"
        || n == "/power"
        || n == "/watts"
        || n == "/power-draw"
        || n == "/battery"
        || n == "/bat"
        || n == "/battery-time"
        || n == "/cycles"
        || n == "/charger"
        || n == "/adapter"
        || n == "/powernap"
        || n == "/power-nap"
        || n == "/womp"
        || n == "/wake-for-network"
        || n == "/wake-on-lan"
        || n == "/wol"
        || n == "/proximitywake"
        || n == "/proximity-wake"
        || n == "/displaysleep"
        || n == "/display-sleep"
        || n == "/screen-sleep"
        || n == "/disksleep"
        || n == "/disk-sleep"
        || n == "/hard-disk-sleep"
        || n == "/hdd-sleep"
        || n == "/ttyskeepawake"
        || n == "/tty-keep-awake"
        || n == "/ttys-keep-awake"
        || n == "/screensaver"
        || n == "/screen-saver"
        || n == "/idle-time"
        || n == "/screen-lock"
        || n == "/lock-screen"
        || n == "/filevault"
        || n == "/wifi"
        || n == "/ssid"
        || n == "/vpn"
        || n == "/tasks"
        || n == "/uptime"
        // Bare phrases stay with the agent (incl. bare `sleep` — too ambiguous).
        || n == "systemsleep"
        || n == "the systemsleep"
        || n == "system sleep"
        || n == "the system sleep"
        || n == "system-sleep"
        || n == "the system-sleep"
        || n == "computersleep"
        || n == "the computersleep"
        || n == "computer sleep"
        || n == "the computer sleep"
        || n == "computer-sleep"
        || n == "the computer-sleep"
        || n == "sleep"
        || n == "the sleep"
    {
        return false;
    }
    matches!(
        n.as_str(),
        "/systemsleep"
            | "/system-sleep"
            | "/systemsleep-status"
            | "/system-sleep-status"
            | "/systemsleep-state"
            | "/system-sleep-state"
            | "/systemsleep-timer"
            | "/system-sleep-timer"
            | "/computer-sleep"
            | "/computersleep"
            | "/computer-sleep-status"
            | "/computer-sleep-timer"
            | "/mac-systemsleep"
            | "/mac-system-sleep"
            | "/mac-systemsleep-status"
            | "/mac-system-sleep-status"
            | "/mac-computer-sleep"
            | "/mac-computersleep"
            | "/sleep"
            | "/sleep-status"
            | "/sleep-state"
            | "/sleep-timer"
            | "systemsleep status"
            | "the systemsleep status"
            | "system sleep status"
            | "the system sleep status"
            | "system-sleep status"
            | "the system-sleep status"
            | "systemsleep state"
            | "the systemsleep state"
            | "system sleep state"
            | "the system sleep state"
            | "systemsleep timer"
            | "the systemsleep timer"
            | "system sleep timer"
            | "the system sleep timer"
            | "my systemsleep"
            | "my systemsleep status"
            | "my system sleep"
            | "my system sleep status"
            | "is systemsleep on"
            | "is systemsleep off"
            | "is systemsleep enabled"
            | "is systemsleep disabled"
            | "is system sleep on"
            | "is system sleep off"
            | "is system sleep enabled"
            | "is system sleep disabled"
            | "is system-sleep on"
            | "is system-sleep enabled"
            | "what's my systemsleep"
            | "whats my systemsleep"
            | "what is my systemsleep"
            | "what's my systemsleep status"
            | "whats my systemsleep status"
            | "what is my systemsleep status"
            | "what's my system sleep"
            | "whats my system sleep"
            | "what is my system sleep"
            | "what's my system sleep status"
            | "whats my system sleep status"
            | "what is my system sleep status"
            | "what's the systemsleep"
            | "whats the systemsleep"
            | "what is the systemsleep"
            | "what's the systemsleep status"
            | "whats the systemsleep status"
            | "what is the systemsleep status"
            | "what's the system sleep"
            | "whats the system sleep"
            | "what is the system sleep"
            | "what's the system sleep status"
            | "whats the system sleep status"
            | "what is the system sleep status"
            | "how's systemsleep"
            | "hows systemsleep"
            | "how's the systemsleep"
            | "hows the systemsleep"
            | "how's system sleep"
            | "hows system sleep"
            | "how's the system sleep"
            | "hows the system sleep"
            | "view systemsleep"
            | "view the systemsleep"
            | "view systemsleep status"
            | "view the systemsleep status"
            | "view system sleep"
            | "view the system sleep"
            | "view system sleep status"
            | "view the system sleep status"
            | "view system-sleep"
            | "view the system-sleep"
            | "see systemsleep"
            | "see the systemsleep"
            | "see systemsleep status"
            | "see the systemsleep status"
            | "see system sleep"
            | "see the system sleep"
            | "see system sleep status"
            | "see the system sleep status"
            | "open systemsleep"
            | "open the systemsleep"
            | "open systemsleep status"
            | "open the systemsleep status"
            | "open system sleep"
            | "open the system sleep"
            | "open system sleep status"
            | "open the system sleep status"
            | "open system-sleep"
            | "open the system-sleep"
            | "list systemsleep"
            | "list the systemsleep"
            | "list systemsleep status"
            | "list the systemsleep status"
            | "list system sleep"
            | "list the system sleep"
            | "list system sleep status"
            | "list the system sleep status"
            | "list system-sleep"
            | "list the system-sleep"
            | "computer sleep status"
            | "the computer sleep status"
            | "computer-sleep status"
            | "the computer-sleep status"
            | "computersleep status"
            | "the computersleep status"
            | "my computer sleep"
            | "my computer sleep status"
            | "is computer sleep on"
            | "is computer sleep off"
            | "is computer sleep enabled"
            | "is computer-sleep on"
            | "is computer-sleep enabled"
            | "is computersleep on"
            | "is computersleep enabled"
            | "what's my computer sleep"
            | "whats my computer sleep"
            | "what is my computer sleep"
            | "what's the computer sleep"
            | "whats the computer sleep"
            | "what is the computer sleep"
            | "what's the computer sleep status"
            | "whats the computer sleep status"
            | "what is the computer sleep status"
            | "view computer sleep"
            | "view the computer sleep"
            | "view computer sleep status"
            | "view the computer sleep status"
            | "view computer-sleep"
            | "view the computer-sleep"
            | "view computersleep"
            | "view the computersleep"
            | "see computer sleep"
            | "see the computer sleep"
            | "see computer sleep status"
            | "see the computer sleep status"
            | "open computer sleep"
            | "open the computer sleep"
            | "open computer sleep status"
            | "open the computer sleep status"
            | "open computer-sleep"
            | "open the computer-sleep"
            | "open computersleep"
            | "open the computersleep"
            | "list computer sleep"
            | "list the computer sleep"
            | "list computer sleep status"
            | "list the computer sleep status"
            | "list computer-sleep"
            | "list the computer-sleep"
            | "list computersleep"
            | "list the computersleep"
    )
}

/// Zero-LLM System Sleep status from pmset Battery/AC `sleep` (macOS-only; empty elsewhere).
pub fn format_systemsleep_report() -> String {
    let v = crate::metrics::get_systemsleep_status();
    let v = v.trim();
    if v.is_empty() {
        return "**System Sleep** — _no System Sleep reading on this Mac right now._".to_string();
    }
    format!("**System Sleep** · `{v}`")
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
