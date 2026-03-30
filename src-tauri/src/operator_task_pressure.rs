//! Operator-facing **automation pressure** snapshot (OpenClaw-style `status.tasks` parity).
//!
//! # What this measures
//! - **Scheduler:** counts from `schedules.json` + next-fire horizon (local time). Not whether the
//!   scheduler thread is mid-run.
//! - **Ollama HTTP queue:** global permit + per-key FIFO waiters for routed `/api/chat` calls.
//!   Misses invocations with `skip_ollama_queue`.
//! - **Session keyed queue:** per-conversation full-turn serialization; “busy” means the session
//!   mutex is held (another same-key turn may be waiting). Does not expose waiter depth.
//! - **Task files:** markdown tasks under `~/.mac-stats/task/` by filename status segment.
//! - **Ollama router errors:** cumulative per-code counts since process start (not a sliding window).
//!
//! # What this does **not** measure
//! - Discord gateway queue depth, browser/CDP load, disk I/O, or “true” end-to-end latency.
//! - Historical schedule misses when the app was not running.

use chrono::{SecondsFormat, Utc};
use serde::Serialize;
use std::fs;

use crate::config::Config;
use crate::task::status_from_path;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskFilePressureSnapshot {
    pub open: u32,
    pub wip: u32,
    pub paused: u32,
    pub finished: u32,
    pub unsuccessful: u32,
    pub total_task_files: u32,
}

fn task_file_pressure_snapshot() -> TaskFilePressureSnapshot {
    let mut open = 0u32;
    let mut wip = 0u32;
    let mut paused = 0u32;
    let mut finished = 0u32;
    let mut unsuccessful = 0u32;
    let dir = Config::task_dir();
    if !dir.is_dir() {
        return TaskFilePressureSnapshot {
            open,
            wip,
            paused,
            finished,
            unsuccessful,
            total_task_files: 0,
        };
    }
    let Ok(rd) = fs::read_dir(&dir) else {
        return TaskFilePressureSnapshot {
            open,
            wip,
            paused,
            finished,
            unsuccessful,
            total_task_files: 0,
        };
    };
    for ent in rd.flatten() {
        let p = ent.path();
        if !p.is_file() {
            continue;
        }
        let Some(st) = status_from_path(&p) else {
            continue;
        };
        match st.as_str() {
            "open" => open += 1,
            "wip" => wip += 1,
            "paused" => paused += 1,
            "finished" => finished += 1,
            "unsuccessful" => unsuccessful += 1,
            _ => {}
        }
    }
    let total_task_files = open + wip + paused + finished + unsuccessful;
    TaskFilePressureSnapshot {
        open,
        wip,
        paused,
        finished,
        unsuccessful,
        total_task_files,
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorTaskPressureSummary {
    /// RFC3339 UTC when the snapshot was taken.
    pub collected_at_rfc3339_utc: String,
    pub scheduler: crate::scheduler::SchedulerOperatorSnapshot,
    pub ollama_http_queue: crate::ollama_queue::OllamaHttpQueueSnapshot,
    pub session_keyed_queue: crate::keyed_queue::SessionQueueSnapshot,
    pub task_files: TaskFilePressureSnapshot,
    pub ollama_router_error_counts: crate::commands::ollama_run_error::OllamaRunErrorMetrics,
}

impl OperatorTaskPressureSummary {
    fn ollama_error_total(&self) -> u64 {
        self.ollama_router_error_counts
            .counts
            .values()
            .copied()
            .sum()
    }

    /// Whether background logging should emit a line (non-trivial automation activity).
    pub fn is_non_trivial_for_periodic_log(&self) -> bool {
        let q = &self.ollama_http_queue;
        let waiters_sum: u32 = q.keys.iter().map(|k| k.waiters).sum();
        let ollama_in_flight =
            q.global_available_permits < q.global_total_permits || waiters_sum > 0;
        self.session_keyed_queue.busy_session_keys > 0
            || ollama_in_flight
            || self.task_files.wip > 0
            || self.scheduler.imminent_fire_within_120s_count > 0
            || self.ollama_error_total() > 0
    }

    /// One-line summary for tracing when [`Self::is_non_trivial_for_periodic_log`] is true.
    pub fn compact_log_line(&self) -> String {
        let q = &self.ollama_http_queue;
        let waiters_sum: u32 = q.keys.iter().map(|k| k.waiters).sum();
        let sch = &self.scheduler;
        let tf = &self.task_files;
        let sq = &self.session_keyed_queue;
        format!(
            "sched_entries={} next_in_s={:?} imminent120={} tasks_open={} wip={} sess_busy={}/{} ollama_avail={}/{} ollama_key_waiters={} ollama_err_total={}",
            sch.total_entries,
            sch.seconds_until_next_fire,
            sch.imminent_fire_within_120s_count,
            tf.open,
            tf.wip,
            sq.busy_session_keys,
            sq.tracked_session_keys,
            q.global_available_permits,
            q.global_total_permits,
            waiters_sum,
            self.ollama_error_total()
        )
    }
}

pub async fn build_operator_task_pressure_summary() -> OperatorTaskPressureSummary {
    let ollama_http_queue = crate::ollama_queue::snapshot_http_queue().await;
    let session_keyed_queue = crate::keyed_queue::snapshot_session_queue().await;
    OperatorTaskPressureSummary {
        collected_at_rfc3339_utc: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        scheduler: crate::scheduler::scheduler_operator_snapshot(),
        ollama_http_queue,
        session_keyed_queue,
        task_files: task_file_pressure_snapshot(),
        ollama_router_error_counts: crate::commands::ollama_run_error::get_ollama_run_error_metrics(
        ),
    }
}
