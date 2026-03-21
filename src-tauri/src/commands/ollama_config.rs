//! Ollama configuration, startup, and environment-variable resolution.
//!
//! Extracted from `ollama.rs` to keep the main command file focused on
//! chat execution and tool orchestration.

use crate::ollama::{ChatMessage, OllamaClient, OllamaConfig};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::sync::OnceLock;

use crate::commands::ollama_memory::load_soul_content;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaConfigResponse {
    pub endpoint: String,
    pub model: String,
}

/// Configure Ollama connection
#[tauri::command]
pub fn configure_ollama(config: OllamaConfigRequest) -> Result<(), String> {
    use serde_json;
    use tracing::{debug, info};

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

        let temp_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let url = format!("{}/api/tags", endpoint);
        let mut request = temp_client.get(&url);
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
pub(crate) fn read_ollama_api_key_from_env_or_config() -> Option<String> {
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
pub(crate) fn read_ollama_fast_model_from_env_or_config() -> Option<String> {
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
