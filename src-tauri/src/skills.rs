//! Skills: Markdown overlays in `~/.mac-stats/agents/skills/`.
//!
//! Supported layouts:
//! - Legacy flat: `skill-<number>-<topic>.md` (e.g. `skill-1-summarize.md`)
//! - Hermes folder: `<topic>/SKILL.md` (e.g. `changelog-weekly-review/SKILL.md`)
//!
//! Load and parse results are written to the app log (`~/.mac-stats/debug.log` when verbosity >= -vv).

use crate::config::Config;
use serde::Serialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// One skill: number and topic from filename or folder, content from file.
#[derive(Debug, Clone)]
pub struct Skill {
    pub number: u32,
    pub topic: String,
    pub content: String,
    /// Absolute path of the skill file (`skill-N-topic.md` or `topic/SKILL.md`).
    pub path: PathBuf,
}

/// Skill summary for UI (Settings → Skills): number, topic, and file path.
#[derive(Debug, Clone, Serialize)]
pub struct SkillForUi {
    pub number: u32,
    pub topic: String,
    pub path: String,
}

/// Default skills created when the skills directory is empty (planned "two agent skills": summarize, code).
const DEFAULT_SKILLS: &[(u32, &str, &str)] = &[
    (
        1,
        "summarize",
        "You are a summarization assistant. Give a clear, concise summary of the user's text. Preserve key facts and tone.",
    ),
    (
        2,
        "code",
        "You are a code assistant. Help with code: explain, fix, refactor, or write snippets. Prefer short, correct answers.",
    ),
];

/// Ensure the skills directory exists and, if it has no skill files, create the two default skills (1-summarize, 2-code).
fn ensure_default_skills() {
    if let Err(e) = Config::ensure_skills_directory() {
        warn!("Skills: could not create directory: {}", e);
        return;
    }
    let dir = Config::skills_dir();
    let has_any = std::fs::read_dir(&dir)
        .ok()
        .map(|rd| {
            rd.filter_map(Result::ok).any(|e| {
                let p = e.path();
                if p.extension().is_some_and(|ext| ext == "md") {
                    return true;
                }
                p.is_dir() && p.join("SKILL.md").is_file()
            })
        })
        .unwrap_or(false);
    if has_any {
        return;
    }
    for (number, topic, content) in DEFAULT_SKILLS {
        let name = format!("skill-{}-{}.md", number, topic);
        let path = dir.join(&name);
        if path.exists() {
            continue;
        }
        if crate::config::write_text_atomic(&path, content.trim()).is_ok() {
            info!("Skills: created default {}", name);
        } else {
            warn!("Skills: failed to create {}", name);
        }
    }
}

/// Strip optional YAML frontmatter (`---` … `---`) used by Hermes-style SKILL.md files.
pub fn strip_yaml_frontmatter(content: &str) -> String {
    let trimmed = content.trim();
    let Some(rest) = trimmed.strip_prefix("---") else {
        return trimmed.to_string();
    };
    // Frontmatter must start at file start; require a newline after the opening fence.
    let rest = match rest.chars().next() {
        Some('\n') | Some('\r') => rest.trim_start_matches(['\r', '\n']),
        _ => return trimmed.to_string(),
    };
    let Some(end) = rest.find("\n---") else {
        return trimmed.to_string();
    };
    let after = rest[end + 4..].trim_start_matches(['\r', '\n']).trim();
    if after.is_empty() {
        trimmed.to_string()
    } else {
        after.to_string()
    }
}

/// Load all skills from a directory (flat `skill-N-topic.md` + `<topic>/SKILL.md`).
/// Used by [`load_skills`] and unit tests.
pub fn load_skills_from_dir(dir: &Path) -> Vec<Skill> {
    if !dir.is_dir() {
        info!("Skills: directory missing or empty, path={:?}", dir);
        return Vec::new();
    }

    let read_dir = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(e) => {
            warn!("Skills: could not read directory {:?}: {}", dir, e);
            return Vec::new();
        }
    };

    let mut skills = Vec::new();
    let mut seen_topics: HashSet<String> = HashSet::new();
    let mut skipped_name = 0;
    let mut skipped_empty = 0;
    let mut hermes_dirs: Vec<(String, PathBuf)> = Vec::new();

    for entry in read_dir.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            let skill_md = path.join("SKILL.md");
            if skill_md.is_file() {
                if let Some(topic) = path.file_name().and_then(|s| s.to_str()) {
                    let topic = topic.trim();
                    if !topic.is_empty() && !topic.starts_with('.') {
                        hermes_dirs.push((topic.to_string(), skill_md));
                    }
                }
            }
            continue;
        }
        if path.extension().map(|e| e != "md").unwrap_or(true) {
            continue;
        }
        let name = match path.file_stem().and_then(|s| s.to_str()) {
            Some(n) => n,
            None => {
                skipped_name += 1;
                warn!(
                    "Skills: skipping file (no valid stem): {:?}",
                    path.file_name().unwrap_or_default()
                );
                continue;
            }
        };
        let (number, topic) = match parse_skill_filename(name) {
            Some(p) => p,
            None => {
                skipped_name += 1;
                warn!(
                    "Skills: skipping file (invalid name format): {:?} — expected skill-<number>-<topic>.md or <topic>/SKILL.md",
                    path.file_name().unwrap_or_default()
                );
                continue;
            }
        };
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => strip_yaml_frontmatter(&c),
            Err(e) => {
                warn!("Skills: could not read file {:?}: {}", path, e);
                continue;
            }
        };
        if content.is_empty() {
            skipped_empty += 1;
            info!("Skills: skipping empty file {:?}", path);
            continue;
        }
        seen_topics.insert(topic.to_lowercase());
        skills.push(Skill {
            number,
            topic,
            content,
            path,
        });
    }

    hermes_dirs.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    let mut next_number = skills.iter().map(|s| s.number).max().unwrap_or(0).saturating_add(1);
    for (topic, path) in hermes_dirs {
        let topic_key = topic.to_lowercase();
        if seen_topics.contains(&topic_key) {
            info!(
                "Skills: skipping Hermes {:?} — topic already loaded from flat file",
                path
            );
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => strip_yaml_frontmatter(&c),
            Err(e) => {
                warn!("Skills: could not read file {:?}: {}", path, e);
                continue;
            }
        };
        if content.is_empty() {
            skipped_empty += 1;
            info!("Skills: skipping empty file {:?}", path);
            continue;
        }
        seen_topics.insert(topic_key);
        skills.push(Skill {
            number: next_number,
            topic,
            content,
            path,
        });
        next_number = next_number.saturating_add(1);
    }

    skills.sort_by_key(|s| s.number);

    if skills.is_empty() {
        info!("Skills: no valid skill files in {:?}", dir);
        if skipped_name > 0 || skipped_empty > 0 {
            info!(
                "Skills: skipped {} file(s) with invalid name format, {} empty file(s). See docs/016_skill_agent.md (path: ~/.mac-stats/agents/skills/, naming: skill-<number>-<topic>.md or <topic>/SKILL.md).",
                skipped_name, skipped_empty
            );
        }
    } else {
        let list: String = skills
            .iter()
            .map(|s| format!("{}-{}", s.number, s.topic))
            .collect::<Vec<_>>()
            .join(", ");
        info!("Skills: loaded {} from {:?}: {}", skills.len(), dir, list);
        if skipped_name > 0 || skipped_empty > 0 {
            info!(
                "Skills: also skipped {} file(s) (invalid name), {} empty.",
                skipped_name, skipped_empty
            );
        }
    }

    skills
}

/// Load all skills from ~/.mac-stats/agents/skills/.
/// Accepts flat `skill-<number>-<topic>.md` and Hermes `<topic>/SKILL.md`.
/// On error (unreadable file) log and skip that file. Results are logged (available list or failures).
/// If the directory is empty, the two default skills (summarize, code) are created first.
pub fn load_skills() -> Vec<Skill> {
    ensure_default_skills();
    load_skills_from_dir(&Config::skills_dir())
}

/// Progressive disclosure catalog (Hermes skills_list): name + one-line description only.
pub fn skills_catalog_text() -> String {
    let skills = load_skills();
    if skills.is_empty() {
        return "No skills installed under ~/.mac-stats/agents/skills/.".to_string();
    }
    let mut lines = vec![format!("**Skills catalog** ({}):", skills.len())];
    for s in &skills {
        let desc = s
            .content
            .lines()
            .map(|l| l.trim())
            .find(|l| !l.is_empty())
            .unwrap_or("(no description)")
            .chars()
            .take(120)
            .collect::<String>();
        lines.push(format!("- {}-{} — {}", s.number, s.topic, desc));
    }
    lines.push(
        "Load full body with SKILL_VIEW: <number|topic>. Run as side session with SKILL: <number|topic> [task]."
            .into(),
    );
    lines.join("\n")
}

/// Return full skill body for SKILL_VIEW (Hermes skill_view).
pub fn skill_view_text(selector: &str) -> String {
    let skills = load_skills();
    match find_skill_by_number_or_topic(&skills, selector.trim()) {
        Some(skill) => format!(
            "Skill {}-{} (full body):\n\n{}",
            skill.number, skill.topic, skill.content
        ),
        None => {
            let avail: Vec<String> = skills
                .iter()
                .map(|s| format!("{}-{}", s.number, s.topic))
                .collect();
            format!(
                "Unknown skill \"{}\". Available: {}",
                selector.trim(),
                avail.join(", ")
            )
        }
    }
}

/// Return loaded skills for the Settings UI (number, topic, path). Path is full filesystem path.
pub fn list_skills_for_ui() -> Vec<SkillForUi> {
    load_skills()
        .into_iter()
        .map(|s| SkillForUi {
            number: s.number,
            topic: s.topic,
            path: s.path.display().to_string(),
        })
        .collect()
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
    skills.iter().find(|s| {
        s.topic.to_lowercase() == lower || s.topic.to_lowercase().replace(' ', "-") == lower
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

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

    #[test]
    fn strip_frontmatter_removes_yaml() {
        let raw = "---\nname: changelog-weekly-review\ndescription: hygiene\n---\n\n# Body\n\nDo the work.\n";
        let body = strip_yaml_frontmatter(raw);
        assert!(body.starts_with("# Body"));
        assert!(!body.contains("name:"));
    }

    #[test]
    fn strip_frontmatter_leaves_plain() {
        assert_eq!(strip_yaml_frontmatter("plain skill"), "plain skill");
    }

    #[test]
    fn load_skills_from_dir_reads_hermes_folder() {
        let dir = std::env::temp_dir().join(format!(
            "mac-stats-skills-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("changelog-weekly-review")).unwrap();
        fs::write(
            dir.join("skill-1-summarize.md"),
            "You summarize text.",
        )
        .unwrap();
        fs::write(
            dir.join("changelog-weekly-review").join("SKILL.md"),
            "---\nname: changelog-weekly-review\ndescription: hygiene\n---\n\nClean CHANGELOG.md.\n",
        )
        .unwrap();

        let skills = load_skills_from_dir(&dir);
        let _ = fs::remove_dir_all(&dir);

        assert_eq!(skills.len(), 2);
        assert!(find_skill_by_number_or_topic(&skills, "1").is_some());
        let weekly = find_skill_by_number_or_topic(&skills, "changelog-weekly-review")
            .expect("Hermes skill must resolve by topic");
        assert_eq!(weekly.topic, "changelog-weekly-review");
        assert!(weekly.content.contains("Clean CHANGELOG.md"));
        assert!(!weekly.content.contains("name: changelog-weekly-review"));
    }
}
