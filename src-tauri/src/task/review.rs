//! Task review loop: every 10 minutes, list open/wip tasks, close WIP older than 30 min as unsuccessful,
//! and start working on one open task.

use std::time::{Duration, SystemTime};

use chrono::TimeZone;
use tracing::{debug, error, info, warn};

const REVIEW_INTERVAL_SECS: u64 = 10 * 60; // 10 minutes
const WIP_TIMEOUT_SECS: u64 = 30 * 60;     // 30 minutes
const MAX_ITERATIONS_PER_TASK: u32 = 20;
const MAX_TASKS_PER_CYCLE: u32 = 3;

/// Close WIP tasks whose file was last modified more than 30 minutes ago.
/// Sets status to "unsuccessful" and appends a note.
fn close_stale_wips() {
    let list = match crate::task::list_open_and_wip_tasks() {
        Ok(l) => l,
        Err(e) => {
            warn!("Task review: list_open_and_wip_tasks failed: {}", e);
            return;
        }
    };
    let now = SystemTime::now();
    let timeout = Duration::from_secs(WIP_TIMEOUT_SECS);
    for (path, status, mtime) in list {
        if status != "wip" {
            continue;
        }
        let elapsed = now.duration_since(mtime).unwrap_or(Duration::ZERO);
        if elapsed < timeout {
            continue;
        }
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
        info!(
            "Task review: closing stale WIP ({} min old): {}",
            elapsed.as_secs() / 60,
            name
        );
        match crate::task::set_task_status(&path, "unsuccessful") {
            Ok(new_path) => {
                if let Err(e) = crate::task::append_to_task(&new_path, "Closed as unsuccessful (30 min timeout).") {
                    warn!("Task review: append note failed: {}", e);
                }
            }
            Err(e) => {
                warn!("Task review: set_task_status unsuccessful failed: {}", e);
            }
        }
    }
}

/// Agents that the review loop will work on (scheduler picks only these).
const REVIEW_AGENTS: &[&str] = &["scheduler", "default"];

/// Pick one open task path to work on (only tasks assigned to scheduler or default). Returns None if no open tasks.
fn pick_one_open_task() -> Option<std::path::PathBuf> {
    let list = crate::task::list_open_and_wip_tasks().ok()?;
    let open: Vec<_> = list
        .into_iter()
        .filter(|(_, s, _)| s == "open")
        .map(|(p, _, _)| p)
        .filter(|p| {
            crate::task::get_assignee(p)
                .map(|a| REVIEW_AGENTS.contains(&a.as_str()))
                .unwrap_or(true)
        })
        .filter(|p| crate::task::is_ready(p).unwrap_or(false))
        .collect();
    open.into_iter().next()
}

/// Resume paused tasks whose "paused until" time has passed (rename to open, clear paused-until line).
fn resume_paused_tasks() {
    let list = match crate::task::list_all_tasks() {
        Ok(l) => l,
        Err(_) => return,
    };
    let now = chrono::Local::now();
    for (path, status, _) in list {
        if status != "paused" {
            continue;
        }
        let until_str = match crate::task::get_paused_until(&path) {
            Ok(Some(s)) => s,
            _ => continue,
        };
        let until_local = chrono::DateTime::parse_from_rfc3339(&until_str)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Local))
            .or_else(|| {
                chrono::NaiveDateTime::parse_from_str(&until_str, "%Y-%m-%dT%H:%M:%S").ok()
                    .and_then(|n| chrono::Local.from_local_datetime(&n).single())
            });
        let Some(until_local) = until_local else { continue };
        if now >= until_local {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
            info!("Task review: resuming paused task '{}' (paused until {} passed)", name, until_str);
            if let Ok(new_path) = crate::task::set_task_status(&path, "open") {
                let _ = crate::task::set_paused_until(&new_path, None);
            }
        }
    }
}

/// Run one review cycle: close stale WIPs, resume due paused tasks, then work on up to MAX_TASKS_PER_CYCLE open tasks.
async fn run_review_once() {
    close_stale_wips();
    resume_paused_tasks();
    let mut count = 0u32;
    while count < MAX_TASKS_PER_CYCLE {
        let path = match pick_one_open_task() {
            Some(p) => p,
            None => {
                if count == 0 {
                    debug!("Task review: no open tasks to work on");
                }
                break;
            }
        };
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("task");
        info!(
            "Task review: starting work on open task '{}' ({}/{} this cycle)",
            name, count + 1, MAX_TASKS_PER_CYCLE
        );
        match crate::task::runner::run_task_until_finished(path.clone(), MAX_ITERATIONS_PER_TASK).await {
            Ok(reply) => {
                info!("Task review: task '{}' completed ({} chars)", name, reply.chars().count());
            }
            Err(e) => {
                error!("Task review: run_task_until_finished failed for '{}': {}", name, e);
            }
        }
        count += 1;
    }
}

/// Async loop: every 10 minutes run a review cycle.
async fn review_loop() {
    loop {
        tokio::time::sleep(Duration::from_secs(REVIEW_INTERVAL_SECS)).await;
        run_review_once().await;
    }
}

/// Spawn the task review thread. Runs every 10 min: close WIP > 30 min as unsuccessful, work on one open task.
pub fn spawn_review_thread() {
    std::thread::spawn(|| {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(r) => r,
            Err(e) => {
                error!("Task review: failed to create tokio runtime: {}", e);
                return;
            }
        };
        info!("Task review: thread spawned (every {} min, WIP timeout {} min)", REVIEW_INTERVAL_SECS / 60, WIP_TIMEOUT_SECS / 60);
        rt.block_on(review_loop());
    });
}
