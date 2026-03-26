//! Ordered outbound delivery: serial sends, per-send timeouts, payload deduplication,
//! and per-surface chunk policies (Discord, in-app chat, menu bar).
//!
//! See task WIP/UNTESTED ordered outbound reply pipeline (OpenClaw parity).
#![allow(dead_code)]
// `BreakPreference` / `menu_bar_default` / `clip_for_menu_bar_line` are part of the public surface for future surfaces (e.g. menu bar status text).

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

use tracing::warn;

/// Discord API content limit (characters).
pub const DISCORD_CONTENT_MAX_CHARS: usize = 2000;

/// Pause between sequential Discord chunks (rate / ordering comfort).
pub const DISCORD_INTER_CHUNK_DELAY_MS: u64 = 300;

/// Default wall-clock budget for a single outbound send (e.g. one Discord `say`).
const DEFAULT_PER_SEND_TIMEOUT_SECS: u64 = 10;

/// Preferred split boundary when chunking text for a surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BreakPreference {
    Line,
    Sentence,
    Word,
}

/// Per-surface limits and coalescing hints.
#[derive(Clone, Debug)]
pub struct SurfaceChunkPolicy {
    pub max_chars: usize,
    /// For streaming UIs: milliseconds to wait for more text before flushing a chunk.
    /// `0` means emit immediately (no idle coalescing).
    pub coalesce_idle_ms: u64,
    pub break_preference: BreakPreference,
}

impl SurfaceChunkPolicy {
    pub fn discord_default() -> Self {
        Self {
            max_chars: DISCORD_CONTENT_MAX_CHARS,
            coalesce_idle_ms: 0,
            break_preference: BreakPreference::Line,
        }
    }

    /// CPU / Tauri chat: effectively unlimited chunks; small idle coalesce batches rapid stream tokens.
    pub fn tauri_ui_default() -> Self {
        Self {
            max_chars: usize::MAX,
            coalesce_idle_ms: 50,
            break_preference: BreakPreference::Line,
        }
    }

    /// Menu bar title / one-line status (metrics line is separate).
    pub fn menu_bar_default() -> Self {
        Self {
            max_chars: 120,
            coalesce_idle_ms: 0,
            break_preference: BreakPreference::Word,
        }
    }
}

/// Stable hash for deduplicating outbound payloads in one logical reply.
pub fn payload_key(content: &str, message_id: Option<&str>) -> u64 {
    let mut h = DefaultHasher::new();
    message_id.hash(&mut h);
    content.hash(&mut h);
    h.finish()
}

/// Tracks keys already sent for the current assistant reply.
#[derive(Debug, Default)]
pub struct ReplyDedupState {
    sent: std::collections::HashSet<u64>,
}

impl ReplyDedupState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if this payload is new and should be sent.
    pub fn register_if_new(&mut self, content: &str, message_id: Option<&str>) -> bool {
        let k = payload_key(content, message_id);
        self.sent.insert(k)
    }

    pub fn clear(&mut self) {
        self.sent.clear();
    }
}

fn find_sentence_break_before(byte_limit: usize, text: &str) -> Option<usize> {
    let head = text.get(..byte_limit)?;
    for sep in [". ", ".\n", "? ", "? \n", "! ", "!\n"] {
        if let Some(i) = head.rfind(sep) {
            return Some(i + sep.len());
        }
    }
    head.rfind(". ").or_else(|| head.rfind('.')).map(|i| i + 1)
}

fn find_word_break_before(byte_limit: usize, text: &str) -> Option<usize> {
    let head = text.get(..byte_limit)?;
    head.rfind(' ').map(|i| i + 1)
}

/// Split `text` into blocks of at most `policy.max_chars`, preferring `break_preference`.
pub fn split_text_for_policy(text: &str, policy: &SurfaceChunkPolicy) -> Vec<String> {
    let t = text.trim();
    if t.is_empty() {
        return Vec::new();
    }
    if policy.max_chars == usize::MAX {
        return vec![t.to_string()];
    }
    let max = policy.max_chars.max(1);
    let mut out = Vec::new();
    let mut remaining = t.to_string();
    while !remaining.is_empty() {
        let nchars = remaining.chars().count();
        if nchars <= max {
            out.push(remaining);
            break;
        }
        let byte_pos = remaining
            .char_indices()
            .take(max)
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(0);
        let head = &remaining[..byte_pos];
        let tail = &remaining[byte_pos..];
        let split_byte = match policy.break_preference {
            BreakPreference::Line => head
                .rfind('\n')
                .map(|i| i + 1)
                .filter(|&i| i > 0 && i < byte_pos),
            BreakPreference::Sentence => {
                find_sentence_break_before(byte_pos, &remaining).filter(|&i| i > 0 && i < byte_pos)
            }
            BreakPreference::Word => {
                find_word_break_before(byte_pos, &remaining).filter(|&i| i > 0 && i < byte_pos)
            }
        };
        let (chunk, rest) = if let Some(sb) = split_byte {
            remaining.split_at(sb)
        } else {
            (head, tail)
        };
        let chunk = chunk.to_string();
        let rest = rest.to_string();
        if chunk.is_empty() {
            out.push(head.to_string());
            remaining = tail.to_string();
        } else {
            out.push(chunk);
            remaining = rest;
        }
    }
    out
}

/// Discord: optional paragraph split (`split_long`), then per-message limits.
pub fn split_discord_reply(text: &str, split_long: bool) -> Vec<String> {
    let policy = SurfaceChunkPolicy::discord_default();
    if !split_long {
        return split_text_for_policy(text, &policy);
    }
    let parts: Vec<&str> = text
        .split("\n\n")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if parts.len() <= 1 {
        return split_text_for_policy(text, &policy);
    }
    let mut out = Vec::new();
    for p in parts {
        out.extend(split_text_for_policy(p, &policy));
    }
    out
}

/// Clip a single line of status text for the menu bar using the menu-bar policy.
pub fn clip_for_menu_bar_line(text: &str) -> String {
    let policy = SurfaceChunkPolicy::menu_bar_default();
    let v = split_text_for_policy(text, &policy);
    v.into_iter().next().unwrap_or_default()
}

/// Env `MAC_STATS_OUTBOUND_SEND_TIMEOUT_SECS` (default 10), clamped 1..=120.
pub fn per_send_timeout() -> Duration {
    let secs = std::env::var("MAC_STATS_OUTBOUND_SEND_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(DEFAULT_PER_SEND_TIMEOUT_SECS)
        .clamp(1, 120);
    Duration::from_secs(secs)
}

/// Log and return `true` when the caller should abort remaining blocks.
pub fn log_send_timeout(surface: &str, part_index: usize, total: usize) -> bool {
    warn!(
        target: "outbound_pipeline",
        "outbound {}: per-send timeout (part {}/{}); aborting remaining blocks",
        surface, part_index, total
    );
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_discord_respects_max() {
        let s = "a".repeat(2500);
        let chunks = split_discord_reply(&s, false);
        assert!(chunks.len() >= 2);
        assert!(chunks.iter().all(|c| c.chars().count() <= DISCORD_CONTENT_MAX_CHARS));
    }

    #[test]
    fn dedup_skips_identical_chunks() {
        let mut d = ReplyDedupState::new();
        assert!(d.register_if_new("hello", None));
        assert!(!d.register_if_new("hello", None));
        assert!(d.register_if_new("hello", Some("m1")));
    }

    #[test]
    fn menu_bar_clip_short() {
        assert_eq!(clip_for_menu_bar_line("hi"), "hi");
    }

    #[test]
    fn policies_and_break_preferences_cover_api() {
        let _ = SurfaceChunkPolicy::menu_bar_default();
        let mut d = ReplyDedupState::new();
        d.clear();
        let p = SurfaceChunkPolicy {
            max_chars: 40,
            coalesce_idle_ms: 0,
            break_preference: BreakPreference::Sentence,
        };
        let s = "First sentence. Second sentence is longer and should split here.";
        let v = split_text_for_policy(s, &p);
        assert!(v.len() >= 1);
        let p2 = SurfaceChunkPolicy {
            max_chars: 20,
            coalesce_idle_ms: 0,
            break_preference: BreakPreference::Word,
        };
        let v2 = split_text_for_policy("hello world from rust outbound", &p2);
        assert!(v2.iter().all(|c| c.chars().count() <= 20));
    }
}
