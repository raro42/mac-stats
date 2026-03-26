//! Short-term session memory for chat: keep messages in memory and persist to disk when > 3.
//!
//! When a session has more than 3 messages, the conversation is written to
//! `~/.mac-stats/session/session-memory-<sessionid>-<timestamp>-<topic>.md`.
//! Older builds used `session-memory-<topic>-<sessionid>-<timestamp>.md`; loading accepts both.
//!
//! Callers can use `get_messages()` for in-memory history and
//! `load_messages_from_latest_session_file()` to resume from disk (e.g. after restart).
//!
//! **Conversation vs internal artifacts (task-008 Phase 2):** Only normal conversation is
//! persisted — user turns and final assistant replies. Internal execution artifacts
//! (completion-verifier prompts, criteria extraction, tool dumps, correction prompts) are
//! not written and are filtered out when loading so they never appear in prior context.

use crate::config::Config;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::OnceLock;
use tracing::{debug, info, warn};

const PERSIST_THRESHOLD: usize = 3;

/// Current layout: `session-memory-{session_id}-{YYYYMMDD-HHMMSS}-{topic_slug}.md`
static SESSION_FILE_NEW: OnceLock<Regex> = OnceLock::new();
/// Legacy layout (pre topic-last reorder): `session-memory-{topic_slug}-{session_id}-{YYYYMMDD-HHMMSS}.md`
static SESSION_FILE_OLD: OnceLock<Regex> = OnceLock::new();

fn session_file_new_re() -> &'static Regex {
    SESSION_FILE_NEW.get_or_init(|| {
        Regex::new(r"^session-memory-(\d+)-(\d{8}-\d{6})-(.+)\.md$").expect("valid regex")
    })
}

fn session_file_old_re() -> &'static Regex {
    SESSION_FILE_OLD.get_or_init(|| {
        Regex::new(r"^session-memory-(.+)-(\d+)-(\d{8}-\d{6})\.md$").expect("valid regex")
    })
}

/// True if this session file name belongs to `session_id` (new or legacy filename pattern).
fn session_filename_matches_id(name: &str, session_id: u64) -> bool {
    if !name.starts_with("session-memory-") || !name.ends_with(".md") {
        return false;
    }
    if let Some(caps) = session_file_new_re().captures(name) {
        if caps[1].parse::<u64>().ok() == Some(session_id) {
            return true;
        }
    }
    if let Some(caps) = session_file_old_re().captures(name) {
        if caps[2].parse::<u64>().ok() == Some(session_id) {
            return true;
        }
    }
    false
}

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

/// Best-effort export immediately **before** a user-triggered session clear (Discord: reset phrases or `new session:` prefix).
/// When `beforeResetTranscriptPath` or `MAC_STATS_BEFORE_RESET_TRANSCRIPT_PATH` is set, writes JSONL (meta line + one object per message).
/// When only **`beforeResetHook`** / **`MAC_STATS_BEFORE_RESET_HOOK`** is set, writes to `~/.mac-stats/agents/last_session_before_reset.jsonl` by default.
/// Optional hook runs in a background thread via `/bin/sh -c '<hook> \"$1\"' _ <path>`; does not block the reset. Failures are logged only.
/// See `docs/data_files_reference.md` (before-reset export).
pub fn before_session_reset_export(source: &str, session_id: u64, reason: &str) {
    let hook_raw = Config::before_reset_hook_raw();
    let hook_configured = !hook_raw.trim().is_empty();
    let path_configured = !Config::before_reset_transcript_path_raw().trim().is_empty();

    if !hook_configured && !path_configured {
        return;
    }

    let transcript_path: PathBuf = if path_configured {
        match Config::before_reset_transcript_path_resolved() {
            Some(p) => p,
            None => {
                warn!(
                    "Session memory: before_reset transcript path set but could not resolve (~ requires HOME); skipping export"
                );
                return;
            }
        }
    } else {
        Config::default_before_reset_transcript_path()
    };

    let mut messages = get_messages(source, session_id);
    if messages.is_empty() {
        messages = load_messages_from_latest_session_file(source, session_id);
    }

    let ts = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let meta = serde_json::json!({
        "kind": "before_reset_meta",
        "source": source,
        "session_id": session_id,
        "reason": reason,
        "exported_at_utc": ts,
        "message_count": messages.len(),
    });
    let mut body = String::new();
    body.push_str(&meta.to_string());
    body.push('\n');
    for (role, content) in &messages {
        let line = serde_json::json!({ "role": role, "content": content });
        body.push_str(&line.to_string());
        body.push('\n');
    }

    if let Some(parent) = transcript_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            warn!(
                "Session memory: before_reset could not create parent dir {}: {}",
                parent.display(),
                e
            );
        }
    }

    match std::fs::write(&transcript_path, &body) {
        Ok(()) => info!(
            "Session memory: before_reset wrote {} ({} messages, reason={})",
            transcript_path.display(),
            messages.len(),
            reason
        ),
        Err(e) => {
            warn!(
                "Session memory: before_reset write failed {}: {}",
                transcript_path.display(),
                e
            );
            return;
        }
    }

    if !hook_configured {
        return;
    }

    let hook_cmd = hook_raw.trim().to_string();
    let path_for_thread = transcript_path.clone();
    let reason_owned = reason.to_string();
    let source_owned = source.to_string();
    std::thread::spawn(move || {
        let path_str = path_for_thread.to_string_lossy().into_owned();
        let script = format!("{} \"$1\"", hook_cmd);
        let status = std::process::Command::new("/bin/sh")
            .arg("-c")
            .arg(&script)
            .arg("_")
            .arg(&path_str)
            .env("MAC_STATS_BEFORE_RESET_TRANSCRIPT", &path_str)
            .env("MAC_STATS_BEFORE_RESET_REASON", &reason_owned)
            .env("MAC_STATS_BEFORE_RESET_SOURCE", &source_owned)
            .env(
                "MAC_STATS_BEFORE_RESET_SESSION_ID",
                format!("{}", session_id),
            )
            .status();
        match status {
            Ok(s) if s.success() => {
                debug!("Session memory: before_reset hook finished successfully");
            }
            Ok(s) => {
                warn!(
                    "Session memory: before_reset hook exited with status {:?}",
                    s.code()
                );
            }
            Err(e) => {
                warn!("Session memory: before_reset hook spawn failed: {}", e);
            }
        }
    });
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
    let mut snapshot: Option<Vec<(String, String)>> = None;
    {
        let store = match session_store().lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        if let Some(state) = store.get(&key) {
            if !state.messages.is_empty() {
                snapshot = Some(state.messages.clone());
            }
        }
        let has_messages = snapshot.is_some();
        if has_messages {
            drop(store);
            let _ = persist_session(source, session_id);
        }
    }
    if let Some(msgs) = snapshot {
        let conv = count_conversational_messages(&msgs);
        crate::commands::ori_lifecycle::maybe_capture_before_session_reset_fire_and_forget(
            source, session_id, msgs, conv,
        );
    }
    crate::commands::ori_lifecycle::on_session_cleared(source, session_id);
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
    let mut entries: Vec<_> = match std::fs::read_dir(&dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| session_filename_matches_id(n, session_id))
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

/// Finish the current `## User` / `## Assistant` block and append to `out`.
/// If no heading was open, drop any leading lines (same as ignoring a malformed prefix).
fn flush_session_block(
    out: &mut Vec<(String, String)>,
    current_role: &mut Option<&'static str>,
    body_lines: &mut Vec<String>,
) {
    let Some(r) = current_role.take() else {
        body_lines.clear();
        return;
    };
    let body_str = body_lines.join("\n").trim().to_string();
    body_lines.clear();
    if let Some(normalized) = normalize_conversational_message(r, &body_str) {
        if !is_internal_artifact(r, &normalized) {
            out.push((r.to_string(), normalized));
        }
    }
}

/// Parse persisted session markdown. Only lines that trim to exactly `## User` or `## Assistant`
/// start a new block; lines like `## Notes` inside a message stay in the body (splitting on
/// `\n## ` previously dropped those turns — see docs/022_feature_review_plan.md F1).
fn parse_session_markdown(content: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut current_role: Option<&'static str> = None;
    let mut body_lines: Vec<String> = Vec::new();

    for line in content.lines() {
        let t = line.trim();
        if t == "## User" {
            flush_session_block(&mut out, &mut current_role, &mut body_lines);
            current_role = Some("user");
        } else if t == "## Assistant" {
            flush_session_block(&mut out, &mut current_role, &mut body_lines);
            current_role = Some("assistant");
        } else {
            body_lines.push(line.to_string());
        }
    }
    flush_session_block(&mut out, &mut current_role, &mut body_lines);
    out
}

fn parse_session_file(path: &Path) -> Vec<(String, String)> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    parse_session_markdown(&content)
}

#[cfg(test)]
mod tests {
    use super::{
        add_message, clear_session, extract_assistant_final_answer, get_messages,
        load_messages_from_latest_session_file, normalize_conversational_message,
        parse_session_markdown, session_filename_matches_id,
    };
    use std::sync::Mutex;
    use std::time::Duration;

    /// Tests override `MAC_STATS_SESSION_DIR`; serialize so env does not race.
    static SESSION_DIR_TEST_LOCK: Mutex<()> = Mutex::new(());

    struct SessionDirOverride {
        previous: Option<String>,
    }

    impl SessionDirOverride {
        fn set(path: &std::path::Path) -> Self {
            let previous = std::env::var("MAC_STATS_SESSION_DIR").ok();
            std::env::set_var("MAC_STATS_SESSION_DIR", path.as_os_str());
            Self { previous }
        }
    }

    impl Drop for SessionDirOverride {
        fn drop(&mut self) {
            match &self.previous {
                Some(v) => std::env::set_var("MAC_STATS_SESSION_DIR", v),
                None => std::env::remove_var("MAC_STATS_SESSION_DIR"),
            }
        }
    }

    #[test]
    fn get_messages_before_add_user_excludes_current_turn() {
        // Mirrors Discord: load `prior` via get_messages, then add_message("user", …).
        let sid = 99998_u64;
        clear_session("discord", sid);
        add_message("discord", sid, "user", "first");
        add_message("discord", sid, "assistant", "reply");
        let prior = get_messages("discord", sid);
        assert_eq!(prior.len(), 2);
        assert_eq!(prior[1].1, "reply");
        add_message("discord", sid, "user", "current question");
        let after = get_messages("discord", sid);
        assert_eq!(after.len(), 3);
        assert!(
            !prior.iter().any(|(_, c)| c == "current question"),
            "prior snapshot must not include the user turn added after get_messages"
        );
        clear_session("discord", sid);
    }

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

    #[test]
    fn session_filename_matches_new_layout() {
        assert!(session_filename_matches_id(
            "session-memory-42-20260320-153045-my-topic.md",
            42
        ));
        assert!(!session_filename_matches_id(
            "session-memory-42-20260320-153045-my-topic.md",
            99
        ));
    }

    #[test]
    fn session_filename_matches_legacy_layout() {
        assert!(session_filename_matches_id(
            "session-memory-my-topic-42-20260320-153045.md",
            42
        ));
        assert!(!session_filename_matches_id(
            "session-memory-my-topic-42-20260320-153045.md",
            7
        ));
    }

    #[test]
    fn session_filename_rejects_non_session_files() {
        assert!(!session_filename_matches_id(
            "other-42-20260320-153045.md",
            42
        ));
        assert!(!session_filename_matches_id(
            "session-memory-42-20260320-153045.md",
            42
        ));
    }

    #[test]
    fn parse_session_markdown_well_formed_two_turns() {
        let md = "## User\n\nHello\n\n## Assistant\n\nHi there\n";
        let v = parse_session_markdown(md);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].0, "user");
        assert_eq!(v[0].1, "Hello");
        assert_eq!(v[1].0, "assistant");
        assert_eq!(v[1].1, "Hi there");
    }

    /// Lines that look like markdown headings but are not exactly `## User` / `## Assistant` must
    /// stay inside the message (022 F1: `## ` in content).
    #[test]
    fn parse_session_markdown_keeps_fake_headings_in_body() {
        let md = "## User\n\nSee ## Notes\nbelow\n\n## Assistant\n\nOK\n";
        let v = parse_session_markdown(md);
        assert_eq!(v.len(), 2);
        assert!(v[0].1.contains("## Notes"));
        assert!(v[0].1.contains("below"));
        assert_eq!(v[1].1, "OK");
    }

    #[test]
    fn parse_session_markdown_empty_user_block_skipped() {
        let md = "## User\n\n## Assistant\n\nOnly reply\n";
        let v = parse_session_markdown(md);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].0, "assistant");
        assert_eq!(v[0].1, "Only reply");
    }

    #[test]
    fn parse_session_markdown_leading_garbage_before_first_heading_dropped() {
        let md = "orphan line\n## User\n\nHi\n";
        let v = parse_session_markdown(md);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].1, "Hi");
    }

    /// Disk resume for session id 77 with current filename layout (022 §3 F1).
    #[test]
    fn load_messages_from_latest_session_file_new_layout() {
        let _guard = SESSION_DIR_TEST_LOCK.lock().expect("session dir test lock");
        let base = std::env::temp_dir().join(format!(
            "mac-stats-session-load-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).expect("mkdir session test dir");
        let _override = SessionDirOverride::set(&base);

        let sid = 77_u64;
        let path = base.join(format!(
            "session-memory-{sid}-20260322-120000-resume-topic.md"
        ));
        std::fs::write(&path, "## User\n\nfrom disk\n\n## Assistant\n\nack\n")
            .expect("write session file");

        let loaded = load_messages_from_latest_session_file("test", sid);
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].0, "user");
        assert_eq!(loaded[0].1, "from disk");
        assert_eq!(loaded[1].0, "assistant");
        assert_eq!(loaded[1].1, "ack");

        let _ = std::fs::remove_dir_all(&base);
    }

    /// Legacy `session-memory-{topic}-{id}-{ts}.md` still loads (022 §3 F1 / D1).
    #[test]
    fn load_messages_from_latest_session_file_legacy_layout() {
        let _guard = SESSION_DIR_TEST_LOCK.lock().expect("session dir test lock");
        let base = std::env::temp_dir().join(format!(
            "mac-stats-session-legacy-load-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).expect("mkdir session test dir");
        let _override = SessionDirOverride::set(&base);

        let sid = 88_u64;
        let path = base.join("session-memory-old-topic-88-20260321-090000.md");
        std::fs::write(
            &path,
            "## User\n\nlegacy file\n\n## Assistant\n\nlegacy reply\n",
        )
        .expect("write legacy session file");

        let loaded = load_messages_from_latest_session_file("test", sid);
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].1, "legacy file");
        assert_eq!(loaded[1].1, "legacy reply");

        let _ = std::fs::remove_dir_all(&base);
    }

    /// When two files match the session id, the one with newer `modified()` wins.
    #[test]
    fn load_messages_from_latest_session_file_prefers_newer_mtime() {
        let _guard = SESSION_DIR_TEST_LOCK.lock().expect("session dir test lock");
        let base = std::env::temp_dir().join(format!(
            "mac-stats-session-mtime-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).expect("mkdir session test dir");
        let _override = SessionDirOverride::set(&base);

        let sid = 55_u64;
        let older = base.join(format!("session-memory-{sid}-20260101-000000-first.md"));
        std::fs::write(&older, "## User\n\nolder\n\n## Assistant\n\na\n").expect("older");
        std::thread::sleep(Duration::from_millis(1100));
        let newer = base.join(format!("session-memory-{sid}-20260202-000000-second.md"));
        std::fs::write(&newer, "## User\n\nnewer\n\n## Assistant\n\nb\n").expect("newer");

        let loaded = load_messages_from_latest_session_file("test", sid);
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].1, "newer");
        assert_eq!(loaded[1].1, "b");

        let _ = std::fs::remove_dir_all(&base);
    }
}
