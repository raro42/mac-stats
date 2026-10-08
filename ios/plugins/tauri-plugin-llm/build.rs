// The web layer does not call the plugin: only the app's Rust code uses it, so there are
// no commands or permissions to expose.
const COMMANDS: &[&str] = &[];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).ios_path("ios").build();

    // Models need more memory than iOS grants an app by default.
    // `tauri ios init` rewrites the entitlements file, so the key is re-inserted on every
    // build. Outside an iOS build started by the Tauri CLI this does nothing.
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
        .expect("failed to update the entitlements file");
    }
}
