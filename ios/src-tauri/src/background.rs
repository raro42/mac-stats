//! Background samples for the saved history (Background App Refresh). On by default; the
//! user can turn them off in the app's Settings, and iOS lets them turn Background App
//! Refresh off per app. The Swift side lives in `BackgroundRefresh.swift` (plugin).

use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_llm::LlmExt;

use crate::chat::ChatState;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistorySettings {
    /// The app's switch.
    pub background: bool,
    /// iOS's Background App Refresh status for the app: `available`, `denied`,
    /// `restricted`, or `unavailable` off iOS.
    pub ios_status: String,
}

fn enabled(state: &ChatState) -> bool {
    state.settings().background_history.unwrap_or(true)
}

async fn current(app: &AppHandle, state: &ChatState) -> HistorySettings {
    let ios_status = app.llm().background_refresh_status().await.unwrap_or_else(|e| {
        eprintln!("background refresh status: {e}");
        "unknown".into()
    });
    HistorySettings { background: enabled(state), ios_status }
}

#[tauri::command]
pub async fn history_settings(app: AppHandle, state: State<'_, ChatState>) -> Result<HistorySettings, AppError> {
    Ok(current(&app, &state).await)
}

#[tauri::command]
pub async fn set_background_history(
    app: AppHandle,
    state: State<'_, ChatState>,
    enabled: bool,
) -> Result<HistorySettings, AppError> {
    state.update_settings(|s| s.background_history = Some(enabled))?;
    app.llm().set_background_refresh(enabled).await?;
    Ok(current(&app, &state).await)
}

/// At startup, tells Swift the current choice (it schedules the first wake-up, or cancels).
pub fn sync(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let on = enabled(&app.state::<ChatState>());
        if let Err(e) = app.llm().set_background_refresh(on).await {
            eprintln!("background refresh setup: {e}");
        }
    });
}
