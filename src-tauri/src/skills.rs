//! Skills: Markdown files in ~/.mac-stats/skills/skill-<number>-<topic>.md
//! Used as system-prompt overlays so different agents can respond differently.
//! Load and parse results are written to the app log (~/.mac-stats/debug.log when verbosity >= -vv).
//! Any future code that creates or modifies skill files should also log and consider notifying the user (e.g. status or Tauri event).

use crate::config::Config;
use tracing::{debug, info, warn};

/// One skill: number and topic from filename, content from file.
#[derive(Debug, Clone)]
pub struct Skill {
    pub number: u32,
    pub topic: String,
    pub content: String,
}

/// Load all skills from ~/.mac-stats/skills/. Files must match skill-<number>-<topic>.md.
/// On error (unreadable file) log and skip that file. Results are logged (available list or failures).
pub fn load_skills() -> Vec<Skill> {
    let dir = Config::skills_dir();
    if !dir.is_dir() {
        info!("Skills: directory missing or empty, path={:?}", dir);
        return Vec::new();
    }

    let mut skills = Vec::new();
    let read_dir = match std::fs::read_dir(&dir) {
        Ok(r) => r,
        Err(e) => {
            warn!("Skills: could not read directory {:?}: {}", dir, e);
            return skills;
        }
    };

    for entry in read_dir.filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().map(|e| e != "md").unwrap_or(true) {
            continue;
        }
        let name = match path.file_stem().and_then(|s| s.to_str()) {
            Some(n) => n,
            None => continue,
        };
        let (number, topic) = match parse_skill_filename(name) {
            Some(p) => p,
            None => continue,
        };
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c.trim().to_string(),
            Err(e) => {
                warn!("Skills: could not read file {:?}: {}", path, e);
                continue;
            }
        };
        if content.is_empty() {
            debug!("Skills: skipping empty file {:?}", path);
            continue;
        }
        skills.push(Skill {
            number,
            topic,
            content,
        });
    }

    skills.sort_by_key(|s| s.number);

    if skills.is_empty() {
        info!("Skills: no valid skill files in {:?}", dir);
    } else {
        let list: String = skills
            .iter()
            .map(|s| format!("{}-{}", s.number, s.topic))
            .collect::<Vec<_>>()
            .join(", ");
        info!(
            "Skills: loaded {} from {:?}: {}",
            skills.len(),
            dir,
            list
        );
    }

    skills
}

/// Parse "skill-123-topic-name" into (123, "topic-name").
fn parse_skill_filename(name: &str) -> Option<(u32, String)> {
    let rest = name.strip_prefix("skill-")?;
    let (num_str, topic) = rest.split_once('-')?;
    let number = num_str.parse::<u32>().ok()?;
    let topic = topic.trim().to_string();
    if topic.is_empty() {
        return None;
    }
    Some((number, topic))
}

/// Find a skill by number (e.g. 2) or by topic slug (e.g. "code"). Case-insensitive for topic.
pub fn find_skill_by_number_or_topic<'a>(skills: &'a [Skill], selector: &str) -> Option<&'a Skill> {
    let selector = selector.trim();
    if let Ok(n) = selector.parse::<u32>() {
        return skills.iter().find(|s| s.number == n);
    }
    let lower = selector.to_lowercase();
    skills
        .iter()
        .find(|s| s.topic.to_lowercase() == lower || s.topic.to_lowercase().replace(' ', "-") == lower)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_skill_filename_ok() {
        assert_eq!(
            parse_skill_filename("skill-1-summarize"),
            Some((1, "summarize".to_string()))
        );
        assert_eq!(
            parse_skill_filename("skill-2-code"),
            Some((2, "code".to_string()))
        );
    }

    #[test]
    fn parse_skill_filename_invalid() {
        assert!(parse_skill_filename("skill-1").is_none());
        assert!(parse_skill_filename("skill-x-foo").is_none());
        assert!(parse_skill_filename("other-1-foo").is_none());
    }
}
