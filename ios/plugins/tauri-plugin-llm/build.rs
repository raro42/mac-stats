// La web no llama al plugin: solo lo usa el Rust de la app, así que no hay comandos
// ni permisos que exponer.
const COMMANDS: &[&str] = &[];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).ios_path("ios").build();

    // Los modelos necesitan más memoria de la que iOS concede por defecto a una app.
    // `tauri ios init` reescribe el archivo de entitlements, así que se reinserta en cada
    // build. Fuera de una build de iOS lanzada por la CLI de Tauri no hace nada.
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rerun-if-env-changed=TAURI_IOS_PROJECT_PATH");
        println!("cargo:rerun-if-env-changed=TAURI_IOS_APP_NAME");
        if let (Ok(project), Ok(app)) = (
            std::env::var("TAURI_IOS_PROJECT_PATH"),
            std::env::var("TAURI_IOS_APP_NAME"),
        ) {
            println!("cargo:rerun-if-changed={project}/{app}_iOS/{app}_iOS.entitlements");
        }
        tauri_plugin::mobile::update_entitlements(|entitlements| {
            entitlements.insert(
                "com.apple.developer.kernel.increased-memory-limit".into(),
                true.into(),
            );
        })
        .expect("no se pudo actualizar el archivo de entitlements");
    }
}
