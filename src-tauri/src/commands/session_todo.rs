//! Hermes-style in-session TODO list (survives compaction reinjection).

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};

fn store() -> &'static Mutex<HashMap<String, Vec<TodoItem>>> {
    static STORE: OnceLock<Mutex<HashMap<String, Vec<TodoItem>>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: String,
    pub content: String,
    pub status: String,
}

fn session_key(discord_channel_id: Option<u64>) -> String {
    match discord_channel_id {
        Some(id) => format!("discord-{}", id),
        None => "main".to_string(),
    }
}

fn validate_status(s: &str) -> bool {
    matches!(
        s,
        "pending" | "in_progress" | "completed" | "cancelled"
    )
}

/// Format list for prompt reinjection / tool responses.
pub fn format_todos(items: &[TodoItem]) -> String {
    if items.is_empty() {
        return "(empty TODO list)".to_string();
    }
    let mut lines = vec!["**Session TODO:**".to_string()];
    for (i, t) in items.iter().enumerate() {
        lines.push(format!(
            "{}. [{}] {} — {}",
            i + 1,
            t.status,
            t.id,
            t.content
        ));
    }
    lines.join("\n")
}

pub fn read(discord_channel_id: Option<u64>) -> Vec<TodoItem> {
    let key = session_key(discord_channel_id);
    store()
        .lock()
        .ok()
        .and_then(|g| g.get(&key).cloned())
        .unwrap_or_default()
}

pub fn clear(discord_channel_id: Option<u64>) {
    let key = session_key(discord_channel_id);
    if let Ok(mut g) = store().lock() {
        g.remove(&key);
    }
}

/// Handle `TODO:` tool args.
/// - empty / `read` → list
/// - `clear` → wipe
/// - `set <json array>` → replace
/// - `merge <json array>` → merge by id
/// - `add <id> | <content>` → append pending
/// - `done <id>` / `cancel <id>` / `start <id>` → status update
pub fn handle_todo(arg: &str, discord_channel_id: Option<u64>) -> String {
    let arg = arg.trim();
    let key = session_key(discord_channel_id);
    let lower = arg.to_lowercase();

    if arg.is_empty() || lower == "read" || lower == "list" {
        return format_todos(&read(discord_channel_id));
    }
    if lower == "clear" {
        clear(discord_channel_id);
        return "TODO list cleared.".to_string();
    }

    let mut guard = match store().lock() {
        Ok(g) => g,
        Err(_) => return "TODO store locked.".to_string(),
    };
    let items = guard.entry(key).or_default();

    if let Some(rest) = arg.strip_prefix("set ").or_else(|| arg.strip_prefix("SET ")) {
        match parse_items_json(rest.trim()) {
            Ok(list) => {
                *items = list;
                return format!("TODO replaced.\n{}", format_todos(items));
            }
            Err(e) => return e,
        }
    }
    if let Some(rest) = arg
        .strip_prefix("merge ")
        .or_else(|| arg.strip_prefix("MERGE "))
    {
        match parse_items_json(rest.trim()) {
            Ok(incoming) => {
                for t in incoming {
                    if let Some(existing) = items.iter_mut().find(|x| x.id == t.id) {
                        if !t.content.is_empty() {
                            existing.content = t.content;
                        }
                        if validate_status(&t.status) {
                            existing.status = t.status;
                        }
                    } else {
                        items.push(t);
                    }
                }
                return format!("TODO merged.\n{}", format_todos(items));
            }
            Err(e) => return e,
        }
    }
    if let Some(rest) = arg.strip_prefix("add ").or_else(|| arg.strip_prefix("ADD ")) {
        let rest = rest.trim();
        let (id, content) = if let Some((a, b)) = rest.split_once('|') {
            (a.trim().to_string(), b.trim().to_string())
        } else if let Some((a, b)) = rest.split_once(' ') {
            (a.trim().to_string(), b.trim().to_string())
        } else {
            (format!("t{}", items.len() + 1), rest.to_string())
        };
        if content.is_empty() {
            return "TODO add requires content. Usage: TODO: add <id> | <content>".to_string();
        }
        if let Some(ex) = items.iter_mut().find(|x| x.id == id) {
            ex.content = content;
            ex.status = "pending".into();
        } else {
            items.push(TodoItem {
                id,
                content,
                status: "pending".into(),
            });
        }
        return format!("TODO updated.\n{}", format_todos(items));
    }
    for (prefix, status) in [
        ("done ", "completed"),
        ("complete ", "completed"),
        ("cancel ", "cancelled"),
        ("start ", "in_progress"),
        ("pending ", "pending"),
    ] {
        let lower = arg.to_lowercase();
        if let Some(id) = lower.strip_prefix(prefix).map(|s| s.trim()) {
            if let Some(ex) = items.iter_mut().find(|x| x.id.eq_ignore_ascii_case(id)) {
                ex.status = status.into();
                let id_owned = ex.id.clone();
                let snapshot = items.clone();
                return format!(
                    "TODO {} → {}.\n{}",
                    id_owned,
                    status,
                    format_todos(&snapshot)
                );
            }
            return format!("TODO id '{}' not found.\n{}", id, format_todos(items));
        }
    }

    "Usage: TODO: (read) | add <id> | <content> | done <id> | start <id> | cancel <id> | set <json> | merge <json> | clear".to_string()
}

fn parse_items_json(s: &str) -> Result<Vec<TodoItem>, String> {
    let v: serde_json::Value =
        serde_json::from_str(s).map_err(|e| format!("Invalid TODO JSON: {}", e))?;
    let arr = v
        .as_array()
        .ok_or_else(|| "TODO JSON must be an array of {id,content,status}".to_string())?;
    let mut out = Vec::new();
    for item in arr {
        let id = item
            .get("id")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let content = item
            .get("content")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let status = item
            .get("status")
            .and_then(|x| x.as_str())
            .unwrap_or("pending")
            .trim()
            .to_lowercase();
        if id.is_empty() || content.is_empty() {
            continue;
        }
        let status = if validate_status(&status) {
            status
        } else {
            "pending".into()
        };
        out.push(TodoItem {
            id,
            content,
            status,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_done() {
        clear(Some(42));
        let r = handle_todo("add a1 | Ship Agent Ops", Some(42));
        assert!(r.contains("Ship Agent Ops"));
        let r = handle_todo("done a1", Some(42));
        assert!(r.contains("completed"));
        clear(Some(42));
    }
}
