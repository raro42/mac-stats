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
    if matches!(
        n.as_str(),
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
    ) {
        return Some("Hey — I'm here. What do you need?".to_string());
    }
    if matches!(
        n.as_str(),
        "thanks" | "thank you" | "thx" | "ty" | "danke" | "cheers" | "appreciate it"
    ) {
        return Some("You're welcome.".to_string());
    }
    if is_short_ack_or_signoff(&n) {
        return Some("👍 Got it — here if you need me.".to_string());
    }
    if is_overnight_improvements_ask(&n) {
        return Some(format_instant_overnight_improvements_reply());
    }
    if is_presence_or_who_ask(&n) {
        return Some(format_instant_presence_reply());
    }
    if is_capabilities_ask(&n) {
        return Some(format_instant_capabilities_reply());
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
            "I'm **mac-stats v{}**.",
            crate::config::Config::version()
        ));
    }
    if is_uptime_ask(&n) {
        return Some(format_instant_uptime_reply());
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

/// “Any improvements from last night / overnight coding?” — digester zero-tool slow turns.
fn is_overnight_improvements_ask(n: &str) -> bool {
    if n.chars().count() > 140 {
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
    let asks_improvements = n.contains("improvement")
        || n.contains("what shipped")
        || n.contains("what changed")
        || n.contains("what did you ship")
        || n.contains("what did you change");
    let overnight_context = n.contains("last night")
        || n.contains("overnight")
        || n.contains("coding session")
        || n.contains("last night's");
    asks_improvements && overnight_context
}

fn format_instant_overnight_improvements_reply() -> String {
    format!(
        "Overnight harness kept shipping — I'm on **mac-stats v{}**. Highlights: instant lane \
(presence/uptime/capabilities), Agent Ops polish, native tool fidelity, bounded log growth. \
Open **Agent Ops → Digest** or `~/.mac-stats/improvements/morning_surprise_*.md` for the run log.",
        crate::config::Config::version()
    )
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
browse/screenshots, run allowlisted commands/skills, search past sessions, and help from Discord or the dashboard. \
Ask a concrete task — or open **Agent Ops** for schedules/runs.",
        crate::config::Config::version()
    )
}

/// Meta asks about Discord reach (other agents / seeing channels) — digester zero-tool slow turns.
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
    format!(
        "{greeting}! Hope you're doing well — I'm here if you need anything. ({})",
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
        && (n.contains("you") || n.contains("app") || n.contains("mac-stats") || n.starts_with("what")))
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
    fn extended_greeting_and_thanks_are_instant() {
        for q in ["good afternoon", "hey there", "gm", "cheers", "appreciate it"] {
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
                classify_turn_lane("Can you talk to ultron user on Amvara redmine server?", None),
                TurnLane::Instant { .. }
            ),
            "identity-adjacent asks must not be instant"
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
                assert!(lower.contains("need") || lower.contains("here"));
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
