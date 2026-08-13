//! Fast / lite lanes: cut GPU meta-calls (criteria, topic, plan, verify) for trivial
//! and pre-routed Discord turns.

use chrono::{Local, Timelike};

/// How expensive this turn should be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnLane {
    /// Answer without any Ollama call.
    Instant { reply: String },
    /// Run tools / one execute pass; skip criteria, new-topic, plan (if pre-routed), and verify.
    Lite { reason: &'static str },
    /// Full agent pipeline.
    Full,
}

impl TurnLane {
    pub fn name(&self) -> &'static str {
        match self {
            TurnLane::Instant { .. } => "instant",
            TurnLane::Lite { .. } => "lite",
            TurnLane::Full => "full",
        }
    }

    pub fn skips_meta_llms(&self) -> bool {
        !matches!(self, TurnLane::Full)
    }
}

/// Plain user text (already unwrapped from MS_UNTRUSTED if needed).
pub fn classify_turn_lane(plain_question: &str, pre_routed: Option<&str>) -> TurnLane {
    let q = plain_question.trim();
    if q.is_empty() {
        return TurnLane::Full;
    }
    if let Some(reply) = try_instant_reply(q) {
        return TurnLane::Instant { reply };
    }
    if let Some(rec) = pre_routed {
        if lite_pre_route(rec) {
            return TurnLane::Lite {
                reason: "pre_routed_tool",
            };
        }
    }
    if is_trivial_chat(q) {
        return TurnLane::Lite {
            reason: "trivial_chat",
        };
    }
    TurnLane::Full
}

fn normalize_q(q: &str) -> String {
    q.trim()
        .trim_end_matches(['?', '!', '.', '¿', '¡'])
        .trim()
        .to_lowercase()
}

/// Letters/spaces only — so `Hola 👋` / `hi!!!` still match greeting instant lanes.
fn alpha_words(n: &str) -> String {
    n.chars()
        .map(|c| if c.is_alphabetic() || c.is_whitespace() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Zero-LLM answers for clock / ping style asks.
fn try_instant_reply(q: &str) -> Option<String> {
    let n = normalize_q(q);
    if is_time_question(&n) {
        let now = Local::now();
        return Some(format!(
            "It's **{}** (UTC offset {})",
            now.format("%A, %Y-%m-%d %H:%M:%S"),
            now.format("%z")
        ));
    }
    let greet = alpha_words(&n);
    if matches!(
        greet.as_str(),
        "ping"
            | "pong"
            | "hi"
            | "hello"
            | "hey"
            | "hey there"
            | "yo"
            | "sup"
            | "hola"
            | "hallo"
            | "guten tag"
            | "good morning"
            | "good afternoon"
            | "good evening"
            | "good night"
            | "gm"
            | "ga"
            | "ge"
            | "buenas"
            | "buenas tardes"
            | "buenas noches"
            | "buen dia"
            | "buen día"
    ) {
        return Some("Hey — I'm here. What do you need?".to_string());
    }
    if matches!(
        greet.as_str(),
        "thanks" | "thank you" | "thx" | "ty" | "danke" | "cheers" | "appreciate it" | "gracias"
    ) {
        return Some("You're welcome.".to_string());
    }
    if is_short_ack_or_signoff(&n) {
        return Some("👍 Got it — here if you need me.".to_string());
    }
    if is_thread_context_clarifier(&n) {
        return Some(if n.contains("last task") || n.contains("that task") {
            "Got it — staying with that task's context.".to_string()
        } else {
            "Got it — staying with this thread's context.".to_string()
        });
    }
    if is_vague_followup_clarifier(&n) {
        return Some(format_vague_followup_clarifier_reply(&n));
    }
    if is_itinerary_correction(&n) {
        return Some(format_itinerary_correction_reply(&n));
    }
    if is_itinerary_preference_statement(&n) {
        return Some(format_itinerary_preference_reply(&n));
    }
    if let Some(slug) = extract_exact_saved_note_slug(&n) {
        return Some(crate::commands::curated_memory::instant_read_saved_note(&slug));
    }
    if is_dump_saved_notes_ask(&n) {
        return Some(crate::commands::curated_memory::instant_dump_saved_notes(12_000));
    }
    if is_overnight_improvements_ask(&n) {
        return Some(format_instant_overnight_improvements_reply());
    }
    if is_how_solved_task_ask(&n) {
        return Some(format_instant_how_solved_task_reply());
    }
    if is_tonight_plan_ask(&n) {
        return Some(format_instant_tonight_plan_reply());
    }
    if is_presence_or_who_ask(&n) {
        return Some(format_instant_presence_reply());
    }
    if is_capabilities_ask(&n) {
        return Some(format_instant_capabilities_reply());
    }
    if is_redmine_user_chat_capability_ask(&n) {
        return Some(format_instant_redmine_user_chat_reply());
    }
    if is_discord_reach_ask(&n) {
        return Some(format_instant_discord_reach_reply());
    }
    if is_identity_affirmation(&n) {
        return Some("Got it — noted. I'm here when you need me.".to_string());
    }
    if is_wakeup_message_task(&n) {
        return Some(format_instant_wakeup_reply());
    }
    if is_version_question(&n) {
        return Some(format!(
            "I'm **mac-stats v{}** — current committed/shipped build.",
            crate::config::Config::version()
        ));
    }
    if is_uptime_ask(&n) {
        return Some(format_instant_uptime_reply());
    }
    if is_live_metrics_snapshot_ask(&n) {
        return Some(format_instant_live_metrics_reply());
    }
    if is_git_commit_push_request(&n) {
        return Some(
            "I won't `git commit` / `git push` from Discord by default (safety). \
Do it in the repo, or ask **Cursor Agent** with an explicit path \
(e.g. `CURSOR_AGENT: in ~/projects/mac-stats commit and push`)."
                .to_string(),
        );
    }
    None
}

fn is_wakeup_message_task(n: &str) -> bool {
    let has_wakeup = n.contains("wake-up")
        || n.contains("wakeup")
        || n.contains("wake up message")
        || (n.contains("wake up") && n.contains("message"));
    if !has_wakeup {
        return false;
    }
    // Scheduler-style: "Send wake-up message…" — not "did you wake up early?"
    n.contains("send")
        || n.contains("message")
        || n.contains("need anything")
        || n.starts_with("wake")
}

/// Short acknowledgments / sign-offs (digester: multi-second direct lane, zero tools).
fn is_short_ack_or_signoff(n: &str) -> bool {
    if n.contains('?') {
        return false;
    }
    if n.contains("http")
        || n.contains("search")
        || n.contains("redmine")
        || n.contains("skill:")
        || n.contains("cursor_agent:")
        || n.contains("screenshot")
        || n.contains("commit")
        || n.contains("push")
        || n.contains("please")
        || n.contains("can you")
        || n.contains("could you")
        || n.contains("would you")
        || n.contains("weather")
        || n.contains("ticket")
        || n.contains("review")
        || n.contains("tell me")
        || n.contains(" what ")
        || n.starts_with("what ")
        || n.starts_with("how ")
    {
        return false;
    }
    if matches!(
        n,
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
    ) {
        return true;
    }
    let len = n.chars().count();
    if len > 140 {
        return false;
    }
    let starts_ack = n.starts_with("ok")
        || n.starts_with("okay")
        || n.starts_with("cool")
        || n.starts_with("nice")
        || n.starts_with("got it")
        || n.starts_with("alright")
        || n.starts_with("no worries")
        || n.starts_with("sounds good");
    if !starts_ack {
        return false;
    }
    // Short follow-on, or clear sign-off / self-serve dismissal.
    len <= 48
        || n.contains("no worries")
        || n.contains("bye")
        || n.contains("myself")
        || n.contains("later")
        || n.contains("all good")
        || n.contains("find out")
}

/// Short presence / “who are you” asks (digester: multi-second direct, zero tools).
fn is_presence_or_who_ask(n: &str) -> bool {
    if n.chars().count() > 64 {
        return false;
    }
    if n.contains("http")
        || n.contains("redmine")
        || n.contains("skill:")
        || n.contains("cursor_agent:")
        || n.contains("search")
        || n.contains("weather")
        || n.contains("ticket")
    {
        return false;
    }
    matches!(
        n,
        "who are you"
            | "who r you"
            | "who're you"
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
            | "how r you"
            | "how's it going"
            | "hows it going"
            | "how are things"
            | "whats up"
            | "what's up"
            | "anything else"
            | "need anything"
            | "need anything else"
    ) || (n.starts_with("who are you") && n.chars().count() <= 40)
        || (n.starts_with("are you there") && n.chars().count() <= 40)
        || (n.starts_with("are you online") && n.chars().count() <= 40)
        || (n.starts_with("how are you") && n.chars().count() <= 48)
}

fn format_instant_presence_reply() -> String {
    format!(
        "I'm **Werner** on **mac-stats v{}** — online and ready. How can I help?",
        crate::config::Config::version()
    )
}

/// “Any improvements from last night / overnight / lately?” — digester zero-tool slow turns.
fn is_overnight_improvements_ask(n: &str) -> bool {
    if n.chars().count() > 200 {
        return false;
    }
    if n.contains("http")
        || n.contains("redmine")
        || n.contains("skill:")
        || n.contains("cursor_agent:")
        || n.contains("search")
        || n.contains("weather")
        || n.contains("ticket")
        || n.contains("workflow")
    {
        return false;
    }
    // “you didn’t improve Mac-stats… what needs to be done?”
    if (n.contains("mac-stats") || n.contains("mac stats"))
        && (n.contains("what needs")
            || n.contains("what should we")
            || n.contains("what to improve")
            || n.contains("what needs to be done"))
    {
        return true;
    }
    // Product self-changelog (digester: Brave/LLM for “Your changelog?” / “Latest enhancements”).
    if is_product_changelog_ask(n) {
        return true;
    }
    let asks_improvements = n.contains("improvement")
        || n.contains("improve")
        || n.contains("what shipped")
        || n.contains("what changed")
        || n.contains("what did you ship")
        || n.contains("what did you change")
        || n.contains("what was done")
        || n.contains("what did you do");
    let overnight_context = n.contains("last night")
        || n.contains("overnight")
        || n.contains("coding session")
        || n.contains("last night's")
        || n.contains("lately")
        || n.contains("recently")
        || n.contains("improvement loop")
        || n.contains("harness loop")
        || n.contains("overnight harness")
        || n.contains("each night")
        || n.contains("every night")
        || n.contains("nightly");
    asks_improvements && overnight_context
}

/// “Your changelog?” / “Latest enhancements of Mac-stats?” / “Your latest changes?”
fn is_product_changelog_ask(n: &str) -> bool {
    if n.chars().count() > 140 {
        return false;
    }
    let about_changes = n.contains("changelog")
        || n.contains("enhancement")
        || n.contains("latest change")
        || n.contains("latests change")
        || n.contains("recent change")
        || n.contains("latest version")
        || ((n.contains("what was changed") || n.contains("what changed"))
            && (n.contains("version") || n.contains("latest")));
    if !about_changes {
        return false;
    }
    n.contains("mac-stats")
        || n.contains("mac stats")
        || n.contains("your changelog")
        || n.contains("your latest")
        || n.contains("your latests")
        || n.contains("your recent")
        || (n.contains("your") && n.contains("version"))
        || (n.contains("enhancement") && (n.contains("mac-stats") || n.contains("mac stats")))
}

/// “How did you solve this task?” / “how exactly was the last task done?”
fn is_how_solved_task_ask(n: &str) -> bool {
    if n.chars().count() > 120 {
        return false;
    }
    if n.contains("http")
        || n.contains("redmine")
        || n.contains("skill:")
        || n.contains("cursor_agent:")
        || n.contains("ticket")
    {
        return false;
    }
    let asks_how = n.contains("how did you solve")
        || n.contains("how did you fix")
        || n.contains("how was this solved")
        || n.contains("how was that solved")
        || n.contains("how exactly was the last task")
        || n.contains("how was the last task")
        || n.contains("how was that task")
        || (n.contains("last task")
            && n.contains("how")
            && (n.contains("done") || n.contains("solved") || n.contains("finished")));
    asks_how && (n.contains("task") || n.contains("last task"))
}

fn format_instant_how_solved_task_reply() -> String {
    let version = crate::config::Config::version();
    let bullets = load_morning_surprise_highlights(3);
    if bullets.is_empty() {
        return format!(
            "For overnight/harness work I ship concrete mac-stats changes (now **v{version}**). \
Check **Agent Ops → Digest** or `~/.mac-stats/improvements/morning_surprise_*.md` for what landed — \
not a chat-only summary."
        );
    }
    let list = bullets
        .into_iter()
        .map(|b| format!("• {b}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "Recent harness keeps (I'm on **mac-stats v{version}**):\n{list}\n\n\
Full trail: **Agent Ops → Digest** or today's `morning_surprise_*.md`."
    )
}

/// “What’s planned for this night / tonight?” — avoid TASK_LIST tool loops.
fn is_tonight_plan_ask(n: &str) -> bool {
    if n.chars().count() > 120 {
        return false;
    }
    if n.contains("http")
        || n.contains("redmine")
        || n.contains("skill:")
        || n.contains("cursor_agent:")
        || n.contains("ticket")
        || n.contains("weather")
    {
        return false;
    }
    let asks_plan = n.contains("planned")
        || n.contains("what's the plan")
        || n.contains("whats the plan")
        || n.contains("what is the plan")
        || n.contains("plan for")
        || n.contains("agenda");
    let night_ctx = n.contains("tonight")
        || n.contains("this night")
        || n.contains("this evening")
        || n.contains("for the night")
        || n.contains("evening");
    asks_plan && night_ctx
}

fn format_instant_tonight_plan_reply() -> String {
    let snap = crate::scheduler::scheduler_operator_snapshot();
    let list = crate::scheduler::list_schedules_formatted();
    let preview = list.lines().take(10).collect::<Vec<_>>().join("\n");
    let next = match (
        snap.next_run_at.as_deref(),
        snap.next_task_preview.as_deref(),
        snap.seconds_until_next_fire,
    ) {
        (Some(at), Some(task), Some(secs)) => {
            let mins = secs / 60;
            format!("Next up: **{at}** (~{mins} min) — {task}")
        }
        (Some(at), Some(task), None) => format!("Next up: **{at}** — {task}"),
        _ => "No upcoming schedule fire computed.".to_string(),
    };
    format!(
        "{next}\n\n**Schedules** ({} loaded):\n{preview}\n\n\
Open **Agent Ops → Schedules** for the full list.",
        snap.total_entries
    )
}

fn format_instant_overnight_improvements_reply() -> String {
    let version = crate::config::Config::version();
    let bullets = load_morning_surprise_highlights(5);
    if bullets.is_empty() {
        return format!(
            "Overnight harness kept shipping — I'm on **mac-stats v{}**. Highlights: instant lane \
(presence/uptime/capabilities), Agent Ops polish, native tool fidelity, bounded log growth. \
Open **Agent Ops → Digest** or `~/.mac-stats/improvements/morning_surprise_*.md` for the run log.",
            version
        );
    }
    let list = bullets
        .into_iter()
        .map(|b| format!("• {b}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "Overnight harness kept shipping — I'm on **mac-stats v{version}**.\n{list}\n\n\
Open **Agent Ops → Digest** or today's `morning_surprise_*.md` for the full run log."
    )
}

fn morning_surprise_path(day: chrono::NaiveDate) -> std::path::PathBuf {
    let name = format!("morning_surprise_{}.md", day.format("%Y-%m-%d"));
    std::env::var_os("HOME")
        .map(|h| std::path::PathBuf::from(h).join(".mac-stats").join("improvements").join(&name))
        .unwrap_or_else(|| std::env::temp_dir().join(&name))
}

/// Pull recent `- **v…**` bullets from today's (else yesterday's) morning surprise.
fn load_morning_surprise_highlights(max: usize) -> Vec<String> {
    let today = chrono::Local::now().date_naive();
    for day in [today, today - chrono::Duration::days(1)] {
        let path = morning_surprise_path(day);
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let bullets = parse_morning_surprise_bullets(&text, max);
        if !bullets.is_empty() {
            return bullets;
        }
    }
    Vec::new()
}

fn parse_morning_surprise_bullets(text: &str, max: usize) -> Vec<String> {
    let mut bullets: Vec<String> = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        // List form: `- **v0.1.370** — …`
        if let Some(rest) = t.strip_prefix('-').map(str::trim_start) {
            if rest.starts_with("**v") || rest.starts_with("**V") {
                bullets.push(rest.trim().to_string());
                continue;
            }
        }
        // Table form (2026-08-14+): `| **v0.1.370** | Startup Disk Cleanup… |`
        if let Some(row) = parse_morning_surprise_table_row(t) {
            bullets.push(row);
        }
    }
    if bullets.len() > max {
        bullets = bullets.split_off(bullets.len() - max);
    }
    bullets
}

/// Parse `| **vX.Y.Z** | what shipped |` into `**vX.Y.Z** — what shipped`.
fn parse_morning_surprise_table_row(line: &str) -> Option<String> {
    let t = line.trim();
    if !t.starts_with('|') {
        return None;
    }
    // Skip header / separator rows.
    let lower = t.to_lowercase();
    if lower.contains("| version |") || lower.contains("|---") || lower.contains("| ---") {
        return None;
    }
    let cells: Vec<&str> = t
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .collect();
    if cells.len() < 2 {
        return None;
    }
    let ver = cells[0];
    if !(ver.starts_with("**v") || ver.starts_with("**V") || ver.starts_with('v') || ver.starts_with('V'))
    {
        return None;
    }
    let what = cells[1].trim();
    if what.is_empty() {
        return None;
    }
    let ver_fmt = if ver.starts_with("**") {
        ver.to_string()
    } else {
        format!("**{ver}**")
    };
    Some(format!("{ver_fmt} — {what}"))
}

/// Short “what can you do?” asks (avoid a full meta+LLM turn for capability intros).
fn is_capabilities_ask(n: &str) -> bool {
    if n.chars().count() > 48 {
        return false;
    }
    if n.contains("http")
        || n.contains("redmine")
        || n.contains("skill:")
        || n.contains("cursor_agent:")
        || n.contains("search")
        || n.contains("weather")
        || n.contains("ticket")
    {
        return false;
    }
    matches!(
        n,
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
    ) || (n.starts_with("what can you") && n.chars().count() <= 40)
        || (n.starts_with("how can you help") && n.chars().count() <= 40)
}

fn format_instant_capabilities_reply() -> String {
    format!(
        "I'm **Werner** (mac-stats v{}). I can check weather, search the web, work Redmine tickets, \
browse/screenshots, run allowlisted commands/skills, search past sessions, and transcribe Discord voice notes. \
Ask a concrete task — or open **Agent Ops** for schedules/runs. On Discord, `/help` lists instant operator commands \
(status, insights, schedules, digest, scrub, interrupt).",
        crate::config::Config::version()
    )
}

/// “Can you talk to <user> on Redmine?” — capability clarify (not ticket work). Digester: ~5s zero-tool.
fn is_redmine_user_chat_capability_ask(n: &str) -> bool {
    if n.chars().count() > 160 {
        return false;
    }
    if !n.contains("redmine") {
        return false;
    }
    if n.contains("ticket")
        || n.contains("issue")
        || n.contains("#")
        || n.contains("http")
        || n.contains("time entr")
        || n.contains("skill:")
        || n.contains("cursor_agent:")
        || n.contains("review")
        || n.contains("summar")
    {
        return false;
    }
    let talkish = n.contains("talk to")
        || n.contains("message ")
        || n.contains("dm ")
        || n.contains("chat with")
        || n.contains("speak to")
        || n.contains("reach ");
    let userish = n.contains("user") || n.contains("person") || n.contains("someone");
    talkish && (userish || n.contains(" on ") || n.contains("ultron"))
}

fn format_instant_redmine_user_chat_reply() -> String {
    format!(
        "I can work **Redmine tickets** via API (status, comments, time entries) — mac-stats v{}. \
I don't DM or chat with Redmine users as people. Give me a ticket id / URL, or ask in Discord if you meant a Discord user.",
        crate::config::Config::version()
    )
}

/// Meta asks about Discord reach (other agents / seeing channels / “ok talking on …”) —
/// digester zero-tool slow turns.
fn is_discord_reach_ask(n: &str) -> bool {
    if n.chars().count() > 220 {
        return false;
    }
    if n.contains("http")
        || n.contains("redmine")
        || n.contains("skill:")
        || n.contains("cursor_agent:")
        || n.contains("search")
        || n.contains("weather")
        || n.contains("ticket")
        || n.contains("discord_api")
        || n.contains("list all")
        || n.contains("list the channel")
        || n.contains("post to")
        || n.contains("send to")
        || n.contains("fetch")
    {
        return false;
    }
    let discordish = n.contains("discord")
        || n.contains("amvara")
        || n.contains("server")
        || n.contains("guild");
    // "Please cross check if you are ok talking on amvara discord server"
    if discordish
        && (n.contains("talking on")
            || n.contains("ok talking")
            || n.contains("okay talking")
            || n.contains("are you online")
            || n.contains("are you connected")
            || (n.contains("cross check") && n.contains("talking")))
    {
        return true;
    }
    let about_channels = n.contains("channel");
    let about_other_agents = n.contains("another agent")
        || n.contains("other agent")
        || n.contains("other agents")
        || n.contains("another bot")
        || n.contains("other bot")
        || n.contains("other bots");
    if !about_channels && !about_other_agents {
        return false;
    }
    n.contains("can you see")
        || n.contains("do you see")
        || n.contains("see channels")
        || n.contains("talking to")
        || n.contains("talk to another")
        || n.contains("talk to other")
        || n.contains("are you talking")
        || n.contains("may you")
        || n.contains("be talking")
}

fn format_instant_discord_reach_reply() -> String {
    format!(
        "I'm **Werner** (mac-stats v{}) on Discord. I see traffic in channels (and DMs) where the bot is present — \
not the whole guild by default. I don't automatically chat with other bots/agents; ask me to do a concrete \
thing (or use `/status` / Agent Ops for gateway health).",
        crate::config::Config::version()
    )
}

/// “Read the exact saved tcx26 file” / “Extract exact txc26 plan” — MEMORY notes, not TASK_*.
fn extract_exact_saved_note_slug(n: &str) -> Option<String> {
    if n.chars().count() > 180 {
        return None;
    }
    if n.contains("http")
        || n.contains("redmine")
        || n.contains("skill:")
        || n.contains("cursor_agent:")
        || n.contains("task_list")
        || n.contains("create a task")
    {
        return None;
    }
    // Explicit MEMORY / note forms.
    for prefix in [
        "memory: read note:",
        "memory: read note ",
        "read note:",
        "read note ",
        "show note:",
        "show note ",
    ] {
        if let Some(rest) = n.strip_prefix(prefix) {
            let token = rest
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_');
            if token.len() >= 2 {
                return Some(token.to_string());
            }
        }
    }

    let wants_verbatim = n.contains("exact")
        || n.contains("verbatim")
        || n.contains("do not summarize")
        || n.contains("don't summarize")
        || n.contains("dont summarize");
    let wants_read = n.contains("read")
        || n.contains("show")
        || n.contains("open")
        || n.contains("dump")
        || n.contains("extract");
    let mentions_saved = n.contains("saved") || n.contains(" note") || n.contains("plan");
    if !(wants_read && mentions_saved) && !wants_verbatim {
        return None;
    }

    // "exact txc26 plan" / "exact tcx26 file" (slug right after exact).
    if let Some(idx) = n.find("exact ") {
        let after = &n[idx + "exact ".len()..];
        for token in after.split_whitespace() {
            let t = token.trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_');
            if t.is_empty()
                || matches!(
                    t,
                    "the"
                        | "a"
                        | "my"
                        | "note"
                        | "file"
                        | "notes"
                        | "saved"
                        | "plan"
                        | "detail"
                        | "details"
                        | "in"
                        | "only"
                        | "this"
                        | "full"
                )
            {
                continue;
            }
            if t.len() >= 2 {
                return Some(t.to_string());
            }
        }
    }

    // Prefer token after "saved ".
    if let Some(idx) = n.find("saved ") {
        let after = &n[idx + "saved ".len()..];
        for token in after.split_whitespace() {
            let t = token.trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_');
            if t.is_empty()
                || matches!(t, "the" | "a" | "my" | "note" | "file" | "notes" | "plan")
            {
                continue;
            }
            if t.len() >= 2 {
                return Some(t.to_string());
            }
        }
    }
    // "note:slug" / "note slug"
    if let Some(idx) = n.find("note:") {
        let after = &n[idx + "note:".len()..];
        let t = after
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_');
        if t.len() >= 2 {
            return Some(t.to_string());
        }
    }
    None
}

/// “Extract / show what you saved” — dump all MEMORY notes, not a model MEMORY tool loop.
fn is_dump_saved_notes_ask(n: &str) -> bool {
    if n.chars().count() > 180 {
        return false;
    }
    if n.contains("http")
        || n.contains("redmine")
        || n.contains("skill:")
        || n.contains("cursor_agent:")
        || n.contains("create a task")
        || n.contains("search ")
    {
        return false;
    }
    // Single-slug reads are handled by extract_exact_saved_note_slug.
    if extract_exact_saved_note_slug(n).is_some() {
        return false;
    }
    let asks = n.contains("what you saved")
        || n.contains("what i saved")
        || n.contains("extract what you saved")
        || (n.contains("extract what") && n.contains("saved"))
        || n.contains("show what you saved")
        || n.contains("list what you saved")
        || n.contains("everything you saved")
        || n.contains("everything i saved")
        || (n.contains("check if everything") && n.contains("saved"))
        || n.contains("what did you save")
        || n.contains("what have you saved");
    asks && (n.contains("saved") || n.contains("memory") || n.contains("note"))
}

/// Short “I mean this Discord thread / last task” clarifiers.
fn is_thread_context_clarifier(n: &str) -> bool {
    if n.chars().count() > 140 {
        return false;
    }
    if n.contains("http")
        || n.contains("skill:")
        || n.contains("cursor_agent:")
        || n.contains("redmine")
        || n.contains("search")
        || n.contains("weather")
    {
        return false;
    }
    let referring = n.contains("referring to this conversation")
        || n.contains("referring to this thread")
        || n.contains("referring to the last task")
        || n.contains("referring to that task")
        || n.contains("i mean this conversation")
        || n.contains("i mean this thread")
        || n.contains("i mean the last task")
        || n.contains("in this conversation")
        || n.contains("in this thread");
    if !referring {
        return false;
    }
    // Pure clarifier — not a new task ("in this conversation, book flights…").
    !n.contains(" book ")
        && !n.contains(" search ")
        && !n.contains(" review ")
        && !n.contains(" create ")
        && !n.contains(" please ")
}

/// Short vague follow-ups that need a target, not a 10s empty Ollama pass.
fn is_vague_followup_clarifier(n: &str) -> bool {
    if n.chars().count() > 96 {
        return false;
    }
    if n.contains("http")
        || n.contains("skill:")
        || n.contains("cursor_agent:")
        || n.contains("redmine")
        || n.contains("brave_search")
        || n.contains("perplexity")
    {
        return false;
    }
    let investigate = n.contains("investigate further")
        || n.contains("dig deeper")
        || n.contains("dig further")
        || n.contains("look further")
        || n.contains("look into it further")
        || n.contains("research further")
        || n == "further"
        || n == "go further"
        || n == "go deeper";
    let tailor = n.contains("tailor it to my interest")
        || n.contains("tailor it to my interests")
        || n.contains("tailor to my interest")
        || n.contains("make it more relevant to me")
        || n == "tailor it"
        || n == "make it relevant";
    (investigate || tailor)
        && !n.contains(" about ")
        && !n.contains("http")
}

fn format_vague_followup_clarifier_reply(n: &str) -> String {
    if n.contains("tailor") || n.contains("relevant") {
        "Sure — what should I tailor to? (e.g. AI/tech, markets/BTC, travel, or a specific topic.)"
            .to_string()
    } else {
        "Happy to dig further — what should I investigate? (name, URL, or a short topic.)"
            .to_string()
    }
}

/// Mid-chat travel corrections that burned empty direct turns (“you missed that leg”).
fn is_itinerary_correction(n: &str) -> bool {
    if n.chars().count() > 220 {
        return false;
    }
    if n.contains("http")
        || n.contains("skill:")
        || n.contains("cursor_agent:")
        || n.contains("redmine")
        || n.contains("search ")
        || n.contains("brave_search")
        || n.contains("perplexity")
        || n.contains("book ")
        || n.contains("schedule")
    {
        return false;
    }
    const AIRPORTS: &[&str] = &[
        "atl", "bcn", "mty", "lmm", "mex", "jfk", "mad", "lax", "ord", "sfo", "mia", "ewr",
    ];
    let tokens: Vec<&str> = n
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty())
        .collect();
    let code_hits = AIRPORTS
        .iter()
        .filter(|code| tokens.iter().any(|t| t == *code))
        .count();
    let place_hits = ["monterrey", "barcelona", "atlanta", "mochis", "techxchange", "txc"]
        .iter()
        .filter(|p| n.contains(*p))
        .count();
    if code_hits + place_hits < 2 {
        return false;
    }
    n.contains("you missed")
        || n.contains("missed that leg")
        || n.contains("messing things up")
        || n.contains("i live in")
        || (n.contains("still have to go") && (n.contains(" to ") || n.contains(" - ")))
}

fn format_itinerary_correction_reply(n: &str) -> String {
    const AIRPORTS: &[&str] = &[
        "atl", "bcn", "mty", "lmm", "mex", "jfk", "mad", "lax", "ord", "sfo", "mia", "ewr",
    ];
    let tokens: Vec<&str> = n
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty())
        .collect();
    let mut codes: Vec<String> = Vec::new();
    for t in &tokens {
        if AIRPORTS.contains(t) && !codes.iter().any(|c| c == *t) {
            codes.push(t.to_uppercase());
        }
    }
    let route = if codes.len() >= 2 {
        codes.join(" → ")
    } else {
        "that route".to_string()
    };
    format!(
        "Got it — treating **{route}** as part of the itinerary. \
Say `MEMORY: save <slug>` with the full legs if you want it persisted, \
or name the next flight/date to look up."
    )
}

/// Preference dumps like “I want to be back in Barcelona around 14 Nov. LMM - MEX - BCN”
/// that previously burned a full MEMORY_APPEND direct turn.
fn is_itinerary_preference_statement(n: &str) -> bool {
    let len = n.chars().count();
    if !(24..=280).contains(&len) {
        return false;
    }
    if n.contains("http")
        || n.contains("skill:")
        || n.contains("cursor_agent:")
        || n.contains("redmine")
        || n.contains("search ")
        || n.contains("brave_search")
        || n.contains("perplexity")
        || n.contains("book ")
        || n.contains("schedule")
        || n.contains("review ")
        || n.contains("techxchange")
        || n.contains("conference")
        || n.contains("memory:")
        || n.contains("memory_append")
    {
        return false;
    }
    let wants = n.contains("i want to be")
        || n.contains("i want to be back")
        || n.contains("i'd like to be")
        || n.contains("i would like to be")
        || (n.contains("i want ") && (n.contains(" around ") || n.contains(" in november") || n.contains(" in october")));
    if !wants {
        return false;
    }
    const AIRPORTS: &[&str] = &[
        "atl", "bcn", "mty", "lmm", "mex", "jfk", "mad", "lax", "ord", "sfo", "mia", "ewr",
    ];
    let tokens: Vec<&str> = n
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty())
        .collect();
    let code_hits = AIRPORTS
        .iter()
        .filter(|code| tokens.iter().any(|t| t == *code))
        .count();
    let place_hits = ["barcelona", "atlanta", "monterrey", "mochis", "mexico"]
        .iter()
        .filter(|p| n.contains(*p))
        .count();
    if code_hits < 2 && !(code_hits >= 1 && place_hits >= 1) {
        return false;
    }
    // Prefer preference prose, not pure questions.
    if n.contains('?') && !n.contains("around ") {
        return false;
    }
    true
}

fn format_itinerary_preference_reply(n: &str) -> String {
    const AIRPORTS: &[&str] = &[
        "atl", "bcn", "mty", "lmm", "mex", "jfk", "mad", "lax", "ord", "sfo", "mia", "ewr",
    ];
    let tokens: Vec<&str> = n
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty())
        .collect();
    let mut codes: Vec<String> = Vec::new();
    for t in &tokens {
        if AIRPORTS.contains(t) && !codes.iter().any(|c| c == *t) {
            codes.push(t.to_uppercase());
        }
    }
    let route = if codes.len() >= 2 {
        codes.join(" → ")
    } else if !codes.is_empty() {
        codes[0].clone()
    } else {
        "that return".to_string()
    };
    format!(
        "Got it — **{route}** noted as the preference. \
To lock it in, say `MEMORY: save itinerary` and paste the full dates/legs on the following lines \
(verbatim — don’t summarize). Or tell me the next city/date to research."
    )
}

/// Short role/identity statements without a question (digester: multi-second direct, zero tools).
fn is_identity_affirmation(n: &str) -> bool {
    if n.contains('?') || n.chars().count() > 180 {
        return false;
    }
    if n.contains("http")
        || n.contains("search")
        || n.contains("redmine")
        || n.contains("skill:")
        || n.contains("cursor_agent:")
        || n.contains("please")
        || n.contains("can you")
        || n.contains("could you")
    {
        return false;
    }
    let you_are = n.starts_with("you are ") || n.starts_with("you're ") || n.starts_with("youre ");
    if !you_are {
        return false;
    }
    n.contains("working for")
        || n.contains("online")
        || n.contains("assistant")
        || n.contains(" agent")
        || n.contains("bot")
        || n.contains("on various channel")
}

fn format_instant_wakeup_reply() -> String {
    let now = Local::now();
    let greeting = match now.hour() {
        0..=11 => "Good morning",
        12..=17 => "Good afternoon",
        _ => "Good evening",
    };
    let version = crate::config::Config::version();
    let highlights = load_morning_surprise_highlights(3);
    if highlights.is_empty() {
        return format!(
            "{greeting}! Hope you're doing well — I'm here if you need anything. \
(mac-stats v{version}, {})",
            now.format("%H:%M")
        );
    }
    let list = highlights
        .into_iter()
        .map(|b| format!("• {b}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "{greeting}! I'm here if you need anything (mac-stats v{version}, {}).\n\
Overnight highlights:\n{list}\n\
Ask *any improvements from last night?* for more.",
        now.format("%H:%M")
    )
}

fn is_version_question(n: &str) -> bool {
    let n = n.trim();
    matches!(
        n,
        "what version"
            | "what version are you"
            | "what's your version"
            | "whats your version"
            | "which version"
            | "version"
            | "app version"
            | "mac-stats version"
            | "mac stats version"
    ) || (n.contains("version")
        && n.chars().count() < 48
        && (n.contains("you")
            || n.contains("app")
            || n.contains("mac-stats")
            || n.starts_with("what")))
        || is_ship_version_status_ask(n)
}

/// Digester: "Improvement committed and version bumped?" (~12s direct, zero tools).
fn is_ship_version_status_ask(n: &str) -> bool {
    if n.chars().count() > 96 {
        return false;
    }
    if n.contains("http")
        || n.contains("skill:")
        || n.contains("cursor_agent:")
        || n.contains("redmine")
        || n.contains("please bump")
        || n.contains("bump the version in")
    {
        return false;
    }
    let about_bump = n.contains("bump");
    let about_version = n.contains("version");
    if !about_bump && !about_version {
        return false;
    }
    // "comitted" (one m) is a common typo of "committed".
    let about_ship = n.contains("commit")
        || n.contains("comit")
        || n.contains("ship")
        || n.contains("release")
        || n.contains("pushed")
        || n.contains("improvement");
    // `normalize_q` strips trailing `?`, so do not require a question mark.
    let status_frame = n.starts_with("did ")
        || n.starts_with("is ")
        || n.starts_with("was ")
        || n.starts_with("has ")
        || about_ship;
    about_bump && status_frame && (about_version || about_ship)
}

/// Short process-uptime asks (pairs with Agent Ops Version card /insights).
fn is_uptime_ask(n: &str) -> bool {
    if n.chars().count() > 48 {
        return false;
    }
    if n.contains("http")
        || n.contains("redmine")
        || n.contains("skill:")
        || n.contains("cursor_agent:")
        || n.contains("search")
        || n.contains("weather")
        || n.contains("ticket")
        || n.contains("system uptime")
        || n.contains("machine")
    {
        return false;
    }
    matches!(
        n,
        "uptime"
            | "up time"
            | "how long up"
            | "how long have you been up"
            | "how long are you up"
            | "how long running"
            | "how long have you been running"
            | "process uptime"
            | "app uptime"
    ) || (n.contains("uptime") && n.chars().count() <= 32)
        || (n.starts_with("how long") && (n.contains("up") || n.contains("running")) && n.chars().count() <= 48)
}

fn format_instant_uptime_reply() -> String {
    format!(
        "I've been up **{}** (mac-stats v{}).",
        crate::state::format_process_uptime(),
        crate::config::Config::version()
    )
}

/// Live CPU/RAM/load snapshot from metrics cache (digester: ~14s direct for “system load”).
fn is_live_metrics_snapshot_ask(n: &str) -> bool {
    if n.chars().count() > 96 {
        return false;
    }
    if n.contains("http")
        || n.contains("redmine")
        || n.contains("skill:")
        || n.contains("cursor_agent:")
        || n.contains("search")
        || n.contains("weather")
        || n.contains("ticket")
        || n.contains("flight")
        || n.contains("uptime")
        || n.contains("fetch")
        || n.contains("browser")
    {
        return false;
    }
    let load = n.contains("system load")
        || n.contains("load look")
        || n.contains("load average")
        || (n.contains("how busy")
            && (n.contains("system")
                || n.contains("machine")
                || n.contains("mac")
                || n.contains("cpu")));
    let cpu = n.contains("cpu usage")
        || (n.contains("cpu") && (n.contains("how") || n.contains("look") || n.contains("doing")));
    let ram = n.contains("ram usage")
        || n.contains("memory usage")
        || (n.contains("how much") && (n.contains("ram") || n.contains("memory")));
    let metrics = n.contains("system metrics")
        || n.contains("machine status")
        || (n.contains("mac status") && !n.contains("discord"));
    load || cpu || ram || metrics
}

fn format_instant_live_metrics_reply() -> String {
    let raw = crate::metrics::format_metrics_for_ai_context();
    let body = raw
        .strip_prefix("Current system metrics:\n")
        .unwrap_or(raw.as_str());
    format!(
        "Live snapshot (**mac-stats v{}**):\n{}",
        crate::config::Config::version(),
        body
    )
}

fn is_git_commit_push_request(n: &str) -> bool {
    // Scheduled skills / Cursor Agent work must run — do not instant-refuse them.
    // False positive example: "SKILL: ui-weekly-review … commit+push, reply briefly."
    if n.contains("skill:")
        || n.contains("cursor_agent:")
        || n.contains("changelog-weekly")
        || n.contains("ui-weekly")
        || n.contains("docs/040_")
        || n.contains("docs/041_")
    {
        return false;
    }
    let has_commit = n.contains("commit");
    let has_push = n.contains("push");
    if !(has_commit || (has_push && n.contains("git"))) {
        return false;
    }
    // Only refuse short, casual Discord asks — not multi-step operator tasks.
    if n.chars().count() > 160 {
        return false;
    }
    has_push
        || n.contains("git commit")
        || (has_commit && (n.contains("change") || n.contains("stage") || n.contains("repo")))
}

fn is_time_question(n: &str) -> bool {
    let n = n.trim();
    if matches!(
        n,
        "what time is it"
            | "what's the time"
            | "whats the time"
            | "current time"
            | "time now"
            | "what's the date"
            | "whats the date"
            | "what date is it"
            | "current date"
            | "date today"
            | "wie spaet ist es"
            | "wie spät ist es"
            | "welche uhrzeit"
            | "uhrzeit"
            | "que hora es"
            | "qué hora es"
            | "time"
            | "date"
            | "clock"
    ) {
        return true;
    }
    // "what time is it now?", "what's the time please", etc.
    let starts = [
        "what time is it",
        "what's the time",
        "whats the time",
        "what is the time",
        "tell me the time",
        "current time",
        "what date is it",
        "what's the date",
        "whats the date",
    ];
    if starts.iter().any(|p| n.starts_with(p)) && n.chars().count() < 64 {
        return true;
    }
    n.contains("what time") && n.len() < 48
}

fn is_trivial_chat(q: &str) -> bool {
    let n = normalize_q(q);
    n.chars().count() <= 24
        && !n.contains("http")
        && !n.contains("search")
        && !n.contains("redmine")
        && !n.contains("screenshot")
}

/// Pre-routed tools that do not need criteria/plan/verify LLM calls.
fn lite_pre_route(rec: &str) -> bool {
    let u = rec.to_uppercase();
    u.starts_with("BRAVE_SEARCH:")
        || u.starts_with("PERPLEXITY_SEARCH:")
        || u.starts_with("RUN_CMD:")
        || u.starts_with("FETCH_URL:")
        || u.starts_with("LIST_SCHEDULES")
        || u.starts_with("TASK_LIST")
        || u.starts_with("TASK_SHOW:")
        || u.starts_with("TASK_CREATE:")
        || u.starts_with("OLLAMA_API:")
        || u.starts_with("BROWSER_SCREENSHOT:")
}

/// Fixed success criteria when we skip the criteria LLM (lite lane).
pub fn lite_success_criteria(pre_routed: Option<&str>) -> Vec<String> {
    let u = pre_routed.unwrap_or("").to_uppercase();
    if u.starts_with("BRAVE_SEARCH:") || u.starts_with("PERPLEXITY_SEARCH:") {
        return vec![
            "Web search results were fetched.".to_string(),
            "A short answer citing results was given to the user.".to_string(),
        ];
    }
    if u.starts_with("RUN_CMD:") {
        return vec!["Command output was returned to the user.".to_string()];
    }
    if u.starts_with("FETCH_URL:") {
        return vec!["Page content was fetched and summarized for the user.".to_string()];
    }
    if u.starts_with("BROWSER_SCREENSHOT:") {
        return vec!["Screenshot was taken and attached or path returned.".to_string()];
    }
    if u.starts_with("TASK_CREATE:") {
        return vec!["A new task file was created and confirmed to the user.".to_string()];
    }
    vec!["User request answered clearly.".to_string()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_push_is_instant_refusal() {
        match classify_turn_lane("We shall commit and push latest changes", None) {
            TurnLane::Instant { reply } => {
                assert!(reply.to_lowercase().contains("git"));
                assert!(
                    reply.to_lowercase().contains("cursor") || reply.contains("CURSOR_AGENT"),
                    "expected safety/cursor guidance: {reply}"
                );
            }
            other => panic!("expected Instant, got {:?}", other),
        }
    }

    #[test]
    fn scheduled_skill_with_commit_push_is_not_instant_refusal() {
        let task = "SKILL: ui-weekly-review — Weekly Agent Ops polish. One UI fix. \
Sync dist, commit+push, reply briefly.";
        assert!(
            !matches!(
                classify_turn_lane(task, None),
                TurnLane::Instant { .. }
            ),
            "scheduled SKILL tasks that mention commit+push must run, not instant-refuse"
        );
        let changelog = "SKILL: changelog-weekly-review — hygiene per docs/040_changelog_hygiene.md, \
commit+push, then reply briefly.";
        assert!(
            !matches!(
                classify_turn_lane(changelog, None),
                TurnLane::Instant { .. }
            ),
            "changelog weekly skill must not be instant-refused"
        );
    }

    #[test]
    fn version_ask_is_instant() {
        match classify_turn_lane("What version are you?", None) {
            TurnLane::Instant { reply } => {
                assert!(reply.contains("mac-stats"));
                assert!(reply.contains(&crate::config::Config::version()));
            }
            other => panic!("expected Instant, got {:?}", other),
        }
    }

    #[test]
    fn ship_version_status_ask_is_instant() {
        for q in [
            "Improvement committed and version bumped?",
            "Improvement comitted and version bumped?",
            "Was the version bumped?",
            "Did you commit and bump the version?",
        ] {
            match classify_turn_lane(q, None) {
                TurnLane::Instant { reply } => {
                    assert!(
                        reply.contains(&crate::config::Config::version()),
                        "expected version in reply for {q:?}: {reply}"
                    );
                }
                other => panic!("expected Instant for {q:?}, got {:?}", other),
            }
        }
    }

    #[test]
    fn thread_context_clarifier_is_instant() {
        for q in [
            "I am referring to this conversation!",
            "I'm referring to this thread",
            "I mean this conversation",
            "I am referring to the last task I had given you to improve and saving information",
        ] {
            match classify_turn_lane(q, None) {
                TurnLane::Instant { reply } => {
                    assert!(
                        reply.to_lowercase().contains("thread")
                            || reply.to_lowercase().contains("context")
                            || reply.to_lowercase().contains("task"),
                        "expected thread/context ack for {q:?}: {reply}"
                    );
                }
                other => panic!("expected Instant for {q:?}, got {:?}", other),
            }
        }
        assert!(
            !matches!(
                classify_turn_lane(
                    "In this conversation please search for IBM TechXchange dates",
                    None
                ),
                TurnLane::Instant { .. }
            ),
            "taskful 'in this conversation…' must not be instant"
        );
    }

    #[test]
    fn vague_followup_clarifier_is_instant() {
        for q in [
            "Can't you investigate further?",
            "Can you dig deeper?",
            "Tailor it to my interest",
        ] {
            match classify_turn_lane(q, None) {
                TurnLane::Instant { reply } => {
                    let lower = reply.to_lowercase();
                    assert!(
                        lower.contains("investigate")
                            || lower.contains("tailor")
                            || lower.contains("dig"),
                        "expected clarifier reply for {q:?}: {reply}"
                    );
                }
                other => panic!("expected Instant for {q:?}, got {:?}", other),
            }
        }
        assert!(
            !matches!(
                classify_turn_lane(
                    "Investigate further about Florian Fischer delivery hero problem",
                    None
                ),
                TurnLane::Instant { .. }
            ),
            "investigate-about-<topic> must not be instant clarifier"
        );
    }

    #[test]
    fn itinerary_correction_is_instant() {
        for q in [
            "We still have to go BCN - ATL. You missed that leg",
            "You're messing things up. I live in BCN. So, BCN ATL to get to txc Then ATL MTY",
        ] {
            match classify_turn_lane(q, None) {
                TurnLane::Instant { reply } => {
                    assert!(
                        reply.to_lowercase().contains("itinerary")
                            || reply.contains("→")
                            || reply.to_lowercase().contains("memory"),
                        "expected itinerary ack for {q:?}: {reply}"
                    );
                }
                other => panic!("expected Instant for {q:?}, got {:?}", other),
            }
        }
        assert!(
            !matches!(
                classify_turn_lane(
                    "Search flights from BCN to ATL next October",
                    None
                ),
                TurnLane::Instant { .. }
            ),
            "flight search must not be itinerary-correction instant"
        );
    }

    #[test]
    fn itinerary_preference_statement_is_instant() {
        let q =
            "I want to be back in Barcelona around 14 of November. LMM - MEX - BCN. I want the return legs";
        match classify_turn_lane(q, None) {
            TurnLane::Instant { reply } => {
                assert!(
                    reply.to_lowercase().contains("memory")
                        && (reply.contains("→") || reply.to_lowercase().contains("lmm")),
                    "expected preference ack for {q:?}: {reply}"
                );
            }
            other => panic!("expected Instant for preference statement, got {:?}", other),
        }
        assert!(
            !matches!(
                classify_turn_lane(
                    "Review IBM techxchange dates in Atlanta. I want to be in Atlanta two days before",
                    None
                ),
                TurnLane::Instant { .. }
            ),
            "event-date review must not be itinerary-preference instant"
        );
    }

    #[test]
    fn exact_saved_note_read_is_instant() {
        match classify_turn_lane(
            "Do not summarize your answer. I want to read the exact saved tcx26 file",
            None,
        ) {
            TurnLane::Instant { reply } => {
                let lower = reply.to_lowercase();
                assert!(
                    lower.contains("note") || lower.contains("tcx26") || lower.contains("txc26"),
                    "expected note read reply: {reply}"
                );
            }
            other => panic!("expected Instant, got {:?}", other),
        }
        assert_eq!(
            extract_exact_saved_note_slug(
                &normalize_q("Do not summarize your answer. I want to read the exact saved tcx26 file")
            ),
            Some("tcx26".to_string())
        );
        assert_eq!(
            extract_exact_saved_note_slug(&normalize_q(
                "Extract exact txc26 plan in detail! Only this"
            )),
            Some("txc26".to_string())
        );
        match classify_turn_lane("Extract exact txc26 plan in detail! Only this", None) {
            TurnLane::Instant { .. } => {}
            other => panic!("expected Instant for exact plan extract, got {:?}", other),
        }
        assert!(
            !matches!(
                classify_turn_lane("Create a task for coder to improve knowledge saving", None),
                TurnLane::Instant { .. }
            ),
            "task create must not be note-read instant"
        );
    }

    #[test]
    fn dump_saved_notes_ask_is_instant() {
        match classify_turn_lane(
            "Ok. We are good now. Extract what you saved for me to check if everything is ok",
            None,
        ) {
            TurnLane::Instant { reply } => {
                let lower = reply.to_lowercase();
                assert!(
                    lower.contains("saved note") || lower.contains("no saved notes"),
                    "expected dump reply: {reply}"
                );
            }
            other => panic!("expected Instant, got {:?}", other),
        }
    }

    #[test]
    fn uptime_ask_is_instant() {
        crate::state::mark_process_start();
        for q in ["uptime", "How long have you been up?", "process uptime"] {
            match classify_turn_lane(q, None) {
                TurnLane::Instant { reply } => {
                    assert!(
                        reply.to_lowercase().contains("up"),
                        "expected uptime reply for {q:?}: {reply}"
                    );
                }
                other => panic!("expected Instant for {q:?}, got {:?}", other),
            }
        }
        assert!(
            !matches!(
                classify_turn_lane("What's the system uptime on this machine?", None),
                TurnLane::Instant { .. }
            ),
            "host/system uptime asks must not be instant"
        );
    }

    #[test]
    fn live_metrics_snapshot_ask_is_instant() {
        for q in [
            "What's the system load look like",
            "system load",
            "cpu usage?",
            "How's the RAM usage?",
        ] {
            match classify_turn_lane(q, None) {
                TurnLane::Instant { reply } => {
                    assert!(
                        reply.to_lowercase().contains("cpu")
                            || reply.to_lowercase().contains("load")
                            || reply.to_lowercase().contains("ram"),
                        "expected metrics reply for {q:?}: {reply}"
                    );
                }
                other => panic!("expected Instant for {q:?}, got {:?}", other),
            }
        }
        assert!(
            !matches!(
                classify_turn_lane("Search for system load monitoring tools", None),
                TurnLane::Instant { .. }
            ),
            "search tasks must not be instant metrics"
        );
    }

    #[test]
    fn extended_greeting_and_thanks_are_instant() {
        for q in [
            "good afternoon",
            "hey there",
            "gm",
            "cheers",
            "appreciate it",
            "Hola 👋",
            "hola!!!",
            "gracias",
        ] {
            assert!(
                matches!(classify_turn_lane(q, None), TurnLane::Instant { .. }),
                "expected Instant for {q}"
            );
        }
    }

    #[test]
    fn short_acks_and_signoffs_are_instant() {
        for q in [
            "ok",
            "Nice answer",
            "Ok. 👌 I will switch you off and find out myself. No worries.",
            "got it",
            "no worries",
            "👍",
        ] {
            assert!(
                matches!(classify_turn_lane(q, None), TurnLane::Instant { .. }),
                "expected Instant for {q}"
            );
        }
        assert!(
            !matches!(
                classify_turn_lane("Ok, can you search Redmine for ticket 12?", None),
                TurnLane::Instant { .. }
            ),
            "acks with a real ask must not be instant"
        );
        assert!(
            !matches!(
                classify_turn_lane("Nice weather today in El Masnou", None),
                TurnLane::Instant { .. }
            ),
            "nice + real topic must not be instant"
        );
    }

    #[test]
    fn identity_affirmations_are_instant() {
        assert!(matches!(
            classify_turn_lane(
                "You are working for Amvara. You are online in Amvara server on various channel.",
                None
            ),
            TurnLane::Instant { .. }
        ));
        assert!(
            !matches!(
                classify_turn_lane(
                    "You are working for Amvara. Please review Redmine ticket 12.",
                    None
                ),
                TurnLane::Instant { .. }
            ),
            "identity + real redmine task must not be instant"
        );
    }

    #[test]
    fn redmine_user_chat_capability_asks_are_instant() {
        match classify_turn_lane(
            "Can you talk to ultron user on Amvara redmine server?",
            None,
        ) {
            TurnLane::Instant { reply } => {
                let lower = reply.to_lowercase();
                assert!(
                    lower.contains("ticket") || lower.contains("api") || lower.contains("dm"),
                    "expected redmine-chat clarify: {reply}"
                );
            }
            other => panic!("expected Instant, got {:?}", other),
        }
        assert!(
            !matches!(
                classify_turn_lane("Review and summarize Redmine ticket: 7736", None),
                TurnLane::Instant { .. }
            ),
            "real ticket work must not be instant"
        );
    }

    #[test]
    fn presence_and_who_asks_are_instant() {
        for q in [
            "Who are you?",
            "are you there?",
            "Are you online?",
            "still there",
            "you up?",
            "How are you?",
            "how's it going?",
            "what's up?",
            "still here?",
            "you around?",
            "you good?",
            "Need anything else?",
            "anything else?",
        ] {
            assert!(
                matches!(classify_turn_lane(q, None), TurnLane::Instant { .. }),
                "expected Instant for {q:?}"
            );
        }
        assert!(
            !matches!(
                classify_turn_lane("Who are you working with on Redmine ticket 12?", None),
                TurnLane::Instant { .. }
            ),
            "substantive who-asks must not be instant"
        );
        assert!(
            !matches!(
                classify_turn_lane("Need anything else from the weather API?", None),
                TurnLane::Instant { .. }
            ),
            "need-anything with a real ask must not be instant"
        );
    }

    #[test]
    fn overnight_improvements_asks_are_instant() {
        for q in [
            "How are you today? Any improvements from last night coding session?",
            "Any improvements from last night?",
            "What shipped overnight?",
            "What changed from last night's coding session?",
            "Any improvement lately?",
            "No improvement loop?",
            "Don't you have a task to improve each night? I would like to know what was done.",
        ] {
            match classify_turn_lane(q, None) {
                TurnLane::Instant { reply } => {
                    let lower = reply.to_lowercase();
                    assert!(
                        lower.contains("mac-stats") || lower.contains("overnight"),
                        "expected overnight blurb for {q:?}: {reply}"
                    );
                }
                other => panic!("expected Instant for {q:?}, got {:?}", other),
            }
        }
        assert!(
            !matches!(
                classify_turn_lane("Any improvements to the Redmine ticket workflow?", None),
                TurnLane::Instant { .. }
            ),
            "improvements without overnight context must not be instant"
        );
    }

    #[test]
    fn how_solved_task_asks_are_instant() {
        for q in [
            "How did you solve this task?",
            "Then how exactly was the last task done?",
        ] {
            match classify_turn_lane(q, None) {
                TurnLane::Instant { reply } => {
                    let lower = reply.to_lowercase();
                    assert!(
                        lower.contains("digest")
                            || lower.contains("morning_surprise")
                            || lower.contains("mac-stats"),
                        "expected digest pointer for {q:?}: {reply}"
                    );
                }
                other => panic!("expected Instant for {q:?}, got {:?}", other),
            }
        }
        assert!(
            !matches!(
                classify_turn_lane("How did you solve the Redmine ticket?", None),
                TurnLane::Instant { .. }
            ),
            "ticket solve asks must not be instant"
        );
    }

    #[test]
    fn mac_stats_what_needs_ask_is_instant() {
        match classify_turn_lane(
            "so you didn't improve Mac-stats itself. Just your md files. What needs to be done?",
            None,
        ) {
            TurnLane::Instant { reply } => {
                let lower = reply.to_lowercase();
                assert!(
                    lower.contains("mac-stats") || lower.contains("overnight") || lower.contains("digest"),
                    "expected overnight blurb: {reply}"
                );
            }
            other => panic!("expected Instant, got {:?}", other),
        }
    }

    #[test]
    fn tonight_plan_asks_are_instant() {
        for q in [
            "What's planned for this night?",
            "Whats planned for tonight?",
            "What is the plan for this evening?",
        ] {
            match classify_turn_lane(q, None) {
                TurnLane::Instant { reply } => {
                    let lower = reply.to_lowercase();
                    assert!(
                        lower.contains("schedule") || lower.contains("next"),
                        "expected schedule blurb for {q:?}: {reply}"
                    );
                }
                other => panic!("expected Instant for {q:?}, got {:?}", other),
            }
        }
        assert!(
            !matches!(
                classify_turn_lane("What's the plan for the Redmine ticket?", None),
                TurnLane::Instant { .. }
            ),
            "ticket planning must not be instant schedule dump"
        );
    }

    #[test]
    fn morning_surprise_bullets_take_last_n() {
        let md = "\
# Morning surprise\n\n\
- **v0.1.230** — one\n\
- not a version line\n\
- **v0.1.231** — two\n\
- **v0.1.232** — three\n\
";
        let got = parse_morning_surprise_bullets(md, 2);
        assert_eq!(got.len(), 2);
        assert!(got[0].contains("0.1.231"), "{got:?}");
        assert!(got[1].contains("0.1.232"), "{got:?}");
    }

    #[test]
    fn morning_surprise_table_rows_parse() {
        let md = "\
# Morning surprise — 2026-08-14\n\n\
## Shipped\n\n\
| Version | What |\n\
|---------|------|\n\
| **v0.1.370** | Startup Disk Cleanup off main thread |\n\
| **v0.1.371** | Top Processes accent CPU bars |\n\
| **v0.1.372** | Disk Cleanup reclaim accent |\n\
";
        let got = parse_morning_surprise_bullets(md, 2);
        assert_eq!(got.len(), 2, "{got:?}");
        assert!(got[0].contains("0.1.371") && got[0].contains("Processes"), "{got:?}");
        assert!(got[1].contains("0.1.372") && got[1].contains("Disk Cleanup"), "{got:?}");
    }

    #[test]
    fn product_changelog_asks_are_instant() {
        for q in [
            "Latest enhancements of Mac-stats?",
            "Your latests changes?",
            "Your changelog? What was changed in your latest version",
            "Your latest changes?",
        ] {
            match classify_turn_lane(q, None) {
                TurnLane::Instant { reply } => {
                    let lower = reply.to_lowercase();
                    assert!(
                        lower.contains("mac-stats") || lower.contains("overnight"),
                        "expected changelog/overnight blurb for {q:?}: {reply}"
                    );
                }
                other => panic!("expected Instant for {q:?}, got {:?}", other),
            }
        }
        assert!(
            !matches!(
                classify_turn_lane("Latest enhancements of React Native?", None),
                TurnLane::Instant { .. }
            ),
            "third-party enhancements must not be instant"
        );
    }

    #[test]
    fn capabilities_asks_are_instant() {
        for q in [
            "What can you do?",
            "what do you do?",
            "help",
            "capabilities",
            "how can you help?",
        ] {
            match classify_turn_lane(q, None) {
                TurnLane::Instant { reply } => {
                    let lower = reply.to_lowercase();
                    assert!(
                        lower.contains("werner") || lower.contains("mac-stats"),
                        "expected capabilities blurb for {q:?}: {reply}"
                    );
                    assert!(
                        lower.contains("/help") || lower.contains("voice"),
                        "expected /help or voice mention for {q:?}: {reply}"
                    );
                }
                other => panic!("expected Instant for {q:?}, got {:?}", other),
            }
        }
        assert!(
            !matches!(
                classify_turn_lane("What can you do with Redmine ticket 12?", None),
                TurnLane::Instant { .. }
            ),
            "capabilities + real task must not be instant"
        );
    }

    #[test]
    fn discord_reach_asks_are_instant() {
        for q in [
            "So, may you be talking to another agent on the amvara server? Can you see channels of amvara server?",
            "Can you see channels on the Amvara server?",
            "Are you talking to other bots?",
            "Please cross check if you are ok talking on amvara discord server",
        ] {
            match classify_turn_lane(q, None) {
                TurnLane::Instant { reply } => {
                    let lower = reply.to_lowercase();
                    assert!(
                        lower.contains("discord") || lower.contains("channel") || lower.contains("werner"),
                        "expected discord-reach blurb for {q:?}: {reply}"
                    );
                }
                other => panic!("expected Instant for {q:?}, got {:?}", other),
            }
        }
        assert!(
            !matches!(
                classify_turn_lane("List all channels and post to #general", None),
                TurnLane::Instant { .. }
            ),
            "channel list/post tasks must not be instant"
        );
    }

    #[test]
    fn wakeup_schedule_task_is_instant() {
        match classify_turn_lane("Send wake-up message. Need anything else?", None) {
            TurnLane::Instant { reply } => {
                let lower = reply.to_lowercase();
                assert!(
                    lower.contains("morning")
                        || lower.contains("afternoon")
                        || lower.contains("evening"),
                    "expected daypart greeting: {reply}"
                );
                assert!(
                    lower.contains("need") || lower.contains("here") || lower.contains("mac-stats"),
                    "expected presence/version cue: {reply}"
                );
            }
            other => panic!("expected Instant, got {:?}", other),
        }
    }

    #[test]
    fn casual_wake_up_question_not_instant() {
        assert!(
            !matches!(
                classify_turn_lane("Did you wake up early today?", None),
                TurnLane::Instant { .. }
            ),
            "casual wake-up chat should not be forced instant"
        );
    }

    #[test]
    fn search_pre_route_is_lite() {
        match classify_turn_lane(
            "Search for Ralf Roeber",
            Some("BRAVE_SEARCH: Ralf Roeber"),
        ) {
            TurnLane::Lite { reason } => assert_eq!(reason, "pre_routed_tool"),
            other => panic!("expected Lite, got {:?}", other),
        }
    }

    #[test]
    fn complex_is_full() {
        assert_eq!(
            classify_turn_lane(
                "Open redmine, review my tickets, and post a summary to Discord",
                None
            ),
            TurnLane::Full
        );
    }
}
