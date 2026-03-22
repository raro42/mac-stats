//! Window-related Tauri commands (e.g. toggle CPU window from chat).

use tauri::AppHandle;

/// Ensure the CPU window is open (creates it if missing).
///
/// If a `cpu` window already exists—visible or hidden—it is closed and then recreated, so the call
/// always ends with a visible window (same as the menu bar path; see `crate::ui::status_bar::toggle_cpu_window`).
///
/// ## Threading (Tauri 1)
///
/// The command body runs off the AppKit main thread; window APIs must run on the main thread, so this
/// uses `tauri::AppHandle::run_on_main_thread`. That call blocks the **command** thread until the closure
/// finishes on the main thread (standard Tauri pattern). The closure only performs a short close/create
/// sequence; the UI event loop keeps processing while work is queued. Invoked from chat reserved word
/// `--cpu`.
///
/// ## From inside the CPU window
///
/// The reserved word runs the same path as the menu bar: any existing `cpu` window is **closed** and, if
/// no window remains, a **new** one is created. Invoking `--cpu` while chat is open inside that window
/// therefore tears down the WebView and loads a fresh CPU window—same intentional semantics as
/// “always end with a visible window,” not an in-place hide.
#[tauri::command]
pub fn toggle_cpu_window(app: AppHandle) -> Result<(), String> {
    let handle = app.clone();
    app.run_on_main_thread(move || {
        crate::ui::status_bar::toggle_cpu_window(&handle);
    })
    .map_err(|e| e.to_string())
}
