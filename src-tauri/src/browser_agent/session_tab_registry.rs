//! OpenClaw-style session-scoped browser tab tracking.
//!
//! Tabs opened via `BROWSER_NAVIGATE` with `new_tab` are recorded under the active
//! agent session key. On Discord/UI session clear we close those targets so Chrome
//! does not keep orphaned automation tabs after reset.
//!
//! Each target stores `tracked_at` (OpenClaw `trackedAt`) so `/browser` can show
//! how long the oldest close-on-reset tab has been waiting.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

/// Target id → when it was first tracked for this session.
type TrackedTargets = HashMap<String, Instant>;

fn tracked_tabs() -> &'static Mutex<HashMap<String, TrackedTargets>> {
    static S: OnceLock<Mutex<HashMap<String, TrackedTargets>>> = OnceLock::new();
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
/// Re-tracking the same target keeps the original `tracked_at`.
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
        let entry = map.entry(key).or_default();
        entry.entry(tid).or_insert_with(Instant::now);
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
    set.into_keys().collect()
}

/// Re-insert targets after a failed close attempt (fresh `tracked_at`).
pub fn retrack_for_session(session_key: &str, target_ids: impl IntoIterator<Item = String>) {
    let key = session_key.trim();
    if key.is_empty() {
        return;
    }
    let now = Instant::now();
    let mut ids: TrackedTargets = target_ids
        .into_iter()
        .filter_map(|t| normalize_target_id(&t).map(|id| (id, now)))
        .collect();
    if ids.is_empty() {
        return;
    }
    if let Ok(mut map) = tracked_tabs().lock() {
        let entry = map.entry(key.to_string()).or_default();
        for (id, at) in ids.drain() {
            entry.entry(id).or_insert(at);
        }
    }
}

/// Count of tracked tabs for a session (diagnostics / `/browser` glance).
pub fn count_tracked_for_session(session_key: &str) -> usize {
    let Ok(map) = tracked_tabs().lock() else {
        return 0;
    };
    map.get(session_key).map(|s| s.len()).unwrap_or(0)
}

/// Total automation tabs tracked across all sessions (OpenClaw close-on-reset leftover).
pub fn count_tracked_tabs_total() -> usize {
    let Ok(map) = tracked_tabs().lock() else {
        return 0;
    };
    map.values().map(|s| s.len()).sum()
}

/// Session keys with ≥1 tracked tab (OpenClaw per-session map leftover).
/// Distinct from tab count — two Discord channels can each hold close-on-reset tabs.
pub fn count_tracked_sessions() -> usize {
    let Ok(map) = tracked_tabs().lock() else {
        return 0;
    };
    map.values().filter(|s| !s.is_empty()).count()
}

/// Age in seconds of the oldest tracked tab (OpenClaw `trackedAt` leftover).
/// `None` when nothing is tracked.
pub fn oldest_tracked_age_secs() -> Option<u64> {
    let Ok(map) = tracked_tabs().lock() else {
        return None;
    };
    let mut oldest: Option<Instant> = None;
    for set in map.values() {
        for at in set.values() {
            oldest = Some(match oldest {
                Some(prev) if *at < prev => *at,
                Some(prev) => prev,
                None => *at,
            });
        }
    }
    oldest.map(|at| at.elapsed().as_secs())
}

/// Active automation owner key while an agent turn holds `ActiveBrowserSessionGuard`.
/// Distinct from tracked tab/session counts — shows who may open close-on-reset tabs now.
/// `None` when no turn owns the browser (calm omit on `/browser`).
pub fn active_session_owner() -> Option<String> {
    let Ok(g) = active_session_key().lock() else {
        return None;
    };
    g.as_ref()
        .map(|k| k.trim().to_string())
        .filter(|k| !k.is_empty())
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
        assert_eq!(count_tracked_tabs_total(), 2);
        assert_eq!(count_tracked_sessions(), 1);

        let taken = take_tracked_for_session("discord-1");
        assert_eq!(taken.len(), 2);
        assert_eq!(count_tracked_for_session("discord-1"), 0);
        assert_eq!(count_tracked_tabs_total(), 0);
        assert_eq!(count_tracked_sessions(), 0);
        reset_for_tests();
    }

    #[test]
    fn count_tracked_sessions_spans_keys() {
        let _lock = registry_test_lock();
        reset_for_tests();
        let _g1 = ActiveBrowserSessionGuard::enter(Some(1));
        track_for_active_session("a");
        drop(_g1);
        let _g2 = ActiveBrowserSessionGuard::enter(Some(2));
        track_for_active_session("b");
        track_for_active_session("c");
        assert_eq!(count_tracked_tabs_total(), 3);
        assert_eq!(count_tracked_sessions(), 2);
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

    #[test]
    fn oldest_tracked_age_secs_reports_when_tracked() {
        let _lock = registry_test_lock();
        reset_for_tests();
        assert_eq!(oldest_tracked_age_secs(), None);
        let _g = ActiveBrowserSessionGuard::enter(Some(3));
        track_for_active_session("aged");
        let age = oldest_tracked_age_secs();
        assert!(age.is_some(), "expected age after track");
        assert!(age.unwrap() < 5, "fresh track should be near zero secs");
        // Re-track same id must keep original tracked_at (not reset).
        std::thread::sleep(std::time::Duration::from_millis(20));
        track_for_active_session("aged");
        let age2 = oldest_tracked_age_secs().unwrap_or(0);
        assert!(age2 >= age.unwrap_or(0), "re-track must not rewind age");
        reset_for_tests();
        assert_eq!(oldest_tracked_age_secs(), None);
    }

    #[test]
    fn active_session_owner_follows_guard() {
        let _lock = registry_test_lock();
        reset_for_tests();
        assert_eq!(active_session_owner(), None);
        {
            let _g = ActiveBrowserSessionGuard::enter(Some(42));
            assert_eq!(active_session_owner().as_deref(), Some("discord-42"));
        }
        assert_eq!(active_session_owner(), None);
        let _local = ActiveBrowserSessionGuard::enter(None);
        assert_eq!(active_session_owner().as_deref(), Some("local-0"));
        reset_for_tests();
        assert_eq!(active_session_owner(), None);
    }
}
