//! iOS Stats plugin: on-device language model on the iPhone with llama.cpp.
//!
//! The native side lives in `ios/tauri-plugin-llm/` (Swift). This crate only exposes
//! an async API to the app's Rust code; the web layer has no direct access.

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod error;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
pub use desktop::Llm;
#[cfg(mobile)]
pub use mobile::Llm;

/// Access to the plugin from `App`, `AppHandle` or `Window`.
pub trait LlmExt<R: Runtime> {
    fn llm(&self) -> &Llm<R>;
}

impl<R: Runtime, T: Manager<R>> crate::LlmExt<R> for T {
    fn llm(&self) -> &Llm<R> {
        self.state::<Llm<R>>().inner()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("llm")
        .setup(|app, api| {
            #[cfg(mobile)]
            let llm = mobile::init(app, api)?;
            #[cfg(desktop)]
            let llm = desktop::init(app, api)?;
            app.manage(llm);
            Ok(())
        })
        .build()
}
