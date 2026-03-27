//! Ollama integration module
//!
//! Local LLM chat interface using Ollama API.
//! Includes model info (context size) via POST /api/show and cache.
//! Sub-module `models` provides role-based model classification and resolution.

pub(crate) mod model_list_cache;
pub mod models;

use crate::circuit_breaker::CircuitBreaker;
use crate::security;
use crate::{mac_stats_debug, mac_stats_info};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use url::Url;

/// How long successful `POST /api/show` context lookups stay in the in-memory cache.
const MODEL_INFO_CACHE_TTL: Duration = Duration::from_secs(600);

/// Whether an error string often means Ollama was still starting (transport or model not ready yet).
/// Used for one-shot post-start retries in chat and startup warmup — keep narrow to avoid masking real failures.
pub(crate) fn ollama_error_suggests_transient_cold_start(msg: &str) -> bool {
    let m = msg.to_lowercase();
    m.contains("connection refused")
        || m.contains("actively refused")
        || m.contains("failed to connect")
        || m.contains("connection reset")
        || m.contains("broken pipe")
        || m.contains("error sending request")
        || (m.contains("model") && (m.contains("not found") || m.contains("not available")))
}

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
    /// HTTP client timeout in seconds for /api/chat. None = use default (120).
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:11434".to_string(),
            model: "llama2".to_string(),
            api_key: None,
            temperature: None,
            num_ctx: None,
            timeout_secs: None,
        }
    }
}

impl OllamaConfig {
    pub fn validate(&self) -> Result<()> {
        Url::parse(&self.endpoint).context("Invalid Ollama endpoint URL")?;
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
    /// Optional base64-encoded images (for vision models). Ollama API: "images": ["<base64>"].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
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
    /// Send empty array to disable Ollama's native tool-call parsing.
    /// Without this, models with built-in tool support (qwen3, command-r, etc.)
    /// emit tool-call JSON that Ollama tries to parse, causing
    /// "error parsing tool call" failures. We handle tools via text prefixes instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<serde_json::Value>>,
}

/// Chat response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: ChatMessage,
    pub done: bool,
}

/// Ollama API error payload (e.g. {"error": "model not found"}).
#[derive(Debug, Clone, Deserialize)]
pub struct OllamaErrorResponse {
    pub error: String,
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
    /// Context window size in tokens (from num_ctx or model default). Default 8192 if unknown.
    pub context_size_tokens: u32,
}

impl Default for ModelInfo {
    fn default() -> Self {
        Self {
            context_size_tokens: 8192,
        }
    }
}

/// Where the effective context token budget came from (for debug logs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelContextBudgetSource {
    /// Valid cached `/api/show` result (within TTL).
    Cache,
    ParsedFromShow,
    /// `/api/show` succeeded but had no `num_ctx` / `context_length`; used name heuristics.
    HeuristicFromShow,
    /// `/api/show` succeeded but had no context hints; used [`ModelInfo::default`].
    DefaultFromShow,
    /// `/api/show` failed; used name heuristics.
    HeuristicFallback,
    /// `/api/show` failed; used [`ModelInfo::default`].
    DefaultFallback,
}

impl ModelContextBudgetSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cache => "cache",
            Self::ParsedFromShow => "api_show",
            Self::HeuristicFromShow => "heuristic_from_show",
            Self::DefaultFromShow => "default_from_show",
            Self::HeuristicFallback => "heuristic_fallback",
            Self::DefaultFallback => "default_fallback",
        }
    }
}

#[derive(Debug, Clone)]
struct CachedModelInfoEntry {
    info: ModelInfo,
    inserted_at: Instant,
}

/// Cache key: (endpoint, model_name).
fn model_info_cache() -> &'static Mutex<HashMap<(String, String), CachedModelInfoEntry>> {
    static CACHE: OnceLock<Mutex<HashMap<(String, String), CachedModelInfoEntry>>> =
        OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Conservative context estimate from the model tag when `/api/show` omits or fails to supply `num_ctx`.
pub fn estimate_context_tokens_from_model_name(model_name: &str) -> Option<u32> {
    let m = model_name.to_lowercase();
    if m.contains("mistral") || m.contains("qwen") {
        return Some(32768);
    }
    if m.contains("llama") || m.contains("gemma") {
        return Some(8192);
    }
    None
}

fn context_tokens_from_show_or_name(
    response: &serde_json::Value,
    model_name: &str,
) -> (u32, ModelContextBudgetSource) {
    if let Some(n) = parse_context_size_from_show(response) {
        return (n, ModelContextBudgetSource::ParsedFromShow);
    }
    if let Some(n) = estimate_context_tokens_from_model_name(model_name) {
        return (n, ModelContextBudgetSource::HeuristicFromShow);
    }
    let d = ModelInfo::default().context_size_tokens;
    (d, ModelContextBudgetSource::DefaultFromShow)
}

/// Fetch from `/api/show` when possible; on failure use name heuristics or [`ModelInfo::default`].
/// Used for chat context budgeting so a transient Ollama slowdown does not force a tiny 4k window.
pub async fn resolve_model_context_budget(
    endpoint: &str,
    model_name: &str,
    api_key: Option<&str>,
) -> (ModelInfo, ModelContextBudgetSource) {
    match fetch_model_info_for_cache(endpoint, model_name, api_key).await {
        Ok((info, src)) => (info, src),
        Err(e) => {
            mac_stats_debug!(
                "ollama/api",
                "resolve_model_context_budget: show failed for model={}: {}",
                model_name,
                e
            );
            if let Some(n) = estimate_context_tokens_from_model_name(model_name) {
                (
                    ModelInfo {
                        context_size_tokens: n,
                    },
                    ModelContextBudgetSource::HeuristicFallback,
                )
            } else {
                (
                    ModelInfo::default(),
                    ModelContextBudgetSource::DefaultFallback,
                )
            }
        }
    }
}

async fn fetch_model_info_for_cache(
    endpoint: &str,
    model_name: &str,
    api_key: Option<&str>,
) -> Result<(ModelInfo, ModelContextBudgetSource), String> {
    let key = (endpoint.to_string(), model_name.to_string());
    {
        let mut guard = model_info_cache().lock().map_err(|e| e.to_string())?;
        if let Some(entry) = guard.get(&key) {
            if entry.inserted_at.elapsed() < MODEL_INFO_CACHE_TTL {
                return Ok((entry.info.clone(), ModelContextBudgetSource::Cache));
            }
            guard.remove(&key);
        }
    }

    let url = format!("{}/api/show", endpoint.trim_end_matches('/'));
    let body = serde_json::json!({ "name": model_name });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("HTTP client: {}", e))?;

    let mut request = client.post(&url).json(&body);
    // Do not log request/response headers or bodies that may contain credentials.
    if let Some(key) = api_key {
        request = request.header("Authorization", format!("Bearer {}", key));
    }

    let http = request
        .send()
        .await
        .map_err(|e| format!("Show model request failed: {}", e))?;
    let status = http.status();
    let body_text = http
        .text()
        .await
        .map_err(|e| format!("Show model response body: {}", e))?;
    if !status.is_success() {
        return Err(format!(
            "Show model HTTP {} {}",
            status,
            body_text.chars().take(240).collect::<String>()
        ));
    }

    let response: serde_json::Value =
        serde_json::from_str(&body_text).map_err(|e| format!("Show model JSON parse: {}", e))?;

    if let Some(err) = response.get("error").and_then(|e| e.as_str()) {
        if !err.trim().is_empty() {
            return Err(format!("Ollama show: {}", err));
        }
    }

    let (context_size_tokens, src) = context_tokens_from_show_or_name(&response, model_name);
    let info = ModelInfo {
        context_size_tokens,
    };

    {
        let mut guard = model_info_cache().lock().map_err(|e| e.to_string())?;
        guard.insert(
            key,
            CachedModelInfoEntry {
                info: info.clone(),
                inserted_at: Instant::now(),
            },
        );
    }

    Ok((info, src))
}

/// Fetch model info from Ollama POST /api/show and cache it (10-minute TTL per entry).
/// Returns `Err` if the HTTP call fails or Ollama returns an error payload (strict callers, e.g. startup).
pub async fn get_model_info(
    endpoint: &str,
    model_name: &str,
    api_key: Option<&str>,
) -> Result<ModelInfo, String> {
    let (info, _) = fetch_model_info_for_cache(endpoint, model_name, api_key).await?;
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

/// Get cached model info or default (8192). Does not fetch; respects TTL.
#[allow(dead_code)]
pub fn get_model_info_cached(endpoint: &str, model_name: &str) -> ModelInfo {
    let key = (endpoint.to_string(), model_name.to_string());
    if let Ok(guard) = model_info_cache().lock() {
        if let Some(entry) = guard.get(&key) {
            if entry.inserted_at.elapsed() < MODEL_INFO_CACHE_TTL {
                return entry.info.clone();
            }
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
        config.validate()?;
        mac_stats_info!(
            "ollama/api",
            "Ollama: Initializing client with endpoint: {}",
            config.endpoint
        );

        let timeout_secs = config.timeout_secs.unwrap_or(120);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()?;

        Ok(Self { config, client })
    }

    /// Returns a clone of the HTTP client for use after releasing the global lock (e.g. in async paths).
    /// Cloning reqwest::Client is cheap (shared connection pool).
    pub fn http_client(&self) -> reqwest::Client {
        self.client.clone()
    }

    /// Check if Ollama is available
    #[allow(dead_code)] // May be used in future or via direct client access
    pub async fn check_connection(&self) -> Result<bool> {
        if let Err(e) = ollama_http_circuit_allow() {
            mac_stats_debug!(
                "ollama/api",
                "Ollama: connection check skipped (circuit): {}",
                e
            );
            return Ok(false);
        }
        let url = format!("{}/api/tags", self.config.endpoint);
        mac_stats_debug!("ollama/api", "Ollama: Checking connection to {}", url);

        let mut request = self.client.get(&url);

        // Add API key if configured
        if let Ok(Some(api_key)) = self.config.get_api_key() {
            let masked = security::mask_credential(&api_key);
            request = request.header("Authorization", format!("Bearer {}", api_key));
            mac_stats_debug!(
                "ollama/api",
                "Ollama: Using API key for authentication (masked: {})",
                masked
            );
        }

        match request.send().await {
            Ok(response) => {
                let success = response.status().is_success();
                if success {
                    ollama_http_circuit_record_success();
                    mac_stats_info!(
                        "ollama/api",
                        "Ollama: Connection successful to {}",
                        self.config.endpoint
                    );
                } else {
                    ollama_http_circuit_record_failure(response.status().is_server_error());
                    mac_stats_debug!(
                        "ollama/api",
                        "Ollama: Connection failed - HTTP status: {}",
                        response.status()
                    );
                }
                Ok(success)
            }
            Err(e) => {
                mac_stats_debug!("ollama/api", "Ollama: Connection error: {}", e);
                ollama_http_circuit_record_failure(true);
                Ok(false)
            }
        }
    }

    /// Send chat message (async, non-blocking)
    #[allow(dead_code)] // May be used in future or via direct client access
    pub async fn chat(&self, messages: Vec<ChatMessage>) -> Result<ChatResponse> {
        use serde_json;

        let url = format!("{}/api/chat", self.config.endpoint);
        mac_stats_info!("ollama/api", "Ollama: Using endpoint: {}", url);
        mac_stats_info!(
            "ollama/api",
            "Ollama: Streaming is disabled (stream: false)"
        );

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
            tools: Some(vec![]),
        };

        // Log raw request JSON before sending
        let request_json = serde_json::to_string_pretty(&request)
            .unwrap_or_else(|_| "Failed to serialize request".to_string());
        mac_stats_info!(
            "ollama/api",
            "Ollama: Sending HTTP POST to {} with request JSON:\n{}",
            url,
            request_json
        );

        let mut http_request = self.client.post(&url).json(&request);

        // Add API key if configured
        if let Ok(Some(api_key)) = self.config.get_api_key() {
            let masked = security::mask_credential(&api_key);
            http_request = http_request.header("Authorization", format!("Bearer {}", api_key));
            mac_stats_debug!(
                "ollama/api",
                "Ollama: Using API key for chat request (masked: {})",
                masked
            );
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
        mac_stats_info!(
            "ollama/api",
            "Ollama: Received HTTP response in {:?} with response JSON:\n{}",
            duration,
            response_json
        );

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
        ollama_http_circuit_allow().map_err(|e| anyhow::anyhow!(e))?;
        let url = format!("{}/api/tags", self.config.endpoint.trim_end_matches('/'));
        mac_stats_debug!("ollama/api", "Ollama: GET {}", url);
        let mut request = self.client.get(&url);
        if let Ok(Some(api_key)) = self.config.get_api_key() {
            request = request.header("Authorization", format!("Bearer {}", api_key));
            mac_stats_debug!("ollama/api", "Ollama: Using API key for tags");
        }
        let http = match request.send().await {
            Ok(r) => r,
            Err(e) => {
                ollama_http_circuit_record_failure(true);
                return Err(e).context("Failed to request /api/tags");
            }
        };
        let status = http.status();
        if !status.is_success() {
            ollama_http_circuit_record_failure(status.is_server_error());
            let text = http.text().await.unwrap_or_default();
            anyhow::bail!(
                "Ollama /api/tags HTTP {} {}",
                status,
                text.chars().take(240).collect::<String>()
            );
        }
        let response = match http.json::<ListResponse>().await {
            Ok(r) => r,
            Err(e) => {
                ollama_http_circuit_record_failure(false);
                return Err(e).context("Failed to parse /api/tags response");
            }
        };
        ollama_http_circuit_record_success();
        mac_stats_info!(
            "ollama/api",
            "Ollama: list_models_full returned {} models",
            response.models.len()
        );
        Ok(response)
    }

    /// Get Ollama server version (GET /api/version).
    pub async fn get_version(&self) -> Result<VersionResponse> {
        let url = format!("{}/api/version", self.config.endpoint.trim_end_matches('/'));
        mac_stats_debug!("ollama/api", "Ollama: GET {}", url);
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
        let url = format!("{}/api/ps", self.config.endpoint.trim_end_matches('/'));
        mac_stats_debug!("ollama/api", "Ollama: GET {}", url);
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
        let url = format!("{}/api/pull", self.config.endpoint.trim_end_matches('/'));
        let body = serde_json::json!({ "model": model, "stream": stream });
        mac_stats_debug!(
            "ollama/api",
            "Ollama: POST {} model={} stream={}",
            url,
            model,
            stream
        );
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
            let body = response
                .bytes()
                .await
                .context("Failed to read pull response body")?;
            let last_status = parse_pull_ndjson(&body)?;
            mac_stats_info!("ollama/api", "Ollama: pull finished: {}", last_status);
        }
        Ok(())
    }

    /// Delete a model from disk (DELETE /api/delete).
    pub async fn delete_model(&self, model: &str) -> Result<()> {
        let url = format!("{}/api/delete", self.config.endpoint.trim_end_matches('/'));
        let body = serde_json::json!({ "model": model });
        mac_stats_debug!("ollama/api", "Ollama: DELETE {} model={}", url, model);
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
        let url = format!("{}/api/embed", self.config.endpoint.trim_end_matches('/'));
        let req = EmbedRequest {
            model: model.to_string(),
            input,
            truncate,
            dimensions,
            keep_alive: None,
        };
        mac_stats_debug!("ollama/api", "Ollama: POST {} model={}", url, model);
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
        let url = format!("{}/api/chat", self.config.endpoint.trim_end_matches('/'));
        let body = serde_json::json!({
            "model": model,
            "messages": [],
            "stream": false,
            "keep_alive": 0
        });
        mac_stats_debug!("ollama/api", "Ollama: POST {} unload model={}", url, model);
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
        let url = format!(
            "{}/api/generate",
            self.config.endpoint.trim_end_matches('/')
        );
        let mut body = serde_json::json!({
            "model": model,
            "prompt": "",
            "stream": false
        });
        if let Some(ka) = keep_alive {
            body["keep_alive"] = serde_json::Value::String(ka.to_string());
        }
        mac_stats_debug!(
            "ollama/api",
            "Ollama: POST {} load model={} keep_alive={:?}",
            url,
            model,
            keep_alive
        );
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

// --- Per-endpoint HTTP circuit (shared by /api/chat and /api/tags) ---

static OLLAMA_CIRCUIT_DEBUG_FORCE_LOGGED: AtomicBool = AtomicBool::new(false);

/// When `MAC_STATS_DEBUG_FORCE_OPEN_OLLAMA_CIRCUIT` is `1`/`true`/`yes`, Ollama HTTP is blocked
/// and the menu bar shows **Ollama ✕** without needing a real outage (manual QA only).
fn ollama_http_circuit_debug_force_open() -> bool {
    match std::env::var("MAC_STATS_DEBUG_FORCE_OPEN_OLLAMA_CIRCUIT") {
        Ok(v) => {
            let t = v.trim();
            t == "1" || t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("yes")
        }
        Err(_) => false,
    }
}

fn ollama_http_circuit() -> &'static Mutex<CircuitBreaker> {
    static CB: OnceLock<Mutex<CircuitBreaker>> = OnceLock::new();
    CB.get_or_init(|| Mutex::new(CircuitBreaker::new_ollama()))
}

/// Gate Ollama HTTP calls (`/api/chat`, `/api/tags`, etc.).
pub fn ollama_http_circuit_allow() -> Result<(), String> {
    if ollama_http_circuit_debug_force_open() {
        if !OLLAMA_CIRCUIT_DEBUG_FORCE_LOGGED.swap(true, Ordering::SeqCst) {
            mac_stats_info!(
                "circuit",
                "Ollama circuit: MAC_STATS_DEBUG_FORCE_OPEN_OLLAMA_CIRCUIT is set — blocking Ollama HTTP (debug QA only)"
            );
        }
        return Err(
            "Ollama is temporarily unavailable (circuit open, will retry in 30s)".to_string(),
        );
    }
    let mut g = ollama_http_circuit()
        .lock()
        .map_err(|_| "Ollama circuit lock poisoned".to_string())?;
    g.allow_request()
}

pub fn ollama_http_circuit_record_success() {
    if let Ok(mut g) = ollama_http_circuit().lock() {
        g.record_success();
    }
}

pub fn ollama_http_circuit_record_failure(should_trip: bool) {
    if let Ok(mut g) = ollama_http_circuit().lock() {
        g.record_failure(should_trip);
    }
}

/// When true, menu bar may show a short "Ollama ✕" hint (circuit fully open).
pub fn ollama_http_circuit_is_open_for_menu() -> bool {
    if ollama_http_circuit_debug_force_open() {
        return true;
    }
    ollama_http_circuit()
        .lock()
        .ok()
        .is_some_and(|g| g.is_open_blocking())
}

/// Classify chat/transport error strings for circuit trip (infra vs client/model errors).
pub fn ollama_chat_error_should_trip(msg: &str) -> bool {
    if msg.contains("circuit open") {
        return false;
    }
    if msg.starts_with("Ollama error:") {
        return false;
    }
    let lower = msg.to_lowercase();
    if lower.contains("ollama http 4") || lower.contains("http 400") && lower.contains("ollama") {
        return false;
    }
    if lower.contains("http 5")
        || lower.contains("503")
        || lower.contains("502")
        || lower.contains("504")
    {
        return true;
    }
    if lower.contains("failed to send chat request")
        || lower.contains("failed to read response body")
        || lower.contains("stream read error")
        || lower.contains("ollama is busy or unavailable")
        || lower.contains("timed out")
        || lower.contains("timeout")
        || lower.contains("connection refused")
        || lower.contains("connection reset")
    {
        return true;
    }
    false
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

#[cfg(test)]
mod model_context_estimate_tests {
    use super::*;

    #[test]
    fn estimate_mistral_and_qwen() {
        assert_eq!(
            estimate_context_tokens_from_model_name("mistral:7b"),
            Some(32768)
        );
        assert_eq!(
            estimate_context_tokens_from_model_name("Qwen2.5:latest"),
            Some(32768)
        );
    }

    #[test]
    fn estimate_llama_and_gemma() {
        assert_eq!(
            estimate_context_tokens_from_model_name("llama3.2:latest"),
            Some(8192)
        );
        assert_eq!(
            estimate_context_tokens_from_model_name("gemma2:2b"),
            Some(8192)
        );
    }

    #[test]
    fn default_model_info_is_8192() {
        assert_eq!(ModelInfo::default().context_size_tokens, 8192);
    }
}
