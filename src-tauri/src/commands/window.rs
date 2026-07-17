//! Window-related Tauri commands (e.g. toggle CPU window from chat).

use tauri::AppHandle;

/// Toggle the CPU window (show/hide). Creates it only if missing.
///
/// Reuses an existing `cpu` WebView when present — hide/show instead of destroy/recreate —
/// so menu-bar and `--cpu` chat reserved-word paths stay fast.
#[tauri::command]
pub fn toggle_cpu_window(app: AppHandle) -> Result<(), String> {
    let handle = app.clone();
    app.run_on_main_thread(move || {
        crate::ui::status_bar::toggle_cpu_window(&handle);
    })
    .map_err(|e| e.to_string())
}
