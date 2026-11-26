//! OpenClaw-style session-scoped browser tab tracking.
//!
//! Tabs opened via `BROWSER_NAVIGATE` with `new_tab` are recorded under the active
//! agent session key. On Discord/UI session clear we close those targets so Chrome
//! does not keep orphaned automation tabs after reset.

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

fn tracked_tabs() -> &'static Mutex<HashMap<String, HashSet<String>>> {
    static S: OnceLock<Mutex<HashMap<String, HashSet<String>>>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(HashMap::new()))
}

fn active_session_key() -> &'static Mutex<Option<String>> {
    static S: OnceLock<Mutex<Option<String>>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(None))
}

/// Session memory style key: `{source}-{session_id}` (e.g. `discord-123`).
pub fn session_key(source: &str, session_id: u64) -> String {
    format!("{source}-{session_id}")
}

/// RAII owner for the active automation session key during an agent turn.
pub struct ActiveBrowserSessionGuard {
    key: String,
}

impl ActiveBrowserSessionGuard {
    /// Enter ownership for a Discord channel or the shared local slot.
    pub fn enter(discord_channel_id: Option<u64>) -> Self {
        let key = match discord_channel_id {
            Some(id) => session_key("discord", id),
            None => session_key("local", 0),
        };
        if let Ok(mut g) = active_session_key().lock() {
            *g = Some(key.clone());
        }
        Self { key }
    }
}

impl Drop for ActiveBrowserSessionGuard {
    fn drop(&mut self) {
        if let Ok(mut g) = active_session_key().lock() {
            if g.as_deref() == Some(self.key.as_str()) {
                *g = None;
            }
        }
    }
}

fn normalize_target_id(raw: &str) -> Option<String> {
    let t = raw.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

/// Record a tab opened for the active session (no-op when no active owner).
pub fn track_for_active_session(target_id: &str) {
    let Some(tid) = normalize_target_id(target_id) else {
        return;
    };
    let key = match active_session_key().lock() {
        Ok(g) => g.clone(),
        Err(_) => None,
    };
    let Some(key) = key else {
        return;
    };
    if let Ok(mut map) = tracked_tabs().lock() {
        map.entry(key).or_default().insert(tid);
    }
}

/// Drop a target from every session (e.g. after `BROWSER_CLOSE_TAB`).
pub fn untrack_everywhere(target_id: &str) {
    let Some(tid) = normalize_target_id(target_id) else {
        return;
    };
    if let Ok(mut map) = tracked_tabs().lock() {
        let mut empty_keys = Vec::new();
        for (key, set) in map.iter_mut() {
            set.remove(&tid);
            if set.is_empty() {
                empty_keys.push(key.clone());
            }
        }
        for key in empty_keys {
            map.remove(&key);
        }
    }
}

/// Take (and clear) all tracked target ids for a session key.
pub fn take_tracked_for_session(session_key: &str) -> Vec<String> {
    let key = session_key.trim();
    if key.is_empty() {
        return Vec::new();
    }
    let Ok(mut map) = tracked_tabs().lock() else {
        return Vec::new();
    };
    let Some(set) = map.remove(key) else {
        return Vec::new();
    };
    set.into_iter().collect()
}

/// Re-insert targets after a failed close attempt.
pub fn retrack_for_session(session_key: &str, target_ids: impl IntoIterator<Item = String>) {
    let key = session_key.trim();
    if key.is_empty() {
        return;
    }
    let mut ids: HashSet<String> = target_ids
        .into_iter()
        .filter_map(|t| normalize_target_id(&t))
        .collect();
    if ids.is_empty() {
        return;
    }
    if let Ok(mut map) = tracked_tabs().lock() {
        map.entry(key.to_string())
            .or_default()
            .extend(ids.drain());
    }
}

/// Count of tracked tabs for a session (tests / diagnostics).
#[cfg(test)]
pub fn count_tracked_for_session(session_key: &str) -> usize {
    let Ok(map) = tracked_tabs().lock() else {
        return 0;
    };
    map.get(session_key).map(|s| s.len()).unwrap_or(0)
}

#[cfg(test)]
pub fn reset_for_tests() {
    if let Ok(mut map) = tracked_tabs().lock() {
        map.clear();
    }
    if let Ok(mut g) = active_session_key().lock() {
        *g = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn registry_test_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn track_requires_active_session() {
        let _lock = registry_test_lock();
        reset_for_tests();
        track_for_active_session("tab-a");
        assert_eq!(count_tracked_for_session("discord-1"), 0);
        assert_eq!(count_tracked_for_session("local-0"), 0);

        let _g = ActiveBrowserSessionGuard::enter(Some(1));
        track_for_active_session("tab-a");
        track_for_active_session("  tab-b  ");
        track_for_active_session("");
        assert_eq!(count_tracked_for_session("discord-1"), 2);

        let taken = take_tracked_for_session("discord-1");
        assert_eq!(taken.len(), 2);
        assert_eq!(count_tracked_for_session("discord-1"), 0);
        reset_for_tests();
    }

    #[test]
    fn untrack_everywhere_clears_empty_sessions() {
        let _lock = registry_test_lock();
        reset_for_tests();
        let _g = ActiveBrowserSessionGuard::enter(Some(9));
        track_for_active_session("only");
        untrack_everywhere("only");
        assert_eq!(count_tracked_for_session("discord-9"), 0);
        reset_for_tests();
    }

    #[test]
    fn retrack_restores_after_failed_close() {
        let _lock = registry_test_lock();
        reset_for_tests();
        retrack_for_session(
            "discord-2",
            vec!["t1".into(), "t2".into(), "".into()],
        );
        assert_eq!(count_tracked_for_session("discord-2"), 2);
        let taken = take_tracked_for_session("discord-2");
        retrack_for_session("discord-2", taken);
        assert_eq!(count_tracked_for_session("discord-2"), 2);
        reset_for_tests();
    }

    #[test]
    fn local_guard_uses_local_0() {
        let _lock = registry_test_lock();
        reset_for_tests();
        let _g = ActiveBrowserSessionGuard::enter(None);
        track_for_active_session("ui-tab");
        assert_eq!(count_tracked_for_session("local-0"), 1);
        reset_for_tests();
    }
}
