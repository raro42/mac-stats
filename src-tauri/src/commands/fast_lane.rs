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
