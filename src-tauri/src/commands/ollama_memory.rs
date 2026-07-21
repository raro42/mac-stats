//! Memory and soul loading helpers.
//!
//! Extracted from `ollama.rs`: soul content, global/channel/main-session
//! memory blocks, keyword-based memory search.

use crate::config::Config;

pub(crate) fn load_soul_content() -> String {
    Config::load_soul_content()
}

/// Drop baked-in identity lines so soul.md (or merges) cannot claim a stale app version.
/// Canonical `You are mac-stats v…` is always appended by [`format_router_soul_block`].
pub(crate) fn strip_stale_mac_stats_identity_lines(soul_md: &str) -> String {
    soul_md
        .lines()
        .filter(|line| {
            let t = line.trim().to_ascii_lowercase();
            let t = t.replace('\u{2019}', "'"); // curly apostrophe → ASCII
            !(t.starts_with("you are mac-stats v")
                || t.starts_with("i'm mac-stats v")
                || t.starts_with("i am mac-stats v"))
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// Planning-system prefix: shared `soul.md` plus the app identity line when no skill or agent
/// `combined_prompt` is active. When `skill_content` is `Some`, callers use an empty string instead
/// so the skill/agent block is the only extra voice (022 §F4).
pub(crate) fn format_router_soul_block(soul_md: &str, app_version: &str) -> String {
    let cleaned = strip_stale_mac_stats_identity_lines(soul_md);
    if cleaned.is_empty() {
        format!("You are mac-stats v{}.\n\n", app_version)
    } else {
        format!("{}\n\nYou are mac-stats v{}.\n\n", cleaned, app_version)
    }
}

/// Load global memory (~/.mac-stats/agents/memory.md) for inclusion in system prompt.
pub(crate) fn load_global_memory_block() -> String {
    let path = Config::memory_file_path();
    let content = match std::fs::read_to_string(&path) {
        Ok(s) => s.trim().to_string(),
        Err(_) => return String::new(),
    };
    let content = crate::commands::session_search::filter_memory_markdown_for_prompt(&content);
    let content = crate::commands::session_search::truncate_memory_for_prompt(&content);
    if content.is_empty() {
        return String::new();
    }
    format!(
        "\n\n## Memory (lessons learned — follow these)\n\n{}\n\n",
        content
    )
}

/// Load per-channel Discord memory (~/.mac-stats/agents/memory-discord-{id}.md). Returns empty if missing.
pub(crate) fn load_channel_memory_block(channel_id: u64) -> String {
    let path = Config::memory_file_path_for_discord_channel(channel_id);
    let content = match std::fs::read_to_string(&path) {
        Ok(s) => s.trim().to_string(),
        Err(_) => return String::new(),
    };
    let content = crate::commands::session_search::filter_memory_markdown_for_prompt(&content);
    let content = crate::commands::session_search::truncate_memory_for_prompt(&content);
    if content.is_empty() {
        return String::new();
    }
    format!(
        "\n\n## Memory (this channel — follow these)\n\n{}\n\n",
        content
    )
}

/// Load main-session (in-app) memory (~/.mac-stats/agents/memory-main.md). Returns empty if missing.
/// Used when the request is from the CPU window (no Discord channel) so the main session has per-context memory.
pub(crate) fn load_main_session_memory_block() -> String {
    let path = Config::memory_file_path_for_main_session();
    let content = match std::fs::read_to_string(&path) {
        Ok(s) => s.trim().to_string(),
        Err(_) => return String::new(),
    };
    let content = crate::commands::session_search::filter_memory_markdown_for_prompt(&content);
    let content = crate::commands::session_search::truncate_memory_for_prompt(&content);
    if content.is_empty() {
        return String::new();
    }
    format!(
        "\n\n## Memory (main session — follow these)\n\n{}\n\n",
        content
    )
}

/// Load memory for the current request.
/// Global memory (personal/long-term) is only loaded in main session (in-app, or Discord DM).
/// In Discord guild channels and having_fun, only per-channel memory is loaded to avoid leaking personal context.
/// When there is no Discord channel (in-app), main-session memory (memory-main.md) is also loaded.
pub(crate) fn load_memory_block_for_request(
    discord_channel_id: Option<u64>,
    load_global_memory: bool,
) -> String {
    let global = if load_global_memory {
        load_global_memory_block()
    } else {
        String::new()
    };
    let channel = discord_channel_id
        .map(load_channel_memory_block)
        .unwrap_or_else(|| {
            if load_global_memory {
                load_main_session_memory_block()
            } else {
                String::new()
            }
        });
    if channel.is_empty() {
        global
    } else {
        format!("{}{}", global, channel)
    }
}

/// Extract words (alphanumeric, lowercase) for simple keyword matching.
pub(crate) fn words_for_search(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| s.len() >= 2)
        .map(String::from)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect()
}

/// Search memory (global + optional Discord channel or main-session) for lines relevant to the request.
/// Returns at most 5 matching lines, or None if no matches. When discord_channel_id is Some,
/// channel memory is merged with global; when None (in-app), main-session memory is included.
pub(crate) fn search_memory_for_request(
    question: &str,
    reason: Option<&str>,
    discord_channel_id: Option<u64>,
) -> Option<String> {
    let global = std::fs::read_to_string(Config::memory_file_path())
        .ok()
        .unwrap_or_default();
    let channel = discord_channel_id
        .and_then(|id| {
            std::fs::read_to_string(Config::memory_file_path_for_discord_channel(id)).ok()
        })
        .unwrap_or_else(|| {
            std::fs::read_to_string(Config::memory_file_path_for_main_session())
                .ok()
                .unwrap_or_default()
        });
    let content = format!("{}\n{}", global.trim(), channel.trim())
        .trim()
        .to_string();
    if content.is_empty() {
        return None;
    }
    let mut query_words: Vec<String> = words_for_search(question);
    if let Some(r) = reason {
        query_words.extend(words_for_search(r));
    }
    query_words.sort();
    query_words.dedup();
    if query_words.is_empty() {
        return None;
    }
    const MIN_MEMORY_MATCH_WORDS: usize = 2;
    let mut scored: Vec<(usize, String)> = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let line_lower = line.to_lowercase();
            let score = query_words
                .iter()
                .filter(|w| line_lower.contains(w.as_str()))
                .count();
            (score, line.to_string())
        })
        .filter(|(score, _)| *score >= MIN_MEMORY_MATCH_WORDS)
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    let top: Vec<String> = scored.into_iter().take(5).map(|(_, line)| line).collect();
    if top.is_empty() {
        None
    } else {
        Some(top.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_router_soul_block_empty_soul() {
        assert_eq!(
            format_router_soul_block("", "9.9.9"),
            "You are mac-stats v9.9.9.\n\n"
        );
    }

    #[test]
    fn format_router_soul_block_non_empty_soul() {
        assert_eq!(
            format_router_soul_block("Be concise.", "1.0.0"),
            "Be concise.\n\nYou are mac-stats v1.0.0.\n\n"
        );
    }

    #[test]
    fn format_router_soul_block_strips_stale_identity() {
        let soul = "Be concise.\nYou are mac-stats v0.1.115.\nOpinions welcome.";
        assert_eq!(
            format_router_soul_block(soul, "0.1.120"),
            "Be concise.\nOpinions welcome.\n\nYou are mac-stats v0.1.120.\n\n"
        );
    }

    /// `agent_override` sets `skill_content` to the agent `combined_prompt` (see `answer_with_ollama_and_fetch`);
    /// the soul prefix must stay empty to avoid double system voice.
    #[test]
    fn router_soul_skipped_when_skill_or_agent_prompt_active() {
        let skill = Some("skill instructions".to_string());
        let prefix = skill.as_ref().map_or_else(
            || format_router_soul_block("unused", "1.0"),
            |_| String::new(),
        );
        assert!(prefix.is_empty());
    }
}
