//! Hermes-style cross-session recall over `~/.mac-stats/session/*.md`.
//!
//! Substring search + extractive per-file summaries (no SQLite FTS / summarizer LLM).

use crate::config::Config;
use crate::session_memory::parse_session_markdown;
use std::fs;

const MAX_FILES: usize = 5;
const MAX_SNIPPETS_PER_FILE: usize = 2;
const SNIPPET_CHARS: usize = 140;
const MAX_SUMMARY_TURNS: usize = 3;
const USER_SUMMARY_CHARS: usize = 140;
const ASSISTANT_SUMMARY_CHARS: usize = 220;

/// `SESSION_SEARCH: <query>` — search persisted session markdown.
///
/// When `exclude_session_id` is set (Hermes parity), skip files for that session — the agent
/// already has the current transcript in context.
pub fn handle_session_search(arg: &str) -> String {
    handle_session_search_excluding(arg, None)
}

pub fn handle_session_search_excluding(arg: &str, exclude_session_id: Option<u64>) -> String {
    let query = arg.trim();
    if query.chars().count() < 2 {
        return "SESSION_SEARCH requires a query (at least 2 characters).".to_string();
    }
    let dir = Config::session_dir();
    if !dir.is_dir() {
        return format!("No session directory at {}.", dir.display());
    }
    let q_lower = query.to_lowercase();
    let mut hits: Vec<FileHit> = Vec::new();
    let mut skipped_current = 0usize;

    let rd = match fs::read_dir(&dir) {
        Ok(r) => r,
        Err(e) => return format!("Cannot read session dir: {}", e),
    };
    for ent in rd.filter_map(Result::ok) {
        let path = ent.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        if !name.starts_with("session-memory-") {
            continue;
        }
        if let Some(sid) = exclude_session_id {
            if crate::session_memory::session_filename_matches_id(&name, sid) {
                skipped_current += 1;
                continue;
            }
        }
        let text = match fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        if !text.to_lowercase().contains(&q_lower) {
            continue;
        }
        let snippets = collect_snippets(&text, &q_lower);
        let summary = extractive_summary(&text, &q_lower);
        let score = snippets.len()
            + text
                .to_lowercase()
                .matches(&q_lower)
                .count()
                .min(20)
            + if summary.is_empty() { 0 } else { 2 };
        let mtime = ent
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        hits.push(FileHit {
            name,
            score,
            mtime,
            summary,
            snippets,
        });
    }

    if hits.is_empty() {
        let mut msg = format!("No past sessions matched {:?}.", query);
        if skipped_current > 0 {
            msg.push_str(&format!(
                " (Skipped {} current-session file(s).)",
                skipped_current
            ));
        }
        return msg;
    }
    hits.sort_by(|a, b| b.score.cmp(&a.score).then(b.mtime.cmp(&a.mtime)));
    hits.truncate(MAX_FILES);

    let mut out = vec![format!(
        "**Session search** for {:?} — {} file(s) (extractive summaries):",
        query,
        hits.len()
    )];
    if skipped_current > 0 {
        out[0].push_str(&format!(
            " · excluded {} current-session file(s)",
            skipped_current
        ));
    }
    for (i, h) in hits.iter().enumerate() {
        let meta = file_meta_line(&h.name);
        out.push(format!(
            "\n{}. `{}`{} (score {})",
            i + 1,
            h.name,
            meta,
            h.score
        ));
        if !h.summary.is_empty() {
            out.push(format!("   **Summary:** {}", h.summary));
        }
        for s in &h.snippets {
            out.push(format!("   - evidence: …{}…", s));
        }
    }
    out.push(
        "\nCite these if relevant; do not invent past decisions. Prefer the summary over raw evidence."
            .into(),
    );
    out.join("\n")
}

struct FileHit {
    name: String,
    score: usize,
    mtime: u64,
    summary: String,
    snippets: Vec<String>,
}

fn file_meta_line(name: &str) -> String {
    // session-memory-{id}-{YYYYMMDD}-{HHMMSS}-{topic}.md
    let stem = name.trim_end_matches(".md");
    let rest = stem.strip_prefix("session-memory-").unwrap_or(stem);
    let mut it = rest.splitn(3, '-');
    let _id = it.next();
    let date = it.next();
    let rest2 = it.next().unwrap_or("");
    let (time, topic) = match rest2.split_once('-') {
        Some((t, topic)) if t.len() == 6 && t.chars().all(|c| c.is_ascii_digit()) => (t, topic),
        _ => ("", rest2),
    };
    match date {
        Some(d) if d.len() == 8 && d.chars().all(|c| c.is_ascii_digit()) => {
            let pretty = format!("{}-{}-{}", &d[0..4], &d[4..6], &d[6..8]);
            let topic = topic
                .replace('_', " ")
                .trim_matches(|c| c == '?' || c == '.')
                .to_string();
            if time.is_empty() {
                format!(" — {pretty} · {topic}")
            } else {
                let clock = format!("{}:{}:{}", &time[0..2], &time[2..4], &time[4..6]);
                format!(" — {pretty} {clock} · {topic}")
            }
        }
        _ => String::new(),
    }
}

/// Hermes-style focused recap without an extra LLM call.
fn extractive_summary(text: &str, q_lower: &str) -> String {
    let turns = parse_session_markdown(text);
    if turns.is_empty() {
        return String::new();
    }

    let mut match_idxs: Vec<usize> = turns
        .iter()
        .enumerate()
        .filter(|(_, (_, c))| c.to_lowercase().contains(q_lower))
        .map(|(i, _)| i)
        .collect();
    if match_idxs.is_empty() {
        // File matched via non-turn text; summarize first exchange.
        match_idxs.push(0);
    }

    let mut parts: Vec<String> = Vec::new();
    let mut used_user: Vec<String> = Vec::new();
    for &idx in match_idxs.iter().take(MAX_SUMMARY_TURNS * 2) {
        let user_idx = nearest_user_before(&turns, idx);
        let asst_idx = nearest_assistant_after(&turns, user_idx.unwrap_or(idx));
        let Some(ui) = user_idx else {
            continue;
        };
        let user = compress_line(&turns[ui].1, USER_SUMMARY_CHARS);
        if user.is_empty() || used_user.iter().any(|u| u == &user) {
            continue;
        }
        used_user.push(user.clone());
        let outcome = asst_idx
            .map(|ai| compress_assistant(&turns[ai].1, ASSISTANT_SUMMARY_CHARS))
            .unwrap_or_else(|| "(no assistant reply)".into());
        let tools = tools_mentioned(&turns[asst_idx.unwrap_or(ui)].1);
        let mut bit = format!("Asked «{user}» → {outcome}");
        if !tools.is_empty() {
            bit.push_str(&format!(" [tools: {}]", tools.join(", ")));
        }
        parts.push(bit);
        if parts.len() >= MAX_SUMMARY_TURNS {
            break;
        }
    }
    parts.join(" · ")
}

fn nearest_user_before(turns: &[(String, String)], idx: usize) -> Option<usize> {
    (0..=idx).rev().find(|&i| turns[i].0 == "user")
}

fn nearest_assistant_after(turns: &[(String, String)], from: usize) -> Option<usize> {
    (from..turns.len()).find(|&i| turns[i].0 == "assistant")
}

fn compress_line(s: &str, max: usize) -> String {
    let one = s
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    truncate_chars(&one, max)
}

fn compress_assistant(s: &str, max: usize) -> String {
    // Prefer first non-tool-line prose; skip giant tool dumps.
    let mut lines: Vec<&str> = Vec::new();
    for line in s.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if looks_like_tool_invocation(t) {
            continue;
        }
        if t.starts_with("```") {
            break;
        }
        lines.push(t);
        if lines.join(" ").chars().count() >= max {
            break;
        }
    }
    if lines.is_empty() {
        // Fall back to tool line names only.
        let tools = tools_mentioned(s);
        if !tools.is_empty() {
            return format!("used {}", tools.join(", "));
        }
        return truncate_chars(&compress_line(s, max), max);
    }
    truncate_chars(&lines.join(" "), max)
}

fn looks_like_tool_invocation(line: &str) -> bool {
    let upper = line.to_ascii_uppercase();
    crate::commands::tool_registry::TOOLS
        .iter()
        .any(|t| {
            upper.starts_with(&format!("{}:", t.name))
                || upper.starts_with(&format!("{} ", t.name))
        })
}

fn tools_mentioned(s: &str) -> Vec<String> {
    let upper = s.to_ascii_uppercase();
    let mut out = Vec::new();
    for t in crate::commands::tool_registry::TOOLS {
        let needle = format!("{}:", t.name);
        if upper.contains(&needle) && !out.iter().any(|x| x == t.name) {
            out.push(t.name.to_string());
        }
        if out.len() >= 6 {
            break;
        }
    }
    out
}

fn truncate_chars(s: &str, max: usize) -> String {
    let n = s.chars().count();
    if n <= max {
        return s.to_string();
    }
    let mut t: String = s.chars().take(max.saturating_sub(1)).collect();
    t.push('…');
    t
}

fn collect_snippets(text: &str, q_lower: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        let l = line.trim();
        if l.is_empty() {
            continue;
        }
        if l.to_lowercase().contains(q_lower) {
            let s: String = l.chars().take(SNIPPET_CHARS).collect();
            out.push(s);
            if out.len() >= MAX_SNIPPETS_PER_FILE {
                break;
            }
        }
    }
    out
}

/// Drop timeout / meta-lesson boilerplate from memory entries (Hermes hygiene).
pub fn looks_like_memory_pollution(entry: &str) -> bool {
    let n = entry.to_lowercase();
    let t = n.trim();
    // Bare section headers / empty scaffolding from failed lesson extraction
    if t == "learned"
        || t == "- learned"
        || t == "tools that worked vs. tools that failed"
        || t == "correct ids or endpoints discovered"
        || t == "user corrections about how things should work"
        || t == "mistakes to avoid in future"
        || t == "no lessons were found from this conversation."
    {
        return true;
    }
    // Compacted / tool-loop transcript dumps that leaked into MEMORY_APPEND
    if t.starts_with("[user]:")
        || t.starts_with("[assistant]:")
        || t.starts_with("run_cmd (")
        || t.starts_with("perplexity_search:")
        || t.starts_with("brave_search:")
        || t.starts_with("fetch_url:")
        || t.starts_with("redmine_api:")
        || t.starts_with("**note:**")
        || t.starts_with("note: in the last section")
    {
        return true;
    }
    n.contains("turn timed out")
        || n.contains("wall-clock budget")
        || n.contains("agentroutertimeout")
        || n.contains("if this keeps happening, try a narrower")
        || n.contains("if no lessons, write")
        || n.contains("output only these two sections")
        || n.contains("bullet points of important lessons")
        || n.contains("tools that worked vs")
        || n.contains("mistakes to avoid")
        || n.contains("correct ids or endpoints")
        || n.contains("user corrections about how things should work")
        || n.contains("compact this conversation")
        || n.contains("ms_untrusted_begin")
        || n.contains("<<<ms_untrusted")
        || n.contains("[partial progress:")
        || n.contains("agent router turn timeout")
        || n.contains("the run was stopped so the channel")
        || n.contains("agent tool run:")
        || n.contains("the user’s current question is")
        || n.contains("the user's current question is")
        || n.contains("[continue for next question]")
        || n.contains("last assistant text:")
        || n.contains("rule applies to both previous")
        || (n.contains("*cursor agent") && n.len() < 80)
        || (n.starts_with("*redmine") && n.len() < 120)
}

/// Filter memory markdown (bullet lines) for prompt injection — drop pollution in-memory.
pub fn filter_memory_markdown_for_prompt(content: &str) -> String {
    content
        .lines()
        .filter_map(|line| {
            let raw = line.trim();
            if raw.is_empty() {
                return None;
            }
            let entry = raw.trim_start_matches('-').trim();
            if entry.is_empty() || looks_like_memory_pollution(entry) {
                return None;
            }
            Some(line.to_string())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// One-shot scrub of existing memory*.md files under agents/.
pub fn scrub_polluted_memory_files() -> (usize, usize) {
    let dir = Config::agents_dir();
    let mut files = 0usize;
    let mut removed = 0usize;
    let Ok(rd) = fs::read_dir(&dir) else {
        return (0, 0);
    };
    for ent in rd.filter_map(Result::ok) {
        let path = ent.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        if !(name == "memory.md" || name.starts_with("memory-discord-") || name == "memory-main.md")
        {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let before: Vec<String> = text
            .lines()
            .map(|l| l.trim().trim_start_matches('-').trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();
        let after: Vec<String> = before
            .iter()
            .filter(|e| !looks_like_memory_pollution(e))
            .cloned()
            .collect();
        if after.len() == before.len() {
            continue;
        }
        files += 1;
        removed += before.len() - after.len();
        let body: String = after.iter().map(|e| format!("- {}\n", e)).collect();
        let _ = fs::write(&path, body);
    }
    (files, removed)
}

/// Cap memory block size for the system prompt (bytes of markdown after filter).
pub const MEMORY_PROMPT_MAX_CHARS: usize = 2_500;

pub fn truncate_memory_for_prompt(filtered: &str) -> String {
    if filtered.len() <= MEMORY_PROMPT_MAX_CHARS {
        return filtered.to_string();
    }
    // Keep the tail (newest lessons usually appended).
    let start = filtered.len().saturating_sub(MEMORY_PROMPT_MAX_CHARS);
    let slice = &filtered[start..];
    let cut = slice.find('\n').map(|i| i + 1).unwrap_or(0);
    format!("…\n{}", &slice[cut..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_pollution() {
        assert!(looks_like_memory_pollution(
            "Turn timed out — The full agent run exceeded its wall-clock budget (300s)"
        ));
        assert!(!looks_like_memory_pollution(
            "Prefer Open-Meteo for El Masnou weather"
        ));
        assert!(looks_like_memory_pollution(
            "Compact this conversation:\n[user]: hi"
        ));
        assert!(looks_like_memory_pollution(
            "Tools that worked vs. tools that failed"
        ));
        assert!(looks_like_memory_pollution(
            "*Agent tool run: 2 step(s), 0 with errors,  103354 ms total; tools: AGENT, DONE.*"
        ));
        assert!(looks_like_memory_pollution(
            "If this keeps happening, try a narrower question, a faster model, or widen the matching `agentRouterTurnTimeoutSecs*` value."
        ));
        assert!(looks_like_memory_pollution("[user]: What time is it?"));
        assert!(looks_like_memory_pollution(
            "RUN_CMD (ok): cd ~/.mac-stats && git add -A && git status --short | head -50"
        ));
        assert!(looks_like_memory_pollution(
            "**Note:** In the last section, the “DROP” rule applies to both previous and current questions."
        ));
        let filtered = filter_memory_markdown_for_prompt(
            "- Prefer Open-Meteo for El Masnou weather\n- Tools that worked vs. tools that failed\n- Learned\n- [user]: What version are you?\n",
        );
        assert!(filtered.to_lowercase().contains("open-meteo"), "{filtered}");
        assert!(!filtered.to_lowercase().contains("tools that worked"), "{filtered}");
        assert!(!filtered.to_lowercase().contains("[user]"), "{filtered}");
    }

    #[test]
    fn extractive_summary_focuses_on_query() {
        let md = r#"## User

What time is it?

## Assistant

It's Monday noon.

## User

What version are you?

## Assistant

I'm mac-stats v0.1.95.

## User

Weather in El Masnou

## Assistant

BRAVE_SEARCH: El Masnou weather
Sunny, 24C.
"#;
        let s = extractive_summary(md, "version");
        assert!(s.to_lowercase().contains("version"), "{s}");
        assert!(s.contains("0.1.95") || s.contains("mac-stats"), "{s}");
        assert!(!s.to_lowercase().contains("el masnou"), "{s}");
    }

    #[test]
    fn file_meta_parses_timestamp() {
        let m = file_meta_line(
            "session-memory-1467450744864243725-20260720-173305-what-time-is-it_.md",
        );
        assert!(m.contains("2026-07-20"), "{m}");
        assert!(m.contains("17:33:05"), "{m}");
    }

    #[test]
    fn current_session_filename_matcher_hermes_exclude() {
        let name = "session-memory-1467450744864243725-20260720-173305-what-time-is-it_.md";
        assert!(crate::session_memory::session_filename_matches_id(
            name,
            1467450744864243725
        ));
        assert!(!crate::session_memory::session_filename_matches_id(name, 99));
        let legacy = "session-memory-old-topic-88-20260321-090000.md";
        assert!(crate::session_memory::session_filename_matches_id(legacy, 88));
    }
}
