//! Hermes-style agent-managed skills: create / edit / patch / delete under
//! `~/.mac-stats/agents/skills/skill-<n>-<topic>.md`.

use crate::config::Config;
use crate::skills::{find_skill_by_number_or_topic, load_skills};
use tracing::info;

const MAX_CONTENT: usize = 12_000;
const MAX_TOPIC: usize = 48;

fn sanitize_topic(raw: &str) -> Result<String, String> {
    let t = raw
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else if c.is_whitespace() || c == '/' {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|c| *c != '\0')
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if t.is_empty() || t.len() > MAX_TOPIC {
        return Err(format!(
            "Invalid topic (use 1–{} alphanumeric/hyphen chars).",
            MAX_TOPIC
        ));
    }
    if t == "summarize" || t == "code" {
        // Allow editing defaults, but creating duplicates of defaults is ok with new numbers.
    }
    Ok(t)
}

fn scan_skill_content(content: &str) -> Option<&'static str> {
    if crate::commands::session_search::looks_like_memory_pollution(content) {
        return Some("compaction/timeout boilerplate");
    }
    let lower = content.to_lowercase();
    for (needle, label) in [
        ("ignore previous instructions", "prompt injection"),
        ("ignore all instructions", "prompt injection"),
        ("system prompt override", "prompt injection"),
        ("disregard your instructions", "prompt injection"),
        ("rm -rf", "destructive shell"),
    ] {
        if lower.contains(needle) {
            return Some(label);
        }
    }
    None
}

fn next_skill_number() -> u32 {
    load_skills()
        .iter()
        .map(|s| s.number)
        .max()
        .unwrap_or(0)
        .saturating_add(1)
        .max(1)
}

fn skill_path(number: u32, topic: &str) -> std::path::PathBuf {
    Config::skills_dir().join(format!("skill-{}-{}.md", number, topic))
}

fn resolve_existing(selector: &str) -> Result<(u32, String, std::path::PathBuf), String> {
    let skills = load_skills();
    let skill = find_skill_by_number_or_topic(&skills, selector)
        .ok_or_else(|| format!("Unknown skill \"{}\". Use SKILLS_LIST first.", selector))?;
    let path = skill_path(skill.number, &skill.topic);
    Ok((skill.number, skill.topic.clone(), path))
}

/// `SKILL_MANAGE: create <topic> | <body>`
/// `SKILL_MANAGE: edit <id|topic> | <body>`
/// `SKILL_MANAGE: patch <id|topic> | <old> => <new>`
/// `SKILL_MANAGE: delete <id|topic>`
pub fn handle_skill_manage(arg: &str) -> String {
    let arg = arg.trim();
    if arg.is_empty() {
        return "Usage: SKILL_MANAGE: create <topic> | <content> | edit <id> | <content> | patch <id> | <old> => <new> | delete <id>".to_string();
    }
    let (action, rest) = match arg.split_once(|c: char| c.is_whitespace()) {
        Some((a, r)) => (a.to_lowercase(), r.trim()),
        None => (arg.to_lowercase(), ""),
    };
    match action.as_str() {
        "create" => create_skill(rest),
        "edit" => edit_skill(rest),
        "patch" => patch_skill(rest),
        "delete" => delete_skill(rest),
        _ => "Unknown SKILL_MANAGE action. Use create | edit | patch | delete.".to_string(),
    }
}

fn split_pipe(rest: &str) -> Option<(&str, &str)> {
    rest.split_once('|').map(|(a, b)| (a.trim(), b.trim()))
}

fn create_skill(rest: &str) -> String {
    let Some((topic_raw, content)) = split_pipe(rest) else {
        return "SKILL_MANAGE create: use `create <topic> | <skill body>`".to_string();
    };
    let topic = match sanitize_topic(topic_raw) {
        Ok(t) => t,
        Err(e) => return e,
    };
    if content.len() < 20 {
        return "SKILL_MANAGE create: content too short (need a real procedure).".to_string();
    }
    if content.len() > MAX_CONTENT {
        return format!("SKILL_MANAGE create: content exceeds {} chars.", MAX_CONTENT);
    }
    if let Some(threat) = scan_skill_content(content) {
        return format!("Blocked: skill content matched '{}'.", threat);
    }
    if let Err(e) = Config::ensure_skills_directory() {
        return format!("Cannot create skills dir: {}", e);
    }
    let n = next_skill_number();
    let path = skill_path(n, &topic);
    if path.exists() {
        return format!("Skill file already exists: {}", path.display());
    }
    match std::fs::write(&path, content.trim()) {
        Ok(()) => {
            info!("SKILL_MANAGE: created {}", path.display());
            format!(
                "Created skill {}-{} at {}.\nUse SKILL_VIEW: {} or SKILL: {} [task].",
                n,
                topic,
                path.display(),
                n,
                n
            )
        }
        Err(e) => format!("Failed to write skill: {}", e),
    }
}

fn edit_skill(rest: &str) -> String {
    let Some((selector, content)) = split_pipe(rest) else {
        return "SKILL_MANAGE edit: use `edit <id|topic> | <full body>`".to_string();
    };
    if content.len() < 20 || content.len() > MAX_CONTENT {
        return "SKILL_MANAGE edit: content length out of range.".to_string();
    }
    if let Some(threat) = scan_skill_content(content) {
        return format!("Blocked: skill content matched '{}'.", threat);
    }
    let (n, topic, path) = match resolve_existing(selector) {
        Ok(v) => v,
        Err(e) => return e,
    };
    match std::fs::write(&path, content.trim()) {
        Ok(()) => {
            info!("SKILL_MANAGE: edited {}", path.display());
            format!("Updated skill {}-{}.", n, topic)
        }
        Err(e) => format!("Failed to edit skill: {}", e),
    }
}

fn patch_skill(rest: &str) -> String {
    let Some((selector, patch)) = split_pipe(rest) else {
        return "SKILL_MANAGE patch: use `patch <id|topic> | <old> => <new>`".to_string();
    };
    let Some((old, new)) = patch
        .split_once("=>")
        .or_else(|| patch.split_once("->"))
        .map(|(a, b)| (a.trim(), b.trim()))
    else {
        return "SKILL_MANAGE patch: need `<old> => <new>`".to_string();
    };
    if old.is_empty() {
        return "SKILL_MANAGE patch: empty old substring.".to_string();
    }
    if let Some(threat) = scan_skill_content(new) {
        return format!("Blocked: patch new text matched '{}'.", threat);
    }
    let (n, topic, path) = match resolve_existing(selector) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let Ok(body) = std::fs::read_to_string(&path) else {
        return format!("Cannot read {}", path.display());
    };
    if !body.contains(old) {
        return format!("No match for {:?} in skill {}-{}.", old, n, topic);
    }
    let updated = body.replacen(old, new, 1);
    if updated.len() > MAX_CONTENT {
        return "Patch would exceed size limit.".to_string();
    }
    match std::fs::write(&path, updated) {
        Ok(()) => {
            info!("SKILL_MANAGE: patched {}", path.display());
            format!("Patched skill {}-{}.", n, topic)
        }
        Err(e) => format!("Failed to patch: {}", e),
    }
}

fn delete_skill(selector: &str) -> String {
    let (n, topic, path) = match resolve_existing(selector) {
        Ok(v) => v,
        Err(e) => return e,
    };
    // Protect built-in defaults from accidental wipe
    if (n == 1 && topic == "summarize") || (n == 2 && topic == "code") {
        return format!(
            "Refusing to delete built-in skill {}-{} (edit/patch instead).",
            n, topic
        );
    }
    match std::fs::remove_file(&path) {
        Ok(()) => {
            info!("SKILL_MANAGE: deleted {}", path.display());
            format!("Deleted skill {}-{}.", n, topic)
        }
        Err(e) => format!("Failed to delete: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topic_sanitize() {
        assert_eq!(sanitize_topic("HN Top Stories").unwrap(), "hn-top-stories");
        assert!(sanitize_topic("!!!").is_err());
    }
}
