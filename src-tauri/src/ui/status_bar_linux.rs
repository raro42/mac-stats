//! Linux window UI.
//!
//! There is no macOS status item here. The CPU window is the app surface.

use tauri::utils::config::{BackgroundThrottlingPolicy, Color};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::config::Config;
use crate::logging::write_structured_log;
use crate::metrics::SystemMetrics;

#[allow(unused_imports)]
use crate::{debug1, debug2, debug3};

/// Build status text from metrics. Same layout as the Mac menu bar string.
pub fn build_status_text(metrics: &SystemMetrics) -> String {
    if Config::menu_bar_compact() {
        let temp = crate::state::TEMP_CACHE
            .try_lock()
            .ok()
            .and_then(|g| g.as_ref().map(|(t, _)| *t))
            .filter(|t| *t > 0.0);
        let cpu = metrics.cpu.round() as i32;
        let ssd = metrics.disk.round() as i32;
        return match temp {
            Some(t) => format!("CPU\tSSD\tT\n{cpu}%\t{ssd}%\t{:.0}°", t.round()),
            None => format!("CPU\tSSD\n{cpu}%\t{ssd}%"),
        };
    }
    let label_line = "CPU\tGPU\tRAM\tSSD".to_string();
    let value_line = format!(
        "{:.0}%\t{:.0}%\t{:.0}%\t{:.0}%",
        metrics.cpu.round() as i32,
        metrics.gpu.round() as i32,
        metrics.ram.round() as i32,
        metrics.disk.round() as i32
    );
    format!("{label_line}\n{value_line}")
}

pub fn process_menu_bar_update() {}

pub fn setup_status_item() {
    debug1!("Linux: no menu bar status item");
}

/// CPU window control: closes any visible `cpu` window, or creates one.
pub fn toggle_cpu_window(app_handle: &AppHandle) {
    if let Some(window) = app_handle.get_webview_window("cpu") {
        let is_visible = window.is_visible().unwrap_or(false);
        if is_visible {
            debug1!("CPU window is visible, destroying it");
            save_cpu_window_geometry(&window);
            let _ = window.destroy();
        } else {
            save_cpu_window_geometry(&window);
            let _ = window.destroy();
            create_cpu_window(app_handle);
        }
    } else {
        debug1!("CPU window doesn't exist, creating it");
        create_cpu_window(app_handle);
    }
}

fn save_cpu_window_geometry(window: &tauri::WebviewWindow) {
    use crate::config::{Config, CpuWindowGeometry};
    let Ok(pos) = window.outer_position() else {
        return;
    };
    let Ok(size) = window.inner_size() else {
        return;
    };
    let scale = window.scale_factor().unwrap_or(1.0).max(0.5);
    let geo = CpuWindowGeometry {
        x: (pos.x as f64) / scale,
        y: (pos.y as f64) / scale,
        width: (size.width as f64) / scale,
        height: (size.height as f64) / scale,
    };
    if let Err(e) = Config::set_cpu_window_geometry(&geo) {
        debug2!("Failed to save CPU window geometry: {}", e);
    }
}

/// Create the CPU details window.
pub fn create_cpu_window(app_handle: &tauri::AppHandle) {
    debug1!("Creating CPU window...");
    write_structured_log(
        "ui/status_bar_linux.rs",
        "create_cpu_window ENTRY",
        &serde_json::json!({}),
        "I",
    );

    let decorations = Config::get_window_decorations();
    let cpu_url = format!("cpu.html?v={}", env!("CARGO_PKG_VERSION"));
    let saved = Config::cpu_window_geometry();
    let (default_w, default_h) = if Config::cpu_window_compact() {
        (520.0, 560.0)
    } else {
        (820.0, 995.0)
    };
    let (w, h) = saved
        .map(|g| (g.width, g.height))
        .unwrap_or((default_w, default_h));

    // Prefer software compositing when the compositor path pegs a WebKit core (#14).
    // Safe no-op if already set by the operator; WKWebView on macOS ignores this.
    if std::env::var_os("WEBKIT_DISABLE_COMPOSITING_MODE").is_none() {
        // SAFETY: set before the first WebView is created in this process path.
        unsafe {
            std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        }
    }

    // Opaque fill; Suspend is macOS 14+ (no-op on Linux). Keeps parity with macOS (#14).
    let cpu_window =
        WebviewWindowBuilder::new(app_handle, "cpu", WebviewUrl::App(cpu_url.into()))
            .title("mac-stats · glad you're here")
            .visible(true)
            .inner_size(w, h)
            .resizable(true)
            .always_on_top(false)
            .decorations(decorations)
            .transparent(false)
            .background_color(Color(242, 242, 246, 255))
            .background_throttling(BackgroundThrottlingPolicy::Suspend)
            .build();

    match cpu_window {
        Ok(window) => {
            debug1!("CPU window created successfully");
            if let Some(g) = saved {
                use tauri::{LogicalPosition, Position};
                let _ = window.set_position(Position::Logical(LogicalPosition::new(g.x, g.y)));
            }
            use crate::state::PROCESS_CACHE;
            if let Ok(mut cache) = PROCESS_CACHE.try_lock() {
                *cache = None;
            }
            use crate::state::LAST_CPU_DETAILS_CALL;
            if let Ok(mut last_call) = LAST_CPU_DETAILS_CALL.try_lock() {
                *last_call = None;
            }
            let _ = window.show();
            let _ = window.set_focus();
            let _ = window.unminimize();

            let window_for_close = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    save_cpu_window_geometry(&window_for_close);
                    let _ = window_for_close.destroy();
                    debug1!("CPU window close requested — destroyed");
                }
            });
        }
        Err(e) => {
            debug1!("ERROR: Failed to create CPU window: {:?}", e);
        }
    }
}
