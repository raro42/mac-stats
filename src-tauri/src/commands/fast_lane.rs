//! Fast / lite lanes: cut GPU meta-calls (criteria, topic, plan, verify) for trivial
//! and pre-routed Discord turns.

use chrono::Local;

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
            | "yo"
            | "sup"
            | "hola"
            | "hallo"
            | "guten tag"
            | "good morning"
            | "good evening"
            | "good night"
    ) {
        return Some("Hey — I'm here. What do you need?".to_string());
    }
    if matches!(n.as_str(), "thanks" | "thank you" | "thx" | "ty" | "danke") {
        return Some("You're welcome.".to_string());
    }
    if is_git_commit_push_request(&n) {
        return Some(
            "I can't run `git commit` / `git push` from Discord — `git` isn't on the RUN_CMD allowlist \
(only things like `date`, `ls`, `cat`, …). Do the commit locally in the repo, or ask **Cursor Agent** \
with an explicit path (e.g. `CURSOR_AGENT: in ~/projects/mac-stats commit and push`)."
                .to_string(),
        );
    }
    None
}

fn is_git_commit_push_request(n: &str) -> bool {
    let has_commit = n.contains("commit");
    let has_push = n.contains("push");
    if !(has_commit || (has_push && n.contains("git"))) {
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
                assert!(reply.to_lowercase().contains("allowlist") || reply.contains("RUN_CMD"));
            }
            other => panic!("expected Instant, got {:?}", other),
        }
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
