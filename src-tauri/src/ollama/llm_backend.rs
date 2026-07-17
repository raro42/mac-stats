//! Distinguishes **Ollama** vs **OpenAI-compatible** HTTP servers (e.g. llama.cpp, vLLM).
//!
//! Chat transport uses **automatic format selection** per response (`chat_response_from_api_body`
//! and streaming line parsing). This module adds **probe + inference** so the app knows which
//! family is in use and can show it in Settings.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Which local/remote LLM HTTP dialect the configured endpoint appears to speak.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmBackendKind {
    /// Not yet determined, or endpoints returned nothing we recognize.
    Unknown,
    /// Ollama (`GET /api/version`, Server: ollama, …).
    Ollama,
    /// OpenAI-compatible API (`/v1/models`, `/v1/chat/completions`, …), including llama.cpp server.
    OpenAiCompatible,
}

impl LlmBackendKind {
    /// Short string for logs and `get_ollama_config`.
    pub fn as_label(&self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Ollama => "ollama",
            Self::OpenAiCompatible => "openai_compatible",
        }
    }

    /// Human-readable description for the dashboard.
    pub fn as_description(&self) -> &'static str {
        match self {
            Self::Unknown => "Not detected yet — chat still uses automatic request/response handling.",
            Self::Ollama => "Ollama server detected.",
            Self::OpenAiCompatible => "OpenAI-compatible server detected (e.g. llama.cpp).",
        }
    }
}

/// Infer dialect from a **non-streaming** `POST /api/chat` (or compatible) response body.
pub fn infer_llm_kind_from_chat_response_body(body: &str) -> Option<LlmBackendKind> {
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    if v.get("choices").is_some() {
        return Some(LlmBackendKind::OpenAiCompatible);
    }
    if v.get("message").is_some() && v.get("done").is_some() {
        return Some(LlmBackendKind::Ollama);
    }
    None
}

fn apply_auth(
    mut req: reqwest::RequestBuilder,
    bearer_token: Option<&str>,
) -> reqwest::RequestBuilder {
    if let Some(t) = bearer_token {
        req = req.header("Authorization", format!("Bearer {}", t));
    }
    req
}

/// HTTP probe: `Server` header, then `GET /api/version`, then whether `GET /v1/models` succeeds.
pub async fn probe_llm_backend_kind(
    endpoint: &str,
    client: &reqwest::Client,
    bearer_token: Option<&str>,
) -> LlmBackendKind {
    let ep = endpoint.trim_end_matches('/');

    let v1 = apply_auth(client.get(format!("{}/v1/models", ep)), bearer_token)
        .timeout(Duration::from_secs(5))
        .send()
        .await;

    if let Ok(resp) = &v1 {
        if resp.status().is_success() {
            if let Some(server) = resp.headers().get("server").and_then(|h| h.to_str().ok()) {
                let s = server.to_lowercase();
                if s.contains("llama.cpp") || s.contains("llamacpp") {
                    return LlmBackendKind::OpenAiCompatible;
                }
                if s.contains("ollama") {
                    return LlmBackendKind::Ollama;
                }
            }
        }
    }

    let ver = apply_auth(client.get(format!("{}/api/version", ep)), bearer_token)
        .timeout(Duration::from_secs(5))
        .send()
        .await;

    if let Ok(resp) = ver {
        if resp.status().is_success() {
            if let Ok(text) = resp.text().await {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                    if v.get("version").and_then(|x| x.as_str()).is_some() {
                        return LlmBackendKind::Ollama;
                    }
                }
            }
        }
    }

    if let Ok(resp) = v1 {
        if resp.status().is_success() {
            return LlmBackendKind::OpenAiCompatible;
        }
    }

    LlmBackendKind::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infer_openai_from_choices() {
        let body = r#"{"choices":[{"message":{"role":"assistant","content":"x"}}]}"#;
        assert_eq!(
            infer_llm_kind_from_chat_response_body(body),
            Some(LlmBackendKind::OpenAiCompatible)
        );
    }

    #[test]
    fn infer_ollama_from_message_done() {
        let body = r#"{"message":{"role":"assistant","content":"x"},"done":true}"#;
        assert_eq!(
            infer_llm_kind_from_chat_response_body(body),
            Some(LlmBackendKind::Ollama)
        );
    }
}
