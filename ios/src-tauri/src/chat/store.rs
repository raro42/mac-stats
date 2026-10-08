//! Conversations saved in the app container: one JSON file per conversation plus an
//! index, always written to a temporary file and renamed (atomic write).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri_plugin_llm::DetectedLanguage;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReplyStats {
    pub model_id: String,
    pub n_gen: u32,
    pub tg_tps: f64,
    pub stop_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StoredMessage {
    /// "user" or "assistant".
    pub role: String,
    pub content: String,
    pub ts: i64,
    /// System note given to the model with this question: iPhone data and reply
    /// language. Saved so later turns replay the exact same prompt.
    #[serde(default, alias = "deviceNote", skip_serializing_if = "Option::is_none")]
    pub turn_note: Option<String>,
    /// Language of this user message, only when the on-device detection was clear.
    /// The last one is the conversation's current reply language.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<DetectedLanguage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stats: Option<ReplyStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub messages: Vec<StoredMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConversationSummary {
    pub id: String,
    pub title: String,
    pub updated_at: i64,
    pub message_count: usize,
}

impl Conversation {
    pub fn new(id: String, now: i64) -> Self {
        Conversation { id, title: String::new(), created_at: now, updated_at: now, messages: Vec::new() }
    }

    pub fn summary(&self) -> ConversationSummary {
        ConversationSummary {
            id: self.id.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            message_count: self.messages.len(),
        }
    }
}

/// Title from the first question (one line, max. 48 characters). Empty when there is
/// no text: the web layer then shows a translated "New conversation".
pub fn title_from(question: &str) -> String {
    let line = question.lines().next().unwrap_or("").trim();
    let mut title: String = line.chars().take(48).collect();
    if line.chars().count() > 48 {
        title.push('…');
    }
    title
}

/// IDs come from the web UI: only hexadecimal ones are accepted, so they can never
/// point outside the conversations folder.
pub fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 32 && id.chars().all(|c| c.is_ascii_hexdigit())
}

fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path)
}

pub struct ChatStore {
    dir: PathBuf,
}

impl ChatStore {
    pub fn new(dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(&dir);
        ChatStore { dir }
    }

    fn path(&self, id: &str) -> Option<PathBuf> {
        valid_id(id).then(|| self.dir.join(format!("{id}.json")))
    }

    fn index_path(&self) -> PathBuf {
        self.dir.join("index.json")
    }

    /// Conversations from newest to oldest.
    pub fn list(&self) -> Vec<ConversationSummary> {
        let mut list: Vec<ConversationSummary> = fs::read(self.index_path())
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        list.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        list
    }

    pub fn get(&self, id: &str) -> Option<Conversation> {
        let bytes = fs::read(self.path(id)?).ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    pub fn save(&self, conversation: &Conversation) -> io::Result<()> {
        let path = self
            .path(&conversation.id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid conversation id"))?;
        write_atomic(&path, &serde_json::to_vec_pretty(conversation)?)?;
        let mut index = self.list();
        index.retain(|c| c.id != conversation.id);
        index.push(conversation.summary());
        write_atomic(&self.index_path(), &serde_json::to_vec_pretty(&index)?)
    }

    pub fn delete(&self, id: &str) -> io::Result<()> {
        if let Some(path) = self.path(id) {
            let _ = fs::remove_file(path);
        }
        let mut index = self.list();
        index.retain(|c| c.id != id);
        write_atomic(&self.index_path(), &serde_json::to_vec_pretty(&index)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store(name: &str) -> ChatStore {
        let dir = std::env::temp_dir().join(format!("ios-stats-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        ChatStore::new(dir)
    }

    fn message(role: &str, content: &str) -> StoredMessage {
        StoredMessage { role: role.into(), content: content.into(), ts: 1, turn_note: None, language: None, stats: None }
    }

    #[test]
    fn save_get_list_and_delete() {
        let store = temp_store("crud");
        let mut a = Conversation::new("a1".into(), 10);
        a.messages.push(message("user", "hola"));
        store.save(&a).unwrap();
        let mut b = Conversation::new("b2".into(), 20);
        b.updated_at = 30;
        store.save(&b).unwrap();

        assert_eq!(store.get("a1").unwrap(), a);
        let ids: Vec<_> = store.list().into_iter().map(|c| c.id).collect();
        assert_eq!(ids, ["b2", "a1"]);

        store.delete("a1").unwrap();
        assert!(store.get("a1").is_none());
        assert_eq!(store.list().len(), 1);
    }

    #[test]
    fn saving_again_replaces_the_index_entry() {
        let store = temp_store("resave");
        let mut c = Conversation::new("c3".into(), 1);
        store.save(&c).unwrap();
        c.messages.push(message("user", "otra"));
        c.updated_at = 2;
        store.save(&c).unwrap();
        let list = store.list();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].message_count, 1);
    }

    #[test]
    fn ids_cannot_escape_the_folder() {
        let store = temp_store("ids");
        assert!(!valid_id("../index"));
        assert!(!valid_id(""));
        assert!(store.get("../../etc/passwd").is_none());
        let bad = Conversation::new("../x".into(), 1);
        assert!(store.save(&bad).is_err());
    }

    #[test]
    fn old_files_with_device_note_still_load() {
        let old = r#"{"role":"user","content":"hola","ts":1,"deviceNote":"Datos"}"#;
        let m: StoredMessage = serde_json::from_str(old).unwrap();
        assert_eq!(m.turn_note.as_deref(), Some("Datos"));
        assert_eq!(m.language, None);
    }

    #[test]
    fn titles_are_one_short_line() {
        assert_eq!(title_from("How is my battery?\nmore text"), "How is my battery?");
        assert_eq!(title_from("   "), "");
        assert_eq!(Conversation::new("a1".into(), 0).title, "");
        let long = "a".repeat(60);
        assert_eq!(title_from(&long).chars().count(), 49);
    }
}
