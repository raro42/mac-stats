//! Ollama model management Tauri commands (list, pull, delete, embed, load/unload).

use crate::ollama::{
    EmbedInput, EmbedResponse, ListResponse, OllamaClient, PsResponse, VersionResponse,
};
use serde::{Deserialize, Serialize};

use super::ollama_config::get_ollama_client;

/// List available Ollama models (async, non-blocking)
#[tauri::command]
pub async fn list_ollama_models() -> Result<Vec<String>, String> {
    use tracing::info;

    info!("Ollama: Listing available models (cached /api/tags)...");

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

    let resolved_key = api_key
        .as_ref()
        .and_then(|acc| crate::security::get_credential(acc).ok().flatten());

    let list = crate::ollama::model_list_cache::fetch_tags_cached(
        &endpoint,
        resolved_key.as_deref(),
    )
    .await?;

    let models: Vec<String> = list.models.into_iter().map(|m| m.name).collect();
    info!("Ollama: Model list has {} names", models.len());
    Ok(models)
}

/// List available Ollama models with full details (GET /api/tags).
#[tauri::command]
pub async fn list_ollama_models_full() -> Result<ListResponse, String> {
    let (endpoint, api_key) = {
        let guard = get_ollama_client().lock().map_err(|e| e.to_string())?;
        let client = guard
            .as_ref()
            .ok_or_else(|| "Ollama not configured".to_string())?;
        (
            client.config.endpoint.clone(),
            client.config.api_key.clone(),
        )
    };
    let resolved_key = api_key
        .as_ref()
        .and_then(|acc| crate::security::get_credential(acc).ok().flatten());
    crate::ollama::model_list_cache::fetch_tags_cached(&endpoint, resolved_key.as_deref()).await
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
