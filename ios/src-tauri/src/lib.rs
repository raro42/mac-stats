use tauri::Manager;

mod chat;
mod error;
mod lab;
mod language;
mod metrics;
mod selftest;

/// The web UI shows the model lab only in debug builds.
#[tauri::command]
fn debug_build() -> bool {
    cfg!(debug_assertions)
}

/// Automated tests (debug only): with `IOS_STATS_DEMO_PROMPT` the web UI opens the
/// chat and sends that question at startup.
#[tauri::command]
fn debug_demo_prompt() -> Option<String> {
    if cfg!(debug_assertions) {
        std::env::var("IOS_STATS_DEMO_PROMPT").ok()
    } else {
        None
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_llm::init())
        .setup(|app| {
            metrics::init(app.handle());
            // Model destination (copied with devicectl or downloaded in phase B).
            let _ = std::fs::create_dir_all(chat::catalog::models_dir());
            app.manage(chat::ChatState::new(app.handle()));
            // Automatic model benchmark (phase A), debug builds only.
            if cfg!(debug_assertions) {
                if let Ok(spec) = std::env::var("IOS_STATS_BENCH") {
                    tauri::async_runtime::spawn(lab::auto_bench(app.handle().clone(), spec));
                }
                if std::env::var("IOS_STATS_SELFTEST").is_ok() {
                    tauri::async_runtime::spawn(selftest::run(app.handle().clone()));
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            debug_build,
            debug_demo_prompt,
            language::app_language,
            language::set_app_language,
            metrics::metrics_subscribe,
            metrics::metrics_history,
            metrics::device_info,
            lab::lab_models,
            lab::lab_load,
            lab::lab_unload,
            lab::lab_bench,
            lab::lab_generate,
            lab::lab_cancel,
            chat::chat_models,
            chat::chat_select_model,
            chat::chat_status,
            chat::chat_download,
            chat::chat_cancel_download,
            chat::chat_delete_model,
            chat::chat_load,
            chat::chat_unload,
            chat::chat_list,
            chat::chat_get,
            chat::chat_delete,
            chat::chat_cancel,
            chat::chat_send,
        ])
        .run(tauri::generate_context!())
        .expect("error while starting iOS Stats");
}
