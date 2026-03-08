//! User information from ~/.mac-stats/user-info.json.
//!
//! Supports many users keyed by id (e.g. Discord user id). Details are merged into
//! the agent context when the request comes from a known user (e.g. Discord message author).
//! When a Discord message is handled, the stored display_name for that user is updated if it
//! differs from the author's current display name (auto-sync).

use crate::config::Config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::debug;

/// Per-user details stored in user-info.json.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserDetails {
    /// Override display name (e.g. preferred name). If unset, Discord/context name is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Free-form notes about the user (e.g. preferences, timezone).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// Timezone (e.g. "Europe/Paris") for time-sensitive answers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    /// Extra key-value details for future use.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, String>,
}

/// File format: list of users with id and details.
#[derive(Debug, Default, Serialize, Deserialize)]
struct UserInfoFile {
    #[serde(default)]
    users: Vec<UserEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserEntry {
    /// User identifier (e.g. Discord snowflake as string).
    id: String,
    #[serde(flatten)]
    details: UserDetails,
}

/// In-memory map: id -> details. Loaded on each lookup so file changes are picked up.
fn load_user_info_map() -> HashMap<String, UserDetails> {
    let path = Config::user_info_file_path();
    if !path.exists() {
        return HashMap::new();
    }
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            debug!("user_info: failed to read {:?}: {}", path, e);
            return HashMap::new();
        }
    };
    let file_data: UserInfoFile = match serde_json::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            debug!("user_info: failed to parse {:?}: {}", path, e);
            return HashMap::new();
        }
    };
    let mut map = HashMap::new();
    for entry in file_data.users {
        if !entry.id.is_empty() {
            map.insert(entry.id, entry.details);
        }
    }
    map
}

/// Get stored details for a user by id (e.g. Discord user id as u64). Returns None if not found or file missing.
pub fn get_user_details(user_id: u64) -> Option<UserDetails> {
    let id_str = user_id.to_string();
    load_user_info_map().get(&id_str).cloned()
}

/// Load the full file structure (for round-trip read → modify → write).
fn load_user_info_file(path: &Path) -> UserInfoFile {
    if !path.exists() {
        return UserInfoFile::default();
    }
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            debug!("user_info: failed to read {:?}: {}", path, e);
            return UserInfoFile::default();
        }
    };
    match serde_json::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            debug!("user_info: failed to parse {:?}: {}", path, e);
            UserInfoFile::default()
        }
    }
}

/// If the stored display_name for this user differs from the given one (or is missing),
/// update it in ~/.mac-stats/user-info.json. If the user is not in the file, add a minimal
/// entry so the file stays in sync with Discord. No-op if display_name is empty.
pub fn maybe_update_display_name_from_discord(user_id: u64, display_name: &str) {
    let name = display_name.trim();
    if name.is_empty() {
        return;
    }
    let path = Config::user_info_file_path();
    let mut file_data = load_user_info_file(&path);

    let id_str = user_id.to_string();
    let mut changed = false;
    if let Some(entry) = file_data.users.iter_mut().find(|e| e.id == id_str) {
        let current = entry.details.display_name.as_deref().unwrap_or("");
        if current != name {
            entry.details.display_name = Some(name.to_string());
            changed = true;
        }
    } else {
        let details = UserDetails {
            display_name: Some(name.to_string()),
            ..Default::default()
        };
        file_data.users.push(UserEntry {
            id: id_str,
            details,
        });
        changed = true;
    }

    if !changed {
        return;
    }
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match std::fs::write(
        &path,
        serde_json::to_string_pretty(&file_data).unwrap_or_default(),
    ) {
        Ok(()) => debug!("user_info: updated display_name for user {} to {:?}", user_id, name),
        Err(e) => debug!("user_info: failed to write {:?}: {}", path, e),
    }
}

/// Build a one-line summary of user details for the agent context (e.g. "Notes: …; Timezone: …").
pub fn format_user_details_for_context(details: &UserDetails) -> String {
    let mut parts = Vec::new();
    if let Some(ref n) = details.notes {
        if !n.is_empty() {
            parts.push(format!("Notes: {}", n));
        }
    }
    if let Some(ref t) = details.timezone {
        if !t.is_empty() {
            parts.push(format!("Timezone: {}", t));
        }
    }
    for (k, v) in &details.extra {
        if !k.is_empty() && !v.is_empty() {
            parts.push(format!("{}: {}", k, v));
        }
    }
    if parts.is_empty() {
        String::new()
    } else {
        parts.join("; ")
    }
}
