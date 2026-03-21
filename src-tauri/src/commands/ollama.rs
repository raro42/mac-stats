//! Ollama Tauri commands

use tauri::Manager;

use crate::ollama::{ChatMessage, OllamaClient, OllamaConfig};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::pin::Pin;
use std::process::Command;
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use std::sync::OnceLock;

use crate::commands::ollama_models::{
    delete_ollama_model, get_ollama_version, list_ollama_models, list_ollama_models_full,
    list_ollama_running_models, load_ollama_model, ollama_embeddings, pull_ollama_model,
    unload_ollama_model,
};
use crate::commands::redmine_helpers::{
    extract_redmine_time_entries_summary_for_reply, extract_ticket_id,
    grounded_redmine_time_entries_failure_reply, is_grounded_redmine_time_entries_blocked_reply,
    is_redmine_review_or_summarize_only, is_redmine_time_entries_request,
    question_explicitly_requests_json, redmine_direct_fallback_hint,
    redmine_request_for_routing, redmine_time_entries_range,
};
use crate::commands::compaction::{
    compact_conversation_history, COMPACTION_THRESHOLD, MIN_CONVERSATIONAL_FOR_COMPACTION,
};
use crate::commands::ollama_memory::{
    load_memory_block_for_request, load_soul_content, search_memory_for_request,
};
use crate::commands::perplexity_helpers::{
    build_perplexity_news_tool_suffix, build_perplexity_verbose_summary, is_likely_article_like_result,
    is_news_query, search_perplexity_with_news_fallback,
};
use crate::commands::tool_parsing::{
    normalize_browser_tool_arg, normalize_inline_tool_sequences, parse_all_tools_from_response,
    parse_fetch_url_from_response, parse_one_tool_at_line, parse_python_script_from_response,
    parse_tool_from_response, truncate_search_query_arg, MAX_BROWSER_TOOLS_PER_RUN,
};

/// Tool instructions appended to soul for non-agent chat (code execution + FETCH_URL).
const NON_AGENT_TOOL_INSTRUCTIONS: &str = "\n\nYou are a general purpose AI. If you are asked for actual data like day or weather information, or flight information or stock information. Then we need to compile that information using specially crafted clients for doing so. You will put \"[variable-name]\" into the answer to signal that we need to go another step and ask an agent to fulfil the answer.\n\nWhenever asked with \"[variable-name]\", you must provide a javascript snippet to be executed in the browser console to retrieve that information. Mark the answer to be executed as javascript. Do not put any other words around it. Do not insert formatting. Only return the code to be executed. This is needed for the next AI to understand and execute the same. When answering, use the role: code-assistant in the response. When you return executable code:\n- Start the response with: ROLE=code-assistant\n- On the next line, output ONLY executable JavaScript\n- Do not add explanations or formatting\n\nFor web pages: To fetch a page and use its content (e.g. \"navigate to X and get Y\"), reply with exactly one line: FETCH_URL: <full URL> (e.g. FETCH_URL: https://www.example.com). The app will fetch the page and give you the text; then answer the user based on that.";

/// Default system prompt for non-agent Ollama chat: soul (from file or bundled) + tool instructions.
pub fn default_non_agent_system_prompt() -> String {
    let soul = load_soul_content();
    format!("{}{}", soul, NON_AGENT_TOOL_INSTRUCTIONS)
}

/// Tauri command: return the default system prompt (soul + tools) for non-agent Ollama chat.
/// Used by the frontend when no custom system prompt is set (e.g. for legacy ollama_chat message building).
#[tauri::command]
pub fn get_default_ollama_system_prompt() -> String {
    default_non_agent_system_prompt()
}

// Global Ollama client (in production, use proper state management)
pub(crate) fn get_ollama_client() -> &'static Mutex<Option<OllamaClient>> {
    static OLLAMA_CLIENT: OnceLock<Mutex<Option<OllamaClient>>> = OnceLock::new();
    OLLAMA_CLIENT.get_or_init(|| Mutex::new(None))
}

/// Return the configured default Ollama model name, if any. Used so the model can answer "which model are you?" accurately.
pub fn get_default_ollama_model_name() -> Option<String> {
    let guard = get_ollama_client().lock().ok()?;
    let client = guard.as_ref()?;
    Some(client.config.model.clone())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaConfigRequest {
    pub endpoint: String,
    pub model: String,
    pub api_key_keychain_account: Option<String>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub num_ctx: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
}

/// Configure Ollama connection
#[tauri::command]
pub fn configure_ollama(config: OllamaConfigRequest) -> Result<(), String> {
    use serde_json;
    use tracing::{debug, info};

    // Log raw config JSON
    let config_json = serde_json::to_string_pretty(&config)
        .unwrap_or_else(|_| "Failed to serialize config".to_string());
    info!("Ollama: Configuration request JSON:\n{}", config_json);

    let ollama_config = OllamaConfig {
        endpoint: config.endpoint.clone(),
        model: config.model.clone(),
        api_key: config.api_key_keychain_account.clone(),
        temperature: config.temperature,
        num_ctx: config.num_ctx,
        timeout_secs: Some(crate::config::Config::ollama_chat_timeout_secs()),
    };

    ollama_config.validate().map_err(|e| {
        debug!("Ollama: Configuration validation failed: {}", e);
        e.to_string()
    })?;

    let endpoint = config.endpoint.clone();
    info!("Ollama: Using endpoint: {}", endpoint);

    let client = OllamaClient::new(ollama_config).map_err(|e| {
        debug!("Ollama: Failed to create client: {}", e);
        e.to_string()
    })?;

    *get_ollama_client().lock().map_err(|e| e.to_string())? = Some(client);

    info!(
        "Ollama: Configuration successful with endpoint: {}",
        endpoint
    );
    Ok(())
}

/// Response for get_ollama_config (endpoint and model only; no API key).
#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaConfigResponse {
    pub endpoint: String,
    pub model: String,
}

/// Return current Ollama endpoint and model if configured. Used by Settings UI to pre-fill the form.
#[tauri::command]
pub fn get_ollama_config() -> Option<OllamaConfigResponse> {
    let guard = get_ollama_client().lock().ok()?;
    let client = guard.as_ref()?;
    Some(OllamaConfigResponse {
        endpoint: client.config.endpoint.clone(),
        model: client.config.model.clone(),
    })
}

/// List model names at an arbitrary endpoint (GET /api/tags). Does not require Ollama to be configured.
/// Used by Settings UI to populate the model dropdown before or after configuring.
#[tauri::command]
pub async fn list_ollama_models_at_endpoint(endpoint: String) -> Result<Vec<String>, String> {
    use tracing::debug;

    let endpoint = endpoint.trim().trim_end_matches('/').to_string();
    if endpoint.is_empty() {
        return Err("Endpoint URL is required".to_string());
    }
    let url = format!("{}/api/tags", endpoint);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    let response: serde_json::Value = client
        .get(&url)
        .send()
        .await
        .map_err(|e| {
            debug!("Ollama: list_ollama_models_at_endpoint failed: {}", e);
            format!("Failed to request models: {}", e)
        })?
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
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
    Ok(models)
}

/// Check Ollama connection (async, non-blocking)
#[tauri::command]
pub async fn check_ollama_connection() -> Result<bool, String> {
    use tracing::{debug, info};

    // Clone the client config to avoid holding the lock across await
    let client_config = {
        let client_guard = get_ollama_client().lock().map_err(|e| e.to_string())?;

        if let Some(ref client) = *client_guard {
            Some((
                client.config.endpoint.clone(),
                client.config.model.clone(),
                client.config.api_key.clone(),
            ))
        } else {
            debug!("Ollama: Client not configured");
            return Ok(false);
        }
    };

    if let Some((endpoint, _model, api_key)) = client_config {
        info!("Ollama: Checking connection to endpoint: {}", endpoint);

        // Create a temporary client for this check
        let temp_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let url = format!("{}/api/tags", endpoint);
        let mut request = temp_client.get(&url);
        // Do not log request/response headers or bodies that may contain credentials.
        // Add API key if configured
        if let Some(keychain_account) = &api_key {
            if let Ok(Some(api_key_value)) = crate::security::get_credential(keychain_account) {
                request = request.header("Authorization", format!("Bearer {}", api_key_value));
            }
        }

        let result = request
            .send()
            .await
            .map(|resp| resp.status().is_success())
            .unwrap_or(false);

        if result {
            info!("Ollama: Connection successful");
        } else {
            debug!("Ollama: Connection failed (endpoint not reachable)");
        }
        Ok(result)
    } else {
        Ok(false)
    }
}

/// Called at app startup so the Ollama agent is available for Discord, scheduler, and CPU window
/// without requiring the user to open the CPU window first. If the client is not yet configured,
/// configures with default endpoint and auto-detects the first available model from Ollama.
/// Also builds and caches a ModelCatalog so agents can resolve model_role at load time.
pub async fn ensure_ollama_agent_ready_at_startup() {
    use tracing::{debug, info};

    const DEFAULT_ENDPOINT: &str = "http://localhost:11434";

    let already_configured = {
        let guard = match get_ollama_client().lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        guard.is_some()
    };

    if !already_configured {
        info!(
            "Ollama agent: not configured at startup, detecting models from {}",
            DEFAULT_ENDPOINT
        );
        let model = detect_first_model(DEFAULT_ENDPOINT, None).await;
        info!("Ollama agent: using model '{}'", model);
        let default = OllamaConfigRequest {
            endpoint: DEFAULT_ENDPOINT.to_string(),
            model,
            api_key_keychain_account: None,
            temperature: None,
            num_ctx: None,
        };
        if let Err(e) = configure_ollama(default) {
            debug!(
                "Ollama agent: default config failed (endpoint may be down): {}",
                e
            );
            return;
        }
    }

    match check_ollama_connection().await {
        Ok(true) => {
            info!("Ollama agent: ready at startup (endpoint reachable)");
            let (endpoint, model, api_key_account) = {
                let guard = match get_ollama_client().lock() {
                    Ok(g) => g,
                    Err(_) => return,
                };
                match guard.as_ref() {
                    Some(c) => (
                        c.config.endpoint.clone(),
                        c.config.model.clone(),
                        c.config.api_key.clone(),
                    ),
                    None => return,
                }
            };
            let api_key = api_key_account
                .as_ref()
                .and_then(|acc| crate::security::get_credential(acc).ok().flatten());
            if let Ok(info) =
                crate::ollama::get_model_info(&endpoint, &model, api_key.as_deref()).await
            {
                info!(
                    "Ollama agent: model {} context size {} tokens",
                    model, info.context_size_tokens
                );
            }

            // Build model catalog from full model list and cache it for agent model resolution
            build_and_cache_model_catalog(&endpoint, api_key.as_deref()).await;
        }
        Ok(false) => {
            debug!("Ollama agent: endpoint not reachable at startup (will retry when used)")
        }
        Err(e) => debug!("Ollama agent: startup check failed: {}", e),
    }
}

/// Fetch the full model list from Ollama, build a ModelCatalog, and cache it globally.
/// Subsequent calls to load_agents() will use this catalog to resolve model_role fields.
async fn build_and_cache_model_catalog(endpoint: &str, api_key: Option<&str>) {
    use tracing::{info, warn};

    let url = format!("{}/api/tags", endpoint.trim_end_matches('/'));
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            warn!("ModelCatalog: failed to create HTTP client: {}", e);
            return;
        }
    };
    let mut req = client.get(&url);
    if let Some(key) = api_key {
        req = req.header("Authorization", format!("Bearer {}", key));
    }
    let resp = match req.send().await {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => {
            warn!("ModelCatalog: /api/tags returned {}", r.status());
            return;
        }
        Err(e) => {
            warn!("ModelCatalog: /api/tags request failed: {}", e);
            return;
        }
    };
    let list: crate::ollama::ListResponse = match resp.json().await {
        Ok(l) => l,
        Err(e) => {
            warn!("ModelCatalog: failed to parse /api/tags: {}", e);
            return;
        }
    };

    let catalog = crate::ollama::models::ModelCatalog::from_model_list(&list.models);
    info!(
        "ModelCatalog: cached {} classified models for agent model resolution",
        catalog.models.len()
    );
    crate::ollama::models::set_global_catalog(catalog);

    // Trigger initial agent load to resolve models and log the results at startup
    let agents = crate::agents::load_agents();
    if !agents.is_empty() {
        let summary: Vec<String> = agents
            .iter()
            .map(|a| {
                let label = a.slug.as_deref().unwrap_or(&a.name);
                let model = a.model.as_deref().unwrap_or("(default)");
                let role = a.model_role.as_deref().unwrap_or("(none)");
                format!("{}: {} [role={}]", label, model, role)
            })
            .collect();
        info!("Startup model assignments: {}", summary.join(", "));
    }
}

/// Query GET /api/tags and return the first model name, or "llama3.2" as a fallback.
async fn detect_first_model(endpoint: &str, api_key: Option<&str>) -> String {
    // OLLAMA_MODEL env var or .config.env override
    if let Some(override_model) = read_ollama_model_override() {
        tracing::info!("Ollama agent: using model override '{}'", override_model);
        return override_model;
    }
    let url = format!("{}/api/tags", endpoint.trim_end_matches('/'));
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(_) => return "llama3.2".to_string(),
    };
    let mut req = client.get(&url);
    if let Some(key) = api_key {
        req = req.header("Authorization", format!("Bearer {}", key));
    }
    let resp = match req.send().await {
        Ok(r) if r.status().is_success() => r,
        _ => return "llama3.2".to_string(),
    };
    match resp.json::<crate::ollama::ListResponse>().await {
        Ok(list) if !list.models.is_empty() => {
            tracing::info!(
                "Ollama agent: {} model(s) available: {}",
                list.models.len(),
                list.models
                    .iter()
                    .map(|m| m.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            // Prefer local models; use cloud only as fallback (cloud models require ollama.com auth).
            let local = list
                .models
                .iter()
                .find(|m| !crate::ollama::models::is_cloud_model(&m.name));
            let chosen = local
                .map(|m| m.name.clone())
                .unwrap_or_else(|| list.models[0].name.clone());
            if local.is_none() && list.models.len() > 1 {
                tracing::debug!(
                    "Ollama agent: no local model found, using cloud fallback '{}'",
                    chosen
                );
            }
            chosen
        }
        _ => "llama3.2".to_string(),
    }
}

/// Read OLLAMA_API_KEY from env or .config.env. Used when Ollama requires auth (e.g. remote server)
/// so Discord/scheduler/UI all send the key without requiring Keychain in every context.
fn read_ollama_api_key_from_env_or_config() -> Option<String> {
    if let Ok(v) = std::env::var("OLLAMA_API_KEY") {
        let v = v.trim().to_string();
        if !v.is_empty() {
            return Some(v);
        }
    }
    let paths = [
        std::env::current_dir().ok().map(|d| d.join(".config.env")),
        std::env::current_dir()
            .ok()
            .map(|d| d.join("src-tauri").join(".config.env")),
        std::env::var("HOME").ok().map(|h| {
            std::path::PathBuf::from(h)
                .join(".mac-stats")
                .join(".config.env")
        }),
    ];
    for maybe_path in paths.iter().flatten() {
        if let Ok(content) = std::fs::read_to_string(maybe_path) {
            for line in content.lines() {
                let t = line.trim();
                if t.starts_with("OLLAMA_API_KEY=") || t.starts_with("OLLAMA-API-KEY=") {
                    if let Some((_, v)) = t.split_once('=') {
                        let v = v.trim().to_string();
                        if !v.is_empty() {
                            return Some(v);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Read OLLAMA_FAST_MODEL from env or .config.env. When set, used as the default model for
/// agent router (Discord/scheduler) when no channel/message override — gives faster replies (e.g. qwen2.5:1.5b).
fn read_ollama_fast_model_from_env_or_config() -> Option<String> {
    if let Ok(v) = std::env::var("OLLAMA_FAST_MODEL") {
        let v = v.trim().to_string();
        if !v.is_empty() {
            return Some(v);
        }
    }
    let paths = [
        std::env::current_dir().ok().map(|d| d.join(".config.env")),
        std::env::current_dir()
            .ok()
            .map(|d| d.join("src-tauri").join(".config.env")),
        std::env::var("HOME").ok().map(|h| {
            std::path::PathBuf::from(h)
                .join(".mac-stats")
                .join(".config.env")
        }),
    ];
    for maybe_path in paths.iter().flatten() {
        if let Ok(content) = std::fs::read_to_string(maybe_path) {
            for line in content.lines() {
                let t = line.trim();
                if t.starts_with("OLLAMA_FAST_MODEL=") || t.starts_with("OLLAMA-FAST-MODEL=") {
                    if let Some((_, v)) = t.split_once('=') {
                        let v = v.trim().to_string();
                        if !v.is_empty() {
                            return Some(v);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Read OLLAMA_MODEL from env or .config.env files.
fn read_ollama_model_override() -> Option<String> {
    if let Ok(v) = std::env::var("OLLAMA_MODEL") {
        let v = v.trim().to_string();
        if !v.is_empty() {
            return Some(v);
        }
    }
    let paths = [
        std::env::current_dir().ok().map(|d| d.join(".config.env")),
        std::env::current_dir()
            .ok()
            .map(|d| d.join("src-tauri").join(".config.env")),
        std::env::var("HOME").ok().map(|h| {
            std::path::PathBuf::from(h)
                .join(".mac-stats")
                .join(".config.env")
        }),
    ];
    for maybe_path in paths.iter().flatten() {
        // Do not log file content or path; file may contain secrets.
        if let Ok(content) = std::fs::read_to_string(maybe_path) {
            for line in content.lines() {
                let t = line.trim();
                if t.starts_with("OLLAMA_MODEL=") || t.starts_with("OLLAMA-MODEL=") {
                    if let Some((_, v)) = t.split_once('=') {
                        let v = v.trim().to_string();
                        if !v.is_empty() {
                            return Some(v);
                        }
                    }
                }
            }
        }
    }
    None
}

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
fn deduplicate_consecutive_messages(
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

/// One line of Ollama NDJSON stream: `{"message":{"content":"..."},"done":false}`.
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
    use serde_json;
    use tracing::info;

    let request_json = serde_json::to_string_pretty(&request)
        .unwrap_or_else(|_| "Failed to serialize request".to_string());
    info!("Ollama: Chat request JSON:\n{}", request_json);

    send_ollama_chat_messages(request.messages, None, None).await
}

use crate::commands::agent_descriptions::{
    build_agent_descriptions, DISCORD_GROUP_CHANNEL_GUIDANCE, DISCORD_PLATFORM_FORMATTING,
};
use crate::commands::browser_helpers::{
    append_latest_browser_state_guidance, browser_retry_grounding_prompt,
    explicit_no_playable_video_finding, extract_browser_navigation_target,
    is_browser_task_request, is_video_review_request,
    should_use_http_fallback_after_browser_action_error, wants_visible_browser,
};
use crate::commands::schedule_helpers::{parse_schedule_arg, ScheduleParseResult};

/// Heuristic: chars to tokens (conservative).
const CHARS_PER_TOKEN: usize = 4;

/// Reserve tokens for model reply and wrapper text.
const RESERVE_TOKENS: u32 = 512;

/// When over limit by at most 1/this fraction, truncate only (no summarization) to avoid extra Ollama call.
const TRUNCATE_ONLY_THRESHOLD_DENOM: u32 = 4;

/// Truncate at last newline or space before max_chars so we don't cut mid-word. O(max_chars).
fn truncate_at_boundary(body: &str, max_chars: usize) -> String {
    let mut last_break = max_chars;
    let mut i = 0;
    for c in body.chars() {
        if i >= max_chars {
            break;
        }
        if c == '\n' || c == ' ' {
            last_break = i + 1;
        }
        i += 1;
    }
    if i <= max_chars {
        return body.to_string();
    }
    body.chars().take(last_break).collect()
}

/// Reduce fetched page content to fit the model context: summarize via Ollama if needed, else truncate.
/// Uses byte-length heuristic for fast path and "slightly over" path to avoid full char count; only
/// when summarization is needed do we count chars for logging.
async fn reduce_fetched_content_to_fit(
    body: &str,
    context_size_tokens: u32,
    estimated_used_tokens: u32,
    model_override: Option<String>,
    options_override: Option<crate::ollama::ChatOptions>,
) -> Result<String, String> {
    use tracing::info;

    let max_tokens_for_body = context_size_tokens
        .saturating_sub(RESERVE_TOKENS)
        .saturating_sub(estimated_used_tokens);
    let max_chars = (max_tokens_for_body as usize).saturating_mul(CHARS_PER_TOKEN);

    // Fast path: cheap byte heuristic (len/4 >= char_count/4 for UTF-8). Avoids char count when body fits.
    let body_tokens_upper = body.len() / CHARS_PER_TOKEN;
    if body_tokens_upper <= max_tokens_for_body as usize {
        return Ok(body.to_string());
    }

    // Slightly over: within 25% of limit → truncate only, no summarization (saves one Ollama round-trip).
    let threshold = max_tokens_for_body + (max_tokens_for_body / TRUNCATE_ONLY_THRESHOLD_DENOM);
    if body_tokens_upper <= threshold as usize {
        let truncated = truncate_at_boundary(body, max_chars);
        return Ok(format!(
            "{} (content truncated due to context limit)",
            truncated.trim_end()
        ));
    }

    // Way over: summarization path. Compute exact token estimate only for logging.
    let body_tokens_est = body.chars().count() / CHARS_PER_TOKEN;
    info!(
        "Agent router: page content too large (est. {} tokens), max {} tokens; reducing",
        body_tokens_est, max_tokens_for_body
    );

    let body_truncated_for_request = truncate_at_boundary(body, max_chars);
    let summary_tokens = (max_tokens_for_body / 2).max(256);
    let summarization_messages = vec![
        crate::ollama::ChatMessage {
            role: "system".to_string(),
            content: format!(
                "Summarize the following web page content in under {} tokens, keeping the most relevant information for answering questions. Output only the summary, no preamble.",
                summary_tokens
            ),
            images: None,
        },
        crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: body_truncated_for_request,
            images: None,
        },
    ];

    match send_ollama_chat_messages(summarization_messages, model_override, options_override).await
    {
        Ok(resp) => {
            let summary = resp.message.content.trim().to_string();
            if summary.is_empty() {
                let fallback = truncate_at_boundary(body, max_chars);
                Ok(format!(
                    "{} (content truncated due to context limit)",
                    fallback.trim_end()
                ))
            } else {
                Ok(summary)
            }
        }
        Err(e) => {
            info!("Agent router: summarization failed ({}), truncating", e);
            let fallback = truncate_at_boundary(body, max_chars);
            Ok(format!(
                "{} (content truncated due to context limit)",
                fallback.trim_end()
            ))
        }
    }
}

/// Run a single Ollama request in a new session (no conversation history). Used for SKILL agent.
/// System message = skill content, user message = task. Returns the assistant reply or error string.
async fn run_skill_ollama_session(
    skill_content: &str,
    user_message: &str,
    model_override: Option<String>,
    options_override: Option<crate::ollama::ChatOptions>,
) -> Result<String, String> {
    use tracing::info;
    let messages = vec![
        crate::ollama::ChatMessage {
            role: "system".to_string(),
            content: skill_content.to_string(),
            images: None,
        },
        crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: user_message.to_string(),
            images: None,
        },
    ];
    info!(
        "Agent router: SKILL session request (user message {} chars)",
        user_message.chars().count()
    );
    let response = send_ollama_chat_messages(messages, model_override, options_override).await?;
    Ok(response.message.content.trim().to_string())
}

/// Run an Ollama request for an LLM agent (soul+mood+skill as system prompt, task as user message).
/// Uses the agent's model if set; otherwise default. No conversation history. Logs agent name/id.
/// If the agent's response contains DISCORD_API: tool calls, executes them and feeds results back
/// in a loop (up to max_tool_iterations) so agents like the Discord Expert can do multi-step API work.
/// Used by the tool loop (AGENT:) and by the agent-test CLI.
fn build_agent_runtime_context(now: chrono::DateTime<chrono::FixedOffset>) -> String {
    let local_date = now.format("%Y-%m-%d").to_string();
    let local_time = now.format("%Y-%m-%d %H:%M:%S %:z").to_string();
    let utc_now = now.with_timezone(&chrono::Utc);
    let utc_date = utc_now.format("%Y-%m-%d").to_string();
    let utc_time = utc_now.format("%Y-%m-%d %H:%M:%S UTC").to_string();
    format!(
        "## Runtime context\n\n- Current local date: {}\n- Current local time: {}\n- Current UTC date: {}\n- Current UTC time: {}\n- For date-sensitive tool calls such as Redmine \"today\" queries, use the current UTC date ({}) unless the task explicitly asks for local time.",
        local_date, local_time, utc_date, utc_time, utc_date
    )
}

pub(crate) async fn run_agent_ollama_session(
    agent: &crate::agents::Agent,
    user_message: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
    include_global_memory: bool,
) -> Result<String, String> {
    use tracing::info;
    let runtime_context = build_agent_runtime_context(chrono::Local::now().fixed_offset());
    let system_prompt = if include_global_memory {
        &agent.combined_prompt
    } else {
        &agent.combined_prompt_without_memory
    };
    if !include_global_memory {
        info!(
            "Agent: {} ({}) running without global memory (non-main session)",
            agent.name, agent.id
        );
    }
    info!(
        "Agent: {} ({}) running (model: {:?}, prompt {} chars)",
        agent.name,
        agent.id,
        agent.model,
        system_prompt.chars().count()
    );
    info!(
        "Agent: {} ({}) runtime date anchor injected",
        agent.name, agent.id
    );
    let mut messages = vec![
        crate::ollama::ChatMessage {
            role: "system".to_string(),
            content: format!("{}\n\n{}", system_prompt, runtime_context),
            images: None,
        },
        crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: user_message.to_string(),
            images: None,
        },
    ];
    let max_iters = agent.max_tool_iterations;
    let mut iteration = 0u32;
    loop {
        let response =
            send_ollama_chat_messages(messages.clone(), agent.model.clone(), None).await?;
        let out = response.message.content.trim().to_string();
        info!(
            "Agent: {} ({}) iter {} returned ({} chars)",
            agent.name,
            agent.id,
            iteration,
            out.chars().count()
        );

        if let Some(tool_result) = execute_agent_tool_call(&out, status_tx).await {
            if !question_explicitly_requests_json(user_message) {
                if let Some(summary) = extract_redmine_time_entries_summary_for_reply(&tool_result)
                {
                    info!(
                        "Agent: {} ({}) returning direct Redmine time-entry summary",
                        agent.name, agent.id
                    );
                    return Ok(summary);
                }
            }
            iteration += 1;
            if iteration >= max_iters {
                info!(
                    "Agent: {} ({}) hit max tool iterations ({})",
                    agent.name, agent.id, max_iters
                );
                return Ok(out);
            }
            messages.push(crate::ollama::ChatMessage {
                role: "assistant".to_string(),
                content: out,
                images: None,
            });
            messages.push(crate::ollama::ChatMessage {
                role: "user".to_string(),
                content: tool_result,
                images: None,
            });
            continue;
        }

        return Ok(out);
    }
}

/// Normalize Discord API path: strip model commentary after " — " so the path is valid for HTTP.
/// E.g. "/channels/123/messages?limit=10 — fetch the last 10 messages" -> "/channels/123/messages?limit=10"
fn normalize_discord_api_path(path_and_commentary: &str) -> String {
    let s = path_and_commentary.trim();
    let path_only = if let Some(idx) = s.find(" — ") {
        s[..idx].trim()
    } else {
        s
    };
    path_only.to_string()
}


fn truncate_text_on_line_boundaries(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut out = String::new();
    let mut used = 0usize;
    for line in text.lines() {
        let line_len = line.chars().count();
        let extra = if out.is_empty() {
            line_len
        } else {
            line_len + 1
        };
        if used + extra > max_chars {
            break;
        }
        if !out.is_empty() {
            out.push('\n');
            used += 1;
        }
        out.push_str(line);
        used += line_len;
    }
    if out.trim().is_empty() {
        let take = max_chars.saturating_sub(12);
        let mut truncated: String = text.chars().take(take).collect();
        truncated.push_str("\n[truncated]");
        return truncated;
    }
    out.push_str("\n[truncated]");
    out
}

fn summarize_response_for_verification(
    question: &str,
    response_content: &str,
    attachment_count: usize,
) -> String {
    if response_content.trim().is_empty() && attachment_count > 0 {
        return format!("{} attachment(s) were sent to the user.", attachment_count);
    }
    let preferred = if is_redmine_time_entries_request(question) {
        extract_redmine_time_entries_summary_for_reply(response_content)
            .unwrap_or_else(|| response_content.to_string())
    } else {
        response_content.to_string()
    };
    let max_chars = if is_redmine_time_entries_request(question) {
        4000
    } else {
        1500
    };
    truncate_text_on_line_boundaries(&preferred, max_chars)
}

fn strip_tool_result_instructions(tool_result: &str) -> String {
    let mut cleaned = tool_result;
    for marker in [
        "\n\nUse this data to answer the user's question.",
        "\n\nUse only this Redmine data to continue or answer",
        "\n\nUse this data to answer.",
        "\n\nUse this to answer the user's question.",
    ] {
        if let Some(idx) = cleaned.find(marker) {
            cleaned = &cleaned[..idx];
            break;
        }
    }
    for prefix in [
        "Redmine API result:\n\n",
        "Discord API result:\n\n",
        "Here is the command output:\n\n",
        "Here is the page content:\n\n",
        "Search results:\n\n",
    ] {
        if let Some(rest) = cleaned.strip_prefix(prefix) {
            cleaned = rest;
            break;
        }
    }
    cleaned.trim().to_string()
}

fn final_reply_from_tool_results(question: &str, tool_result: &str) -> String {
    if let Some(reply) = grounded_redmine_time_entries_failure_reply(question, tool_result) {
        return reply;
    }
    if !question_explicitly_requests_json(question) {
        if let Some(summary) = extract_redmine_time_entries_summary_for_reply(tool_result) {
            return summary;
        }
    }
    let cleaned = strip_tool_result_instructions(tool_result);
    if cleaned.is_empty() {
        "The requested tool ran, but no final user-facing answer was produced.".to_string()
    } else {
        cleaned
    }
}

/// Execute a tool call found in an agent's response. Supports agent-safe APIs like DISCORD_API and REDMINE_API.
/// Returns Some(result_text) if a tool was executed, None if no tool call was found.
async fn execute_agent_tool_call(
    content: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> Option<String> {
    use tracing::info;
    let (tool, arg) = parse_agent_tool_from_response(content)?;
    match tool.as_str() {
        "DISCORD_API" => {
            let arg = arg.trim();
            let (method, rest) = match arg.find(' ') {
                Some(i) => (arg[..i].trim().to_string(), arg[i..].trim().to_string()),
                None => ("GET".to_string(), arg.to_string()),
            };
            let (path_raw, body) = if let Some(idx) = rest.find(" {") {
                let (p, b) = rest.split_at(idx);
                (p.trim().to_string(), Some(b.trim().to_string()))
            } else {
                (rest.clone(), None)
            };
            let path = normalize_discord_api_path(&path_raw);
            if path.is_empty() {
                return Some(
                    "DISCORD_API requires a path (e.g. GET /users/@me/guilds). Try again."
                        .to_string(),
                );
            }
            if let Some(tx) = status_tx {
                let _ = tx.send(format!("Discord API: {} {}", &method, &path));
            }
            info!("Agent tool: DISCORD_API {} {}", &method, &path);
            match crate::discord::api::discord_api_request(&method, &path, body.as_deref()).await {
                Ok(result) => Some(format!(
                    "DISCORD_API result ({} {}):\n\n{}\n\nUse this data to continue or answer the user's question. If you need more data, make another DISCORD_API call.",
                    &method, &path, result
                )),
                Err(e) => {
                    let msg = crate::discord::api::sanitize_discord_api_error(&e);
                    Some(format!(
                        "DISCORD_API failed ({} {}): {}. Explain the error to the user or try a different approach.",
                        &method, &path, msg
                    ))
                }
            }
        }
        "REDMINE_API" => {
            let arg = arg.trim();
            let (method, rest) = match arg.find(' ') {
                Some(i) => (arg[..i].trim().to_string(), arg[i..].trim()),
                None => ("GET".to_string(), arg),
            };
            let (path, body) = if let Some(idx) = rest.find(" {") {
                let (p, b) = rest.split_at(idx);
                (p.trim().to_string(), Some(b.trim().to_string()))
            } else {
                (rest.trim().to_string(), None)
            };
            if path.is_empty() {
                return Some(
                    "REDMINE_API requires a path (for example: GET /issues/1234.json?include=journals,attachments). Try again."
                        .to_string(),
                );
            }
            if let Some(tx) = status_tx {
                let _ = tx.send(format!("Querying Redmine: {} {}", &method, &path));
            }
            info!("Agent tool: REDMINE_API {} {}", &method, &path);
            match crate::redmine::redmine_api_request(&method, &path, body.as_deref()).await {
                Ok(result) => Some(format!(
                    "REDMINE_API result ({} {}):\n\n{}\n\nUse only this Redmine data to continue or answer the user's question. For time-entry queries, prefer the derived summary and actual ticket list already included above; do not invent ticket ids or subjects. If this is a ticket review, reply with Summary, Status & completion, Missing, and Final thoughts. If you need more Redmine data, make another REDMINE_API call.",
                    &method, &path, result
                )),
                Err(e) => Some(format!(
                    "REDMINE_API failed ({} {}): {}. Explain the failure clearly and do not invent any Redmine data.",
                    &method, &path, e
                )),
            }
        }
        _ => None,
    }
}

/// Parse agent-safe tool calls from an agent response.
///
/// This reuses the main router's tool normalization so specialist agents still work
/// when the model wraps the tool in `RECOMMEND:` or emits an inline chain such as
/// `RUN_CMD: ... then REDMINE_API ...`. Unsupported tools are ignored; the first
/// allowed agent-safe tool is returned.
fn parse_agent_tool_from_response(content: &str) -> Option<(String, String)> {
    let normalized = normalize_inline_tool_sequences(content);
    let lines: Vec<&str> = normalized.lines().collect();
    let mut idx = 0;
    while idx < lines.len() {
        if let Some(((tool, arg), next)) = parse_one_tool_at_line(&lines, idx) {
            if tool == "DISCORD_API" || tool == "REDMINE_API" {
                return Some((tool, arg));
            }
            idx = next;
        } else {
            idx += 1;
        }
    }
    None
}

/// Resolve Mastodon credentials: instance URL and access token.
/// Checks env vars (MASTODON_INSTANCE_URL, MASTODON_ACCESS_TOKEN), then ~/.mac-stats/.config.env,
/// then Keychain (mastodon_instance_url, mastodon_access_token).
pub(crate) fn get_mastodon_config() -> Option<(String, String)> {
    let resolve = |env_key: &str, file_key: &str, keychain_key: &str| -> Option<String> {
        if let Ok(v) = std::env::var(env_key) {
            let v = v.trim().to_string();
            if !v.is_empty() {
                return Some(v);
            }
        }
        for base in [
            std::env::current_dir().ok(),
            std::env::var("HOME").ok().map(std::path::PathBuf::from),
        ]
        .into_iter()
        .flatten()
        {
            let paths = [
                base.join(".config.env"),
                base.join(".mac-stats").join(".config.env"),
            ];
            for p in &paths {
                // Do not log file content or path; file may contain secrets.
                if let Ok(content) = std::fs::read_to_string(p) {
                    for line in content.lines() {
                        if let Some(val) = line.strip_prefix(file_key) {
                            let val = val.trim().trim_matches('"').trim().to_string();
                            if !val.is_empty() {
                                return Some(val);
                            }
                        }
                    }
                }
            }
        }
        if let Ok(Some(v)) = crate::security::get_credential(keychain_key) {
            if !v.is_empty() {
                return Some(v);
            }
        }
        None
    };
    let instance = resolve(
        "MASTODON_INSTANCE_URL",
        "MASTODON_INSTANCE_URL=",
        "mastodon_instance_url",
    )?;
    let token = resolve(
        "MASTODON_ACCESS_TOKEN",
        "MASTODON_ACCESS_TOKEN=",
        "mastodon_access_token",
    )?;
    Some((instance.trim_end_matches('/').to_string(), token))
}

/// Post a status to Mastodon. Visibility: public, unlisted, private, or direct.
async fn mastodon_post(status: &str, visibility: &str) -> Result<String, String> {
    let (instance, token) = get_mastodon_config()
        .ok_or("Mastodon not configured. Set MASTODON_INSTANCE_URL and MASTODON_ACCESS_TOKEN in env or ~/.mac-stats/.config.env")?;
    let url = format!("{}/api/v1/statuses", instance);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("HTTP client: {}", e))?;
    let payload = serde_json::json!({
        "status": status,
        "visibility": visibility,
    });
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Mastodon API request failed: {}", e))?;
    let status_code = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if status_code.is_success() {
        let url = serde_json::from_str::<serde_json::Value>(&body)
            .ok()
            .and_then(|v| v.get("url").and_then(|u| u.as_str()).map(|s| s.to_string()));
        Ok(match url {
            Some(u) => format!("Posted to Mastodon: {}", u),
            None => "Posted to Mastodon successfully.".to_string(),
        })
    } else {
        Err(format!("Mastodon API error {}: {}", status_code, body))
    }
}

/// Append a line to a file, creating it if needed. Returns the path on success.
pub(crate) fn append_to_file(path: &std::path::Path, content: &str) -> Result<std::path::PathBuf, String> {
    use std::io::Write;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {}", e))?;
    }
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("open: {}", e))?;
    f.write_all(content.as_bytes())
        .map_err(|e| format!("write: {}", e))?;
    Ok(path.to_path_buf())
}

/// Detect prior assistant messages that mention 401/token errors about Discord (from FETCH_URL misuse).
/// Used to annotate conversation history so the model doesn't repeat the mistake.
fn looks_like_discord_401_confusion(content: &str) -> bool {
    let lower = content.to_lowercase();
    (lower.contains("401") || lower.contains("unauthorized"))
        && (lower.contains("token")
            || lower.contains("credential")
            || lower.contains("authentication"))
        && (lower.contains("discord") || lower.contains("guild") || lower.contains("channel"))
}

/// Shared API for Discord (and other agents): ask Ollama how to solve, then run agents (FETCH_URL, BRAVE_SEARCH, RUN_JS).
/// 1) Planning: send user question + agent list, get RECOMMEND: plan.
/// 2) Execution: send plan + "now answer using agents", loop on FETCH_URL / BRAVE_SEARCH / RUN_JS (max 5 tool calls).
///    If `status_tx` is provided (e.g. from Discord), short status messages are sent so the user sees we're still working.
///    If `discord_reply_channel_id` is set (when the request came from Discord), SCHEDULE will store it so the scheduler can post results to that channel (DM or mention channel).
///    When `discord_user_id` and `discord_user_name` are set (from Discord message author), the prompt is prefixed with "You are talking to Discord user **{name}** (user id: {id})."
///    When set, `model_override` and `options_override` apply only to this request (e.g. from Discord "model: llama3" line).
///    Extract a URL from the question for pre-routing (e.g. screenshot). Prefers https?:// then www.
///    Strips trailing punctuation from the URL.
fn extract_url_from_question(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    for prefix in ["https://", "http://"] {
        if let Some(pos) = lower.find(prefix) {
            let start = text[pos..]
                .char_indices()
                .next()
                .map(|(o, _)| pos + o)
                .unwrap_or(pos);
            let after = &text[start + prefix.len()..];
            let end = after
                .char_indices()
                .find(|(_, c)| {
                    *c == ' '
                        || *c == '\n'
                        || *c == '\t'
                        || *c == ')'
                        || *c == ']'
                        || *c == '"'
                        || *c == '>'
                })
                .map(|(i, _)| i)
                .unwrap_or(after.len());
            let url = format!("{}{}", &text[start..start + prefix.len()], &after[..end]);
            let url = url.trim_end_matches(['.', ',', ';', '!', '?']);
            if !url.is_empty() && url.len() > prefix.len() {
                return Some(url.to_string());
            }
        }
    }
    // www. something
    if let Some(pos) = lower.find("www.") {
        let after = &text[pos..];
        let end = after
            .char_indices()
            .find(|(_, c)| {
                *c == ' '
                    || *c == '\n'
                    || *c == '\t'
                    || *c == ')'
                    || *c == ']'
                    || *c == '"'
                    || *c == '>'
            })
            .map(|(i, _)| i)
            .unwrap_or(after.len());
        let url = after[..end].trim_end_matches(['.', ',', ';', '!', '?']);
        if url.len() > 4 {
            let full = if url.starts_with("http") {
                url.to_string()
            } else {
                format!("https://{}", url)
            };
            return Some(full);
        }
    }
    None
}

/// If the question clearly asks for a screenshot of a URL, return RECOMMEND string for pre-routing.
/// Browser-use style: screenshot only works on current page. Pre-route to NAVIGATE + SCREENSHOT: current.
/// Skip pre-route when user wants multi-step (navigate all, find X) — let planner handle it.
/// Skip pre-route when user asks to remove/dismiss cookie consent — planner must add BROWSER_CLICK before SCREENSHOT.
fn extract_screenshot_recommendation(question: &str) -> Option<String> {
    let q = question.trim();
    let q_lower = q.to_lowercase();
    let has_multi_step = q_lower.contains("navigate all")
        || q_lower.contains("find ")
        || q_lower.contains("when you found")
        || q_lower.contains("when found")
        || (q_lower.contains("click on")
            || q_lower.contains("and click")
            || q_lower.contains("then click"));
    let asks_cookie_consent = q_lower.contains("cookie")
        && (q_lower.contains("remove")
            || q_lower.contains("dismiss")
            || q_lower.contains("banner")
            || q_lower.contains("consent"));
    if has_multi_step || asks_cookie_consent {
        return None;
    }
    let has_screenshot_intent = q_lower.contains("screenshot")
        || q_lower.contains("take a screenshot")
        || q_lower.contains("create a screenshot")
        || q_lower.contains("capture the page")
        || (q_lower.contains("capture")
            && (q_lower.contains("page") || q_lower.contains("browser")));
    let has_browser_or_url_context = q_lower.contains("browser")
        || q_lower.contains("chrome")
        || q_lower.contains("goto")
        || q_lower.contains("go to")
        || q_lower.contains("visit")
        || q_lower.contains("navigate")
        || q_lower.contains("http")
        || q_lower.contains("www.");
    if has_screenshot_intent && has_browser_or_url_context {
        if let Some(url) = extract_url_from_question(q) {
            let rec = format!("BROWSER_NAVIGATE: {}\nBROWSER_SCREENSHOT: current", url);
            tracing::info!(
                "Agent router: pre-routed to BROWSER_NAVIGATE + BROWSER_SCREENSHOT (browser-use style): {}",
                crate::logging::ellipse(&url, 60)
            );
            return Some(rec);
        }
    }
    None
}

/// Extract the trimmed argument after the last literal tool prefix in the user's message.
/// Example: "Run this command: RUN_CMD: cat /etc/hosts" -> "cat /etc/hosts".
fn extract_last_prefixed_argument(text: &str, prefix: &str) -> Option<String> {
    if prefix.is_empty() || text.len() < prefix.len() {
        return None;
    }
    let mut last_match = None;
    for (idx, _) in text.char_indices() {
        if let Some(candidate) = text.get(idx..idx + prefix.len()) {
            if candidate.eq_ignore_ascii_case(prefix) {
                last_match = Some(idx);
            }
        }
    }
    let start = last_match?;
    let arg = text.get(start + prefix.len()..)?.trim();
    if arg.is_empty() {
        None
    } else {
        Some(arg.to_string())
    }
}

/// True when the planner's recommendation is only "DONE: no" or "DONE: success" with no actual tool steps.
fn is_bare_done_plan(s: &str) -> bool {
    let t = s.trim().trim_matches('*').trim();
    t.eq_ignore_ascii_case("DONE: no") || t.eq_ignore_ascii_case("DONE: success")
}

/// Normalize reply text for comparison: strip DONE line and trim.
fn normalize_reply_for_compare(s: &str) -> String {
    let mut out = s.trim().to_string();
    // Remove trailing "DONE: success" / "DONE: no" line
    for suffix in [
        "\n\nDONE: success",
        "\n\nDONE: no",
        "\nDONE: success",
        "\nDONE: no",
    ] {
        if out.ends_with(suffix) {
            out = out.strip_suffix(suffix).unwrap_or(&out).trim().to_string();
            break;
        }
    }
    out
}

/// True when the final reply is effectively the same as the intermediate (so we show "Final answer is the same as intermediate.").
fn is_final_same_as_intermediate(intermediate: &str, final_answer: &str) -> bool {
    let a = normalize_reply_for_compare(intermediate);
    let b = normalize_reply_for_compare(final_answer);
    if a == b {
        return true;
    }
    // Final is a short confirmation (retry only added "Confirmed." / "Done." etc.)
    if b.len() < 120 && !b.contains('\n') && a.len() > 150 {
        let b_lower = b.to_lowercase();
        if b_lower.contains("same as intermediate")
            || b_lower.contains("confirmed")
            || (b_lower.contains("done") && !b_lower.contains("error"))
            || b_lower == "done."
            || b_lower == "confirmed."
        {
            return true;
        }
    }
    false
}

fn is_agent_unavailable_error(error: &str) -> bool {
    let e = error.to_lowercase();
    e.contains("busy or unavailable")
        || e.contains("timed out")
        || e.contains("timeout")
        || e.contains("503")
}

fn user_explicitly_asked_for_screenshot(question: &str) -> bool {
    let q = question.to_lowercase();
    q.contains("screenshot")
        || q.contains("take a screenshot")
        || q.contains("create a screenshot")
        || (q.contains("capture") && (q.contains("page") || q.contains("browser")))
}

/// Reply from the agent: text plus optional attachment paths (e.g. screenshots) for Discord.
#[derive(Debug, Clone)]
pub struct OllamaReply {
    pub text: String,
    pub attachment_paths: Vec<PathBuf>,
}

/// Request-local execution context for a single Discord/Ollama run (task-008 Phase 1).
/// Holds only state belonging to this request so verification retries cannot inherit
/// stale criteria, tool payloads, or task context from prior requests.
#[derive(Clone)]
pub struct RequestRunContext {
    pub request_id: String,
    pub retry_count: u32,
    pub original_user_question: String,
    pub discord_channel_id: Option<u64>,
    pub discord_user_id: Option<u64>,
    pub discord_user_name: Option<String>,
}

/// One short Ollama call to extract 1–3 concrete success criteria from the user request.
/// Returns None on error or empty so the run is not blocked.
async fn extract_success_criteria(
    question: &str,
    model_override: Option<String>,
    options_override: Option<crate::ollama::ChatOptions>,
) -> Option<Vec<String>> {
    let q: String = question.chars().take(800).collect();
    let system = "You are an assistant that extracts success criteria. Reply with 1 to 3 concrete success criteria, one per line (e.g. 'screenshot of the page attached', 'page content fetched'). No numbering, no extra text.";
    let user = format!(
        "User request:\n\n{}\n\nList 1–3 concrete success criteria (one per line). Reply with only the criteria.",
        q
    );
    let messages = vec![
        crate::ollama::ChatMessage {
            role: "system".to_string(),
            content: system.to_string(),
            images: None,
        },
        crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: user,
            images: None,
        },
    ];
    let response = match send_ollama_chat_messages(messages, model_override, options_override).await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::debug!("Agent router: success criteria extraction failed: {}", e);
            return None;
        }
    };
    let text = response.message.content.trim();
    let criteria: Vec<String> = text
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty() && s.len() > 2)
        .take(3)
        .map(String::from)
        .collect();
    if criteria.is_empty() {
        tracing::debug!(
            "Agent router: success criteria extraction returned no criteria (empty or parse)"
        );
        None
    } else {
        Some(criteria)
    }
}

fn sanitize_success_criteria(question: &str, criteria: Vec<String>) -> Vec<String> {
    let q = question.to_lowercase();
    let explicit_last_30_days = q.contains("last 30 days")
        || q.contains("past 30 days")
        || q.contains("30-day")
        || q.contains("30 day")
        || q.contains("this month")
        || q.contains("last month");
    let explicit_last_week = q.contains("last week")
        || q.contains("past week")
        || q.contains("this week")
        || q.contains("last 7 days")
        || q.contains("past 7 days");
    let generic_news_request = q.contains("news");
    let explicit_football_request = q.contains("football")
        || q.contains("soccer")
        || q.contains("fc barcelona")
        || q.contains("barcelona fc")
        || q.contains("barça")
        || q.contains("barca")
        || q.contains("club")
        || q.contains("match")
        || q.contains("transfer")
        || q.contains("la liga");
    let explicit_named_sources = q.contains("bbc")
        || q.contains("cnn")
        || q.contains("reuters")
        || q.contains("ap ")
        || q.contains("associated press");
    let review_videos_request =
        q.contains("video") && (q.contains("review") || q.contains("check"));
    let explicit_playback_request = q.contains("play video")
        || q.contains("play the video")
        || q.contains("playable")
        || q.contains("watch the video")
        || q.contains("start the video");

    let mut sanitized = Vec::new();
    for criterion in criteria {
        let trimmed = criterion.trim();
        if trimmed.is_empty() {
            continue;
        }
        let lower = trimmed.to_lowercase();
        if lower.contains("last 30 days") && !explicit_last_30_days {
            if generic_news_request {
                sanitized.push("recent news items were summarized".to_string());
            }
            continue;
        }
        if (lower.contains("last week")
            || lower.contains("past week")
            || lower.contains("last 7 days")
            || lower.contains("past 7 days"))
            && !explicit_last_week
        {
            if generic_news_request {
                let replacement = "information includes dates and relevant details".to_string();
                if !sanitized.iter().any(|existing| existing == &replacement) {
                    sanitized.push(replacement);
                }
            }
            continue;
        }
        if generic_news_request && !explicit_football_request
            && (lower.contains("football club")
                || lower.contains("sports website")
                || lower.contains("related to the team"))
            {
                let replacement = if lower.contains("sports website") {
                    "credible named sources cited".to_string()
                } else if lower.contains("related to the team") {
                    "major recent developments involving Barcelona were covered".to_string()
                } else {
                    "recent news items involving Barcelona were summarized".to_string()
                };
                if !sanitized.iter().any(|existing| existing == &replacement) {
                    sanitized.push(replacement);
                }
                continue;
            }
        if generic_news_request
            && !explicit_named_sources
            && (lower.contains("bbc") || lower.contains("cnn") || lower.contains("reuters"))
        {
            if !sanitized
                .iter()
                .any(|existing| existing == "credible named sources cited")
            {
                sanitized.push("credible named sources cited".to_string());
            }
            continue;
        }
        if review_videos_request
            && !explicit_playback_request
            && (lower.contains("video") || lower.contains("playable"))
        {
            let replacement = "video availability or playability was checked".to_string();
            if !sanitized.iter().any(|existing| existing == &replacement) {
                sanitized.push(replacement);
            }
            continue;
        }
        if !sanitized.iter().any(|existing| existing == trimmed) {
            sanitized.push(trimmed.to_string());
        }
    }
    sanitized
}


/// Build a short summary of the last N turns (user/assistant pairs) for the new-topic check.
/// Each message content is truncated to avoid blowing context.
fn summarize_last_turns(messages: &[crate::ollama::ChatMessage], max_turns: usize) -> String {
    const PER_MSG: usize = 120;
    let take = (max_turns * 2).min(messages.len()); // pairs = 2 messages each
    let start = messages.len().saturating_sub(take);
    messages[start..]
        .iter()
        .map(|m| {
            let c: String = m.content.chars().take(PER_MSG).collect();
            let suffix = if m.content.chars().count() > PER_MSG {
                "…"
            } else {
                ""
            };
            format!("{}: {}{}", m.role, c, suffix)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// One short Ollama call to detect if the user's current message is a new topic vs same thread (section 6.A).
/// Returns Ok(true) for NEW_TOPIC, Ok(false) for SAME_TOPIC. On error returns Err (caller may keep history).
/// Uses the given model (prefer local/small when running locally to avoid cost; skip entirely when cloud).
async fn detect_new_topic(
    question: &str,
    last_turns_summary: &str,
    model: &str,
) -> Result<bool, String> {
    let system = "You are a classifier. Answer only with exactly one of: NEW_TOPIC or SAME_TOPIC.";
    let user = format!(
        "Given the user's current message and a short summary of the last turns, is the user starting a **new topic** (reply NEW_TOPIC) or continuing the **same thread** (SAME_TOPIC)?\n\nCurrent message: {}\n\nLast turns:\n{}",
        question.chars().take(500).collect::<String>(),
        last_turns_summary
    );
    let messages = vec![
        crate::ollama::ChatMessage {
            role: "system".to_string(),
            content: system.to_string(),
            images: None,
        },
        crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: user,
            images: None,
        },
    ];
    let response = send_ollama_chat_messages(messages, Some(model.to_string()), None).await?;
    let text = response.message.content.trim().to_uppercase();
    if text.contains("NEW_TOPIC") {
        Ok(true)
    } else if text.contains("SAME_TOPIC") {
        Ok(false)
    } else {
        Err(format!(
            "Unexpected response (expected NEW_TOPIC or SAME_TOPIC): {}…",
            text.chars().take(80).collect::<String>()
        ))
    }
}

/// Read the first image file (png, jpg, jpeg, webp) from paths and return its base64 encoding.
fn first_image_as_base64(paths: &[PathBuf]) -> Option<String> {
    let ext_ok = |p: &Path| {
        p.extension()
            .and_then(|e| e.to_str())
            .map(|e| matches!(e.to_lowercase().as_str(), "png" | "jpg" | "jpeg" | "webp"))
            .unwrap_or(false)
    };
    for path in paths {
        if ext_ok(path) {
            if let Ok(bytes) = std::fs::read(path) {
                return Some(base64::engine::general_purpose::STANDARD.encode(&bytes));
            }
        }
    }
    None
}

/// One short Ollama call to check if we fully satisfied the user's request.
/// Returns (satisfied, optional reason when not satisfied).
/// When we have image attachment(s) and a local vision model, sends the first image and asks "Does this image satisfy the request?"; otherwise text-only verification.
/// When page_content_from_browser is Some (e.g. from BROWSER_EXTRACT), it is included so the verifier can check requested text against JS-rendered content (SPAs).
/// Returns the verification-prompt block when the last news search had only hub/landing pages and the request is news-like; otherwise "".
fn verification_news_hub_only_block(news_search_was_hub_only: Option<bool>, question: &str) -> &'static str {
    const HUB_ONLY_BLOCK: &str = "The search results given to the assistant for this news request were only hub/landing/tag/standings pages (no concrete article links). If the assistant's answer presents them as complete recent news articles, reply NO and state that article-grade sources were not found.\n\n";
    if news_search_was_hub_only == Some(true) && is_news_query(question) {
        HUB_ONLY_BLOCK
    } else {
        ""
    }
}

/// For news/current-events requests: tell verifier to accept concise/bullet answers and only require 2+ sources and dates when available.
fn verification_news_format_note(question: &str) -> &'static str {
    const NOTE: &str = "For this news/current-events request: reply YES if the answer has at least 2 named sources and includes dates when available in the results; do not reply NO only because the answer is short or in bullet form.\n\n";
    if is_news_query(question) {
        NOTE
    } else {
        ""
    }
}

/// Returns (satisfied, optional reason when not satisfied).
/// When we have image attachment(s) and a local vision model, sends the first image and asks "Does this image satisfy the request?"; otherwise text-only verification.
/// When page_content_from_browser is Some (e.g. from BROWSER_EXTRACT), it is included so the verifier can check requested text against JS-rendered content (SPAs).
/// When news_search_was_hub_only is Some(true), the verifier must not accept an answer that presents hub/landing/tag/standings pages as complete recent news.
#[allow(clippy::too_many_arguments)]
async fn verify_completion(
    question: &str,
    response_content: &str,
    attachment_paths: &[PathBuf],
    success_criteria: Option<&[String]>,
    page_content_from_browser: Option<&str>,
    news_search_was_hub_only: Option<bool>,
    model_override: Option<String>,
    options_override: Option<crate::ollama::ChatOptions>,
) -> Result<(bool, Option<String>), String> {
    use crate::ollama::models::{get_vision_model_for_verification, is_vision_capable};

    let has_attachments = !attachment_paths.is_empty();
    let screenshot_requested = user_explicitly_asked_for_screenshot(question);
    // Keep verification summaries line-safe so large Redmine replies do not get truncated
    // in the middle of a ticket row and trigger fake "missing details" failures.
    let response_summary =
        summarize_response_for_verification(question, response_content, attachment_paths.len());
    if is_grounded_redmine_time_entries_blocked_reply(question, &response_summary) {
        tracing::info!("Agent router: verification accepted grounded Redmine blocked-state answer");
        return Ok((true, None));
    }
    if is_video_review_request(question) && explicit_no_playable_video_finding(&response_summary) {
        tracing::info!("Agent router: verification accepted explicit no-playable-video finding");
        return Ok((true, None));
    }
    let system = "You are a completion checker. Answer only with YES or NO, and if NO add one short sentence after a newline explaining what's missing.";
    let criteria_block = success_criteria
        .filter(|c| !c.is_empty())
        .map(|c| {
            format!(
                "Success criteria (from user request):\n{}\n\n",
                c.join("\n")
            )
        })
        .unwrap_or_default();
    let browser_content_block = page_content_from_browser
        .filter(|s| !s.trim().is_empty())
        .map(|s| {
            let truncated = s.chars().take(8000).collect::<String>();
            format!(
                "Rendered page text from browser (JS-rendered; use this to check if requested text appears on the page):\n\n{}\n\n",
                truncated
            )
        })
        .unwrap_or_default();
    let attachment_block = if screenshot_requested || has_attachments {
        format!(
            "Attachments sent: {}.\n\n",
            if has_attachments { "yes" } else { "no" }
        )
    } else {
        String::new()
    };
    let news_hub_only_block =
        verification_news_hub_only_block(news_search_was_hub_only, question);
    let news_format_note = verification_news_format_note(question);
    let verification_tail = if screenshot_requested {
        "Did we fully satisfy the request (including any requested screenshot/file attachment)? Reply YES or NO. If NO, on the next line add one sentence: what's missing."
    } else {
        "Did we fully satisfy the request? Reply YES or NO. If NO, on the next line add one sentence: what's missing."
    };
    let user_text = format!(
        "Original request: {}\n\n{}{}{}What we did (summary): {}\n\n{}{}{}",
        question.chars().take(500).collect::<String>(),
        criteria_block,
        news_hub_only_block,
        news_format_note,
        response_summary,
        browser_content_block,
        attachment_block,
        verification_tail
    );

    // Vision path: when we have an image and a vision model, send image + prompt for better verification.
    let image_b64 = first_image_as_base64(attachment_paths);
    let vision_model = model_override
        .as_ref()
        .filter(|m| is_vision_capable(m))
        .cloned()
        .or_else(get_vision_model_for_verification);
    let tried_vision = image_b64.is_some() && vision_model.is_some();

    let (messages, verification_model) = if let (Some(b64), Some(vm)) = (image_b64, vision_model) {
        tracing::debug!("Agent router: verification using vision model {}", vm);
        (
            vec![
                crate::ollama::ChatMessage {
                    role: "system".to_string(),
                    content: system.to_string(),
                    images: None,
                },
                crate::ollama::ChatMessage {
                    role: "user".to_string(),
                    content: user_text.clone(),
                    images: Some(vec![b64]),
                },
            ],
            Some(vm),
        )
    } else {
        (
            vec![
                crate::ollama::ChatMessage {
                    role: "system".to_string(),
                    content: system.to_string(),
                    images: None,
                },
                crate::ollama::ChatMessage {
                    role: "user".to_string(),
                    content: user_text,
                    images: None,
                },
            ],
            model_override.clone(),
        )
    };

    let response = match send_ollama_chat_messages(
        messages,
        verification_model.clone(),
        options_override.clone(),
    )
    .await
    {
        Ok(r) => r.message.content,
        Err(e) => {
            // If we tried vision and it failed, fall back to text-only once.
            if tried_vision {
                tracing::debug!(
                    "Agent router: vision verification failed ({}), falling back to text-only",
                    e
                );
                let messages_text = vec![
                    crate::ollama::ChatMessage {
                        role: "system".to_string(),
                        content: system.to_string(),
                        images: None,
                    },
                    crate::ollama::ChatMessage {
                        role: "user".to_string(),
                        content: format!(
                            "Original request: {}\n\n{}{}{}What we did (summary): {}\n\n{}Did we fully satisfy the request? Reply YES or NO. If NO, on the next line add one sentence: what's missing.",
                            question.chars().take(500).collect::<String>(),
                            criteria_block,
                            news_hub_only_block,
                            news_format_note,
                            response_summary,
                            if screenshot_requested {
                                "Attachments sent: yes.\n\n"
                            } else {
                                ""
                            }
                        ),
                        images: None,
                    },
                ];
                match send_ollama_chat_messages(
                    messages_text,
                    model_override,
                    options_override.clone(),
                )
                .await
                {
                    Ok(r) => r.message.content,
                    Err(e2) => {
                        tracing::warn!("Completion verification (text fallback) failed: {}", e2);
                        return Ok((true, None));
                    }
                }
            } else {
                tracing::warn!("Completion verification call failed: {}", e);
                return Ok((true, None));
            }
        }
    };
    let response_upper = response.trim().to_uppercase();
    let satisfied = response_upper.starts_with("YES");
    tracing::debug!(
        "Agent router: verification result: {} (response: {}...)",
        if satisfied { "YES" } else { "NO" },
        response.trim().chars().take(80).collect::<String>()
    );
    let reason = if !satisfied {
        let first_line = response.lines().next().unwrap_or("").trim();
        let rest = response
            .lines()
            .skip(1)
            .map(str::trim)
            .find(|s| !s.is_empty())
            .or_else(|| response.lines().nth(1).map(str::trim))
            .filter(|s| !s.is_empty());
        rest.or(if first_line.len() > 3 {
            Some(first_line)
        } else {
            None
        })
        .map(|s| s.to_string())
    } else {
        None
    };
    Ok((satisfied, reason))
}

fn original_request_for_retry(
    question: &str,
    conversation_history: Option<&[crate::ollama::ChatMessage]>,
    is_verification_retry: bool,
) -> String {
    if !is_verification_retry {
        return question.to_string();
    }
    conversation_history
        .and_then(|history| {
            history
                .iter()
                .rev()
                .find(|msg| msg.role == "user" && !msg.content.trim().is_empty())
        })
        .map(|msg| msg.content.trim().to_string())
        .unwrap_or_else(|| question.to_string())
}

/// When set, `skill_content` is prepended to system prompts (from ~/.mac-stats/agents/skills/skill-<n>-<topic>.md).
/// When set, `agent_override` uses that agent's model and combined_prompt (soul+mood+skill) for the main run (e.g. Discord "agent: 001").
/// When `allow_schedule` is false (e.g. when running from the scheduler), the SCHEDULE tool is disabled so a scheduled task cannot create more schedules.
/// When `conversation_history` is set (e.g. from Discord session memory), it is prepended so the model sees prior turns and can resolve "there", "it", etc.
/// When `escalation` is true (e.g. user said "think harder" or "get it done"), we inject a stronger "you MUST complete the task" instruction and allow more tool steps.
/// When `retry_on_verification_no` is true and verification says we didn't satisfy the request, we run one retry with a "complete the remaining steps" prompt; the retry run is called with `retry_on_verification_no: false` so we don't retry indefinitely.
/// When `from_remote` is true (Discord, scheduler, task runner), browser runs default to headless (no visible window) unless the question explicitly asks to see the browser (e.g. "visible", "show me").
/// Returns a boxed future to allow one level of async recursion (retry path).
#[allow(clippy::too_many_arguments)]
pub fn answer_with_ollama_and_fetch(
    question: &str,
    status_tx: Option<tokio::sync::mpsc::UnboundedSender<String>>,
    discord_reply_channel_id: Option<u64>,
    discord_user_id: Option<u64>,
    discord_user_name: Option<String>,
    model_override: Option<String>,
    options_override: Option<crate::ollama::ChatOptions>,
    skill_content: Option<String>,
    agent_override: Option<crate::agents::Agent>,
    allow_schedule: bool,
    conversation_history: Option<Vec<crate::ollama::ChatMessage>>,
    escalation: bool,
    retry_on_verification_no: bool,
    from_remote: bool,
    // When set (e.g. from Discord message attachments), the first user message is sent with these base64-encoded images for vision models.
    attachment_images_base64: Option<Vec<String>>,
    // When set (retry path from Discord), we format the reply as "--- Intermediate answer:\n\n{this}\n\n---" + final or "Final answer is the same as intermediate."
    discord_intermediate: Option<String>,
    // When true (verification retry path), we keep conversation history and skip NEW_TOPIC check so the model retains context.
    is_verification_retry: bool,
    // Original end-user request for this run. When set, verification/criteria extraction use this
    // instead of inferring from retry prompts or conversation history.
    original_user_request: Option<String>,
    // Request-local criteria extracted on the first pass. Reused on retry to avoid context bleed.
    success_criteria_override: Option<Vec<String>>,
    // When Some(true) = Discord DM (main session, load global memory). When Some(false) = Discord guild (do not load global memory). When None = not Discord (load global memory).
    discord_is_dm: Option<bool>,
    // When Some, use this request id so retries share the same id for end-to-end log correlation (task-008 Phase 1).
    request_id_override: Option<String>,
    // 0 = first run; 1 = verification retry. Logged for request tracing.
    retry_count: u32,
) -> Pin<Box<dyn Future<Output = Result<OllamaReply, String>> + Send>> {
    let load_global_memory = discord_is_dm.is_none_or(|dm| dm);
    if discord_is_dm == Some(false) {
        tracing::info!(
            "Agent router: Discord guild channel — not loading global memory (main-session only)"
        );
    }
    let question = question.to_string();
    let mut conversation_history = conversation_history.map(|v| v.to_vec());
    let attachment_images_base64 = attachment_images_base64.map(|v| v.to_vec());
    let discord_intermediate = discord_intermediate.map(|s| s.to_string());
    let original_user_request = original_user_request.map(|s| s.to_string());
    let success_criteria_override = success_criteria_override.map(|v| v.to_vec());
    Box::pin(async move {
        use tracing::info;
        let question = question.as_str();
        let request_for_verification = original_user_request.clone().unwrap_or_else(|| {
            original_request_for_retry(
                question,
                conversation_history.as_deref(),
                is_verification_retry,
            )
        });
        let request_id: String = request_id_override.unwrap_or_else(|| {
            format!(
                "{:08x}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64
                    & 0xFFFF_FFFF
            )
        });

        let run_ctx = RequestRunContext {
            request_id: request_id.clone(),
            retry_count,
            original_user_question: request_for_verification.clone(),
            discord_channel_id: discord_reply_channel_id,
            discord_user_id,
            discord_user_name: discord_user_name.clone(),
        };

        if run_ctx.retry_count > 0 {
            info!(
                "Agent router [{}]: session start (verification retry {}, request-local criteria only) — {}",
                run_ctx.request_id,
                run_ctx.retry_count,
                crate::config::Config::version_display()
            );
        } else {
            info!(
                "Agent router [{}]: session start — {}",
                run_ctx.request_id,
                crate::config::Config::version_display()
            );
        }
        tracing::debug!(
            request_id = %run_ctx.request_id,
            retry_count = run_ctx.retry_count,
            channel_id = ?run_ctx.discord_channel_id,
            user_id = ?run_ctx.discord_user_id,
            user_name = ?run_ctx.discord_user_name.as_deref(),
            question_preview = %run_ctx.original_user_question.chars().take(60).collect::<String>(),
            "request run context"
        );

        // When Discord user asks for screenshots to be sent here, focus on current task only (no prior chat).
        // Skip clearing on verification retry so the model keeps context (e.g. original request, cookie consent retry).
        if !is_verification_retry && discord_reply_channel_id.is_some() {
            let q = question.to_lowercase();
            if q.contains("screenshot")
                && (q.contains("send") || q.contains("here") || q.contains("discord"))
                && conversation_history
                    .as_ref()
                    .is_some_and(|h| !h.is_empty())
                {
                    info!(
                        "Agent router: clearing history for Discord screenshot-send request (focus on current task)"
                    );
                    conversation_history = Some(vec![]);
                }
        }

        let (model_override, skill_content, mut max_tool_iterations) =
            if let Some(ref a) = agent_override {
                (
                    a.model.clone().or(model_override),
                    Some(a.combined_prompt.clone()),
                    a.max_tool_iterations,
                )
            } else {
                (
                    model_override,
                    skill_content,
                    15u32, // default when no agent override
                )
            };
        if escalation {
            max_tool_iterations = max_tool_iterations.saturating_add(10);
            info!(
                "Agent router: escalation mode — max_tool_iterations raised to {}",
                max_tool_iterations
            );
        }

        if let Some(ref model) = model_override {
            let available = list_ollama_models()
                .await
                .map_err(|e| format!("Could not list models: {}", e))?;
            let found = available
                .iter()
                .any(|m| m == model || m.starts_with(&format!("{}:", model)));
            if !found {
                return Err(format!(
                    "Model '{}' not found. Available: {}",
                    model,
                    available.join(", ")
                ));
            }
        }

        let (endpoint, effective_model, api_key) = {
            let guard = get_ollama_client().lock().map_err(|e| e.to_string())?;
            let client = guard
                .as_ref()
                .ok_or_else(|| "Ollama not configured".to_string())?;
            let effective = model_override.clone().unwrap_or_else(|| {
                read_ollama_fast_model_from_env_or_config()
                    .unwrap_or_else(|| client.config.model.clone())
            });
            // Prefer local model: if the chosen model is cloud (e.g. qwen3.5:cloud) and we have no
            // explicit override, use first local model so Discord/scheduler work without ollama.com auth.
            let effective = if model_override.is_some() {
                effective
            } else if crate::ollama::models::is_cloud_model(&effective) {
                crate::ollama::models::get_first_local_model_name().unwrap_or(effective)
            } else {
                effective
            };
            let api_key = client
                .config
                .api_key
                .as_ref()
                .and_then(|acc| crate::security::get_credential(acc).ok().flatten())
                .or_else(read_ollama_api_key_from_env_or_config);
            (client.config.endpoint.clone(), effective, api_key)
        };
        // Use resolved model (local when default was cloud) for all chat calls in this request.
        let model_override = Some(effective_model.clone());
        let model_info =
            crate::ollama::get_model_info(&endpoint, &effective_model, api_key.as_deref())
                .await
                .unwrap_or_else(|_| crate::ollama::ModelInfo::default());

        let send_status = |msg: &str| {
            if let Some(ref tx) = status_tx {
                let _ = tx.send(msg.to_string());
            }
        };

        let question_for_plan_and_exec = if escalation {
            format!(
                "[The user is not satisfied and wants the task actually completed. You MUST use tools to fulfill the request; do not reply with only text.]\n\n{}",
                question
            )
        } else {
            question.to_string()
        };

        let q_preview: String = question.chars().take(120).collect();
        if question.len() > 120 {
            info!(
                "Agent router [{}]: starting (question: {}... [{} chars])",
                request_id,
                q_preview,
                question.len()
            );
        } else {
            info!(
                "Agent router [{}]: starting (question: {})",
                request_id, q_preview
            );
        }
        // Discord message limit 2000; wrapper ~50 chars → leave room for full question
        const DISCORD_STATUS_QUESTION_MAX: usize = 1940;
        let truncate_status = |s: &str, max: usize| {
            let taken: String = s.chars().take(max).collect();
            if s.chars().count() > max {
                format!("{}…", taken)
            } else {
                taken
            }
        };
        send_status(&format!(
            "Asking Ollama for a plan (sending your question: \"{}\")…",
            truncate_status(question, DISCORD_STATUS_QUESTION_MAX)
        ));

        // Criteria at start: extract 1–3 success criteria to feed into end verification
        send_status("Extracting success criteria…");
        let success_criteria = if let Some(criteria) = success_criteria_override.clone() {
            info!(
                "Agent router [{}]: reusing {} request-local success criteria",
                request_id,
                criteria.len()
            );
            Some(sanitize_success_criteria(
                &request_for_verification,
                criteria,
            ))
        } else if is_redmine_review_or_summarize_only(&request_for_verification) {
            info!(
                "Agent router: review/summarize-only Redmine request — overriding success criteria to summary-only"
            );
            Some(vec![
                "Summary of ticket content provided to the user.".to_string()
            ])
        } else if is_redmine_time_entries_request(&request_for_verification) {
            info!(
                "Agent router: Redmine time-entries request — overriding success criteria to Redmine data-only"
            );
            Some(vec![
                "Actual Redmine time entries for the requested period were fetched.".to_string(),
                "A clear summary of hours or worked tickets was provided from that Redmine data."
                    .to_string(),
            ])
        } else if is_news_query(&request_for_verification) {
            info!(
                "Agent router: news/current-events request — using news success criteria"
            );
            Some(vec![
                "At least 2 named sources (publication or site name).".to_string(),
                "Dates included when available in the search results.".to_string(),
                "Short factual summary or bullet list (concise is OK).".to_string(),
            ])
        } else {
            extract_success_criteria(
                &request_for_verification,
                model_override.clone(),
                options_override.clone(),
            )
            .await
            .map(|criteria| sanitize_success_criteria(&request_for_verification, criteria))
        };
        let criteria_status = match &success_criteria {
            Some(c) if !c.is_empty() => {
                info!(
                    "Agent router [{}]: extracted {} success criteria",
                    request_id,
                    c.len()
                );
                truncate_status(&c.join("; "), 200)
            }
            _ => {
                tracing::debug!(
                    "Agent router: no success criteria extracted (verification will use request summary only)"
                );
                "(none)".to_string()
            }
        };
        send_status(&format!(
            "EDIT:Extracted success criteria: {}",
            criteria_status
        ));

        let mut is_new_topic = false;
        // 6.A New-topic check: one short LLM call when we have history and use a local model.
        // Skip when using a cloud model to minimize cost (only when cloud do we care about extra calls).
        // Skip on verification retry so the model keeps context (original request, cookie consent, etc.).
        if is_verification_retry || crate::ollama::models::is_cloud_model(&effective_model) {
            tracing::debug!(
                request_id = %request_id,
                verification_retry = is_verification_retry,
                "skipping new-topic check (retry or cloud model)"
            );
        } else if let Some(ref hist) = conversation_history {
            const NEW_TOPIC_MIN_HISTORY: usize = 2;
            if hist.len() >= NEW_TOPIC_MIN_HISTORY {
                send_status("Checking if new topic…");
                let summary = summarize_last_turns(hist, 3);
                match detect_new_topic(question, &summary, &effective_model).await {
                    Ok(true) => {
                        info!(
                            "Agent router [{}]: NEW_TOPIC — using no prior context; any verification retry will use only request-local criteria",
                            request_id
                        );
                        is_new_topic = true;
                    }
                    Ok(false) => {
                        info!(
                            "Agent router [{}]: SAME_TOPIC — keeping prior context",
                            request_id
                        );
                    }
                    Err(e) => {
                        tracing::debug!(
                            request_id = %request_id,
                            "new-topic check failed (keeping history): {}",
                            e
                        );
                    }
                }
            }
        }

        const CONVERSATION_HISTORY_CAP: usize = 20;
        let raw_history: Vec<crate::ollama::ChatMessage> = conversation_history
            .unwrap_or_default()
            .into_iter()
            .rev()
            .take(CONVERSATION_HISTORY_CAP)
            .rev()
            .collect();
        if raw_history.is_empty() {
            info!("Agent router [{}]: no prior session", request_id);
        } else {
            info!(
                "Agent router [{}]: prior session {} messages (capped at {})",
                request_id,
                raw_history.len(),
                CONVERSATION_HISTORY_CAP
            );
        }

        let conversation_history: Vec<crate::ollama::ChatMessage> = if raw_history.len()
            >= COMPACTION_THRESHOLD
        {
            // Skip compaction when session has no real conversational value (task-008 Phase 5).
            let pairs: Vec<(String, String)> = raw_history
                .iter()
                .map(|m| (m.role.clone(), m.content.clone()))
                .collect();
            let conversational = crate::session_memory::count_conversational_messages(&pairs);
            if conversational < MIN_CONVERSATIONAL_FOR_COMPACTION {
                info!(
                    "Session compaction [{}]: skipped (no real conversational value: {} conversational messages, need at least {}); keeping full history ({} messages)",
                    request_id, conversational, MIN_CONVERSATIONAL_FOR_COMPACTION, raw_history.len()
                );
                raw_history
                    .into_iter()
                    .map(|mut msg| {
                        if msg.role == "assistant" && looks_like_discord_401_confusion(&msg.content) {
                            msg.content.push_str("\n\n[SYSTEM CORRECTION: The above 401 was from FETCH_URL (no token). Use DISCORD_API instead.]");
                        }
                        msg
                    })
                    .collect()
            } else {
            send_status("Compacting session memory…");
            info!(
                "Session compaction [{}]: {} messages exceed threshold ({}), compacting",
                request_id,
                raw_history.len(),
                COMPACTION_THRESHOLD
            );
            match compact_conversation_history(&raw_history, question, discord_reply_channel_id).await {
                Ok((context, lessons)) => {
                    info!(
                        "Session compaction [{}]: produced context ({} chars), lessons: {}",
                        request_id,
                        context.len(),
                        lessons.as_ref().map(|_| "present").unwrap_or("none")
                    );
                    if let Some(ref lesson_text) = lessons {
                        let memory_path = discord_reply_channel_id
                            .map(crate::config::Config::memory_file_path_for_discord_channel)
                            .unwrap_or_else(crate::config::Config::memory_file_path);
                        for line in lesson_text.lines() {
                            let line = line.trim().trim_start_matches("- ").trim();
                            if !line.is_empty() && line.len() > 5 {
                                let entry = format!("- {}\n", line);
                                let _ = append_to_file(&memory_path, &entry);
                            }
                        }
                        info!("Session compaction [{}]: wrote lessons to {:?}", request_id, memory_path);
                    }
                    let context_lower = context.to_lowercase();
                    let not_needed = context_lower.contains("not needed for this request")
                        || context_lower.contains("covered different topics");
                    if let Some(channel_id) = discord_reply_channel_id {
                        let compacted = vec![
                            ("system".to_string(), context.clone()),
                            ("user".to_string(), question.to_string()),
                        ];
                        crate::session_memory::replace_session("discord", channel_id, compacted);
                    }
                    if is_new_topic || not_needed {
                        info!(
                            "Session compaction: prior context not relevant to current question (new topic), using no prior context"
                        );
                        vec![]
                    } else {
                        info!(
                            "Session compaction: replaced {} messages with summary ({} chars)",
                            raw_history.len(),
                            context.len()
                        );
                        vec![crate::ollama::ChatMessage {
                            role: "system".to_string(),
                            content: format!(
                                "Previous session context (compacted from {} messages):\n\n{}",
                                raw_history.len(),
                                context
                            ),
                            images: None,
                        }]
                    }
                }
                Err(e) => {
                    let n = raw_history.len();
                    let err_s = e.to_string();
                    let is_skip = err_s.contains("no real conversational value");
                    if is_skip {
                        info!(
                            "Session compaction skipped: {}; keeping full history ({} messages)",
                            err_s, n
                        );
                    } else {
                        let msg = if err_s.to_lowercase().contains("unauthorized")
                            || err_s.contains("401")
                        {
                            format!(
                                "Session compaction failed: {} (use a local model for compaction; cloud models may require auth). Keeping full history ({} messages) for this request.",
                                e, n
                            )
                        } else {
                            format!(
                                "Session compaction failed: {}; keeping full history ({} messages) for this request.",
                                e, n
                            )
                        };
                        tracing::warn!("{}", msg);
                    }
                    raw_history
                    .into_iter()
                    .map(|mut msg| {
                        if msg.role == "assistant" && looks_like_discord_401_confusion(&msg.content) {
                            msg.content.push_str("\n\n[SYSTEM CORRECTION: The above 401 was from FETCH_URL (no token). Use DISCORD_API instead.]");
                        }
                        msg
                    })
                    .collect()
                }
            }
            }
        } else if is_new_topic {
            vec![]
        } else {
            raw_history
            .into_iter()
            .map(|mut msg| {
                if msg.role == "assistant" && looks_like_discord_401_confusion(&msg.content) {
                    msg.content.push_str("\n\n[SYSTEM CORRECTION: The above 401 was from FETCH_URL (no token). Use DISCORD_API instead.]");
                }
                msg
            })
            .collect()
        };
        if !conversation_history.is_empty() {
            info!(
                "Agent router: using {} prior messages as context",
                conversation_history.len()
            );
        }

        let from_discord = discord_reply_channel_id.is_some();
        let discord_platform_formatting: String = if from_discord {
            info!("Agent router: Discord reply — injecting platform formatting (no tables, wrap links in <>)");
            if discord_is_dm == Some(false) {
                format!(
                    "{}{}",
                    DISCORD_PLATFORM_FORMATTING,
                    DISCORD_GROUP_CHANNEL_GUIDANCE
                )
            } else {
                DISCORD_PLATFORM_FORMATTING.to_string()
            }
        } else {
            String::new()
        };
        let agent_descriptions = build_agent_descriptions(from_discord, Some(question)).await;
        info!(
            "Agent router: agent list built ({} chars)",
            agent_descriptions.len()
        );

        // When no agent/skill override, prepend soul (~/.mac-stats/agents/soul.md) so Ollama gets personality/vibe in context.
        let router_soul = skill_content.as_ref().map_or_else(
            || {
                let s = load_soul_content();
                if s.is_empty() {
                    format!(
                        "You are mac-stats v{}.\n\n",
                        crate::config::Config::version()
                    )
                } else {
                    format!(
                        "{}\n\nYou are mac-stats v{}.\n\n",
                        s,
                        crate::config::Config::version()
                    )
                }
            },
            |_| String::new(),
        );

        let discord_user_context = match (discord_user_id, &discord_user_name) {
            (Some(id), name_opt) => {
                let name = name_opt.as_deref().unwrap_or("").to_string();
                let stored = crate::user_info::get_user_details(id);
                let display_name = stored
                    .as_ref()
                    .and_then(|d| d.display_name.as_deref())
                    .filter(|s| !s.is_empty())
                    .unwrap_or(name.as_str());
                let mut ctx = if !display_name.is_empty() {
                    format!(
                        "You are talking to Discord user **{}** (user id: {}). Use this when addressing the user or when calling Discord API with this user.",
                        display_name, id
                    )
                } else {
                    format!(
                        "You are talking to Discord user (user id: {}). Use this when calling Discord API with this user.",
                        id
                    )
                };
                if let Some(ref details) = stored {
                    let extra = crate::user_info::format_user_details_for_context(details);
                    if !extra.is_empty() {
                        ctx.push_str(&format!("\nUser details: {}.", extra));
                    }
                }
                ctx.push_str("\n\n");
                ctx
            }
            _ => String::new(),
        };

        // --- Pre-routing: deterministic tool dispatch for unambiguous patterns ---
        // Screenshot + URL → BROWSER_SCREENSHOT; "run <command>" → RUN_CMD; ticket → REDMINE_API.
        let pre_routed_recommendation = extract_screenshot_recommendation(question).or_else(|| {
            if crate::commands::run_cmd::is_local_cmd_allowed() {
                let q = question.trim();
                let q_lower = q.to_lowercase();
                let cmd_rest = if let Some(cmd) = extract_last_prefixed_argument(q, "RUN_CMD:") {
                    cmd
                } else if q_lower.starts_with("run command:") {
                    q[12..].trim().to_string() // "run command:".len() == 12
                } else if q_lower.starts_with("run ") {
                    q[4..].trim().to_string() // "run ".len() == 4
                } else {
                    String::new()
                };
                if !cmd_rest.is_empty() {
                    let rec = format!("RUN_CMD: {}", cmd_rest);
                    info!(
                        "Agent router: pre-routed to RUN_CMD (run command): {}",
                        crate::logging::ellipse(&cmd_rest, 60)
                    );
                    Some(rec)
                } else if crate::redmine::is_configured() {
                    let redmine_request = redmine_request_for_routing(
                        q,
                        &request_for_verification,
                        is_verification_retry,
                    );
                    let redmine_request_lower = redmine_request.to_lowercase();
                    if is_redmine_time_entries_request(redmine_request) {
                        let (from, to) = redmine_time_entries_range(redmine_request);
                        let rec = format!(
                            "REDMINE_API: GET /time_entries.json?from={}&to={}&limit=100",
                            from, to
                        );
                        info!(
                            "Agent router: pre-routed to REDMINE_API for time entries ({}..{})",
                            from, to
                        );
                        Some(rec)
                    } else {
                        let ticket_id = extract_ticket_id(&redmine_request_lower);
                        let wants_update = redmine_request_lower.contains("update")
                            || redmine_request_lower.contains("add comment")
                            || redmine_request_lower.contains("with the next steps")
                            || redmine_request_lower.contains("post a comment")
                            || redmine_request_lower.contains("write ")
                            || redmine_request_lower.contains("put ");
                        if let Some(id) = ticket_id
                            .filter(|_| {
                                redmine_request_lower.contains("ticket")
                                    || redmine_request_lower.contains("issue")
                                    || redmine_request_lower.contains("redmine")
                            })
                            .filter(|_| !wants_update)
                        {
                            let rec = format!(
                                "REDMINE_API: GET /issues/{}.json?include=journals,attachments",
                                id
                            );
                            info!("Agent router: pre-routed to REDMINE_API for ticket #{}", id);
                            Some(rec)
                        } else {
                            None
                        }
                    }
                } else {
                    None
                }
            } else if crate::redmine::is_configured() {
                let redmine_request = redmine_request_for_routing(
                    question,
                    &request_for_verification,
                    is_verification_retry,
                );
                let redmine_request_lower = redmine_request.to_lowercase();
                if is_redmine_time_entries_request(redmine_request) {
                    let (from, to) = redmine_time_entries_range(redmine_request);
                    let rec = format!(
                        "REDMINE_API: GET /time_entries.json?from={}&to={}&limit=100",
                        from, to
                    );
                    info!(
                        "Agent router: pre-routed to REDMINE_API for time entries ({}..{})",
                        from, to
                    );
                    Some(rec)
                } else {
                    let ticket_id = extract_ticket_id(&redmine_request_lower);
                    let wants_update = redmine_request_lower.contains("update")
                        || redmine_request_lower.contains("add comment")
                        || redmine_request_lower.contains("with the next steps")
                        || redmine_request_lower.contains("post a comment")
                        || redmine_request_lower.contains("write ")
                        || redmine_request_lower.contains("put ");
                    if let Some(id) = ticket_id
                        .filter(|_| {
                            redmine_request_lower.contains("ticket")
                                || redmine_request_lower.contains("issue")
                                || redmine_request_lower.contains("redmine")
                        })
                        .filter(|_| !wants_update)
                    {
                        let rec = format!(
                            "REDMINE_API: GET /issues/{}.json?include=journals,attachments",
                            id
                        );
                        info!("Agent router: pre-routed to REDMINE_API for ticket #{}", id);
                        Some(rec)
                    } else {
                        None
                    }
                }
            } else {
                None
            }
        });

        // --- Planning step: ask Ollama how it would solve the question (skip if pre-routed) ---
        let mut recommendation = if let Some(pre_routed) = pre_routed_recommendation {
            info!("Agent router: skipping LLM planning (pre-routed)");
            pre_routed
        } else {
            info!(
                "Agent router [{}]: planning step — asking Ollama for RECOMMEND",
                request_id
            );
            let planning_prompt = crate::config::Config::load_planning_prompt();
            let planning_system_content = match &skill_content {
                Some(skill) => format!(
                    "{}Additional instructions from skill:\n\n{}\n\n---\n\n{}\n\n{}{}",
                    discord_user_context, skill, planning_prompt, agent_descriptions,
                    discord_platform_formatting
                ),
                None => format!(
                    "{}{}{}\n\n{}{}",
                    router_soul, discord_user_context, planning_prompt, agent_descriptions,
                    discord_platform_formatting
                ),
            };
            let mut planning_messages: Vec<crate::ollama::ChatMessage> =
                vec![crate::ollama::ChatMessage {
                    role: "system".to_string(),
                    content: planning_system_content,
                    images: None,
                }];
            for msg in &conversation_history {
                planning_messages.push(msg.clone());
            }
            let model_hint = model_override.as_ref().map(|m| format!("\n\nFor this request the user selected Ollama model: {}. The app will use that model for the reply; recommend answering the question (or using an agent) with that in mind.", m)).unwrap_or_default();
            let today_utc = chrono::Utc::now().format("%Y-%m-%d");
            info!(
                "Agent router [{}]: planning RECOMMEND with current date (UTC) {}",
                request_id, today_utc
            );
            planning_messages.push(crate::ollama::ChatMessage {
                role: "user".to_string(),
                content: format!(
                    "Current date (UTC): {}.\n\nCurrent user question: {}{}\n\nReply with RECOMMEND: your plan.",
                    today_utc, question_for_plan_and_exec, model_hint
                ),
                images: attachment_images_base64.clone(),
            });
            let plan_response = send_ollama_chat_messages(
                planning_messages,
                model_override.clone(),
                options_override.clone(),
            )
            .await?;
            let mut rec = plan_response.message.content.trim().to_string();
            while rec.to_uppercase().starts_with("RECOMMEND: ")
                || rec.to_uppercase().starts_with("RECOMMEND:")
            {
                let prefix_len = if rec.len() >= 11 && rec[..11].to_uppercase() == "RECOMMEND: " {
                    11
                } else {
                    10
                };
                rec = rec[prefix_len..].trim().to_string();
            }
            rec
        };
        // When planner returned only "DONE: no" or "DONE: success" and the question indicates attachment failure, use a synthetic reply-only instruction so the user gets a proper summary (task: avoid bare DONE as execution plan).
        if is_bare_done_plan(&recommendation) {
            let q = question.to_lowercase();
            if q.contains("could not be attached") || q.contains("could not attach") {
                info!(
                    "Agent router [{}]: planner returned bare DONE plan but question indicates attachment failure; using synthetic summary instruction",
                    request_id
                );
                recommendation = "Reply with a brief summary of what was done and that the app could not attach the file(s) to Discord, then end with **DONE: no**. Do not run further tools.".to_string();
            }
        }
        info!(
            "Agent router [{}]: understood plan — {}",
            request_id,
            recommendation.chars().take(200).collect::<String>()
        );
        send_status(&format!(
            "Executing plan: {}…",
            truncate_status(&recommendation, 72)
        ));

        // Tools that benefit from a clean session (no stale conversation context).
        // Redmine reviews must not be polluted by prior turns — the model hallucinates.
        let fresh_session_tools = ["REDMINE_API"];
        let rec_upper = recommendation.to_uppercase();
        let needs_fresh_session = fresh_session_tools.iter().any(|t| rec_upper.contains(t));
        let conversation_history = if needs_fresh_session && !conversation_history.is_empty() {
            info!("Agent router: clearing conversation history for fresh-session tool");
            Vec::new()
        } else {
            conversation_history
        };

        // --- Execution: system prompt with agents + plan, then tool loop ---
        let execution_prompt_raw = crate::config::Config::load_execution_prompt();
        let execution_prompt = execution_prompt_raw.replace("{{AGENTS}}", &agent_descriptions);

        /// Log content in full if ≤500 chars (or always in -vv), else ellipse (first half + "..." + last half).
        const LOG_CONTENT_MAX: usize = 500;
        let log_verbosity = crate::logging::VERBOSITY.load(Ordering::Relaxed);
        let log_content = |content: &str| {
            let n = content.chars().count();
            if log_verbosity >= 2 || n <= LOG_CONTENT_MAX {
                content.to_string()
            } else {
                crate::logging::ellipse(content, LOG_CONTENT_MAX)
            }
        };

        // Include current system metrics so the model can answer accurately when the user asks about CPU, RAM, disk, etc.
        let metrics_block = crate::metrics::format_metrics_for_ai_context();
        let metrics_for_system = format!("\n\n{}", metrics_block);
        let model_identity = format!(
            "\n\nYou are replying as the Ollama model: **{}**. If the user asks which model you are (or what model you run on), name this model.",
            effective_model
        );
        // When Discord user asked for screenshots to be sent here, remind executor to actually run BROWSER_NAVIGATE + BROWSER_SCREENSHOT per URL.
        let discord_screenshot_reminder = if discord_reply_channel_id.is_some() {
            let q = question.to_lowercase();
            if q.contains("screenshot")
                && (q.contains("send") || q.contains("here") || q.contains("discord"))
            {
                "\n\n**Discord:** The user asked for screenshot(s) to be sent here. You MUST call BROWSER_NAVIGATE then BROWSER_SCREENSHOT: current for each URL; the app will attach the images to the reply."
            } else {
                ""
            }
        } else {
            ""
        };
        // When user asks "how to query Redmine API" for time/hours, executor should run at least one REDMINE_API call (e.g. GET /time_entries.json with current month) so the reply includes a real example.
        let redmine_howto_reminder = {
            let q = question.to_lowercase();
            if (q.contains("how to") || q.contains("how do"))
                && (q.contains("redmine")
                    || q.contains("time entries")
                    || q.contains("spent time")
                    || q.contains("hours"))
            {
                "\n\n**Redmine:** For \"how to query\" time entries or spent hours, run at least one REDMINE_API: GET /time_entries.json with from= and to= for the current month so your reply includes a real example (not only placeholder IDs or wrong year)."
            } else {
                ""
            }
        };
        // News/current-events: instruct model to answer with short bullet list, at least 2 named sources, dates when available.
        let news_format_reminder = if is_news_query(question) {
            "\n\n**News/current events:** Reply with a short bullet list or compact summary. Include at least 2 named sources (publication or site) and dates when available in the search results. Concise answers are OK."
        } else {
            ""
        };

        let normalized_recommendation = normalize_inline_tool_sequences(&recommendation);

        // Fast path: if the recommendation already contains a parseable tool call, execute it
        // directly instead of asking Ollama a second time to regurgitate the same tool line.
        let direct_tool = parse_tool_from_response(&recommendation);
        let (mut messages, mut response_content) = if let Some((ref tool, ref arg)) = direct_tool {
            let all_tools = parse_all_tools_from_response(&normalized_recommendation);
            if all_tools.len() > 1 {
                info!(
                    "Agent router: plan contains {} direct tools, will run in sequence: {} — skipping execution Ollama call",
                    all_tools.len(),
                    all_tools.iter().map(|(t, _)| t.as_str()).collect::<Vec<_>>().join(", ")
                );
            } else {
                info!(
                    "Agent router: plan contains direct tool call {}:{} — skipping execution Ollama call",
                    tool,
                    crate::logging::ellipse(arg, 60)
                );
            }
            let memory_block =
                load_memory_block_for_request(discord_reply_channel_id, load_global_memory);
            let execution_system_content = match &skill_content {
                Some(skill) => format!(
                    "{}Additional instructions from skill:\n\n{}\n\n---\n\n{}{}{}{}{}{}{}",
                    discord_user_context,
                    skill,
                    execution_prompt,
                    metrics_for_system,
                    discord_screenshot_reminder,
                    redmine_howto_reminder,
                    news_format_reminder,
                    discord_platform_formatting,
                    model_identity
                ),
                None => format!(
                    "{}{}{}{}{}{}{}{}{}{}",
                    router_soul,
                    memory_block,
                    discord_user_context,
                    execution_prompt,
                    metrics_for_system,
                    discord_screenshot_reminder,
                    redmine_howto_reminder,
                    news_format_reminder,
                    discord_platform_formatting,
                    model_identity
                ),
            };
            let mut msgs: Vec<crate::ollama::ChatMessage> = vec![crate::ollama::ChatMessage {
                role: "system".to_string(),
                content: execution_system_content,
                images: None,
            }];
            for msg in &conversation_history {
                msgs.push(msg.clone());
            }
            msgs.push(crate::ollama::ChatMessage {
                role: "user".to_string(),
                content: question_for_plan_and_exec.clone(),
                images: attachment_images_base64.clone(),
            });
            // Preserve multi-tool chains so the executor runs them step by step (not one RUN_CMD with the whole chain).
            let synthetic = if all_tools.len() > 1 {
                all_tools
                    .iter()
                    .map(|(t, a)| format!("{}: {}", t, a))
                    .collect::<Vec<_>>()
                    .join("\n")
            } else if normalized_recommendation.contains('\n') {
                normalized_recommendation.clone()
            } else {
                format!("{}: {}", tool, arg)
            };
            (msgs, synthetic)
        } else {
            info!(
                "Agent router: execution step — sending plan + question, starting tool loop (max {} tools)",
                max_tool_iterations
            );
            let memory_block =
                load_memory_block_for_request(discord_reply_channel_id, load_global_memory);
            let execution_system_content = match &skill_content {
                Some(skill) => format!(
                    "{}Additional instructions from skill:\n\n{}\n\n---\n\n{}{}{}{}{}{}\n\nYour plan: {}{}",
                    discord_user_context,
                    skill,
                    execution_prompt,
                    metrics_for_system,
                    discord_screenshot_reminder,
                    redmine_howto_reminder,
                    news_format_reminder,
                    discord_platform_formatting,
                    recommendation,
                    model_identity
                ),
                None => format!(
                    "{}{}{}{}{}{}{}{}{}\n\nYour plan: {}{}",
                    router_soul,
                    memory_block,
                    discord_user_context,
                    execution_prompt,
                    metrics_for_system,
                    discord_screenshot_reminder,
                    redmine_howto_reminder,
                    news_format_reminder,
                    discord_platform_formatting,
                    recommendation,
                    model_identity
                ),
            };
            let mut msgs: Vec<crate::ollama::ChatMessage> = vec![crate::ollama::ChatMessage {
                role: "system".to_string(),
                content: execution_system_content,
                images: None,
            }];
            for msg in &conversation_history {
                msgs.push(msg.clone());
            }
            msgs.push(crate::ollama::ChatMessage {
                role: "user".to_string(),
                content: question_for_plan_and_exec.clone(),
                images: attachment_images_base64.clone(),
            });
            let response = send_ollama_chat_messages(
                msgs.clone(),
                model_override.clone(),
                options_override.clone(),
            )
            .await?;
            let content = response.message.content.clone();
            let n = content.chars().count();
            info!(
                "Agent router: first response received ({} chars): {}",
                n,
                log_content(&content)
            );
            // Fallback: if Ollama returned empty but the recommendation contains a parseable tool,
            // synthesize the tool call so the tool loop can execute it.
            if n == 0 {
                if let Some((tool, arg)) = parse_tool_from_response(&recommendation) {
                    let synthetic = if normalized_recommendation.contains('\n') {
                        normalized_recommendation.clone()
                    } else {
                        format!("{}: {}", tool, arg)
                    };
                    info!(
                        "Agent router: empty response — falling back to tool from recommendation: {}",
                        crate::logging::ellipse(&synthetic, 80)
                    );
                    (msgs, synthetic)
                } else {
                    (msgs, content)
                }
            } else {
                (msgs, content)
            }
        };

        let mut tool_count: u32 = 0;
        // Browser mode: "headless" in question → no visible window. From Discord/scheduler/task (from_remote) → headless unless user explicitly asks to see the browser (so retries stay headless).
        let prefer_headless = if from_remote {
            !wants_visible_browser(question)
        } else {
            question.to_lowercase().contains("headless")
        };
        crate::browser_agent::set_prefer_headless_for_run(prefer_headless);
        // Paths to attach when replying on Discord (e.g. BROWSER_SCREENSHOT); only under ~/.mac-stats/screenshots/.
        let mut attachment_paths: Vec<PathBuf> = Vec::new();
        let mut screenshot_requested_by_tool_run = false;
        // Collect (agent_name, reply) for each AGENT call so we can append a conversation transcript when multiple agents participated.
        let mut agent_conversation: Vec<(String, String)> = Vec::new();
        // Dedupe repeated identical DISCORD_API calls so the model can't loop on the same request.
        let mut last_successful_discord_call: Option<(String, String)> = None;
        // Dedupe repeated identical RUN_CMD so the model can't loop (e.g. task says run once then TASK_APPEND/TASK_STATUS).
        let mut last_run_cmd_arg: Option<String> = None;
        // When the model does TASK_APPEND right after RUN_CMD, use this so the task file gets the full command output (not a summary).
        let mut last_run_cmd_raw_output: Option<String> = None;
        // Track the task file we're working on so we can append the full conversation at the end.
        let mut current_task_path: Option<std::path::PathBuf> = None;
        // When the model calls DONE we break the tool loop; strip the DONE line from the final reply.
        let mut exited_via_done: bool = false;
        // Last BROWSER_EXTRACT result (JS-rendered page text) for completion verification so we check against real content, not FETCH_URL HTML.
        let mut last_browser_extract: Option<String> = None;
        // Cap browser tools per run to prevent runaway NAVIGATE/CLICK loops (see docs/032_browser_loop_and_status_fix_plan.md).
        let mut browser_tool_count: u32 = 0;
        // Set when we skip a browser tool due to cap; used to append a user-facing note to the reply.
        let mut browser_tool_cap_reached: bool = false;
        // Repetition detection: refuse duplicate consecutive browser action (same NAVIGATE URL or same CLICK index).
        let mut last_browser_tool_arg: Option<(String, String)> = None;
        // For news requests: if the last PERPLEXITY_SEARCH returned only hub/landing/tag/standings pages, verification must not accept a confident news answer.
        let mut last_news_search_was_hub_only: Option<bool> = None;

        while tool_count < max_tool_iterations {
            let tools = parse_all_tools_from_response(&response_content);
            if tools.is_empty() {
                info!(
                    "Agent router: no tool call in response ({} chars), treating as final answer: {}",
                    response_content.chars().count(),
                    log_content(&response_content)
                );
                break;
            }
            if tools.len() > 1 {
                info!(
                    "Agent router: running {} tools in one turn: {}",
                    tools.len(),
                    tools
                        .iter()
                        .map(|(t, _)| t.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            let multi_tool_turn = tools.len() > 1;
            let mut done_claimed: Option<bool> = None;
            let mut tool_results: Vec<String> = Vec::with_capacity(tools.len());
            for (tool, arg) in tools {
                if tool_count >= max_tool_iterations {
                    break;
                }
                tool_count += 1;
                // Set when we execute a browser tool this iteration; used to update last_browser_tool_arg after the match.
                let mut executed_browser_tool_arg: Option<(String, String)> = None;
                // When the plan puts the whole chain in one line (e.g. PERPLEXITY_SEARCH: spanish newspapers then BROWSER_NAVIGATE...), pass only the search query to PERPLEXITY/BRAVE.
                let arg = if tool == "PERPLEXITY_SEARCH" || tool == "BRAVE_SEARCH" {
                    truncate_search_query_arg(&arg)
                } else {
                    arg
                };
                let arg_preview: String = arg.chars().take(80).collect();
                if arg.chars().count() > 80 {
                    info!(
                        "Agent router: running tool {}/{} — {} (arg: {}...)",
                        tool_count, max_tool_iterations, tool, arg_preview
                    );
                } else {
                    info!(
                        "Agent router: running tool {}/{} — {} (arg: {})",
                        tool_count, max_tool_iterations, tool, arg_preview
                    );
                }

                // Cap browser tools per run to prevent runaway loops (032_browser_loop_and_status_fix_plan).
                let is_browser_tool = matches!(
                    tool.as_str(),
                    "BROWSER_NAVIGATE"
                        | "BROWSER_CLICK"
                        | "BROWSER_INPUT"
                        | "BROWSER_SCROLL"
                        | "BROWSER_EXTRACT"
                        | "BROWSER_SEARCH_PAGE"
                        | "BROWSER_SCREENSHOT"
                );
                if is_browser_tool {
                    let normalized_arg = normalize_browser_tool_arg(&tool, &arg);
                    // Repetition detection: same action as previous step → skip and tell model (032).
                    if last_browser_tool_arg.as_ref() == Some(&(tool.clone(), normalized_arg.clone())) {
                        let msg = "Same browser action as previous step; use a different action or reply with DONE.".to_string();
                        tool_results.push(msg);
                        info!(
                            "Agent router: duplicate browser action skipped ({} with same arg)",
                            tool
                        );
                        continue;
                    }
                    if browser_tool_count >= MAX_BROWSER_TOOLS_PER_RUN {
                        browser_tool_cap_reached = true;
                        let msg = format!(
                            "Maximum browser actions per run reached ({}). Reply with your answer or DONE: success / DONE: no.",
                            MAX_BROWSER_TOOLS_PER_RUN
                        );
                        tool_results.push(msg);
                        info!(
                            "Agent router: browser tool cap reached ({}), skipping {}",
                            MAX_BROWSER_TOOLS_PER_RUN, tool
                        );
                        continue;
                    }
                    browser_tool_count += 1;
                    executed_browser_tool_arg = Some((tool.clone(), normalized_arg));
                    info!(
                        "Agent router: browser tool #{}/{} this run",
                        browser_tool_count, MAX_BROWSER_TOOLS_PER_RUN
                    );
                }
                if tool == "BROWSER_SCREENSHOT" {
                    screenshot_requested_by_tool_run = true;
                }

                let user_message = match tool.as_str() {
                    "DONE" => {
                        let arg_lower = arg.trim().to_lowercase();
                        let claimed_success = arg_lower.is_empty()
                            || arg_lower.contains("success")
                            || arg_lower.contains("yes")
                            || arg_lower.contains("true");
                        done_claimed = Some(claimed_success);
                        send_status(if claimed_success {
                            "✅ Task marked done (success)."
                        } else {
                            "⏹️ Task marked done (could not complete)."
                        });
                        info!(
                            "Agent router: model called DONE (success={}), exiting tool loop",
                            claimed_success
                        );
                        String::new()
                    }
                    "FETCH_URL" if arg.contains("discord.com") => {
                        let path = if let Some(pos) = arg.find("/api/v10") {
                            arg[pos + "/api/v10".len()..].to_string()
                        } else if let Some(pos) = arg.find("/api/") {
                            arg[pos + "/api".len()..].to_string()
                        } else {
                            String::new()
                        };
                        if !path.is_empty() {
                            info!(
                                "Agent router: redirecting FETCH_URL discord.com -> DISCORD_API GET {}",
                                path
                            );
                            send_status(&format!("Discord API: GET {}", path));
                            match crate::discord::api::discord_api_request("GET", &path, None).await
                            {
                                Ok(result) => format!(
                                    "Discord API result (GET {}):\n\n{}\n\nUse this to answer the user's question.",
                                    path, result
                                ),
                                Err(e) => format!(
                                    "Discord API failed (GET {}): {}. Try DISCORD_API: GET {} or delegate to AGENT: discord-expert.",
                                    path, e, path
                                ),
                            }
                        } else {
                            info!(
                                "Agent router: blocked FETCH_URL for discord.com (no API path). Redirecting to discord-expert."
                            );
                            "Cannot fetch discord.com pages directly. Discord requires authenticated API access. Use AGENT: discord-expert for all Discord tasks, or use DISCORD_API: GET <path> with the correct API endpoint.".to_string()
                        }
                    }
                    "FETCH_URL" => {
                        send_status(&format!(
                            "🌐 Fetching page at {}…",
                            crate::logging::ellipse(&arg, 45)
                        ));
                        info!("Discord/Ollama: FETCH_URL requested: {}", arg);
                        let url = arg.to_string();
                        let fetch_result = tokio::task::spawn_blocking(move || {
                            crate::commands::browser::fetch_page_content(&url)
                        })
                        .await
                        .map_err(|e| format!("Fetch task: {}", e))?
                        .map_err(|e| format!("Fetch page failed: {}", e));
                        match fetch_result {
                            Ok(body) => {
                                let estimated_used =
                                    (messages.iter().map(|m| m.content.len()).sum::<usize>()
                                        + agent_descriptions.len())
                                        / CHARS_PER_TOKEN
                                        + 50;
                                let body_fit = reduce_fetched_content_to_fit(
                                    &body,
                                    model_info.context_size_tokens,
                                    estimated_used as u32,
                                    model_override.clone(),
                                    options_override.clone(),
                                )
                                .await?;
                                format!(
                                    "Here is the page content:\n\n{}\n\nPlease answer the user's question based on this content.",
                                    body_fit
                                )
                            }
                            Err(e) => {
                                if e.contains("401") {
                                    info!(
                                        "Discord/Ollama: Fetch returned 401 Unauthorized, stopping"
                                    );
                                    "That URL returned 401 Unauthorized. Do not try another URL. Answer based on what you know.".to_string()
                                } else {
                                    info!(
                                        "Discord/Ollama: FETCH_URL failed: {}",
                                        crate::logging::ellipse(&e, 300)
                                    );
                                    let url_lower = arg.to_lowercase();
                                    let redmine_hint = if url_lower.contains("redmine")
                                        || url_lower.contains("/issues/")
                                    {
                                        " For Redmine tickets use REDMINE_API or say \"review ticket <id>\"."
                                    } else {
                                        ""
                                    };
                                    format!(
                                        "That URL could not be fetched (connection or server error).{redmine_hint} Answer without that page."
                                    )
                                }
                            }
                        }
                    }
                    "BROWSER_SCREENSHOT" => {
                        let url_arg = arg.trim().to_string();
                        let is_current =
                            url_arg.is_empty() || url_arg.eq_ignore_ascii_case("current");
                        // Browser-use style: screenshot only works on current page. Reject BROWSER_SCREENSHOT: <url>.
                        if !is_current {
                            info!(
                                "Agent router: rejecting BROWSER_SCREENSHOT: {} — use NAVIGATE first, then SCREENSHOT: current",
                                crate::logging::ellipse(&url_arg, 60)
                            );
                            format!(
                                "BROWSER_SCREENSHOT only works on the current page. Use BROWSER_NAVIGATE: {} first, then BROWSER_SCREENSHOT: current. Never use BROWSER_SCREENSHOT: <url>.",
                                url_arg
                            )
                        } else {
                            send_status("📸 Taking screenshot of current page");
                            match tokio::task::spawn_blocking(
                                crate::browser_agent::take_screenshot_current_page,
                            )
                            .await
                            {
                                Ok(Ok(path)) => {
                                    attachment_paths.push(path.clone());
                                    if let Some(ref tx) = status_tx {
                                        let _ = tx.send(format!("ATTACH:{}", path.display()));
                                    }
                                    format!(
                                        "Screenshot of current page saved to: {}.\n\nTell the user the screenshot was taken; the app will attach it in Discord.",
                                        path.display()
                                    )
                                }
                                Ok(Err(e)) => {
                                    info!(
                                        "Agent router [{}]: BROWSER_SCREENSHOT (current) failed: {}",
                                        request_id,
                                        crate::logging::ellipse(&e, 200)
                                    );
                                    format!(
                                        "Screenshot of current page failed: {}. (Use BROWSER_NAVIGATE and BROWSER_CLICK first with CDP; then BROWSER_SCREENSHOT: current. Chrome may need to be on port 9222.)",
                                        e
                                    )
                                }
                                Err(e) => format!("Screenshot task error: {}", e),
                            }
                        }
                    }
                    "BROWSER_NAVIGATE" => {
                        let raw_arg = arg.trim().to_string();
                        if raw_arg.is_empty() {
                            "BROWSER_NAVIGATE requires a URL (e.g. BROWSER_NAVIGATE: https://www.example.com). Please try again with a URL.".to_string()
                        } else if let Some(url_arg) = extract_browser_navigation_target(&raw_arg) {
                            let new_tab = raw_arg
                                .split_whitespace()
                                .any(|w| w.eq_ignore_ascii_case("new_tab"));
                            send_status(&format!(
                                "🧭 Navigating to {}…{}",
                                url_arg,
                                if new_tab { " (new tab)" } else { "" }
                            ));
                            info!(
                                "Agent router [{}]: BROWSER_NAVIGATE: URL sent to CDP: {} new_tab={}",
                                request_id, url_arg, new_tab
                            );
                            match tokio::task::spawn_blocking({
                                let u = url_arg.clone();
                                move || crate::browser_agent::navigate_and_get_state_with_options(&u, new_tab)
                            })
                            .await
                            {
                                Ok(Ok(state_str)) => state_str,
                                Ok(Err(cdp_err)) => {
                                    info!(
                                        "BROWSER_NAVIGATE CDP failed, ensuring Chrome on 9222 and retrying: {}",
                                        crate::logging::ellipse(&cdp_err, 120)
                                    );
                                    tokio::task::spawn_blocking(|| {
                                        crate::browser_agent::ensure_chrome_on_port(9222)
                                    })
                                    .await
                                    .ok();
                                    match tokio::task::spawn_blocking({
                                        let u = url_arg.clone();
                                        move || crate::browser_agent::navigate_and_get_state_with_options(&u, new_tab)
                                    })
                                    .await
                                    {
                                        Ok(Ok(state_str)) => state_str,
                                        Ok(Err(cdp_err2)) => {
                                            info!(
                                                "BROWSER_NAVIGATE CDP retry failed, trying HTTP fallback: {}",
                                                crate::logging::ellipse(&cdp_err2, 120)
                                            );
                                            match tokio::task::spawn_blocking(move || {
                                                crate::browser_agent::navigate_http(&url_arg)
                                            })
                                            .await
                                            {
                                                Ok(Ok(state_str)) => state_str,
                                                Ok(Err(http_err)) => format!(
                                                    "BROWSER_NAVIGATE failed (CDP: {}). HTTP fallback also failed: {}",
                                                    crate::logging::ellipse(&cdp_err2, 80),
                                                    http_err
                                                ),
                                                Err(e) => format!(
                                                    "BROWSER_NAVIGATE HTTP fallback task error: {}",
                                                    e
                                                ),
                                            }
                                        }
                                        Err(e) => {
                                            format!("BROWSER_NAVIGATE CDP retry task error: {}", e)
                                        }
                                    }
                                }
                                Err(e) => format!("BROWSER_NAVIGATE task error: {}", e),
                            }
                        } else {
                            append_latest_browser_state_guidance(&format!(
                                "BROWSER_NAVIGATE requires a concrete URL. The step {:?} was not executed because it did not contain a grounded browser target. This was an agent planning/parsing issue, not evidence about the site.",
                                raw_arg
                            ))
                        }
                    }
                    "BROWSER_GO_BACK" => {
                        send_status("🔙 Going back…");
                        info!("Agent router [{}]: BROWSER_GO_BACK", request_id);
                        match tokio::task::spawn_blocking(crate::browser_agent::go_back).await {
                            Ok(Ok(state_str)) => state_str,
                            Ok(Err(e)) => append_latest_browser_state_guidance(&format!(
                                "BROWSER_GO_BACK failed: {}",
                                e
                            )),
                            Err(e) => format!("BROWSER_GO_BACK task error: {}", e),
                        }
                    }
                    "BROWSER_CLICK" => {
                        let index_arg = arg.split_whitespace().next().unwrap_or("").trim();
                        let index = index_arg.parse::<u32>().ok();
                        let status_msg = match index {
                            Some(idx) => {
                                let label = crate::browser_agent::get_last_element_label(idx);
                                if let Some(l) = label {
                                    format!(
                                        "🖱️ Clicking element {} ({})",
                                        idx,
                                        crate::logging::ellipse(&l, 30)
                                    )
                                } else {
                                    format!("🖱️ Clicking element {}", idx)
                                }
                            }
                            None => format!(
                                "🖱️ Clicking element {}",
                                if index_arg.is_empty() { "?" } else { index_arg }
                            ),
                        };
                        send_status(&status_msg);
                        match index {
                    Some(idx) => {
                        info!("BROWSER_CLICK: index {}", idx);
                        match tokio::task::spawn_blocking(move || crate::browser_agent::click_by_index(idx)).await {
                            Ok(Ok(state_str)) => state_str,
                            Ok(Err(cdp_err)) => {
                                if should_use_http_fallback_after_browser_action_error(
                                    "BROWSER_CLICK",
                                    &cdp_err,
                                ) {
                                    match tokio::task::spawn_blocking(move || crate::browser_agent::click_http(idx)).await {
                                        Ok(Ok(state_str)) => state_str,
                                        Ok(Err(e)) => append_latest_browser_state_guidance(&format!("BROWSER_CLICK failed: {}", e)),
                                        Err(e) => format!("BROWSER_CLICK task error: {}", e),
                                    }
                                } else {
                                    append_latest_browser_state_guidance(&format!(
                                        "BROWSER_CLICK failed: {}",
                                        cdp_err
                                    ))
                                }
                            }
                            Err(e) => append_latest_browser_state_guidance(&format!("BROWSER_CLICK task error: {}", e)),
                        }
                    }
                    None => append_latest_browser_state_guidance("BROWSER_CLICK requires a numeric index (e.g. BROWSER_CLICK: 3). Use the index from the Current page Elements list."),
                }
                    }
                    "BROWSER_INPUT" => {
                        let mut parts = arg.trim().splitn(2, |c: char| c.is_whitespace());
                        let index_arg = parts.next().unwrap_or("").trim();
                        let index_for_status = index_arg.parse::<u32>().ok();
                        let status_msg = match index_for_status {
                            Some(idx) => {
                                let label = crate::browser_agent::get_last_element_label(idx);
                                if let Some(l) = label {
                                    format!(
                                        "✍️ Typing into element {} ({})…",
                                        idx,
                                        crate::logging::ellipse(&l, 30)
                                    )
                                } else {
                                    format!("✍️ Typing into element {}…", idx)
                                }
                            }
                            None => format!(
                                "✍️ Typing into element {}…",
                                if index_arg.is_empty() { "?" } else { index_arg }
                            ),
                        };
                        send_status(&status_msg);
                        let text = parts.next().unwrap_or("").trim().to_string();
                        let index = index_arg.parse::<u32>().ok();
                        match index {
                    Some(idx) => {
                        info!("BROWSER_INPUT: index {} ({} chars)", idx, text.len());
                        let text_clone = text.clone();
                        match tokio::task::spawn_blocking(move || crate::browser_agent::input_by_index(idx, &text_clone)).await {
                            Ok(Ok(state_str)) => state_str,
                            Ok(Err(cdp_err)) => {
                                if should_use_http_fallback_after_browser_action_error(
                                    "BROWSER_INPUT",
                                    &cdp_err,
                                ) {
                                    match tokio::task::spawn_blocking(move || crate::browser_agent::input_http(idx, &text)).await {
                                    Ok(Ok(state_str)) => state_str,
                                    Ok(Err(e)) => append_latest_browser_state_guidance(&format!("BROWSER_INPUT failed: {}", e)),
                                    Err(e) => format!("BROWSER_INPUT task error: {}", e),
                                }
                                } else {
                                    append_latest_browser_state_guidance(&format!(
                                        "BROWSER_INPUT failed: {}",
                                        cdp_err
                                    ))
                                }
                            }
                            Err(e) => append_latest_browser_state_guidance(&format!("BROWSER_INPUT task error: {}", e)),
                        }
                    }
                    None => append_latest_browser_state_guidance("BROWSER_INPUT requires a numeric index and text (e.g. BROWSER_INPUT: 4 search query). Use the index from the Current page Elements list."),
                }
                    }
                    "BROWSER_SCROLL" => {
                        let scroll_arg = if arg.trim().is_empty() {
                            "down".to_string()
                        } else {
                            arg.trim().to_string()
                        };
                        send_status(&format!(
                            "📜 Scrolling {}…",
                            crate::logging::ellipse(&scroll_arg, 20)
                        ));
                        match tokio::task::spawn_blocking(move || {
                            crate::browser_agent::scroll_page(&scroll_arg)
                        })
                        .await
                        {
                            Ok(Ok(state_str)) => state_str,
                            Ok(Err(e)) => {
                                info!(
                                    "BROWSER_SCROLL failed: {}",
                                    crate::logging::ellipse(&e, 200)
                                );
                                format!("BROWSER_SCROLL failed: {}", e)
                            }
                            Err(e) => format!("BROWSER_SCROLL task error: {}", e),
                        }
                    }
                    "BROWSER_EXTRACT" => {
                        send_status("📄 Extracting page text…");
                        match tokio::task::spawn_blocking(crate::browser_agent::extract_page_text)
                            .await
                        {
                            Ok(Ok(text)) => text,
                            Ok(Err(_cdp_err)) => {
                                match tokio::task::spawn_blocking(
                                    crate::browser_agent::extract_http,
                                )
                                .await
                                {
                                    Ok(Ok(text)) => text,
                                    Ok(Err(e)) => format!(
                                        "BROWSER_EXTRACT failed: {}. (Navigate to a page first with BROWSER_NAVIGATE.)",
                                        e
                                    ),
                                    Err(e) => format!("BROWSER_EXTRACT task error: {}", e),
                                }
                            }
                            Err(e) => format!("BROWSER_EXTRACT task error: {}", e),
                        }
                    }
                    "BROWSER_SEARCH_PAGE" => {
                        let pattern = arg.trim().to_string();
                        if pattern.is_empty() {
                            "BROWSER_SEARCH_PAGE requires a search pattern (e.g. BROWSER_SEARCH_PAGE: Ralf Röber). Use to find specific text on the current page.".to_string()
                        } else {
                            send_status(&format!(
                                "🔍 Searching page for \"{}\"…",
                                crate::logging::ellipse(&pattern, 30)
                            ));
                            match tokio::task::spawn_blocking(move || {
                                crate::browser_agent::search_page_text(&pattern)
                            })
                            .await
                            {
                                Ok(Ok(result)) => result,
                                Ok(Err(e)) => {
                                    info!(
                                        "BROWSER_SEARCH_PAGE failed: {}",
                                        crate::logging::ellipse(&e, 200)
                                    );
                                    format!(
                                        "BROWSER_SEARCH_PAGE failed: {}. (Navigate to a page first with BROWSER_NAVIGATE.)",
                                        e
                                    )
                                }
                                Err(e) => format!("BROWSER_SEARCH_PAGE task error: {}", e),
                            }
                        }
                    }
                    "BRAVE_SEARCH" => {
                        send_status(&format!(
                            "🌐 Searching the web for \"{}\"…",
                            crate::logging::ellipse(&arg, 35)
                        ));
                        info!("Discord/Ollama: BRAVE_SEARCH requested: {}", arg);
                        match crate::commands::brave::get_brave_api_key() {
                    Some(api_key) => match crate::commands::brave::brave_web_search(&arg, &api_key).await {
                        Ok(results) => format!(
                            "Brave Search results:\n\n{}\n\nUse these to answer the user's question.",
                            results
                        ),
                        Err(e) => format!("Brave Search failed: {}. Answer without search results.", e),
                    },
                    None => "Brave Search is not configured (no BRAVE_API_KEY in env or .config.env). Answer without search results.".to_string(),
                }
                    }
                    "PERPLEXITY_SEARCH" => {
                        send_status(&format!(
                            "🔎 Searching (Perplexity) for \"{}\"…",
                            crate::logging::ellipse(&arg, 35)
                        ));
                        info!("Discord/Ollama: PERPLEXITY_SEARCH requested: {}", arg);
                        let q_lower = question.to_lowercase();
                        let is_news_query = is_news_query(question);
                        let snippet_max = crate::config::Config::perplexity_snippet_max_chars();
                        let max_results = crate::config::Config::perplexity_max_results();
                        match search_perplexity_with_news_fallback(
                            question,
                            &arg,
                            max_results,
                            snippet_max,
                        )
                        .await
                        {
                            Ok((shaped_results, urls, filtered_any, refined_query_used)) => {
                                let want_screenshots = (q_lower.contains("screenshot")
                                    || q_lower.contains("screen shot"))
                                    && (q_lower.contains("visit")
                                        || q_lower.contains("url")
                                        || q_lower.contains(" 5 ")
                                        || q_lower.contains(" 3 ")
                                        || q_lower.contains("send me")
                                        || q_lower.contains("send the")
                                        || q_lower.contains("in discord")
                                        || q_lower.contains(" here "));
                                let urls: Vec<String> = urls
                                    .into_iter()
                                    .filter(|url| {
                                        url.starts_with("http://") || url.starts_with("https://")
                                    })
                                    .take(5)
                                    .collect();
                                // In verbose mode (Discord): brief feedback that results were received before next step.
                                const MAX_PERPLEXITY_SUMMARY_CHARS: usize = 380;
                                if status_tx.is_some() {
                                    let n = shaped_results.len();
                                    let titles: String = shaped_results
                                        .iter()
                                        .take(5)
                                        .map(|r| r.title.trim().to_string())
                                        .filter(|t| !t.is_empty())
                                        .collect::<Vec<_>>()
                                        .join(", ");
                                    let summary = build_perplexity_verbose_summary(
                                        n,
                                        titles,
                                        MAX_PERPLEXITY_SUMMARY_CHARS,
                                    );
                                    send_status(&summary);
                                }
                                let num_results = shaped_results.len();
                                let search_had_article_like = is_news_query
                                    && shaped_results
                                        .iter()
                                        .any(|r| {
                                            is_likely_article_like_result(
                                                &r.title,
                                                &r.url,
                                                &r.snippet,
                                            )
                                        });
                                if is_news_query {
                                    last_news_search_was_hub_only = Some(!search_had_article_like);
                                    if !search_had_article_like {
                                        info!(
                                            "Agent router: news search returned only hub/landing pages; completion verification will require article-grade evidence"
                                        );
                                    }
                                }
                                // Structured markdown: numbered results with explicit Title/URL/Date/Snippet so the model parses and cites reliably.
                                let results: String = shaped_results
                                    .into_iter()
                                    .enumerate()
                                    .map(|(i, r)| {
                                        let date_str = r
                                            .date
                                            .as_deref()
                                            .or(r.last_updated.as_deref())
                                            .unwrap_or("")
                                            .trim();
                                        let date_line = if date_str.is_empty() {
                                            String::new()
                                        } else {
                                            format!("- **Date:** {}\n", date_str)
                                        };
                                        let page_type = if is_news_query
                                            && is_likely_article_like_result(
                                                &r.title, &r.url, &r.snippet,
                                            ) {
                                            "- **Page type:** article-like\n"
                                        } else if is_news_query {
                                            "- **Page type:** hub/landing page\n"
                                        } else {
                                            ""
                                        };
                                        format!(
                                            "### {}. {}\n- **URL:** {}\n{}{}- **Snippet:** {}",
                                            i + 1,
                                            r.title,
                                            r.url,
                                            date_line,
                                            page_type,
                                            r.snippet
                                        )
                                    })
                                    .collect::<Vec<_>>()
                                    .join("\n\n");
                                let mut result_text = if results.is_empty() {
                                    "Perplexity search returned no results. Answer from general knowledge.".to_string()
                                } else {
                                    format!(
                                        "## Perplexity Search Results ({} items)\n\n{}\n\nUse these to answer the user's question. Cite source number, title or URL, and date when given.",
                                        num_results, results
                                    )
                                };
                                if is_news_query && !results.is_empty() {
                                    result_text.push_str(&build_perplexity_news_tool_suffix(
                                        search_had_article_like,
                                        refined_query_used.as_deref(),
                                        filtered_any,
                                    ));
                                }
                                if want_screenshots && !urls.is_empty() {
                                    info!(
                                        "Agent router: auto-visit and screenshot for {} URLs (user asked for screenshots)",
                                        urls.len()
                                    );
                                    for (i, url) in urls.iter().enumerate() {
                                        send_status(&format!(
                                            "🧭 Visiting {} of {}…",
                                            i + 1,
                                            urls.len()
                                        ));
                                        let nav_result = tokio::task::spawn_blocking({
                                            let u = url.clone();
                                            move || crate::browser_agent::navigate_and_get_state(&u)
                                        })
                                        .await;
                                        match nav_result {
                                            Ok(Ok(_)) => {
                                                send_status(&format!(
                                                    "📸 Taking screenshot {} of {}…",
                                                    i + 1,
                                                    urls.len()
                                                ));
                                                let shot_result = tokio::task::spawn_blocking(crate::browser_agent::take_screenshot_current_page).await;
                                                if let Ok(Ok(path)) = shot_result {
                                                    attachment_paths.push(path.clone());
                                                    if let Some(ref tx) = status_tx {
                                                        let _ = tx.send(format!(
                                                            "ATTACH:{}",
                                                            path.display()
                                                        ));
                                                    }
                                                    info!(
                                                        "Agent router: auto-screenshot {} saved to {:?}",
                                                        i + 1,
                                                        path
                                                    );
                                                }
                                            }
                                            Ok(Err(e)) => {
                                                info!(
                                                    "Agent router: auto-navigate {} failed: {}",
                                                    url,
                                                    crate::logging::ellipse(&e, 80)
                                                );
                                            }
                                            Err(e) => {
                                                info!(
                                                    "Agent router: auto-navigate task error: {}",
                                                    e
                                                );
                                            }
                                        }
                                    }
                                    result_text.push_str(&format!(
                                "\n\nI navigated to and took screenshots of {} page(s). The app will attach them in Discord.",
                                urls.len()
                            ));
                                }
                                info!(
                                    "Agent router [{}]: PERPLEXITY_SEARCH returned {} results, blob {} bytes",
                                    request_id,
                                    num_results,
                                    result_text.len()
                                );
                                result_text
                            }
                            Err(e) => {
                                if status_tx.is_some() {
                                    send_status(&format!(
                                        "Perplexity search failed: {}",
                                        crate::logging::ellipse(&e.to_string(), 120)
                                    ));
                                }
                                format!(
                                    "Perplexity search failed: {}. Answer without search results.",
                                    e
                                )
                            }
                        }
                    }
                    // RUN_JS: agent-triggered; runs with process privileges via run_js_via_node.
                    // Treat agent output as untrusted code. Audit log code length + hash only (no full body).
                    "RUN_JS" => {
                        const CODE_PREVIEW_LEN: usize = 50;
                        let code_preview: String = arg
                            .trim()
                            .lines()
                            .next()
                            .unwrap_or(arg.trim())
                            .chars()
                            .take(CODE_PREVIEW_LEN)
                            .collect();
                        let code_label = if arg.trim().chars().count() > CODE_PREVIEW_LEN {
                            format!("{}…", code_preview.trim())
                        } else {
                            code_preview.trim().to_string()
                        };
                        let code_ref = if code_label.is_empty() {
                            "…"
                        } else {
                            &code_label
                        };
                        send_status(&format!("Running code: {}…", code_ref));
                        // Audit: code length + content hash only (no full code in logs).
                        let code_len = arg.chars().count();
                        let code_hash = {
                            use std::hash::{Hash, Hasher};
                            let mut h = std::collections::hash_map::DefaultHasher::new();
                            arg.hash(&mut h);
                            h.finish()
                        };
                        info!(
                            "RUN_JS audit: len={} hash_hex={:016x} preview={}",
                            code_len, code_hash, code_ref
                        );
                        match run_js_via_node(&arg) {
                            Ok(result) => format!(
                                "JavaScript result:\n\n{}\n\nUse this to answer the user's question.",
                                result
                            ),
                            Err(e) => {
                                info!("Discord/Ollama: RUN_JS failed: {}", e);
                                format!(
                                    "JavaScript execution failed: {}. Answer the user's question without running code.",
                                    e
                                )
                            }
                        }
                    }
                    "SKILL" => {
                        send_status("Using skill…");
                        let arg = arg.trim();
                        let (selector, task_message) = if let Some(space_idx) = arg.find(' ') {
                            let (sel, rest) = arg.split_at(space_idx);
                            (sel.trim(), rest.trim())
                        } else {
                            (arg, "")
                        };
                        let skills = crate::skills::load_skills();
                        match crate::skills::find_skill_by_number_or_topic(&skills, selector) {
                            Some(skill) => {
                                send_status(&format!(
                                    "Using skill {}-{}…",
                                    skill.number, skill.topic
                                ));
                                info!(
                                    "Agent router: using skill {} ({}) — new session (no main context)",
                                    skill.number, skill.topic
                                );
                                let user_msg = if task_message.is_empty() {
                                    question
                                } else {
                                    task_message
                                };
                                match run_skill_ollama_session(
                                    &skill.content,
                                    user_msg,
                                    model_override.clone(),
                                    options_override.clone(),
                                )
                                .await
                                {
                                    Ok(result) => format!(
                                        "Skill \"{}-{}\" result:\n\n{}\n\nUse this to answer the user's question.",
                                        skill.number, skill.topic, result
                                    ),
                                    Err(e) => {
                                        info!("Agent router: SKILL session failed: {}", e);
                                        format!(
                                            "Skill \"{}-{}\" failed: {}. Answer without this result.",
                                            skill.number, skill.topic, e
                                        )
                                    }
                                }
                            }
                            None => {
                                info!(
                                    "Agent router: SKILL unknown selector \"{}\" (available: {:?})",
                                    selector,
                                    skills
                                        .iter()
                                        .map(|s| format!("{}-{}", s.number, s.topic))
                                        .collect::<Vec<_>>()
                                );
                                format!(
                                    "Unknown skill \"{}\". Available skills: {}. Answer without using a skill.",
                                    selector,
                                    skills
                                        .iter()
                                        .map(|s| format!("{}-{}", s.number, s.topic))
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                )
                            }
                        }
                    }
                    "AGENT" => {
                        let arg = arg.trim();
                        let (selector, task_message) = if let Some(space_idx) = arg.find(' ') {
                            let (sel, rest) = arg.split_at(space_idx);
                            (sel.trim(), rest.trim())
                        } else {
                            (arg, "")
                        };
                        // Proxy agent: cursor-agent runs the CLI instead of Ollama (handoff when local model isn't enough).
                        let selector_lower = selector.to_lowercase();
                        if (selector_lower == "cursor-agent" || selector_lower == "cursor_agent")
                            && crate::commands::cursor_agent::is_cursor_agent_available()
                        {
                            let prompt = if task_message.is_empty() {
                                question.to_string()
                            } else {
                                task_message.to_string()
                            };
                            send_status("Running Cursor Agent…");
                            info!(
                                "Agent router: AGENT cursor-agent proxy (prompt {} chars)",
                                prompt.len()
                            );
                            let prompt_clone = prompt.clone();
                            match tokio::task::spawn_blocking(move || {
                                crate::commands::cursor_agent::run_cursor_agent(&prompt_clone)
                            })
                            .await
                            .map_err(|e| format!("Cursor Agent task: {}", e))
                            .and_then(|r| r)
                            {
                                Ok(result) => {
                                    info!(
                                        "Agent router: cursor-agent proxy completed ({} chars)",
                                        result.len()
                                    );
                                    format!(
                                        "Cursor Agent (proxy) result:\n\n{}\n\nUse this to answer the user's question.",
                                        result.trim()
                                    )
                                }
                                Err(e) => {
                                    info!("Agent router: cursor-agent proxy failed: {}", e);
                                    format!(
                                        "AGENT cursor-agent failed: {}. Answer without this result.",
                                        e
                                    )
                                }
                            }
                        } else if selector_lower == "cursor-agent"
                            || selector_lower == "cursor_agent"
                        {
                            "Cursor Agent is not available (cursor-agent CLI not on PATH). Answer without it.".to_string()
                        } else {
                        let agents = crate::agents::load_agents();
                        match crate::agents::find_agent_by_id_or_name(&agents, selector) {
                            Some(agent) => {
                                // Break out of AGENT: orchestrator loop when the last message was already
                                // an orchestrator result (model keeps re-invoking instead of replying).
                                let is_orchestrator = agent.id == "000";
                                let last_is_orchestrator_result = messages
                                    .last()
                                    .filter(|m| m.role == "user")
                                    .map(|m| m.content.starts_with("Agent \"Orchestrator\""))
                                    .unwrap_or(false);
                                if is_orchestrator && last_is_orchestrator_result {
                                    info!(
                                        "Agent router: skipping repeated AGENT: orchestrator (loop breaker)"
                                    );
                                    "The orchestrator already replied above. Reply with a one-sentence summary for the user and **DONE: success** or **DONE: no**. Do not output AGENT: orchestrator again.".to_string()
                                } else {
                                    let mut user_msg: String = if task_message.is_empty() {
                                        question.to_string()
                                    } else {
                                        task_message.to_string()
                                    };
                                    // When invoking discord-expert from Discord, fetch guild/channel metadata via API and inject so the agent has current context.
                                    let is_discord_expert =
                                        agent.slug.as_deref().is_some_and(|s| {
                                            s.eq_ignore_ascii_case("discord-expert")
                                        }) || agent.id == "004";
                                    let is_redmine_agent = agent
                                        .slug
                                        .as_deref()
                                        .is_some_and(|s| s.eq_ignore_ascii_case("redmine"))
                                        || agent.id == "006";
                                    if is_discord_expert {
                                        if let Some(channel_id) = discord_reply_channel_id {
                                            send_status("Fetching Discord guild/channel context…");
                                            match crate::discord::api::fetch_guild_channel_metadata(
                                                channel_id,
                                            )
                                            .await
                                            {
                                                Ok(meta) => {
                                                    user_msg = format!(
                                                        "Current Discord context (use these IDs in DISCORD_API calls):\n{}\n\nUser request: {}",
                                                        meta, user_msg
                                                    );
                                                    info!(
                                                        "Agent router: injected Discord guild/channel metadata for discord-expert (channel {})",
                                                        channel_id
                                                    );
                                                }
                                                Err(e) => {
                                                    tracing::debug!(
                                                        "Agent router: Discord metadata fetch failed (channel {}): {}",
                                                        channel_id,
                                                        e
                                                    );
                                                }
                                            }
                                        }
                                    }
                                    const STATUS_MSG_MAX: usize = 120;
                                    let preview: String =
                                        user_msg.chars().take(STATUS_MSG_MAX).collect();
                                    let status_text = if user_msg.chars().count() > STATUS_MSG_MAX {
                                        format!("{}…", preview)
                                    } else {
                                        preview
                                    };
                                    send_status(&format!(
                                        "{} -> Ollama: {}",
                                        agent.name, status_text
                                    ));
                                    match run_agent_ollama_session(
                                        agent,
                                        &user_msg,
                                        status_tx.as_ref(),
                                        load_global_memory,
                                    )
                                    .await
                                    {
                                        Ok(result) => {
                                            let label = format!("{} ({})", agent.name, agent.id);
                                            agent_conversation
                                                .push((label.clone(), result.trim().to_string()));
                                            format!(
                                                "Agent \"{}\" ({}) result:\n\n{}\n\nUse this to answer the user's question.",
                                                agent.name, agent.id, result
                                            )
                                        }
                                        Err(e) => {
                                            info!("Agent router: AGENT session failed: {}", e);
                                            if is_redmine_agent && is_agent_unavailable_error(&e) {
                                                format!(
                                                    "Agent \"{}\" ({}) failed: {}.\n\nRe-plan this request without AGENT: redmine. {} Do not use FETCH_URL and do not reply with only another RUN_CMD.",
                                                    agent.name,
                                                    agent.id,
                                                    e,
                                                    redmine_direct_fallback_hint(question)
                                                )
                                            } else {
                                                format!(
                                                    "Agent \"{}\" ({}) failed: {}. Answer without this result.",
                                                    agent.name, agent.id, e
                                                )
                                            }
                                        }
                                    }
                                }
                            }
                            None => {
                                let list: String = agents
                                    .iter()
                                    .map(|a| {
                                        a.slug.as_deref().unwrap_or(a.name.as_str()).to_string()
                                    })
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                info!(
                                    "Agent router: AGENT unknown selector \"{}\" (available: {})",
                                    selector, list
                                );
                                format!(
                                    "Unknown agent \"{}\". Available agents: {}. Answer without using an agent.",
                                    selector, list
                                )
                            }
                        }
                        }
                    }
                    "SCHEDULE" => {
                        if !allow_schedule {
                            info!("Agent router: SCHEDULE ignored (disabled in scheduler context)");
                            "Scheduling is not available when running from a scheduled task. Do not add a schedule; complete the task without scheduling."
                        .to_string()
                        } else {
                            let schedule_preview: String = arg.chars().take(50).collect();
                            let schedule_preview = schedule_preview.trim();
                            send_status(&format!(
                                "Scheduling: {}…",
                                if schedule_preview.is_empty() {
                                    "…"
                                } else {
                                    schedule_preview
                                }
                            ));
                            info!(
                                "Agent router: SCHEDULE requested (arg len={})",
                                arg.chars().count()
                            );
                            match parse_schedule_arg(&arg) {
                                Ok(ScheduleParseResult::Cron { cron_str, task }) => {
                                    let id = format!("discord-{}", chrono::Utc::now().timestamp());
                                    let reply_to_channel_id =
                                        discord_reply_channel_id.map(|u| u.to_string());
                                    match crate::scheduler::add_schedule(
                                        id.clone(),
                                        cron_str.clone(),
                                        task.clone(),
                                        reply_to_channel_id,
                                    ) {
                                        Ok(crate::scheduler::ScheduleAddOutcome::Added) => {
                                            info!(
                                                "Agent router: SCHEDULE added (id={}, cron={})",
                                                id, cron_str
                                            );
                                            let task_preview: String =
                                                task.chars().take(100).collect();
                                            format!(
                                                "Schedule added successfully. Schedule ID: **{}**. The scheduler will run this task (cron: {}): \"{}\". Tell the user the schedule ID is {} and they can remove it later with \"Remove schedule: {}\" or by saying REMOVE_SCHEDULE: {}.",
                                                id,
                                                cron_str,
                                                task_preview.trim(),
                                                id,
                                                id,
                                                id
                                            )
                                        }
                                        Ok(crate::scheduler::ScheduleAddOutcome::AlreadyExists) => {
                                            info!(
                                                "Agent router: SCHEDULE skipped (same task already scheduled)"
                                            );
                                            "This task is already scheduled with the same cron and description. Tell the user no duplicate was added."
                                        .to_string()
                                        }
                                        Err(e) => {
                                            info!("Agent router: SCHEDULE failed: {}", e);
                                            format!(
                                                "Failed to add schedule: {}. Tell the user and suggest they check ~/.mac-stats/schedules.json.",
                                                e
                                            )
                                        }
                                    }
                                }
                                Ok(ScheduleParseResult::At { at_str, task }) => {
                                    let id = format!("discord-{}", chrono::Utc::now().timestamp());
                                    let reply_to_channel_id =
                                        discord_reply_channel_id.map(|u| u.to_string());
                                    match crate::scheduler::add_schedule_at(
                                        id.clone(),
                                        at_str.clone(),
                                        task.clone(),
                                        reply_to_channel_id,
                                    ) {
                                        Ok(crate::scheduler::ScheduleAddOutcome::Added) => {
                                            info!(
                                                "Agent router: SCHEDULE at added (id={}, at={})",
                                                id, at_str
                                            );
                                            let task_preview: String =
                                                task.chars().take(100).collect();
                                            format!(
                                                "One-time schedule added. Schedule ID: **{}** (at {}): \"{}\". Tell the user the schedule ID is {} and they can remove it with \"Remove schedule: {}\" or REMOVE_SCHEDULE: {}.",
                                                id,
                                                at_str,
                                                task_preview.trim(),
                                                id,
                                                id,
                                                id
                                            )
                                        }
                                        Ok(crate::scheduler::ScheduleAddOutcome::AlreadyExists) => {
                                            info!("Agent router: SCHEDULE at skipped (duplicate)");
                                            "This one-time schedule was already added. Tell the user no duplicate was added.".to_string()
                                        }
                                        Err(e) => {
                                            info!("Agent router: SCHEDULE at failed: {}", e);
                                            format!(
                                                "Failed to add one-shot schedule: {}. Tell the user and suggest they check ~/.mac-stats/schedules.json.",
                                                e
                                            )
                                        }
                                    }
                                }
                                Err(e) => {
                                    info!("Agent router: SCHEDULE parse failed: {}", e);
                                    format!(
                                        "Could not parse schedule (expected e.g. \"every 5 minutes <task>\", \"at <datetime> <task>\", or \"<cron> <task>\"): {}. Ask the user to rephrase.",
                                        e
                                    )
                                }
                            }
                        }
                    }
                    "REMOVE_SCHEDULE" => {
                        let id = arg.trim();
                        if id.is_empty() {
                            "REMOVE_SCHEDULE requires a schedule ID (e.g. discord-1770648842). Ask the user which schedule to remove or to provide the ID.".to_string()
                        } else {
                            send_status(&format!("Removing schedule: {}…", id));
                            info!("Agent router: REMOVE_SCHEDULE requested: id={}", id);
                            match crate::scheduler::remove_schedule_by_id(id) {
                                Ok(true) => format!(
                                    "Schedule {} has been removed. Tell the user it is cancelled.",
                                    id
                                ),
                                Ok(false) => format!(
                                    "No schedule found with ID \"{}\". The ID may be wrong or already removed. Tell the user.",
                                    id
                                ),
                                Err(e) => {
                                    format!("Failed to remove schedule: {}. Tell the user.", e)
                                }
                            }
                        }
                    }
                    "LIST_SCHEDULES" => {
                        send_status("Listing schedules…");
                        info!("Agent router: LIST_SCHEDULES requested");
                        let list = crate::scheduler::list_schedules_formatted();
                        format!("{}\n\nUse this to answer the user.", list)
                    }
                    "RUN_CMD" => {
                        info!(
                            "Agent router: RUN_CMD requested: {}",
                            crate::logging::ellipse(&arg, 120)
                        );
                        if !crate::commands::run_cmd::is_local_cmd_allowed() {
                            "RUN_CMD is not available (disabled by ALLOW_LOCAL_CMD=0). Answer without running local commands.".to_string()
                        } else if last_run_cmd_arg.as_deref() == Some(arg.as_str()) {
                            info!(
                                "Agent router: RUN_CMD duplicate (same arg as last run), skipping execution"
                            );
                            "You already ran this command; the result is in the message above. Do not run RUN_CMD again. Reply with TASK_APPEND then TASK_STATUS as the task instructs.".to_string()
                        } else {
                            const MAX_CMD_RETRIES: u32 = 3;
                            let mut current_cmd = arg.to_string();
                            let mut last_output = String::new();

                            for attempt in 0..=MAX_CMD_RETRIES {
                                send_status(&format!(
                                    "Running local command{}: {}",
                                    if attempt > 0 {
                                        format!(" (retry {})", attempt)
                                    } else {
                                        String::new()
                                    },
                                    current_cmd
                                ));
                                info!("Agent router: RUN_CMD attempt {}: {}", attempt, current_cmd);
                                match tokio::task::spawn_blocking({
                                    let cmd = current_cmd.clone();
                                    move || crate::commands::run_cmd::run_local_command(&cmd)
                                })
                                .await
                                .map_err(|e| format!("RUN_CMD task: {}", e))
                                .and_then(|r| r)
                                {
                                    Ok(output) => {
                                        last_run_cmd_raw_output = Some(output.clone());
                                        info!(
                                            "Agent router: RUN_CMD completed, stored output for next TASK_APPEND ({} chars)",
                                            output.len()
                                        );
                                        last_output = format!(
                                            "Here is the command output:\n\n{}\n\nUse this to answer the user's question.",
                                            output
                                        );
                                        break;
                                    }
                                    Err(e) => {
                                        info!(
                                            "Agent router: RUN_CMD failed (attempt {}): {}",
                                            attempt, e
                                        );
                                        if multi_tool_turn {
                                            last_output = format!(
                                                "RUN_CMD failed in a multi-step plan: {}.\n\nRe-plan the full task from here. Keep the request in the correct tool domain. If Redmine data is still needed, use REDMINE_API directly with concrete parameters. Do not reply with only another RUN_CMD.",
                                                e
                                            );
                                            break;
                                        }
                                        if attempt >= MAX_CMD_RETRIES {
                                            last_output = format!(
                                                "RUN_CMD failed after {} retries: {}.\n\nAnswer the user's question only (e.g. explain that the export or command failed). Do not include Redmine time entries, summaries, or other tool output that is unrelated to this request.",
                                                MAX_CMD_RETRIES, e
                                            );
                                            break;
                                        }
                                        // Ask Ollama to fix the command (up to 2 attempts: full prompt, then format-only if parse fails)
                                        let allowed =
                                            crate::commands::run_cmd::allowed_commands().join(", ");
                                        let mut fix_prompt = format!(
                                            "The command `{}` failed with error:\n{}\n\nReply with ONLY the corrected command on a single line, in this exact format:\nRUN_CMD: <corrected command>\n\nAllowed commands: {}. Paths must be under ~/.mac-stats.",
                                            current_cmd, e, allowed
                                        );
                                        let mut fix_parse_retried = false;
                                        loop {
                                            let fix_messages = vec![crate::ollama::ChatMessage {
                                                role: "user".to_string(),
                                                content: fix_prompt.clone(),
                                                images: None,
                                            }];
                                            match send_ollama_chat_messages(
                                                fix_messages,
                                                model_override.clone(),
                                                options_override.clone(),
                                            )
                                            .await
                                            {
                                                Ok(resp) => {
                                                    let fixed = resp.message.content.trim().to_string();
                                                    info!(
                                                        "Agent router: RUN_CMD fix suggestion: {}",
                                                        crate::logging::ellipse(&fixed, 120)
                                                    );
                                                    match parse_tool_from_response(&fixed) {
                                                        Some((tool, new_arg)) if tool == "RUN_CMD" => {
                                                            current_cmd = new_arg;
                                                            break;
                                                        }
                                                        _ => {
                                                            if !fix_parse_retried {
                                                                fix_parse_retried = true;
                                                                fix_prompt = format!(
                                                                    "Your previous reply was not in the required format. Reply with exactly one line: RUN_CMD: <command>. No other text, no explanation.\n\nOriginal error: {}\nFailed command: {}",
                                                                    e, current_cmd
                                                                );
                                                                continue;
                                                            }
                                                            info!(
                                                                "Agent router: RUN_CMD fix suggestion not parseable as RUN_CMD (after format retry)"
                                                            );
                                                            last_output = format!(
                                                                "RUN_CMD failed: {}. The model's corrected command was not in the required format (exactly one line: RUN_CMD: <command>). Answer the user's question only; do not include Redmine or other unrelated tool output.",
                                                                e
                                                            );
                                                            break;
                                                        }
                                                    }
                                                }
                                                Err(ollama_err) => {
                                                    info!(
                                                        "Agent router: RUN_CMD fix Ollama call failed: {}",
                                                        ollama_err
                                                    );
                                                    last_output = format!(
                                                        "RUN_CMD failed: {}. Could not get a corrected command from the model. Answer the user's question only; do not include Redmine or other unrelated tool output.",
                                                        e
                                                    );
                                                    break;
                                                }
                                            }
                                        }
                                        if !last_output.is_empty() {
                                            break;
                                        }
                                    }
                                }
                            }
                            last_output
                        }
                    }
                    "PYTHON_SCRIPT" => {
                        if !crate::commands::python_agent::is_python_script_allowed() {
                            "PYTHON_SCRIPT is not available (disabled by ALLOW_PYTHON_SCRIPT=0). Answer without running Python.".to_string()
                        } else {
                            match parse_python_script_from_response(&response_content) {
                        Some((id, topic, script_body)) => {
                            let script_label = format!("{} ({})", id, topic);
                            send_status(&format!("Running Python script '{}'…", script_label));
                            info!("Agent router: PYTHON_SCRIPT running script '{}' (id={}, topic={}, body {} chars)", script_label, id, topic, script_body.len());
                            match tokio::task::spawn_blocking({
                                let id = id.clone();
                                let topic = topic.clone();
                                let script_body = script_body.clone();
                                move || crate::commands::python_agent::run_python_script(&id, &topic, &script_body)
                            })
                            .await
                            .map_err(|e| format!("PYTHON_SCRIPT task: {}", e))
                            .and_then(|r| r)
                            {
                                Ok(stdout) => format!(
                                    "Python script result:\n\n{}\n\nUse this to answer the user's question.",
                                    stdout
                                ),
                                Err(e) => format!(
                                    "PYTHON_SCRIPT failed: {}. Answer without this result.",
                                    e
                                ),
                            }
                        }
                        None => "PYTHON_SCRIPT requires: PYTHON_SCRIPT: <id> <topic> and then the Python code on the next lines or in a ```python block.".to_string(),
                    }
                        }
                    }
                    "DISCORD_API" => {
                        let arg = arg.trim();
                        let (method, rest) = match arg.find(' ') {
                            Some(i) => (arg[..i].trim().to_string(), arg[i..].trim()),
                            None => ("GET".to_string(), arg),
                        };
                        let (path_raw, body) = if let Some(idx) = rest.find(" {") {
                            let (p, b) = rest.split_at(idx);
                            (p.trim().to_string(), Some(b.trim().to_string()))
                        } else {
                            (rest.to_string(), None)
                        };
                        let path = normalize_discord_api_path(&path_raw);
                        if path.is_empty() {
                            "DISCORD_API requires: DISCORD_API: <METHOD> <path> or DISCORD_API: POST <path> {\"content\":\"...\"}.".to_string()
                        } else if last_successful_discord_call
                            .as_ref()
                            .map(|(m, p)| m == &method && p == &path)
                            .unwrap_or(false)
                        {
                            "You already received the data for this endpoint above. Format it for the user and reply; do not call DISCORD_API again for the same path.".to_string()
                        } else {
                            let status_msg = format!("Calling Discord API: {} {}", method, path);
                            send_status(&status_msg);
                            info!("Discord API: {} {}", method, path);
                            match crate::discord::api::discord_api_request(
                                &method,
                                &path,
                                body.as_deref(),
                            )
                            .await
                            {
                                Ok(result) => {
                                    last_successful_discord_call =
                                        Some((method.clone(), path.clone()));
                                    format!(
                                        "Discord API result:\n\n{}\n\nUse this to answer the user's question.",
                                        result
                                    )
                                }
                                Err(e) => {
                                    let msg = crate::discord::api::sanitize_discord_api_error(&e);
                                    format!(
                                        "Discord API failed: {}. Answer without this result.",
                                        msg
                                    )
                                }
                            }
                        }
                    }
                    "OLLAMA_API" => {
                        let arg = arg.trim();
                        let (action, rest) = match arg.find(' ') {
                            Some(i) => (arg[..i].trim().to_lowercase(), arg[i..].trim()),
                            None => (arg.to_lowercase(), ""),
                        };
                        let status_detail = if rest.is_empty() {
                            format!("Ollama API: {}…", action)
                        } else {
                            let preview: String = rest.chars().take(40).collect();
                            format!("Ollama API: {} {}…", action, preview)
                        };
                        send_status(&status_detail);
                        info!(
                            "Agent router: OLLAMA_API requested: action={}, rest={} chars",
                            action,
                            rest.chars().count()
                        );
                        let result = match action.as_str() {
                            "list_models" => list_ollama_models_full().await.map(|r| {
                                serde_json::to_string_pretty(&r)
                                    .unwrap_or_else(|_| "[]".to_string())
                            }),
                            "version" => get_ollama_version().await.map(|r| r.version),
                            "running" => list_ollama_running_models().await.map(|r| {
                                serde_json::to_string_pretty(&r)
                                    .unwrap_or_else(|_| "[]".to_string())
                            }),
                            "pull" => {
                                let parts: Vec<&str> = rest.split_whitespace().collect();
                                let model =
                                    parts.first().map(|s| (*s).to_string()).unwrap_or_default();
                                let stream = parts.get(1).map(|s| *s == "true").unwrap_or(true);
                                if model.is_empty() {
                                    Err("OLLAMA_API pull requires a model name.".to_string())
                                } else {
                                    pull_ollama_model(model, stream)
                                        .await
                                        .map(|_| "Pull completed.".to_string())
                                }
                            }
                            "delete" => {
                                let model = rest.to_string();
                                if model.is_empty() {
                                    Err("OLLAMA_API delete requires a model name.".to_string())
                                } else {
                                    delete_ollama_model(model)
                                        .await
                                        .map(|_| "Model deleted.".to_string())
                                }
                            }
                            "embed" => {
                                let parts: Vec<&str> = rest.splitn(2, ' ').map(str::trim).collect();
                                if parts.len() < 2 || parts[1].is_empty() {
                                    Err("OLLAMA_API embed requires: embed <model> <text>."
                                        .to_string())
                                } else {
                                    let model = parts[0].to_string();
                                    let input = serde_json::Value::String(parts[1].to_string());
                                    ollama_embeddings(model, input, None).await.map(|r| {
                                        serde_json::to_string_pretty(&r)
                                            .unwrap_or_else(|_| "{}".to_string())
                                    })
                                }
                            }
                            "load" => {
                                let parts: Vec<&str> =
                                    rest.splitn(2, char::is_whitespace).map(str::trim).collect();
                                let model =
                                    parts.first().map(|s| (*s).to_string()).unwrap_or_default();
                                let keep_alive = parts
                                    .get(1)
                                    .filter(|s| !s.is_empty())
                                    .map(|s| (*s).to_string());
                                if model.is_empty() {
                                    Err("OLLAMA_API load requires a model name.".to_string())
                                } else {
                                    load_ollama_model(model, keep_alive)
                                        .await
                                        .map(|_| "Model loaded.".to_string())
                                }
                            }
                            "unload" => {
                                let model = rest.to_string();
                                if model.is_empty() {
                                    Err("OLLAMA_API unload requires a model name.".to_string())
                                } else {
                                    unload_ollama_model(model)
                                        .await
                                        .map(|_| "Model unloaded.".to_string())
                                }
                            }
                            _ => Err(format!(
                                "Unknown OLLAMA_API action: {}. Use list_models, version, running, pull, delete, embed, load, or unload.",
                                action
                            )),
                        };
                        match result {
                            Ok(msg) => format!(
                                "Ollama API result:\n\n{}\n\nUse this to answer the user's question.",
                                msg
                            ),
                            Err(e) => {
                                format!("OLLAMA_API failed: {}. Answer without this result.", e)
                            }
                        }
                    }
                    "TASK_APPEND" => {
                        let (path_or_id, content) = match arg.find(' ') {
                            Some(i) => (arg[..i].trim(), arg[i..].trim()),
                            None => ("", ""),
                        };
                        if path_or_id.is_empty() || content.is_empty() {
                            "TASK_APPEND requires: TASK_APPEND: <path or task id> <content>."
                                .to_string()
                        } else {
                            match crate::task::resolve_task_path(path_or_id) {
                                Ok(path) => {
                                    current_task_path = Some(path.clone());
                                    let task_label = crate::task::task_file_name(&path);
                                    send_status(&format!("Appending to task '{}'…", task_label));
                                    // If we just ran RUN_CMD, append the full command output to the task (model often sends a summary only).
                                    let content_to_append = if let Some(raw) =
                                        last_run_cmd_raw_output.take()
                                    {
                                        info!(
                                            "Agent router: TASK_APPEND using full RUN_CMD output ({} chars) for task '{}'",
                                            raw.chars().count(),
                                            task_label
                                        );
                                        raw
                                    } else {
                                        content.to_string()
                                    };
                                    info!(
                                        "Agent router: TASK_APPEND for task '{}' ({} chars)",
                                        task_label,
                                        content_to_append.chars().count()
                                    );
                                    match crate::task::append_to_task(&path, &content_to_append) {
                                        Ok(()) => format!(
                                            "Appended to task file '{}'. Use this to continue.",
                                            task_label
                                        ),
                                        Err(e) => format!("TASK_APPEND failed: {}.", e),
                                    }
                                }
                                Err(e) => format!("TASK_APPEND failed: {}.", e),
                            }
                        }
                    }
                    "TASK_STATUS" => {
                        let parts: Vec<&str> = arg.split_whitespace().collect();
                        if parts.len() < 2 {
                            "TASK_STATUS requires: TASK_STATUS: <path or task id> wip|finished."
                                .to_string()
                        } else {
                            // Find first valid status word (allow trailing punctuation: "finished." -> "finished")
                            let mut path_or_id = parts[0].to_string();
                            let mut status: Option<String> = None;
                            for (i, part) in parts.iter().skip(1).enumerate() {
                                let s = part.trim_end_matches(['.', ',', ';']).to_lowercase();
                                if ["wip", "finished", "unsuccessful", "paused"]
                                    .contains(&s.as_str())
                                {
                                    status = Some(s);
                                    if i > 0 {
                                        path_or_id = parts[..=i].join(" ");
                                    }
                                    break;
                                }
                            }
                            match status {
                        None => {
                            "TASK_STATUS status must be wip, finished, unsuccessful, or paused.".to_string()
                        }
                        Some(status) => match crate::task::resolve_task_path(&path_or_id) {
                            Ok(path) => {
                                if status == "finished"
                                    && !crate::task::all_sub_tasks_closed(&path).unwrap_or(true)
                                {
                                    "Cannot set status to finished: not all sub-tasks (## Sub-tasks: ...) are finished or unsuccessful.".to_string()
                                } else {
                                    match crate::task::set_task_status(&path, &status) {
                                        Ok(new_path) => {
                                            current_task_path = Some(new_path.clone());
                                            format!(
                                                "Task status set to {} (file: {}).",
                                                status,
                                                crate::task::task_file_name(&new_path)
                                            )
                                        }
                                        Err(e) => format!("TASK_STATUS failed: {}.", e),
                                    }
                                }
                            }
                            Err(e) => format!("TASK_STATUS failed: {}.", e),
                        },
                    }
                        }
                    }
                    "TASK_CREATE" => {
                        let segs: Vec<&str> = arg.splitn(3, ' ').map(str::trim).collect();
                        if segs.len() >= 3 && !segs[2].is_empty() {
                            let topic = segs[0];
                            let id = segs[1];
                            // Truncate at " then " / " then TASK" so we don't store the next tool call in the task body
                            let initial_content = segs[2];
                            let content =
                                if let Some(pos) = initial_content.to_uppercase().find(" THEN ") {
                                    initial_content[..pos].trim()
                                } else {
                                    initial_content
                                };
                            let reply_to = discord_reply_channel_id;
                            match crate::task::create_task(topic, id, content, None, reply_to) {
                                Ok(path) => {
                                    current_task_path = Some(path.clone());
                                    let name = crate::task::task_file_name(&path);
                                    format!(
                                        "Task created: {}. Use TASK_APPEND: {} or TASK_APPEND: <id> <content> and TASK_STATUS to update.",
                                        name, name
                                    )
                                }
                                Err(e) => format!("TASK_CREATE failed: {}.", e),
                            }
                        } else {
                            "TASK_CREATE requires: TASK_CREATE: <topic> <id> <initial content>."
                                .to_string()
                        }
                    }
                    "TASK_SHOW" => {
                        if arg.trim().is_empty() {
                            "TASK_SHOW requires: TASK_SHOW: <path or task id>.".to_string()
                        } else {
                            send_status("Showing task…");
                            info!("Agent router: TASK_SHOW requested: {}", arg.trim());
                            match crate::task::resolve_task_path(arg.trim()) {
                                Ok(path) => {
                                    current_task_path = Some(path.clone());
                                    match crate::task::show_task_content(&path) {
                                        Ok((status, assignee, content)) => {
                                            const MAX_CHANNEL_MSG: usize = 1900;
                                            let body = format!(
                                                "**Status:** {} | **Assigned:** {}\n\n{}",
                                                status, assignee, content
                                            );
                                            let msg = if body.chars().count() <= MAX_CHANNEL_MSG {
                                                body
                                            } else {
                                                crate::logging::ellipse(&body, MAX_CHANNEL_MSG)
                                            };
                                            send_status(&msg);
                                            "Task content was sent to the user in the channel. They can ask you to TASK_APPEND or TASK_STATUS for this task.".to_string()
                                        }
                                        Err(e) => format!("TASK_SHOW failed: {}.", e),
                                    }
                                }
                                Err(e) => format!("TASK_SHOW failed: {}.", e),
                            }
                        }
                    }
                    "TASK_ASSIGN" => {
                        let parts: Vec<&str> = arg.split_whitespace().collect();
                        if parts.len() < 2 {
                            "TASK_ASSIGN requires: TASK_ASSIGN: <path or task id> <agent_id> (e.g. scheduler, discord, cpu, default).".to_string()
                        } else {
                            let path_or_id = parts[..parts.len() - 1].join(" ");
                            let agent_id_raw = parts[parts.len() - 1];
                            // Normalize so "CURSOR_AGENT" / "cursor-agent" => scheduler (review loop only picks scheduler/default)
                            let agent_id = match agent_id_raw.to_uppercase().as_str() {
                                "CURSOR_AGENT" | "CURSOR-AGENT" => "scheduler",
                                _ => agent_id_raw,
                            };
                            send_status(&format!("Assigning task to {}…", agent_id));
                            info!(
                                "Agent router: TASK_ASSIGN {} -> {} (raw: {})",
                                path_or_id, agent_id, agent_id_raw
                            );
                            match crate::task::resolve_task_path(&path_or_id) {
                                Ok(path) => {
                                    current_task_path = Some(path.clone());
                                    match crate::task::set_assignee(&path, agent_id) {
                                        Ok(()) => {
                                            let _ = crate::task::append_to_task(
                                                &path,
                                                &format!("Reassigned to {}.", agent_id),
                                            );
                                            format!("Task assigned to {}.", agent_id)
                                        }
                                        Err(e) => format!("TASK_ASSIGN failed: {}.", e),
                                    }
                                }
                                Err(e) => format!("TASK_ASSIGN failed: {}.", e),
                            }
                        }
                    }
                    "TASK_SLEEP" => {
                        let parts: Vec<&str> = arg.split_whitespace().collect();
                        let (path_or_id, until_str) = if parts.len() >= 3
                            && parts[parts.len() - 2].eq_ignore_ascii_case("until")
                        {
                            (parts[..parts.len() - 2].join(" "), parts[parts.len() - 1])
                        } else if parts.len() >= 2 {
                            (parts[..parts.len() - 1].join(" "), parts[parts.len() - 1])
                        } else {
                            ("".to_string(), "")
                        };
                        if path_or_id.is_empty() || until_str.is_empty() {
                            "TASK_SLEEP requires: TASK_SLEEP: <path or task id> until <ISO datetime> (e.g. 2025-02-10T09:00:00).".to_string()
                        } else {
                            send_status("Pausing task…");
                            info!(
                                "Agent router: TASK_SLEEP {} until {}",
                                path_or_id, until_str
                            );
                            match crate::task::resolve_task_path(&path_or_id) {
                                Ok(path) => {
                                    current_task_path = Some(path.clone());
                                    if let Ok(new_path) =
                                        crate::task::set_task_status(&path, "paused")
                                    {
                                        current_task_path = Some(new_path.clone());
                                        let _ = crate::task::set_paused_until(
                                            &new_path,
                                            Some(until_str),
                                        );
                                        let _ = crate::task::append_to_task(
                                            &new_path,
                                            &format!("Paused until {}.", until_str),
                                        );
                                    }
                                    format!(
                                        "Task paused until {}. It will resume automatically after that time.",
                                        until_str
                                    )
                                }
                                Err(e) => format!("TASK_SLEEP failed: {}.", e),
                            }
                        }
                    }
                    "TASK_LIST" => {
                        let show_all = arg.trim().to_lowercase() == "all"
                            || arg.trim().to_lowercase() == "all tasks"
                            || arg.trim().to_lowercase().starts_with("all ");
                        let result = if show_all {
                            send_status("Listing all tasks (by status)…");
                            info!("Agent router: TASK_LIST all requested");
                            match crate::task::format_list_all_tasks() {
                                Ok(list) => {
                                    const MAX_CHANNEL_MSG: usize = 1900;
                                    const LIST_MAX: usize = MAX_CHANNEL_MSG - 20;
                                    let msg = if list.chars().count() <= LIST_MAX {
                                        format!("**All tasks**\n\n{}", list)
                                    } else {
                                        format!(
                                            "**All tasks**\n\n{}",
                                            crate::logging::ellipse(&list, LIST_MAX)
                                        )
                                    };
                                    send_status(&msg);
                                    "The full task list (Open, WIP, Finished, Unsuccessful) was sent to the user in the channel. Acknowledge that you showed all tasks. Task ids are the filenames; the user can use TASK_APPEND or TASK_STATUS with those ids.".to_string()
                                }
                                Err(e) => format!("TASK_LIST failed: {}.", e),
                            }
                        } else {
                            send_status("Listing open and WIP tasks…");
                            info!("Agent router: TASK_LIST requested");
                            match crate::task::format_list_open_and_wip_tasks() {
                                Ok(list) => {
                                    const MAX_CHANNEL_MSG: usize = 1900;
                                    const LIST_MAX: usize = MAX_CHANNEL_MSG - 20;
                                    let msg = if list.chars().count() <= LIST_MAX {
                                        format!("**Active task list**\n\n{}", list)
                                    } else {
                                        format!(
                                            "**Active task list**\n\n{}",
                                            crate::logging::ellipse(&list, LIST_MAX)
                                        )
                                    };
                                    send_status(&msg);
                                    "The task list was sent to the user in the channel. Acknowledge that you showed the list. Task ids are the filenames; the user can use TASK_APPEND or TASK_STATUS with those ids.".to_string()
                                }
                                Err(e) => format!("TASK_LIST failed: {}.", e),
                            }
                        };
                        result
                    }
                    "MCP" => {
                        send_status("Calling MCP tool…");
                        info!(
                            "Agent router: MCP requested (arg len={})",
                            arg.chars().count()
                        );
                        match crate::mcp::get_mcp_server_url() {
                            Some(server_url) => {
                                let (mcp_tool_name, mcp_args) = if let Some(space) = arg.find(' ') {
                                    let (name, rest) = arg.split_at(space);
                                    let rest = rest.trim();
                                    let args = if rest.starts_with('{') {
                                        serde_json::from_str(rest).ok()
                                    } else {
                                        Some(serde_json::json!({ "input": rest }))
                                    };
                                    (name.to_string(), args)
                                } else {
                                    (arg.clone(), None)
                                };
                                match crate::mcp::call_tool(&server_url, &mcp_tool_name, mcp_args)
                                    .await
                                {
                                    Ok(result) => {
                                        info!(
                                            "Agent router: MCP tool {} completed ({} chars)",
                                            mcp_tool_name,
                                            result.len()
                                        );
                                        format!(
                                            "MCP tool \"{}\" result:\n\n{}\n\nUse this to answer the user's question.",
                                            mcp_tool_name, result
                                        )
                                    }
                                    Err(e) => {
                                        info!(
                                            "Agent router: MCP tool {} failed: {}",
                                            mcp_tool_name, e
                                        );
                                        format!(
                                            "MCP tool \"{}\" failed: {}. Answer the user without this result.",
                                            mcp_tool_name, e
                                        )
                                    }
                                }
                            }
                            None => {
                                info!("Agent router: MCP not configured (no MCP_SERVER_URL)");
                                "MCP is not configured (set MCP_SERVER_URL in env or .config.env). Answer without using MCP.".to_string()
                            }
                        }
                    }
                    "CURSOR_AGENT" => {
                        if !crate::commands::cursor_agent::is_cursor_agent_available() {
                            "CURSOR_AGENT is not available (cursor-agent CLI not found on PATH). Answer without it.".to_string()
                        } else {
                            let prompt = arg.trim().to_string();
                            if prompt.is_empty() {
                                "CURSOR_AGENT requires a prompt: CURSOR_AGENT: <detailed coding task>".to_string()
                            } else {
                                let preview: String = prompt.chars().take(80).collect();
                                send_status(&format!("Running Cursor Agent: {}…", preview));
                                info!(
                                    "Agent router: CURSOR_AGENT running prompt ({} chars)",
                                    prompt.len()
                                );
                                match tokio::task::spawn_blocking({
                                    let p = prompt.clone();
                                    move || crate::commands::cursor_agent::run_cursor_agent(&p)
                                })
                                .await
                                .map_err(|e| format!("CURSOR_AGENT task: {}", e))
                                .and_then(|r| r)
                                {
                                    Ok(output) => {
                                        info!(
                                            "Agent router: CURSOR_AGENT completed ({} chars output)",
                                            output.len()
                                        );
                                        let truncated = if output.chars().count() > 4000 {
                                            let half = 1800;
                                            let start: String = output.chars().take(half).collect();
                                            let end: String = output
                                                .chars()
                                                .rev()
                                                .take(half)
                                                .collect::<String>()
                                                .chars()
                                                .rev()
                                                .collect();
                                            format!("{}...\n[truncated]\n...{}", start, end)
                                        } else {
                                            output
                                        };
                                        format!(
                                            "Cursor Agent result:\n\n{}\n\nUse this to answer the user's question.",
                                            truncated
                                        )
                                    }
                                    Err(e) => {
                                        info!("Agent router: CURSOR_AGENT failed: {}", e);
                                        format!(
                                            "CURSOR_AGENT failed: {}. Answer the user without this result.",
                                            e
                                        )
                                    }
                                }
                            }
                        }
                    }
                    "REDMINE_API" => {
                        let arg = arg.trim();
                        let (method, rest) = match arg.find(' ') {
                            Some(i) => (arg[..i].trim().to_string(), arg[i..].trim()),
                            None => ("GET".to_string(), arg),
                        };
                        let (path, body) = if let Some(idx) = rest.find(" {") {
                            let (p, b) = rest.split_at(idx);
                            (p.trim().to_string(), Some(b.trim().to_string()))
                        } else {
                            (rest.to_string(), None)
                        };
                        if path.is_empty() {
                            "REDMINE_API requires: REDMINE_API: GET /issues/1234.json?include=journals,attachments".to_string()
                        } else {
                            send_status(&format!("Querying Redmine: {} {}", method, path));
                            info!(
                                "Agent router [{}]: REDMINE_API {} {}",
                                request_id, method, path
                            );
                            match crate::redmine::redmine_api_request(
                                &method,
                                &path,
                                body.as_deref(),
                            )
                            .await
                            {
                                Ok(result) => {
                                    let mut msg = if path.contains("time_entries") {
                                        format!(
                                            "Redmine API result:\n\n{}\n\nUse this data to answer the user's question. The derived summary above already lists the actual tickets worked (if any), totals, users, projects, and entry details. Use that instead of inventing ticket ids or subjects.",
                                            result
                                        )
                                    } else {
                                        format!(
                                            "Redmine API result:\n\n{}\n\nUse this data to answer the user's question. Summarize the issue clearly: subject, description quality, what's missing, status, assignee, and key comments.",
                                            result
                                        )
                                    };
                                    if method.to_uppercase() == "GET" {
                                        if is_redmine_review_or_summarize_only(question) {
                                            msg.push_str(
                                        "\n\nThe user asked only to review/summarize. Do NOT update the ticket or add a comment. Reply with your summary and DONE: success.",
                                    );
                                        } else if path.contains("time_entries") {
                                                msg.push_str(
                                            "\n\nUse this data to answer. If the user asked for tickets worked, list the actual issue ids and subjects from the derived summary. If the user asked for \"this month\", use from/to for the current month. If success criteria require JSON format, reply with valid JSON only (e.g. total hours, ticket list, project breakdown).",
                                        );
                                            } else {
                                                let id = path
                                                    .trim_start_matches('/')
                                                    .strip_prefix("issues/")
                                                    .map(|s| {
                                                        s.split(['.', '?'])
                                                            .next()
                                                            .unwrap_or("")
                                                            .to_string()
                                                    })
                                                    .unwrap_or_default();
                                                if !id.is_empty()
                                                    && id.chars().all(|c| c.is_ascii_digit())
                                                {
                                                    msg.push_str(&format!(
                                                "\n\nIf the user asked to **update** this ticket or **add a comment**, your next reply MUST be exactly one line: REDMINE_API: PUT /issues/{}.json {{\"issue\":{{\"notes\":\"<your comment text>\"}}}}. Do not reply with only a summary.",
                                                id
                                            ));
                                                }
                                            }
                                    }
                                    msg
                                }
                                Err(e) => format!(
                                    "Redmine API failed: {}. Answer without this result.",
                                    e
                                ),
                            }
                        }
                    }
                    "MASTODON_POST" => {
                        let arg = arg.trim();
                        if arg.is_empty() {
                            "MASTODON_POST requires text. Usage: MASTODON_POST: <text to post>. Optional visibility prefix: MASTODON_POST: unlisted: <text> (default: public).".to_string()
                        } else {
                            let (visibility, text) = if let Some(rest) = arg
                                .strip_prefix("unlisted:")
                                .or_else(|| arg.strip_prefix("unlisted "))
                            {
                                ("unlisted", rest.trim())
                            } else if let Some(rest) = arg
                                .strip_prefix("private:")
                                .or_else(|| arg.strip_prefix("private "))
                            {
                                ("private", rest.trim())
                            } else if let Some(rest) = arg
                                .strip_prefix("direct:")
                                .or_else(|| arg.strip_prefix("direct "))
                            {
                                ("direct", rest.trim())
                            } else if let Some(rest) = arg
                                .strip_prefix("public:")
                                .or_else(|| arg.strip_prefix("public "))
                            {
                                ("public", rest.trim())
                            } else {
                                ("public", arg)
                            };
                            send_status(&format!("Posting to Mastodon ({})…", visibility));
                            info!(
                                "Agent router: MASTODON_POST visibility={} text={}",
                                visibility,
                                crate::logging::ellipse(text, 100)
                            );
                            match mastodon_post(text, visibility).await {
                                Ok(msg) => msg,
                                Err(e) => format!("Mastodon post failed: {}", e),
                            }
                        }
                    }
                    "MEMORY_APPEND" => {
                        let arg = arg.trim();
                        if arg.is_empty() {
                            "MEMORY_APPEND requires content. Usage: MEMORY_APPEND: <lesson> or MEMORY_APPEND: agent:<slug-or-id> <lesson>".to_string()
                        } else {
                            let (target, lesson) = if arg.to_lowercase().starts_with("agent:") {
                                let rest = arg["agent:".len()..].trim();
                                if let Some(space_idx) = rest.find(' ') {
                                    let (sel, content) = rest.split_at(space_idx);
                                    (Some(sel.trim().to_string()), content.trim().to_string())
                                } else {
                                    (None, arg.to_string())
                                }
                            } else {
                                (None, arg.to_string())
                            };
                            let lesson_line = format!("- {}\n", lesson.trim_start_matches("- "));
                            let result = if let Some(selector) = target {
                                let agents = crate::agents::load_agents();
                                if let Some(agent) =
                                    crate::agents::find_agent_by_id_or_name(&agents, &selector)
                                {
                                    if let Some(dir) = crate::agents::get_agent_dir(&agent.id) {
                                        let path = dir.join("memory.md");
                                        append_to_file(&path, &lesson_line)
                                    } else {
                                        Err(format!("Agent directory not found for '{}'", selector))
                                    }
                                } else {
                                    Err(format!("Agent '{}' not found", selector))
                                }
                            } else {
                                let path = discord_reply_channel_id
                                    .map(
                                        crate::config::Config::memory_file_path_for_discord_channel,
                                    )
                                    .unwrap_or_else(crate::config::Config::memory_file_path);
                                append_to_file(&path, &lesson_line)
                            };
                            match result {
                                Ok(path) => {
                                    info!("Agent router: MEMORY_APPEND wrote to {:?}", path);
                                    format!(
                                        "Memory updated ({}). The lesson will be included in future prompts.",
                                        path.display()
                                    )
                                }
                                Err(e) => {
                                    info!("Agent router: MEMORY_APPEND failed: {}", e);
                                    format!("Failed to update memory: {}", e)
                                }
                            }
                        }
                    }
                    _ => {
                        let msg = format!(
                            "Unknown tool \"{}\". Use one of the available tools from the agent list (e.g. FETCH_URL, BRAVE_SEARCH, BROWSER_NAVIGATE, DONE) or reply with your answer.",
                            tool
                        );
                        tracing::warn!(
                            "Agent router: unknown tool \"{}\" (arg: {} chars), sending hint to model",
                            tool,
                            arg.chars().count()
                        );
                        msg
                    }
                };

                let result_len = user_message.chars().count();
                info!(
                    "Agent router: tool {} completed, sending result back to Ollama ({} chars): {}",
                    tool,
                    result_len,
                    log_content(&user_message)
                );

                if tool == "RUN_CMD" && user_message.starts_with("Here is the command output") {
                    last_run_cmd_arg = Some(arg.clone());
                } else if tool != "RUN_CMD" {
                    last_run_cmd_arg = None;
                }
                // Only clear stored RUN_CMD output when we run something other than RUN_CMD or TASK_APPEND
                // (so it stays set for the next TASK_APPEND after RUN_CMD).
                if tool != "TASK_APPEND" && tool != "RUN_CMD" {
                    last_run_cmd_raw_output = None;
                }
                if tool == "BROWSER_EXTRACT"
                    && !user_message.is_empty()
                    && !user_message.contains("BROWSER_EXTRACT failed")
                {
                    last_browser_extract = Some(user_message.clone());
                }

                let is_browser_error = if tool == "BROWSER_SCREENSHOT" {
                    user_message.starts_with("Screenshot of current page failed")
                        || user_message.starts_with("Screenshot task error")
                } else if tool.starts_with("BROWSER_") {
                    user_message.starts_with(&format!("{} failed", tool))
                        || user_message.starts_with(&format!("{} task error", tool))
                        || user_message.starts_with(&format!("{} HTTP fallback task error", tool))
                        || user_message.starts_with(&format!("{} CDP retry task error", tool))
                } else {
                    false
                };

                let is_multi_tool_run_cmd_error = tool == "RUN_CMD"
                    && user_message.starts_with("RUN_CMD failed in a multi-step plan");
                tool_results.push(user_message);
                if let Some(pair) = executed_browser_tool_arg {
                    last_browser_tool_arg = Some(pair);
                }
                if is_browser_error || is_multi_tool_run_cmd_error {
                    info!(
                        "Agent router: {} returned an error, aborting remaining tools in this turn",
                        tool
                    );
                    break;
                }
            }

            let user_message = tool_results.join("\n\n---\n\n");
            if let Some(blocked_reply) = grounded_redmine_time_entries_failure_reply(
                &request_for_verification,
                &user_message,
            ) {
                info!("Agent router: returning grounded Redmine blocked-state reply");
                response_content = blocked_reply;
                break;
            }
            if !question_explicitly_requests_json(question) {
                if let Some(summary) = extract_redmine_time_entries_summary_for_reply(&user_message)
                {
                    info!("Agent router: returning direct Redmine time-entry summary");
                    response_content = summary;
                    if done_claimed.is_some() {
                        exited_via_done = true;
                    }
                    break;
                }
            }
            if done_claimed.is_some() {
                if !user_message.trim().is_empty() {
                    response_content = final_reply_from_tool_results(question, &user_message);
                }
                exited_via_done = true;
                break;
            }

            messages.push(crate::ollama::ChatMessage {
                role: "assistant".to_string(),
                content: response_content.clone(),
                images: None,
            });
            let tool_result_role = if user_message.starts_with("Here is the command output") {
                "system"
            } else {
                "user"
            };
            messages.push(crate::ollama::ChatMessage {
                role: tool_result_role.to_string(),
                content: user_message,
                images: None,
            });

            let follow_up = send_ollama_chat_messages(
                messages.clone(),
                model_override.clone(),
                options_override.clone(),
            )
            .await?;
            response_content = follow_up.message.content.clone();

            // Fallback: if Ollama returned empty after a successful tool result, use the raw
            // tool output directly so the user at least sees what the tool produced.
            if response_content.trim().is_empty() {
                if let Some(last_msg) = messages.last() {
                    let raw = &last_msg.content;
                    if raw.starts_with("Here is the command output")
                        || raw.starts_with("Here is the page content")
                        || raw.starts_with("MCP tool")
                        || raw.starts_with("Search results")
                        || raw.starts_with("Discord API")
                    {
                        info!(
                            "Agent router: Ollama returned empty after tool success — using raw tool output as response"
                        );
                        // Strip the instruction suffix we appended for the model
                        let cleaned = raw
                            .replace("\n\nUse this to answer the user's question.", "")
                            .replace("Here is the command output:\n\n", "")
                            .replace("Here is the page content:\n\n", "");
                        response_content = cleaned;
                    }
                }
            }

            if tool_count >= max_tool_iterations {
                info!(
                    "Agent router: max tool iterations reached ({}), using last response as final",
                    max_tool_iterations
                );
            }
        }

        if exited_via_done {
            let lines: Vec<&str> = response_content.lines().collect();
            if let Some(last) = lines.last() {
                let t = last.trim();
                if t.to_uppercase().starts_with("DONE") && t.contains(':') {
                    let keep = lines.len().saturating_sub(1);
                    response_content = lines[..keep].join("\n").trim_end().to_string();
                }
            }
        }

        let final_len = response_content.chars().count();
        info!(
            "Agent router: done after {} tool(s), returning final response ({} chars): {}",
            tool_count,
            final_len,
            log_content(&response_content)
        );

        // When multiple agents participated, ensure the user sees the conversation: append a transcript if we have 2+ agent turns and the final reply is short (so we don't hide a long model summary).
        if agent_conversation.len() >= 2 {
            const SHORT_REPLY_THRESHOLD: usize = 500;
            if response_content.chars().count() < SHORT_REPLY_THRESHOLD
                || response_content.contains("Thank you for providing")
                || response_content.contains("If you have any specific tasks")
            {
                let mut transcript = String::from("\n\n---\n**Conversation:**\n\n");
                for (label, reply) in &agent_conversation {
                    transcript.push_str("**");
                    transcript.push_str(label);
                    transcript.push_str(":**\n");
                    transcript.push_str(reply);
                    transcript.push_str("\n\n");
                }
                response_content.push_str(transcript.trim_end());
            }
        }

        // Log the full conversation (user question + assistant reply) into the task file when we touched a task this run.
        // Skip when the "user" message is the task runner's prompt (synthetic), so we don't log runner turns as User/Assistant.
        let is_runner_prompt = question
            .trim_start()
            .starts_with("Current task file content:");
        if let Some(ref path) = current_task_path {
            if !is_runner_prompt {
                if let Err(e) =
                    crate::task::append_conversation_block(path, question, &response_content)
                {
                    info!(
                        "Agent router: could not append conversation to task file: {}",
                        e
                    );
                } else {
                    info!(
                        "Agent router: appended conversation to task {}",
                        crate::task::task_file_name(path)
                    );
                }
            } else {
                info!(
                    "Agent router: skipped appending conversation (task runner turn) for {}",
                    crate::task::task_file_name(path)
                );
            }
        }

        // Heuristic guard: screenshot requested but no attachment
        let user_asked_screenshot = user_explicitly_asked_for_screenshot(&request_for_verification);
        if (user_asked_screenshot || screenshot_requested_by_tool_run)
            && attachment_paths.is_empty()
        {
            response_content
                .push_str("\n\nNote: A screenshot was requested but none was attached.");
            info!(
                "Agent router: heuristic guard — screenshot requested but no attachment, appended note"
            );
        }

        // User-facing note when browser action limit was reached (032_browser_loop_and_status_fix_plan).
        if browser_tool_cap_reached {
            response_content.push_str(&format!(
                "\n\nNote: Browser action limit ({} per run) was reached; some actions were skipped.",
                MAX_BROWSER_TOOLS_PER_RUN
            ));
            info!(
                "Agent router: browser tool cap was reached, appended user-facing note"
            );
        }

        // Completion verification: one short Ollama call; if not satisfied, retry once (A2) or append disclaimer
        let criteria_count = success_criteria.as_ref().map(|c| c.len()).unwrap_or(0);
        info!(
            "Agent router [{}]: running completion verification ({} criteria, {} attachment(s))",
            request_id,
            criteria_count,
            attachment_paths.len()
        );
        match verify_completion(
            &request_for_verification,
            &response_content,
            &attachment_paths,
            success_criteria.as_deref(),
            last_browser_extract.as_deref(),
            last_news_search_was_hub_only,
            model_override.clone(),
            options_override.clone(),
        )
        .await
        {
            Ok((false, reason)) => {
                // Use only the user's question for memory search — not the verification reason.
                // The reason contains generic words ("request", "verified", "assistant", "tools") that
                // match unrelated memory (e.g. GitLab/Redmine) and confuse the retry.
                let memory_snippet = search_memory_for_request(
                    &request_for_verification,
                    None,
                    discord_reply_channel_id,
                );
                if retry_on_verification_no {
                    let retry_base = reason
                    .as_deref()
                    .map(|r| format!("Verification said we didn't fully complete: {}. Complete the remaining steps now, then reply.", r.trim()))
                    .unwrap_or_else(|| "Verification said we didn't fully complete. Complete the remaining steps now, then reply.".to_string());
                    // When verifier said no and the reason mentions screenshots/attachments, steer the
                    // model to reply with a summary and DONE — do not invoke AGENT: orchestrator or
                    // create tasks, which can lead to unrelated actions (e.g. organize ~/tmp).
                    let reason_lower = reason
                        .as_deref()
                        .map(|r| r.to_lowercase())
                        .unwrap_or_default();
                    let reason_about_attachments = reason_lower.contains("screenshot")
                        || reason_lower.contains("attachment")
                        || (reason_lower.contains("missing")
                            && (reason_lower.contains("upload") || reason_lower.contains("sent")));
                    let reason_about_time_or_data = reason_lower.contains("time")
                        || reason_lower.contains("actual data")
                        || reason_lower.contains("spent time")
                        || reason_lower.contains("project parameter")
                        || (reason_lower.contains("missing")
                            && (reason_lower.contains("data")
                                || reason_lower.contains("parameter")));
                    let reason_about_json_format = reason_lower.contains("json format")
                        || reason_lower.contains("not in json")
                        || (reason_lower.contains("response") && reason_lower.contains("json"));
                    let reason_about_news_sourcing = reason_lower.contains("source")
                        || reason_lower.contains("date")
                        || reason_lower.contains("publication")
                        || reason_lower.contains("credible")
                        || reason_lower.contains("article");
                    let response_has_ticket_summary = response_content.len() > 150
                        && (response_content.contains("Subject")
                            || response_content.contains("Description")
                            || response_content.contains("Status")
                            || response_content.contains("Redmine"));
                    let retry_base_with_hint = if is_redmine_time_entries_request(
                        &request_for_verification,
                    ) {
                        let (from, to) = redmine_time_entries_range(&request_for_verification);
                        format!(
                            "This is a Redmine time-entry list/report request. Stay in that domain only. Do not use BROWSER_*, FETCH_URL, RUN_CMD, TASK_*, or single-issue endpoints like /issues/{{id}}.json unless the user explicitly asks for them. Base the answer only on the actual Redmine time-entry data already fetched, or re-fetch the same period with REDMINE_API: GET /time_entries.json?from={}&to={}&limit=100 if needed. If the result is empty, say no time entries or worked tickets were found for that period. Do not mention screenshots or attachments. Do not return raw tool directives as the final user answer.\n\n{}",
                            from, to, retry_base
                        )
                    } else if is_redmine_review_or_summarize_only(&request_for_verification)
                        && response_has_ticket_summary
                    {
                        "The request was only to review/summarize. A summary was already provided. Reply with a brief confirmation and DONE: success; do not update or close the ticket.".to_string()
                    } else if reason_about_json_format {
                        format!(
                            "Success criteria require a response in JSON format. Reply with **valid JSON only** (e.g. total hours, project breakdown, user contributions); do not reply with prose or markdown lists.\n\n{}",
                            retry_base
                        )
                    } else if is_news_query(&request_for_verification) && reason_about_news_sourcing
                    {
                        info!(
                            "Agent router: verification NO for news (article-grade/sourcing); retrying with PERPLEXITY_SEARCH hint"
                        );
                        format!(
                            "This is a news-summary request. Stay in search-and-summary mode only. Re-run PERPLEXITY_SEARCH with a refined query if needed, but do **not** browse generic homepages, do **not** open BBC/CNN/NYTimes landing pages, and do **not** use screenshots or attachments. Reply with 3 concise bullet points. Each bullet must include: headline/topic, source name, publication date, and one-sentence factual summary. Prefer article-like results; if only hub/landing pages are available, say that clearly.\n\n{}",
                            retry_base
                        )
                    } else if reason_about_time_or_data
                        && (request_for_verification.to_lowercase().contains("redmine")
                            || request_for_verification.to_lowercase().contains("time")
                            || request_for_verification.to_lowercase().contains("spent"))
                    {
                        format!(
                            "Use the correct Redmine API for time entries: REDMINE_API: GET /time_entries.json with from= and to= for the requested period (for example 2026-03-01..2026-03-31 for this month, or the same day for today) and include limit=100 so the results are not truncated. Omit optional filters like user_id or project_id unless the user explicitly asked for them. Do not use /search.json for time entries. Then reply with the data or a clear summary.\n\n{}",
                            retry_base
                        )
                    } else if !attachment_paths.is_empty() && reason_about_attachments {
                        let n = attachment_paths.len();
                        format!(
                            "The app already attached {} file(s) to this reply. If the only missing item was screenshots/attachments, \
                         reply with a brief summary of what was done and end with **DONE: success**. \
                         Do not invoke AGENT: orchestrator and do not create new tasks.\n\n{}",
                            n, retry_base
                        )
                    } else if attachment_paths.is_empty()
                        && user_explicitly_asked_for_screenshot(&request_for_verification)
                        && reason_lower.contains("screenshot")
                    {
                        format!(
                            "Screenshots could not be attached to this reply. Reply with a brief summary of what was done (e.g. screenshot taken and saved), state that the app could not attach it to Discord, and end with **DONE: no**. \
                         Do not invoke AGENT: orchestrator and do not create new tasks.\n\n{}",
                            retry_base
                        )
                    } else if reason_lower.contains("cookie")
                        || (reason_lower.contains("consent") && reason_lower.contains("banner"))
                    {
                        format!(
                            "Original request: \"{}\". Verification said the cookie consent banner was not addressed. A screenshot may already have been taken. Complete the remaining step: dismiss the cookie banner (BROWSER_CLICK on the consent button using the Elements list) then BROWSER_SCREENSHOT: current if needed, or reply with a brief summary and **DONE: no** if the browser session is no longer available.\n\n{}",
                            request_for_verification.trim(),
                            retry_base
                        )
                    } else if is_browser_task_request(&request_for_verification)
                        && (last_browser_extract.is_some()
                            || response_content.contains("Current page:")
                            || reason_lower.contains("browser")
                            || reason_lower.contains("click")
                            || reason_lower.contains("video")
                            || reason_lower.contains("screenshot"))
                    {
                        browser_retry_grounding_prompt(&request_for_verification, &retry_base)
                    } else {
                        retry_base
                    };
                    let retry_question = if is_news_query(&request_for_verification) {
                        retry_base_with_hint
                    } else if let Some(ref from_memory) = memory_snippet {
                        format!(
                            "From memory (possibly relevant):\n{}\n\n{}",
                            from_memory, retry_base_with_hint
                        )
                    } else {
                        retry_base_with_hint
                    };
                    info!(
                        "Agent router [{}]: verification said not satisfied, retrying once with: {}...",
                        request_id,
                        retry_question.chars().take(60).collect::<String>()
                    );
                    let mut updated_history: Vec<crate::ollama::ChatMessage> =
                        conversation_history.clone();
                    updated_history.push(crate::ollama::ChatMessage {
                        role: "user".to_string(),
                        content: question.to_string(),
                        images: None,
                    });
                    updated_history.push(crate::ollama::ChatMessage {
                        role: "assistant".to_string(),
                        content: response_content.clone(),
                        images: None,
                    });
                    let pass_intermediate =
                        discord_reply_channel_id.map(|_| response_content.clone());
                    return answer_with_ollama_and_fetch(
                        retry_question.as_str(),
                        status_tx,
                        discord_reply_channel_id,
                        discord_user_id,
                        discord_user_name,
                        model_override,
                        options_override,
                        skill_content,
                        agent_override,
                        allow_schedule,
                        Some(updated_history),
                        escalation,
                        false, // don't retry again
                        from_remote,
                        None,              // don't re-send attachment images on retry
                        pass_intermediate, // format reply as Intermediate + Final when returning to Discord
                        true,              // is_verification_retry: keep context, skip NEW_TOPIC
                        Some(request_for_verification.clone()),
                        success_criteria.clone(),
                        discord_is_dm,
                        Some(request_id.clone()), // same request_id for end-to-end log correlation
                        1,                       // retry_count
                    )
                    .await;
                }
                let reason_preview = reason
                    .as_deref()
                    .map(|r| r.chars().take(80).collect::<String>())
                    .unwrap_or_default();
                // Handoff: when local model didn't satisfy, try cursor-agent for any task (coding or general, e.g. news/screenshot requests).
                let try_cursor_agent_handoff =
                    crate::commands::cursor_agent::is_cursor_agent_available();
                if try_cursor_agent_handoff {
                    info!(
                        "Agent router: verification not satisfied, handing off to Cursor Agent (general fallback)"
                    );
                    let request_clone = request_for_verification.clone();
                    match tokio::task::spawn_blocking(move || {
                        crate::commands::cursor_agent::run_cursor_agent(&request_clone)
                    })
                    .await
                    .map_err(|e| format!("Cursor Agent handoff task: {}", e))
                    .and_then(|r| r)
                    {
                        Ok(cursor_result) => {
                            info!(
                                "Agent router: cursor-agent handoff completed ({} chars)",
                                cursor_result.len()
                            );
                            response_content.clear();
                            response_content.push_str(
                                "The local model couldn't fully complete this, so I handed it off to Cursor Agent. Here's what it did:\n\n",
                            );
                            response_content.push_str(cursor_result.trim());
                        }
                        Err(e) => {
                            info!("Agent router: cursor-agent handoff failed: {}", e);
                            let disclaimer = reason
                                .map(|r| {
                                    format!(
                                        "\n\nNote: We may not have fully met your request: {}.",
                                        r.trim()
                                    )
                                })
                                .unwrap_or_else(|| {
                                    "\n\nNote: We may not have fully met your request.".to_string()
                                });
                            response_content.push_str(&disclaimer);
                        }
                    }
                } else {
                let disclaimer = reason
                    .map(|r| {
                        format!(
                            "\n\nNote: We may not have fully met your request: {}.",
                            r.trim()
                        )
                    })
                    .unwrap_or_else(|| {
                        "\n\nNote: We may not have fully met your request.".to_string()
                    });
                response_content.push_str(&disclaimer);
                }
                // Do not append "From past sessions" memory dump to the user-visible reply — it creates a messy mix and confuses Discord users.
                if let Some(ref from_memory) = memory_snippet {
                    info!(
                        "Agent router: memory search found {} chars (not appended to reply)",
                        from_memory.len()
                    );
                }
                info!(
                    "Agent router: verification said not satisfied, appended disclaimer (reason: {}...)",
                    reason_preview
                );
            }
            Ok((true, _)) => {
                info!("Agent router: verification passed (satisfied)");
            }
            Err(e) => {
                tracing::debug!("Agent router: verification failed (ignored): {}", e);
            }
        }

        let final_text = if discord_reply_channel_id.is_some()
            && discord_intermediate.as_ref().is_some()
        {
            let inter = discord_intermediate.as_ref().unwrap();
            let same = is_final_same_as_intermediate(inter, &response_content);
            if same {
                format!(
                    "--- Intermediate answer:\n\n{}\n\n---\n\nFinal answer is the same as intermediate.",
                    inter.trim()
                )
            } else {
                format!(
                    "--- Intermediate answer:\n\n{}\n\n---\n\n--- Final answer:\n\n{}\n\n---",
                    inter.trim(),
                    response_content.trim()
                )
            }
        } else {
            response_content
        };

        Ok(OllamaReply {
            text: final_text,
            attachment_paths,
        })
    })
}

/// Run JavaScript via Node.js (if available). Used for RUN_JS in Discord/agent context.
/// Writes code to a temp file and runs `node -e "..."` to eval and print the result.
///
/// **Security:** RUN_JS is agent-triggered and runs with process privileges. Agent or prompt
/// compromise can lead to arbitrary code execution. Treat agent output as untrusted code.
fn run_js_via_node(code: &str) -> Result<String, String> {
    let tmp_dir = crate::config::Config::tmp_js_dir();
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let _ = std::fs::create_dir_all(&tmp_dir);
    let path = tmp_dir.join(format!("mac_stats_js_{}_{}.js", std::process::id(), stamp));
    let path_str = path
        .to_str()
        .ok_or_else(|| "Invalid temp path".to_string())?;

    let mut f = std::fs::File::create(&path).map_err(|e| format!("Create temp file: {}", e))?;
    f.write_all(code.as_bytes())
        .map_err(|e| format!("Write temp file: {}", e))?;
    f.flush().map_err(|e| format!("Flush: {}", e))?;
    drop(f);

    // Node -e script: read file, eval code, print result (no user code in -e, so no escaping).
    let eval_script = r#"const fs=require('fs');const p=process.argv[1];const c=fs.readFileSync(p,'utf8');try{const r=eval(c);console.log(r!==undefined?String(r):'undefined');}catch(e){console.error(e.message);process.exit(1);}"#;
    let out = Command::new("node")
        .arg("-e")
        .arg(eval_script)
        .arg(path_str)
        .output()
        .map_err(|e| format!("Node not available or failed: {}", e))?;

    let _ = std::fs::remove_file(&path);

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(stderr.trim().to_string());
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(stdout.trim().to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaChatWithExecutionRequest {
    pub question: String,
    pub system_prompt: Option<String>,
    pub conversation_history: Option<Vec<crate::ollama::ChatMessage>>,
    /// When true (default), stream response chunks to frontend via "ollama-chat-chunk" for better UX.
    #[serde(default = "default_stream_true")]
    pub stream: bool,
}

fn default_stream_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaChatWithExecutionResponse {
    pub needs_code_execution: bool,
    pub code: Option<String>,
    pub intermediate_response: Option<String>,
    pub final_answer: Option<String>,
    pub error: Option<String>,
    pub context_message: Option<String>, // Store context for follow-up
}

/// If the CPU window is not open, schedule opening or showing it on the main thread so the user can see the chat.
fn ensure_cpu_window_open() {
    use crate::state::APP_HANDLE;
    use crate::ui::status_bar::create_cpu_window;

    let need_open = APP_HANDLE
        .get()
        .and_then(|app_handle| {
            app_handle
                .get_window("cpu")
                .and_then(|w| w.is_visible().ok())
                .map(|visible| !visible)
        })
        .unwrap_or(true);

    if !need_open {
        return;
    }
    if let Some(app_handle) = APP_HANDLE.get() {
        let app_handle = app_handle.clone();
        let _ = app_handle.run_on_main_thread(move || {
            if let Some(handle) = APP_HANDLE.get() {
                if let Some(window) = handle.get_window("cpu") {
                    if !window.is_visible().unwrap_or(true) {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                } else {
                    create_cpu_window(handle);
                }
            }
        });
    }
}

/// Unified Ollama chat command that handles code execution flow
/// This command:
/// 1. Gets system metrics
/// 2. Sends question to Ollama
/// 3. Handles FETCH_URL tool (fetch page, send content back)
/// 4. Detects if code needs execution
/// 5. Returns structured response
#[tauri::command]
pub async fn ollama_chat_with_execution(
    request: OllamaChatWithExecutionRequest,
) -> Result<OllamaChatWithExecutionResponse, String> {
    use crate::metrics::format_metrics_for_ai_context;
    use tracing::info;

    ensure_cpu_window_open();

    info!(
        "Ollama Chat with Execution: Starting for question: {}",
        request.question
    );

    // Include gathered metrics so the model can answer accurately when the user asks about CPU, RAM, etc.
    let metrics_block = format_metrics_for_ai_context();
    let context_message = format!("{}\n\nUser question: {}", metrics_block, request.question);

    // Get system prompt: use soul.md (~/.mac-stats/agents/soul.md or bundled default) + tools when not overridden
    let system_prompt = request
        .system_prompt
        .unwrap_or_else(default_non_agent_system_prompt);

    // Build messages array with conversation history
    let mut messages = vec![crate::ollama::ChatMessage {
        role: "system".to_string(),
        content: system_prompt.clone(),
        images: None,
    }];

    // Add conversation history if provided (exclude system messages - we already have one)
    if let Some(ref history) = request.conversation_history {
        for msg in history {
            // Only add user and assistant messages from history (skip system messages)
            if msg.role == "user" || msg.role == "assistant" {
                messages.push(msg.clone());
            }
        }
        info!(
            "Ollama Chat with Execution: Added {} messages from conversation history",
            history
                .iter()
                .filter(|m| m.role == "user" || m.role == "assistant")
                .count()
        );
    }

    // Add current user message
    messages.push(crate::ollama::ChatMessage {
        role: "user".to_string(),
        content: context_message.clone(),
        images: None,
    });

    info!("Ollama Chat with Execution: Sending initial request to Ollama (stream={})", request.stream);
    let mut response = if request.stream {
        send_ollama_chat_messages_streaming(messages.clone(), None, None).await
    } else {
        send_ollama_chat_messages(messages.clone(), None, None).await
    }
    .map_err(|e| format!("Failed to send chat request: {}", e))?;

    let mut response_content = response.message.content.clone();
    const MAX_FETCH_ITERATIONS: u32 = 3;
    let mut fetch_count: u32 = 0;

    // FETCH_URL tool loop: if model returns FETCH_URL: <url>, fetch page and send content back to Ollama
    while fetch_count < MAX_FETCH_ITERATIONS {
        let url = match parse_fetch_url_from_response(&response_content) {
            Some(u) => u,
            None => break,
        };
        fetch_count += 1;
        info!("Ollama Chat with Execution: FETCH_URL requested: {}", url);

        let page_content = crate::commands::browser::fetch_page_content(&url)
            .map_err(|e| format!("Fetch page failed: {}", e))?;
        info!(
            "Ollama Chat with Execution: Fetched {} chars from {}",
            page_content.len(),
            url
        );

        // Build follow-up: current messages + assistant's FETCH_URL message + user with page content
        let mut follow_up_messages = messages.clone();
        follow_up_messages.push(crate::ollama::ChatMessage {
            role: "assistant".to_string(),
            content: response_content.clone(),
            images: None,
        });
        follow_up_messages.push(crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: format!(
                "Here is the page content:\n\n{}\n\nPlease answer the user's question based on this content.",
                page_content
            ),
            images: None,
        });

        let follow_up_request = ChatRequest {
            messages: follow_up_messages.clone(),
        };
        response = ollama_chat(follow_up_request)
            .await
            .map_err(|e| format!("Failed to send follow-up after fetch: {}", e))?;
        response_content = response.message.content.clone();
        messages = follow_up_messages;
        info!(
            "Ollama Chat with Execution: Received response after fetch ({} chars)",
            response_content.len()
        );
    }

    info!(
        "Ollama Chat with Execution: Received response ({} chars)",
        response_content.len()
    );

    // Process response content - handle escaped newlines
    let mut processed_content = response_content.replace("\\n", "\n");
    // Remove "javascript\n" if present
    processed_content = processed_content.replace("javascript\n", "");

    if let Some(code) = crate::commands::tool_parsing::detect_and_extract_js_code(&processed_content) {
        info!(
            "Ollama Chat with Execution: Extracted code ({} chars):\n{}",
            code.len(),
            code
        );

        if code.is_empty() {
            return Ok(OllamaChatWithExecutionResponse {
                needs_code_execution: false,
                code: None,
                intermediate_response: Some(processed_content),
                final_answer: None,
                error: Some("No code found in code-assistant response".to_string()),
                context_message: Some(context_message),
            });
        }

        return Ok(OllamaChatWithExecutionResponse {
            needs_code_execution: true,
            code: Some(code),
            intermediate_response: Some(processed_content),
            final_answer: None,
            error: None,
            context_message: Some(context_message),
        });
    }

    // Regular response, no code execution needed
    info!("Ollama Chat with Execution: Regular response (no code execution)");
    Ok(OllamaChatWithExecutionResponse {
        needs_code_execution: false,
        code: None,
        intermediate_response: None,
        final_answer: Some(processed_content),
        error: None,
        context_message: Some(context_message),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaChatContinueResponse {
    pub needs_code_execution: bool,
    pub code: Option<String>,
    pub intermediate_response: Option<String>,
    pub final_answer: Option<String>,
    pub context_message: Option<String>, // For next iteration if needed
}

/// Continue Ollama chat after code execution
/// Takes the execution result and sends follow-up to Ollama
/// Returns structured response - may need more code execution (ping-pong)
#[tauri::command]
pub async fn ollama_chat_continue_with_result(
    _code: String,
    execution_result: String,
    original_question: String,
    context_message: String,
    intermediate_response: String,
    system_prompt: Option<String>,
    conversation_history: Option<Vec<crate::ollama::ChatMessage>>,
) -> Result<OllamaChatContinueResponse, String> {
    use tracing::info;

    ensure_cpu_window_open();

    info!(
        "Ollama Chat Continue: Code executed, result: {}",
        execution_result
    );

    let system_prompt = system_prompt.unwrap_or_else(default_non_agent_system_prompt);

    let follow_up_message = format!(
        "I have executed your last codeblocks and the result is: {}\n\nCan you now answer the original question: {}?",
        execution_result, original_question
    );

    // Build messages array with conversation history
    let mut messages = vec![crate::ollama::ChatMessage {
        role: "system".to_string(),
        content: system_prompt.clone(),
        images: None,
    }];

    // Add conversation history if provided (exclude system messages - we already have one)
    if let Some(ref history) = conversation_history {
        for msg in history {
            // Only add user and assistant messages from history (skip system messages)
            if msg.role == "user" || msg.role == "assistant" {
                messages.push(msg.clone());
            }
        }
        info!(
            "Ollama Chat Continue: Added {} messages from conversation history",
            history
                .iter()
                .filter(|m| m.role == "user" || m.role == "assistant")
                .count()
        );
    }

    // Add the conversation flow for this code execution cycle
    messages.push(crate::ollama::ChatMessage {
        role: "user".to_string(),
        content: context_message.clone(),
        images: None,
    });
    messages.push(crate::ollama::ChatMessage {
        role: "assistant".to_string(),
        content: intermediate_response.clone(),
        images: None,
    });
    messages.push(crate::ollama::ChatMessage {
        role: "user".to_string(),
        content: follow_up_message,
        images: None,
    });

    let chat_request = ChatRequest { messages };

    info!("Ollama Chat Continue: Sending follow-up to Ollama");
    let response = ollama_chat(chat_request)
        .await
        .map_err(|e| format!("Failed to send follow-up: {}", e))?;

    let response_content = response.message.content;
    info!(
        "Ollama Chat Continue: Received response ({} chars)",
        response_content.len()
    );

    // Process response content - handle escaped newlines
    let mut processed_content = response_content.replace("\\n", "\n");
    // Remove "javascript\n" if present
    processed_content = processed_content.replace("javascript\n", "");

    if let Some(code) = crate::commands::tool_parsing::detect_and_extract_js_code(&processed_content) {
        info!(
            "Ollama Chat Continue: Extracted code for re-execution ({} chars):\n{}",
            code.len(),
            code
        );

        if code.is_empty() {
            return Ok(OllamaChatContinueResponse {
                needs_code_execution: false,
                code: None,
                intermediate_response: Some(processed_content),
                final_answer: None,
                context_message: Some(context_message),
            });
        }

        return Ok(OllamaChatContinueResponse {
            needs_code_execution: true,
            code: Some(code),
            intermediate_response: Some(processed_content),
            final_answer: None,
            context_message: Some(context_message),
        });
    }

    // Final answer received
    info!("Ollama Chat Continue: Received final answer (no more code execution needed)");
    Ok(OllamaChatContinueResponse {
        needs_code_execution: false,
        code: None,
        intermediate_response: None,
        final_answer: Some(processed_content),
        context_message: Some(context_message),
    })
}

#[cfg(test)]
mod tests {
    use super::{
        build_agent_runtime_context,
        extract_last_prefixed_argument, final_reply_from_tool_results,
        original_request_for_retry, parse_agent_tool_from_response,
        sanitize_success_criteria, summarize_response_for_verification,
        truncate_text_on_line_boundaries, verification_news_hub_only_block,
    };

    #[test]
    fn extract_last_prefixed_argument_prefers_last_run_cmd_literal() {
        let question = "Run this command and show me the output: RUN_CMD: cat /etc/hosts";
        assert_eq!(
            extract_last_prefixed_argument(question, "RUN_CMD:"),
            Some("cat /etc/hosts".to_string())
        );
    }

    #[test]
    fn parse_agent_tool_from_response_supports_redmine_api() {
        assert_eq!(
            parse_agent_tool_from_response(
                "REDMINE_API: GET /issues/7209.json?include=journals,attachments"
            ),
            Some((
                "REDMINE_API".to_string(),
                "GET /issues/7209.json?include=journals,attachments".to_string()
            ))
        );
    }

    #[test]
    fn parse_agent_tool_from_response_supports_recommend_wrapper() {
        assert_eq!(
            parse_agent_tool_from_response(
                "RECOMMEND: REDMINE_API: GET /issues/7209.json?include=journals,attachments"
            ),
            Some((
                "REDMINE_API".to_string(),
                "GET /issues/7209.json?include=journals,attachments".to_string()
            ))
        );
    }

    #[test]
    fn parse_agent_tool_from_response_skips_unsupported_prefix_tool() {
        assert_eq!(
            parse_agent_tool_from_response(
                "RECOMMEND: RUN_CMD: date +%Y-%m-%d then REDMINE_API GET /time_entries.json?from=2026-03-06&to=2026-03-06&limit=100"
            ),
            Some((
                "REDMINE_API".to_string(),
                "GET /time_entries.json?from=2026-03-06&to=2026-03-06&limit=100".to_string()
            ))
        );
    }

    #[test]
    fn build_agent_runtime_context_anchors_today_to_local_date() {
        let now = chrono::DateTime::parse_from_rfc3339("2026-03-07T00:15:16+02:00").unwrap();
        let context = build_agent_runtime_context(now);
        assert!(context.contains("Current local date: 2026-03-07"));
        assert!(context.contains("Current UTC date: 2026-03-06"));
        assert!(context.contains(
            "For date-sensitive tool calls such as Redmine \"today\" queries, use the current UTC date (2026-03-06)"
        ));
    }

    #[test]
    fn truncate_text_on_line_boundaries_does_not_cut_mid_line() {
        let text = "line one\nline two is longer\nline three";
        assert_eq!(
            truncate_text_on_line_boundaries(text, 12),
            "line one\n[truncated]"
        );
    }

    #[test]
    fn summarize_response_for_verification_prefers_structured_redmine_summary() {
        let response = "Redmine API result:\n\nDerived Redmine time-entry summary\nRange: 2026-03-06..2026-03-06\nFetched 1 time entry (total available: 1). Total hours: 2.00\n\nTickets worked:\n- #7209 Fix login — 2.00h across 1 entry (project: Core; users: Ralf; activities: Development)\n\nEntry details:\n- 2026-03-06 | entry 1 | #7209 Fix login | 2.00h | Development | project: Core | user: Ralf\n\nUse this data to answer the user's question.";
        let summary = summarize_response_for_verification(
            "Provide me the list of redmine tickets work on today.",
            response,
            0,
        );
        assert!(summary.contains("Derived Redmine time-entry summary"));
        assert!(summary.contains("#7209 Fix login"));
        assert!(!summary.contains("Entry details:"));
    }

    #[test]
    fn final_reply_from_tool_results_uses_redmine_summary_not_raw_tool_wrapper() {
        let tool_result = "Redmine API result:\n\nDerived Redmine time-entry summary\nRange: 2026-03-06..2026-03-06\nFetched 0 time entries (total available: 0). Total hours: 0.00\n\nTickets worked:\n- None found in these time entries.\n\nUse this data to answer the user's question.";
        let reply = final_reply_from_tool_results(
            "Provide me the list of redmine tickets work on today.",
            tool_result,
        );
        assert!(reply.starts_with("Derived Redmine time-entry summary"));
        assert!(!reply.starts_with("Redmine API result:"));
    }

    #[test]
    fn original_request_for_retry_uses_last_real_user_message() {
        let history = vec![
            crate::ollama::ChatMessage {
                role: "user".to_string(),
                content: "Original request".to_string(),
                images: None,
            },
            crate::ollama::ChatMessage {
                role: "assistant".to_string(),
                content: "Intermediate reply".to_string(),
                images: None,
            },
        ];
        let retry_prompt = "Verification said we didn't fully complete the task.";
        assert_eq!(
            original_request_for_retry(retry_prompt, Some(&history), true),
            "Original request".to_string()
        );
    }

    #[test]
    fn original_request_for_retry_ignores_history_when_not_retrying() {
        let history = vec![crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: "Older request".to_string(),
            images: None,
        }];
        assert_eq!(
            original_request_for_retry("Current request", Some(&history), false),
            "Current request".to_string()
        );
    }

    #[test]
    fn sanitize_success_criteria_removes_invented_last_30_days_window() {
        let criteria = vec![
            "reliable news sources cited".to_string(),
            "recent articles (last 30 days)".to_string(),
            "source name and publication date included".to_string(),
        ];
        assert_eq!(
            sanitize_success_criteria(
                "Can you look on the Internet for news involving Barcelona? Mention sources and dates.",
                criteria
            ),
            vec![
                "reliable news sources cited".to_string(),
                "recent news items were summarized".to_string(),
                "source name and publication date included".to_string(),
            ]
        );
    }

    #[test]
    fn sanitize_success_criteria_removes_invented_named_source_examples() {
        let criteria = vec![
            "articles from credible sources like BBC or CNN".to_string(),
            "dates of the news articles".to_string(),
        ];
        assert_eq!(
            sanitize_success_criteria(
                "Can you look on the Internet for news involving Barcelona? Mention sources and dates.",
                criteria
            ),
            vec![
                "credible named sources cited".to_string(),
                "dates of the news articles".to_string(),
            ]
        );
    }

    #[test]
    fn sanitize_success_criteria_relaxes_video_playability_for_review_requests() {
        let criteria = vec![
            "homepage loaded successfully".to_string(),
            "\"about\" section navigable".to_string(),
            "videos playable without errors".to_string(),
        ];
        assert_eq!(
            sanitize_success_criteria(
                "Use browser to review www.amvara.de, click on about and review videos.",
                criteria
            ),
            vec![
                "homepage loaded successfully".to_string(),
                "\"about\" section navigable".to_string(),
                "video availability or playability was checked".to_string(),
            ]
        );
    }

    #[test]
    fn sanitize_success_criteria_removes_invented_football_focus_for_generic_barcelona_news() {
        let criteria = vec![
            "recent news articles about Barcelona football club".to_string(),
            "verified sources such as reputable sports websites".to_string(),
            "coverage of major events or updates related to the team".to_string(),
        ];
        assert_eq!(
            sanitize_success_criteria(
                "Can you look on the Internet for news involving Barcelona?",
                criteria
            ),
            vec![
                "recent news items involving Barcelona were summarized".to_string(),
                "credible named sources cited".to_string(),
                "major recent developments involving Barcelona were covered".to_string(),
            ]
        );
    }

    #[test]
    fn sanitize_success_criteria_removes_invented_last_week_for_generic_news() {
        let criteria = vec![
            "recent news articles about Barcelona from credible sources".to_string(),
            "information includes dates and relevant details".to_string(),
            "articles are from the last week".to_string(),
        ];
        assert_eq!(
            sanitize_success_criteria(
                "Can you look on the Internet for news involving Barcelona?",
                criteria
            ),
            vec![
                "recent news articles about Barcelona from credible sources".to_string(),
                "information includes dates and relevant details".to_string(),
            ]
        );
    }

    #[test]
    fn verification_news_hub_only_block_included_when_hub_only_and_news_query() {
        let block = verification_news_hub_only_block(Some(true), "what's the latest news on Barcelona?");
        assert!(!block.is_empty());
        assert!(block.contains("hub/landing/tag/standings"));
        assert!(block.contains("article-grade sources were not found"));
    }

    #[test]
    fn verification_news_hub_only_block_empty_when_not_news_query() {
        assert_eq!(
            verification_news_hub_only_block(Some(true), "what is 2+2?"),
            ""
        );
    }

    #[test]
    fn verification_news_hub_only_block_empty_when_not_hub_only() {
        assert_eq!(
            verification_news_hub_only_block(Some(false), "latest headlines"),
            ""
        );
        assert_eq!(
            verification_news_hub_only_block(None, "latest headlines"),
            ""
        );
    }
}
