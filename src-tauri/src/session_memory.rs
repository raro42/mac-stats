//! Short-term session memory for chat: keep messages in memory and persist to disk when > 3.
//!
//! When a session has more than 3 messages, the conversation is written to
//! `~/.mac-stats/session/session-memory-<topic>-<sessionid>-<timestamp>.md`.

use crate::config::Config;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;
use tracing::debug;

const PERSIST_THRESHOLD: usize = 3;

struct SessionState {
    messages: Vec<(String, String)>,
    topic_slug: Option<String>,
    created_at: Option<chrono::DateTime<chrono::Local>>,
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
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { '_' })
        .collect();
    let s = s.trim().replace(' ', "-").trim_matches('-').to_lowercase();
    if s.is_empty() {
        "chat".to_string()
    } else {
        s
    }
}

/// Add a message to the session and persist to disk when we have more than 3 messages.
/// `source` e.g. "discord", `session_id` e.g. Discord channel id.
pub fn add_message(source: &str, session_id: u64, role: &str, content: &str) {
    let key = format!("{}-{}", source, session_id);
    let mut store = match session_store().lock() {
        Ok(g) => g,
        Err(_) => return,
    };

    let state = store.entry(key.clone()).or_insert_with(|| SessionState {
        messages: Vec::new(),
        topic_slug: None,
        created_at: None,
    });

    if state.created_at.is_none() {
        state.created_at = Some(chrono::Local::now());
    }
    if role == "user" && state.topic_slug.is_none() {
        state.topic_slug = Some(topic_slug(content, 40));
    }

    state.messages.push((role.to_string(), content.to_string()));

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
        let store = session_store().lock().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::Other, "session store lock failed")
        })?;
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
        topic_slug,
        session_id,
        timestamp
    );
    let path = dir.join(filename);

    let mut body = String::new();
    for (role, content) in &messages {
        let heading = if role == "user" { "User" } else { "Assistant" };
        body.push_str(&format!("## {}\n\n{}\n\n", heading, content));
    }

    std::fs::write(&path, body)?;
    debug!("Session memory: wrote {} ({} messages)", path.display(), messages.len());
    Ok(())
}
