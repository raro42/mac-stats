//! Ollama chat transport — send messages to Ollama and return responses.
//!
//! Extracted from `ollama.rs` to keep the file focused on the orchestrator (`answer_with_ollama_and_fetch`).

use serde::Deserialize;
use std::sync::atomic::Ordering;

use crate::commands::ollama_config::{
    get_ollama_client, read_ollama_api_key_from_env_or_config, ChatRequest,
};

/// Merge config defaults with per-request options. Request override wins.
fn merge_chat_options(
    config_temp: Option<f32>,
    config_num_ctx: Option<u32>,
    options_override: Option<crate::ollama::ChatOptions>,
) -> Option<crate::ollama::ChatOptions> {
    let o = options_override.unwrap_or_default();
    let temperature = o.temperature.or(config_temp);
    let num_ctx = o.num_ctx.or(config_num_ctx);
    if temperature.is_some() || num_ctx.is_some() {
        Some(crate::ollama::ChatOptions {
            temperature,
            num_ctx,
        })
    } else {
        None
    }
}

/// Remove consecutive duplicate messages (same role and content) to avoid wasting tokens and confusing the model.
pub(crate) fn deduplicate_consecutive_messages(
    messages: Vec<crate::ollama::ChatMessage>,
) -> Vec<crate::ollama::ChatMessage> {
    let mut out: Vec<crate::ollama::ChatMessage> = Vec::with_capacity(messages.len());
    for msg in messages {
        let is_dup = out
            .last()
            .map(|last| last.role == msg.role && last.content == msg.content)
            .unwrap_or(false);
        if !is_dup {
            out.push(msg);
        }
    }
    out
}

/// Internal: send messages to Ollama and return the chat response.
/// Used by the ollama_chat command and by answer_with_ollama_and_fetch (Discord / agent).
/// When set, `model_override` and `options_override` apply only to this request.
pub async fn send_ollama_chat_messages(
    messages: Vec<crate::ollama::ChatMessage>,
    model_override: Option<String>,
    options_override: Option<crate::ollama::ChatOptions>,
) -> Result<crate::ollama::ChatResponse, String> {
    use tracing::{debug, info};

    let messages = deduplicate_consecutive_messages(messages);

    let (endpoint, model, api_key, config_temp, config_num_ctx, http_client) = {
        let client_guard = get_ollama_client().lock().map_err(|e| e.to_string())?;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| "Ollama not configured".to_string())?;
        (
            client.config.endpoint.clone(),
            client.config.model.clone(),
            client.config.api_key.clone(),
            client.config.temperature,
            client.config.num_ctx,
            client.http_client(),
        )
    };

    let effective_model = model_override.unwrap_or(model);
    let options = merge_chat_options(config_temp, config_num_ctx, options_override);

    let url = format!("{}/api/chat", endpoint.trim_end_matches('/'));
    let chat_request = crate::ollama::ChatRequest {
        model: effective_model,
        messages,
        stream: false,
        options,
        tools: Some(vec![]),
    };

    // Log outgoing request (ping) so logs show full ping-pong with Ollama.
    // In -vv or higher, never truncate.
    const REQUEST_LOG_MAX: usize = 4000;
    let request_json = serde_json::to_string_pretty(&chat_request)
        .unwrap_or_else(|_| "Failed to serialize request".to_string());
    let verbosity = crate::logging::VERBOSITY.load(Ordering::Relaxed);
    if verbosity >= 2 || request_json.len() <= REQUEST_LOG_MAX {
        info!("Ollama → Request (POST /api/chat):\n{}", request_json);
    } else {
        let ellipsed = crate::logging::ellipse(&request_json, REQUEST_LOG_MAX);
        info!(
            "Ollama → Request (POST /api/chat) ({} chars total):\n{}",
            request_json.len(),
            ellipsed
        );
    }

    let api_key_value = api_key
        .as_ref()
        .and_then(|acc| crate::security::get_credential(acc).ok().flatten())
        .or_else(read_ollama_api_key_from_env_or_config);
    // task-001: retry once on timeout or 503, then return user-friendly message
    const RETRY_DELAY_SECS: u64 = 2;
    let mut last_send_err: Option<String> = None;
    for attempt in 0..2 {
        if attempt > 0 {
            tokio::time::sleep(std::time::Duration::from_secs(RETRY_DELAY_SECS)).await;
        }
        let mut http_request = http_client.post(&url).json(&chat_request);
        if let Some(key) = &api_key_value {
            let _masked = crate::security::mask_credential(key);
            http_request = http_request.header("Authorization", format!("Bearer {}", key));
            debug!("Ollama: Using API key for chat request");
        }
        match http_request.send().await {
            Ok(resp) => {
                let status = resp.status();
                let body = resp
                    .text()
                    .await
                    .map_err(|e| format!("Failed to read response body: {}", e))?;
                if status.as_u16() == 503 && attempt < 1 {
                    info!(
                        "Ollama returned 503, retrying in {}s (attempt {})",
                        RETRY_DELAY_SECS,
                        attempt + 1
                    );
                    continue;
                }
                let response: crate::ollama::ChatResponse = match serde_json::from_str(&body) {
                    Ok(r) => r,
                    Err(_) => {
                        if let Ok(err_payload) =
                            serde_json::from_str::<crate::ollama::OllamaErrorResponse>(&body)
                        {
                            return Err(format!("Ollama error: {}", err_payload.error));
                        }
                        if !status.is_success() {
                            return Err(format!("Ollama HTTP {}: {}", status, body.trim()));
                        }
                        return Err(format!(
                            "Ollama returned invalid response (missing message): {}",
                            body.trim()
                        ));
                    }
                };
                if !status.is_success() {
                    let msg = response.message.content.trim();
                    return Err(format!(
                        "Ollama HTTP {}: {}",
                        status,
                        if msg.is_empty() { body.as_str() } else { msg }
                    ));
                }
                // Success: log and return
                let content = &response.message.content;
                let n = content.chars().count();
                const RESPONSE_LOG_MAX: usize = 1000;
                if verbosity >= 2 || n <= RESPONSE_LOG_MAX {
                    info!("Ollama ← Response ({} chars):\n{}", n, content);
                } else {
                    let ellipsed = crate::logging::ellipse(content, RESPONSE_LOG_MAX);
                    info!("Ollama ← Response ({} chars):\n{}", n, ellipsed);
                }
                return Ok(response);
            }
            Err(e) => {
                let err_str = e.to_string();
                last_send_err = Some(err_str.clone());
                let is_timeout = e.is_timeout() || err_str.to_lowercase().contains("timed out");
                if is_timeout && attempt < 1 {
                    info!(
                        "Ollama request timed out, retrying in {}s (attempt {})",
                        RETRY_DELAY_SECS,
                        attempt + 1
                    );
                    continue;
                }
                if is_timeout {
                    return Err("Ollama is busy or unavailable; try again in a moment.".to_string());
                }
                return Err(format!("Failed to send chat request: {}", e));
            }
        }
    }
    Err(last_send_err.unwrap_or_else(|| "No response".to_string()))
}

#[derive(Debug, Deserialize)]
struct OllamaStreamLine {
    #[serde(default)]
    message: OllamaStreamMessage,
    #[serde(default)]
    done: bool,
}
#[derive(Debug, Default, Deserialize)]
struct OllamaStreamMessage {
    #[serde(default)]
    content: Option<String>,
}

/// Same as send_ollama_chat_messages but with stream: true; emits "ollama-chat-chunk" with
/// `{ content: string }` for each delta and returns the full response when done.
pub async fn send_ollama_chat_messages_streaming(
    messages: Vec<crate::ollama::ChatMessage>,
    model_override: Option<String>,
    options_override: Option<crate::ollama::ChatOptions>,
) -> Result<crate::ollama::ChatResponse, String> {
    use crate::state::APP_HANDLE;
    use futures_util::StreamExt;
    use tauri::Manager;
    use tracing::{debug, info};

    let messages = deduplicate_consecutive_messages(messages);

    let (endpoint, model, api_key, config_temp, config_num_ctx, http_client) = {
        let client_guard = get_ollama_client().lock().map_err(|e| e.to_string())?;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| "Ollama not configured".to_string())?;
        (
            client.config.endpoint.clone(),
            client.config.model.clone(),
            client.config.api_key.clone(),
            client.config.temperature,
            client.config.num_ctx,
            client.http_client(),
        )
    };

    let effective_model = model_override.unwrap_or(model);
    let options = merge_chat_options(config_temp, config_num_ctx, options_override);

    let url = format!("{}/api/chat", endpoint.trim_end_matches('/'));
    let chat_request = crate::ollama::ChatRequest {
        model: effective_model.clone(),
        messages: messages.clone(),
        stream: true,
        options,
        tools: Some(vec![]),
    };

    let verbosity = crate::logging::VERBOSITY.load(Ordering::Relaxed);
    if verbosity >= 2 {
        info!("Ollama → Request (POST /api/chat stream=true)");
    }

    let api_key_value = api_key
        .as_ref()
        .and_then(|acc| crate::security::get_credential(acc).ok().flatten())
        .or_else(read_ollama_api_key_from_env_or_config);

    let mut http_request = http_client.post(&url).json(&chat_request);
    if let Some(key) = &api_key_value {
        let _masked = crate::security::mask_credential(key);
        http_request = http_request.header("Authorization", format!("Bearer {}", key));
        debug!("Ollama: Using API key for streaming chat request");
    }

    let resp = http_request
        .send()
        .await
        .map_err(|e| format!("Failed to send chat request: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp
            .text()
            .await
            .unwrap_or_else(|_| String::new());
        if let Ok(err_payload) = serde_json::from_str::<crate::ollama::OllamaErrorResponse>(&body) {
            return Err(format!("Ollama error: {}", err_payload.error));
        }
        return Err(format!("Ollama HTTP {}: {}", status, body.trim()));
    }

    let app_handle = APP_HANDLE.get().ok_or_else(|| "App handle not available for streaming".to_string())?;
    let mut stream = resp.bytes_stream();
    let mut buf = Vec::<u8>::new();
    let mut full_content = String::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("Stream read error: {}", e))?;
        buf.extend_from_slice(&chunk);
        // Process complete lines (NDJSON)
        while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
            let line = std::mem::take(&mut buf);
            let (line_bytes, rest) = line.split_at(pos);
            buf = rest[1..].to_vec();
            let line_str = match std::str::from_utf8(line_bytes) {
                Ok(s) => s.trim(),
                Err(_) => continue,
            };
            if line_str.is_empty() {
                continue;
            }
            let parsed: OllamaStreamLine = match serde_json::from_str(line_str) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if let Some(delta) = parsed.message.content {
                full_content.push_str(&delta);
                let _ = app_handle.emit_all("ollama-chat-chunk", serde_json::json!({ "content": delta }));
            }
            if parsed.done {
                let response = crate::ollama::ChatResponse {
                    message: crate::ollama::ChatMessage {
                        role: "assistant".to_string(),
                        content: full_content.clone(),
                        images: None,
                    },
                    done: true,
                };
                if verbosity >= 2 {
                    let n = full_content.chars().count();
                    info!("Ollama ← Stream done ({} chars)", n);
                }
                return Ok(response);
            }
        }
    }

    // Stream ended without done: true; return what we have
    let response = crate::ollama::ChatResponse {
        message: crate::ollama::ChatMessage {
            role: "assistant".to_string(),
            content: full_content,
            images: None,
        },
        done: true,
    };
    Ok(response)
}

/// Send chat message to Ollama (async, non-blocking)
#[tauri::command]
pub async fn ollama_chat(request: ChatRequest) -> Result<crate::ollama::ChatResponse, String> {
    use tracing::info;

    let request_json = serde_json::to_string_pretty(&request)
        .unwrap_or_else(|_| "Failed to serialize request".to_string());
    info!("Ollama: Chat request JSON:\n{}", request_json);

    send_ollama_chat_messages(request.messages, None, None).await
}
