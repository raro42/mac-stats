//! Typed errors for the agent router (`answer_with_ollama_and_fetch`): stable codes, user-facing
//! messages, aggregate counters, and classification from string/anyhow chains.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use serde::Serialize;

use crate::commands::content_reduction::is_context_overflow_error;
use crate::mac_stats_info;

static ERROR_COUNTS: OnceLock<Mutex<HashMap<String, u64>>> = OnceLock::new();

fn counts_map() -> &'static Mutex<HashMap<String, u64>> {
    ERROR_COUNTS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Increment the counter for this error code (called when a router invocation fails after retries).
pub fn record_error_code(code: &'static str) {
    if let Ok(mut g) = counts_map().lock() {
        *g.entry(code.to_string()).or_insert(0) += 1;
    }
    mac_stats_info!(
        "ollama/chat",
        "OllamaRunError metrics: incremented code={} (snapshot: {})",
        code,
        format_metrics_for_log()
    );
}

fn format_metrics_for_log() -> String {
    let Ok(g) = counts_map().lock() else {
        return "(metrics lock poisoned)".to_string();
    };
    let mut pairs: Vec<_> = g.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
    pairs.sort();
    if pairs.is_empty() {
        "(none)".to_string()
    } else {
        pairs.join(", ")
    }
}

/// Snapshot of per-code failure counts since process start (debug / Tauri).
#[derive(Debug, Clone, Serialize)]
pub struct OllamaRunErrorMetrics {
    pub counts: HashMap<String, u64>,
}

#[tauri::command]
pub fn get_ollama_run_error_metrics() -> OllamaRunErrorMetrics {
    let counts = counts_map().lock().map(|g| g.clone()).unwrap_or_default();
    OllamaRunErrorMetrics { counts }
}

/// Typed router failure with a stable `code` for metrics and branching.
#[derive(Debug, Clone)]
pub enum OllamaRunError {
    Timeout {
        message: String,
    },
    ServiceUnavailable {
        message: String,
    },
    ModelNotFound {
        message: String,
    },
    ToolDenied {
        message: String,
    },
    BrowserSessionLost {
        message: String,
    },
    ContextOverflow {
        message: String,
    },
    InternalError {
        message: String,
    },
    /// Inbound work dropped: event is older than the session abort cutoff (no user-facing text).
    StaleInboundAfterAbort,
}

impl OllamaRunError {
    pub fn code(&self) -> &'static str {
        match self {
            OllamaRunError::Timeout { .. } => "TIMEOUT",
            OllamaRunError::ServiceUnavailable { .. } => "SERVICE_UNAVAILABLE",
            OllamaRunError::ModelNotFound { .. } => "MODEL_NOT_FOUND",
            OllamaRunError::ToolDenied { .. } => "TOOL_DENIED",
            OllamaRunError::BrowserSessionLost { .. } => "BROWSER_SESSION_LOST",
            OllamaRunError::ContextOverflow { .. } => "CONTEXT_OVERFLOW",
            OllamaRunError::InternalError { .. } => "INTERNAL_ERROR",
            OllamaRunError::StaleInboundAfterAbort => "STALE_INBOUND_AFTER_ABORT",
        }
    }

    /// End-user oriented text (Discord, scheduler, CLI).
    /// Original classification input (for re-wrapping when an inner `answer_with_ollama_and_fetch` returns `Err`).
    pub fn raw_detail(&self) -> String {
        match self {
            OllamaRunError::Timeout { message }
            | OllamaRunError::ServiceUnavailable { message }
            | OllamaRunError::ModelNotFound { message }
            | OllamaRunError::ToolDenied { message }
            | OllamaRunError::BrowserSessionLost { message }
            | OllamaRunError::ContextOverflow { message }
            | OllamaRunError::InternalError { message } => message.clone(),
            OllamaRunError::StaleInboundAfterAbort => {
                "dropped stale inbound after abort cutoff".to_string()
            }
        }
    }

    /// Append partial-progress summary on failure (Discord, etc.) when the failure looks like a
    /// timeout or transport stall. Covers [`InternalError`](Self::InternalError) whose text still
    /// mentions timeout after classification gaps, in addition to [`Timeout`](Self::Timeout) /
    /// [`ServiceUnavailable`](Self::ServiceUnavailable).
    pub fn should_attach_partial_progress(&self) -> bool {
        match self {
            OllamaRunError::Timeout { .. } | OllamaRunError::ServiceUnavailable { .. } => true,
            OllamaRunError::StaleInboundAfterAbort => false,
            _ => {
                let blob = format!("{}|{}", self.raw_detail(), self.user_message()).to_lowercase();
                blob.contains("timed out")
                    || blob.contains("timeout")
                    || blob.contains("time out")
                    || blob.contains("deadline exceeded")
            }
        }
    }

    pub fn user_message(&self) -> String {
        match self {
            OllamaRunError::Timeout { message } => {
                let detail = message.trim();
                if detail.is_empty() {
                    "The request timed out — try again or use a smaller model.".to_string()
                } else if detail.contains("Limit: Ollama per-request timeout") {
                    detail.to_string()
                } else {
                    format!(
                        "The request timed out — try again or use a smaller model. ({})",
                        detail
                    )
                }
            }
            OllamaRunError::ServiceUnavailable { message } => {
                let detail = message.trim();
                if detail.is_empty() {
                    "Ollama is temporarily unavailable (503) — try again in a moment.".to_string()
                } else {
                    format!(
                        "Ollama is temporarily unavailable — try again in a moment. ({})",
                        detail
                    )
                }
            }
            OllamaRunError::ModelNotFound { message } => message.clone(),
            OllamaRunError::ToolDenied { message } => message.clone(),
            OllamaRunError::BrowserSessionLost { message } => {
                let detail = message.trim();
                if detail.is_empty() {
                    "The browser session was lost — try your request again.".to_string()
                } else {
                    format!(
                        "The browser session was lost — try your request again. ({})",
                        detail
                    )
                }
            }
            OllamaRunError::ContextOverflow { message } => {
                if let Some(s) =
                    crate::commands::content_reduction::sanitize_ollama_error_for_user(message)
                {
                    s
                } else {
                    "This conversation is too large for the model context — start a new topic or use a larger context model.".to_string()
                }
            }
            OllamaRunError::InternalError { message } => message.clone(),
            OllamaRunError::StaleInboundAfterAbort => String::new(),
        }
    }

    /// Map a router error string (often from `sanitize_ollama_error_for_user` or raw transport text).
    pub fn classify(raw: &str) -> Self {
        let t = raw.trim();
        if is_context_overflow_error(t) {
            return OllamaRunError::ContextOverflow {
                message: t.to_string(),
            };
        }
        let lower = t.to_lowercase();
        if (lower.contains("model '") && lower.contains("not found"))
            || (lower.contains("not found") && lower.contains("available:"))
            || lower.contains("unknown model")
        {
            return OllamaRunError::ModelNotFound {
                message: t.to_string(),
            };
        }
        if lower.contains("503")
            || lower.contains("service unavailable")
            || lower.contains("unavailable; try again")
        {
            return OllamaRunError::ServiceUnavailable {
                message: t.to_string(),
            };
        }
        if is_browser_session_lost_heuristic(&lower, t) {
            return OllamaRunError::BrowserSessionLost {
                message: t.to_string(),
            };
        }
        if lower.contains("timed out")
            || lower.contains("timeout")
            || lower.contains("time out")
            || lower.contains("deadline exceeded")
            || t == "Ollama is busy or unavailable; try again in a moment."
        {
            return OllamaRunError::Timeout {
                message: t.to_string(),
            };
        }
        if lower.contains("unknown tool")
            || lower.contains("tool denied")
            || lower.contains("not allowed")
            || lower.contains("run_js")
                && (lower.contains("disabled") || lower.contains("not enabled"))
            || lower.contains("permission denied")
                && (lower.contains("tool") || lower.contains("browser"))
        {
            return OllamaRunError::ToolDenied {
                message: t.to_string(),
            };
        }
        OllamaRunError::InternalError {
            message: t.to_string(),
        }
    }

    /// Classify from an `anyhow` error chain (tooling / dispatch that uses anyhow).
    pub fn classify_anyhow(err: &anyhow::Error) -> Self {
        let chain: String = err
            .chain()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(" | ");
        Self::classify(&chain)
    }
}

fn is_browser_session_lost_heuristic(lower: &str, raw: &str) -> bool {
    lower.contains("websocket")
        || lower.contains("web socket")
        || lower.contains("cdp")
        || lower.contains("connection reset")
        || (lower.contains("connection refused")
            && (lower.contains("9222") || lower.contains("chrome") || lower.contains("browser")))
        || lower.contains("browser unresponsive")
        || lower.contains("chrome child process")
        || lower.contains("no longer running")
        || lower.contains("session was reset")
        || lower.contains("renderer crashed")
        || lower.contains("cleared session")
        || (lower.contains("browser") && lower.contains("connection error"))
        || (raw.contains("BROWSER_") && raw.contains("CDP retry"))
}

impl std::fmt::Display for OllamaRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.user_message())
    }
}

impl std::error::Error for OllamaRunError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_model_not_found() {
        let e = OllamaRunError::classify("Model 'foo' not found. Available: bar");
        assert_eq!(e.code(), "MODEL_NOT_FOUND");
    }

    #[test]
    fn classify_503() {
        let e = OllamaRunError::classify("Ollama HTTP 503: service unavailable");
        assert_eq!(e.code(), "SERVICE_UNAVAILABLE");
    }

    #[test]
    fn classify_browser() {
        let e = OllamaRunError::classify(
            "Browser unresponsive: Chrome child process 123 is no longer running",
        );
        assert_eq!(e.code(), "BROWSER_SESSION_LOST");
    }

    #[test]
    fn classify_context_overflow() {
        let e = OllamaRunError::classify("Ollama error: context length exceeded");
        assert_eq!(e.code(), "CONTEXT_OVERFLOW");
    }

    #[test]
    fn should_attach_partial_progress_codes() {
        let t = OllamaRunError::Timeout {
            message: String::new(),
        };
        assert!(t.should_attach_partial_progress());
        let s = OllamaRunError::ServiceUnavailable {
            message: String::new(),
        };
        assert!(s.should_attach_partial_progress());
    }

    #[test]
    fn should_attach_partial_progress_internal_timeout_shaped() {
        let e = OllamaRunError::InternalError {
            message: "upstream: request timed out waiting for response".to_string(),
        };
        assert!(e.should_attach_partial_progress());
    }

    #[test]
    fn should_attach_partial_progress_skips_stale_inbound() {
        assert!(!OllamaRunError::StaleInboundAfterAbort.should_attach_partial_progress());
    }
}
