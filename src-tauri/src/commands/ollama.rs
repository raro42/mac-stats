//! Ollama Tauri commands

use tauri::Manager;

use crate::config::Config;
use crate::ollama::{
    ChatMessage, EmbedInput, EmbedResponse, ListResponse, OllamaClient, OllamaConfig, PsResponse,
    VersionResponse,
};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::pin::Pin;
use std::process::Command;
use std::str::FromStr;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::atomic::Ordering;

/// Tool instructions appended to soul for non-agent chat (code execution + FETCH_URL).
const NON_AGENT_TOOL_INSTRUCTIONS: &str = "\n\nYou are a general purpose AI. If you are asked for actual data like day or weather information, or flight information or stock information. Then we need to compile that information using specially crafted clients for doing so. You will put \"[variable-name]\" into the answer to signal that we need to go another step and ask an agent to fulfil the answer.\n\nWhenever asked with \"[variable-name]\", you must provide a javascript snippet to be executed in the browser console to retrieve that information. Mark the answer to be executed as javascript. Do not put any other words around it. Do not insert formatting. Only return the code to be executed. This is needed for the next AI to understand and execute the same. When answering, use the role: code-assistant in the response. When you return executable code:\n- Start the response with: ROLE=code-assistant\n- On the next line, output ONLY executable JavaScript\n- Do not add explanations or formatting\n\nFor web pages: To fetch a page and use its content (e.g. \"navigate to X and get Y\"), reply with exactly one line: FETCH_URL: <full URL> (e.g. FETCH_URL: https://www.example.com). The app will fetch the page and give you the text; then answer the user based on that.";

/// Load soul content from ~/.mac-stats/agents/soul.md (or write default there if missing).
fn load_soul_content() -> String {
    Config::load_soul_content()
}

/// Load global memory (~/.mac-stats/agents/memory.md) for inclusion in system prompt.
fn load_global_memory_block() -> String {
    let path = Config::memory_file_path();
    let content = match std::fs::read_to_string(&path) {
        Ok(s) => s.trim().to_string(),
        Err(_) => return String::new(),
    };
    if content.is_empty() {
        return String::new();
    }
    format!(
        "\n\n## Memory (lessons learned — follow these)\n\n{}\n\n",
        content
    )
}

/// Load per-channel Discord memory (~/.mac-stats/agents/memory-discord-{id}.md). Returns empty if missing.
fn load_channel_memory_block(channel_id: u64) -> String {
    let path = Config::memory_file_path_for_discord_channel(channel_id);
    let content = match std::fs::read_to_string(&path) {
        Ok(s) => s.trim().to_string(),
        Err(_) => return String::new(),
    };
    if content.is_empty() {
        return String::new();
    }
    format!(
        "\n\n## Memory (this channel — follow these)\n\n{}\n\n",
        content
    )
}

/// Load memory for the current request: global + per-channel when replying in a Discord channel.
/// Keeps channel conversations from mixing; DMs and each server channel have their own lesson file.
fn load_memory_block_for_request(discord_channel_id: Option<u64>) -> String {
    let global = load_global_memory_block();
    let channel = discord_channel_id
        .map(load_channel_memory_block)
        .unwrap_or_default();
    if channel.is_empty() {
        global
    } else {
        format!("{}{}", global, channel)
    }
}

/// Extract words (alphanumeric, lowercase) for simple keyword matching.
fn words_for_search(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| s.len() >= 2)
        .map(String::from)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect()
}

/// Search memory (global + optional Discord channel) for lines relevant to the request.
/// Returns at most 5 matching lines, or None if no matches. When discord_channel_id is Some,
/// channel memory is merged with global so we search both.
fn search_memory_for_request(
    question: &str,
    reason: Option<&str>,
    discord_channel_id: Option<u64>,
) -> Option<String> {
    let global = std::fs::read_to_string(Config::memory_file_path())
        .ok()
        .unwrap_or_default();
    let channel = discord_channel_id
        .and_then(|id| {
            std::fs::read_to_string(Config::memory_file_path_for_discord_channel(id)).ok()
        })
        .unwrap_or_default();
    let content = format!("{}\n{}", global.trim(), channel.trim())
        .trim()
        .to_string();
    if content.is_empty() {
        return None;
    }
    let mut query_words: Vec<String> = words_for_search(question);
    if let Some(r) = reason {
        query_words.extend(words_for_search(r));
    }
    query_words.sort();
    query_words.dedup();
    if query_words.is_empty() {
        return None;
    }
    // Require at least 2 query words to match so we don't inject memory that only matched one generic word (e.g. "the", "request").
    const MIN_MEMORY_MATCH_WORDS: usize = 2;
    let mut scored: Vec<(usize, String)> = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let line_lower = line.to_lowercase();
            let score = query_words
                .iter()
                .filter(|w| line_lower.contains(w.as_str()))
                .count();
            (score, line.to_string())
        })
        .filter(|(score, _)| *score >= MIN_MEMORY_MATCH_WORDS)
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    let top: Vec<String> = scored.into_iter().take(5).map(|(_, line)| line).collect();
    if top.is_empty() {
        None
    } else {
        Some(top.join("\n"))
    }
}

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
fn get_ollama_client() -> &'static Mutex<Option<OllamaClient>> {
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

    let (endpoint, model, api_key, config_temp, config_num_ctx) = {
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
        )
    };

    let effective_model = model_override.unwrap_or(model);
    let options = merge_chat_options(config_temp, config_num_ctx, options_override);

    let timeout_secs = crate::config::Config::ollama_chat_timeout_secs();
    let temp_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

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
        let mut http_request = temp_client.post(&url).json(&chat_request);
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

/// Base agent descriptions (without MCP). Includes RUN_JS, FETCH_URL, BROWSER_*, BRAVE_SEARCH, SCHEDULE.
const AGENT_DESCRIPTIONS_BASE: &str = r#"We have 10 base tools available:

1. **RUN_JS** (JavaScript superpowers): Execute JavaScript in the app context (e.g. browser console). Use for: dynamic data, DOM inspection, client-side state. To invoke: reply with exactly one line: RUN_JS: <JavaScript code>. Note: In some contexts (e.g. Discord) JS is not executed; then answer without running code.

2. **FETCH_URL**: Fetch the full text of a web page. Use for: reading a specific URL's content. To invoke: reply with exactly one line: FETCH_URL: <full URL> (e.g. FETCH_URL: https://www.example.com). The app will return the page text.

3. **BROWSER_SCREENSHOT**: Take a screenshot of the **current page only**. Use BROWSER_SCREENSHOT: current (or BROWSER_SCREENSHOT: with no arg). **You must navigate first**: use BROWSER_NAVIGATE: <url>, then optionally BROWSER_CLICK through links, then BROWSER_SCREENSHOT: current. Never use BROWSER_SCREENSHOT: <url> — that is invalid. For \"screenshot this URL\" use BROWSER_NAVIGATE: <url> then BROWSER_SCREENSHOT: current.

4. **BROWSER_NAVIGATE**, **BROWSER_CLICK**, **BROWSER_INPUT**, **BROWSER_SCROLL**, **BROWSER_EXTRACT**, **BROWSER_SEARCH_PAGE** (lightweight browser): Use for multi-step browser tasks. **Browser mode**: user says \"headless\" → no visible window. User says \"browser\" or default → visible Chrome. BROWSER_NAVIGATE: <url> — open URL and return current page state with numbered elements. BROWSER_CLICK: <index> — click the element at that index (1-based). BROWSER_INPUT: <index> <text> — type text into the element at that index. BROWSER_SCROLL: <direction> — scroll: \"down\", \"up\", \"bottom\", \"top\", or pixels. BROWSER_EXTRACT — return full visible text of current page. BROWSER_SEARCH_PAGE: <pattern> — search page text for a pattern (like grep); returns matches with context. Use to find specific text (e.g. a name) without reading the whole page. For \"find X on this site\": BROWSER_NAVIGATE to start URL, BROWSER_CLICK through links (Team, Contact, etc.), BROWSER_SEARCH_PAGE: \"X\" to check if found, repeat until found, then BROWSER_SCREENSHOT: current. After each action you get \"Current page: ...\" and an Elements list. **If the Elements list shows cookie consent** (e.g. \"Rechazar todo\", \"Aceptar todo\", \"Accept all\", \"Reject all\"), **click the accept button first** (use the index of \"Aceptar todo\" or \"Accept all\") before typing in search boxes or submitting. Reply with exactly one line per tool (e.g. BROWSER_NAVIGATE: https://google.com then BROWSER_CLICK: 27 for Aceptar todo, then BROWSER_INPUT: 10 <query> then BROWSER_CLICK: 9 for the search button).

5. **BRAVE_SEARCH**: Web search via Brave Search API. Use for: finding current info, facts, multiple sources. To invoke: reply with exactly one line: BRAVE_SEARCH: <search query>. The app will return search results.

6. **SCHEDULE** (scheduler): Add a task to run at scheduled times (recurring or one-shot). Use when the user wants something to run later or repeatedly. Three formats (reply exactly one line):
   - SCHEDULE: every N minutes <task> (e.g. SCHEDULE: every 5 minutes Execute RUN_JS to fetch CPU and RAM).
   - SCHEDULE: <cron expression> <task> — cron is 6-field (sec min hour day month dow) or 5-field (min hour day month dow; we accept and prepend 0 for seconds). Examples below.
   - SCHEDULE: at <datetime> <task> — one-shot (e.g. reminder tomorrow 5am: use RUN_CMD: date +%Y-%m-%d to get today, then SCHEDULE: at 2025-02-09T05:00:00 Remind me of my flight). Datetime must be ISO local: YYYY-MM-DDTHH:MM:SS or YYYY-MM-DD HH:MM.
   We add to ~/.mac-stats/schedules.json and return a schedule ID (e.g. discord-1770648842). Always tell the user this ID so they can remove it later with REMOVE_SCHEDULE.

7. **REMOVE_SCHEDULE**: Remove a scheduled task by its ID. Use when the user asks to remove, delete, or cancel a schedule (e.g. "Remove schedule: discord-1770648842"). Reply with exactly one line: REMOVE_SCHEDULE: <schedule-id> (e.g. REMOVE_SCHEDULE: discord-1770648842).

8. **LIST_SCHEDULES**: List all active schedules (id, type, next run, task). Use when the user asks to list schedules, show schedules, what's scheduled, what reminders are set, etc. Reply with exactly one line: LIST_SCHEDULES or LIST_SCHEDULES:.

When you have fully completed the user's request (or cannot complete it), you may end your reply with exactly one line: **DONE: success** (task completed) or **DONE: no** (could not complete). This stops further tool runs; the app still runs completion verification. Prefer DONE when you are done rather than replying with text alone."#;

/// Cron examples for SCHEDULE (6-field: sec min hour day month dow). Shown to the model so it can pick the right pattern (see crontab.guru for more).
const SCHEDULE_CRON_EXAMPLES: &str = r#"

SCHEDULE cron examples (6-field: sec min hour day month dow). Use as SCHEDULE: <expression> <task>:
- Every minute: 0 * * * * *
- Every 5 minutes: 0 */5 * * * *
- Every day at 5:00: 0 0 5 * * *
- Every day at midnight: 0 0 0 * * *
- Every Monday: 0 0 * * * 1
- Every weekday at 9am: 0 0 9 * * 1-5
- Once a day at 8am: 0 0 8 * * *"#;

/// RUN_CMD agent description (appended when ALLOW_LOCAL_CMD is not 0). Allowlist is read from orchestrator skill.md.
fn format_run_cmd_description(num: u32) -> String {
    let allowed = crate::commands::run_cmd::allowed_commands().join(", ");
    format!(
        "\n\n{}. **RUN_CMD** (local read-only): Run a restricted local command. Use for: reading app data under ~/.mac-stats (schedules.json, config, task files), or current time/user (date, whoami), or allowed CLI tools. To invoke: reply with exactly one line: RUN_CMD: <command> [args] (e.g. RUN_CMD: cat ~/.mac-stats/schedules.json, RUN_CMD: date, RUN_CMD: cursor-agent --help). Allowed: {}; file paths must be under ~/.mac-stats; date, whoami, ps, cursor-agent and similar need no path.",
        num, allowed
    )
}

/// Build the SKILL agent description paragraph when skills exist. Use {} for agent number.
/// This text is sent to Ollama in the planning and execution steps so it can recommend and invoke SKILL.
fn build_skill_agent_description(num: u32, skills: &[crate::skills::Skill]) -> String {
    let list: String = skills
        .iter()
        .map(|s| format!("{}-{}", s.number, s.topic))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "\n\n{}. **SKILL**: Use a specialized skill for a focused task (e.g. summarize text, create a joke, get date/time). Each skill runs in a separate Ollama session (no main conversation history); the result is injected back so you can cite or refine it. Prefer SKILL when the user wants a single focused outcome that matches one of the skills below. To invoke: reply with exactly one line: SKILL: <number or topic> [optional task]. Available skills: {}.",
        num, list
    )
}

/// Build the AGENT description paragraph when LLM agents exist. Lists agents by slug or name so the model can invoke AGENT: <slug or id> [task].
fn build_agent_agent_description(num: u32, agents: &[crate::agents::Agent]) -> String {
    let list: String = agents
        .iter()
        .map(|a| a.slug.as_deref().unwrap_or(a.name.as_str()).to_string())
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "\n\n{}. **AGENT**: Run a specialized LLM agent (its own model and prompt). Use when a task fits an agent below. To invoke: reply with exactly one line: AGENT: <slug or id> [optional task]. If no task is given, the current user question is used. Available agents: {}.",
        num, list
    )
}

/// Discord API endpoint list (injected when request is from Discord). Condensed for agent context.
const DISCORD_API_ENDPOINTS_CONTEXT: &str = r#"
IMPORTANT: For Discord tasks, prefer **AGENT: discord-expert** — it fetches guild and channel data via the API and can make multiple calls autonomously.
If calling directly: use DISCORD_API: GET <path> (NOT FETCH_URL — FETCH_URL has no Discord token and will get 401).
Guild/channel data: GET /users/@me/guilds (bot's servers), GET /guilds/{guild_id}/channels (channels in a server). Also: GET /guilds/{guild_id}/members/search?query=name, POST /channels/{channel_id}/messages {"content":"..."}"#;

/// Build agent descriptions string: base, optional SKILL (when skills exist), optional RUN_CMD, then MCP when configured.
/// When from_discord is true and Discord is configured, appends DISCORD_API agent and endpoint list.
/// When question is provided and Redmine is configured, create-context (projects, trackers, etc.) is only appended if the question suggests create/update.
async fn build_agent_descriptions(from_discord: bool, question: Option<&str>) -> String {
    use tracing::info;
    let skills = crate::skills::load_skills();
    let mut base = AGENT_DESCRIPTIONS_BASE.to_string();
    base.push_str(SCHEDULE_CRON_EXAMPLES);
    let mut num = 6u32;
    if !skills.is_empty() {
        base.push_str(&build_skill_agent_description(num, &skills));
        num += 1;
    }
    if crate::commands::run_cmd::is_local_cmd_allowed() {
        base.push_str(&format_run_cmd_description(num));
        base.push_str(" When the user asks you to run a command, organize files, or use cursor-agent, use RUN_CMD or CURSOR_AGENT (if listed below); do not refuse by saying you cannot run external commands.");
        num += 1;
    }
    base.push_str(&format!(
        "\n\n{}. **TASK** (task files under ~/.mac-stats/task/): Use when working on a task file or when the user asks for tasks. When the user wants agents to chat or have a conversation, invoke AGENT: orchestrator (or the right agent) so the conversation runs; do not only create a task. TASK_LIST: default is open and WIP only (reply: TASK_LIST or TASK_LIST: ). TASK_LIST: all — list all tasks grouped by status (reply: TASK_LIST: all when the user asks for all tasks). TASK_SHOW: <path or id> — show that task's content and status to the user. TASK_APPEND: append feedback (reply: TASK_APPEND: <path or task id> <content>). TASK_STATUS: set status (reply: TASK_STATUS: <path or task id> wip|finished|unsuccessful). When the user says \"close the task\", \"finish\", \"mark done\", or \"cancel\" a task, reply TASK_STATUS: <path or id> finished or unsuccessful. TASK_CREATE: create a new task (reply: TASK_CREATE: <topic> <id> <initial content>). Put the **full** user request into the initial content, including duration (e.g. \"research for 15 minutes\"), scope, and topic — the whole content is stored. For cursor-agent tasks follow your skill (section Cursor-agent tasks). If a task with that topic and id already exists, use TASK_APPEND or TASK_STATUS instead. For TASK_APPEND/TASK_STATUS use the task file name (e.g. task-20250222-120000-open) or the short id or topic (e.g. 1, research). TASK_ASSIGN: <path or id> <agent_id> — use scheduler, discord, cpu, or default (CURSOR_AGENT is normalized to scheduler). Paths must be under ~/.mac-stats/task.",
        num
    ));
    num += 1;
    base.push_str(&format!(
        "\n\n{}. **OLLAMA_API** (Ollama model management): List models (with details), get server version, list running models, pull/delete/load/unload models, generate embeddings. Use when the user asks what models are installed, to pull or delete a model, to free memory (unload), or to get embeddings for text. To invoke: reply with exactly one line: OLLAMA_API: <action> [args]. Actions: list_models (no args), version (no args), running (no args), pull <model> [stream true|false], delete <model>, embed <model> <text>, load <model> [keep_alive e.g. 5m], unload <model>. Results are returned as JSON or text.",
        num
    ));
    num += 1;
    if crate::commands::perplexity::is_perplexity_configured().unwrap_or(false) {
        base.push_str(&format!(
            "\n\n{}. **PERPLEXITY_SEARCH**: Web search via Perplexity API. Use for: current info, facts, recent events, multi-source answers. To invoke: reply with exactly one line: PERPLEXITY_SEARCH: <search query>. The app returns search results with snippets and URLs.",
            num
        ));
        num += 1;
    }
    if crate::commands::python_agent::is_python_script_allowed() {
        base.push_str(&format!(
            "\n\n{}. **PYTHON_SCRIPT**: Run Python code. Reply with exactly one line: PYTHON_SCRIPT: <id> <topic>, then put the Python code on the following lines or inside a ```python ... ``` block. The app writes ~/.mac-stats/scripts/python-script-<id>-<topic>.py, runs it with python3, and returns stdout (or error). Use for data processing, calculations, or local scripts.",
            num
        ));
        num += 1;
    }
    if from_discord && crate::discord::get_discord_token().is_some() {
        base.push_str(&format!(
            "\n\n{}. **DISCORD_API**: Call Discord HTTP API to list servers (guilds), channels, members, or get user info. Invoke with one line: DISCORD_API: GET <path> or DISCORD_API: POST <path> [json body]. Path is relative to https://discord.com/api/v10 (e.g. GET /users/@me/guilds, GET /guilds/{{guild_id}}/channels, GET /guilds/{{guild_id}}/members, GET /users/{{user_id}}, POST /channels/{{channel_id}}/messages with body {{\"content\":\"...\"}}).",
            num
        ));
        base.push_str(DISCORD_API_ENDPOINTS_CONTEXT);
        num += 1;
    }
    if crate::commands::cursor_agent::is_cursor_agent_available() {
        base.push_str(&format!(
            "\n\n{}. **CURSOR_AGENT** (Cursor AI coding agent): Delegate coding tasks to the Cursor Agent CLI (an AI pair-programmer with full codebase access). Use when the user asks to write code, refactor, fix bugs, create files, organize a folder, or make changes in a project. To invoke: reply with exactly one line: CURSOR_AGENT: <detailed prompt describing the task>. The result (what cursor-agent did and its output) is returned. You have access to this tool — use it when the user asks to run cursor-agent or to organize/code something; do not say you cannot run external commands.",
            num
        ));
        num += 1;
    }
    if crate::redmine::is_configured() {
        // Short descriptor: when + one-line formats. Full endpoint docs not in prompt; pre-route handles "review ticket N".
        // Time entries: use /time_entries.json with from/to (not /search.json) for spent time / hours this month.
        base.push_str(&format!(
            "\n\n{}. **REDMINE_API**: Redmine issues, projects, and time entries. Use for: review ticket, list/search issues, spent time/hours this month, tickets worked today, create or update issue. Invoke one line: REDMINE_API: GET /issues/{{id}}.json?include=journals,attachments — or GET /time_entries.json?from=YYYY-MM-DD&to=YYYY-MM-DD&limit=100 (optional project_id=ID or user_id=ID) for time entries — or GET /search.json?q=<keyword>&issues=1 — or POST /issues.json {{...}} — or PUT /issues/{{id}}.json {{\"issue\":{{\"notes\":\"...\"}}}}. For spent time, hours, or tickets worked use /time_entries.json with concrete from/to dates and a large enough limit; do not use /search.json. Always .json suffix.",
            num
        ));
        // Create context (projects, trackers, statuses) only when user might create/update — avoids polluting prompt for simple "review ticket N".
        let wants_create_or_update = question
            .map(|q| {
                let q_lower = q.to_lowercase();
                q_lower.contains("create")
                    || q_lower.contains("new issue")
                    || q_lower.contains("update")
                    || q_lower.contains("add comment")
                    || q_lower.contains("post a comment")
                    || q_lower.contains("write ")
            })
            .unwrap_or(true);
        if wants_create_or_update {
            if let Some(ctx) = crate::redmine::get_redmine_create_context().await {
                base.push_str("\n\n");
                base.push_str(&ctx);
            }
        }
        num += 1;
    }
    if get_mastodon_config().is_some() {
        base.push_str(&format!(
            "\n\n{}. **MASTODON_POST**: Post a status (toot) to Mastodon. To invoke: reply with exactly one line: MASTODON_POST: <text to post>. Default visibility is public. Optional visibility prefix: MASTODON_POST: unlisted: <text>, MASTODON_POST: private: <text>, MASTODON_POST: direct: <text>. Keep posts concise (<500 chars). The post URL is returned on success.",
            num
        ));
        num += 1;
    }
    base.push_str(&format!(
        "\n\n{}. **MEMORY_APPEND** (persistent memory): Save a lesson learned for future sessions. Use when something important was discovered (a mistake to avoid, a working approach, a user preference). To invoke: reply with exactly one line: MEMORY_APPEND: <lesson> (in Discord: saves to this channel's memory; otherwise global) or MEMORY_APPEND: agent:<slug-or-id> <lesson> (saves to that agent's memory only). Keep lessons concise and actionable.",
        num
    ));
    num += 1;
    let agent_list = crate::agents::load_agents();
    if !agent_list.is_empty() {
        base.push_str(&build_agent_agent_description(num, &agent_list));
        num += 1;
    }
    let Some(server_url) = crate::mcp::get_mcp_server_url() else {
        return base;
    };
    info!("Agent router: MCP configured, fetching tool list from server");
    match crate::mcp::list_tools(&server_url).await {
        Ok(tools) => {
            if tools.is_empty() {
                info!("Agent router: MCP server returned no tools");
                return base;
            }
            let mut mcp_section = format!(
                "\n\n{}. **MCP** (tools from configured MCP server, {} tools): Use when the task matches a tool below. To invoke: reply with exactly one line: MCP: <tool_name> <arguments>. Arguments can be JSON (e.g. MCP: get_weather {{\"location\": \"NYC\"}}) or plain text (e.g. MCP: fetch_url https://example.com).\n\nAvailable MCP tools:\n",
                num,
                tools.len()
            );
            for t in &tools {
                let desc = t.description.as_deref().unwrap_or("(no description)");
                mcp_section.push_str(&format!("- **{}**: {}\n", t.name, desc));
            }
            base + &mcp_section
        }
        Err(e) => {
            info!(
                "Agent router: MCP list_tools failed ({}), omitting MCP from agent list",
                e
            );
            base
        }
    }
}

/// Heuristic: chars to tokens (conservative).
const CHARS_PER_TOKEN: usize = 4;

/// Reserve tokens for model reply and wrapper text.
const RESERVE_TOKENS: u32 = 512;

/// Reduce fetched page content to fit the model context: summarize via Ollama if needed, else truncate.
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
    let body_tokens_est = body.chars().count() / CHARS_PER_TOKEN;

    if body_tokens_est <= max_tokens_for_body as usize {
        return Ok(body.to_string());
    }

    info!(
        "Agent router: page content too large (est. {} tokens), max {} tokens; reducing",
        body_tokens_est, max_tokens_for_body
    );

    let body_truncated_for_request: String = body.chars().take(max_chars).collect();
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
                let fallback: String = body.chars().take(max_chars).collect();
                Ok(format!(
                    "{} (content truncated due to context limit)",
                    fallback
                ))
            } else {
                Ok(summary)
            }
        }
        Err(e) => {
            info!("Agent router: summarization failed ({}), truncating", e);
            let fallback: String = body.chars().take(max_chars).collect();
            Ok(format!(
                "{} (content truncated due to context limit)",
                fallback
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
) -> Result<String, String> {
    use tracing::info;
    let runtime_context = build_agent_runtime_context(chrono::Local::now().fixed_offset());
    info!(
        "Agent: {} ({}) running (model: {:?}, prompt {} chars)",
        agent.name,
        agent.id,
        agent.model,
        agent.combined_prompt.chars().count()
    );
    info!(
        "Agent: {} ({}) runtime date anchor injected",
        agent.name, agent.id
    );
    let mut messages = vec![
        crate::ollama::ChatMessage {
            role: "system".to_string(),
            content: format!("{}\n\n{}", agent.combined_prompt, runtime_context),
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

fn question_explicitly_requests_json(question: &str) -> bool {
    let q = question.to_lowercase();
    q.contains("json")
        || q.contains("machine readable")
        || q.contains("structured output")
        || q.contains("structured data")
}

fn extract_redmine_time_entries_summary_for_reply(tool_result: &str) -> Option<String> {
    let start = tool_result.find("Derived Redmine time-entry summary")?;
    let mut summary = &tool_result[start..];
    for marker in [
        "\n\nUse this data to answer",
        "\n\nUse only this Redmine data to continue or answer",
    ] {
        if let Some(idx) = summary.find(marker) {
            summary = &summary[..idx];
            break;
        }
    }
    let summary = if let Some(idx) = summary.find("\n\nEntry details:\n") {
        &summary[..idx]
    } else {
        summary
    };
    let cleaned = summary.trim().to_string();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

fn extract_redmine_failure_message(text: &str) -> Option<String> {
    let start = text.find("Redmine API failed:")?;
    let rest = text[start + "Redmine API failed:".len()..].trim();
    let first_block = rest.split("\n\n---\n\n").next().unwrap_or(rest).trim();
    let without_instruction = first_block
        .strip_suffix("Answer without this result.")
        .unwrap_or(first_block)
        .trim();
    if without_instruction.is_empty() {
        None
    } else {
        Some(without_instruction.trim_end_matches('.').trim().to_string())
    }
}

fn is_redmine_infrastructure_failure_text(text: &str) -> bool {
    let t = text.to_lowercase();
    t.contains("redmine not configured")
        || t.contains("redmine_url missing")
        || t.contains("redmine_api_key missing")
        || t.contains("invalid url")
        || t.contains("dns")
        || t.contains("failed to lookup address")
        || t.contains("failed to lookup address information")
        || t.contains("name or service not known")
        || t.contains("nodename nor servname provided")
        || t.contains("no address associated with hostname")
        || t.contains("could not resolve host")
        || t.contains("connection refused")
        || t.contains("unreachable")
}

fn format_redmine_time_entries_period(question: &str) -> String {
    let (from, to) = redmine_time_entries_range(question);
    if from == to {
        from
    } else {
        format!("{}..{}", from, to)
    }
}

fn grounded_redmine_time_entries_failure_reply(question: &str, text: &str) -> Option<String> {
    if !is_redmine_time_entries_request(question) {
        return None;
    }

    let failure = extract_redmine_failure_message(text)?;
    if !is_redmine_infrastructure_failure_text(&failure) {
        return None;
    }

    let failure_lower = failure.to_lowercase();
    if failure_lower.contains("no time entries")
        || failure_lower.contains("no worked tickets")
        || failure_lower.contains("tickets were found")
    {
        return None;
    }

    let blocker = if failure_lower.contains("redmine not configured")
        || failure_lower.contains("redmine_url missing")
        || failure_lower.contains("redmine_api_key missing")
    {
        "Redmine is not configured on this machine."
    } else if failure_lower.contains("invalid url") {
        "the configured Redmine URL is invalid."
    } else if failure_lower.contains("dns")
        || failure_lower.contains("failed to lookup address")
        || failure_lower.contains("failed to lookup address information")
        || failure_lower.contains("name or service not known")
        || failure_lower.contains("nodename nor servname provided")
        || failure_lower.contains("no address associated with hostname")
        || failure_lower.contains("could not resolve host")
    {
        "the configured Redmine host could not be resolved."
    } else {
        "the configured Redmine host could not be reached."
    };

    Some(format!(
        "Could not retrieve Redmine time entries for {} because {} No Redmine data was fetched. Fix the Redmine configuration or connectivity, then retry.",
        format_redmine_time_entries_period(question),
        blocker
    ))
}

fn is_grounded_redmine_time_entries_blocked_reply(question: &str, response_content: &str) -> bool {
    if !is_redmine_time_entries_request(question) {
        return false;
    }

    let t = response_content.to_lowercase();
    let mentions_blocked_fetch = t.contains("could not retrieve redmine time entries")
        || (t.contains("redmine api failed")
            && is_redmine_infrastructure_failure_text(response_content));
    let mentions_infra_blocker = is_redmine_infrastructure_failure_text(response_content)
        || t.contains("no redmine data was fetched");
    let invents_empty_result = t.contains("no time entries or tickets were found")
        || t.contains("no time entries were found")
        || t.contains("no worked tickets were found")
        || t.contains("tickets were found for that period");

    mentions_blocked_fetch && mentions_infra_blocker && !invents_empty_result
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

fn is_news_query(question: &str) -> bool {
    let q = question.to_lowercase();
    q.contains("news")
        || q.contains("latest")
        || q.contains("recent")
        || q.contains("headlines")
        || q.contains("current events")
}

fn normalized_search_result_domain(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| {
            u.host_str()
                .map(|s| s.trim_start_matches("www.").to_string())
        })
        .unwrap_or_default()
}

fn is_likely_article_like_result(title: &str, url: &str, snippet: &str) -> bool {
    score_search_result_for_news(title, url, snippet) > 0
}

fn score_search_result_for_news(title: &str, url: &str, snippet: &str) -> i32 {
    let title_l = title.to_lowercase();
    let url_l = url.to_lowercase();
    let snippet_l = snippet.to_lowercase();
    let domain = normalized_search_result_domain(url);
    let path = url::Url::parse(url)
        .ok()
        .map(|u| u.path().trim_matches('/').to_string())
        .unwrap_or_default();
    let path_depth = path.split('/').filter(|s| !s.is_empty()).count();

    let mut score = 0i32;

    if path_depth >= 2 {
        score += 2;
    }
    if path.contains('-') && path.len() > 18 {
        score += 2;
    }
    if snippet.lines().count() <= 3 {
        score += 1;
    }
    if [
        "reuters.com",
        "apnews.com",
        "bbc.com",
        "euronews.com",
        "catalannews.com",
    ]
    .iter()
    .any(|d| domain.ends_with(d))
    {
        score += 2;
    }

    if path.is_empty() || path == "news" || path.ends_with("/news") || path.ends_with("/news/") {
        score -= 3;
    }
    if url_l.contains("/tag/") || url_l.contains("/category/") || url_l.contains("/topics/") {
        score -= 3;
    }
    if url_l.contains("wikipedia.org/wiki/")
        || domain.ends_with("wikipedia.org")
        || domain.ends_with("spain.info")
        || url_l.contains("/destination/")
        || url_l.contains("/destinazione/")
    {
        score -= 5;
    }
    if title_l.contains("top stories")
        || title_l.contains("latest ")
        || title_l.contains("breaking ")
        || title_l.contains("scores")
        || title_l.contains("standings")
        || title_l.contains("rumors")
        || title_l.contains("official channel")
        || title_l.contains("what to see and do")
    {
        score -= 3;
    }
    if snippet_l.contains("view on x")
        || snippet_l.contains("rumor")
        || snippet_l.contains("standings")
        || snippet_l.contains("scores")
        || snippet_l.contains("trendiest")
        || snippet_l.contains("tourist")
    {
        score -= 2;
    }
    if snippet.lines().count() >= 5 {
        score -= 1;
    }
    if domain.contains("newsnow") || domain.contains("transferfeed") {
        score -= 2;
    }

    score
}

fn shape_perplexity_results_for_question(
    question: &str,
    results: Vec<crate::commands::perplexity::PerplexitySearchResult>,
    snippet_max: usize,
) -> (
    Vec<crate::commands::perplexity::PerplexitySearchResult>,
    Vec<String>,
    bool,
) {
    let is_news = is_news_query(question);
    if !is_news {
        let urls = results.iter().map(|r| r.url.clone()).collect();
        return (results, urls, false);
    }

    let mut ranked: Vec<_> = results
        .into_iter()
        .map(|r| {
            let score = score_search_result_for_news(&r.title, &r.url, &r.snippet);
            (score, r)
        })
        .collect();
    ranked.sort_by(|a, b| {
        b.0.cmp(&a.0).then_with(|| {
            let ad =
                a.1.date
                    .as_deref()
                    .or(a.1.last_updated.as_deref())
                    .unwrap_or("");
            let bd =
                b.1.date
                    .as_deref()
                    .or(b.1.last_updated.as_deref())
                    .unwrap_or("");
            bd.cmp(ad)
        })
    });

    let mut kept = Vec::new();
    let mut urls = Vec::new();
    let mut per_domain: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut filtered_any = false;
    for (score, mut result) in ranked {
        let domain = normalized_search_result_domain(&result.url);
        let domain_count = per_domain.get(&domain).copied().unwrap_or(0);
        let article_like = score > 0;
        let allow = article_like || kept.len() < 3;
        if !allow || domain_count >= 2 {
            filtered_any = true;
            continue;
        }
        if result.snippet.chars().count() > snippet_max {
            result.snippet = format!(
                "{}…",
                result.snippet.chars().take(snippet_max).collect::<String>()
            );
        }
        per_domain.insert(domain, domain_count + 1);
        urls.push(result.url.clone());
        kept.push(result);
        if kept.len() >= 6 {
            break;
        }
    }

    (kept, urls, filtered_any)
}

async fn search_perplexity_with_news_fallback(
    question: &str,
    query: &str,
    max_results: u32,
    snippet_max: usize,
) -> Result<
    (
        Vec<crate::commands::perplexity::PerplexitySearchResult>,
        Vec<String>,
        bool,
        Option<String>,
    ),
    String,
> {
    let mut effective_query = query.trim().to_string();
    if is_news_query(question)
        && (effective_query.chars().count() < 18 || effective_query.split_whitespace().count() < 3)
    {
        effective_query = format!("{} latest news sources dates", question.trim());
    }

    let first = crate::commands::perplexity::perplexity_search(
        crate::commands::perplexity::PerplexitySearchRequest {
            query: effective_query.clone(),
            max_results: Some(max_results),
        },
    )
    .await?;

    let is_news = is_news_query(question);
    let mut used_query = None;
    let (mut shaped, mut urls, mut filtered_any) =
        shape_perplexity_results_for_question(question, first.results, snippet_max);

    let need_fallback = is_news
        && !shaped.is_empty()
        && shaped
            .iter()
            .all(|r| !is_likely_article_like_result(&r.title, &r.url, &r.snippet));

    if need_fallback {
        let fallback_query = format!("{} recent article source date", effective_query.trim());
        tracing::info!(
            "Perplexity search: first pass for news returned only hub/landing pages, retrying with refined query: {}",
            fallback_query
        );
        let second = crate::commands::perplexity::perplexity_search(
            crate::commands::perplexity::PerplexitySearchRequest {
                query: fallback_query.clone(),
                max_results: Some(max_results),
            },
        )
        .await?;
        let (fallback_shaped, fallback_urls, fallback_filtered) =
            shape_perplexity_results_for_question(question, second.results, snippet_max);
        let fallback_has_article_like = fallback_shaped
            .iter()
            .any(|r| is_likely_article_like_result(&r.title, &r.url, &r.snippet));
        if fallback_has_article_like {
            shaped = fallback_shaped;
            urls = fallback_urls;
            filtered_any = fallback_filtered;
            used_query = Some(fallback_query);
        }
    }

    Ok((shaped, urls, filtered_any, used_query))
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

/// Minimum number of messages before session compaction triggers.
const COMPACTION_THRESHOLD: usize = 8;

/// Compact a long conversation history into a concise summary using a fast model.
/// Extracts verified facts, successful outcomes, and user intent; drops failures and hallucinations.
/// Also extracts lessons learned (returned separately for memory.md).
async fn compact_conversation_history(
    messages: &[crate::ollama::ChatMessage],
    current_question: &str,
) -> Result<(String, Option<String>), String> {
    use tracing::info;

    let small_model = crate::ollama::models::get_global_catalog()
        .and_then(|c| c.resolve_role("small").map(|m| m.name.clone()));

    let model = small_model.or_else(|| {
        let guard = get_ollama_client().lock().ok()?;
        let client = guard.as_ref()?;
        Some(client.config.model.clone())
    });

    let conversation_text: String = messages
        .iter()
        .map(|m| format!("[{}]: {}", m.role, m.content))
        .collect::<Vec<_>>()
        .join("\n\n");

    let system_prompt = r#"You are a session compactor. Given a conversation between a user and an assistant, produce TWO sections:

## CONTEXT
A concise summary (max 300 words) of ONLY verified facts and successful outcomes **relevant to the user's current question**. Rules:
- If the conversation spans **multiple unrelated topics**, summarize ONLY what is relevant to the **current question** (see below). If the current question is clearly a **new topic** (unrelated to most of the history), output exactly: "Previous context covered different topics; not needed for this request." and keep CONTEXT to that one sentence.
- KEEP: IDs confirmed by API responses (guild IDs, channel IDs, user IDs), successful API calls and their actual results, user preferences and standing instructions, established context the user built up — but only if relevant to the current question.
- DROP: Failed attempts (401 errors, wrong tool usage, timeouts), hallucinated or unverified claims (assistant saying something happened without API confirmation), apologies, suggestions that weren't followed, repeated back-and-forth about the same error.
- If the assistant claimed an action succeeded but there's no API result confirming it, mark it as UNVERIFIED.
- Write as a factual briefing, not a conversation recap.

## LESSONS
Bullet points of important lessons learned (if any). Things like:
- Tools that worked vs. tools that failed
- Correct IDs or endpoints discovered
- User corrections about how things should work
- Mistakes to avoid in future

If no lessons, write "None."

Output ONLY these two sections, nothing else."#;

    let user_msg = format!(
        "The user's current question is: \"{}\"\n\nCompact this conversation:\n\n{}",
        current_question, conversation_text
    );

    let msgs = vec![
        crate::ollama::ChatMessage {
            role: "system".to_string(),
            content: system_prompt.to_string(),
            images: None,
        },
        crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: user_msg,
            images: None,
        },
    ];

    info!(
        "Session compaction: sending {} messages ({} chars) to model {:?}",
        messages.len(),
        conversation_text.len(),
        model
    );

    let response = send_ollama_chat_messages(msgs, model, None).await?;
    let output = response.message.content.trim().to_string();

    let (context, lessons) = parse_compaction_output(&output);
    info!(
        "Session compaction: produced context ({} chars), lessons: {}",
        context.len(),
        lessons.as_deref().unwrap_or("none")
    );

    Ok((context, lessons))
}

/// Parse the compaction output into context and lessons sections.
fn parse_compaction_output(output: &str) -> (String, Option<String>) {
    let lower = output.to_lowercase();
    let context_header = lower.find("## context");
    let lessons_header = lower.find("## lessons");

    let context_body_start = context_header.map(|i| i + "## context".len());
    let lessons_body_start = lessons_header.map(|i| i + "## lessons".len());

    let context = match (context_body_start, lessons_header) {
        (Some(cs), Some(lh)) => output[cs..lh].trim().to_string(),
        (Some(cs), None) => output[cs..].trim().to_string(),
        _ => output.to_string(),
    };

    let lessons = lessons_body_start
        .map(|ls| output[ls..].trim().to_string())
        .filter(|s| !s.is_empty() && s.to_lowercase() != "none." && s.to_lowercase() != "none");

    (context, lessons)
}

/// Minimum messages to compact in the 30-min periodic pass (lower than on-request 8 so we flush more).
const PERIODIC_COMPACTION_MIN_MESSAGES: usize = 4;
/// Sessions with no activity for this long are considered inactive; after compacting they are cleared.
const INACTIVE_THRESHOLD_MINUTES: i64 = 30;

/// Run session compaction for all in-memory sessions that meet the threshold.
/// Writes lessons to global memory; replaces active sessions with summary, clears inactive ones.
/// Call from a 30-minute background loop.
pub async fn run_periodic_session_compaction() {
    use tracing::info;
    let sessions = crate::session_memory::list_sessions();
    let now = chrono::Local::now();
    let inactive_cutoff = now - chrono::Duration::minutes(INACTIVE_THRESHOLD_MINUTES);
    for entry in sessions {
        if entry.message_count < PERIODIC_COMPACTION_MIN_MESSAGES {
            continue;
        }
        let messages: Vec<crate::ollama::ChatMessage> =
            crate::session_memory::get_messages(&entry.source, entry.session_id)
                .into_iter()
                .map(|(role, content)| crate::ollama::ChatMessage {
                    role,
                    content,
                    images: None,
                })
                .collect();
        if messages.len() < PERIODIC_COMPACTION_MIN_MESSAGES {
            continue;
        }
        info!(
            "Periodic session compaction: {} {} ({} messages, last_activity {:?})",
            entry.source,
            entry.session_id,
            messages.len(),
            entry.last_activity
        );
        // task-001: retry once on failure
        let mut actual_question = "Periodic session compaction.".to_string();
        for msg in messages.iter().rev() {
            if msg.role == "user" {
                actual_question = msg.content.clone();
                break;
            }
        }

        let compact_result = compact_conversation_history(&messages, &actual_question).await;
        let compact_result = match compact_result {
            Ok(ok) => Ok(ok),
            Err(e) => {
                tracing::warn!(
                    "Periodic session compaction failed for {} {}: {}, retrying once in 3s",
                    entry.source,
                    entry.session_id,
                    e
                );
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                compact_conversation_history(&messages, &actual_question).await
            }
        };
        match compact_result {
            Ok((context, lessons)) => {
                if let Some(ref lesson_text) = lessons {
                    let memory_path = if entry.source == "discord" {
                        crate::config::Config::memory_file_path_for_discord_channel(
                            entry.session_id,
                        )
                    } else {
                        crate::config::Config::memory_file_path()
                    };
                    for line in lesson_text.lines() {
                        let line = line.trim().trim_start_matches("- ").trim();
                        if !line.is_empty() && line.len() > 5 {
                            let entry_line = format!("- {}\n", line);
                            let _ = append_to_file(&memory_path, &entry_line);
                        }
                    }
                    info!(
                        "Periodic session compaction: wrote lessons to {:?}",
                        memory_path
                    );
                }
                let inactive = entry.last_activity < inactive_cutoff;
                if inactive {
                    crate::session_memory::clear_session(&entry.source, entry.session_id);
                    info!(
                        "Periodic session compaction: cleared inactive session {} {}",
                        entry.source, entry.session_id
                    );
                } else {
                    let compacted = vec![("system".to_string(), context)];
                    crate::session_memory::replace_session(
                        &entry.source,
                        entry.session_id,
                        compacted,
                    );
                    info!(
                        "Periodic session compaction: replaced active session {} {} with summary",
                        entry.source, entry.session_id
                    );
                }
            }
            Err(e) => {
                tracing::warn!(
                    "Periodic session compaction failed for {} {}: {}",
                    entry.source,
                    entry.session_id,
                    e
                );
            }
        }
    }
}

/// Resolve Mastodon credentials: instance URL and access token.
/// Checks env vars (MASTODON_INSTANCE_URL, MASTODON_ACCESS_TOKEN), then ~/.mac-stats/.config.env,
/// then Keychain (mastodon_instance_url, mastodon_access_token).
fn get_mastodon_config() -> Option<(String, String)> {
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
fn append_to_file(path: &std::path::Path, content: &str) -> Result<std::path::PathBuf, String> {
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
/// If `status_tx` is provided (e.g. from Discord), short status messages are sent so the user sees we're still working.
/// If `discord_reply_channel_id` is set (when the request came from Discord), SCHEDULE will store it so the scheduler can post results to that channel (DM or mention channel).
/// When `discord_user_id` and `discord_user_name` are set (from Discord message author), the prompt is prefixed with "You are talking to Discord user **{name}** (user id: {id})."
/// When set, `model_override` and `options_override` apply only to this request (e.g. from Discord "model: llama3" line).
/// Extract a URL from the question for pre-routing (e.g. screenshot). Prefers https?:// then www.
/// Strips trailing punctuation from the URL.
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

/// Extract a numeric ticket/issue ID from text like "ticket #1234", "#1234", "issue 1234", "review redmine 7209".
fn extract_ticket_id(text: &str) -> Option<u64> {
    // Match #NNNN
    if let Some(pos) = text.find('#') {
        let after = &text[pos + 1..];
        let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !digits.is_empty() {
            return digits.parse().ok();
        }
    }
    // Match "ticket NNNN", "issue NNNN", or "redmine NNNN" (e.g. "Review redmine 7209")
    for keyword in &["ticket ", "issue ", "redmine "] {
        if let Some(pos) = text.find(keyword) {
            let after = &text[pos + keyword.len()..];
            let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            if !digits.is_empty() {
                return digits.parse().ok();
            }
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

/// True when the user asked only to review or summarize a Redmine ticket (no update, add comment, or close).
/// Used to avoid injecting PUT hint and to narrow success criteria so verification does not require ticket changes.
fn is_redmine_review_or_summarize_only(question: &str) -> bool {
    let q = question.trim().to_lowercase();
    let has_redmine_ticket = (q.contains("redmine") || q.contains("ticket") || q.contains("issue"))
        && extract_ticket_id(question).is_some();
    let review_or_summarize =
        q.contains("review") || q.contains("summarize") || q.contains("summarise");
    let no_mutate = !q.contains("update")
        && !q.contains("add comment")
        && !q.contains("post a comment")
        && !q.contains("close")
        && !q.contains("resolve")
        && !q.contains("write ");
    has_redmine_ticket && review_or_summarize && no_mutate
}

fn is_redmine_relative_day_request(question: &str) -> bool {
    let q = question.trim().to_lowercase();
    q.contains("today") || q.contains("yesterday") || q.contains("yestaerday")
}

fn is_redmine_yesterday_request(question: &str) -> bool {
    let q = question.trim().to_lowercase();
    q.contains("yesterday") || q.contains("yestaerday")
}

fn is_redmine_time_entries_request(question: &str) -> bool {
    let q = question.trim().to_lowercase();
    let mentions_redmine = q.contains("redmine");
    let mentions_time_entries = q.contains("time entries")
        || q.contains("spent time")
        || q.contains("hours this month")
        || q.contains("hours worked")
        || q.contains("time logs")
        || q.contains("tickets worked")
        || q.contains("worked tickets")
        || (q.contains("worked on") && q.contains("month"))
        || (q.contains("worked on") && is_redmine_relative_day_request(&q))
        || (q.contains("work on") && is_redmine_relative_day_request(&q))
        || (q.contains("work today") && q.contains("ticket"))
        || (q.contains("work yesterday") && q.contains("ticket"))
        || (q.contains("worked") && is_redmine_relative_day_request(&q) && q.contains("ticket"));
    mentions_redmine && mentions_time_entries
}

fn is_agent_unavailable_error(error: &str) -> bool {
    let e = error.to_lowercase();
    e.contains("busy or unavailable")
        || e.contains("timed out")
        || e.contains("timeout")
        || e.contains("503")
}

fn redmine_time_entries_range_for_date(
    question: &str,
    today: chrono::NaiveDate,
) -> (String, String) {
    use chrono::Datelike;

    let q = question.trim().to_lowercase();
    if is_redmine_yesterday_request(&q) {
        let day = today
            .pred_opt()
            .unwrap_or(today)
            .format("%Y-%m-%d")
            .to_string();
        return (day.clone(), day);
    }
    if q.contains("today") {
        let day = today.format("%Y-%m-%d").to_string();
        return (day.clone(), day);
    }
    let from = chrono::NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
        .unwrap_or(today)
        .format("%Y-%m-%d")
        .to_string();
    let (next_year, next_month) = if today.month() == 12 {
        (today.year() + 1, 1)
    } else {
        (today.year(), today.month() + 1)
    };
    let next_month_start =
        chrono::NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap_or(today);
    let to = next_month_start
        .pred_opt()
        .unwrap_or(today)
        .format("%Y-%m-%d")
        .to_string();
    (from, to)
}

fn redmine_time_entries_range(question: &str) -> (String, String) {
    redmine_time_entries_range_for_date(question, chrono::Utc::now().date_naive())
}

fn redmine_request_for_routing<'a>(
    question: &'a str,
    request_for_verification: &'a str,
    is_verification_retry: bool,
) -> &'a str {
    if is_verification_retry
        && (is_redmine_time_entries_request(request_for_verification)
            || is_redmine_review_or_summarize_only(request_for_verification))
    {
        request_for_verification
    } else {
        question
    }
}

fn redmine_direct_fallback_hint(question: &str) -> String {
    if is_redmine_time_entries_request(question) {
        let (from, to) = redmine_time_entries_range(question);
        format!(
            "Use REDMINE_API directly with concrete dates: REDMINE_API: GET /time_entries.json?from={}&to={}&limit=100.",
            from, to
        )
    } else if let Some(id) = extract_ticket_id(question) {
        format!(
            "Use REDMINE_API directly: REDMINE_API: GET /issues/{}.json?include=journals,attachments.",
            id
        )
    } else {
        "Use REDMINE_API directly with the correct concrete endpoint for this request.".to_string()
    }
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
    let generic_news_request = q.contains("news");
    let explicit_named_sources = q.contains("bbc")
        || q.contains("cnn")
        || q.contains("reuters")
        || q.contains("ap ")
        || q.contains("associated press");

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
async fn verify_completion(
    question: &str,
    response_content: &str,
    attachment_paths: &[PathBuf],
    success_criteria: Option<&[String]>,
    page_content_from_browser: Option<&str>,
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
    let verification_tail = if screenshot_requested {
        "Did we fully satisfy the request (including any requested screenshot/file attachment)? Reply YES or NO. If NO, on the next line add one sentence: what's missing."
    } else {
        "Did we fully satisfy the request? Reply YES or NO. If NO, on the next line add one sentence: what's missing."
    };
    let user_text = format!(
        "Original request: {}\n\n{}What we did (summary): {}\n\n{}{}{}",
        question.chars().take(500).collect::<String>(),
        criteria_block,
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
                            "Original request: {}\n\n{}What we did (summary): {}\n\n{}Did we fully satisfy the request? Reply YES or NO. If NO, on the next line add one sentence: what's missing.",
                            question.chars().take(500).collect::<String>(),
                            criteria_block,
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
) -> Pin<Box<dyn Future<Output = Result<OllamaReply, String>> + Send>> {
    let question = question.to_string();
    let mut conversation_history = conversation_history.map(|v| v.to_vec());
    let attachment_images_base64 = attachment_images_base64.map(|v| v.to_vec());
    let discord_intermediate = discord_intermediate.map(|s| s.to_string());
    let is_verification_retry = is_verification_retry;
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
        let request_id: String = format!(
            "{:08x}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
                & 0xFFFF_FFFF
        );

        info!(
            "Agent router [{}]: session start — {}",
            request_id,
            crate::config::Config::version_display()
        );

        // When Discord user asks for screenshots to be sent here, focus on current task only (no prior chat).
        // Skip clearing on verification retry so the model keeps context (e.g. original request, cookie consent retry).
        if !is_verification_retry && discord_reply_channel_id.is_some() {
            let q = question.to_lowercase();
            if q.contains("screenshot")
                && (q.contains("send") || q.contains("here") || q.contains("discord"))
            {
                if conversation_history
                    .as_ref()
                    .map_or(false, |h| !h.is_empty())
                {
                    info!(
                        "Agent router: clearing history for Discord screenshot-send request (focus on current task)"
                    );
                    conversation_history = Some(vec![]);
                }
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
                "Summary of ticket content provided to the user.".to_string(),
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
                info!("Agent router: extracted {} success criteria", c.len());
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
        if !is_verification_retry && !crate::ollama::models::is_cloud_model(&effective_model) {
            if let Some(ref hist) = conversation_history {
                const NEW_TOPIC_MIN_HISTORY: usize = 2;
                if hist.len() >= NEW_TOPIC_MIN_HISTORY {
                    send_status("Checking if new topic…");
                    let summary = summarize_last_turns(hist, 3);
                    match detect_new_topic(question, &summary, &effective_model).await {
                        Ok(true) => {
                            info!(
                                "Agent router: new-topic check returned NEW_TOPIC, using no prior context"
                            );
                            is_new_topic = true;
                        }
                        Ok(false) => {}
                        Err(e) => {
                            tracing::debug!(
                                "Agent router: new-topic check failed (keeping history): {}",
                                e
                            );
                        }
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

        let conversation_history: Vec<crate::ollama::ChatMessage> = if raw_history.len()
            >= COMPACTION_THRESHOLD
        {
            send_status("Compacting session memory…");
            info!(
                "Session compaction: {} messages exceed threshold ({}), compacting",
                raw_history.len(),
                COMPACTION_THRESHOLD
            );
            match compact_conversation_history(&raw_history, question).await {
                Ok((context, lessons)) => {
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
                        info!("Session compaction: wrote lessons to {:?}", memory_path);
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
                    let msg = if e.to_string().to_lowercase().contains("unauthorized")
                        || e.to_string().contains("401")
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
                        if ticket_id.is_some()
                            && (redmine_request_lower.contains("ticket")
                                || redmine_request_lower.contains("issue")
                                || redmine_request_lower.contains("redmine"))
                            && !wants_update
                        {
                            let id = ticket_id.unwrap();
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
                    if ticket_id.is_some()
                        && (redmine_request_lower.contains("ticket")
                            || redmine_request_lower.contains("issue")
                            || redmine_request_lower.contains("redmine"))
                        && !wants_update
                    {
                        let id = ticket_id.unwrap();
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
                    "{}Additional instructions from skill:\n\n{}\n\n---\n\n{}\n\n{}",
                    discord_user_context, skill, planning_prompt, agent_descriptions
                ),
                None => format!(
                    "{}{}{}\n\n{}",
                    router_soul, discord_user_context, planning_prompt, agent_descriptions
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
            planning_messages.push(crate::ollama::ChatMessage {
                role: "user".to_string(),
                content: format!(
                    "Current user question: {}{}\n\nReply with RECOMMEND: your plan.",
                    question_for_plan_and_exec, model_hint
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

        let normalized_recommendation = normalize_inline_tool_sequences(&recommendation);

        // Fast path: if the recommendation already contains a parseable tool call, execute it
        // directly instead of asking Ollama a second time to regurgitate the same tool line.
        let direct_tool = parse_tool_from_response(&recommendation);
        let (mut messages, mut response_content) = if let Some((ref tool, ref arg)) = direct_tool {
            info!(
                "Agent router: plan contains direct tool call {}:{} — skipping execution Ollama call",
                tool,
                crate::logging::ellipse(arg, 60)
            );
            let memory_block = load_memory_block_for_request(discord_reply_channel_id);
            let execution_system_content = match &skill_content {
                Some(skill) => format!(
                    "{}Additional instructions from skill:\n\n{}\n\n---\n\n{}{}{}{}{}",
                    discord_user_context,
                    skill,
                    execution_prompt,
                    metrics_for_system,
                    discord_screenshot_reminder,
                    redmine_howto_reminder,
                    model_identity
                ),
                None => format!(
                    "{}{}{}{}{}{}{}{}",
                    router_soul,
                    memory_block,
                    discord_user_context,
                    execution_prompt,
                    metrics_for_system,
                    discord_screenshot_reminder,
                    redmine_howto_reminder,
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
            // Preserve normalized multi-tool chains so the executor can run them step by step.
            let synthetic = if normalized_recommendation.contains('\n') {
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
            let memory_block = load_memory_block_for_request(discord_reply_channel_id);
            let execution_system_content = match &skill_content {
                Some(skill) => format!(
                    "{}Additional instructions from skill:\n\n{}\n\n---\n\n{}{}{}{}\n\nYour plan: {}{}",
                    discord_user_context,
                    skill,
                    execution_prompt,
                    metrics_for_system,
                    discord_screenshot_reminder,
                    redmine_howto_reminder,
                    recommendation,
                    model_identity
                ),
                None => format!(
                    "{}{}{}{}{}{}{}\n\nYour plan: {}{}",
                    router_soul,
                    memory_block,
                    discord_user_context,
                    execution_prompt,
                    metrics_for_system,
                    discord_screenshot_reminder,
                    redmine_howto_reminder,
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
                    if browser_tool_count >= MAX_BROWSER_TOOLS_PER_RUN {
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
                        let url_arg = arg.trim().to_string();
                        send_status(&format!("🧭 Navigating to {}…", url_arg));
                        if url_arg.is_empty() {
                            "BROWSER_NAVIGATE requires a URL (e.g. BROWSER_NAVIGATE: https://www.example.com). Please try again with a URL.".to_string()
                        } else {
                            info!(
                                "Agent router [{}]: BROWSER_NAVIGATE: URL sent to CDP: {}",
                                request_id, url_arg
                            );
                            match tokio::task::spawn_blocking({
                                let u = url_arg.clone();
                                move || crate::browser_agent::navigate_and_get_state(&u)
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
                                        move || crate::browser_agent::navigate_and_get_state(&u)
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
                            Ok(Err(_cdp_err)) => {
                                match tokio::task::spawn_blocking(move || crate::browser_agent::click_http(idx)).await {
                                    Ok(Ok(state_str)) => state_str,
                                    Ok(Err(e)) => format!("BROWSER_CLICK failed: {}", e),
                                    Err(e) => format!("BROWSER_CLICK task error: {}", e),
                                }
                            }
                            Err(e) => format!("BROWSER_CLICK task error: {}", e),
                        }
                    }
                    None => "BROWSER_CLICK requires a numeric index (e.g. BROWSER_CLICK: 3). Use the index from the Current page Elements list.".to_string(),
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
                            Ok(Err(_cdp_err)) => {
                                match tokio::task::spawn_blocking(move || crate::browser_agent::input_http(idx, &text)).await {
                                    Ok(Ok(state_str)) => state_str,
                                    Ok(Err(e)) => format!("BROWSER_INPUT failed: {}", e),
                                    Err(e) => format!("BROWSER_INPUT task error: {}", e),
                                }
                            }
                            Err(e) => format!("BROWSER_INPUT task error: {}", e),
                        }
                    }
                    None => "BROWSER_INPUT requires a numeric index and text (e.g. BROWSER_INPUT: 4 search query). Use the index from the Current page Elements list.".to_string(),
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
                                    result_text.push_str(
                                        "\n\nFor news requests, prefer concrete article/report results over homepages, hub pages, standings pages, rumor indexes, or official landing pages. If a result looks like a hub, use it only as fallback and say so clearly.",
                                    );
                                    if let Some(ref refined_query) = refined_query_used {
                                        result_text.push_str(&format!(
                                            "\nRefined search query used to find article-like results: {}.",
                                            refined_query
                                        ));
                                    }
                                    if filtered_any {
                                        result_text.push_str(
                                            "\nFiltered to higher-signal results and limited repeated domains where possible.",
                                        );
                                    }
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
                                    format!(
                                        "The orchestrator already replied above. Reply with a one-sentence summary for the user and **DONE: success** or **DONE: no**. Do not output AGENT: orchestrator again."
                                    )
                                } else {
                                    let mut user_msg: String = if task_message.is_empty() {
                                        question.to_string()
                                    } else {
                                        task_message.to_string()
                                    };
                                    // When invoking discord-expert from Discord, fetch guild/channel metadata via API and inject so the agent has current context.
                                    let is_discord_expert =
                                        agent.slug.as_deref().map_or(false, |s| {
                                            s.eq_ignore_ascii_case("discord-expert")
                                        }) || agent.id == "004";
                                    let is_redmine_agent = agent
                                        .slug
                                        .as_deref()
                                        .map_or(false, |s| s.eq_ignore_ascii_case("redmine"))
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
                                        // Ask Ollama to fix the command
                                        let allowed =
                                            crate::commands::run_cmd::allowed_commands().join(", ");
                                        let fix_prompt = format!(
                                            "The command `{}` failed with error:\n{}\n\nReply with ONLY the corrected command on a single line, in this exact format:\nRUN_CMD: <corrected command>\n\nAllowed commands: {}. Paths must be under ~/.mac-stats.",
                                            current_cmd, e, allowed
                                        );
                                        let fix_messages = vec![crate::ollama::ChatMessage {
                                            role: "user".to_string(),
                                            content: fix_prompt,
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
                                                if let Some((_, new_arg)) =
                                                    parse_tool_from_response(&fixed)
                                                {
                                                    current_cmd = new_arg;
                                                } else {
                                                    last_output = format!(
                                                        "RUN_CMD failed: {}. AI could not produce a corrected command. Answer the user's question only; do not include Redmine or other unrelated tool output.",
                                                        e
                                                    );
                                                    break;
                                                }
                                            }
                                            Err(ollama_err) => {
                                                info!(
                                                    "Agent router: RUN_CMD fix Ollama call failed: {}",
                                                    ollama_err
                                                );
                                                last_output = format!(
                                                    "RUN_CMD failed: {}. Answer the user's question only; do not include Redmine or other unrelated tool output.",
                                                    e
                                                );
                                                break;
                                            }
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
                                        } else {
                                            if path.contains("time_entries") {
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
                    _ => continue,
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
                if is_browser_error || is_multi_tool_run_cmd_error {
                    info!(
                        "Agent router: {} returned an error, aborting remaining tools in this turn",
                        tool
                    );
                    break;
                }
            }

            let user_message = tool_results.join("\n\n---\n\n");
            if let Some(blocked_reply) =
                grounded_redmine_time_entries_failure_reply(&request_for_verification, &user_message)
            {
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
                        || reason_lower.contains("credible");
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
                    )
                    .await;
                }
                let reason_preview = reason
                    .as_deref()
                    .map(|r| r.chars().take(80).collect::<String>())
                    .unwrap_or_default();
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

/// Result of parsing a SCHEDULE argument: either a recurring cron or a one-shot "at" datetime.
#[derive(Debug)]
enum ScheduleParseResult {
    Cron { cron_str: String, task: String },
    At { at_str: String, task: String },
}

/// Parse SCHEDULE argument. Supports:
/// - "every N minutes <task>"
/// - "at <datetime> <task>" (one-shot; datetime ISO or YYYY-MM-DD HH:MM)
/// - "<cron expression> <task>" (5- or 6-field; 5-field gets "0 " prepended)
fn parse_schedule_arg(arg: &str) -> Result<ScheduleParseResult, String> {
    let trimmed = arg.trim();
    let lower = trimmed.to_lowercase();
    let rest = lower.trim_start();

    // 1. "every N minutes <task>"
    if let Some(after_every) = rest.strip_prefix("every ") {
        let mut n_str = String::new();
        for c in after_every.chars() {
            if c.is_ascii_digit() {
                n_str.push(c);
            } else {
                break;
            }
        }
        let remainder = after_every[n_str.len()..].trim_start();
        if n_str.is_empty() {
            return Err("expected a number after 'every' (e.g. every 5 minutes)".to_string());
        }
        let n: u64 = n_str
            .parse()
            .map_err(|_| "expected integer after 'every'".to_string())?;
        if n == 0 {
            return Err("interval must be at least 1 minute".to_string());
        }
        if !remainder.to_lowercase().starts_with("minute") {
            return Err("expected 'minutes' after the number (e.g. every 5 minutes)".to_string());
        }
        let cron_str = format!("0 */{} * * * *", n);
        let task = trimmed.to_string();
        return Ok(ScheduleParseResult::Cron { cron_str, task });
    }

    // 2. "at <datetime> <task>" (one-shot)
    if let Some(after_at) = rest.strip_prefix("at ") {
        let after_at = after_at.trim_start();
        let tokens: Vec<&str> = after_at.split_whitespace().collect();
        if tokens.is_empty() {
            return Err(
                "at requires a datetime and task (e.g. at 2025-02-09T05:00:00 Remind me)"
                    .to_string(),
            );
        }
        // Try first token as ISO (2025-02-09T05:00:00)
        if tokens[0].contains('T') {
            if let Ok(dt) = parse_at_datetime(tokens[0]) {
                let task = tokens[1..].join(" ").trim().to_string();
                if task.is_empty() {
                    return Err("at requires a task description after the datetime".to_string());
                }
                return Ok(ScheduleParseResult::At { at_str: dt, task });
            }
        }
        // Try first two tokens as "YYYY-MM-DD HH:MM" or "YYYY-MM-DD HH:MM:SS"
        if tokens.len() >= 2 {
            let combined = format!("{} {}", tokens[0], tokens[1]);
            if let Ok(dt) = parse_at_datetime(&combined) {
                let task = tokens[2..].join(" ").trim().to_string();
                if task.is_empty() {
                    return Err("at requires a task description after the datetime".to_string());
                }
                return Ok(ScheduleParseResult::At { at_str: dt, task });
            }
        }
        return Err(
            "invalid at datetime: use YYYY-MM-DDTHH:MM:SS or YYYY-MM-DD HH:MM (local time)"
                .to_string(),
        );
    }

    // 3. Raw cron: first 5 or 6 space-separated tokens, then task
    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    for &n in &[6, 5] {
        if tokens.len() >= n {
            let cron_part: String = if n == 5 {
                format!("0 {}", tokens[..5].join(" "))
            } else {
                tokens[..6].join(" ")
            };
            if cron::Schedule::from_str(&cron_part).is_ok() {
                let task = tokens[n..].join(" ").trim().to_string();
                return Ok(ScheduleParseResult::Cron {
                    cron_str: cron_part,
                    task,
                });
            }
        }
    }

    Err("expected 'every N minutes <task>', 'at <datetime> <task>', or '<cron> <task>' (see SCHEDULE cron examples)".to_string())
}

/// Parse datetime for "at" one-shot. Returns ISO string for storage (local, no Z).
/// Rejects past times.
fn parse_at_datetime(s: &str) -> Result<String, String> {
    use chrono::{Local, TimeZone};
    let s = s.trim();
    let dt = chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Local))
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").map(|n| {
                Local
                    .from_local_datetime(&n)
                    .single()
                    .unwrap_or_else(|| n.and_utc().with_timezone(&Local))
            })
        })
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").map(|n| {
                Local
                    .from_local_datetime(&n)
                    .single()
                    .unwrap_or_else(|| n.and_utc().with_timezone(&Local))
            })
        })
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M").map(|n| {
                Local
                    .from_local_datetime(&n)
                    .single()
                    .unwrap_or_else(|| n.and_utc().with_timezone(&Local))
            })
        })
        .map_err(|e| {
            format!(
                "invalid datetime: {} (use YYYY-MM-DDTHH:MM:SS or YYYY-MM-DD HH:MM)",
                e
            )
        })?;
    let now = Local::now();
    if dt < now {
        return Err("datetime must be in the future".to_string());
    }
    Ok(dt.format("%Y-%m-%dT%H:%M:%S").to_string())
}

/// True if the trimmed line looks like the start of a tool call (e.g. "TASK_APPEND:", "RUN_CMD:").
fn line_starts_with_tool_prefix(line: &str) -> bool {
    let line = line.trim();
    if line.eq_ignore_ascii_case("TASK_LIST") || line.eq_ignore_ascii_case("LIST_SCHEDULES") {
        return true;
    }
    let mut search = line;
    loop {
        let upper = search.to_uppercase();
        if upper.starts_with("RECOMMEND: ") {
            search = search[11..].trim();
        } else if search.len() >= 2 && search.as_bytes()[0].is_ascii_digit() {
            let rest = search.trim_start_matches(|c: char| c.is_ascii_digit());
            if rest.starts_with(". ") || rest.starts_with(") ") || rest.starts_with(": ") {
                search = rest[2..].trim();
            } else {
                break;
            }
        } else if search.starts_with("- ") || search.starts_with("* ") {
            search = search[2..].trim();
        } else {
            break;
        }
    }
    for prefix in TOOL_LINE_PREFIXES {
        if search.to_uppercase().starts_with(prefix) {
            return true;
        }
    }
    false
}

/// Parse one tool starting at the given line index. Returns (tool_name, argument) and the next line index to scan.
fn parse_one_tool_at_line(lines: &[&str], line_index: usize) -> Option<((String, String), usize)> {
    let prefixes = [
        "FETCH_URL:",
        "BRAVE_SEARCH:",
        "BROWSER_SCREENSHOT:",
        "BROWSER_NAVIGATE:",
        "BROWSER_CLICK:",
        "BROWSER_INPUT:",
        "BROWSER_SCROLL:",
        "BROWSER_EXTRACT:",
        "BROWSER_SEARCH_PAGE:",
        "PERPLEXITY_SEARCH:",
        "RUN_JS:",
        "SKILL:",
        "AGENT:",
        "RUN_CMD:",
        "SCHEDULE:",
        "SCHEDULER:",
        "REMOVE_SCHEDULE:",
        "LIST_SCHEDULES:",
        "TASK_LIST:",
        "TASK_SHOW:",
        "TASK_APPEND:",
        "TASK_STATUS:",
        "TASK_CREATE:",
        "TASK_ASSIGN:",
        "TASK_SLEEP:",
        "OLLAMA_API:",
        "PYTHON_SCRIPT:",
        "MCP:",
        "DISCORD_API:",
        "CURSOR_AGENT:",
        "REDMINE_API:",
        "MEMORY_APPEND:",
        "MASTODON_POST:",
        "DONE:",
    ];
    let line = lines.get(line_index)?.trim();
    if line.eq_ignore_ascii_case("TASK_LIST") {
        return Some((("TASK_LIST".to_string(), String::new()), line_index + 1));
    }
    if line.eq_ignore_ascii_case("LIST_SCHEDULES") {
        return Some((
            ("LIST_SCHEDULES".to_string(), String::new()),
            line_index + 1,
        ));
    }
    // Lenient: model sometimes replies with bare tool name (no colon), e.g. "BROWSER_EXTRACT" or "BROWSER_SCREENSHOT"
    if line.eq_ignore_ascii_case("BROWSER_EXTRACT") {
        return Some((
            ("BROWSER_EXTRACT".to_string(), String::new()),
            line_index + 1,
        ));
    }
    if line.eq_ignore_ascii_case("BROWSER_SCREENSHOT") {
        return Some((
            ("BROWSER_SCREENSHOT".to_string(), "current".to_string()),
            line_index + 1,
        ));
    }
    let mut search = line;
    loop {
        let upper = search.to_uppercase();
        if upper.starts_with("RECOMMEND: ") {
            search = search[11..].trim();
        } else if search.len() >= 2 && search.as_bytes()[0].is_ascii_digit() {
            let rest = search.trim_start_matches(|c: char| c.is_ascii_digit());
            if rest.starts_with(". ") || rest.starts_with(") ") || rest.starts_with(": ") {
                search = rest[2..].trim();
            } else {
                break;
            }
        } else if search.starts_with("- ") || search.starts_with("* ") {
            search = search[2..].trim();
        } else {
            break;
        }
    }
    for prefix in prefixes {
        let tool_name = prefix.trim_end_matches(':');
        let search_upper = search.to_uppercase();
        let bare_prefix = format!("{} ", tool_name);
        if search_upper.starts_with(prefix) || search_upper.starts_with(&bare_prefix) {
            let arg_start = if search_upper.starts_with(prefix) {
                prefix.len()
            } else {
                tool_name.len()
            };
            let mut arg = search[arg_start..].trim().to_string();
            if arg.is_empty()
                && prefix != "TASK_LIST:"
                && prefix != "TASK_SHOW:"
                && prefix != "LIST_SCHEDULES:"
                && prefix != "BROWSER_EXTRACT:"
                && prefix != "BROWSER_SCREENSHOT:"
                && prefix != "DONE:"
            {
                continue;
            }
            let tool_name = if tool_name.eq_ignore_ascii_case("SCHEDULER") {
                "SCHEDULE".to_string()
            } else {
                tool_name.to_string()
            };
            let next_line = if tool_name == "TASK_APPEND" || tool_name == "TASK_CREATE" {
                line_index
                    + 1
                    + lines[line_index + 1..]
                        .iter()
                        .take_while(|l| !line_starts_with_tool_prefix(l))
                        .count()
            } else {
                line_index + 1
            };
            if tool_name == "FETCH_URL"
                || tool_name == "BRAVE_SEARCH"
                || tool_name == "BROWSER_SCREENSHOT"
                || tool_name == "BROWSER_NAVIGATE"
                || tool_name == "BROWSER_SEARCH_PAGE"
                || tool_name == "PERPLEXITY_SEARCH"
            {
                if let Some(idx) = arg.find(';') {
                    arg = arg[..idx].trim().to_string();
                }
            }
            if tool_name == "FETCH_URL"
                || tool_name == "BROWSER_SCREENSHOT"
                || tool_name == "BROWSER_NAVIGATE"
            {
                if let Some(first_space) = arg.find(' ') {
                    arg = arg[..first_space].trim().to_string();
                }
                arg = arg.trim_end_matches(['.', ',', ';', ':']).to_string();
            }
            if tool_name == "BROWSER_SEARCH_PAGE" {
                arg = arg.trim_end_matches(['.', ',', ';', ':']).to_string();
            }
            if tool_name != "TASK_APPEND" && tool_name != "TASK_CREATE" {
                if let Some(pos) = arg.find(|c: char| c.is_ascii_digit()).and_then(|_| {
                    let bytes = arg.as_bytes();
                    for i in 1..bytes.len().saturating_sub(2) {
                        if bytes[i].is_ascii_digit()
                            && bytes[i - 1] == b' '
                            && (bytes.get(i + 1) == Some(&b'.') || bytes.get(i + 1) == Some(&b')'))
                            && bytes.get(i + 2) == Some(&b' ')
                        {
                            return Some(i - 1);
                        }
                    }
                    None
                }) {
                    arg = arg[..pos].trim().to_string();
                }
            }
            if tool_name == "PERPLEXITY_SEARCH" || tool_name == "BRAVE_SEARCH" {
                arg = truncate_search_query_arg(&arg);
            }
            if !arg.is_empty()
                || tool_name == "TASK_LIST"
                || tool_name == "TASK_SHOW"
                || tool_name == "LIST_SCHEDULES"
                || tool_name == "BROWSER_EXTRACT"
                || tool_name == "BROWSER_SCREENSHOT"
                || (tool_name == "TASK_SLEEP" && !arg.is_empty())
            {
                return Some(((tool_name, arg), next_line));
            }
        }
    }
    None
}

/// True if the question explicitly asks for a visible browser (e.g. "show me the browser", "visible", "I want to see").
fn wants_visible_browser(question: &str) -> bool {
    let q = question.to_lowercase();
    q.contains("visible")
        || q.contains("show me the browser")
        || q.contains("show me a browser")
        || q.contains("i want to see the browser")
        || q.contains("open a window")
}

/// Build the short Perplexity result summary for verbose Discord (respects max_chars).
pub(crate) fn build_perplexity_verbose_summary(
    n: usize,
    titles: String,
    max_chars: usize,
) -> String {
    if n == 0 {
        "Perplexity: 0 results.".to_string()
    } else if titles.trim().is_empty() {
        format!("Perplexity: {} result(s) received.", n)
    } else {
        let raw = format!("Perplexity: {} result(s) — {}", n, titles.trim());
        if raw.chars().count() > max_chars {
            format!(
                "{}…",
                raw.chars()
                    .take(max_chars - 1)
                    .collect::<String>()
                    .trim_end()
            )
        } else {
            raw
        }
    }
}

/// For PERPLEXITY_SEARCH/BRAVE_SEARCH, the recommendation often contains the whole plan after the query (e.g. "spanish newspapers then BROWSER_NAVIGATE: ..."). Truncate to just the search query so the API gets a clean query.
fn truncate_search_query_arg(arg: &str) -> String {
    let arg = arg.trim();
    let arg_lower = arg.to_lowercase();
    let earliest = [
        " then ",
        " extract ",
        " → ",
        "\n",
        " browser_navigate",
        " browser_navigate:",
        " browser_screenshot:",
        " and then ",
    ]
    .iter()
    .filter_map(|sep| arg_lower.find(sep).map(|i| i))
    .min();
    let base = earliest.map(|i| arg[..i].trim()).unwrap_or(arg);
    base.chars()
        .take(150)
        .collect::<String>()
        .trim()
        .to_string()
}

fn normalize_inline_tool_sequences(content: &str) -> String {
    static INLINE_TOOL_CHAIN_RE: OnceLock<regex::Regex> = OnceLock::new();
    let re = INLINE_TOOL_CHAIN_RE.get_or_init(|| {
        regex::Regex::new(
            r"(?i)(?:\b(?:then|and then|after that|afterward|afterwards|next|finally)\b|;|->)\s+(FETCH_URL|BRAVE_SEARCH|BROWSER_SCREENSHOT|BROWSER_NAVIGATE|BROWSER_CLICK|BROWSER_INPUT|BROWSER_SCROLL|BROWSER_EXTRACT|BROWSER_SEARCH_PAGE|PERPLEXITY_SEARCH|RUN_JS|SKILL|AGENT|RUN_CMD|SCHEDULE|SCHEDULER|REMOVE_SCHEDULE|LIST_SCHEDULES|TASK_LIST|TASK_SHOW|TASK_APPEND|TASK_STATUS|TASK_CREATE|TASK_ASSIGN|TASK_SLEEP|OLLAMA_API|MCP|PYTHON_SCRIPT|DISCORD_API|CURSOR_AGENT|REDMINE_API|MEMORY_APPEND|MASTODON_POST|DONE)(?::)?\s+",
        )
        .expect("inline tool chain regex must compile")
    });
    re.replace_all(content, |caps: &regex::Captures| {
        format!("\n{}: ", &caps[1].to_ascii_uppercase())
    })
    .into_owned()
}

/// Parse one of FETCH_URL:, BRAVE_SEARCH:, RUN_JS:, SCHEDULE:/SCHEDULER:, MCP:, PYTHON_SCRIPT: from assistant content (first match only).
fn parse_tool_from_response(content: &str) -> Option<(String, String)> {
    let normalized = normalize_inline_tool_sequences(content);
    let lines: Vec<&str> = normalized.lines().collect();
    parse_one_tool_at_line(&lines, 0).map(|(pair, _)| pair)
}

/// Max browser tools (NAVIGATE, CLICK, INPUT, SCROLL, EXTRACT, SEARCH_PAGE, SCREENSHOT) per run. Prevents runaway loops.
const MAX_BROWSER_TOOLS_PER_RUN: u32 = 15;

/// Parse all tool invocations from a response (e.g. BROWSER_CLICK and BROWSER_SCREENSHOT on consecutive lines).
/// Returns up to 5 per response so one model reply can trigger multiple actions (fixes screenshot-after-click).
const MAX_TOOLS_PER_RESPONSE: usize = 5;

fn parse_all_tools_from_response(content: &str) -> Vec<(String, String)> {
    let normalized = normalize_inline_tool_sequences(content);
    let lines: Vec<&str> = normalized.lines().collect();
    let mut out = Vec::with_capacity(MAX_TOOLS_PER_RESPONSE);
    let mut idx = 0;
    while idx < lines.len() && out.len() < MAX_TOOLS_PER_RESPONSE {
        if let Some(((tool, arg), next)) = parse_one_tool_at_line(&lines, idx) {
            out.push((tool, arg));
            idx = next;
        } else {
            idx += 1;
        }
    }
    out
}

/// Tool line prefixes that indicate start of another tool (used to stop script body extraction).
const TOOL_LINE_PREFIXES: &[&str] = &[
    "FETCH_URL:",
    "BRAVE_SEARCH:",
    "BROWSER_SCREENSHOT:",
    "BROWSER_NAVIGATE:",
    "BROWSER_CLICK:",
    "BROWSER_INPUT:",
    "BROWSER_SCROLL:",
    "BROWSER_EXTRACT:",
    "BROWSER_SEARCH_PAGE:",
    "PERPLEXITY_SEARCH:",
    "RUN_JS:",
    "SKILL:",
    "AGENT:",
    "RUN_CMD:",
    "SCHEDULE:",
    "SCHEDULER:",
    "REMOVE_SCHEDULE:",
    "LIST_SCHEDULES:",
    "TASK_LIST:",
    "TASK_SHOW:",
    "TASK_APPEND:",
    "TASK_STATUS:",
    "TASK_CREATE:",
    "TASK_ASSIGN:",
    "TASK_SLEEP:",
    "OLLAMA_API:",
    "MCP:",
    "PYTHON_SCRIPT:",
    "DISCORD_API:",
    "CURSOR_AGENT:",
    "REDMINE_API:",
    "MEMORY_APPEND:",
    "DONE:",
];

/// Parse PYTHON_SCRIPT from full response: (id, topic, script_body).
/// Script body is taken from a ```python ... ``` block, or from all lines after PYTHON_SCRIPT: until another tool line or end.
fn parse_python_script_from_response(content: &str) -> Option<(String, String, String)> {
    let prefix = "PYTHON_SCRIPT:";
    let mut id_topic_line: Option<&str> = None;
    let mut python_line_index = None::<usize>;
    for (idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        let search = if trimmed.to_uppercase().starts_with("RECOMMEND: ") {
            trimmed[11..].trim()
        } else {
            trimmed
        };
        if search.to_uppercase().starts_with(prefix) {
            id_topic_line = Some(search[prefix.len()..].trim());
            python_line_index = Some(idx);
            break;
        }
    }
    let id_topic_line = id_topic_line?;
    let parts: Vec<&str> = id_topic_line.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }
    let id = parts[0].to_string();
    let topic = parts[1].to_string();

    // Extract script body: first try ```python ... ```
    if let Some(start) = content.find("```python") {
        let after_marker = &content[start + 9..];
        if let Some(close) = after_marker.find("```") {
            let body = after_marker[..close].trim().to_string();
            if !body.is_empty() {
                return Some((id, topic, body));
            }
        }
    }
    // Also try ``` (no "python") for flexibility
    if let Some(start) = content.find("```") {
        let after_newline = content[start + 3..]
            .find('\n')
            .map(|i| start + 3 + i + 1)
            .unwrap_or(start + 3);
        let rest = &content[after_newline..];
        if let Some(close) = rest.find("```") {
            let body = rest[..close].trim().to_string();
            if !body.is_empty() {
                return Some((id, topic, body));
            }
        }
    }

    // Else: lines after PYTHON_SCRIPT: until another tool line or end
    let python_line_index = python_line_index.unwrap_or(0);
    let lines: Vec<&str> = content.lines().collect();
    let mut body_lines = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if i <= python_line_index {
            continue;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            body_lines.push(trimmed);
            continue;
        }
        let is_other_tool = TOOL_LINE_PREFIXES
            .iter()
            .any(|p| trimmed.to_uppercase().starts_with(p));
        if is_other_tool {
            break;
        }
        body_lines.push(trimmed);
    }
    let body = body_lines.join("\n").trim().to_string();
    if body.is_empty() {
        return None;
    }
    Some((id, topic, body))
}

/// List available Ollama models (async, non-blocking)
#[tauri::command]
pub async fn list_ollama_models() -> Result<Vec<String>, String> {
    use serde_json;
    use tracing::{debug, info};

    info!("Ollama: Listing available models...");

    // Clone client data to avoid holding lock across await
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

    // Create a temporary client for this request (non-blocking)
    let temp_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let url = format!("{}/api/tags", endpoint);
    info!("Ollama: Using endpoint: {}", url);
    let mut request = temp_client.get(&url);

    // Add API key if configured
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

    // Log raw response JSON
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

/// Generate embeddings (POST /api/embed). Input can be a single string or array of strings.
#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaEmbedOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncate: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<u32>,
}

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

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaJsExecutionLog {
    pub code: String,
    pub result: String,
    pub result_type: String,
    pub is_undefined: bool,
    pub success: bool,
    pub error_name: Option<String>,
    pub error_message: Option<String>,
    pub error_stack: Option<String>,
}

/// Log JavaScript code execution from Ollama responses
#[tauri::command]
pub fn log_ollama_js_execution(log: OllamaJsExecutionLog) -> Result<(), String> {
    use tracing::{error, info, warn};

    info!("Ollama JS Execution: ========================================");
    info!("Ollama JS Execution: JavaScript code block detected and executed");
    info!("Ollama JS Execution: Code:\n{}", log.code);
    info!("Ollama JS Execution: Success: {}", log.success);
    info!("Ollama JS Execution: Result type: {}", log.result_type);
    info!("Ollama JS Execution: ========== EXECUTION RESULT ==========");
    info!("Ollama JS Execution: Result: {}", log.result);
    info!("Ollama JS Execution: ========================================");
    info!("Ollama JS Execution: Is undefined: {}", log.is_undefined);

    if log.is_undefined {
        warn!("Ollama JS Execution: WARNING - Result is undefined");
        warn!("Ollama JS Execution: Executed code was:\n{}", log.code);
        warn!("Ollama JS Execution: Possible reasons for undefined:");
        warn!("Ollama JS Execution:   - Code has no return statement");
        warn!("Ollama JS Execution:   - Code explicitly returns undefined");
        warn!("Ollama JS Execution:   - Code throws an error (check error details below)");
        warn!("Ollama JS Execution:   - Code is an async function that doesn't return a value");
    }

    if !log.success {
        error!("Ollama JS Execution: ERROR - Code execution failed");
        if let Some(ref error_name) = log.error_name {
            error!("Ollama JS Execution: Error name: {}", error_name);
        }
        if let Some(ref error_message) = log.error_message {
            error!("Ollama JS Execution: Error message: {}", error_message);
        }
        if let Some(ref error_stack) = log.error_stack {
            error!("Ollama JS Execution: Error stack:\n{}", error_stack);
        }
    }

    info!("Ollama JS Execution: ========================================");

    Ok(())
}

/// Log when checking for JavaScript code in Ollama response
#[tauri::command]
pub fn log_ollama_js_check(response_content: String, response_length: usize) -> Result<(), String> {
    use tracing::info;

    info!("Ollama JS Execution: Checking response for JavaScript code blocks");
    info!(
        "Ollama JS Execution: Response length: {} characters",
        response_length
    );
    const LOG_MAX: usize = 500;
    let verbosity = crate::logging::VERBOSITY.load(Ordering::Relaxed);
    if verbosity >= 2 {
        info!(
            "Ollama JS Execution: Response content:\n{}",
            response_content
        );
    } else {
        info!(
            "Ollama JS Execution: Response content:\n{}",
            crate::logging::ellipse(&response_content, LOG_MAX)
        );
    }

    Ok(())
}

/// Log JavaScript code block extraction results
#[tauri::command]
pub fn log_ollama_js_extraction(found_blocks: usize, blocks: Vec<String>) -> Result<(), String> {
    use tracing::info;

    info!(
        "Ollama JS Execution: Extraction complete - found {} code block(s)",
        found_blocks
    );
    for (i, block) in blocks.iter().enumerate() {
        info!("Ollama JS Execution: Extracted block {}:\n{}", i + 1, block);
    }

    Ok(())
}

/// Log when no JavaScript code blocks are found
#[tauri::command]
pub fn log_ollama_js_no_blocks(response_content: String) -> Result<(), String> {
    use tracing::info;

    info!("Ollama JS Execution: No JavaScript code blocks found in response");
    info!(
        "Ollama JS Execution: Response preview:\n{}",
        response_content
    );

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaChatWithExecutionRequest {
    pub question: String,
    pub system_prompt: Option<String>,
    pub conversation_history: Option<Vec<crate::ollama::ChatMessage>>,
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

/// Parse FETCH_URL: <url> from assistant response. Returns the first valid URL if present (task-002).
fn parse_fetch_url_from_response(content: &str) -> Option<String> {
    let prefix = "FETCH_URL:";
    for line in content.lines() {
        let line = line.trim();
        if line.to_uppercase().starts_with(prefix) {
            let arg = line[prefix.len()..].trim();
            if let Some(url) = crate::commands::browser::extract_first_url(arg) {
                return Some(url);
            }
        }
    }
    None
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

    let chat_request = ChatRequest {
        messages: messages.clone(),
    };

    info!("Ollama Chat with Execution: Sending initial request to Ollama");
    let mut response = ollama_chat(chat_request)
        .await
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

    // Check if this is a code-assistant response
    let trimmed = processed_content.trim();
    let is_code_assistant = trimmed.starts_with("ROLE=code-assistant")
        || trimmed.to_lowercase().starts_with("role=code-assistant");

    // Fallback: Detect JavaScript code patterns even without ROLE=code-assistant prefix
    // This handles cases where Ollama returns code directly
    let looks_like_javascript = if !is_code_assistant {
        let lower = trimmed.to_lowercase();
        // Check for common JavaScript patterns
        lower.contains("console.log")
            || lower.contains("new date()")
            || lower.contains("document.")
            || lower.contains("window.")
            || lower.contains("function")
            || lower.contains("=>")
            || (lower.contains("(")
                && lower.contains(")")
                && (lower.contains("tostring")
                    || lower.contains("tolocaledate")
                    || lower.contains("tolocalestring")
                    || lower.contains("getday")
                    || lower.contains("getdate")
                    || lower.contains("getmonth")
                    || lower.contains("getfullyear")))
    } else {
        false
    };

    let needs_code_execution = is_code_assistant || looks_like_javascript;

    if needs_code_execution {
        if is_code_assistant {
            info!("Ollama Chat with Execution: Detected code-assistant response");
        } else {
            info!(
                "Ollama Chat with Execution: Detected JavaScript code pattern (fallback detection)"
            );
        }

        // Extract code
        let code = if is_code_assistant {
            // Extract code (everything after the first line if ROLE=code-assistant)
            let lines: Vec<&str> = processed_content.split('\n').collect();
            if lines.len() >= 2 {
                lines[1..].join("\n").trim().to_string()
            } else {
                processed_content
                    .replace("ROLE=code-assistant", "")
                    .trim()
                    .to_string()
            }
        } else {
            // Use the entire content as code (no ROLE prefix)
            trimmed.to_string()
        };

        // Remove markdown code block markers
        let code = code
            .replace("```javascript", "")
            .replace("```js", "")
            .replace("```", "")
            .trim()
            .to_string();

        // Handle console.log() - extract the expression inside
        // If code is "console.log(expression)", extract just "expression"
        let code = if code.trim_start().to_lowercase().starts_with("console.log(") {
            // Extract content between console.log( and the matching closing paren
            let start = code.find("console.log(").unwrap_or(0) + "console.log(".len();
            let mut paren_count = 1;
            let mut end = start;
            let chars: Vec<char> = code.chars().collect();
            for (i, ch) in chars.iter().enumerate().skip(start) {
                match ch {
                    '(' => paren_count += 1,
                    ')' => {
                        paren_count -= 1;
                        if paren_count == 0 {
                            end = i;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if end > start {
                code[start..end].trim().to_string()
            } else {
                code
            }
        } else {
            code
        };

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

        // Return code for execution
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

    // Check if Ollama is asking for more code execution (ping-pong)
    let trimmed = processed_content.trim();
    let is_code_assistant = trimmed.starts_with("ROLE=code-assistant")
        || trimmed.to_lowercase().starts_with("role=code-assistant");

    // Fallback: Detect JavaScript code patterns even without ROLE=code-assistant prefix
    let looks_like_javascript = if !is_code_assistant {
        let lower = trimmed.to_lowercase();
        lower.contains("console.log")
            || lower.contains("new date()")
            || lower.contains("document.")
            || lower.contains("window.")
            || lower.contains("function")
            || lower.contains("=>")
            || (lower.contains("(")
                && lower.contains(")")
                && (lower.contains("tostring")
                    || lower.contains("tolocaledate")
                    || lower.contains("tolocalestring")
                    || lower.contains("getday")
                    || lower.contains("getdate")
                    || lower.contains("getmonth")
                    || lower.contains("getfullyear")))
    } else {
        false
    };

    let needs_code_execution = is_code_assistant || looks_like_javascript;

    if needs_code_execution {
        if is_code_assistant {
            info!("Ollama Chat Continue: Detected another code-assistant response (ping-pong)");
        } else {
            info!(
                "Ollama Chat Continue: Detected JavaScript code pattern (ping-pong, fallback detection)"
            );
        }

        // Extract code
        let code = if is_code_assistant {
            // Extract code (everything after the first line if ROLE=code-assistant)
            let lines: Vec<&str> = processed_content.split('\n').collect();
            if lines.len() >= 2 {
                lines[1..].join("\n").trim().to_string()
            } else {
                processed_content
                    .replace("ROLE=code-assistant", "")
                    .trim()
                    .to_string()
            }
        } else {
            // Use the entire content as code (no ROLE prefix)
            trimmed.to_string()
        };

        // Remove markdown code block markers
        let code = code
            .replace("```javascript", "")
            .replace("```js", "")
            .replace("```", "")
            .trim()
            .to_string();

        // Handle console.log() - extract the expression inside
        // If code is "console.log(expression)", extract just "expression"
        let code = if code.trim_start().to_lowercase().starts_with("console.log(") {
            // Extract content between console.log( and the matching closing paren
            let start = code.find("console.log(").unwrap_or(0) + "console.log(".len();
            let mut paren_count = 1;
            let mut end = start;
            let chars: Vec<char> = code.chars().collect();
            for (i, ch) in chars.iter().enumerate().skip(start) {
                match ch {
                    '(' => paren_count += 1,
                    ')' => {
                        paren_count -= 1;
                        if paren_count == 0 {
                            end = i;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if end > start {
                code[start..end].trim().to_string()
            } else {
                code
            }
        } else {
            code
        };

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

        // Return code for execution (ping-pong)
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
        build_agent_runtime_context, build_perplexity_verbose_summary,
        extract_last_prefixed_argument, final_reply_from_tool_results,
        grounded_redmine_time_entries_failure_reply,
        is_likely_article_like_result, original_request_for_retry, parse_agent_tool_from_response,
        parse_all_tools_from_response, parse_tool_from_response, redmine_direct_fallback_hint,
        redmine_request_for_routing, redmine_time_entries_range_for_date,
        is_grounded_redmine_time_entries_blocked_reply,
        sanitize_success_criteria, score_search_result_for_news,
        shape_perplexity_results_for_question, summarize_response_for_verification,
        truncate_text_on_line_boundaries,
    };

    #[test]
    fn perplexity_verbose_summary_zero_results() {
        let s = build_perplexity_verbose_summary(0, String::new(), 380);
        assert_eq!(s, "Perplexity: 0 results.");
    }

    #[test]
    fn perplexity_verbose_summary_with_titles() {
        let s = build_perplexity_verbose_summary(3, "El País, El Mundo, ABC".to_string(), 380);
        assert_eq!(s, "Perplexity: 3 result(s) — El País, El Mundo, ABC");
    }

    #[test]
    fn perplexity_verbose_summary_titles_empty_but_n_nonzero() {
        let s = build_perplexity_verbose_summary(5, String::new(), 380);
        assert_eq!(s, "Perplexity: 5 result(s) received.");
    }

    #[test]
    fn perplexity_verbose_summary_truncated() {
        let long = "A".repeat(400);
        let s = build_perplexity_verbose_summary(1, long.clone(), 380);
        assert!(s.chars().count() <= 380);
        assert!(s.ends_with('…'));
        assert!(s.starts_with("Perplexity: 1 result(s) — "));
    }

    #[test]
    fn perplexity_verbose_summary_just_under_limit() {
        let titles = "X, Y, Z";
        let s = build_perplexity_verbose_summary(3, titles.to_string(), 380);
        assert_eq!(s, "Perplexity: 3 result(s) — X, Y, Z");
    }

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
    fn parse_all_tools_from_response_splits_inline_then_chain() {
        assert_eq!(
            parse_all_tools_from_response(
                "RECOMMEND: RUN_CMD: date +%Y-%m-%d then REDMINE_API GET /time_entries.json?from=2026-03-06&to=2026-03-06&limit=100"
            ),
            vec![
                ("RUN_CMD".to_string(), "date +%Y-%m-%d".to_string()),
                (
                    "REDMINE_API".to_string(),
                    "GET /time_entries.json?from=2026-03-06&to=2026-03-06&limit=100".to_string()
                )
            ]
        );
    }

    #[test]
    fn parse_all_tools_from_response_splits_inline_semicolon_chain() {
        assert_eq!(
            parse_all_tools_from_response(
                "RECOMMEND: RUN_CMD: date; REDMINE_API GET /time_entries.json?from=2026-03-06&to=2026-03-06&limit=100"
            ),
            vec![
                ("RUN_CMD".to_string(), "date".to_string()),
                (
                    "REDMINE_API".to_string(),
                    "GET /time_entries.json?from=2026-03-06&to=2026-03-06&limit=100".to_string()
                )
            ]
        );
    }

    #[test]
    fn parse_tool_from_response_supports_recommend_without_colon() {
        assert_eq!(
            parse_tool_from_response(
                "RECOMMEND: REDMINE_API GET /time_entries.json?from=2026-03-06&to=2026-03-06&limit=100"
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
    fn redmine_time_entries_range_for_date_uses_utc_day_for_today_queries() {
        let today = chrono::NaiveDate::from_ymd_opt(2026, 3, 6).unwrap();
        assert_eq!(
            redmine_time_entries_range_for_date(
                "Provide me the list of redmine tickets work on today.",
                today
            ),
            ("2026-03-06".to_string(), "2026-03-06".to_string())
        );
    }

    #[test]
    fn redmine_time_entries_range_for_date_uses_previous_utc_day_for_yesterday_queries() {
        let today = chrono::NaiveDate::from_ymd_opt(2026, 3, 6).unwrap();
        assert_eq!(
            redmine_time_entries_range_for_date(
                "Give me a list of Redmine tickets worked on yestaerday.",
                today
            ),
            ("2026-03-05".to_string(), "2026-03-05".to_string())
        );
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
    fn grounded_redmine_time_entries_failure_reply_for_dns_blocker_is_user_facing() {
        let reply = grounded_redmine_time_entries_failure_reply(
            "Provide me the list of redmine tickets work on today.",
            "Redmine API failed: Redmine request failed: error sending request for url (https://example.invalid/time_entries.json?from=2026-03-07&to=2026-03-07&limit=100): client error (Connect): dns error: failed to lookup address information. Answer without this result.",
        )
        .expect("expected grounded failure reply");

        assert!(reply.contains("Could not retrieve Redmine time entries for 2026-03-07"));
        assert!(reply.contains("configured Redmine host could not be resolved"));
        assert!(reply.contains("No Redmine data was fetched"));
        assert!(!reply.contains("no time entries were found"));
    }

    #[test]
    fn verification_accepts_grounded_redmine_blocked_reply() {
        assert!(is_grounded_redmine_time_entries_blocked_reply(
            "Provide me the list of redmine tickets work on today.",
            "Could not retrieve Redmine time entries for 2026-03-07 because the configured Redmine host could not be resolved. No Redmine data was fetched. Fix the Redmine configuration or connectivity, then retry."
        ));
    }

    #[test]
    fn redmine_direct_fallback_hint_for_today_avoids_user_id_me() {
        let hint =
            redmine_direct_fallback_hint("List redmine tickets that have been worked on today");
        assert!(hint.contains("/time_entries.json?from="));
        assert!(hint.contains("&to="));
        assert!(hint.contains("&limit=100"));
        assert!(!hint.contains("user_id=me"));
    }

    #[test]
    fn redmine_request_for_routing_prefers_original_request_on_retry() {
        assert_eq!(
            redmine_request_for_routing(
                "Verification said we didn't fully complete. Re-fetch the same period if needed.",
                "Provide me the list of redmine tickets work on today.",
                true,
            ),
            "Provide me the list of redmine tickets work on today."
        );
    }

    #[test]
    fn redmine_request_for_routing_keeps_today_window_on_retry() {
        let routed = redmine_request_for_routing(
            "Verification said we didn't fully complete. Re-fetch the same period if needed.",
            "Provide me the list of redmine tickets work on today.",
            true,
        );
        let today = chrono::NaiveDate::from_ymd_opt(2026, 3, 7).unwrap();
        assert_eq!(
            redmine_time_entries_range_for_date(routed, today),
            ("2026-03-07".to_string(), "2026-03-07".to_string())
        );
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
    fn score_search_result_for_news_prefers_article_like_pages() {
        let article_score = score_search_result_for_news(
            "Barcelona opens new civic center",
            "https://example.com/news/barcelona-opens-new-civic-center",
            "Barcelona opened a new civic center on March 6.",
        );
        let hub_score = score_search_result_for_news(
            "FC Barcelona News | Top Stories",
            "https://www.newsnow.com/us/Sports/Soccer/La+Liga/Barcelona",
            "Top stories and transfer rumors View on X",
        );
        assert!(article_score > hub_score);
    }

    #[test]
    fn shape_perplexity_results_for_question_limits_repeated_domains_for_news() {
        let results = vec![
            crate::commands::perplexity::PerplexitySearchResult {
                title: "Hub page".to_string(),
                url: "https://www.newsnow.com/us/Sports/Soccer/La+Liga/Barcelona".to_string(),
                snippet: "Top stories and rumors".to_string(),
                date: Some("2026-03-06".to_string()),
                last_updated: None,
            },
            crate::commands::perplexity::PerplexitySearchResult {
                title: "Article one".to_string(),
                url: "https://example.com/news/barcelona-culture-update".to_string(),
                snippet: "Culture update from Barcelona.".to_string(),
                date: Some("2026-03-06".to_string()),
                last_updated: None,
            },
            crate::commands::perplexity::PerplexitySearchResult {
                title: "Article two".to_string(),
                url: "https://example.com/news/barcelona-transit-update".to_string(),
                snippet: "Transit update from Barcelona.".to_string(),
                date: Some("2026-03-05".to_string()),
                last_updated: None,
            },
            crate::commands::perplexity::PerplexitySearchResult {
                title: "Article three".to_string(),
                url: "https://example.com/news/barcelona-housing-update".to_string(),
                snippet: "Housing update from Barcelona.".to_string(),
                date: Some("2026-03-04".to_string()),
                last_updated: None,
            },
        ];
        let (shaped, _, filtered_any) = shape_perplexity_results_for_question(
            "Show me recent Barcelona news with sources and dates.",
            results,
            280,
        );
        let example_count = shaped
            .iter()
            .filter(|r| r.url.contains("example.com"))
            .count();
        assert!(filtered_any);
        assert_eq!(example_count, 2);
        assert_eq!(
            shaped.first().map(|r| r.title.as_str()),
            Some("Article one")
        );
    }

    #[test]
    fn shape_perplexity_results_for_question_preserves_hub_only_fallback() {
        let results = vec![
            crate::commands::perplexity::PerplexitySearchResult {
                title: "Catalan News | News in English from Barcelona & Catalonia".to_string(),
                url: "https://www.catalannews.com".to_string(),
                snippet: "### International Women's Day in Barcelona\nMore headlines".to_string(),
                date: Some("2026-03-06".to_string()),
                last_updated: None,
            },
            crate::commands::perplexity::PerplexitySearchResult {
                title: "FC Barcelona News | Barça News & Top Stories".to_string(),
                url: "https://www.newsnow.com/us/Sports/Soccer/La+Liga/Barcelona".to_string(),
                snippet: "Top stories and transfer rumors View on X".to_string(),
                date: Some("2026-03-06".to_string()),
                last_updated: None,
            },
        ];
        let (shaped, _, _) = shape_perplexity_results_for_question(
            "Can you look on the Internet for news involving Barcelona? Mention sources and dates.",
            results,
            280,
        );
        assert_eq!(shaped.len(), 2);
        assert!(
            shaped
                .iter()
                .all(|r| !is_likely_article_like_result(&r.title, &r.url, &r.snippet))
        );
    }
}
