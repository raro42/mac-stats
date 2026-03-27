//! Tauri commands for scheduler UI: list, add (cron or one-shot), remove.

use crate::scheduler::{self, ScheduleAddOutcome};
use std::time::{SystemTime, UNIX_EPOCH};

/// List all schedules with next-run for the Settings UI.
#[tauri::command]
pub fn list_schedules() -> Result<Vec<scheduler::ScheduleForUi>, String> {
    Ok(scheduler::list_schedules_for_ui())
}

/// Recent successful scheduler → Discord deliveries (newest first), for Settings / operator verification.
#[tauri::command]
pub fn list_scheduler_delivery_awareness() -> Result<Vec<scheduler::DeliveryAwarenessEntry>, String>
{
    Ok(scheduler::list_scheduler_delivery_awareness())
}

/// Add a recurring schedule (cron). Id is generated as ui-<unix_ts>.
#[tauri::command]
pub fn add_schedule(
    cron: String,
    task: String,
    reply_to_channel_id: Option<String>,
) -> Result<AddScheduleResult, String> {
    let id = format!(
        "ui-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    );
    match scheduler::add_schedule(id.clone(), cron, task, reply_to_channel_id) {
        Ok(ScheduleAddOutcome::Added) => Ok(AddScheduleResult::Added { id }),
        Ok(ScheduleAddOutcome::AlreadyExists) => Ok(AddScheduleResult::AlreadyExists),
        Err(e) => Err(e),
    }
}

/// Add a one-shot schedule (run once at datetime). Id is generated as ui-<unix_ts>.
#[tauri::command]
pub fn add_schedule_at(
    at: String,
    task: String,
    reply_to_channel_id: Option<String>,
) -> Result<AddScheduleResult, String> {
    let id = format!(
        "ui-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    );
    match scheduler::add_schedule_at(id.clone(), at, task, reply_to_channel_id) {
        Ok(ScheduleAddOutcome::Added) => Ok(AddScheduleResult::Added { id }),
        Ok(ScheduleAddOutcome::AlreadyExists) => Ok(AddScheduleResult::AlreadyExists),
        Err(e) => Err(e),
    }
}

/// Remove a schedule by id.
#[tauri::command]
pub fn remove_schedule(schedule_id: String) -> Result<bool, String> {
    scheduler::remove_schedule_by_id(&schedule_id)
}

#[derive(serde::Serialize)]
#[serde(tag = "status")]
pub enum AddScheduleResult {
    Added { id: String },
    AlreadyExists,
}
