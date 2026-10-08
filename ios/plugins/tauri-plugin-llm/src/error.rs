use serde::{ser::Serializer, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the local model only works on iOS")]
    Unsupported,
    #[cfg(mobile)]
    #[error(transparent)]
    PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),
}

impl Error {
    /// Stable code sent by Swift with `invoke.reject(_, code:)` (see `src/i18n/error-codes.ts`
    /// in the app). `None` for errors that did not come from a coded Swift rejection.
    pub fn code(&self) -> Option<&str> {
        match self {
            #[cfg(mobile)]
            Error::PluginInvoke(tauri::plugin::mobile::PluginInvokeError::InvokeRejected(e)) => e.code.as_deref(),
            _ => None,
        }
    }
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
