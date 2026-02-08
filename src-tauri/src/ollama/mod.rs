//! Ollama integration module
//!
//! Local LLM chat interface using Ollama API.
//! Includes model info (context size) via POST /api/show and cache.

use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use url::Url;
use crate::security;

/// Ollama configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub endpoint: String, // e.g., "http://localhost:11434"
    pub model: String,
    pub api_key: Option<String>, // For remote instances
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub num_ctx: Option<u32>,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:11434".to_string(),
            model: "llama2".to_string(),
            api_key: None,
            temperature: None,
            num_ctx: None,
        }
    }
}

impl OllamaConfig {
    pub fn validate(&self) -> Result<()> {
        Url::parse(&self.endpoint)
            .context("Invalid Ollama endpoint URL")?;
        Ok(())
    }

    /// Get API key from Keychain if configured
    #[allow(dead_code)] // Used in OllamaClient methods
    pub fn get_api_key(&self) -> Result<Option<String>> {
        if let Some(ref keychain_account) = self.api_key {
            security::get_credential(keychain_account)
                .context("Failed to retrieve Ollama API key from Keychain")
        } else {
            Ok(None)
        }
    }
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "user", "assistant", "system"
    pub content: String,
}

/// Per-request chat options (temperature, num_ctx). Serializes to Ollama API `options` object.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChatOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_ctx: Option<u32>,
}

/// Chat request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ChatOptions>,
}

/// Chat response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: ChatMessage,
    pub done: bool,
}

// --- GET /api/tags (list models with details) ---

/// Details sub-object for a model in the tags list.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub families: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantization_level: Option<String>,
}

/// Single model entry from GET /api/tags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSummary {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<ModelDetails>,
}

/// Response from GET /api/tags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse {
    #[serde(default)]
    pub models: Vec<ModelSummary>,
}

// --- GET /api/version ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionResponse {
    pub version: String,
}

// --- GET /api/ps (list running models) ---

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PsModelDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub families: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantization_level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsModel {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<PsModelDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_vram: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_length: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsResponse {
    #[serde(default)]
    pub models: Vec<PsModel>,
}

// --- POST /api/embed ---

/// Input for embeddings: single string or list of strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmbedInput {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedRequest {
    pub model: String,
    pub input: EmbedInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncate: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedResponse {
    pub model: String,
    #[serde(default)]
    pub embeddings: Vec<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_eval_count: Option<u32>,
}

/// Model metadata from POST /api/show (context size for prompt fitting).
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// Context window size in tokens (from num_ctx or model default). Default 4096 if unknown.
    pub context_size_tokens: u32,
}

impl Default for ModelInfo {
    fn default() -> Self {
        Self {
            context_size_tokens: 4096,
        }
    }
}

/// Cache key: (endpoint, model_name).
fn model_info_cache() -> &'static Mutex<HashMap<(String, String), ModelInfo>> {
    static CACHE: OnceLock<Mutex<HashMap<(String, String), ModelInfo>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Fetch model info from Ollama POST /api/show and cache it.
/// Returns cached value if present; otherwise fetches and stores.
pub async fn get_model_info(
    endpoint: &str,
    model_name: &str,
    api_key: Option<&str>,
) -> Result<ModelInfo, String> {
    let key = (endpoint.to_string(), model_name.to_string());
    {
        let guard = model_info_cache().lock().map_err(|e| e.to_string())?;
        if let Some(info) = guard.get(&key) {
            return Ok(info.clone());
        }
    }

    let url = format!("{}/api/show", endpoint.trim_end_matches('/'));
    let body = serde_json::json!({ "name": model_name });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("HTTP client: {}", e))?;

    let mut request = client.post(&url).json(&body);
    if let Some(key) = api_key {
        request = request.header("Authorization", format!("Bearer {}", key));
    }

    let response: serde_json::Value = request
        .send()
        .await
        .map_err(|e| format!("Show model request failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Show model response parse: {}", e))?;

    let context_size_tokens = parse_context_size_from_show(&response).unwrap_or(4096);
    let info = ModelInfo {
        context_size_tokens,
    };

    {
        let mut guard = model_info_cache().lock().map_err(|e| e.to_string())?;
        guard.insert(key.clone(), info.clone());
    }

    Ok(info)
}

/// Parse context size from /api/show response: "parameters" string (num_ctx N) or model_info.
fn parse_context_size_from_show(response: &serde_json::Value) -> Option<u32> {
    if let Some(params_str) = response.get("parameters").and_then(|p| p.as_str()) {
        for line in params_str.lines() {
            let line = line.trim();
            if line.starts_with("num_ctx") {
                let rest = line.trim_start_matches("num_ctx").trim();
                if let Ok(n) = rest.parse::<u32>() {
                    return Some(n);
                }
            }
        }
    }
    if let Some(info) = response.get("model_info").and_then(|m| m.as_object()) {
        for (k, v) in info {
            if k.ends_with("context_length") || k == "context_length" {
                if let Some(n) = v.as_u64() {
                    return Some(n as u32);
                }
            }
        }
    }
    None
}

/// Get cached model info or default (4096). Does not fetch.
#[allow(dead_code)]
pub fn get_model_info_cached(endpoint: &str, model_name: &str) -> ModelInfo {
    let key = (endpoint.to_string(), model_name.to_string());
    if let Ok(guard) = model_info_cache().lock() {
        if let Some(info) = guard.get(&key) {
            return info.clone();
        }
    }
    ModelInfo::default()
}

/// Ollama client
pub struct OllamaClient {
    pub config: OllamaConfig,
    #[allow(dead_code)] // Used in methods that may not be called directly
    client: reqwest::Client,
}

impl OllamaClient {
    pub fn new(config: OllamaConfig) -> Result<Self> {
        use tracing::info;
        
        config.validate()?;
        info!("Ollama: Initializing client with endpoint: {}", config.endpoint);
        
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120)) // Long timeout for LLM
            .build()?;

        Ok(Self {
            config,
            client,
        })
    }

    /// Check if Ollama is available
    #[allow(dead_code)] // May be used in future or via direct client access
    pub async fn check_connection(&self) -> Result<bool> {
        use tracing::{debug, info};
        
        let url = format!("{}/api/tags", self.config.endpoint);
        debug!("Ollama: Checking connection to {}", url);
        
        let mut request = self.client.get(&url);
        
        // Add API key if configured
        if let Ok(Some(api_key)) = self.config.get_api_key() {
            let masked = security::mask_credential(&api_key);
            request = request.header("Authorization", format!("Bearer {}", api_key));
            debug!("Ollama: Using API key for authentication (masked: {})", masked);
        }

        match request.send().await {
            Ok(response) => {
                let success = response.status().is_success();
                if success {
                    info!("Ollama: Connection successful to {}", self.config.endpoint);
                } else {
                    debug!("Ollama: Connection failed - HTTP status: {}", response.status());
                }
                Ok(success)
            },
            Err(e) => {
                debug!("Ollama: Connection error: {}", e);
                Ok(false)
            }
        }
    }

    /// Send chat message (async, non-blocking)
    #[allow(dead_code)] // May be used in future or via direct client access
    pub async fn chat(&self, messages: Vec<ChatMessage>) -> Result<ChatResponse> {
        use tracing::{debug, info};
        use serde_json;
        
        let url = format!("{}/api/chat", self.config.endpoint);
        info!("Ollama: Using endpoint: {}", url);
        info!("Ollama: Streaming is disabled (stream: false)");
        
        let options = if self.config.temperature.is_some() || self.config.num_ctx.is_some() {
            Some(ChatOptions {
                temperature: self.config.temperature,
                num_ctx: self.config.num_ctx,
            })
        } else {
            None
        };
        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: messages.clone(),
            stream: false,
            options,
        };

        // Log raw request JSON before sending
        let request_json = serde_json::to_string_pretty(&request)
            .unwrap_or_else(|_| "Failed to serialize request".to_string());
        info!("Ollama: Sending HTTP POST to {} with request JSON:\n{}", url, request_json);

        let mut http_request = self.client.post(&url).json(&request);
        
        // Add API key if configured
        if let Ok(Some(api_key)) = self.config.get_api_key() {
            let masked = security::mask_credential(&api_key);
            http_request = http_request.header("Authorization", format!("Bearer {}", api_key));
            debug!("Ollama: Using API key for chat request (masked: {})", masked);
        }

        let start_time = std::time::Instant::now();
        let response = http_request
            .send()
            .await
            .context("Failed to send chat request to Ollama")?
            .json::<ChatResponse>()
            .await
            .context("Failed to parse Ollama response")?;
        let duration = start_time.elapsed();

        // Log raw response JSON
        let response_json = serde_json::to_string_pretty(&response)
            .unwrap_or_else(|_| "Failed to serialize response".to_string());
        info!("Ollama: Received HTTP response in {:?} with response JSON:\n{}", duration, response_json);

        Ok(response)
    }

    /// List available models (names only). Backward-compatible wrapper around list_models_full.
    #[allow(dead_code)] // May be used in future or via direct client access
    pub async fn list_models(&self) -> Result<Vec<String>> {
        let list = self.list_models_full().await?;
        Ok(list.models.into_iter().map(|m| m.name).collect())
    }

    /// List available models with full details (GET /api/tags).
    pub async fn list_models_full(&self) -> Result<ListResponse> {
        use tracing::{debug, info};
        let url = format!("{}/api/tags", self.config.endpoint.trim_end_matches('/'));
        debug!("Ollama: GET {}", url);
        let mut request = self.client.get(&url);
        if let Ok(Some(api_key)) = self.config.get_api_key() {
            request = request.header("Authorization", format!("Bearer {}", api_key));
            debug!("Ollama: Using API key for tags");
        }
        let response = request
            .send()
            .await
            .context("Failed to request /api/tags")?
            .json::<ListResponse>()
            .await
            .context("Failed to parse /api/tags response")?;
        info!("Ollama: list_models_full returned {} models", response.models.len());
        Ok(response)
    }

    /// Get Ollama server version (GET /api/version).
    pub async fn get_version(&self) -> Result<VersionResponse> {
        use tracing::debug;
        let url = format!("{}/api/version", self.config.endpoint.trim_end_matches('/'));
        debug!("Ollama: GET {}", url);
        let mut request = self.client.get(&url);
        if let Ok(Some(api_key)) = self.config.get_api_key() {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }
        let response = request
            .send()
            .await
            .context("Failed to request /api/version")?
            .json::<VersionResponse>()
            .await
            .context("Failed to parse /api/version response")?;
        Ok(response)
    }

    /// List models currently loaded in memory (GET /api/ps).
    pub async fn list_running_models(&self) -> Result<PsResponse> {
        use tracing::debug;
        let url = format!("{}/api/ps", self.config.endpoint.trim_end_matches('/'));
        debug!("Ollama: GET {}", url);
        let mut request = self.client.get(&url);
        if let Ok(Some(api_key)) = self.config.get_api_key() {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }
        let response = request
            .send()
            .await
            .context("Failed to request /api/ps")?
            .json::<PsResponse>()
            .await
            .context("Failed to parse /api/ps response")?;
        Ok(response)
    }

    /// Pull (download or update) a model (POST /api/pull). If stream is true, consumes NDJSON and returns the last status.
    pub async fn pull_model(&self, model: &str, stream: bool) -> Result<()> {
        use tracing::{debug, info};
        let url = format!("{}/api/pull", self.config.endpoint.trim_end_matches('/'));
        let body = serde_json::json!({ "model": model, "stream": stream });
        debug!("Ollama: POST {} model={} stream={}", url, model, stream);
        let mut request = self.client.post(&url).json(&body);
        if let Ok(Some(api_key)) = self.config.get_api_key() {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }
        let response = request
            .send()
            .await
            .context("Failed to request /api/pull")?;
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Pull failed: {} {}", status, text);
        }
        if stream {
            let body = response.bytes().await.context("Failed to read pull response body")?;
            let last_status = parse_pull_ndjson(&body)?;
            info!("Ollama: pull finished: {}", last_status);
        }
        Ok(())
    }

    /// Delete a model from disk (DELETE /api/delete).
    pub async fn delete_model(&self, model: &str) -> Result<()> {
        use tracing::debug;
        let url = format!("{}/api/delete", self.config.endpoint.trim_end_matches('/'));
        let body = serde_json::json!({ "model": model });
        debug!("Ollama: DELETE {} model={}", url, model);
        let mut request = self.client.delete(&url).json(&body);
        if let Ok(Some(api_key)) = self.config.get_api_key() {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }
        let response = request
            .send()
            .await
            .context("Failed to request /api/delete")?;
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Delete failed: {} {}", status, text);
        }
        Ok(())
    }

    /// Generate embeddings (POST /api/embed).
    pub async fn generate_embeddings(
        &self,
        model: &str,
        input: EmbedInput,
        truncate: Option<bool>,
        dimensions: Option<u32>,
    ) -> Result<EmbedResponse> {
        use tracing::debug;
        let url = format!("{}/api/embed", self.config.endpoint.trim_end_matches('/'));
        let req = EmbedRequest {
            model: model.to_string(),
            input,
            truncate,
            dimensions,
            keep_alive: None,
        };
        debug!("Ollama: POST {} model={}", url, model);
        let mut request = self.client.post(&url).json(&req);
        if let Ok(Some(api_key)) = self.config.get_api_key() {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }
        let response = request
            .send()
            .await
            .context("Failed to request /api/embed")?
            .json::<EmbedResponse>()
            .await
            .context("Failed to parse /api/embed response")?;
        Ok(response)
    }

    /// Unload a model from memory by sending keep_alive: 0 (POST /api/chat with empty messages).
    pub async fn unload_model(&self, model: &str) -> Result<()> {
        use tracing::debug;
        let url = format!("{}/api/chat", self.config.endpoint.trim_end_matches('/'));
        let body = serde_json::json!({
            "model": model,
            "messages": [],
            "stream": false,
            "keep_alive": 0
        });
        debug!("Ollama: POST {} unload model={}", url, model);
        let mut request = self.client.post(&url).json(&body);
        if let Ok(Some(api_key)) = self.config.get_api_key() {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }
        let response = request
            .send()
            .await
            .context("Failed to send unload request")?;
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Unload failed: {} {}", status, text);
        }
        Ok(())
    }

    /// Load (warm) a model into memory with optional keep_alive (e.g. "5m"). Uses POST /api/generate with a minimal prompt.
    pub async fn load_model(&self, model: &str, keep_alive: Option<&str>) -> Result<()> {
        use tracing::debug;
        let url = format!("{}/api/generate", self.config.endpoint.trim_end_matches('/'));
        let mut body = serde_json::json!({
            "model": model,
            "prompt": "",
            "stream": false
        });
        if let Some(ka) = keep_alive {
            body["keep_alive"] = serde_json::Value::String(ka.to_string());
        }
        debug!("Ollama: POST {} load model={} keep_alive={:?}", url, model, keep_alive);
        let mut request = self.client.post(&url).json(&body);
        if let Ok(Some(api_key)) = self.config.get_api_key() {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }
        let response = request
            .send()
            .await
            .context("Failed to send load request")?;
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Load failed: {} {}", status, text);
        }
        Ok(())
    }
}

/// Parse NDJSON from /api/pull response and return the last "status" value.
fn parse_pull_ndjson(body: &[u8]) -> Result<String> {
    let mut last = "unknown".to_string();
    for line in body.split(|&b| b == b'\n') {
        let line = std::str::from_utf8(line).unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(s) = v.get("status").and_then(|x| x.as_str()) {
                last = s.to_string();
            }
        }
    }
    Ok(last)
}
