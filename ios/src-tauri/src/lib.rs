mod metrics;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            metrics::init(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            metrics::metrics_subscribe,
            metrics::metrics_history,
            metrics::device_info,
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar Pulso");
}
