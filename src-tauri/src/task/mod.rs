//! Task files under ~/.mac-stats/task/ with naming:
//! task-<topic>-<id>-<date-time>-<open|wip|finished|unsuccessful>.md

pub mod review;

use crate::config::Config;
use std::path::{Path, PathBuf};
use std::fs;
use std::time::SystemTime;
use tracing::info;

const VALID_STATUSES: &[&str] = &["open", "wip", "finished", "unsuccessful"];

fn is_valid_status(s: &str) -> bool {
    VALID_STATUSES.contains(&s)
}

/// Build path for a task file under Config::task_dir().
pub fn task_path(topic: &str, id: &str, datetime: &str, status: &str) -> PathBuf {
    let filename = format!("task-{}-{}-{}-{}.md", topic, id, datetime, status);
    Config::task_dir().join(filename)
}

/// Parse status from filename (segment before .md: open, wip, finished, or unsuccessful).
/// Returns None if the path does not match the task file naming convention.
pub fn status_from_path(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    let stem = name.strip_suffix(".md")?;
    if !stem.starts_with("task-") {
        return None;
    }
    let parts: Vec<&str> = stem.split('-').collect();
    // task-<topic>-<id>-<date>-<time>-<status>  => at least 6 parts (topic could have dashes)
    // So we need: task, topic..., id, date, time, status. Simplest: last part is status.
    let status = parts.last().copied()?;
    if is_valid_status(status) {
        Some(status.to_string())
    } else {
        None
    }
}

/// Rename task file to new status; return the new path.
pub fn set_task_status(path: &Path, new_status: &str) -> Result<PathBuf, String> {
    if !is_valid_status(new_status) {
        return Err(format!("Invalid status: {} (allowed: open, wip, finished, unsuccessful)", new_status));
    }
    let parent = path.parent().ok_or("No parent dir")?;
    let name = path.file_name().and_then(|n| n.to_str()).ok_or("Invalid filename")?;
    let stem = name.strip_suffix(".md").ok_or("Not a .md file")?;
    if !stem.starts_with("task-") {
        return Err("Not a task file (must start with task-)".to_string());
    }
    // Replace last segment (current status) with new status. Stem is task-X-Y-Z-datetime-STATUS.
    let parts: Vec<&str> = stem.split('-').collect();
    if parts.len() < 2 {
        return Err("Task filename has no status segment".to_string());
    }
    let base = parts[..parts.len() - 1].join("-");
    let new_name = format!("{}-{}.md", base, new_status);
    let new_path = parent.join(&new_name);
    fs::rename(path, &new_path).map_err(|e| format!("Rename failed: {}", e))?;
    info!("Task: status set to {} for {:?}", new_status, new_path);
    Ok(new_path)
}

/// Create a new task file with status "open". Returns the path.
pub fn create_task(topic: &str, id: &str, initial_content: &str) -> Result<PathBuf, String> {
    let _ = Config::ensure_task_directory();
    let topic_slug = slug(topic);
    let datetime = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let path = task_path(&topic_slug, id, &datetime, "open");
    fs::write(&path, initial_content.trim()).map_err(|e| format!("Write task file: {}", e))?;
    info!("Task: created {:?} (topic={}, id={})", path, topic_slug, id);
    Ok(path)
}

/// Append a feedback block to the task file (adds ## Feedback <timestamp> section).
pub fn append_to_task(path: &Path, block: &str) -> Result<(), String> {
    let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let appendix = format!("\n\n## Feedback {}\n\n{}\n", ts, block.trim());
    let mut content = fs::read_to_string(path).map_err(|e| format!("Read task file: {}", e))?;
    content.push_str(&appendix);
    fs::write(path, content).map_err(|e| format!("Write task file: {}", e))?;
    info!("Task: appended {} chars to {:?}", block.trim().len(), path);
    Ok(())
}

/// Read full task file content.
pub fn read_task(path: &Path) -> Result<String, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Read task file: {}", e))?;
    info!("Task: read {:?} ({} chars)", path, content.len());
    Ok(content)
}

/// Filename-safe slug from topic (alphanumeric, spaces to dashes, lowercase).
fn slug(topic: &str) -> String {
    let s: String = topic
        .chars()
        .take(60)
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { '_' })
        .collect();
    let s = s.trim().replace(' ', "-").trim_matches('-').to_lowercase();
    if s.is_empty() {
        "task".to_string()
    } else {
        s
    }
}

/// Resolve path_or_id to a PathBuf under task dir.
/// If it looks like a path (contains '/' or starts with '~'), expand and validate under task dir.
/// Otherwise treat as id: list task dir and find a single file whose name contains the id (prefer open/wip).
pub fn resolve_task_path(path_or_id: &str) -> Result<PathBuf, String> {
    let path_or_id = path_or_id.trim();
    let task_base = Config::task_dir();
    let task_base_canon = task_base.canonicalize().unwrap_or(task_base.clone());
    if path_or_id.contains('/') || path_or_id.starts_with('~') {
        let expanded = expand_tilde(path_or_id);
        let path = Path::new(&expanded);
        let canonical = path.canonicalize().map_err(|e| format!("Path not found: {}", e))?;
        if !canonical.starts_with(&task_base_canon) {
            return Err("Path must be under ~/.mac-stats/task".to_string());
        }
        return Ok(canonical);
    }
    // Resolve by id: list task dir, find files containing this id
    let entries = fs::read_dir(&task_base).map_err(|e| format!("Read task dir: {}", e))?;
    let mut candidates: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.contains(path_or_id))
                .unwrap_or(false)
        })
        .collect();
    if candidates.is_empty() {
        return Err(format!("No task file found for id '{}'. Use full path under ~/.mac-stats/task.", path_or_id));
    }
    if candidates.len() > 1 {
        // Prefer open, then wip
        let order = |s: Option<String>| match s.as_deref() {
            Some("open") => 0,
            Some("wip") => 1,
            _ => 2,
        };
        candidates.sort_by(|a, b| {
            order(status_from_path(a)).cmp(&order(status_from_path(b)))
        });
    }
    Ok(candidates.into_iter().next().unwrap())
}

/// After a rename (set_task_status), resolve the current path of the same task file.
pub fn find_current_path(previous_path: &Path) -> Option<PathBuf> {
    let parent = previous_path.parent()?;
    let stem = previous_path.file_stem()?.to_str()?;
    let base = stem
        .strip_suffix("-open")
        .or_else(|| stem.strip_suffix("-wip"))
        .or_else(|| stem.strip_suffix("-finished"))
        .unwrap_or(stem);
    for status in ["open", "wip", "finished", "unsuccessful"] {
        let p = parent.join(format!("{}-{}.md", base, status));
        if p.exists() {
            return Some(p);
        }
    }
    None
}

/// Format a human-readable list of open and wip tasks for Ollama/user (e.g. "Open tasks:\n- task-foo-1-...\nWIP tasks:\n- ...").
pub fn format_list_open_and_wip_tasks() -> Result<String, String> {
    let list = list_open_and_wip_tasks()?;
    if list.is_empty() {
        return Ok("No open or WIP tasks. Use TASK_CREATE to create a new task.".to_string());
    }
    let mut open = Vec::new();
    let mut wip = Vec::new();
    for (path, status, _mtime) in list {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();
        if status == "open" {
            open.push(name);
        } else {
            wip.push(name);
        }
    }
    let mut out = String::new();
    if !open.is_empty() {
        out.push_str("Open tasks:\n");
        for n in open {
            out.push_str("- ");
            out.push_str(&n);
            out.push('\n');
        }
    }
    if !wip.is_empty() {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str("WIP tasks:\n");
        for n in wip {
            out.push_str("- ");
            out.push_str(&n);
            out.push('\n');
        }
    }
    Ok(out.trim_end().to_string())
}

/// List all task files with status open or wip, with their modification time.
/// Returns (path, status, mtime). Used by the task review loop to find work and close stale WIPs.
pub fn list_open_and_wip_tasks() -> Result<Vec<(PathBuf, String, SystemTime)>, String> {
    let task_base = Config::task_dir();
    if !task_base.exists() {
        return Ok(Vec::new());
    }
    let entries = fs::read_dir(&task_base).map_err(|e| format!("Read task dir: {}", e))?;
    let mut out = Vec::new();
    for entry in entries {
        let path = entry.map_err(|e| format!("Read dir entry: {}", e))?.path();
        if !path.is_file() {
            continue;
        }
        let status = match status_from_path(&path) {
            Some(s) if s == "open" || s == "wip" => s,
            _ => continue,
        };
        let mtime = fs::metadata(&path)
            .and_then(|m| m.modified())
            .unwrap_or_else(|_| SystemTime::UNIX_EPOCH);
        out.push((path, status, mtime));
    }
    Ok(out)
}

fn expand_tilde(s: &str) -> String {
    if s.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{}{}", home, &s[1..]);
        }
    }
    if s == "~" {
        if let Ok(home) = std::env::var("HOME") {
            return home;
        }
    }
    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_from_path() {
        assert_eq!(
            status_from_path(Path::new("/tmp/task-foo-1-20250208-100000-open.md")),
            Some("open".to_string())
        );
        assert_eq!(
            status_from_path(Path::new("task-a-b-20250208-100000-finished.md")),
            Some("finished".to_string())
        );
        assert_eq!(status_from_path(Path::new("other.md")), None);
    }

    #[test]
    fn test_slug() {
        assert_eq!(slug("Hello World"), "hello-world");
        assert!(!slug("x").is_empty());
    }
}
