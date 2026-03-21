//! Ollama model management Tauri commands (list, pull, delete, embed, load/unload).

use crate::ollama::{EmbedInput, EmbedResponse, ListResponse, OllamaClient, PsResponse, VersionResponse};
use serde::{Deserialize, Serialize};

use super::ollama_config::get_ollama_client;

/// List available Ollama models (async, non-blocking)
#[tauri::command]
pub async fn list_ollama_models() -> Result<Vec<String>, String> {
    use serde_json;
    use tracing::{debug, info};

    info!("Ollama: Listing available models...");

    let (endpoint, api_key) = {
        let client_guard = get_ollama_client().lock().map_err(|e| e.to_string())?;

        let client = client_guard
            .as_ref()
            .ok_or_else(|| "Ollama not configured".to_string())?;

        (
            client.config.endpoint.clone(),
            client.config.api_key.clone(),
        )
    };

    let temp_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let url = format!("{}/api/tags", endpoint);
    info!("Ollama: Using endpoint: {}", url);
    let mut request = temp_client.get(&url);

    if let Some(keychain_account) = &api_key {
        if let Ok(Some(api_key_value)) = crate::security::get_credential(keychain_account) {
            let masked = crate::security::mask_credential(&api_key_value);
            request = request.header("Authorization", format!("Bearer {}", api_key_value));
            debug!(
                "Ollama: Using API key for model listing (masked: {})",
                masked
            );
        }
    }

    let response: serde_json::Value = request
        .send()
        .await
        .map_err(|e| {
            debug!("Ollama: Failed to request models: {}", e);
            format!("Failed to request models: {}", e)
        })?
        .json()
        .await
        .map_err(|e| {
            debug!("Ollama: Failed to parse models response: {}", e);
            format!("Failed to parse models response: {}", e)
        })?;

    let response_json = serde_json::to_string_pretty(&response)
        .unwrap_or_else(|_| "Failed to serialize response".to_string());
    info!(
        "Ollama: Received models list HTTP response JSON:\n{}",
        response_json
    );

    let models: Vec<String> = response
        .get("models")
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    m.get("name")
                        .and_then(|n| n.as_str())
                        .map(|s| s.to_string())
                })
                .collect()
        })
        .unwrap_or_default();

    info!("Ollama: Extracted {} models from response", models.len());
    Ok(models)
}

/// List available Ollama models with full details (GET /api/tags).
#[tauri::command]
pub async fn list_ollama_models_full() -> Result<ListResponse, String> {
    let config = {
        let guard = get_ollama_client().lock().map_err(|e| e.to_string())?;
        let client = guard
            .as_ref()
            .ok_or_else(|| "Ollama not configured".to_string())?;
        client.config.clone()
    };
    let client = OllamaClient::new(config).map_err(|e| e.to_string())?;
    client.list_models_full().await.map_err(|e| e.to_string())
}

/// Get Ollama server version (GET /api/version).
#[tauri::command]
pub async fn get_ollama_version() -> Result<VersionResponse, String> {
    let config = {
        let guard = get_ollama_client().lock().map_err(|e| e.to_string())?;
        let client = guard
            .as_ref()
            .ok_or_else(|| "Ollama not configured".to_string())?;
        client.config.clone()
    };
    let client = OllamaClient::new(config).map_err(|e| e.to_string())?;
    client.get_version().await.map_err(|e| e.to_string())
}

/// List models currently loaded in memory (GET /api/ps).
#[tauri::command]
pub async fn list_ollama_running_models() -> Result<PsResponse, String> {
    let config = {
        let guard = get_ollama_client().lock().map_err(|e| e.to_string())?;
        let client = guard
            .as_ref()
            .ok_or_else(|| "Ollama not configured".to_string())?;
        client.config.clone()
    };
    let client = OllamaClient::new(config).map_err(|e| e.to_string())?;
    client
        .list_running_models()
        .await
        .map_err(|e| e.to_string())
}

/// Pull (download or update) a model (POST /api/pull).
#[tauri::command]
pub async fn pull_ollama_model(model: String, stream: bool) -> Result<(), String> {
    let config = {
        let guard = get_ollama_client().lock().map_err(|e| e.to_string())?;
        let client = guard
            .as_ref()
            .ok_or_else(|| "Ollama not configured".to_string())?;
        client.config.clone()
    };
    let client = OllamaClient::new(config).map_err(|e| e.to_string())?;
    client
        .pull_model(&model, stream)
        .await
        .map_err(|e| e.to_string())
}

/// Delete a model from disk (DELETE /api/delete).
#[tauri::command]
pub async fn delete_ollama_model(model: String) -> Result<(), String> {
    let config = {
        let guard = get_ollama_client().lock().map_err(|e| e.to_string())?;
        let client = guard
            .as_ref()
            .ok_or_else(|| "Ollama not configured".to_string())?;
        client.config.clone()
    };
    let client = OllamaClient::new(config).map_err(|e| e.to_string())?;
    client.delete_model(&model).await.map_err(|e| e.to_string())
}

/// Options for embedding generation.
#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaEmbedOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncate: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<u32>,
}

/// Generate embeddings (POST /api/embed). Input can be a single string or array of strings.
#[tauri::command]
pub async fn ollama_embeddings(
    model: String,
    input: serde_json::Value,
    options: Option<OllamaEmbedOptions>,
) -> Result<EmbedResponse, String> {
    let embed_input = match input {
        serde_json::Value::String(s) => EmbedInput::Single(s),
        serde_json::Value::Array(arr) => {
            let strings: Vec<String> = arr
                .into_iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
            EmbedInput::Multiple(strings)
        }
        _ => return Err("input must be a string or array of strings".to_string()),
    };
    let (truncate, dimensions) = options
        .map(|o| (o.truncate, o.dimensions))
        .unwrap_or((None, None));
    let config = {
        let guard = get_ollama_client().lock().map_err(|e| e.to_string())?;
        let client = guard
            .as_ref()
            .ok_or_else(|| "Ollama not configured".to_string())?;
        client.config.clone()
    };
    let client = OllamaClient::new(config).map_err(|e| e.to_string())?;
    client
        .generate_embeddings(&model, embed_input, truncate, dimensions)
        .await
        .map_err(|e| e.to_string())
}

/// Unload a model from memory (keep_alive: 0).
#[tauri::command]
pub async fn unload_ollama_model(model: String) -> Result<(), String> {
    let config = {
        let guard = get_ollama_client().lock().map_err(|e| e.to_string())?;
        let client = guard
            .as_ref()
            .ok_or_else(|| "Ollama not configured".to_string())?;
        client.config.clone()
    };
    let client = OllamaClient::new(config).map_err(|e| e.to_string())?;
    client.unload_model(&model).await.map_err(|e| e.to_string())
}

/// Load (warm) a model into memory. Optional keep_alive e.g. "5m".
#[tauri::command]
pub async fn load_ollama_model(model: String, keep_alive: Option<String>) -> Result<(), String> {
    let config = {
        let guard = get_ollama_client().lock().map_err(|e| e.to_string())?;
        let client = guard
            .as_ref()
            .ok_or_else(|| "Ollama not configured".to_string())?;
        client.config.clone()
    };
    let client = OllamaClient::new(config).map_err(|e| e.to_string())?;
    client
        .load_model(&model, keep_alive.as_deref())
        .await
        .map_err(|e| e.to_string())
}
