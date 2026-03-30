//! Tauri command: operator automation pressure snapshot (scheduler + queues + task files).

/// JSON snapshot for Settings / operators. Async: touches Tokio mutexes in Ollama and session queues.
#[tauri::command]
pub async fn get_operator_task_pressure_summary(
) -> Result<crate::operator_task_pressure::OperatorTaskPressureSummary, String> {
    Ok(crate::operator_task_pressure::build_operator_task_pressure_summary().await)
}
