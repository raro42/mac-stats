//! Task files under ~/.mac-stats/task/ with naming:
//! task-<date-time>-<open|wip|finished|unsuccessful>.md
//! Topic and id are stored in-file (## Topic:, ## Id:) for listing and resolution.

pub mod cli;
pub mod review;
pub mod runner;

use crate::config::Config;
use std::path::{Path, PathBuf};
use std::fs;
use std::time::SystemTime;
use tracing::info;

const VALID_STATUSES: &[&str] = &["open", "wip", "finished", "unsuccessful", "paused"];

fn is_valid_status(s: &str) -> bool {
    VALID_STATUSES.contains(&s)
}

/// Build path for a task file under Config::task_dir(). Naming: task-<datetime>-<status>.md
pub fn task_path(datetime: &str, status: &str) -> PathBuf {
    let filename = format!("task-{}-{}.md", datetime, status);
    Config::task_dir().join(filename)
}

/// Parse status from filename. Stem format: task-<datetime>-<status>, last segment is status.
pub fn status_from_path(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    let stem = name.strip_suffix(".md")?;
    if !stem.starts_with("task-") {
        return None;
    }
    let parts: Vec<&str> = stem.split('-').collect();
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

/// If a task file already exists with the same topic (slug) and id, return its path.
/// Reads ## Topic: and ## Id: from each file; supports legacy filenames (task-topic-id-datetime-status) via id_from_path fallback.
fn existing_task_with_topic_id(topic_slug: &str, id: &str) -> Option<PathBuf> {
    let task_base = Config::task_dir();
    let entries = fs::read_dir(&task_base).ok()?;
    for entry in entries.flatten() {
        let p = entry.path();
        if !p.is_file() || status_from_path(&p).is_none() {
            continue;
        }
        if let Ok(Some(ref file_topic)) = get_topic_from_file(&p) {
            if slug(file_topic) != topic_slug {
                continue;
            }
        } else {
            continue;
        }
        if let Ok(Some(ref file_id)) = get_id_from_file(&p) {
            if file_id == id {
                return Some(p);
            }
        }
    }
    None
}

/// Create a new task file with status "open". Returns the path.
/// If a task with the same topic and id already exists, returns an error (avoids duplicate task explosion).
/// Topic and id are sanitized for safe filenames; topic/id that look like existing task filenames are rejected.
pub fn create_task(
    topic: &str,
    id: &str,
    initial_content: &str,
    assigned_to: Option<&str>,
) -> Result<PathBuf, String> {
    let _ = Config::ensure_task_directory();
    if topic.contains(".md") || (topic.starts_with("task-") && topic.matches('-').count() >= 4) {
        return Err(
            "TASK_CREATE topic looks like an existing task filename. Use TASK_APPEND: <filename or id> <content> to add to that task, or use short topic and id (e.g. TASK_CREATE: research 1 <content>).".to_string()
        );
    }
    let topic_slug = slug(topic);
    let id_safe = sanitize_id(id);
    if let Some(existing) = existing_task_with_topic_id(&topic_slug, &id_safe) {
        return Err(format!(
            "A task with this topic and id already exists: {:?}. Use TASK_APPEND or TASK_STATUS to update it.",
            existing
        ));
    }
    let datetime = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let path = task_path(&datetime, "open");
    let agent = assigned_to.unwrap_or("default").trim();
    let header = format!(
        "{} {}\n{} {}\n{} {}\n\n",
        ASSIGNED_HEADER,
        if agent.is_empty() { "default" } else { agent },
        TOPIC_HEADER,
        topic.trim(),
        ID_HEADER,
        id_safe
    );
    let content = format!("{}{}", header, initial_content.trim());
    fs::write(&path, content).map_err(|e| format!("Write task file: {}", e))?;
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

/// Append a conversation turn to the task file (## Conversation <timestamp> with User and Assistant).
/// Use to log the full exchange so the task file is the single record of what was asked and answered.
pub fn append_conversation_block(path: &Path, user_message: &str, assistant_reply: &str) -> Result<(), String> {
    let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let block = format!(
        "\n\n## Conversation {}\n\n**User:**\n{}\n\n**Assistant:**\n{}\n",
        ts,
        user_message.trim(),
        assistant_reply.trim()
    );
    append_to_task(path, &block)
}

/// Read full task file content.
pub fn read_task(path: &Path) -> Result<String, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Read task file: {}", e))?;
    info!("Task: read {:?} ({} chars)", path, content.len());
    Ok(content)
}

/// In-file header for assignee. Line format: "## Assigned: agent_id"
const ASSIGNED_HEADER: &str = "## Assigned:";
/// In-file header for topic (task-[date]-[status].md does not contain topic in filename).
const TOPIC_HEADER: &str = "## Topic:";
/// In-file header for id (task-[date]-[status].md does not contain id in filename).
const ID_HEADER: &str = "## Id:";

/// Get assignee from task file (first line matching ## Assigned: ...). Default "default".
pub fn get_assignee(path: &Path) -> Result<String, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Read task file: {}", e))?;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with(ASSIGNED_HEADER) {
            let agent = line[ASSIGNED_HEADER.len()..].trim();
            return Ok(if agent.is_empty() {
                "default".to_string()
            } else {
                agent.to_string()
            });
        }
    }
    Ok("default".to_string())
}

/// Get topic from task file (## Topic: ...). None if missing (e.g. legacy file).
pub fn get_topic_from_file(path: &Path) -> Result<Option<String>, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Read task file: {}", e))?;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with(TOPIC_HEADER) {
            let t = line[TOPIC_HEADER.len()..].trim();
            return Ok(Some(if t.is_empty() { "task".to_string() } else { t.to_string() }));
        }
    }
    Ok(None)
}

/// Get id from task file (## Id: ...). None if missing (e.g. legacy file).
pub fn get_id_from_file(path: &Path) -> Result<Option<String>, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Read task file: {}", e))?;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with(ID_HEADER) {
            let id = line[ID_HEADER.len()..].trim();
            return Ok(Some(if id.is_empty() { "1".to_string() } else { id.to_string() }));
        }
    }
    Ok(None)
}

/// Set assignee in task file (add or replace ## Assigned: line).
pub fn set_assignee(path: &Path, agent_id: &str) -> Result<(), String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Read task file: {}", e))?;
    let new_line = format!("{} {}\n", ASSIGNED_HEADER, agent_id.trim());
    let mut found = false;
    let lines: Vec<&str> = content.lines().collect();
    let mut out = String::new();
    for line in &lines {
        if line.trim().starts_with(ASSIGNED_HEADER) {
            out.push_str(&new_line);
            found = true;
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    if !found {
        out = format!("{}{}\n{}", new_line, if out.is_empty() { "" } else { "\n" }, out);
    }
    fs::write(path, out.trim_end()).map_err(|e| format!("Write task file: {}", e))?;
    info!("Task: assignee set to {} for {:?}", agent_id, path);
    Ok(())
}

/// Result of showing a task: status, assignee, and full content.
pub fn show_task_content(path: &Path) -> Result<(String, String, String), String> {
    let status = status_from_path(path).unwrap_or_else(|| "?".to_string());
    let assignee = get_assignee(path).unwrap_or_else(|_| "default".to_string());
    let content = read_task(path)?;
    Ok((status, assignee, content))
}

/// In-file line for pause deadline. Format: "## Paused until: 2025-02-10T09:00:00"
const PAUSED_UNTIL_HEADER: &str = "## Paused until:";

/// Get paused-until datetime from task file (None if not paused or no until).
pub fn get_paused_until(path: &Path) -> Result<Option<String>, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Read task file: {}", e))?;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with(PAUSED_UNTIL_HEADER) {
            let s = line[PAUSED_UNTIL_HEADER.len()..].trim();
            return Ok(if s.is_empty() { None } else { Some(s.to_string()) });
        }
    }
    Ok(None)
}

/// In-file line for dependencies. Format: "## Depends: id1, id2"
const DEPENDS_HEADER: &str = "## Depends:";

/// Get dependency task ids from file (empty if none).
pub fn get_depends_on(path: &Path) -> Result<Vec<String>, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Read task file: {}", e))?;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with(DEPENDS_HEADER) {
            let rest = line[DEPENDS_HEADER.len()..].trim();
            let ids: Vec<String> = rest.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            return Ok(ids);
        }
    }
    Ok(Vec::new())
}

/// In-file line for sub-tasks. Format: "## Sub-tasks: id1, id2"
const SUB_TASKS_HEADER: &str = "## Sub-tasks:";

/// Get sub-task ids from file (empty if none).
pub fn get_sub_tasks(path: &Path) -> Result<Vec<String>, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Read task file: {}", e))?;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with(SUB_TASKS_HEADER) {
            let rest = line[SUB_TASKS_HEADER.len()..].trim();
            let ids: Vec<String> = rest.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            return Ok(ids);
        }
    }
    Ok(Vec::new())
}

/// True if all sub-tasks are finished or unsuccessful.
pub fn all_sub_tasks_closed(path: &Path) -> Result<bool, String> {
    let ids = get_sub_tasks(path)?;
    if ids.is_empty() {
        return Ok(true);
    }
    for id in &ids {
        let sub_path = resolve_task_path(id)?;
        let status = status_from_path(&sub_path).unwrap_or_default();
        if status != "finished" && status != "unsuccessful" {
            return Ok(false);
        }
    }
    Ok(true)
}

/// True if all dependency tasks are finished or unsuccessful (task is ready to run).
pub fn is_ready(path: &Path) -> Result<bool, String> {
    let deps = get_depends_on(path)?;
    if deps.is_empty() {
        return Ok(true);
    }
    let task_base = Config::task_dir();
    for id in &deps {
        let entries = fs::read_dir(&task_base).map_err(|e| format!("Read task dir: {}", e))?;
        let mut found = false;
        for entry in entries {
            let p = entry.map_err(|e| format!("Read dir: {}", e))?.path();
            if !p.is_file() {
                continue;
            }
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !name.contains(id.as_str()) {
                continue;
            }
            found = true;
            let status = status_from_path(&p).unwrap_or_default();
            if status != "finished" && status != "unsuccessful" {
                return Ok(false);
            }
            break;
        }
        if !found {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Set or clear paused-until line in task file (add/replace/remove).
pub fn set_paused_until(path: &Path, until_iso: Option<&str>) -> Result<(), String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Read task file: {}", e))?;
    let mut out = String::new();
    for line in content.lines() {
        if line.trim().starts_with(PAUSED_UNTIL_HEADER) {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    if let Some(until) = until_iso {
        out = format!("{} {}\n\n{}", PAUSED_UNTIL_HEADER, until.trim(), out.trim_start());
    }
    fs::write(path, out.trim_end()).map_err(|e| format!("Write task file: {}", e))?;
    Ok(())
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

/// Filename-safe id: strip quotes, slashes, newlines; keep alphanumeric, dash, underscore; limit length.
fn sanitize_id(id: &str) -> String {
    let s: String = id
        .chars()
        .take(80)
        .filter(|c| !matches!(c, '"' | '\'' | '/' | '\\' | '\n' | '\r'))
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    let s = s.trim().trim_matches('_');
    if s.is_empty() {
        "1".to_string()
    } else {
        s.to_string()
    }
}

/// Id for a task: from in-file ## Id: first; for legacy filenames (task-topic-id-datetime-status) falls back to 4th-from-last segment.
pub fn id_from_path(path: &Path) -> Option<String> {
    if let Ok(Some(id)) = get_id_from_file(path) {
        return Some(id);
    }
    let stem = path.file_stem()?.to_str()?;
    let parts: Vec<&str> = stem.split('-').collect();
    if parts.len() >= 6 {
        Some(parts[parts.len() - 4].to_string())
    } else {
        None
    }
}

/// Task file name only (e.g. task-20260222-140215-open.md) for consistent display and references.
pub fn task_file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("task.md")
        .to_string()
}

/// Resolve path_or_id to a PathBuf under task dir.
/// Accepts: full path (with / or ~), task file name (e.g. task-research-1-20250222-120000-open.md or without .md), or short id (e.g. 1).
/// Full filename is matched first; then short id by exact id segment; then filename contains.
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
    let entries = fs::read_dir(&task_base).map_err(|e| format!("Read task dir: {}", e))?;
    let all_task_files: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .filter(|p| status_from_path(p).is_some())
        .collect();
    // 0) Exact task filename match (model often sends full name e.g. task-research-1-20260222-140215-open.md)
    let by_filename = all_task_files.iter().find(|p| {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        name == path_or_id
            || stem == path_or_id
            || (!path_or_id.ends_with(".md") && name == format!("{}.md", path_or_id))
    });
    if let Some(p) = by_filename {
        return Ok(p.clone());
    }
    // 1) Match by id (from ## Id: in file or legacy filename)
    let by_id: Vec<PathBuf> = all_task_files
        .iter()
        .filter(|p| id_from_path(p).as_deref() == Some(path_or_id))
        .cloned()
        .collect();
    let candidates = if !by_id.is_empty() {
        by_id
    } else {
        // 2) Match by topic (from ## Topic: in file; compare slug or raw)
        let by_topic: Vec<PathBuf> = all_task_files
            .iter()
            .filter(|p| {
                get_topic_from_file(p)
                    .ok()
                    .flatten()
                    .map(|t| slug(&t) == path_or_id || t == path_or_id)
                    .unwrap_or(false)
            })
            .cloned()
            .collect();
        if !by_topic.is_empty() {
            by_topic
        } else {
            // 3) Fallback: filename contains path_or_id
            all_task_files
                .into_iter()
                .filter(|p| {
                    p.file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.contains(path_or_id))
                        .unwrap_or(false)
                })
                .collect()
        }
    };
    if candidates.is_empty() {
        return Err(format!(
            "No task file found for id '{}'. Use task file name or full path under ~/.mac-stats/task.",
            path_or_id
        ));
    }
    if candidates.len() > 1 {
        let order = |s: Option<String>| match s.as_deref() {
            Some("open") => 0,
            Some("wip") => 1,
            _ => 2,
        };
        let mut sorted = candidates;
        sorted.sort_by(|a, b| order(status_from_path(a)).cmp(&order(status_from_path(b))));
        return Ok(sorted.into_iter().next().unwrap());
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
        .or_else(|| stem.strip_suffix("-unsuccessful"))
        .unwrap_or(stem);
    for status in ["open", "wip", "finished", "unsuccessful", "paused"] {
        let p = parent.join(format!("{}-{}.md", base, status));
        if p.exists() {
            return Some(p);
        }
    }
    None
}

/// All status suffixes for a task (same base name, different status file).
const STATUS_SUFFIXES: &[&str] = &["open", "wip", "finished", "unsuccessful", "paused"];

/// Delete all files for a task (open, wip, finished, unsuccessful). Returns number of files removed.
pub fn delete_task(path_or_id: &str) -> Result<usize, String> {
    let path = resolve_task_path(path_or_id)?;
    let parent = path.parent().ok_or("No parent dir")?;
    let stem = path.file_stem().and_then(|n| n.to_str()).ok_or("Invalid filename")?;
    let base = stem
        .strip_suffix("-open")
        .or_else(|| stem.strip_suffix("-wip"))
        .or_else(|| stem.strip_suffix("-finished"))
        .or_else(|| stem.strip_suffix("-unsuccessful"))
        .unwrap_or(stem);
    let mut removed = 0;
    for status in STATUS_SUFFIXES {
        let p = parent.join(format!("{}-{}.md", base, status));
        if p.exists() {
            fs::remove_file(&p).map_err(|e| format!("Remove {:?}: {}", p, e))?;
            removed += 1;
            info!("Task: deleted {:?}", p);
        }
    }
    if removed == 0 {
        return Err("No task files found to delete.".to_string());
    }
    Ok(removed)
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
        let assignee = get_assignee(&path).unwrap_or_else(|_| "default".to_string());
        let line = format!("{} (assigned: {})", name, assignee);
        if status == "open" {
            open.push(line);
        } else {
            wip.push(line);
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

/// Count task files by status. Returns (open, wip, paused, finished, unsuccessful).
pub fn count_tasks_by_status() -> Result<(usize, usize, usize, usize, usize), String> {
    let list = list_all_tasks()?;
    let mut open = 0;
    let mut wip = 0;
    let mut paused = 0;
    let mut finished = 0;
    let mut unsuccessful = 0;
    for (_, status, _) in list {
        match status.as_str() {
            "open" => open += 1,
            "wip" => wip += 1,
            "paused" => paused += 1,
            "finished" => finished += 1,
            "unsuccessful" => unsuccessful += 1,
            _ => {}
        }
    }
    Ok((open, wip, paused, finished, unsuccessful))
}

/// List all task files (any status) with modification time.
/// Returns (path, status, mtime). Used for "all tasks" view.
pub fn list_all_tasks() -> Result<Vec<(PathBuf, String, SystemTime)>, String> {
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
            Some(s) => s,
            _ => continue,
        };
        let mtime = fs::metadata(&path)
            .and_then(|m| m.modified())
            .unwrap_or_else(|_| SystemTime::UNIX_EPOCH);
        out.push((path, status, mtime));
    }
    Ok(out)
}

/// Format all tasks grouped by status: Open, WIP, Finished, Unsuccessful.
pub fn format_list_all_tasks() -> Result<String, String> {
    let list = list_all_tasks()?;
    if list.is_empty() {
        return Ok("No tasks. Use TASK_CREATE to create a new task.".to_string());
    }
    let order = |s: &str| match s {
        "open" => 0,
        "wip" => 1,
        "finished" => 2,
        "unsuccessful" => 3,
        "paused" => 4,
        _ => 5,
    };
    let mut by_status: std::collections::BTreeMap<u8, Vec<String>> = std::collections::BTreeMap::new();
    for (path, status, _mtime) in list {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();
        let assignee = get_assignee(&path).unwrap_or_else(|_| "default".to_string());
        let line = format!("{} (assigned: {})", name, assignee);
        by_status
            .entry(order(&status))
            .or_default()
            .push(line);
    }
    let headers = ["Open", "WIP", "Finished", "Unsuccessful", "Paused"];
    let keys = [0u8, 1, 2, 3, 4];
    let mut out = String::new();
    for (i, &k) in keys.iter().enumerate() {
        if let Some(names) = by_status.get(&k) {
            if !names.is_empty() {
                if !out.is_empty() {
                    out.push_str("\n\n");
                }
                out.push_str(headers[i]);
                out.push_str(" tasks:\n");
                for n in names {
                    out.push_str("- ");
                    out.push_str(n);
                    out.push('\n');
                }
            }
        }
    }
    Ok(out.trim_end().to_string())
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
            status_from_path(Path::new("task-20260222-140215-open.md")),
            Some("open".to_string())
        );
        assert_eq!(
            status_from_path(Path::new("/tmp/task-20250208-100000-finished.md")),
            Some("finished".to_string())
        );
        assert_eq!(status_from_path(Path::new("other.md")), None);
    }

    #[test]
    fn test_slug() {
        assert_eq!(slug("Hello World"), "hello-world");
        assert!(!slug("x").is_empty());
    }

    #[test]
    fn test_id_from_path_legacy_filename() {
        // Legacy format (task-topic-id-datetime-status): id parsed from filename when file doesn't exist
        assert_eq!(
            id_from_path(Path::new("task-research-1-20250222-120000-open.md")),
            Some("1".to_string())
        );
        assert_eq!(
            id_from_path(Path::new("/tmp/task-my-topic-15min-20250222-120000-wip.md")),
            Some("15min".to_string())
        );
    }

    #[test]
    fn test_id_from_path_no_file() {
        assert_eq!(id_from_path(Path::new("other.md")), None);
        assert_eq!(id_from_path(Path::new("task-20260222-140215-open.md")), None);
    }

    #[test]
    fn test_task_file_name() {
        assert_eq!(
            task_file_name(Path::new("/foo/task-20260222-140215-open.md")),
            "task-20260222-140215-open.md"
        );
    }
}
