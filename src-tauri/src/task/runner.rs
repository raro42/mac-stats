//! Run a task file until status is finished. Used by the review loop and scheduler.

use std::path::PathBuf;
use tracing::info;

/// Run a task file until status is finished. Reads the task, sends to Ollama with TASK_APPEND/TASK_STATUS
/// instructions, then re-reads and repeats until status is "finished" or max_iterations is reached.
/// Auto-closes as unsuccessful if max iterations reached without finished.
pub async fn run_task_until_finished(task_path: PathBuf, max_iterations: u32) -> Result<String, String> {
    let task_name = task_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("task")
        .to_string();
    info!("Task loop: working on task '{}'", task_name);
    match crate::task::status_from_path(&task_path).as_deref() {
        Some("finished") => {
            info!("Task loop: task '{}' already finished", task_name);
            return Ok("Task already finished.".to_string());
        }
        Some("unsuccessful") => {
            info!("Task loop: task '{}' already closed as unsuccessful", task_name);
            return Ok("Task already closed as unsuccessful.".to_string());
        }
        _ => {}
    }
    let mut current_path = task_path;
    let mut last_reply = String::new();
    for iteration in 0..max_iterations {
        let content = crate::task::read_task(&current_path).map_err(|e| e.clone())?;
        let question = format!(
            "Current task file content:\n\n{}\n\nDecide the next step. Use TASK_APPEND to add feedback and TASK_STATUS to set wip or finished when done. Reply with your action (TASK_APPEND, TASK_STATUS, or a final summary).",
            content
        );
        info!("Task loop: iteration {}/{} for task '{}'", iteration + 1, max_iterations, task_name);
        last_reply = crate::commands::ollama::answer_with_ollama_and_fetch(
            &question, None, None, None, None, None, None, None, None, false,
        )
        .await?;
        if let Some(ref p) = crate::task::find_current_path(&current_path) {
            current_path = p.clone();
        }
        if crate::task::status_from_path(&current_path).as_deref() == Some("finished") {
            info!("Task loop: task '{}' finished", task_name);
            return Ok(last_reply);
        }
    }
    info!("Task loop: max iterations ({}) reached for task '{}'", max_iterations, task_name);
    if let Some(ref p) = crate::task::find_current_path(&current_path) {
        let status = crate::task::status_from_path(p).unwrap_or_default();
        if status != "finished" && status != "unsuccessful" {
            if let Ok(new_path) = crate::task::set_task_status(p, "unsuccessful") {
                let _ = crate::task::append_to_task(&new_path, "Max iterations reached; closed as unsuccessful.");
            }
        }
    }
    Ok(format!(
        "Max iterations ({}) reached. Last reply: {}",
        max_iterations,
        last_reply.chars().take(500).collect::<String>()
    ))
}
