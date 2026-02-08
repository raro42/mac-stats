//! Scheduler agent: runs tasks at scheduled times from ~/.mac-stats/schedules.json.
//!
//! Loads the file at startup and in a loop: sleeps until the next due time (or a short interval
//! to check for file changes), executes the task (via Ollama + agents or direct FETCH_URL/BRAVE_SEARCH),
//! and re-reads the file whenever it changes (mtime poll) or after each run.

use crate::config::Config;
use chrono::{DateTime, Local, TimeZone};
use cron::Schedule;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Max sleep between checks so we periodically reload the schedule file (seconds).
const MAX_SLEEP_SECS: u64 = 60;

/// How often to check if schedules.json changed (seconds). Enables reload whenever the file is modified.
const FILE_CHECK_INTERVAL_SECS: u64 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScheduleEntryRaw {
    #[serde(default)]
    id: Option<String>,
    cron: Option<String>,
    at: Option<String>,
    task: String,
    /// When set, the scheduler will send the task result to this Discord channel (DM or channel where user asked).
    #[serde(default)]
    reply_to_channel_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ScheduleEntry {
    pub id: Option<String>,
    pub cron: Option<Schedule>,
    pub at: Option<DateTime<Local>>,
    pub task: String,
    pub reply_to_channel_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SchedulesFile {
    schedules: Vec<ScheduleEntryRaw>,
}

/// Returns the modification time of the schedules file, if it exists.
fn schedules_file_mtime() -> Option<std::time::SystemTime> {
    let path = Config::schedules_file_path();
    std::fs::metadata(&path).ok().and_then(|m| m.modified().ok())
}

fn load_schedules() -> Vec<ScheduleEntry> {
    let _ = Config::ensure_schedules_directory();
    let path = Config::schedules_file_path();

    if !path.exists() {
        return Vec::new();
    }

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            warn!("Scheduler: failed to read schedules file {:?}: {}", path, e);
            return Vec::new();
        }
    };

    let file_data: SchedulesFile = match serde_json::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            warn!("Scheduler: failed to parse schedules file {:?}: {}", path, e);
            return Vec::new();
        }
    };

    let mut entries = Vec::new();
    for raw in file_data.schedules {
        if raw.task.is_empty() {
            warn!("Scheduler: skipping entry with empty task (id={:?})", raw.id);
            continue;
        }
        let has_cron = raw.cron.is_some();
        let has_at = raw.at.is_some();
        if has_cron == has_at {
            warn!(
                "Scheduler: entry must have exactly one of cron or at (id={:?})",
                raw.id
            );
            continue;
        }

        if let Some(ref cron_str) = raw.cron {
            match Schedule::from_str(cron_str) {
                Ok(schedule) => {
                    entries.push(ScheduleEntry {
                        id: raw.id.clone(),
                        cron: Some(schedule),
                        at: None,
                        task: raw.task.clone(),
                        reply_to_channel_id: raw.reply_to_channel_id.clone(),
                    });
                }
                Err(e) => {
                    warn!(
                        "Scheduler: invalid cron {:?} (id={:?}): {}",
                        cron_str, raw.id, e
                    );
                }
            }
        } else if let Some(ref at_str) = raw.at {
            // Parse as local datetime (ISO 8601 without Z = local)
            let at_dt = chrono::DateTime::parse_from_rfc3339(at_str)
                .map(|dt| dt.with_timezone(&Local))
                .or_else(|_| {
                    // Try without timezone as local
                    chrono::NaiveDateTime::parse_from_str(at_str, "%Y-%m-%dT%H:%M:%S")
                        .map(|n| Local.from_local_datetime(&n).single().unwrap_or(n.and_utc().with_timezone(&Local)))
                });
            match at_dt {
                Ok(dt) => {
                    entries.push(ScheduleEntry {
                        id: raw.id.clone(),
                        cron: None,
                        at: Some(dt),
                        task: raw.task.clone(),
                        reply_to_channel_id: raw.reply_to_channel_id.clone(),
                    });
                }
                Err(_) => {
                    warn!(
                        "Scheduler: invalid at datetime {:?} (id={:?})",
                        at_str, raw.id
                    );
                }
            }
        }
    }

    info!(
        "Scheduler: loaded {} entries from {:?}",
        entries.len(),
        path
    );
    entries
}

/// Compute the next run time for this entry (in local time). Returns None if one-shot already past or invalid.
fn next_run(entry: &ScheduleEntry, after: DateTime<Local>) -> Option<DateTime<Local>> {
    if let Some(ref schedule) = entry.cron {
        schedule
            .after(&after)
            .next()
    } else if let Some(at) = entry.at {
        if at > after {
            Some(at)
        } else {
            None
        }
    } else {
        None
    }
}

/// Execute a single task: direct tool (FETCH_URL/BRAVE_SEARCH) or Ollama.
/// Returns Some(reply_text) when the task produced a reply (Ollama path) so the caller can e.g. send to Discord; None otherwise.
async fn execute_task(entry: &ScheduleEntry) -> Option<String> {
    let id_info = entry
        .id
        .as_deref()
        .unwrap_or("(no id)");
    let task = entry.task.trim();

    if task.to_uppercase().starts_with("FETCH_URL:") {
        let arg = task["FETCH_URL:".len()..].trim();
        let semi = arg.find(';').unwrap_or(arg.len());
        let url: String = arg[..semi].trim().to_string();
        if url.is_empty() {
            warn!("Scheduler: FETCH_URL with empty URL (id={})", id_info);
            return None;
        }
        info!("Scheduler: running FETCH_URL for {} (id={})", url, id_info);
        match tokio::task::spawn_blocking(move || crate::commands::browser::fetch_page_content(&url)).await {
            Ok(Ok(body)) => {
                info!("Scheduler: FETCH_URL succeeded ({} chars)", body.chars().count());
            }
            Ok(Err(e)) => {
                error!("Scheduler: FETCH_URL failed (id={}): {}", id_info, e);
            }
            Err(e) => {
                error!("Scheduler: FETCH_URL task join error (id={}): {}", id_info, e);
            }
        }
        return None;
    }

    if task.to_uppercase().starts_with("BRAVE_SEARCH:") {
        let query = task["BRAVE_SEARCH:".len()..].trim();
        let semi = query.find(';').unwrap_or(query.len());
        let query = query[..semi].trim();
        if query.is_empty() {
            warn!("Scheduler: BRAVE_SEARCH with empty query (id={})", id_info);
            return None;
        }
        info!("Scheduler: running BRAVE_SEARCH for {} (id={})", query, id_info);
        match crate::commands::brave::get_brave_api_key() {
            Some(api_key) => {
                match crate::commands::brave::brave_web_search(query, &api_key).await {
                    Ok(results) => {
                        info!("Scheduler: BRAVE_SEARCH succeeded ({} chars)", results.chars().count());
                    }
                    Err(e) => {
                        error!("Scheduler: BRAVE_SEARCH failed (id={}): {}", id_info, e);
                    }
                }
            }
            None => {
                warn!("Scheduler: BRAVE_SEARCH skipped (no API key) (id={})", id_info);
            }
        }
        return None;
    }

    if task.to_uppercase().starts_with("TASK:") || task.to_uppercase().starts_with("TASK_RUN:") {
        let prefix_len = if task.to_uppercase().starts_with("TASK_RUN:") {
            "TASK_RUN:".len()
        } else {
            "TASK:".len()
        };
        let path_or_id = task[prefix_len..].trim();
        if path_or_id.is_empty() {
            warn!("Scheduler: TASK: with empty path/id (id={})", id_info);
            return None;
        }
        info!("Scheduler: running task until finished (id={}, path_or_id={})", id_info, path_or_id);
        return match crate::task::resolve_task_path(path_or_id) {
            Ok(path) => {
                let task_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(path_or_id);
                if let Some(ref channel_id_str) = entry.reply_to_channel_id {
                    if let Ok(channel_id) = channel_id_str.parse::<u64>() {
                        let msg = format!("Working on task '{}' now.", task_name);
                        if let Err(e) = crate::discord::send_message_to_channel(channel_id, &msg).await {
                            error!("Scheduler: failed to send 'working on task' to Discord channel {}: {}", channel_id_str, e);
                        } else {
                            info!("Scheduler: sent 'working on task' to Discord channel {}", channel_id_str);
                        }
                    }
                }
                match crate::commands::ollama::run_task_until_finished(path, 10).await {
                    Ok(reply) => {
                        info!("Scheduler: task completed (id={}, {} chars)", id_info, reply.chars().count());
                        Some(reply)
                    }
                    Err(e) => {
                        error!("Scheduler: task run failed (id={}): {}", id_info, e);
                        None
                    }
                }
            }
            Err(e) => {
                error!("Scheduler: task path resolve failed (id={}): {}", id_info, e);
                None
            }
        };
    }

    info!("Scheduler: running via Ollama (id={}): {}...", id_info, task.chars().take(60).collect::<String>());
    match crate::commands::ollama::answer_with_ollama_and_fetch(task, None, None, None, None, None, None, None, false).await {
        Ok(reply) => {
            info!("Scheduler: Ollama completed (id={}, {} chars)", id_info, reply.chars().count());
            Some(reply)
        }
        Err(e) => {
            error!("Scheduler: Ollama failed (id={}): {}", id_info, e);
            None
        }
    }
}

async fn scheduler_loop() {
    loop {
        let mtime_before = schedules_file_mtime();
        let entries = load_schedules();
        let now = Local::now();

        let mut next_runs: Vec<(DateTime<Local>, usize)> = entries
            .iter()
            .enumerate()
            .filter_map(|(i, e)| next_run(e, now).map(|t| (t, i)))
            .collect();

        next_runs.sort_by_key(|(t, _)| *t);

        // Log every schedule's next run so all are visible (not just the soonest).
        for (t, idx) in &next_runs {
            let e = &entries[*idx];
            let id_info = e.id.as_deref().unwrap_or("(no id)");
            debug!(
                "Scheduler: id={} next at {} (task: {}...)",
                id_info,
                t.format("%Y-%m-%d %H:%M:%S"),
                e.task.chars().take(35).collect::<String>()
            );
        }
        if next_runs.len() != entries.len() {
            debug!(
                "Scheduler: {} entries loaded, {} have a next run (others may be one-shot past)",
                entries.len(),
                next_runs.len()
            );
        }

        if next_runs.is_empty() {
            tokio::time::sleep(Duration::from_secs(FILE_CHECK_INTERVAL_SECS)).await;
            if schedules_file_mtime() != mtime_before {
                info!("Scheduler: schedules file changed, reloading");
            }
            continue;
        }

        let (next_time, idx) = next_runs[0];
        let entry = &entries[idx];
        let id_info = entry.id.as_deref().unwrap_or("(no id)");
        debug!(
            "Scheduler: next run at {} for id={} (task: {}...)",
            next_time.format("%Y-%m-%d %H:%M:%S"),
            id_info,
            entry.task.chars().take(40).collect::<String>()
        );
        // Use millisecond precision so we don't spin when next run is < 1 second away (num_seconds() would truncate to 0).
        let wait_ms = (next_time - now).num_milliseconds().max(0) as u64;
        let sleep_millis = wait_ms
            .min(MAX_SLEEP_SECS * 1000)
            .min(FILE_CHECK_INTERVAL_SECS * 1000);
        let sleep_duration = Duration::from_millis(sleep_millis);

        tokio::time::sleep(sleep_duration).await;

        if schedules_file_mtime() != mtime_before {
            info!("Scheduler: schedules file changed, reloading");
            continue;
        }

        let now_after_sleep = Local::now();
        if now_after_sleep >= next_time {
            let entry = &entries[idx];
            let reply = execute_task(entry).await;
            if let (Some(ref channel_id_str), Some(ref text)) = (&entry.reply_to_channel_id, &reply) {
                if let Ok(channel_id) = channel_id_str.parse::<u64>() {
                    if let Err(e) = crate::discord::send_message_to_channel(channel_id, text).await {
                        error!("Scheduler: failed to send result to Discord channel {}: {}", channel_id_str, e);
                    } else {
                        info!("Scheduler: sent result to Discord channel {}", channel_id_str);
                    }
                }
            }
            // One-shot: if it was "at", we don't remove from file; next load will skip it (at is in past).
            // So we just continue and reload.
        }
    }
}

/// Outcome of adding a schedule: either added or skipped because an equivalent already exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleAddOutcome {
    Added,
    AlreadyExists,
}

/// Normalize task for duplicate check (trim, collapse whitespace).
fn task_normalized_for_dedup(task: &str) -> String {
    task.trim().split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Add a schedule entry to the file (e.g. from Discord when Ollama invokes SCHEDULE).
/// Uses cron only (no one-shot "at"). Id should be unique (e.g. "discord-<timestamp>").
/// If an entry with the same cron and same task (normalized) already exists, returns AlreadyExists and does not add.
/// Optional reply_to_channel_id: when set, the scheduler will send the task result to that Discord channel.
/// Logs at INFO so scheduling is visible in debug output and Discord flow.
pub fn add_schedule(
    id: String,
    cron_str: String,
    task: String,
    reply_to_channel_id: Option<String>,
) -> Result<ScheduleAddOutcome, String> {
    let _ = Config::ensure_schedules_directory();
    let path = Config::schedules_file_path();

    let mut file_data = if path.exists() {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read schedules file: {}", e))?;
        serde_json::from_str::<SchedulesFile>(&content)
            .map_err(|e| format!("Failed to parse schedules file: {}", e))?
    } else {
        SchedulesFile {
            schedules: Vec::new(),
        }
    };

    let task_norm = task_normalized_for_dedup(&task);
    let is_duplicate = file_data.schedules.iter().any(|e| {
        e.cron.as_deref() == Some(cron_str.as_str())
            && task_normalized_for_dedup(&e.task) == task_norm
    });
    if is_duplicate {
        info!(
            "Scheduler: skipping duplicate (same cron and task already scheduled)",
        );
        return Ok(ScheduleAddOutcome::AlreadyExists);
    }

    file_data.schedules.push(ScheduleEntryRaw {
        id: Some(id.clone()),
        cron: Some(cron_str.clone()),
        at: None,
        task: task.clone(),
        reply_to_channel_id: reply_to_channel_id.clone(),
    });

    let json = serde_json::to_string_pretty(&file_data)
        .map_err(|e| format!("Failed to serialize schedules: {}", e))?;
    std::fs::write(&path, json).map_err(|e| format!("Failed to write schedules file: {}", e))?;

    info!(
        "Scheduler: schedule added from agent (id={}, cron={}, task_len={}, reply_channel={})",
        id,
        cron_str,
        task.chars().count(),
        reply_to_channel_id.is_some()
    );
    Ok(ScheduleAddOutcome::Added)
}

/// Add a one-shot schedule entry (run once at a specific datetime). Id should be unique.
/// at_str must be ISO format (e.g. 2025-02-09T05:00:00) as used by load_schedules.
pub fn add_schedule_at(
    id: String,
    at_str: String,
    task: String,
    reply_to_channel_id: Option<String>,
) -> Result<ScheduleAddOutcome, String> {
    let _ = Config::ensure_schedules_directory();
    let path = Config::schedules_file_path();

    let mut file_data = if path.exists() {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read schedules file: {}", e))?;
        serde_json::from_str::<SchedulesFile>(&content)
            .map_err(|e| format!("Failed to parse schedules file: {}", e))?
    } else {
        SchedulesFile {
            schedules: Vec::new(),
        }
    };

    file_data.schedules.push(ScheduleEntryRaw {
        id: Some(id.clone()),
        cron: None,
        at: Some(at_str.clone()),
        task: task.clone(),
        reply_to_channel_id: reply_to_channel_id.clone(),
    });

    let json = serde_json::to_string_pretty(&file_data)
        .map_err(|e| format!("Failed to serialize schedules: {}", e))?;
    std::fs::write(&path, json).map_err(|e| format!("Failed to write schedules file: {}", e))?;

    info!(
        "Scheduler: one-shot schedule added (id={}, at={}, task_len={}, reply_channel={})",
        id,
        at_str,
        task.chars().count(),
        reply_to_channel_id.is_some()
    );
    Ok(ScheduleAddOutcome::Added)
}

/// Spawn the scheduler in a background thread. Reads ~/.mac-stats/schedules.json and runs due tasks.
/// Safe to call once at startup.
pub fn spawn_scheduler_thread() {
    std::thread::spawn(|| {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(r) => r,
            Err(e) => {
                error!("Scheduler: failed to create tokio runtime: {}", e);
                return;
            }
        };
        info!("Scheduler: thread spawned");
        rt.block_on(scheduler_loop());
    });
}
