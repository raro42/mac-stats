//! Window-related Tauri commands (e.g. toggle CPU window from chat).

use tauri::AppHandle;

/// Toggle the CPU window (open if closed, close if open). Can be invoked from chat reserved word "--cpu".
#[tauri::command]
pub fn toggle_cpu_window(app: AppHandle) -> Result<(), String> {
    let handle = app.clone();
    app.run_on_main_thread(move || {
        crate::ui::status_bar::toggle_cpu_window(&handle);
    })
    .map_err(|e| e.to_string())
}
