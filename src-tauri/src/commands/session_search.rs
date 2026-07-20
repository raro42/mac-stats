//! Hermes-style cross-session recall over `~/.mac-stats/session/*.md`.
//!
//! Lean port: substring search + ranked snippets (no SQLite FTS / summarizer LLM yet).

use crate::config::Config;
use std::fs;
use std::path::Path;

const MAX_FILES: usize = 5;
const MAX_SNIPPETS_PER_FILE: usize = 4;
const SNIPPET_CHARS: usize = 180;

/// `SESSION_SEARCH: <query>` — search persisted session markdown.
pub fn handle_session_search(arg: &str) -> String {
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
        let text = match fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        if !text.to_lowercase().contains(&q_lower) {
            continue;
        }
        let snippets = collect_snippets(&text, &q_lower);
        let score = snippets.len()
            + text
                .to_lowercase()
                .matches(&q_lower)
                .count()
                .min(20);
        let mtime = ent
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        hits.push(FileHit {
            name,
            path: path.display().to_string(),
            score,
            mtime,
            snippets,
        });
    }

    if hits.is_empty() {
        return format!("No past sessions matched {:?}.", query);
    }
    hits.sort_by(|a, b| b.score.cmp(&a.score).then(b.mtime.cmp(&a.mtime)));
    hits.truncate(MAX_FILES);

    let mut out = vec![format!(
        "**Session search** for {:?} — {} file(s):",
        query,
        hits.len()
    )];
    for (i, h) in hits.iter().enumerate() {
        out.push(format!(
            "\n{}. `{}` (score {}, {})",
            i + 1,
            h.name,
            h.score,
            Path::new(&h.path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
        ));
        for s in &h.snippets {
            out.push(format!("   - …{}…", s));
        }
    }
    out.push(
        "\nCite these if relevant; use READ of session files only via SESSION_SEARCH / Agent Ops — do not invent past decisions."
            .into(),
    );
    out.join("\n")
}

struct FileHit {
    name: String,
    path: String,
    score: usize,
    mtime: u64,
    snippets: Vec<String>,
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
    n.contains("turn timed out")
        || n.contains("wall-clock budget")
        || n.contains("agentroutertimeout")
        || n.contains("if no lessons, write")
        || n.contains("output only these two sections")
        || n.contains("bullet points of important lessons")
        || (n.contains("tools that worked vs") && n.contains("mistakes to avoid"))
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
    }
}
