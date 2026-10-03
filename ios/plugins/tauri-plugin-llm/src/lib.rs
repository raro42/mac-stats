//! Plugin de iOS Stats: modelo de lenguaje local en el iPhone con llama.cpp.
//!
//! La parte nativa está en `ios/tauri-plugin-llm/` (Swift). Este crate solo expone
//! una API asíncrona para el Rust de la app; la web no tiene acceso directo.

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

/// Acceso al plugin desde `App`, `AppHandle` o `Window`.
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
