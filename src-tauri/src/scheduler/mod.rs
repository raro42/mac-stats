//! Scheduler agent: runs tasks at scheduled times from ~/.mac-stats/schedules.json.
//!
//! Loads the file at startup and in a loop: sleeps until the next due time (or a short interval
//! to check for file changes), executes the task (via Ollama + agents or direct FETCH_URL/BRAVE_SEARCH),
//! and re-reads the file whenever it changes (mtime poll) or after each run.

use crate::config::Config;
use crate::mac_stats_info;
use chrono::{DateTime, Local, TimeZone, Utc};
use cron::Schedule;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::time::Duration;
use tracing::{debug, error, info, warn};

pub mod delivery_awareness;
pub mod heartbeat;

pub use delivery_awareness::DeliveryAwarenessEntry;

/// Successful scheduled run with text to optionally post to Discord from the scheduler loop.
struct ScheduleExecuteSuccess {
    reply_text: String,
    already_sent_to_discord: bool,
    delivery_context_key: String,
}

/// Minimum interval when user config is below 1 (safety).
const MIN_CHECK_SECS: u64 = 1;

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
    std::fs::metadata(&path)
        .ok()
        .and_then(|m| m.modified().ok())
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
            warn!(
                "Scheduler: failed to parse schedules file {:?}: {}",
                path, e
            );
            return Vec::new();
        }
    };

    let mut entries = Vec::new();
    for raw in file_data.schedules {
        if raw.task.is_empty() {
            warn!(
                "Scheduler: skipping entry with empty task (id={:?})",
                raw.id
            );
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
                    chrono::NaiveDateTime::parse_from_str(at_str, "%Y-%m-%dT%H:%M:%S").map(|n| {
                        Local
                            .from_local_datetime(&n)
                            .single()
                            .unwrap_or(n.and_utc().with_timezone(&Local))
                    })
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

    debug!(
        "Scheduler: loaded {} entries from {:?}",
        entries.len(),
        path
    );
    entries
}

/// Number of valid schedule entries in `schedules.json` (for feature health dashboard).
pub fn schedule_entry_count() -> usize {
    load_schedules().len()
}

/// UI-facing schedule entry (id, cron/at strings, task, optional reply channel, next run).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScheduleForUi {
    pub id: Option<String>,
    pub cron: Option<String>,
    pub at: Option<String>,
    pub task: String,
    pub reply_to_channel_id: Option<String>,
    pub next_run: Option<String>,
}

/// Compute next run time for a raw entry (for UI display). Returns None if one-shot already past or invalid.
fn next_run_from_raw(raw: &ScheduleEntryRaw) -> Option<String> {
    let now = Local::now();
    if let Some(ref cron_str) = raw.cron {
        if let Ok(schedule) = Schedule::from_str(cron_str) {
            return schedule
                .after(&now)
                .next()
                .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string());
        }
    }
    if let Some(ref at_str) = raw.at {
        let at_dt = chrono::DateTime::parse_from_rfc3339(at_str)
            .map(|dt| dt.with_timezone(&Local))
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(at_str, "%Y-%m-%dT%H:%M:%S").map(|n| {
                    Local
                        .from_local_datetime(&n)
                        .single()
                        .unwrap_or(n.and_utc().with_timezone(&Local))
                })
            });
        if let Ok(dt) = at_dt {
            if dt > now {
                return Some(dt.format("%Y-%m-%d %H:%M:%S").to_string());
            }
        }
    }
    None
}

/// List schedules with next-run for Settings UI.
pub fn list_schedules_for_ui() -> Vec<ScheduleForUi> {
    let _ = Config::ensure_schedules_directory();
    let path = Config::schedules_file_path();

    if !path.exists() {
        return Vec::new();
    }

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let file_data: SchedulesFile = match serde_json::from_str(&content) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    file_data
        .schedules
        .into_iter()
        .filter(|raw| !raw.task.is_empty())
        .filter(|raw| raw.cron.is_some() != raw.at.is_some())
        .map(|raw| ScheduleForUi {
            next_run: next_run_from_raw(&raw),
            id: raw.id,
            cron: raw.cron,
            at: raw.at,
            task: raw.task,
            reply_to_channel_id: raw.reply_to_channel_id,
        })
        .collect()
}

/// Returns a human-readable list of active schedules (id, cron/at, task preview, next run).
/// Used when the user or agent asks to "list schedules".
pub fn list_schedules_formatted() -> String {
    let entries = load_schedules();
    let now = Local::now();
    if entries.is_empty() {
        return "No active schedules.".to_string();
    }
    let mut lines: Vec<String> = Vec::with_capacity(entries.len());
    for (i, e) in entries.iter().enumerate() {
        let id = e.id.as_deref().unwrap_or("(no id)");
        let kind = if e.cron.is_some() {
            "cron"
        } else if e.at.is_some() {
            "one-shot"
        } else {
            "?"
        };
        let next = next_run(e, now)
            .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "—".to_string());
        let task_preview: String = e.task.chars().take(50).collect::<String>();
        if task_preview.len() < e.task.chars().count() {
            lines.push(format!(
                "{}. id: {} ({}), next: {}, task: {}…",
                i + 1,
                id,
                kind,
                next,
                task_preview.trim()
            ));
        } else {
            lines.push(format!(
                "{}. id: {} ({}), next: {}, task: {}",
                i + 1,
                id,
                kind,
                next,
                task_preview.trim()
            ));
        }
    }
    format!(
        "Active schedules ({}):\n{}",
        entries.len(),
        lines.join("\n")
    )
}

/// Compute the next run time for this entry (in local time). Returns None if one-shot already past or invalid.
fn next_run(entry: &ScheduleEntry, after: DateTime<Local>) -> Option<DateTime<Local>> {
    if let Some(ref schedule) = entry.cron {
        schedule.after(&after).next()
    } else {
        entry.at.filter(|&at| at > after)
    }
}

/// Result of running a scheduled task.
/// - Ok(Some(success)): success with optional reply to send from the scheduler loop.
/// - Ok(None): task ran but produced no user-visible Discord outcome (e.g. FETCH_URL/BRAVE_SEARCH internal-only).
/// - Err(msg): task failed; msg is a short user-facing error for Discord when reply_to_channel_id is set.
async fn execute_task(
    entry: &ScheduleEntry,
    due_time_local: DateTime<Local>,
    partial_capture: Option<crate::commands::partial_progress::PartialProgressCapture>,
) -> Result<Option<ScheduleExecuteSuccess>, String> {
    let id_info = entry.id.as_deref().unwrap_or("(no id)");
    let delivery_context_key = delivery_awareness::new_context_key_for_schedule(id_info);
    let task = entry.task.trim();

    if task.to_uppercase().starts_with("FETCH_URL:") {
        let arg = task["FETCH_URL:".len()..].trim();
        let url = match crate::commands::browser::extract_first_url(arg) {
            Some(u) => u,
            None => {
                warn!("Scheduler: FETCH_URL with no valid URL (id={})", id_info);
                return Ok(None);
            }
        };
        info!("Scheduler: running FETCH_URL for {} (id={})", url, id_info);
        match tokio::task::spawn_blocking(move || {
            crate::commands::browser::fetch_page_content(&url)
        })
        .await
        {
            Ok(Ok(body)) => {
                info!(
                    "Scheduler: FETCH_URL succeeded ({} chars)",
                    body.chars().count()
                );
                return Ok(None);
            }
            Ok(Err(e)) => {
                error!("Scheduler: FETCH_URL failed (id={}): {}", id_info, e);
                return Err(e.to_string());
            }
            Err(e) => {
                error!(
                    "Scheduler: FETCH_URL task join error (id={}): {}",
                    id_info, e
                );
                return Err(format!("fetch task error: {}", e));
            }
        }
    }

    if task.to_uppercase().starts_with("BRAVE_SEARCH:") {
        let query = task["BRAVE_SEARCH:".len()..].trim();
        let semi = query.find(';').unwrap_or(query.len());
        let query = query[..semi].trim();
        if query.is_empty() {
            warn!("Scheduler: BRAVE_SEARCH with empty query (id={})", id_info);
            return Ok(None);
        }
        info!(
            "Scheduler: running BRAVE_SEARCH for {} (id={})",
            query, id_info
        );
        match crate::commands::brave::get_brave_api_key() {
            Some(api_key) => {
                match crate::commands::brave::brave_web_search(query, &api_key).await {
                    Ok(results) => {
                        info!(
                            "Scheduler: BRAVE_SEARCH succeeded ({} chars)",
                            results.chars().count()
                        );
                        return Ok(None);
                    }
                    Err(e) => {
                        error!("Scheduler: BRAVE_SEARCH failed (id={}): {}", id_info, e);
                        return Err(e.to_string());
                    }
                }
            }
            None => {
                warn!(
                    "Scheduler: BRAVE_SEARCH skipped (no API key) (id={})",
                    id_info
                );
                return Ok(None);
            }
        }
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
            return Ok(None);
        }
        info!(
            "Scheduler: running task until finished (id={}, path_or_id={})",
            id_info, path_or_id
        );
        match crate::task::resolve_task_path(path_or_id) {
            Ok(path) => {
                let task_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(path_or_id);
                let reply_to_channel = entry
                    .reply_to_channel_id
                    .as_ref()
                    .and_then(|s| s.parse::<u64>().ok());
                if let Some(channel_id) = reply_to_channel {
                    let schedule_prefix = entry
                        .id
                        .as_deref()
                        .map(|sid| format!("[Schedule: {}] ", sid))
                        .unwrap_or_default();
                    let msg = format!("{}Working on task '{}' now.", schedule_prefix, task_name);
                    if let Err(e) = crate::discord::send_message_to_channel(channel_id, &msg).await
                    {
                        error!(
                            "Scheduler: failed to send 'working on task' to Discord channel {}: {}",
                            channel_id, e
                        );
                    } else {
                        info!(
                            "Scheduler: sent 'working on task' to Discord channel {}",
                            channel_id
                        );
                    }
                }
                let prefix = entry
                    .id
                    .as_deref()
                    .map(|sid| format!("[Schedule: {}] ", sid));
                let scheduler_awareness =
                    reply_to_channel.map(|_| (delivery_context_key.clone(), entry.id.clone()));
                match crate::task::runner::run_task_until_finished(
                    path,
                    10,
                    reply_to_channel,
                    prefix,
                    scheduler_awareness,
                    partial_capture.clone(),
                )
                .await
                {
                    Ok((reply, sent_to_discord)) => {
                        info!(
                            "Scheduler: task completed (id={}, {} chars, sent_to_discord={})",
                            id_info,
                            reply.chars().count(),
                            sent_to_discord
                        );
                        return Ok(Some(ScheduleExecuteSuccess {
                            reply_text: reply,
                            already_sent_to_discord: sent_to_discord,
                            delivery_context_key,
                        }));
                    }
                    Err(e) => {
                        error!("Scheduler: task run failed (id={}): {}", id_info, e);
                        return Err(e.to_string());
                    }
                }
            }
            Err(e) => {
                error!(
                    "Scheduler: task path resolve failed (id={}): {}",
                    id_info, e
                );
                return Err(e.to_string());
            }
        }
    }

    info!(
        "Scheduler: running via Ollama (id={}): {}...",
        id_info,
        task.chars().take(60).collect::<String>()
    );
    let due_utc = due_time_local.with_timezone(&Utc);
    let reply_to_ch = entry
        .reply_to_channel_id
        .as_ref()
        .and_then(|s| s.parse::<u64>().ok());
    let session_key = reply_to_ch
        .map(|id| format!("discord:{}", id))
        .unwrap_or_else(|| format!("scheduler:{}", id_info));
    let stale_mid = format!("scheduler-due:{}:{}", id_info, due_utc.timestamp_millis());
    crate::commands::suspicious_patterns::log_untrusted_suspicious_scan("scheduler-task", task);
    let ollama_k = session_key.clone();
    let task_body = task.to_string();
    let schedule_id_log = id_info.to_string();
    let delivery_ck = delivery_context_key.clone();
    crate::keyed_queue::run_serial(session_key, async move {
        match crate::commands::ollama::answer_with_ollama_and_fetch(
            crate::commands::ollama::OllamaRequest {
                question: crate::commands::untrusted_content::wrap_untrusted_content(
                    "scheduler-task",
                    &task_body,
                ),
                retry_on_verification_no: true,
                from_remote: true,
                discord_reply_channel_id: reply_to_ch,
                inbound_stale_guard: Some(crate::commands::abort_cutoff::InboundStaleGuard {
                    message_id: stale_mid,
                    timestamp_utc: due_utc,
                }),
                compaction_hook_source: Some("scheduler".to_string()),
                partial_progress_capture: partial_capture.clone(),
                ollama_queue_key: Some(ollama_k),
                ..Default::default()
            },
        )
        .await
        {
            Ok(reply) => {
                info!(
                    "Scheduler: Ollama completed (id={}, {} chars)",
                    schedule_id_log,
                    reply.text.chars().count()
                );
                crate::commands::judge::run_judge_if_enabled(
                    &task_body,
                    &reply.text,
                    &reply.attachment_paths,
                    None,
                )
                .await;
                Ok(Some(ScheduleExecuteSuccess {
                    reply_text: reply.text,
                    already_sent_to_discord: false,
                    delivery_context_key: delivery_ck,
                }))
            }
            Err(e) => {
                if matches!(
                    &e,
                    crate::commands::ollama_run_error::OllamaRunError::StaleInboundAfterAbort
                ) {
                    debug!(
                        target: "mac_stats::ollama/chat",
                        schedule_id = %schedule_id_log,
                        "Scheduler: Ollama run skipped (stale vs abort cutoff)"
                    );
                    return Ok(None);
                }
                error!("Scheduler: Ollama failed (id={}): {}", schedule_id_log, e);
                Err(e.to_string())
            }
        }
    })
    .await
}

async fn scheduler_loop() {
    loop {
        let check_interval_secs = Config::scheduler_check_interval_secs().max(MIN_CHECK_SECS);
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
            tokio::time::sleep(Duration::from_secs(check_interval_secs)).await;
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
        let sleep_millis = wait_ms.min(check_interval_secs * 1000);
        let sleep_duration = Duration::from_millis(sleep_millis);

        tokio::time::sleep(sleep_duration).await;

        if schedules_file_mtime() != mtime_before {
            info!("Scheduler: schedules file changed, reloading");
            continue;
        }

        let now_after_sleep = Local::now();
        if now_after_sleep >= next_time {
            let entry = &entries[idx];
            let timeout_secs = Config::scheduler_task_timeout_secs();
            let timeout_dur = Duration::from_secs(timeout_secs);
            let id_label = entry.id.as_deref().unwrap_or("(no id)");
            info!(
                "Scheduler: executing task id={} (timeout={}s)",
                id_label, timeout_secs
            );
            let partial = crate::commands::partial_progress::PartialProgressCapture::new();
            let result = match tokio::time::timeout(
                timeout_dur,
                execute_task(entry, next_time, Some(partial.clone())),
            )
            .await
            {
                Ok(inner) => inner,
                Err(_elapsed) => {
                    error!(
                        "Scheduler: task id={} timed out after {}s",
                        id_label, timeout_secs
                    );
                    let mut err = format!("task timed out after {}s", timeout_secs);
                    if let Some(summary) = partial.format_user_summary() {
                        mac_stats_info!(
                            "scheduler",
                            "Scheduler: partial progress after timeout (id={}):\n{}",
                            id_label,
                            summary
                        );
                        err.push_str("\n\n");
                        err.push_str(&summary);
                    }
                    Err(err)
                }
            };
            match result {
                Ok(Some(success)) => {
                    if let Some(ref channel_id_str) = entry.reply_to_channel_id {
                        if !success.already_sent_to_discord {
                            if let Ok(channel_id) = channel_id_str.parse::<u64>() {
                                let message = if let Some(ref sid) = entry.id {
                                    format!("[Schedule: {}]\n\n{}", sid, success.reply_text)
                                } else {
                                    success.reply_text.clone()
                                };
                                if let Err(e) =
                                    crate::discord::send_message_to_channel(channel_id, &message)
                                        .await
                                {
                                    error!(
                                        "Scheduler: failed to send result to Discord channel {}: {}",
                                        channel_id_str, e
                                    );
                                } else {
                                    info!(
                                        "Scheduler: sent result to Discord channel {}",
                                        channel_id_str
                                    );
                                    delivery_awareness::record_if_new(
                                        &success.delivery_context_key,
                                        entry.id.as_deref(),
                                        channel_id,
                                        &message,
                                    );
                                }
                            }
                        }
                    }
                }
                Err(ref msg) => {
                    if let Some(ref channel_id_str) = entry.reply_to_channel_id {
                        if let Ok(channel_id) = channel_id_str.parse::<u64>() {
                            let failure_msg = entry
                                .id
                                .as_deref()
                                .map(|sid| format!("[Schedule: {}] Failed: {}", sid, msg))
                                .unwrap_or_else(|| format!("Schedule failed: {}", msg));
                            if let Err(e) =
                                crate::discord::send_message_to_channel(channel_id, &failure_msg)
                                    .await
                            {
                                error!(
                                    "Scheduler: failed to send failure message to Discord channel {}: {}",
                                    channel_id_str, e
                                );
                            } else {
                                info!(
                                    "Scheduler: sent failure message to Discord channel {}",
                                    channel_id_str
                                );
                            }
                        }
                    }
                }
                Ok(None) => {}
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
    task.split_whitespace().collect::<Vec<_>>().join(" ")
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

    if let Some(cap) = Config::max_schedules() {
        if file_data.schedules.len() >= cap as usize {
            return Err(format!(
                "Maximum number of schedules ({}) reached. Remove some with REMOVE_SCHEDULE or increase maxSchedules in ~/.mac-stats/config.json.",
                cap
            ));
        }
    }

    let task_norm = task_normalized_for_dedup(&task);
    let is_duplicate = file_data.schedules.iter().any(|e| {
        e.cron.as_deref() == Some(cron_str.as_str())
            && task_normalized_for_dedup(&e.task) == task_norm
    });
    if is_duplicate {
        info!("Scheduler: skipping duplicate (same cron and task already scheduled)",);
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

    if let Some(cap) = Config::max_schedules() {
        if file_data.schedules.len() >= cap as usize {
            return Err(format!(
                "Maximum number of schedules ({}) reached. Remove some with REMOVE_SCHEDULE or increase maxSchedules in ~/.mac-stats/config.json.",
                cap
            ));
        }
    }

    let task_norm = task_normalized_for_dedup(&task);
    let is_duplicate = file_data.schedules.iter().any(|e| {
        e.at.as_deref() == Some(at_str.as_str()) && task_normalized_for_dedup(&e.task) == task_norm
    });
    if is_duplicate {
        info!("Scheduler: skipping duplicate one-shot (same at and task already scheduled)");
        return Ok(ScheduleAddOutcome::AlreadyExists);
    }

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

/// Remove a schedule entry by id (e.g. "discord-1770648842"). Returns Ok(true) if removed, Ok(false) if not found.
pub fn remove_schedule_by_id(id: &str) -> Result<bool, String> {
    let _ = Config::ensure_schedules_directory();
    let path = Config::schedules_file_path();

    if !path.exists() {
        return Ok(false);
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read schedules file: {}", e))?;
    let mut file_data: SchedulesFile = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse schedules file: {}", e))?;

    let original_len = file_data.schedules.len();
    file_data.schedules.retain(|e| e.id.as_deref() != Some(id));
    let removed = file_data.schedules.len() < original_len;

    if removed {
        let json = serde_json::to_string_pretty(&file_data)
            .map_err(|e| format!("Failed to serialize schedules: {}", e))?;
        std::fs::write(&path, json)
            .map_err(|e| format!("Failed to write schedules file: {}", e))?;
        info!("Scheduler: schedule removed (id={})", id);
    }

    Ok(removed)
}

/// Recent successful scheduler → Discord posts (newest first) for Settings / operator checks.
pub fn list_scheduler_delivery_awareness() -> Vec<DeliveryAwarenessEntry> {
    delivery_awareness::list_entries_newest_first()
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
