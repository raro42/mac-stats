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

    /// List available models (async, non-blocking)
    #[allow(dead_code)] // May be used in future or via direct client access
    pub async fn list_models(&self) -> Result<Vec<String>> {
        use tracing::{debug, info};
        
        let url = format!("{}/api/tags", self.config.endpoint);
        info!("Ollama: Using endpoint: {}", url);
        debug!("Ollama: Listing models from {}", url);
        
        let mut request = self.client.get(&url);
        
        // Add API key if configured
        if let Ok(Some(api_key)) = self.config.get_api_key() {
            let masked = security::mask_credential(&api_key);
            request = request.header("Authorization", format!("Bearer {}", api_key));
            debug!("Ollama: Using API key for model listing (masked: {})", masked);
        }

        let response: serde_json::Value = request
            .send()
            .await
            .context("Failed to request models from Ollama")?
            .json()
            .await
            .context("Failed to parse models response")?;
        
        // Log raw response JSON
        let response_json = serde_json::to_string_pretty(&response)
            .unwrap_or_else(|_| "Failed to serialize response".to_string());
        info!("Ollama: Received models list HTTP response JSON:\n{}", response_json);
        
        let models: Vec<String> = response
            .get("models")
            .and_then(|m| m.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| m.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        info!("Ollama: Extracted {} models from response", models.len());
        Ok(models)
    }
}
