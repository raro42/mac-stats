//! Ollama Tauri commands

use tauri::Manager;

use crate::config::Config;
use crate::ollama::{
    ChatMessage, EmbedInput, EmbedResponse, ListResponse, OllamaClient, OllamaConfig,
    PsResponse, VersionResponse,
};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::process::Command;
use std::str::FromStr;
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use std::sync::OnceLock;

/// Tool instructions appended to soul for non-agent chat (code execution + FETCH_URL).
const NON_AGENT_TOOL_INSTRUCTIONS: &str = "\n\nYou are a general purpose AI. If you are asked for actual data like day or weather information, or flight information or stock information. Then we need to compile that information using specially crafted clients for doing so. You will put \"[variable-name]\" into the answer to signal that we need to go another step and ask an agent to fulfil the answer.\n\nWhenever asked with \"[variable-name]\", you must provide a javascript snippet to be executed in the browser console to retrieve that information. Mark the answer to be executed as javascript. Do not put any other words around it. Do not insert formatting. Only return the code to be executed. This is needed for the next AI to understand and execute the same. When answering, use the role: code-assistant in the response. When you return executable code:\n- Start the response with: ROLE=code-assistant\n- On the next line, output ONLY executable JavaScript\n- Do not add explanations or formatting\n\nFor web pages: To fetch a page and use its content (e.g. \"navigate to X and get Y\"), reply with exactly one line: FETCH_URL: <full URL> (e.g. FETCH_URL: https://www.example.com). The app will fetch the page and give you the text; then answer the user based on that.";

/// Load soul content from ~/.mac-stats/agents/soul.md (or write default there if missing).
fn load_soul_content() -> String {
    Config::load_soul_content()
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
    use tracing::{debug, info};
    use serde_json;
    
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

    ollama_config.validate()
        .map_err(|e| {
            debug!("Ollama: Configuration validation failed: {}", e);
            e.to_string()
        })?;

    let endpoint = config.endpoint.clone();
    info!("Ollama: Using endpoint: {}", endpoint);
    
    let client = OllamaClient::new(ollama_config)
        .map_err(|e| {
            debug!("Ollama: Failed to create client: {}", e);
            e.to_string()
        })?;

    *get_ollama_client().lock()
        .map_err(|e| e.to_string())? = Some(client);

    info!("Ollama: Configuration successful with endpoint: {}", endpoint);
    Ok(())
}

/// Check Ollama connection (async, non-blocking)
#[tauri::command]
pub async fn check_ollama_connection() -> Result<bool, String> {
    use tracing::{debug, info};
    
    // Clone the client config to avoid holding the lock across await
    let client_config = {
        let client_guard = get_ollama_client().lock()
            .map_err(|e| e.to_string())?;
        
        if let Some(ref client) = *client_guard {
            Some((client.config.endpoint.clone(), client.config.model.clone(), client.config.api_key.clone()))
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
        
        // Add API key if configured
        if let Some(keychain_account) = &api_key {
            if let Ok(Some(api_key_value)) = crate::security::get_credential(keychain_account) {
                request = request.header("Authorization", format!("Bearer {}", api_key_value));
            }
        }
        
        let result = request.send().await
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
        info!("Ollama agent: not configured at startup, detecting models from {}", DEFAULT_ENDPOINT);
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
            debug!("Ollama agent: default config failed (endpoint may be down): {}", e);
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
        Ok(false) => debug!("Ollama agent: endpoint not reachable at startup (will retry when used)"),
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
                list.models.iter().map(|m| m.name.as_str()).collect::<Vec<_>>().join(", ")
            );
            list.models[0].name.clone()
        }
        _ => "llama3.2".to_string(),
    }
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
        std::env::current_dir().ok().map(|d| d.join("src-tauri").join(".config.env")),
        std::env::var("HOME").ok().map(|h| std::path::PathBuf::from(h).join(".mac-stats").join(".config.env")),
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
        Some(crate::ollama::ChatOptions { temperature, num_ctx })
    } else {
        None
    }
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

    let (endpoint, model, api_key, config_temp, config_num_ctx) = {
        let client_guard = get_ollama_client()
            .lock()
            .map_err(|e| e.to_string())?;
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

    let temp_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
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
        info!("Ollama → Request (POST /api/chat) ({} chars total):\n{}", request_json.len(), ellipsed);
    }

    let mut http_request = temp_client.post(&url).json(&chat_request);
    if let Some(keychain_account) = &api_key {
        if let Ok(Some(api_key_value)) = crate::security::get_credential(keychain_account) {
            let _masked = crate::security::mask_credential(&api_key_value);
            http_request = http_request.header("Authorization", format!("Bearer {}", api_key_value));
            debug!("Ollama: Using API key for chat request");
        }
    }

    let response = http_request
        .send()
        .await
        .map_err(|e| format!("Failed to send chat request: {}", e))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    let response: crate::ollama::ChatResponse = match serde_json::from_str(&body) {
        Ok(r) => r,
        Err(_) => {
            if let Ok(err_payload) = serde_json::from_str::<crate::ollama::OllamaErrorResponse>(&body) {
                return Err(format!("Ollama error: {}", err_payload.error));
            }
            if !status.is_success() {
                return Err(format!("Ollama HTTP {}: {}", status, body.trim()));
            }
            return Err(format!("Ollama returned invalid response (missing message): {}", body.trim()));
        }
    };
    if !status.is_success() {
        let msg = response
            .message
            .content
            .trim();
        return Err(format!("Ollama HTTP {}: {}", status, if msg.is_empty() { &body } else { msg }));
    }
    let content = &response.message.content;
    let n = content.chars().count();
    const RESPONSE_LOG_MAX: usize = 1000;
    if verbosity >= 2 || n <= RESPONSE_LOG_MAX {
        info!("Ollama ← Response ({} chars):\n{}", n, content);
    } else {
        let ellipsed = crate::logging::ellipse(content, RESPONSE_LOG_MAX);
        info!("Ollama ← Response ({} chars):\n{}", n, ellipsed);
    }
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

/// Base agent descriptions (without MCP). Includes RUN_JS, FETCH_URL, BRAVE_SEARCH, SCHEDULE.
const AGENT_DESCRIPTIONS_BASE: &str = r#"We have 4 agents available:

1. **RUN_JS** (JavaScript superpowers): Execute JavaScript in the app context (e.g. browser console). Use for: dynamic data, DOM inspection, client-side state. To invoke: reply with exactly one line: RUN_JS: <JavaScript code>. Note: In some contexts (e.g. Discord) JS is not executed; then answer without running code.

2. **FETCH_URL**: Fetch the full text of a web page. Use for: reading a specific URL's content. To invoke: reply with exactly one line: FETCH_URL: <full URL> (e.g. FETCH_URL: https://www.example.com). The app will return the page text.

3. **BRAVE_SEARCH**: Web search via Brave Search API. Use for: finding current info, facts, multiple sources. To invoke: reply with exactly one line: BRAVE_SEARCH: <search query>. The app will return search results.

4. **SCHEDULE** (scheduler): Add a task to run at scheduled times (recurring or one-shot). Use when the user wants something to run later or repeatedly. Three formats (reply exactly one line):
   - SCHEDULE: every N minutes <task> (e.g. SCHEDULE: every 5 minutes Execute RUN_JS to fetch CPU and RAM).
   - SCHEDULE: <cron expression> <task> — cron is 6-field (sec min hour day month dow) or 5-field (min hour day month dow; we accept and prepend 0 for seconds). Examples below.
   - SCHEDULE: at <datetime> <task> — one-shot (e.g. reminder tomorrow 5am: use RUN_CMD: date +%Y-%m-%d to get today, then SCHEDULE: at 2025-02-09T05:00:00 Remind me of my flight). Datetime must be ISO local: YYYY-MM-DDTHH:MM:SS or YYYY-MM-DD HH:MM.
   We add to ~/.mac-stats/schedules.json and return a schedule ID (e.g. discord-1770648842). Always tell the user this ID so they can remove it later with REMOVE_SCHEDULE.

5. **REMOVE_SCHEDULE**: Remove a scheduled task by its ID. Use when the user asks to remove, delete, or cancel a schedule (e.g. "Remove schedule: discord-1770648842"). Reply with exactly one line: REMOVE_SCHEDULE: <schedule-id> (e.g. REMOVE_SCHEDULE: discord-1770648842).

6. **LIST_SCHEDULES**: List all active schedules (id, type, next run, task). Use when the user asks to list schedules, show schedules, what's scheduled, what reminders are set, etc. Reply with exactly one line: LIST_SCHEDULES or LIST_SCHEDULES:."#;

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
IMPORTANT: For Discord tasks, prefer **AGENT: discord-expert** — it makes multiple API calls autonomously and knows all endpoints.
If calling directly: use DISCORD_API: GET <path> (NOT FETCH_URL — FETCH_URL has no Discord token and will get 401).
Key endpoints: GET /users/@me/guilds, GET /guilds/{guild_id}/members/search?query=name, GET /guilds/{guild_id}/channels, POST /channels/{channel_id}/messages {"content":"..."}"#;

/// Build agent descriptions string: base, optional SKILL (when skills exist), optional RUN_CMD, then MCP when configured.
/// When from_discord is true and Discord is configured, appends DISCORD_API agent and endpoint list.
async fn build_agent_descriptions(from_discord: bool) -> String {
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
        num += 1;
    }
    base.push_str(&format!(
        "\n\n{}. **TASK** (task files under ~/.mac-stats/task/): Use when working on a task file or when the user asks for tasks. When the user wants agents to chat or have a conversation, invoke AGENT: orchestrator (or the right agent) so the conversation runs; do not only create a task. TASK_LIST: default is open and WIP only (reply: TASK_LIST or TASK_LIST: ). TASK_LIST: all — list all tasks grouped by status (reply: TASK_LIST: all when the user asks for all tasks). TASK_SHOW: <path or id> — show that task's content and status to the user. TASK_APPEND: append feedback (reply: TASK_APPEND: <path or task id> <content>). TASK_STATUS: set status (reply: TASK_STATUS: <path or task id> wip|finished|unsuccessful). When the user says \"close the task\", \"finish\", \"mark done\", or \"cancel\" a task, reply TASK_STATUS: <path or id> finished or unsuccessful. TASK_CREATE: create a new task (reply: TASK_CREATE: <topic> <id> <initial content>). Put the **full** user request into the initial content, including duration (e.g. \"research for 15 minutes\"), scope, and topic — the whole content is stored. If a task with that topic and id already exists, use TASK_APPEND or TASK_STATUS instead. For TASK_APPEND/TASK_STATUS use the task file name (e.g. task-20250222-120000-open) or the short id or topic (e.g. 1, research). TASK_ASSIGN: <path or id> <agent_id>. Paths must be under ~/.mac-stats/task.",
        num
    ));
    num += 1;
    base.push_str(&format!(
        "\n\n{}. **OLLAMA_API** (Ollama model management): List models (with details), get server version, list running models, pull/delete/load/unload models, generate embeddings. Use when the user asks what models are installed, to pull or delete a model, to free memory (unload), or to get embeddings for text. To invoke: reply with exactly one line: OLLAMA_API: <action> [args]. Actions: list_models (no args), version (no args), running (no args), pull <model> [stream true|false], delete <model>, embed <model> <text>, load <model> [keep_alive e.g. 5m], unload <model>. Results are returned as JSON or text.",
        num
    ));
    num += 1;
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
            "\n\n{}. **CURSOR_AGENT** (Cursor AI coding agent): Delegate coding tasks to the Cursor Agent CLI (an AI pair-programmer with full codebase access). Use when the user asks to write code, refactor, fix bugs, create files, or make changes in a project. The agent has access to read/write files and run shell commands in the configured workspace. To invoke: reply with exactly one line: CURSOR_AGENT: <detailed prompt describing the coding task>. The result (what cursor-agent did and its output) is returned. This is a powerful tool — use it for complex coding tasks that benefit from full codebase context.",
            num
        ));
        num += 1;
    }
    if crate::redmine::is_configured() {
        base.push_str(&format!(
            "\n\n{}. **REDMINE_API** (Redmine project management): Access Redmine issues, projects, and time entries via REST API. Use when the user asks to review a ticket, list issues, check project status, or look up issue details. To invoke: reply with exactly one line: REDMINE_API: GET /issues/1234.json?include=journals,attachments (fetch issue with full history). Key endpoints:\n- GET /issues/{{id}}.json?include=journals,attachments — full issue with comments and files\n- GET /issues.json?assigned_to_id=me&status_id=open — my open issues\n- GET /issues.json?project_id=ID&status_id=open&limit=25 — project issues\n- GET /projects.json — list projects\n- PUT /issues/{{id}}.json — update issue (add notes: {{\"issue\":{{\"notes\":\"comment\"}}}})\nAlways use .json suffix. When reviewing a ticket, fetch with include=journals,attachments to get the full picture.",
            num
        ));
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
        "\n\n{}. **MEMORY_APPEND** (persistent memory): Save a lesson learned for future sessions. Use when something important was discovered (a mistake to avoid, a working approach, a user preference). To invoke: reply with exactly one line: MEMORY_APPEND: <lesson> (saves to global memory, loaded for all agents) or MEMORY_APPEND: agent:<slug-or-id> <lesson> (saves to that agent's memory only). Keep lessons concise and actionable.",
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
            let mut mcp_section = format!("\n\n{}. **MCP** (tools from configured MCP server, {} tools): Use when the task matches a tool below. To invoke: reply with exactly one line: MCP: <tool_name> <arguments>. Arguments can be JSON (e.g. MCP: get_weather {{\"location\": \"NYC\"}}) or plain text (e.g. MCP: fetch_url https://example.com).\n\nAvailable MCP tools:\n", num, tools.len());
            for t in &tools {
                let desc = t.description.as_deref().unwrap_or("(no description)");
                mcp_section.push_str(&format!("- **{}**: {}\n", t.name, desc));
            }
            base + &mcp_section
        }
        Err(e) => {
            info!("Agent router: MCP list_tools failed ({}), omitting MCP from agent list", e);
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
        },
        crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: body_truncated_for_request,
        },
    ];

    match send_ollama_chat_messages(
        summarization_messages,
        model_override,
        options_override,
    )
    .await
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
        },
        crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: user_message.to_string(),
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
pub(crate) async fn run_agent_ollama_session(
    agent: &crate::agents::Agent,
    user_message: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> Result<String, String> {
    use tracing::info;
    info!(
        "Agent: {} ({}) running (model: {:?}, prompt {} chars)",
        agent.name,
        agent.id,
        agent.model,
        agent.combined_prompt.chars().count()
    );
    let mut messages = vec![
        crate::ollama::ChatMessage {
            role: "system".to_string(),
            content: agent.combined_prompt.clone(),
        },
        crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: user_message.to_string(),
        },
    ];
    let max_iters = agent.max_tool_iterations;
    let mut iteration = 0u32;
    loop {
        let response = send_ollama_chat_messages(messages.clone(), agent.model.clone(), None).await?;
        let out = response.message.content.trim().to_string();
        info!(
            "Agent: {} ({}) iter {} returned ({} chars)",
            agent.name, agent.id, iteration, out.chars().count()
        );

        if let Some(tool_result) = execute_agent_tool_call(&out, status_tx).await {
            iteration += 1;
            if iteration >= max_iters {
                info!("Agent: {} ({}) hit max tool iterations ({})", agent.name, agent.id, max_iters);
                return Ok(out);
            }
            messages.push(crate::ollama::ChatMessage {
                role: "assistant".to_string(),
                content: out,
            });
            messages.push(crate::ollama::ChatMessage {
                role: "user".to_string(),
                content: tool_result,
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

/// Execute a tool call found in an agent's response. Currently supports DISCORD_API.
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
                return Some("DISCORD_API requires a path (e.g. GET /users/@me/guilds). Try again.".to_string());
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
                Err(e) => Some(format!(
                    "DISCORD_API failed ({} {}): {}. Explain the error to the user or try a different approach.",
                    &method, &path, e
                )),
            }
        }
        _ => None,
    }
}

/// Parse DISCORD_API: tool calls from an agent's response content.
fn parse_agent_tool_from_response(content: &str) -> Option<(String, String)> {
    let prefixes = ["DISCORD_API:"];
    for line in content.lines() {
        let line = line.trim();
        let mut search = line;
        loop {
            if search.len() >= 2 && search.as_bytes()[0].is_ascii_digit() {
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
            if search.to_uppercase().starts_with(prefix) {
                let arg = search[prefix.len()..].trim().to_string();
                if !arg.is_empty() {
                    return Some((prefix.trim_end_matches(':').to_string(), arg));
                }
            }
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
A concise summary (max 300 words) of ONLY verified facts and successful outcomes. Rules:
- KEEP: IDs confirmed by API responses (guild IDs, channel IDs, user IDs), successful API calls and their actual results, user preferences and standing instructions, established context the user built up.
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
        },
        crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: user_msg,
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
        let messages: Vec<crate::ollama::ChatMessage> = crate::session_memory::get_messages(&entry.source, entry.session_id)
            .into_iter()
            .map(|(role, content)| crate::ollama::ChatMessage { role, content })
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
        match compact_conversation_history(&messages, "Periodic session compaction.").await {
            Ok((context, lessons)) => {
                if let Some(ref lesson_text) = lessons {
                    let memory_path = crate::config::Config::memory_file_path();
                    for line in lesson_text.lines() {
                        let line = line.trim().trim_start_matches("- ").trim();
                        if !line.is_empty() && line.len() > 5 {
                            let entry_line = format!("- {}\n", line);
                            let _ = append_to_file(&memory_path, &entry_line);
                        }
                    }
                    info!("Periodic session compaction: wrote lessons to {:?}", memory_path);
                }
                let inactive = entry.last_activity < inactive_cutoff;
                if inactive {
                    crate::session_memory::clear_session(&entry.source, entry.session_id);
                    info!("Periodic session compaction: cleared inactive session {} {}", entry.source, entry.session_id);
                } else {
                    let compacted = vec![("system".to_string(), context)];
                    crate::session_memory::replace_session(&entry.source, entry.session_id, compacted);
                    info!("Periodic session compaction: replaced active session {} {} with summary", entry.source, entry.session_id);
                }
            }
            Err(e) => {
                tracing::warn!("Periodic session compaction failed for {} {}: {}", entry.source, entry.session_id, e);
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
            if !v.is_empty() { return Some(v); }
        }
        for base in [std::env::current_dir().ok(), std::env::var("HOME").ok().map(std::path::PathBuf::from)].into_iter().flatten() {
            let paths = [base.join(".config.env"), base.join(".mac-stats").join(".config.env")];
            for p in &paths {
                if let Ok(content) = std::fs::read_to_string(p) {
                    for line in content.lines() {
                        if let Some(val) = line.strip_prefix(file_key) {
                            let val = val.trim().trim_matches('"').trim().to_string();
                            if !val.is_empty() { return Some(val); }
                        }
                    }
                }
            }
        }
        if let Ok(Some(v)) = crate::security::get_credential(keychain_key) {
            if !v.is_empty() { return Some(v); }
        }
        None
    };
    let instance = resolve("MASTODON_INSTANCE_URL", "MASTODON_INSTANCE_URL=", "mastodon_instance_url")?;
    let token = resolve("MASTODON_ACCESS_TOKEN", "MASTODON_ACCESS_TOKEN=", "mastodon_access_token")?;
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
        && (lower.contains("token") || lower.contains("credential") || lower.contains("authentication"))
        && (lower.contains("discord") || lower.contains("guild") || lower.contains("channel"))
}

/// Shared API for Discord (and other agents): ask Ollama how to solve, then run agents (FETCH_URL, BRAVE_SEARCH, RUN_JS).
/// 1) Planning: send user question + agent list, get RECOMMEND: plan.
/// 2) Execution: send plan + "now answer using agents", loop on FETCH_URL / BRAVE_SEARCH / RUN_JS (max 5 tool calls).
/// If `status_tx` is provided (e.g. from Discord), short status messages are sent so the user sees we're still working.
/// If `discord_reply_channel_id` is set (when the request came from Discord), SCHEDULE will store it so the scheduler can post results to that channel (DM or mention channel).
/// When `discord_user_id` and `discord_user_name` are set (from Discord message author), the prompt is prefixed with "You are talking to Discord user **{name}** (user id: {id})."
/// When set, `model_override` and `options_override` apply only to this request (e.g. from Discord "model: llama3" line).
/// Extract a numeric ticket/issue ID from text like "ticket #1234", "#1234", "issue 1234".
fn extract_ticket_id(text: &str) -> Option<u64> {
    // Match #NNNN
    if let Some(pos) = text.find('#') {
        let after = &text[pos + 1..];
        let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !digits.is_empty() {
            return digits.parse().ok();
        }
    }
    // Match "ticket NNNN" or "issue NNNN" without #
    for keyword in &["ticket ", "issue "] {
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

/// When set, `skill_content` is prepended to system prompts (from ~/.mac-stats/skills/skill-<n>-<topic>.md).
/// When set, `agent_override` uses that agent's model and combined_prompt (soul+mood+skill) for the main run (e.g. Discord "agent: 001").
/// When `allow_schedule` is false (e.g. when running from the scheduler), the SCHEDULE tool is disabled so a scheduled task cannot create more schedules.
/// When `conversation_history` is set (e.g. from Discord session memory), it is prepended so the model sees prior turns and can resolve "there", "it", etc.
pub async fn answer_with_ollama_and_fetch(
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
) -> Result<String, String> {
    use tracing::info;

    let (model_override, skill_content, max_tool_iterations) = if let Some(ref a) = agent_override {
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

    if let Some(ref model) = model_override {
        let available = list_ollama_models().await.map_err(|e| format!("Could not list models: {}", e))?;
        let found = available.iter().any(|m| m == model || m.starts_with(&format!("{}:", model)));
        if !found {
            return Err(format!("Model '{}' not found. Available: {}", model, available.join(", ")));
        }
    }

    let (endpoint, effective_model, api_key) = {
        let guard = get_ollama_client().lock().map_err(|e| e.to_string())?;
        let client = guard.as_ref().ok_or_else(|| "Ollama not configured".to_string())?;
        let effective = model_override.clone().unwrap_or_else(|| client.config.model.clone());
        let api_key = client
            .config
            .api_key
            .as_ref()
            .and_then(|acc| crate::security::get_credential(acc).ok().flatten());
        (
            client.config.endpoint.clone(),
            effective,
            api_key,
        )
    };
    let model_info = crate::ollama::get_model_info(&endpoint, &effective_model, api_key.as_deref())
        .await
        .unwrap_or_else(|_| crate::ollama::ModelInfo::default());

    let send_status = |msg: &str| {
        if let Some(ref tx) = status_tx {
            let _ = tx.send(msg.to_string());
        }
    };

    let q_preview: String = question.chars().take(120).collect();
    if question.len() > 120 {
        info!("Agent router: starting (question: {}... [{} chars])", q_preview, question.len());
    } else {
        info!("Agent router: starting (question: {})", q_preview);
    }
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
        truncate_status(question, 50)
    ));

    const CONVERSATION_HISTORY_CAP: usize = 20;
    let raw_history: Vec<crate::ollama::ChatMessage> = conversation_history
        .unwrap_or_default()
        .into_iter()
        .rev()
        .take(CONVERSATION_HISTORY_CAP)
        .rev()
        .collect();

    let conversation_history: Vec<crate::ollama::ChatMessage> = if raw_history.len() >= COMPACTION_THRESHOLD {
        send_status("Compacting session memory…");
        info!(
            "Session compaction: {} messages exceed threshold ({}), compacting",
            raw_history.len(), COMPACTION_THRESHOLD
        );
        match compact_conversation_history(&raw_history, question).await {
            Ok((context, lessons)) => {
                if let Some(ref lesson_text) = lessons {
                    let memory_path = crate::config::Config::memory_file_path();
                    for line in lesson_text.lines() {
                        let line = line.trim().trim_start_matches("- ").trim();
                        if !line.is_empty() && line.len() > 5 {
                            let entry = format!("- {}\n", line);
                            let _ = append_to_file(&memory_path, &entry);
                        }
                    }
                    info!("Session compaction: wrote lessons to {:?}", memory_path);
                }
                if let Some(channel_id) = discord_reply_channel_id {
                    let compacted = vec![
                        ("system".to_string(), context.clone()),
                    ];
                    crate::session_memory::replace_session("discord", channel_id, compacted);
                }
                info!("Session compaction: replaced {} messages with summary ({} chars)", raw_history.len(), context.len());
                vec![crate::ollama::ChatMessage {
                    role: "system".to_string(),
                    content: format!("Previous session context (compacted from {} messages):\n\n{}", raw_history.len(), context),
                }]
            }
            Err(e) => {
                tracing::warn!("Session compaction failed: {}, using raw history with 401 annotations", e);
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
    let agent_descriptions = build_agent_descriptions(from_discord).await;
    info!("Agent router: agent list built ({} chars)", agent_descriptions.len());

    // When no agent/skill override, prepend soul (~/.mac-stats/agents/soul.md) so Ollama gets personality/vibe in context.
    let router_soul = skill_content.as_ref().map_or_else(
        || {
            let s = load_soul_content();
            if s.is_empty() {
                format!("You are mac-stats v{}.\n\n", crate::config::Config::version())
            } else {
                format!("{}\n\nYou are mac-stats v{}.\n\n", s, crate::config::Config::version())
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
    // "run <command>" or "run command: <command>" → RUN_CMD, skip LLM planning (so we execute, not explain).
    let pre_routed_recommendation = if crate::commands::run_cmd::is_local_cmd_allowed() {
        let q = question.trim();
        let q_lower = q.to_lowercase();
        let cmd_rest = if q_lower.starts_with("run command:") {
            q[12..].trim() // "run command:".len() == 12
        } else if q_lower.starts_with("run ") {
            q[4..].trim() // "run ".len() == 4
        } else {
            ""
        };
        if !cmd_rest.is_empty() {
            let rec = format!("RUN_CMD: {}", cmd_rest);
            info!("Agent router: pre-routed to RUN_CMD (run command): {}", crate::logging::ellipse(cmd_rest, 60));
            Some(rec)
        } else if crate::redmine::is_configured() {
            let ticket_id = extract_ticket_id(&q_lower);
            if ticket_id.is_some() && (q_lower.contains("ticket") || q_lower.contains("issue") || q_lower.contains("redmine")) {
                let id = ticket_id.unwrap();
                let rec = format!("REDMINE_API: GET /issues/{}.json?include=journals,attachments", id);
                info!("Agent router: pre-routed to REDMINE_API for ticket #{}", id);
                Some(rec)
            } else {
                None
            }
        } else {
            None
        }
    } else if crate::redmine::is_configured() {
        let q_lower = question.to_lowercase();
        let ticket_id = extract_ticket_id(&q_lower);
        if ticket_id.is_some() && (q_lower.contains("ticket") || q_lower.contains("issue") || q_lower.contains("redmine")) {
            let id = ticket_id.unwrap();
            let rec = format!("REDMINE_API: GET /issues/{}.json?include=journals,attachments", id);
            info!("Agent router: pre-routed to REDMINE_API for ticket #{}", id);
            Some(rec)
        } else {
            None
        }
    } else {
        None
    };

    // --- Planning step: ask Ollama how it would solve the question (skip if pre-routed) ---
    let recommendation = if let Some(pre_routed) = pre_routed_recommendation {
        info!("Agent router: skipping LLM planning (pre-routed)");
        pre_routed
    } else {
        info!("Agent router: planning step — asking Ollama for RECOMMEND");
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
        let mut planning_messages: Vec<crate::ollama::ChatMessage> = vec![
            crate::ollama::ChatMessage {
                role: "system".to_string(),
                content: planning_system_content,
            },
        ];
        for msg in &conversation_history {
            planning_messages.push(msg.clone());
        }
        planning_messages.push(crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: format!(
                "Current user question: {}\n\nReply with RECOMMEND: your plan.",
                question
            ),
        });
        let plan_response = send_ollama_chat_messages(planning_messages, model_override.clone(), options_override.clone()).await?;
        let mut rec = plan_response.message.content.trim().to_string();
        while rec.to_uppercase().starts_with("RECOMMEND: ") || rec.to_uppercase().starts_with("RECOMMEND:") {
            let prefix_len = if rec.len() >= 11 && rec[..11].to_uppercase() == "RECOMMEND: " {
                11
            } else {
                10
            };
            rec = rec[prefix_len..].trim().to_string();
        }
        rec
    };
    info!("Agent router: understood plan — {}", recommendation.chars().take(200).collect::<String>());
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

    // Fast path: if the recommendation already contains a parseable tool call, execute it
    // directly instead of asking Ollama a second time to regurgitate the same tool line.
    let direct_tool = parse_tool_from_response(&recommendation);
    let (mut messages, mut response_content) = if let Some((ref tool, ref arg)) = direct_tool {
        info!("Agent router: plan contains direct tool call {}:{} — skipping execution Ollama call", tool, crate::logging::ellipse(arg, 60));
        let execution_system_content = match &skill_content {
            Some(skill) => format!(
                "{}Additional instructions from skill:\n\n{}\n\n---\n\n{}",
                discord_user_context, skill, execution_prompt
            ),
            None => format!(
                "{}{}{}",
                router_soul, discord_user_context, execution_prompt
            ),
        };
        let mut msgs: Vec<crate::ollama::ChatMessage> = vec![
            crate::ollama::ChatMessage { role: "system".to_string(), content: execution_system_content },
        ];
        for msg in &conversation_history {
            msgs.push(msg.clone());
        }
        msgs.push(crate::ollama::ChatMessage { role: "user".to_string(), content: question.to_string() });
        // Synthesize the tool line as if the model had output it — the tool loop
        // will push this as an assistant message before appending the tool result.
        let synthetic = format!("{}: {}", tool, arg);
        (msgs, synthetic)
    } else {
        info!("Agent router: execution step — sending plan + question, starting tool loop (max {} tools)", max_tool_iterations);
        let execution_system_content = match &skill_content {
            Some(skill) => format!(
                "{}Additional instructions from skill:\n\n{}\n\n---\n\n{}\n\nYour plan: {}",
                discord_user_context, skill, execution_prompt, recommendation
            ),
            None => format!(
                "{}{}{}\n\nYour plan: {}",
                router_soul, discord_user_context, execution_prompt, recommendation
            ),
        };
        let mut msgs: Vec<crate::ollama::ChatMessage> = vec![
            crate::ollama::ChatMessage { role: "system".to_string(), content: execution_system_content },
        ];
        for msg in &conversation_history {
            msgs.push(msg.clone());
        }
        msgs.push(crate::ollama::ChatMessage { role: "user".to_string(), content: question.to_string() });
        let response = send_ollama_chat_messages(msgs.clone(), model_override.clone(), options_override.clone()).await?;
        let content = response.message.content.clone();
        let n = content.chars().count();
        info!("Agent router: first response received ({} chars): {}", n, log_content(&content));
        // Fallback: if Ollama returned empty but the recommendation contains a parseable tool,
        // synthesize the tool call so the tool loop can execute it.
        if n == 0 {
            if let Some((tool, arg)) = parse_tool_from_response(&recommendation) {
                let synthetic = format!("{}: {}", tool, arg);
                info!("Agent router: empty response — falling back to tool from recommendation: {}", crate::logging::ellipse(&synthetic, 80));
                (msgs, synthetic)
            } else {
                (msgs, content)
            }
        } else {
            (msgs, content)
        }
    };

    let mut tool_count: u32 = 0;
    // Collect (agent_name, reply) for each AGENT call so we can append a conversation transcript when multiple agents participated.
    let mut agent_conversation: Vec<(String, String)> = Vec::new();
    // Dedupe repeated identical DISCORD_API calls so the model can't loop on the same request.
    let mut last_successful_discord_call: Option<(String, String)> = None;
    // Track the task file we're working on so we can append the full conversation at the end.
    let mut current_task_path: Option<std::path::PathBuf> = None;

    while tool_count < max_tool_iterations {
        let (tool, arg) = match parse_tool_from_response(&response_content) {
            Some((t, a)) => {
                let arg_preview: String = a.chars().take(80).collect();
                let arg_len = a.chars().count();
                if arg_len > 80 {
                    info!("Agent router: understood tool {} (arg: {}... [{} chars])", t, arg_preview, arg_len);
                } else {
                    info!("Agent router: understood tool {} (arg: {})", t, arg_preview);
                }
                (t, a)
            }
            None => {
                info!("Agent router: no tool call in response ({} chars), treating as final answer: {}", response_content.chars().count(), log_content(&response_content));
                break;
            }
        };
        tool_count += 1;
        info!("Agent router: running tool {}/{} — {}", tool_count, max_tool_iterations, tool);

        let user_message = match tool.as_str() {
            "FETCH_URL" if arg.contains("discord.com") => {
                let path = if let Some(pos) = arg.find("/api/v10") {
                    arg[pos + "/api/v10".len()..].to_string()
                } else if let Some(pos) = arg.find("/api/") {
                    arg[pos + "/api".len()..].to_string()
                } else {
                    String::new()
                };
                if !path.is_empty() {
                    info!("Agent router: redirecting FETCH_URL discord.com -> DISCORD_API GET {}", path);
                    send_status(&format!("Discord API: GET {}", path));
                    match crate::discord::api::discord_api_request("GET", &path, None).await {
                        Ok(result) => format!(
                            "Discord API result (GET {}):\n\n{}\n\nUse this to answer the user's question.",
                            path, result
                        ),
                        Err(e) => format!("Discord API failed (GET {}): {}. Try DISCORD_API: GET {} or delegate to AGENT: discord-expert.", path, e, path),
                    }
                } else {
                    info!("Agent router: blocked FETCH_URL for discord.com (no API path). Redirecting to discord-expert.");
                    "Cannot fetch discord.com pages directly. Discord requires authenticated API access. Use AGENT: discord-expert for all Discord tasks, or use DISCORD_API: GET <path> with the correct API endpoint.".to_string()
                }
            }
            "FETCH_URL" => {
                send_status("Fetching page…");
                info!("Discord/Ollama: FETCH_URL requested: {}", arg);
                let url = arg.to_string();
                let fetch_result = tokio::task::spawn_blocking(move || crate::commands::browser::fetch_page_content(&url))
                    .await
                    .map_err(|e| format!("Fetch task: {}", e))?
                    .map_err(|e| format!("Fetch page failed: {}", e));
                match fetch_result {
                    Ok(body) => {
                        let estimated_used = (messages.iter().map(|m| m.content.len()).sum::<usize>()
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
                            info!("Discord/Ollama: Fetch returned 401 Unauthorized, stopping");
                            format!("That URL returned 401 Unauthorized. Do not try another URL. Answer based on what you know.")
                        } else {
                            return Err(e);
                        }
                    }
                }
            }
            "BRAVE_SEARCH" => {
                send_status("Searching the web…");
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
                let code_ref = if code_label.is_empty() { "…" } else { &code_label };
                send_status(&format!("Running code: {}…", code_ref));
                info!("Discord/Ollama: RUN_JS running code: {} [{} chars]", code_ref, arg.chars().count());
                match run_js_via_node(&arg) {
                    Ok(result) => format!(
                        "JavaScript result:\n\n{}\n\nUse this to answer the user's question.",
                        result
                    ),
                    Err(e) => {
                        info!("Discord/Ollama: RUN_JS failed: {}", e);
                        format!("JavaScript execution failed: {}. Answer the user's question without running code.", e)
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
                        send_status(&format!("Using skill {}-{}…", skill.number, skill.topic));
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
                                format!("Skill \"{}-{}\" failed: {}. Answer without this result.", skill.number, skill.topic, e)
                            }
                        }
                    }
                    None => {
                        info!("Agent router: SKILL unknown selector \"{}\" (available: {:?})", selector, skills.iter().map(|s| format!("{}-{}", s.number, s.topic)).collect::<Vec<_>>());
                        format!(
                            "Unknown skill \"{}\". Available skills: {}. Answer without using a skill.",
                            selector,
                            skills.iter().map(|s| format!("{}-{}", s.number, s.topic)).collect::<Vec<_>>().join(", ")
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
                        let user_msg = if task_message.is_empty() {
                            question
                        } else {
                            task_message
                        };
                        const STATUS_MSG_MAX: usize = 120;
                        let preview: String = user_msg.chars().take(STATUS_MSG_MAX).collect();
                        let status_text = if user_msg.chars().count() > STATUS_MSG_MAX {
                            format!("{}…", preview)
                        } else {
                            preview
                        };
                        send_status(&format!("{} -> Ollama: {}", agent.name, status_text));
                        match run_agent_ollama_session(
                            agent,
                            user_msg,
                            status_tx.as_ref(),
                        )
                        .await
                        {
                            Ok(result) => {
                                let label = format!("{} ({})", agent.name, agent.id);
                                agent_conversation.push((label.clone(), result.trim().to_string()));
                                format!(
                                    "Agent \"{}\" ({}) result:\n\n{}\n\nUse this to answer the user's question.",
                                    agent.name, agent.id, result
                                )
                            }
                            Err(e) => {
                                info!("Agent router: AGENT session failed: {}", e);
                                format!(
                                    "Agent \"{}\" ({}) failed: {}. Answer without this result.",
                                    agent.name, agent.id, e
                                )
                            }
                        }
                    }
                    None => {
                        let list: String = agents
                            .iter()
                            .map(|a| a.slug.as_deref().unwrap_or(a.name.as_str()).to_string())
                            .collect::<Vec<_>>()
                            .join(", ");
                        info!("Agent router: AGENT unknown selector \"{}\" (available: {})", selector, list);
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
                        if schedule_preview.is_empty() { "…" } else { schedule_preview }
                    ));
                    info!("Agent router: SCHEDULE requested (arg len={})", arg.chars().count());
                    match parse_schedule_arg(&arg) {
                        Ok(ScheduleParseResult::Cron { cron_str, task }) => {
                            let id = format!("discord-{}", chrono::Utc::now().timestamp());
                            let reply_to_channel_id = discord_reply_channel_id.map(|u| u.to_string());
                            match crate::scheduler::add_schedule(id.clone(), cron_str.clone(), task.clone(), reply_to_channel_id) {
                                Ok(crate::scheduler::ScheduleAddOutcome::Added) => {
                                    info!("Agent router: SCHEDULE added (id={}, cron={})", id, cron_str);
                                    let task_preview: String = task.chars().take(100).collect();
                                    format!(
                                        "Schedule added successfully. Schedule ID: **{}**. The scheduler will run this task (cron: {}): \"{}\". Tell the user the schedule ID is {} and they can remove it later with \"Remove schedule: {}\" or by saying REMOVE_SCHEDULE: {}.",
                                        id, cron_str, task_preview.trim(), id, id, id
                                    )
                                }
                                Ok(crate::scheduler::ScheduleAddOutcome::AlreadyExists) => {
                                    info!("Agent router: SCHEDULE skipped (same task already scheduled)");
                                    "This task is already scheduled with the same cron and description. Tell the user no duplicate was added."
                                        .to_string()
                                }
                                Err(e) => {
                                    info!("Agent router: SCHEDULE failed: {}", e);
                                    format!("Failed to add schedule: {}. Tell the user and suggest they check ~/.mac-stats/schedules.json.", e)
                                }
                            }
                        }
                        Ok(ScheduleParseResult::At { at_str, task }) => {
                            let id = format!("discord-{}", chrono::Utc::now().timestamp());
                            let reply_to_channel_id = discord_reply_channel_id.map(|u| u.to_string());
                            match crate::scheduler::add_schedule_at(id.clone(), at_str.clone(), task.clone(), reply_to_channel_id) {
                                Ok(crate::scheduler::ScheduleAddOutcome::Added) => {
                                    info!("Agent router: SCHEDULE at added (id={}, at={})", id, at_str);
                                    let task_preview: String = task.chars().take(100).collect();
                                    format!(
                                        "One-time schedule added. Schedule ID: **{}** (at {}): \"{}\". Tell the user the schedule ID is {} and they can remove it with \"Remove schedule: {}\" or REMOVE_SCHEDULE: {}.",
                                        id, at_str, task_preview.trim(), id, id, id
                                    )
                                }
                                Ok(crate::scheduler::ScheduleAddOutcome::AlreadyExists) => {
                                    info!("Agent router: SCHEDULE at skipped (duplicate)");
                                    "This one-time schedule was already added. Tell the user no duplicate was added.".to_string()
                                }
                                Err(e) => {
                                    info!("Agent router: SCHEDULE at failed: {}", e);
                                    format!("Failed to add one-shot schedule: {}. Tell the user and suggest they check ~/.mac-stats/schedules.json.", e)
                                }
                            }
                        }
                        Err(e) => {
                            info!("Agent router: SCHEDULE parse failed: {}", e);
                            format!("Could not parse schedule (expected e.g. \"every 5 minutes <task>\", \"at <datetime> <task>\", or \"<cron> <task>\"): {}. Ask the user to rephrase.", e)
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
                        Ok(true) => format!("Schedule {} has been removed. Tell the user it is cancelled.", id),
                        Ok(false) => format!("No schedule found with ID \"{}\". The ID may be wrong or already removed. Tell the user.", id),
                        Err(e) => format!("Failed to remove schedule: {}. Tell the user.", e),
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
                if !crate::commands::run_cmd::is_local_cmd_allowed() {
                    "RUN_CMD is not available (disabled by ALLOW_LOCAL_CMD=0). Answer without running local commands.".to_string()
                } else {
                    const MAX_CMD_RETRIES: u32 = 3;
                    let mut current_cmd = arg.to_string();
                    let mut last_output = String::new();

                    for attempt in 0..=MAX_CMD_RETRIES {
                        send_status(&format!("Running local command{}: {}", if attempt > 0 { format!(" (retry {})", attempt) } else { String::new() }, current_cmd));
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
                                last_output = format!(
                                    "Here is the command output:\n\n{}\n\nUse this to answer the user's question.",
                                    output
                                );
                                break;
                            }
                            Err(e) => {
                                info!("Agent router: RUN_CMD failed (attempt {}): {}", attempt, e);
                                if attempt >= MAX_CMD_RETRIES {
                                    last_output = format!(
                                        "RUN_CMD failed after {} retries: {}. Answer the user's question without this result.",
                                        MAX_CMD_RETRIES, e
                                    );
                                    break;
                                }
                                // Ask Ollama to fix the command
                                let allowed = crate::commands::run_cmd::allowed_commands().join(", ");
                                let fix_prompt = format!(
                                    "The command `{}` failed with error:\n{}\n\nReply with ONLY the corrected command on a single line, in this exact format:\nRUN_CMD: <corrected command>\n\nAllowed commands: {}. Paths must be under ~/.mac-stats.",
                                    current_cmd, e, allowed
                                );
                                let fix_messages = vec![
                                    crate::ollama::ChatMessage { role: "user".to_string(), content: fix_prompt },
                                ];
                                match send_ollama_chat_messages(fix_messages, model_override.clone(), options_override.clone()).await {
                                    Ok(resp) => {
                                        let fixed = resp.message.content.trim().to_string();
                                        info!("Agent router: RUN_CMD fix suggestion: {}", crate::logging::ellipse(&fixed, 120));
                                        if let Some((_, new_arg)) = parse_tool_from_response(&fixed) {
                                            current_cmd = new_arg;
                                        } else {
                                            last_output = format!(
                                                "RUN_CMD failed: {}. AI could not produce a corrected command. Answer the user's question without this result.",
                                                e
                                            );
                                            break;
                                        }
                                    }
                                    Err(ollama_err) => {
                                        info!("Agent router: RUN_CMD fix Ollama call failed: {}", ollama_err);
                                        last_output = format!(
                                            "RUN_CMD failed: {}. Answer the user's question without this result.",
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
                } else if last_successful_discord_call.as_ref().map(|(m, p)| m == &method && p == &path).unwrap_or(false) {
                    "You already received the data for this endpoint above. Format it for the user and reply; do not call DISCORD_API again for the same path.".to_string()
                } else {
                    let status_msg = format!("Calling Discord API: {} {}", method, path);
                    send_status(&status_msg);
                    info!("Discord API: {} {}", method, path);
                    match crate::discord::api::discord_api_request(&method, &path, body.as_deref()).await {
                        Ok(result) => {
                            last_successful_discord_call = Some((method.clone(), path.clone()));
                            format!(
                                "Discord API result:\n\n{}\n\nUse this to answer the user's question.",
                                result
                            )
                        }
                        Err(e) => format!("Discord API failed: {}. Answer without this result.", e),
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
                info!("Agent router: OLLAMA_API requested: action={}, rest={} chars", action, rest.chars().count());
                let result = match action.as_str() {
                    "list_models" => {
                        list_ollama_models_full().await.map(|r| serde_json::to_string_pretty(&r).unwrap_or_else(|_| "[]".to_string())).map_err(|e| e)
                    }
                    "version" => get_ollama_version().await.map(|r| r.version).map_err(|e| e),
                    "running" => {
                        list_ollama_running_models().await.map(|r| serde_json::to_string_pretty(&r).unwrap_or_else(|_| "[]".to_string())).map_err(|e| e)
                    }
                    "pull" => {
                        let parts: Vec<&str> = rest.split_whitespace().collect();
                        let model = parts.first().map(|s| (*s).to_string()).unwrap_or_default();
                        let stream = parts.get(1).map(|s| *s == "true").unwrap_or(true);
                        if model.is_empty() {
                            Err("OLLAMA_API pull requires a model name.".to_string())
                        } else {
                            pull_ollama_model(model, stream).await.map(|_| "Pull completed.".to_string())
                        }
                    }
                    "delete" => {
                        let model = rest.to_string();
                        if model.is_empty() {
                            Err("OLLAMA_API delete requires a model name.".to_string())
                        } else {
                            delete_ollama_model(model).await.map(|_| "Model deleted.".to_string())
                        }
                    }
                    "embed" => {
                        let parts: Vec<&str> = rest.splitn(2, ' ').map(str::trim).collect();
                        if parts.len() < 2 || parts[1].is_empty() {
                            Err("OLLAMA_API embed requires: embed <model> <text>.".to_string())
                        } else {
                            let model = parts[0].to_string();
                            let input = serde_json::Value::String(parts[1].to_string());
                            ollama_embeddings(model, input, None).await.map(|r| serde_json::to_string_pretty(&r).unwrap_or_else(|_| "{}".to_string())).map_err(|e| e)
                        }
                    }
                    "load" => {
                        let parts: Vec<&str> = rest.splitn(2, char::is_whitespace).map(str::trim).collect();
                        let model = parts.first().map(|s| (*s).to_string()).unwrap_or_default();
                        let keep_alive = parts.get(1).filter(|s| !s.is_empty()).map(|s| (*s).to_string());
                        if model.is_empty() {
                            Err("OLLAMA_API load requires a model name.".to_string())
                        } else {
                            load_ollama_model(model, keep_alive).await.map(|_| "Model loaded.".to_string())
                        }
                    }
                    "unload" => {
                        let model = rest.to_string();
                        if model.is_empty() {
                            Err("OLLAMA_API unload requires a model name.".to_string())
                        } else {
                            unload_ollama_model(model).await.map(|_| "Model unloaded.".to_string())
                        }
                    }
                    _ => Err(format!("Unknown OLLAMA_API action: {}. Use list_models, version, running, pull, delete, embed, load, or unload.", action)),
                };
                match result {
                    Ok(msg) => format!("Ollama API result:\n\n{}\n\nUse this to answer the user's question.", msg),
                    Err(e) => format!("OLLAMA_API failed: {}. Answer without this result.", e),
                }
            }
            "TASK_APPEND" => {
                let (path_or_id, content) = match arg.find(' ') {
                    Some(i) => (arg[..i].trim(), arg[i..].trim()),
                    None => ("", ""),
                };
                if path_or_id.is_empty() || content.is_empty() {
                    "TASK_APPEND requires: TASK_APPEND: <path or task id> <content>.".to_string()
                } else {
                    match crate::task::resolve_task_path(path_or_id) {
                        Ok(path) => {
                            current_task_path = Some(path.clone());
                            let task_label = crate::task::task_file_name(&path);
                            send_status(&format!("Appending to task '{}'…", task_label));
                            info!("Agent router: TASK_APPEND for task '{}' ({} chars)", task_label, content.chars().count());
                            match crate::task::append_to_task(&path, content) {
                                Ok(()) => format!("Appended to task file '{}'. Use this to continue.", task_label),
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
                    "TASK_STATUS requires: TASK_STATUS: <path or task id> wip|finished.".to_string()
                } else {
                    let path_or_id = parts[..parts.len() - 1].join(" ");
                    let status = parts[parts.len() - 1].to_lowercase();
                    if !["wip", "finished", "unsuccessful", "paused"].contains(&status.as_str()) {
                        "TASK_STATUS status must be wip, finished, unsuccessful, or paused.".to_string()
                    } else {
                        match crate::task::resolve_task_path(&path_or_id) {
                            Ok(path) => {
                                if status == "finished" && !crate::task::all_sub_tasks_closed(&path).unwrap_or(true) {
                                    "Cannot set status to finished: not all sub-tasks (## Sub-tasks: ...) are finished or unsuccessful.".to_string()
                                } else {
                                    match crate::task::set_task_status(&path, &status) {
                                        Ok(new_path) => {
                                            current_task_path = Some(new_path.clone());
                                            format!("Task status set to {} (file: {}).", status, crate::task::task_file_name(&new_path))
                                        }
                                        Err(e) => format!("TASK_STATUS failed: {}.", e),
                                    }
                                }
                            }
                            Err(e) => format!("TASK_STATUS failed: {}.", e),
                        }
                    }
                }
            }
            "TASK_CREATE" => {
                let segs: Vec<&str> = arg.splitn(3, ' ').map(str::trim).collect();
                if segs.len() >= 3 && !segs[2].is_empty() {
                    let topic = segs[0];
                    let id = segs[1];
                    let initial_content = segs[2];
                    match crate::task::create_task(topic, id, initial_content, None) {
                        Ok(path) => {
                            current_task_path = Some(path.clone());
                            let name = crate::task::task_file_name(&path);
                            format!("Task created: {}. Use TASK_APPEND: {} or TASK_APPEND: <id> <content> and TASK_STATUS to update.", name, name)
                        }
                        Err(e) => format!("TASK_CREATE failed: {}.", e),
                    }
                } else {
                    "TASK_CREATE requires: TASK_CREATE: <topic> <id> <initial content>.".to_string()
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
                    let agent_id = parts[parts.len() - 1];
                    send_status(&format!("Assigning task to {}…", agent_id));
                    info!("Agent router: TASK_ASSIGN {} -> {}", path_or_id, agent_id);
                    match crate::task::resolve_task_path(&path_or_id) {
                        Ok(path) => {
                            current_task_path = Some(path.clone());
                            match crate::task::set_assignee(&path, agent_id) {
                                Ok(()) => {
                                    let _ = crate::task::append_to_task(&path, &format!("Reassigned to {}.", agent_id));
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
                let (path_or_id, until_str) = if parts.len() >= 3 && parts[parts.len() - 2].eq_ignore_ascii_case("until") {
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
                    info!("Agent router: TASK_SLEEP {} until {}", path_or_id, until_str);
                    match crate::task::resolve_task_path(&path_or_id) {
                        Ok(path) => {
                            current_task_path = Some(path.clone());
                            if let Ok(new_path) = crate::task::set_task_status(&path, "paused") {
                                current_task_path = Some(new_path.clone());
                                let _ = crate::task::set_paused_until(&new_path, Some(until_str));
                                let _ = crate::task::append_to_task(&new_path, &format!("Paused until {}.", until_str));
                            }
                            format!("Task paused until {}. It will resume automatically after that time.", until_str)
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
                                format!("**All tasks**\n\n{}", crate::logging::ellipse(&list, LIST_MAX))
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
                                format!("**Active task list**\n\n{}", crate::logging::ellipse(&list, LIST_MAX))
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
                info!("Agent router: MCP requested (arg len={})", arg.chars().count());
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
                        match crate::mcp::call_tool(&server_url, &mcp_tool_name, mcp_args).await {
                            Ok(result) => {
                                info!("Agent router: MCP tool {} completed ({} chars)", mcp_tool_name, result.len());
                                format!(
                                    "MCP tool \"{}\" result:\n\n{}\n\nUse this to answer the user's question.",
                                    mcp_tool_name, result
                                )
                            }
                            Err(e) => {
                                info!("Agent router: MCP tool {} failed: {}", mcp_tool_name, e);
                                format!("MCP tool \"{}\" failed: {}. Answer the user without this result.", mcp_tool_name, e)
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
                        info!("Agent router: CURSOR_AGENT running prompt ({} chars)", prompt.len());
                        match tokio::task::spawn_blocking({
                            let p = prompt.clone();
                            move || crate::commands::cursor_agent::run_cursor_agent(&p)
                        })
                        .await
                        .map_err(|e| format!("CURSOR_AGENT task: {}", e))
                        .and_then(|r| r)
                        {
                            Ok(output) => {
                                info!("Agent router: CURSOR_AGENT completed ({} chars output)", output.len());
                                let truncated = if output.chars().count() > 4000 {
                                    let half = 1800;
                                    let start: String = output.chars().take(half).collect();
                                    let end: String = output.chars().rev().take(half).collect::<String>().chars().rev().collect();
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
                                format!("CURSOR_AGENT failed: {}. Answer the user without this result.", e)
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
                    info!("Agent router: REDMINE_API {} {}", method, path);
                    match crate::redmine::redmine_api_request(&method, &path, body.as_deref()).await {
                        Ok(result) => format!(
                            "Redmine API result:\n\n{}\n\nUse this data to answer the user's question. Summarize the issue clearly: subject, description quality, what's missing, status, assignee, and key comments.",
                            result
                        ),
                        Err(e) => format!("Redmine API failed: {}. Answer without this result.", e),
                    }
                }
            }
            "MASTODON_POST" => {
                let arg = arg.trim();
                if arg.is_empty() {
                    "MASTODON_POST requires text. Usage: MASTODON_POST: <text to post>. Optional visibility prefix: MASTODON_POST: unlisted: <text> (default: public).".to_string()
                } else {
                    let (visibility, text) = if let Some(rest) = arg.strip_prefix("unlisted:").or_else(|| arg.strip_prefix("unlisted ")) {
                        ("unlisted", rest.trim())
                    } else if let Some(rest) = arg.strip_prefix("private:").or_else(|| arg.strip_prefix("private ")) {
                        ("private", rest.trim())
                    } else if let Some(rest) = arg.strip_prefix("direct:").or_else(|| arg.strip_prefix("direct ")) {
                        ("direct", rest.trim())
                    } else if let Some(rest) = arg.strip_prefix("public:").or_else(|| arg.strip_prefix("public ")) {
                        ("public", rest.trim())
                    } else {
                        ("public", arg)
                    };
                    send_status(&format!("Posting to Mastodon ({})…", visibility));
                    info!("Agent router: MASTODON_POST visibility={} text={}", visibility, crate::logging::ellipse(text, 100));
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
                        if let Some(agent) = crate::agents::find_agent_by_id_or_name(&agents, &selector) {
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
                        let path = crate::config::Config::memory_file_path();
                        append_to_file(&path, &lesson_line)
                    };
                    match result {
                        Ok(path) => {
                            info!("Agent router: MEMORY_APPEND wrote to {:?}", path);
                            format!("Memory updated ({}). The lesson will be included in future prompts.", path.display())
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
        info!("Agent router: tool {} completed, sending result back to Ollama ({} chars): {}", tool, result_len, log_content(&user_message));

        messages.push(crate::ollama::ChatMessage {
            role: "assistant".to_string(),
            content: response_content.clone(),
        });
        let tool_result_role = if user_message.starts_with("Here is the command output") {
            "system"
        } else {
            "user"
        };
        messages.push(crate::ollama::ChatMessage {
            role: tool_result_role.to_string(),
            content: user_message,
        });

        let follow_up = send_ollama_chat_messages(messages.clone(), model_override.clone(), options_override.clone()).await?;
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
                    info!("Agent router: Ollama returned empty after tool success — using raw tool output as response");
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
            info!("Agent router: max tool iterations reached ({}), using last response as final", max_tool_iterations);
        }
    }

    let final_len = response_content.chars().count();
    info!("Agent router: done after {} tool(s), returning final response ({} chars): {}", tool_count, final_len, log_content(&response_content));

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
    let is_runner_prompt = question.trim_start().starts_with("Current task file content:");
    if let Some(ref path) = current_task_path {
        if !is_runner_prompt {
            if let Err(e) = crate::task::append_conversation_block(path, question, &response_content) {
                info!("Agent router: could not append conversation to task file: {}", e);
            } else {
                info!("Agent router: appended conversation to task {}", crate::task::task_file_name(path));
            }
        } else {
            info!("Agent router: skipped appending conversation (task runner turn) for {}", crate::task::task_file_name(path));
        }
    }

    Ok(response_content)
}

/// Run JavaScript via Node.js (if available). Used for RUN_JS in Discord/agent context.
/// Writes code to a temp file and runs `node -e "..."` to eval and print the result.
fn run_js_via_node(code: &str) -> Result<String, String> {
    let tmp_dir = std::env::temp_dir();
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
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
        let n: u64 = n_str.parse().map_err(|_| "expected integer after 'every'".to_string())?;
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
            return Err("at requires a datetime and task (e.g. at 2025-02-09T05:00:00 Remind me)".to_string());
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
        return Err("invalid at datetime: use YYYY-MM-DDTHH:MM:SS or YYYY-MM-DD HH:MM (local time)".to_string());
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
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
                .map(|n| Local.from_local_datetime(&n).single().unwrap_or_else(|| n.and_utc().with_timezone(&Local)))
        })
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                .map(|n| Local.from_local_datetime(&n).single().unwrap_or_else(|| n.and_utc().with_timezone(&Local)))
        })
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M")
                .map(|n| Local.from_local_datetime(&n).single().unwrap_or_else(|| n.and_utc().with_timezone(&Local)))
        })
        .map_err(|e| format!("invalid datetime: {} (use YYYY-MM-DDTHH:MM:SS or YYYY-MM-DD HH:MM)", e))?;
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

/// Parse one of FETCH_URL:, BRAVE_SEARCH:, RUN_JS:, SCHEDULE:/SCHEDULER:, MCP:, PYTHON_SCRIPT: from assistant content.
/// Also accepts lines starting with "RECOMMEND: " (e.g. "RECOMMEND: SCHEDULER: Every 5 minutes...").
/// For TASK_APPEND and TASK_CREATE, content is taken to the end of the block (all lines until the next tool line) so research/full text is stored completely.
/// Returns (tool_name, argument) or None.
fn parse_tool_from_response(content: &str) -> Option<(String, String)> {
    let prefixes = ["FETCH_URL:", "BRAVE_SEARCH:", "RUN_JS:", "SKILL:", "AGENT:", "RUN_CMD:", "SCHEDULE:", "SCHEDULER:", "REMOVE_SCHEDULE:", "LIST_SCHEDULES:", "TASK_LIST:", "TASK_SHOW:", "TASK_APPEND:", "TASK_STATUS:", "TASK_CREATE:", "TASK_ASSIGN:", "TASK_SLEEP:", "OLLAMA_API:", "PYTHON_SCRIPT:", "MCP:", "DISCORD_API:", "CURSOR_AGENT:", "REDMINE_API:", "MEMORY_APPEND:", "MASTODON_POST:"];
    let lines: Vec<&str> = content.lines().collect();
    for (line_index, line) in lines.iter().enumerate() {
        let line = line.trim();
        // Ollama sometimes replies with just "TASK_LIST" or "LIST_SCHEDULES" (no colon); treat as tool call with empty arg.
        if line.eq_ignore_ascii_case("TASK_LIST") {
            return Some(("TASK_LIST".to_string(), String::new()));
        }
        if line.eq_ignore_ascii_case("LIST_SCHEDULES") {
            return Some(("LIST_SCHEDULES".to_string(), String::new()));
        }
        // Strip leading list numbering ("1. ", "2) ", "- ", "* ") and RECOMMEND: prefixes.
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
            if search.to_uppercase().starts_with(prefix) {
                let mut arg = search[prefix.len()..].trim().to_string();
                if arg.is_empty() && prefix != "TASK_LIST:" && prefix != "TASK_SHOW:" && prefix != "LIST_SCHEDULES:" {
                    continue;
                }
                let tool_name = prefix.trim_end_matches(':');
                let tool_name = if tool_name.eq_ignore_ascii_case("SCHEDULER") {
                    "SCHEDULE".to_string()
                } else {
                    tool_name.to_string()
                };
                // TASK_APPEND and TASK_CREATE: take full content including all following lines until the next tool line (so research/long text is stored completely).
                if tool_name == "TASK_APPEND" || tool_name == "TASK_CREATE" {
                    let rest_lines: Vec<&str> = lines[line_index + 1..]
                        .iter()
                        .take_while(|l| !line_starts_with_tool_prefix(l))
                        .copied()
                        .collect();
                    if !rest_lines.is_empty() {
                        arg.push('\n');
                        arg.push_str(&rest_lines.join("\n"));
                    }
                }
                // Ollama sometimes concatenates multiple tools on one line. Truncate at first ';' for URLs/searches.
                if tool_name == "FETCH_URL" || tool_name == "BRAVE_SEARCH" {
                    if let Some(idx) = arg.find(';') {
                        arg = arg[..idx].trim().to_string();
                    }
                }
                // Truncate at next numbered step boundary for single-line tools (not TASK_APPEND/TASK_CREATE — those keep full content).
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
                if !arg.is_empty() || tool_name == "TASK_LIST" || tool_name == "TASK_SHOW" || tool_name == "LIST_SCHEDULES" {
                    return Some((tool_name, arg));
                }
                if tool_name == "TASK_SLEEP" && !arg.is_empty() {
                    return Some((tool_name, arg));
                }
            }
        }
    }
    None
}

/// Tool line prefixes that indicate start of another tool (used to stop script body extraction).
const TOOL_LINE_PREFIXES: &[&str] = &[
    "FETCH_URL:", "BRAVE_SEARCH:", "RUN_JS:", "SKILL:", "AGENT:", "RUN_CMD:", "SCHEDULE:", "SCHEDULER:", "REMOVE_SCHEDULE:", "LIST_SCHEDULES:",
    "TASK_LIST:", "TASK_SHOW:", "TASK_APPEND:", "TASK_STATUS:", "TASK_CREATE:", "TASK_ASSIGN:", "TASK_SLEEP:", "OLLAMA_API:", "MCP:", "PYTHON_SCRIPT:", "DISCORD_API:", "CURSOR_AGENT:", "REDMINE_API:", "MEMORY_APPEND:",
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
        let after_newline = content[start + 3..].find('\n').map(|i| start + 3 + i + 1).unwrap_or(start + 3);
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
        let is_other_tool = TOOL_LINE_PREFIXES.iter().any(|p| trimmed.to_uppercase().starts_with(p));
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
    use tracing::{debug, info};
    use serde_json;
    
    info!("Ollama: Listing available models...");
    
    // Clone client data to avoid holding lock across await
    let (endpoint, api_key) = {
        let client_guard = get_ollama_client().lock()
            .map_err(|e| e.to_string())?;

        let client = client_guard.as_ref()
            .ok_or_else(|| "Ollama not configured".to_string())?;
        
        (client.config.endpoint.clone(), client.config.api_key.clone())
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
            debug!("Ollama: Using API key for model listing (masked: {})", masked);
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

/// List available Ollama models with full details (GET /api/tags).
#[tauri::command]
pub async fn list_ollama_models_full() -> Result<ListResponse, String> {
    let config = {
        let guard = get_ollama_client().lock().map_err(|e| e.to_string())?;
        let client = guard.as_ref().ok_or_else(|| "Ollama not configured".to_string())?;
        client.config.clone()
    };
    let client = OllamaClient::new(config).map_err(|e| e.to_string())?;
    client
        .list_models_full()
        .await
        .map_err(|e| e.to_string())
}

/// Get Ollama server version (GET /api/version).
#[tauri::command]
pub async fn get_ollama_version() -> Result<VersionResponse, String> {
    let config = {
        let guard = get_ollama_client().lock().map_err(|e| e.to_string())?;
        let client = guard.as_ref().ok_or_else(|| "Ollama not configured".to_string())?;
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
        let client = guard.as_ref().ok_or_else(|| "Ollama not configured".to_string())?;
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
        let client = guard.as_ref().ok_or_else(|| "Ollama not configured".to_string())?;
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
        let client = guard.as_ref().ok_or_else(|| "Ollama not configured".to_string())?;
        client.config.clone()
    };
    let client = OllamaClient::new(config).map_err(|e| e.to_string())?;
    client
        .delete_model(&model)
        .await
        .map_err(|e| e.to_string())
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
        let client = guard.as_ref().ok_or_else(|| "Ollama not configured".to_string())?;
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
        let client = guard.as_ref().ok_or_else(|| "Ollama not configured".to_string())?;
        client.config.clone()
    };
    let client = OllamaClient::new(config).map_err(|e| e.to_string())?;
    client
        .unload_model(&model)
        .await
        .map_err(|e| e.to_string())
}

/// Load (warm) a model into memory. Optional keep_alive e.g. "5m".
#[tauri::command]
pub async fn load_ollama_model(model: String, keep_alive: Option<String>) -> Result<(), String> {
    let config = {
        let guard = get_ollama_client().lock().map_err(|e| e.to_string())?;
        let client = guard.as_ref().ok_or_else(|| "Ollama not configured".to_string())?;
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
    info!("Ollama JS Execution: Response length: {} characters", response_length);
    const LOG_MAX: usize = 500;
    let verbosity = crate::logging::VERBOSITY.load(Ordering::Relaxed);
    if verbosity >= 2 {
        info!("Ollama JS Execution: Response content:\n{}", response_content);
    } else {
        info!("Ollama JS Execution: Response content:\n{}", crate::logging::ellipse(&response_content, LOG_MAX));
    }
    
    Ok(())
}

/// Log JavaScript code block extraction results
#[tauri::command]
pub fn log_ollama_js_extraction(found_blocks: usize, blocks: Vec<String>) -> Result<(), String> {
    use tracing::info;
    
    info!("Ollama JS Execution: Extraction complete - found {} code block(s)", found_blocks);
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
    info!("Ollama JS Execution: Response preview:\n{}", response_content);
    
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

/// Parse FETCH_URL: <url> from assistant response. Returns the URL if present.
fn parse_fetch_url_from_response(content: &str) -> Option<String> {
    let prefix = "FETCH_URL:";
    for line in content.lines() {
        let line = line.trim();
        if line.to_uppercase().starts_with(prefix) {
            let url = line[prefix.len()..].trim();
            if !url.is_empty() {
                return Some(url.to_string());
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
    use tracing::info;
    use crate::metrics::{get_cpu_details, get_metrics};

    ensure_cpu_window_open();

    info!("Ollama Chat with Execution: Starting for question: {}", request.question);

    // Get system metrics for context
    let cpu_details = get_cpu_details();
    let system_metrics = get_metrics();
    
    // Create context message
    let context_message = format!(
        "Current system status:\n- CPU: {:.1}%\n- Temperature: {:.1}°C\n- Frequency: {:.2} GHz\n- RAM: {:.1}%\n- Battery: {}\n\nUser question: {}",
        cpu_details.usage,
        cpu_details.temperature,
        cpu_details.frequency,
        system_metrics.ram,
        if cpu_details.has_battery {
            format!("{:.0}%", cpu_details.battery_level)
        } else {
            "N/A".to_string()
        },
        request.question
    );
    
    // Get system prompt: use soul.md (~/.mac-stats/agents/soul.md or bundled default) + tools when not overridden
    let system_prompt = request
        .system_prompt
        .unwrap_or_else(default_non_agent_system_prompt);
    
    // Build messages array with conversation history
    let mut messages = vec![
        crate::ollama::ChatMessage {
            role: "system".to_string(),
            content: system_prompt.clone(),
        },
    ];
    
    // Add conversation history if provided (exclude system messages - we already have one)
    if let Some(ref history) = request.conversation_history {
        for msg in history {
            // Only add user and assistant messages from history (skip system messages)
            if msg.role == "user" || msg.role == "assistant" {
                messages.push(msg.clone());
            }
        }
        info!("Ollama Chat with Execution: Added {} messages from conversation history", 
              history.iter().filter(|m| m.role == "user" || m.role == "assistant").count());
    }
    
    // Add current user message
    messages.push(crate::ollama::ChatMessage {
        role: "user".to_string(),
        content: context_message.clone(),
    });
    
    let chat_request = ChatRequest {
        messages: messages.clone(),
    };
    
    info!("Ollama Chat with Execution: Sending initial request to Ollama");
    let mut response = ollama_chat(chat_request).await
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
        info!("Ollama Chat with Execution: Fetched {} chars from {}", page_content.len(), url);

        // Build follow-up: current messages + assistant's FETCH_URL message + user with page content
        let mut follow_up_messages = messages.clone();
        follow_up_messages.push(crate::ollama::ChatMessage {
            role: "assistant".to_string(),
            content: response_content.clone(),
        });
        follow_up_messages.push(crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: format!(
                "Here is the page content:\n\n{}\n\nPlease answer the user's question based on this content.",
                page_content
            ),
        });

        let follow_up_request = ChatRequest {
            messages: follow_up_messages.clone(),
        };
        response = ollama_chat(follow_up_request).await
            .map_err(|e| format!("Failed to send follow-up after fetch: {}", e))?;
        response_content = response.message.content.clone();
        messages = follow_up_messages;
        info!("Ollama Chat with Execution: Received response after fetch ({} chars)", response_content.len());
    }

    info!("Ollama Chat with Execution: Received response ({} chars)", response_content.len());
    
    // Process response content - handle escaped newlines
    let mut processed_content = response_content.replace("\\n", "\n");
    // Remove "javascript\n" if present
    processed_content = processed_content.replace("javascript\n", "");
    
    // Check if this is a code-assistant response
    let trimmed = processed_content.trim();
    let is_code_assistant = trimmed.starts_with("ROLE=code-assistant") || 
                           trimmed.to_lowercase().starts_with("role=code-assistant");
    
    // Fallback: Detect JavaScript code patterns even without ROLE=code-assistant prefix
    // This handles cases where Ollama returns code directly
    let looks_like_javascript = if !is_code_assistant {
        let lower = trimmed.to_lowercase();
        // Check for common JavaScript patterns
        lower.contains("console.log") ||
        lower.contains("new date()") ||
        lower.contains("document.") ||
        lower.contains("window.") ||
        lower.contains("function") ||
        lower.contains("=>") ||
        (lower.contains("(") && lower.contains(")") && 
         (lower.contains("tostring") || lower.contains("tolocaledate") || 
          lower.contains("tolocalestring") || lower.contains("getday") ||
          lower.contains("getdate") || lower.contains("getmonth") ||
          lower.contains("getfullyear")))
    } else {
        false
    };
    
    let needs_code_execution = is_code_assistant || looks_like_javascript;
    
    if needs_code_execution {
        if is_code_assistant {
            info!("Ollama Chat with Execution: Detected code-assistant response");
        } else {
            info!("Ollama Chat with Execution: Detected JavaScript code pattern (fallback detection)");
        }
        
        // Extract code
        let code = if is_code_assistant {
            // Extract code (everything after the first line if ROLE=code-assistant)
            let lines: Vec<&str> = processed_content.split('\n').collect();
            if lines.len() >= 2 {
                lines[1..].join("\n").trim().to_string()
            } else {
                processed_content.replace("ROLE=code-assistant", "").trim().to_string()
            }
        } else {
            // Use the entire content as code (no ROLE prefix)
            trimmed.to_string()
        };
        
        // Remove markdown code block markers
        let code = code.replace("```javascript", "")
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
        
        info!("Ollama Chat with Execution: Extracted code ({} chars):\n{}", code.len(), code);
        
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

    info!("Ollama Chat Continue: Code executed, result: {}", execution_result);
    
    let system_prompt = system_prompt
        .unwrap_or_else(default_non_agent_system_prompt);
    
    let follow_up_message = format!(
        "I have executed your last codeblocks and the result is: {}\n\nCan you now answer the original question: {}?",
        execution_result,
        original_question
    );
    
    // Build messages array with conversation history
    let mut messages = vec![
        crate::ollama::ChatMessage {
            role: "system".to_string(),
            content: system_prompt.clone(),
        },
    ];
    
    // Add conversation history if provided (exclude system messages - we already have one)
    if let Some(ref history) = conversation_history {
        for msg in history {
            // Only add user and assistant messages from history (skip system messages)
            if msg.role == "user" || msg.role == "assistant" {
                messages.push(msg.clone());
            }
        }
        info!("Ollama Chat Continue: Added {} messages from conversation history", 
              history.iter().filter(|m| m.role == "user" || m.role == "assistant").count());
    }
    
    // Add the conversation flow for this code execution cycle
    messages.push(crate::ollama::ChatMessage {
        role: "user".to_string(),
        content: context_message.clone(),
    });
    messages.push(crate::ollama::ChatMessage {
        role: "assistant".to_string(),
        content: intermediate_response.clone(),
    });
    messages.push(crate::ollama::ChatMessage {
        role: "user".to_string(),
        content: follow_up_message,
    });
    
    let chat_request = ChatRequest {
        messages,
    };
    
    info!("Ollama Chat Continue: Sending follow-up to Ollama");
    let response = ollama_chat(chat_request).await
        .map_err(|e| format!("Failed to send follow-up: {}", e))?;
    
    let response_content = response.message.content;
    info!("Ollama Chat Continue: Received response ({} chars)", response_content.len());
    
    // Process response content - handle escaped newlines
    let mut processed_content = response_content.replace("\\n", "\n");
    // Remove "javascript\n" if present
    processed_content = processed_content.replace("javascript\n", "");
    
    // Check if Ollama is asking for more code execution (ping-pong)
    let trimmed = processed_content.trim();
    let is_code_assistant = trimmed.starts_with("ROLE=code-assistant") || 
                           trimmed.to_lowercase().starts_with("role=code-assistant");
    
    // Fallback: Detect JavaScript code patterns even without ROLE=code-assistant prefix
    let looks_like_javascript = if !is_code_assistant {
        let lower = trimmed.to_lowercase();
        lower.contains("console.log") ||
        lower.contains("new date()") ||
        lower.contains("document.") ||
        lower.contains("window.") ||
        lower.contains("function") ||
        lower.contains("=>") ||
        (lower.contains("(") && lower.contains(")") && 
         (lower.contains("tostring") || lower.contains("tolocaledate") || 
          lower.contains("tolocalestring") || lower.contains("getday") ||
          lower.contains("getdate") || lower.contains("getmonth") ||
          lower.contains("getfullyear")))
    } else {
        false
    };
    
    let needs_code_execution = is_code_assistant || looks_like_javascript;
    
    if needs_code_execution {
        if is_code_assistant {
            info!("Ollama Chat Continue: Detected another code-assistant response (ping-pong)");
        } else {
            info!("Ollama Chat Continue: Detected JavaScript code pattern (ping-pong, fallback detection)");
        }
        
        // Extract code
        let code = if is_code_assistant {
            // Extract code (everything after the first line if ROLE=code-assistant)
            let lines: Vec<&str> = processed_content.split('\n').collect();
            if lines.len() >= 2 {
                lines[1..].join("\n").trim().to_string()
            } else {
                processed_content.replace("ROLE=code-assistant", "").trim().to_string()
            }
        } else {
            // Use the entire content as code (no ROLE prefix)
            trimmed.to_string()
        };
        
        // Remove markdown code block markers
        let code = code.replace("```javascript", "")
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
        
        info!("Ollama Chat Continue: Extracted code for re-execution ({} chars):\n{}", code.len(), code);
        
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
