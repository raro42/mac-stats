//! Run a task file until status is finished. Used by the review loop and scheduler.
//! When the request came from Discord (or a schedule with reply_to_channel_id), sends the finished task summary back to that channel.

use std::path::PathBuf;
use tracing::{error, info};

/// Result of running a task: (summary_text, already_sent_to_discord).
/// When already_sent_to_discord is true, the caller (e.g. scheduler) should not send again.
pub type RunTaskResult = Result<(String, bool), String>;

/// Run a task file until status is finished. Reads the task, sends to Ollama with TASK_APPEND/TASK_STATUS
/// instructions, then re-reads and repeats until status is "finished" or max_iterations is reached.
/// Auto-closes as unsuccessful if max iterations reached without finished.
/// When reply_to_discord_channel is Some (from scheduler or from task file ## Reply-to: discord <id>), sends the finished task summary to that channel.
/// message_prefix is prepended when sending (e.g. "[Schedule: id] "). Caller should pass None if not from scheduler.
pub async fn run_task_until_finished(
    task_path: PathBuf,
    max_iterations: u32,
    reply_to_discord_channel: Option<u64>,
    message_prefix: Option<String>,
) -> RunTaskResult {
    let task_name = task_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("task")
        .to_string();
    info!("Task loop: working on task '{}'", task_name);
    match crate::task::status_from_path(&task_path).as_deref() {
        Some("finished") => {
            info!("Task loop: task '{}' already finished", task_name);
            let summary = "Task already finished.".to_string();
            let sent = send_finished_summary_if_channel(
                &task_path,
                reply_to_discord_channel,
                &message_prefix,
                &summary,
            )
            .await;
            return Ok((summary, sent));
        }
        Some("unsuccessful") => {
            info!(
                "Task loop: task '{}' already closed as unsuccessful",
                task_name
            );
            return Ok(("Task already closed as unsuccessful.".to_string(), false));
        }
        _ => {}
    }
    let mut current_path = task_path;
    let mut last_reply = String::new();
    for iteration in 0..max_iterations {
        let content = crate::task::read_task(&current_path).map_err(|e| e.clone())?;
        let assignee =
            crate::task::get_assignee(&current_path).unwrap_or_else(|_| "default".to_string());
        let agents = crate::agents::load_agents();
        let agent_override = crate::agents::find_agent_by_id_or_name(&agents, &assignee).cloned();
        let question = format!(
            "Current task file content:\n\n{}\n\nDecide the next step. For implement/refactor/add-feature/code tasks: use CURSOR_AGENT: <instruction> to have the editor apply changes, then TASK_APPEND with the result and TASK_STATUS when done. Otherwise use TASK_APPEND to add feedback and TASK_STATUS to set wip or finished. Reply with your action (CURSOR_AGENT, TASK_APPEND, TASK_STATUS, or a final summary).",
            content
        );
        info!(
            "Task loop: iteration {}/{} for task '{}' (assignee: {})",
            iteration + 1,
            max_iterations,
            task_name,
            assignee
        );
        let reply = crate::commands::ollama::answer_with_ollama_and_fetch(
            &question,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            agent_override,
            false,
            None,
            false,
            true,
            true,
            None,
            None,
            false,
            None,
            None,
        )
        .await?;
        last_reply = reply.text;
        if let Some(ref p) = crate::task::find_current_path(&current_path) {
            current_path = p.clone();
        }
        if crate::task::status_from_path(&current_path).as_deref() == Some("finished") {
            info!("Task loop: task '{}' finished", task_name);
            let sent = send_finished_summary_if_channel(
                &current_path,
                reply_to_discord_channel,
                &message_prefix,
                &last_reply,
            )
            .await;
            return Ok((last_reply, sent));
        }
    }
    info!(
        "Task loop: max iterations ({}) reached for task '{}'",
        max_iterations, task_name
    );
    if let Some(ref p) = crate::task::find_current_path(&current_path) {
        let status = crate::task::status_from_path(p).unwrap_or_default();
        if status != "finished" && status != "unsuccessful" {
            if let Ok(new_path) = crate::task::set_task_status(p, "unsuccessful") {
                let _ = crate::task::append_to_task(
                    &new_path,
                    "Max iterations reached; closed as unsuccessful.",
                );
            }
        }
    }
    let summary = format!(
        "Max iterations ({}) reached. Last reply: {}",
        max_iterations,
        last_reply.chars().take(500).collect::<String>()
    );
    let sent = send_finished_summary_if_channel(
        &current_path,
        reply_to_discord_channel,
        &message_prefix,
        &summary,
    )
    .await;
    Ok((summary, sent))
}

/// If we have a Discord channel (from caller or from task file ## Reply-to: discord <id>), send the finished summary. Returns true if sent.
async fn send_finished_summary_if_channel(
    task_path: &PathBuf,
    reply_to_override: Option<u64>,
    message_prefix: &Option<String>,
    summary: &str,
) -> bool {
    let channel_id =
        reply_to_override.or_else(|| crate::task::get_reply_to_discord_channel(task_path));
    if let Some(channel_id) = channel_id {
        let prefix = message_prefix.as_deref().unwrap_or("");
        let message = format!("{}Task finished.\n\n{}", prefix, summary.trim());
        match crate::discord::send_message_to_channel(channel_id, &message).await {
            Ok(()) => {
                info!(
                    "Task runner: sent finished summary to Discord channel {}",
                    channel_id
                );
                true
            }
            Err(e) => {
                error!(
                    "Task runner: failed to send finished summary to Discord channel {}: {}",
                    channel_id, e
                );
                false
            }
        }
    } else {
        false
    }
}
