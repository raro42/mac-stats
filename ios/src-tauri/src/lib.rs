/// Comprobación de extremo a extremo del puente JS → Rust.
#[tauri::command]
fn ping() -> String {
    format!("pong desde Rust ({} / {})", std::env::consts::OS, std::env::consts::ARCH)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![ping])
        .run(tauri::generate_context!())
        .expect("error al iniciar Pulso");
}
