//! Tool handlers for TASK_*, SCHEDULE, REMOVE_SCHEDULE, LIST_SCHEDULES.
//!
//! Extracted from `ollama.rs` to reduce the size of `answer_with_ollama_and_fetch`.
//! Each handler takes only what it needs and returns the tool result string
//! (plus optional status messages to push to the channel).

use std::path::PathBuf;
use tracing::info;

use crate::commands::schedule_helpers::{parse_schedule_arg, ScheduleParseResult};

/// Send a status update to the channel (if sender is available).
fn send_status(tx: &Option<tokio::sync::mpsc::UnboundedSender<String>>, msg: &str) {
    if let Some(ref tx) = tx {
        let _ = tx.send(msg.to_string());
    }
}

// ---------------------------------------------------------------------------
// SCHEDULE
// ---------------------------------------------------------------------------

pub(crate) fn handle_schedule(
    arg: &str,
    allow_schedule: bool,
    discord_reply_channel_id: Option<u64>,
    status_tx: &Option<tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    if !allow_schedule {
        info!("Agent router: SCHEDULE ignored (disabled in scheduler context)");
        return "Scheduling is not available when running from a scheduled task. Do not add a schedule; complete the task without scheduling."
            .to_string();
    }

    let schedule_preview: String = arg.chars().take(50).collect();
    let schedule_preview = schedule_preview.trim();
    send_status(
        status_tx,
        &format!(
            "Scheduling: {}…",
            if schedule_preview.is_empty() {
                "…"
            } else {
                schedule_preview
            }
        ),
    );
    info!(
        "Agent router: SCHEDULE requested (arg len={})",
        arg.chars().count()
    );

    match parse_schedule_arg(arg) {
        Ok(ScheduleParseResult::Cron { cron_str, task }) => {
            let id = format!("discord-{}", chrono::Utc::now().timestamp());
            let reply_to_channel_id = discord_reply_channel_id.map(|u| u.to_string());
            match crate::scheduler::add_schedule(
                id.clone(),
                cron_str.clone(),
                task.clone(),
                reply_to_channel_id,
            ) {
                Ok(crate::scheduler::ScheduleAddOutcome::Added) => {
                    info!(
                        "Agent router: SCHEDULE added (id={}, cron={})",
                        id, cron_str
                    );
                    let task_preview: String = task.chars().take(100).collect();
                    format!(
                        "Schedule added successfully. Schedule ID: **{}**. The scheduler will run this task (cron: {}): \"{}\". Tell the user the schedule ID is {} and they can remove it later with \"Remove schedule: {}\" or by saying REMOVE_SCHEDULE: {}.",
                        id,
                        cron_str,
                        task_preview.trim(),
                        id,
                        id,
                        id
                    )
                }
                Ok(crate::scheduler::ScheduleAddOutcome::AlreadyExists) => {
                    info!("Agent router: SCHEDULE skipped (same task already scheduled)");
                    "This task is already scheduled with the same cron and description. Tell the user no duplicate was added."
                        .to_string()
                }
                Err(e) => {
                    info!("Agent router: SCHEDULE failed: {}", e);
                    format!(
                        "Failed to add schedule: {}. Tell the user and suggest they check ~/.mac-stats/schedules.json.",
                        e
                    )
                }
            }
        }
        Ok(ScheduleParseResult::At { at_str, task }) => {
            let id = format!("discord-{}", chrono::Utc::now().timestamp());
            let reply_to_channel_id = discord_reply_channel_id.map(|u| u.to_string());
            match crate::scheduler::add_schedule_at(
                id.clone(),
                at_str.clone(),
                task.clone(),
                reply_to_channel_id,
            ) {
                Ok(crate::scheduler::ScheduleAddOutcome::Added) => {
                    info!(
                        "Agent router: SCHEDULE at added (id={}, at={})",
                        id, at_str
                    );
                    let task_preview: String = task.chars().take(100).collect();
                    format!(
                        "One-time schedule added. Schedule ID: **{}** (at {}): \"{}\". Tell the user the schedule ID is {} and they can remove it with \"Remove schedule: {}\" or REMOVE_SCHEDULE: {}.",
                        id,
                        at_str,
                        task_preview.trim(),
                        id,
                        id,
                        id
                    )
                }
                Ok(crate::scheduler::ScheduleAddOutcome::AlreadyExists) => {
                    info!("Agent router: SCHEDULE at skipped (duplicate)");
                    "This one-time schedule was already added. Tell the user no duplicate was added."
                        .to_string()
                }
                Err(e) => {
                    info!("Agent router: SCHEDULE at failed: {}", e);
                    format!(
                        "Failed to add one-shot schedule: {}. Tell the user and suggest they check ~/.mac-stats/schedules.json.",
                        e
                    )
                }
            }
        }
        Err(e) => {
            info!("Agent router: SCHEDULE parse failed: {}", e);
            format!(
                "Could not parse schedule (expected e.g. \"every 5 minutes <task>\", \"at <datetime> <task>\", or \"<cron> <task>\"): {}. Ask the user to rephrase.",
                e
            )
        }
    }
}

// ---------------------------------------------------------------------------
// REMOVE_SCHEDULE
// ---------------------------------------------------------------------------

pub(crate) fn handle_remove_schedule(
    arg: &str,
    status_tx: &Option<tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    let id = arg.trim();
    if id.is_empty() {
        return "REMOVE_SCHEDULE requires a schedule ID (e.g. discord-1770648842). Ask the user which schedule to remove or to provide the ID.".to_string();
    }
    send_status(status_tx, &format!("Removing schedule: {}…", id));
    info!("Agent router: REMOVE_SCHEDULE requested: id={}", id);
    match crate::scheduler::remove_schedule_by_id(id) {
        Ok(true) => format!(
            "Schedule {} has been removed. Tell the user it is cancelled.",
            id
        ),
        Ok(false) => format!(
            "No schedule found with ID \"{}\". The ID may be wrong or already removed. Tell the user.",
            id
        ),
        Err(e) => {
            format!("Failed to remove schedule: {}. Tell the user.", e)
        }
    }
}

// ---------------------------------------------------------------------------
// LIST_SCHEDULES
// ---------------------------------------------------------------------------

pub(crate) fn handle_list_schedules(
    status_tx: &Option<tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    send_status(status_tx, "Listing schedules…");
    info!("Agent router: LIST_SCHEDULES requested");
    let list = crate::scheduler::list_schedules_formatted();
    format!("{}\n\nUse this to answer the user.", list)
}

// ---------------------------------------------------------------------------
// TASK_APPEND
// ---------------------------------------------------------------------------

pub(crate) fn handle_task_append(
    arg: &str,
    current_task_path: &mut Option<PathBuf>,
    last_run_cmd_raw_output: &mut Option<String>,
    status_tx: &Option<tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    let (path_or_id, content) = match arg.find(' ') {
        Some(i) => (arg[..i].trim(), arg[i..].trim()),
        None => ("", ""),
    };
    if path_or_id.is_empty() || content.is_empty() {
        return "TASK_APPEND requires: TASK_APPEND: <path or task id> <content>.".to_string();
    }
    match crate::task::resolve_task_path(path_or_id) {
        Ok(path) => {
            *current_task_path = Some(path.clone());
            let task_label = crate::task::task_file_name(&path);
            send_status(
                status_tx,
                &format!("Appending to task '{}'…", task_label),
            );
            let content_to_append = if let Some(raw) = last_run_cmd_raw_output.take() {
                info!(
                    "Agent router: TASK_APPEND using full RUN_CMD output ({} chars) for task '{}'",
                    raw.chars().count(),
                    task_label
                );
                raw
            } else {
                content.to_string()
            };
            info!(
                "Agent router: TASK_APPEND for task '{}' ({} chars)",
                task_label,
                content_to_append.chars().count()
            );
            match crate::task::append_to_task(&path, &content_to_append) {
                Ok(()) => format!(
                    "Appended to task file '{}'. Use this to continue.",
                    task_label
                ),
                Err(e) => format!("TASK_APPEND failed: {}.", e),
            }
        }
        Err(e) => format!("TASK_APPEND failed: {}.", e),
    }
}

// ---------------------------------------------------------------------------
// TASK_STATUS
// ---------------------------------------------------------------------------

pub(crate) fn handle_task_status(
    arg: &str,
    current_task_path: &mut Option<PathBuf>,
) -> String {
    let parts: Vec<&str> = arg.split_whitespace().collect();
    if parts.len() < 2 {
        return "TASK_STATUS requires: TASK_STATUS: <path or task id> wip|finished.".to_string();
    }
    let mut path_or_id = parts[0].to_string();
    let mut status: Option<String> = None;
    for (i, part) in parts.iter().skip(1).enumerate() {
        let s = part.trim_end_matches(['.', ',', ';']).to_lowercase();
        if ["wip", "finished", "unsuccessful", "paused"].contains(&s.as_str()) {
            status = Some(s);
            if i > 0 {
                path_or_id = parts[..=i].join(" ");
            }
            break;
        }
    }
    match status {
        None => {
            "TASK_STATUS status must be wip, finished, unsuccessful, or paused.".to_string()
        }
        Some(status) => match crate::task::resolve_task_path(&path_or_id) {
            Ok(path) => {
                if status == "finished"
                    && !crate::task::all_sub_tasks_closed(&path).unwrap_or(true)
                {
                    "Cannot set status to finished: not all sub-tasks (## Sub-tasks: ...) are finished or unsuccessful.".to_string()
                } else {
                    match crate::task::set_task_status(&path, &status) {
                        Ok(new_path) => {
                            *current_task_path = Some(new_path.clone());
                            format!(
                                "Task status set to {} (file: {}).",
                                status,
                                crate::task::task_file_name(&new_path)
                            )
                        }
                        Err(e) => format!("TASK_STATUS failed: {}.", e),
                    }
                }
            }
            Err(e) => format!("TASK_STATUS failed: {}.", e),
        },
    }
}

// ---------------------------------------------------------------------------
// TASK_CREATE
// ---------------------------------------------------------------------------

pub(crate) fn handle_task_create(
    arg: &str,
    discord_reply_channel_id: Option<u64>,
    current_task_path: &mut Option<PathBuf>,
) -> String {
    let segs: Vec<&str> = arg.splitn(3, ' ').map(str::trim).collect();
    if segs.len() >= 3 && !segs[2].is_empty() {
        let topic = segs[0];
        let id = segs[1];
        let initial_content = segs[2];
        let content = if let Some(pos) = initial_content.to_uppercase().find(" THEN ") {
            initial_content[..pos].trim()
        } else {
            initial_content
        };
        let reply_to = discord_reply_channel_id;
        match crate::task::create_task(topic, id, content, None, reply_to) {
            Ok(path) => {
                *current_task_path = Some(path.clone());
                let name = crate::task::task_file_name(&path);
                format!(
                    "Task created: {}. Use TASK_APPEND: {} or TASK_APPEND: <id> <content> and TASK_STATUS to update.",
                    name, name
                )
            }
            Err(e) => format!("TASK_CREATE failed: {}.", e),
        }
    } else {
        "TASK_CREATE requires: TASK_CREATE: <topic> <id> <initial content>.".to_string()
    }
}

// ---------------------------------------------------------------------------
// TASK_SHOW
// ---------------------------------------------------------------------------

pub(crate) fn handle_task_show(
    arg: &str,
    current_task_path: &mut Option<PathBuf>,
    status_tx: &Option<tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    if arg.trim().is_empty() {
        return "TASK_SHOW requires: TASK_SHOW: <path or task id>.".to_string();
    }
    send_status(status_tx, "Showing task…");
    info!("Agent router: TASK_SHOW requested: {}", arg.trim());
    match crate::task::resolve_task_path(arg.trim()) {
        Ok(path) => {
            *current_task_path = Some(path.clone());
            match crate::task::show_task_content(&path) {
                Ok((status_str, assignee, content)) => {
                    const MAX_CHANNEL_MSG: usize = 1900;
                    let body = format!(
                        "**Status:** {} | **Assigned:** {}\n\n{}",
                        status_str, assignee, content
                    );
                    let msg = if body.chars().count() <= MAX_CHANNEL_MSG {
                        body
                    } else {
                        crate::logging::ellipse(&body, MAX_CHANNEL_MSG)
                    };
                    send_status(status_tx, &msg);
                    "Task content was sent to the user in the channel. They can ask you to TASK_APPEND or TASK_STATUS for this task.".to_string()
                }
                Err(e) => format!("TASK_SHOW failed: {}.", e),
            }
        }
        Err(e) => format!("TASK_SHOW failed: {}.", e),
    }
}

// ---------------------------------------------------------------------------
// TASK_ASSIGN
// ---------------------------------------------------------------------------

pub(crate) fn handle_task_assign(
    arg: &str,
    current_task_path: &mut Option<PathBuf>,
    status_tx: &Option<tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    let parts: Vec<&str> = arg.split_whitespace().collect();
    if parts.len() < 2 {
        return "TASK_ASSIGN requires: TASK_ASSIGN: <path or task id> <agent_id> (e.g. scheduler, discord, cpu, default).".to_string();
    }
    let path_or_id = parts[..parts.len() - 1].join(" ");
    let agent_id_raw = parts[parts.len() - 1];
    let agent_id = match agent_id_raw.to_uppercase().as_str() {
        "CURSOR_AGENT" | "CURSOR-AGENT" => "scheduler",
        _ => agent_id_raw,
    };
    send_status(status_tx, &format!("Assigning task to {}…", agent_id));
    info!(
        "Agent router: TASK_ASSIGN {} -> {} (raw: {})",
        path_or_id, agent_id, agent_id_raw
    );
    match crate::task::resolve_task_path(&path_or_id) {
        Ok(path) => {
            *current_task_path = Some(path.clone());
            match crate::task::set_assignee(&path, agent_id) {
                Ok(()) => {
                    let _ = crate::task::append_to_task(
                        &path,
                        &format!("Reassigned to {}.", agent_id),
                    );
                    format!("Task assigned to {}.", agent_id)
                }
                Err(e) => format!("TASK_ASSIGN failed: {}.", e),
            }
        }
        Err(e) => format!("TASK_ASSIGN failed: {}.", e),
    }
}

// ---------------------------------------------------------------------------
// TASK_SLEEP
// ---------------------------------------------------------------------------

pub(crate) fn handle_task_sleep(
    arg: &str,
    current_task_path: &mut Option<PathBuf>,
    status_tx: &Option<tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    let parts: Vec<&str> = arg.split_whitespace().collect();
    let (path_or_id, until_str) = if parts.len() >= 3
        && parts[parts.len() - 2].eq_ignore_ascii_case("until")
    {
        (parts[..parts.len() - 2].join(" "), parts[parts.len() - 1])
    } else if parts.len() >= 2 {
        (parts[..parts.len() - 1].join(" "), parts[parts.len() - 1])
    } else {
        ("".to_string(), "")
    };
    if path_or_id.is_empty() || until_str.is_empty() {
        return "TASK_SLEEP requires: TASK_SLEEP: <path or task id> until <ISO datetime> (e.g. 2025-02-10T09:00:00).".to_string();
    }
    send_status(status_tx, "Pausing task…");
    info!(
        "Agent router: TASK_SLEEP {} until {}",
        path_or_id, until_str
    );
    match crate::task::resolve_task_path(&path_or_id) {
        Ok(path) => {
            *current_task_path = Some(path.clone());
            if let Ok(new_path) = crate::task::set_task_status(&path, "paused") {
                *current_task_path = Some(new_path.clone());
                let _ = crate::task::set_paused_until(&new_path, Some(until_str));
                let _ = crate::task::append_to_task(
                    &new_path,
                    &format!("Paused until {}.", until_str),
                );
            }
            format!(
                "Task paused until {}. It will resume automatically after that time.",
                until_str
            )
        }
        Err(e) => format!("TASK_SLEEP failed: {}.", e),
    }
}

// ---------------------------------------------------------------------------
// TASK_LIST
// ---------------------------------------------------------------------------

pub(crate) fn handle_task_list(
    arg: &str,
    status_tx: &Option<tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    let show_all = arg.trim().to_lowercase() == "all"
        || arg.trim().to_lowercase() == "all tasks"
        || arg.trim().to_lowercase().starts_with("all ");
    if show_all {
        send_status(status_tx, "Listing all tasks (by status)…");
        info!("Agent router: TASK_LIST all requested");
        match crate::task::format_list_all_tasks() {
            Ok(list) => {
                const MAX_CHANNEL_MSG: usize = 1900;
                const LIST_MAX: usize = MAX_CHANNEL_MSG - 20;
                let msg = if list.chars().count() <= LIST_MAX {
                    format!("**All tasks**\n\n{}", list)
                } else {
                    format!(
                        "**All tasks**\n\n{}",
                        crate::logging::ellipse(&list, LIST_MAX)
                    )
                };
                send_status(status_tx, &msg);
                "The full task list (Open, WIP, Finished, Unsuccessful) was sent to the user in the channel. Acknowledge that you showed all tasks. Task ids are the filenames; the user can use TASK_APPEND or TASK_STATUS with those ids.".to_string()
            }
            Err(e) => format!("TASK_LIST failed: {}.", e),
        }
    } else {
        send_status(status_tx, "Listing open and WIP tasks…");
        info!("Agent router: TASK_LIST requested");
        match crate::task::format_list_open_and_wip_tasks() {
            Ok(list) => {
                const MAX_CHANNEL_MSG: usize = 1900;
                const LIST_MAX: usize = MAX_CHANNEL_MSG - 20;
                let msg = if list.chars().count() <= LIST_MAX {
                    format!("**Active task list**\n\n{}", list)
                } else {
                    format!(
                        "**Active task list**\n\n{}",
                        crate::logging::ellipse(&list, LIST_MAX)
                    )
                };
                send_status(status_tx, &msg);
                "The task list was sent to the user in the channel. Acknowledge that you showed the list. Task ids are the filenames; the user can use TASK_APPEND or TASK_STATUS with those ids.".to_string()
            }
            Err(e) => format!("TASK_LIST failed: {}.", e),
        }
    }
}
