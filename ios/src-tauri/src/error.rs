//! Errors returned to the web layer. Each one has a stable code from
//! `src/i18n/error-codes.ts` plus optional parameters, and the web layer shows the
//! translated message. Raw system text never reaches the UI: it only goes to the log.

use serde::{Serialize, Serializer};
use serde_json::{Map, Value};

macro_rules! error_codes {
    ($($variant:ident => $code:literal,)*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum ErrorCode {
            $($variant,)*
        }

        impl ErrorCode {
            pub const ALL: &'static [ErrorCode] = &[$(ErrorCode::$variant,)*];

            pub fn as_str(self) -> &'static str {
                match self {
                    $(ErrorCode::$variant => $code,)*
                }
            }
        }
    };
}

error_codes! {
    Internal => "internal",
    Storage => "storage",
    Network => "network",
    NetworkOffline => "network_offline",
    NetworkTimeout => "network_timeout",
    CellularNotAllowed => "cellular_not_allowed",
    EmptyQuestion => "empty_question",
    TooHot => "too_hot",
    AppInBackground => "app_in_background",
    ModelUnknown => "model_unknown",
    ModelNotInstalled => "model_not_installed",
    ModelNotLoaded => "model_not_loaded",
    ModelFileMissing => "model_file_missing",
    ModelLoadFailed => "model_load_failed",
    NotEnoughMemory => "not_enough_memory",
    ContextFailed => "context_failed",
    ChatTemplateFailed => "chat_template_failed",
    TokenizeFailed => "tokenize_failed",
    ConversationTooLong => "conversation_too_long",
    GenerationFailed => "generation_failed",
    DownloadBusy => "download_busy",
    DownloadCancelled => "download_cancelled",
    DownloadFailed => "download_failed",
    NotEnoughStorage => "not_enough_storage",
    ChecksumMismatch => "checksum_mismatch",
    InsecureUrl => "insecure_url",
}

impl ErrorCode {
    pub fn parse(code: &str) -> Option<ErrorCode> {
        ErrorCode::ALL.iter().copied().find(|c| c.as_str() == code)
    }
}

impl Serialize for ErrorCode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct AppError {
    pub code: ErrorCode,
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub params: Map<String, Value>,
}

impl AppError {
    pub fn new(code: ErrorCode) -> Self {
        AppError { code, params: Map::new() }
    }

    /// Adds a parameter for the message. `needed`, `available` and `size` are byte counts.
    pub fn with(mut self, name: &str, value: impl Into<Value>) -> Self {
        self.params.insert(name.into(), value.into());
        self
    }

    pub fn is(&self, code: ErrorCode) -> bool {
        self.code == code
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code.as_str())?;
        if !self.params.is_empty() {
            write!(f, " {}", Value::Object(self.params.clone()))?;
        }
        Ok(())
    }
}

impl From<ErrorCode> for AppError {
    fn from(code: ErrorCode) -> Self {
        AppError::new(code)
    }
}

impl From<tauri_plugin_llm::Error> for AppError {
    fn from(e: tauri_plugin_llm::Error) -> Self {
        let code = e.code().and_then(ErrorCode::parse).unwrap_or(ErrorCode::Internal);
        eprintln!("llm error [{}]: {e}", code.as_str());
        AppError::new(code)
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        eprintln!("storage error: {e}");
        AppError::new(ErrorCode::Storage)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        eprintln!("serialization error: {e}");
        AppError::new(ErrorCode::Internal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The web layer's list (`src/i18n/error-codes.ts`) must match this enum exactly, so
    /// every code the app can return has a translation.
    #[test]
    fn codes_match_the_web_list() {
        let source = include_str!("../../src/i18n/error-codes.ts");
        let start = source.find("ERROR_CODES = [").expect("ERROR_CODES array") + "ERROR_CODES = [".len();
        let end = start + source[start..].find(']').expect("end of ERROR_CODES");
        let web: Vec<&str> = source[start..end]
            .lines()
            .map(|l| l.trim().trim_end_matches(',').trim_matches('"'))
            .filter(|l| !l.is_empty())
            .collect();
        let rust: Vec<&str> = ErrorCode::ALL.iter().map(|c| c.as_str()).collect();
        assert_eq!(web, rust);
    }

    #[test]
    fn serializes_as_code_and_params() {
        let e = AppError::new(ErrorCode::ModelNotInstalled).with("name", "Qwen3.5 2B");
        assert_eq!(
            serde_json::to_value(&e).unwrap(),
            serde_json::json!({ "code": "model_not_installed", "params": { "name": "Qwen3.5 2B" } })
        );
        assert_eq!(serde_json::to_value(AppError::new(ErrorCode::TooHot)).unwrap(), serde_json::json!({ "code": "too_hot" }));
    }

    #[test]
    fn unknown_codes_do_not_parse() {
        assert_eq!(ErrorCode::parse("download_busy"), Some(ErrorCode::DownloadBusy));
        assert_eq!(ErrorCode::parse("nope"), None);
    }
}
