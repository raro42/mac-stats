//! Short-term session memory for chat: keep messages in memory and persist to disk when > 3.
//!
//! When a session has more than 3 messages, the conversation is written to
//! `~/.mac-stats/session/session-memory-<sessionid>-<timestamp>-<topic>.md`.
//!
//! Callers can use `get_messages()` for in-memory history and
//! `load_messages_from_latest_session_file()` to resume from disk (e.g. after restart).
//!
//! **Conversation vs internal artifacts (task-008 Phase 2):** Only normal conversation is
//! persisted — user turns and final assistant replies. Internal execution artifacts
//! (completion-verifier prompts, criteria extraction, tool dumps, correction prompts) are
//! not written and are filtered out when loading so they never appear in prior context.

use crate::config::Config;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::sync::OnceLock;
use tracing::debug;

const PERSIST_THRESHOLD: usize = 3;

/// Returns true if this (role, content) looks like an internal execution artifact rather than
/// normal conversation. Such messages are not persisted and are filtered out when loading.
fn is_internal_artifact(role: &str, content: &str) -> bool {
    let t = content.trim();
    if t.is_empty() {
        return true;
    }
    // Completion-verifier / criteria-extraction prompts (we inject these; never store as conversation).
    if t.contains("extracts success criteria")
        || t.contains("Success criteria (from user request)")
        || t.contains("Reply with YES or NO")
        || t.contains("Does the following response satisfy")
        || t.contains("Success criteria require a response in JSON format")
    {
        return true;
    }
    // Escalation / correction prompt we inject (internal, not user-facing).
    if t.contains("The user is not satisfied and wants the task actually completed") {
        return true;
    }
    // Tool result wrappers: message that is predominantly a raw tool dump, not the model's answer.
    if role == "assistant" {
        let first_line = t.lines().next().unwrap_or("").trim();
        if first_line.starts_with("REDMINE_API result (")
            || first_line.starts_with("FETCH_URL result")
            || first_line.starts_with("PERPLEXITY_SEARCH result")
            || first_line.starts_with("BRAVE_SEARCH result")
            || first_line.starts_with("Tool result:")
        {
            return true;
        }
    }
    false
}

/// Count messages that are normal conversation (not internal artifacts).
/// Used by session compactor to skip compaction when there is no real conversational value (task-008 Phase 5).
pub fn count_conversational_messages(messages: &[(String, String)]) -> usize {
    messages
        .iter()
        .filter(|(role, content)| !is_internal_artifact(role, content))
        .count()
}

fn extract_assistant_final_answer(content: &str) -> Option<String> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return None;
    }
    if !trimmed.starts_with("--- Intermediate answer:") {
        return Some(trimmed.to_string());
    }

    if let Some(idx) = trimmed.find("--- Final answer:") {
        let final_part = trimmed[idx + "--- Final answer:".len()..]
            .trim()
            .trim_matches('-')
            .trim();
        if !final_part.is_empty() {
            return Some(final_part.to_string());
        }
    }

    if trimmed.contains("Final answer is the same as intermediate.") {
        let without_prefix = trimmed["--- Intermediate answer:".len()..].trim();
        let intermediate = without_prefix
            .split("\n\n---\n\n")
            .next()
            .unwrap_or(without_prefix)
            .trim();
        if !intermediate.is_empty() {
            return Some(intermediate.to_string());
        }
    }

    Some(trimmed.to_string())
}

fn normalize_conversational_message(role: &str, content: &str) -> Option<String> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return None;
    }
    match role {
        "assistant" => extract_assistant_final_answer(trimmed),
        _ => Some(trimmed.to_string()),
    }
}

struct SessionState {
    messages: Vec<(String, String)>,
    topic_slug: Option<String>,
    created_at: Option<chrono::DateTime<chrono::Local>>,
    /// Last time a message was added; used for active vs inactive (e.g. 30-min compaction).
    last_activity: Option<chrono::DateTime<chrono::Local>>,
}

fn session_store() -> &'static Mutex<HashMap<String, SessionState>> {
    static STORE: OnceLock<Mutex<HashMap<String, SessionState>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Make a filename-safe slug from the first user message (topic).
fn topic_slug(content: &str, max_len: usize) -> String {
    let s: String = content
        .chars()
        .take(max_len)
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let s = s.trim().replace(' ', "-").trim_matches('-').to_lowercase();
    if s.is_empty() {
        "chat".to_string()
    } else {
        s
    }
}

/// Built-in fallback phrases when ~/.mac-stats/agents/session_reset_phrases.md is missing or empty.
const SESSION_RESET_PHRASES_FALLBACK: &[&str] = &[
    "new session",
    "clear session",
    "reset",
    "new topic",
    "start over",
    "fresh start",
    "neue sitzung",
    "sitzung löschen",
    "zurücksetzen",
    "nueva sesión",
    "limpiar sesión",
    "reiniciar",
    "nouvelle session",
    "effacer la session",
    "recommencer",
];

/// True if the user message asks to clear/reset the session (any language). Use before loading history.
/// Phrases are loaded from ~/.mac-stats/agents/session_reset_phrases.md (one per line; user-editable).
/// If the file is missing or yields no phrases, a built-in list is used. Matching is case-insensitive substring.
pub fn user_wants_session_reset(message: &str) -> bool {
    let normalized = message.trim().to_lowercase();
    if normalized.is_empty() {
        return false;
    }
    let phrases = Config::load_session_reset_phrases();
    let mut iter: Box<dyn Iterator<Item = &str>> = if phrases.is_empty() {
        Box::new(SESSION_RESET_PHRASES_FALLBACK.iter().copied())
    } else {
        Box::new(phrases.iter().map(String::as_str))
    };
    iter.any(|phrase| normalized.contains(&phrase.to_lowercase()))
}

/// Returns the Session Startup instruction plus current date/time (UTC) to inject after a session reset.
/// Used so the agent knows to run Session Startup and which daily memory files (if any) to read.
pub fn session_reset_instruction_with_date_utc() -> String {
    let now = chrono::Utc::now();
    let date_time = now.format("%Y-%m-%d %H:%M UTC");
    format!(
        "A new session was started. Current date/time: {}. Execute your Session Startup: read soul, user-info, and daily memory for today and yesterday; in main session also read MEMORY. Then greet the user briefly.",
        date_time
    )
}

/// Add a message to the session and persist to disk when we have more than 3 messages.
/// `source` e.g. "discord", `session_id` e.g. Discord channel id.
/// Internal artifacts (verifier prompts, criteria, tool dumps) are not persisted.
pub fn add_message(source: &str, session_id: u64, role: &str, content: &str) {
    let Some(content) = normalize_conversational_message(role, content) else {
        return;
    };
    if is_internal_artifact(role, &content) {
        debug!("Session memory: skipping internal artifact (not persisted)");
        return;
    }
    let key = format!("{}-{}", source, session_id);
    let mut store = match session_store().lock() {
        Ok(g) => g,
        Err(_) => return,
    };

    let now = chrono::Local::now();
    let state = store.entry(key.clone()).or_insert_with(|| SessionState {
        messages: Vec::new(),
        topic_slug: None,
        created_at: None,
        last_activity: None,
    });

    if state.created_at.is_none() {
        state.created_at = Some(now);
    }
    state.last_activity = Some(now);
    if role == "user" && state.topic_slug.is_none() {
        state.topic_slug = Some(topic_slug(&content, 40));
    }

    state.messages.push((role.to_string(), content));

    if state.messages.len() > PERSIST_THRESHOLD {
        drop(store);
        if let Err(e) = persist_session(source, session_id) {
            debug!("Session memory: persist failed: {}", e);
        }
    }
}

fn persist_session(source: &str, session_id: u64) -> std::io::Result<()> {
    let key = format!("{}-{}", source, session_id);
    let (messages, topic_slug, created_at) = {
        let store = session_store()
            .lock()
            .map_err(|_| std::io::Error::other("session store lock failed"))?;
        let state = store.get(&key).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "session not found")
        })?;
        let topic = state
            .topic_slug
            .clone()
            .unwrap_or_else(|| "chat".to_string());
        let ts = state.created_at.unwrap_or_else(chrono::Local::now);
        (state.messages.clone(), topic, ts)
    };

    Config::ensure_session_directory()?;
    let dir = Config::session_dir();
    let timestamp = created_at.format("%Y%m%d-%H%M%S");
    let filename = format!(
        "session-memory-{}-{}-{}.md",
        session_id, timestamp, topic_slug
    );
    let path = dir.join(filename);

    let mut body = String::new();
    for (role, content) in &messages {
        let heading = if role == "user" { "User" } else { "Assistant" };
        body.push_str(&format!("## {}\n\n{}\n\n", heading, content));
    }

    std::fs::write(&path, body)?;
    debug!(
        "Session memory: wrote {} ({} messages)",
        path.display(),
        messages.len()
    );
    Ok(())
}

/// Clear in-memory session history. Persists current messages to disk first (if any).
pub fn clear_session(source: &str, session_id: u64) {
    let key = format!("{}-{}", source, session_id);
    {
        let store = match session_store().lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        let has_messages = store.get(&key).is_some_and(|s| !s.messages.is_empty());
        if has_messages {
            drop(store);
            let _ = persist_session(source, session_id);
        }
    }
    if let Ok(mut store) = session_store().lock() {
        store.remove(&key);
    }
    debug!("Session memory: cleared session {}", key);
}

/// Replace the in-memory session with compacted messages. Persists the old session first.
/// Used after session compaction to replace verbose history with a concise summary.
pub fn replace_session(source: &str, session_id: u64, new_messages: Vec<(String, String)>) {
    let key = format!("{}-{}", source, session_id);
    {
        let store = match session_store().lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        let has_messages = store.get(&key).is_some_and(|s| !s.messages.is_empty());
        if has_messages {
            drop(store);
            let _ = persist_session(source, session_id);
        }
    }
    let normalized_messages: Vec<(String, String)> = new_messages
        .into_iter()
        .filter_map(|(role, content)| {
            let normalized = normalize_conversational_message(&role, &content)?;
            if is_internal_artifact(&role, &normalized) {
                return None;
            }
            Some((role, normalized))
        })
        .collect();
    let now = chrono::Local::now();
    if let Ok(mut store) = session_store().lock() {
        let state = store.entry(key).or_insert_with(|| SessionState {
            messages: Vec::new(),
            topic_slug: None,
            created_at: None,
            last_activity: None,
        });
        if state.created_at.is_none() {
            state.created_at = Some(now);
        }
        state.last_activity = Some(now);
        state.messages = normalized_messages;
        debug!(
            "Session memory: replaced session with {} compacted messages",
            state.messages.len()
        );
    }
}

/// One session entry for listing (source, session_id, message_count, last_activity).
pub struct SessionEntry {
    pub source: String,
    pub session_id: u64,
    pub message_count: usize,
    pub last_activity: chrono::DateTime<chrono::Local>,
}

/// List all in-memory sessions. Used by the 30-min compaction loop.
pub fn list_sessions() -> Vec<SessionEntry> {
    let store = match session_store().lock() {
        Ok(g) => g,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    for (key, state) in store.iter() {
        let last_activity = match state.last_activity.or(state.created_at) {
            Some(t) => t,
            None => continue,
        };
        let (source, session_id) = match key.split_once('-') {
            Some((s, id)) => match id.parse::<u64>() {
                Ok(id) => (s.to_string(), id),
                Err(_) => continue,
            },
            None => continue,
        };
        out.push(SessionEntry {
            source,
            session_id,
            message_count: state.messages.len(),
            last_activity,
        });
    }
    out
}

/// Return the current in-memory messages for this session (role, content).
/// Call this *before* adding the current user message so the result is prior turns only.
pub fn get_messages(source: &str, session_id: u64) -> Vec<(String, String)> {
    let key = format!("{}-{}", source, session_id);
    let store = match session_store().lock() {
        Ok(g) => g,
        Err(_) => return Vec::new(),
    };
    store
        .get(&key)
        .map(|state| {
            state
                .messages
                .iter()
                .filter_map(|(role, content)| {
                    let normalized = normalize_conversational_message(role, content)?;
                    if is_internal_artifact(role, &normalized) {
                        return None;
                    }
                    Some((role.clone(), normalized))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Load messages from the most recent session file for this session (e.g. after app restart).
/// File format: `## User\n\n...\n\n## Assistant\n\n...`. Returns (role, content) with role "user" or "assistant".
pub fn load_messages_from_latest_session_file(
    _source: &str,
    session_id: u64,
) -> Vec<(String, String)> {
    let dir = Config::session_dir();
    if !dir.is_dir() {
        return Vec::new();
    }
    let prefix = format!("session-memory-{}-", session_id);
    let mut entries: Vec<_> = match std::fs::read_dir(&dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with(&prefix))
            })
            .collect(),
        Err(_) => return Vec::new(),
    };
    entries.sort_by(|a, b| {
        b.path()
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .cmp(&a.path().metadata().and_then(|m| m.modified()).ok())
    });
    let path = match entries.into_iter().next().map(|e| e.path()) {
        Some(p) => p,
        None => return Vec::new(),
    };
    parse_session_file(&path)
}

fn parse_session_file(path: &Path) -> Vec<(String, String)> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    for block in content.split("\n## ") {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }
        let (role, body) = if block.starts_with("User\n") {
            ("user", block["User\n".len()..].trim())
        } else if block.starts_with("Assistant\n") {
            ("assistant", block["Assistant\n".len()..].trim())
        } else {
            continue;
        };
        if let Some(normalized) = normalize_conversational_message(role, body) {
            if !is_internal_artifact(role, &normalized) {
                out.push((role.to_string(), normalized));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{
        add_message, clear_session, extract_assistant_final_answer, get_messages,
        normalize_conversational_message,
    };

    #[test]
    fn internal_artifacts_not_persisted() {
        // Use a unique session id so we don't collide with real sessions.
        let sid = 99999_u64;
        clear_session("discord", sid);
        add_message("discord", sid, "user", "Hello");
        add_message(
            "discord",
            sid,
            "assistant",
            "You are an assistant that extracts success criteria. Reply with 1 to 3 concrete success criteria.",
        );
        let msgs = get_messages("discord", sid);
        // Only the user message should be present; the internal criteria prompt must not be stored.
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].0, "user");
        assert_eq!(msgs[0].1, "Hello");
        clear_session("discord", sid);
    }

    #[test]
    fn extract_assistant_final_answer_prefers_final_section() {
        let wrapped = "--- Intermediate answer:\n\nFirst try.\n\n---\n\n--- Final answer:\n\nSecond try.\n\n---";
        assert_eq!(
            extract_assistant_final_answer(wrapped),
            Some("Second try.".to_string())
        );
    }

    #[test]
    fn extract_assistant_final_answer_keeps_intermediate_when_marked_same() {
        let wrapped = "--- Intermediate answer:\n\nReusable answer.\n\n---\n\nFinal answer is the same as intermediate.";
        assert_eq!(
            extract_assistant_final_answer(wrapped),
            Some("Reusable answer.".to_string())
        );
    }

    #[test]
    fn normalize_conversational_message_discards_empty_content() {
        assert_eq!(normalize_conversational_message("assistant", "   "), None);
    }
}
