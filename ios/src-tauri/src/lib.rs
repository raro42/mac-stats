mod lab;
mod metrics;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_llm::init())
        .setup(|app| {
            metrics::init(app.handle());
            // Destino de los modelos (se copian con devicectl o se descargan en la fase B).
            let _ = std::fs::create_dir_all(lab::models_dir());
            // Medición automática de modelos (fase A), solo en builds de depuración.
            if cfg!(debug_assertions) {
                if let Ok(spec) = std::env::var("IOS_STATS_BENCH") {
                    tauri::async_runtime::spawn(lab::auto_bench(app.handle().clone(), spec));
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            metrics::metrics_subscribe,
            metrics::metrics_history,
            metrics::device_info,
            lab::lab_models,
            lab::lab_load,
            lab::lab_unload,
            lab::lab_bench,
            lab::lab_generate,
            lab::lab_cancel,
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar iOS Stats");
}
