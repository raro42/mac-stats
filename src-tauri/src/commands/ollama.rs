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
        }
        Ok(false) => debug!("Ollama agent: endpoint not reachable at startup (will retry when used)"),
        Err(e) => debug!("Ollama agent: startup check failed: {}", e),
    }
}

/// Query GET /api/tags and return the first model name, or "llama3.2" as a fallback.
async fn detect_first_model(endpoint: &str, api_key: Option<&str>) -> String {
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

5. **REMOVE_SCHEDULE**: Remove a scheduled task by its ID. Use when the user asks to remove, delete, or cancel a schedule (e.g. "Remove schedule: discord-1770648842"). Reply with exactly one line: REMOVE_SCHEDULE: <schedule-id> (e.g. REMOVE_SCHEDULE: discord-1770648842)."#;

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

/// RUN_CMD agent description (appended when ALLOW_LOCAL_CMD is not 0). Call with agent number via format_run_cmd_description(n).
fn format_run_cmd_description(num: u32) -> String {
    format!(
        "\n\n{}. **RUN_CMD** (local read-only): Run a restricted local command. Use for: reading app data under ~/.mac-stats (schedules.json, config, task files), or current time/user (date, whoami). To invoke: reply with exactly one line: RUN_CMD: <command> [args] (e.g. RUN_CMD: cat ~/.mac-stats/schedules.json, RUN_CMD: date, RUN_CMD: whoami, RUN_CMD: date +%Y-%m-%d, RUN_CMD: grep pattern ~/.mac-stats/task/file.md). Allowed: cat, head, tail, ls, grep, date, whoami; file paths must be under ~/.mac-stats; date and whoami need no path.",
        num
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
Discord API (base: https://discord.com/api/v10). Use DISCORD_API: <METHOD> <path> [json body for POST]:
- GET /users/@me — current bot user
- GET /users/@me/guilds — list servers (guilds) the bot is in (optional ?with_counts=true)
- GET /guilds/{guild_id}/channels — list channels in a server
- GET /guilds/{guild_id}/members?limit=100 — list members (use after=user_id for pagination)
- GET /guilds/{guild_id}/members/search?query=name — search members by nickname/username
- GET /users/{user_id} — get user by ID
- GET /channels/{channel_id} — get channel
- POST /channels/{channel_id}/messages — send message (body: {"content":"..."})"#;

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
        "\n\n{}. **TASK** (task files under ~/.mac-stats/task/): Use when working on a task file or when the user asks for tasks. When the user wants agents to chat or have a conversation, invoke AGENT: orchestrator (or the right agent) so the conversation runs; do not only create a task. TASK_LIST: default is open and WIP only (reply: TASK_LIST or TASK_LIST: ). TASK_LIST: all — list all tasks grouped by status (reply: TASK_LIST: all when the user asks for all tasks). TASK_SHOW: <path or id> — show that task's content and status to the user so they can read and request updates. TASK_APPEND: append feedback (reply: TASK_APPEND: <path or task id> <content>). TASK_STATUS: set status (reply: TASK_STATUS: <path or task id> wip|finished|unsuccessful). When the user says \"close the task\", \"finish\", \"mark done\", or \"cancel\" a task, reply TASK_STATUS: <path or id> finished (success) or TASK_STATUS: <path or id> unsuccessful (failed) — do not use wip. TASK_CREATE: create a new task (reply: TASK_CREATE: <topic> <id> <initial content>). If a task with that topic and id already exists, use TASK_APPEND or TASK_STATUS instead. TASK_ASSIGN: <path or id> <agent_id> — reassign task to scheduler, discord, cpu, or default. Paths must be under ~/.mac-stats/task.",
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

/// Run a single Ollama request for an LLM agent (soul+mood+skill as system prompt, task as user message).
/// Uses the agent's model if set; otherwise default. No conversation history. Logs agent name/id.
/// Used by the tool loop (AGENT:) and by the agent-test CLI.
pub(crate) async fn run_agent_ollama_session(
    agent: &crate::agents::Agent,
    user_message: &str,
    _status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> Result<String, String> {
    use tracing::info;
    info!(
        "Agent: {} ({}) running (model: {:?}, prompt {} chars)",
        agent.name,
        agent.id,
        agent.model,
        agent.combined_prompt.chars().count()
    );
    let messages = vec![
        crate::ollama::ChatMessage {
            role: "system".to_string(),
            content: agent.combined_prompt.clone(),
        },
        crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: user_message.to_string(),
        },
    ];
    let response = send_ollama_chat_messages(messages, agent.model.clone(), None).await?;
    let out = response.message.content.trim().to_string();
    info!("Agent: {} ({}) returned ({} chars)", agent.name, agent.id, out.chars().count());
    Ok(out)
}

/// Shared API for Discord (and other agents): ask Ollama how to solve, then run agents (FETCH_URL, BRAVE_SEARCH, RUN_JS).
/// 1) Planning: send user question + agent list, get RECOMMEND: plan.
/// 2) Execution: send plan + "now answer using agents", loop on FETCH_URL / BRAVE_SEARCH / RUN_JS (max 5 tool calls).
/// If `status_tx` is provided (e.g. from Discord), short status messages are sent so the user sees we're still working.
/// If `discord_reply_channel_id` is set (when the request came from Discord), SCHEDULE will store it so the scheduler can post results to that channel (DM or mention channel).
/// When `discord_user_id` and `discord_user_name` are set (from Discord message author), the prompt is prefixed with "You are talking to Discord user **{name}** (user id: {id})."
/// When set, `model_override` and `options_override` apply only to this request (e.g. from Discord "model: llama3" line).
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
    let conversation_history: Vec<crate::ollama::ChatMessage> = conversation_history
        .unwrap_or_default()
        .into_iter()
        .rev()
        .take(CONVERSATION_HISTORY_CAP)
        .rev()
        .collect();
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
                String::new()
            } else {
                format!("{}\n\n", s)
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

    // --- Planning step: ask Ollama how it would solve the question ---
    info!("Agent router: planning step — asking Ollama for RECOMMEND");
    const PLANNING_PROMPT: &str = "You are a helpful assistant. We will give you a user question and a list of available agents. Reply with your recommended approach in this exact format: RECOMMEND: <your plan in one or two sentences: which agents to use and in what order>. Do not execute anything yet. If the user wants agents to have a conversation or chat together, your plan must start with AGENT: orchestrator (or the appropriate agent) so the conversation actually runs; do not only create a task file (TASK_CREATE).";
    let planning_system_content = match &skill_content {
        Some(skill) => format!(
            "{}Additional instructions from skill:\n\n{}\n\n---\n\n{}\n\n{}",
            discord_user_context, skill, PLANNING_PROMPT, agent_descriptions
        ),
        None => format!(
            "{}{}{}\n\n{}",
            router_soul, discord_user_context, PLANNING_PROMPT, agent_descriptions
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
    let recommendation = plan_response.message.content.trim().to_string();
    info!("Agent router: understood plan — RECOMMEND: {}", recommendation.chars().take(200).collect::<String>());
    send_status(&format!(
        "Executing plan: {}…",
        truncate_status(&recommendation, 72)
    ));

    // --- Execution: system prompt with agents + plan, then tool loop ---
    const EXECUTION_PROMPT: &str = "You are a helpful assistant. Use the agents when needed. When you need an agent, output exactly one line: FETCH_URL: <url> or BRAVE_SEARCH: <query> or RUN_JS: <code> or SKILL: <number or topic> [task] or AGENT: <slug or id> [task] (if LLM agents are listed) or RUN_CMD: <command and args> or OLLAMA_API: <action> [args] (list_models, version, running, pull/delete/load/unload, embed) or PYTHON_SCRIPT: <id> <topic> (then put Python code on next lines or in a ```python block) or SCHEDULE: every N minutes <task> or SCHEDULE: <cron> <task> or SCHEDULE: at <ISO datetime> <task> or REMOVE_SCHEDULE: <schedule-id> or TASK_LIST or TASK_LIST: all or TASK_SHOW: <id> (show task to user) or TASK_APPEND/TASK_STATUS/TASK_CREATE or MCP: <tool_name> <arguments> (if MCP tools are listed) or DISCORD_API: <METHOD> <path> (if from Discord). We will run it and give you the result. When an Agent (or other tool) result is given, your reply to the user MUST include or relay that result—e.g. list the agents, show the fetched content, or summarize the outcome. Do not reply with only a generic acknowledgment like \"Thank you for providing the information.\" Then continue with your answer. Answer concisely.";

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
                "{}Additional instructions from skill:\n\n{}\n\n---\n\n{}\n\n{}",
                discord_user_context, skill, EXECUTION_PROMPT, agent_descriptions
            ),
            None => format!(
                "{}{}{}\n\n{}",
                router_soul, discord_user_context, EXECUTION_PROMPT, agent_descriptions
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
                "{}Additional instructions from skill:\n\n{}\n\n---\n\n{}\n\n{}\n\nYour plan: {}",
                discord_user_context, skill, EXECUTION_PROMPT, agent_descriptions, recommendation
            ),
            None => format!(
                "{}{}{}\n\n{}\n\nYour plan: {}",
                router_soul, discord_user_context, EXECUTION_PROMPT, agent_descriptions, recommendation
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
        (msgs, content)
    };

    let mut tool_count: u32 = 0;
    // Collect (agent_name, reply) for each AGENT call so we can append a conversation transcript when multiple agents participated.
    let mut agent_conversation: Vec<(String, String)> = Vec::new();

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
            "RUN_CMD" => {
                if !crate::commands::run_cmd::is_local_cmd_allowed() {
                    "RUN_CMD is not available (disabled by ALLOW_LOCAL_CMD=0). Answer without running local commands.".to_string()
                } else {
                    send_status(&format!("Running local command: {}", arg));
                    info!("Agent router: RUN_CMD requested: {}", arg);
                    match tokio::task::spawn_blocking({
                        let arg = arg.to_string();
                        move || crate::commands::run_cmd::run_local_command(&arg)
                    })
                    .await
                    .map_err(|e| format!("RUN_CMD task: {}", e))
                    .and_then(|r| r)
                    {
                        Ok(output) => format!(
                            "Here is the command output:\n\n{}\n\nUse this to answer the user's question.",
                            output
                        ),
                        Err(e) => format!(
                            "RUN_CMD failed: {}. Answer the user's question without this result.",
                            e
                        ),
                    }
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
                let (path, body) = if let Some(idx) = rest.find(" {") {
                    let (p, b) = rest.split_at(idx);
                    (p.trim().to_string(), Some(b.trim().to_string()))
                } else {
                    (rest.to_string(), None)
                };
                if path.is_empty() {
                    "DISCORD_API requires: DISCORD_API: <METHOD> <path> or DISCORD_API: POST <path> {\"content\":\"...\"}.".to_string()
                } else {
                    let status_msg = format!("Calling Discord API: {} {}", method, path);
                    send_status(&status_msg);
                    info!("Discord API: {} {}", method, path);
                    match crate::discord::api::discord_api_request(&method, &path, body.as_deref()).await {
                        Ok(result) => format!(
                            "Discord API result:\n\n{}\n\nUse this to answer the user's question.",
                            result
                        ),
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
                            let task_label = path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or(path_or_id);
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
                                        Ok(new_path) => format!("Task status set to {} (file: {:?}).", status, new_path),
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
                        Ok(path) => format!("Task created: {:?}. Use TASK_APPEND and TASK_STATUS to update.", path),
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
                        Ok(path) => match crate::task::show_task_content(&path) {
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
                        },
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
                            if let Ok(new_path) = crate::task::set_task_status(&path, "paused") {
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

/// Parse one of FETCH_URL:, BRAVE_SEARCH:, RUN_JS:, SCHEDULE:/SCHEDULER:, MCP:, PYTHON_SCRIPT: from assistant content.
/// Also accepts lines starting with "RECOMMEND: " (e.g. "RECOMMEND: SCHEDULER: Every 5 minutes...").
/// Returns (tool_name, argument) or None.
fn parse_tool_from_response(content: &str) -> Option<(String, String)> {
    let prefixes = ["FETCH_URL:", "BRAVE_SEARCH:", "RUN_JS:", "SKILL:", "AGENT:", "RUN_CMD:", "SCHEDULE:", "SCHEDULER:", "REMOVE_SCHEDULE:", "TASK_LIST:", "TASK_SHOW:", "TASK_APPEND:", "TASK_STATUS:", "TASK_CREATE:", "TASK_ASSIGN:", "TASK_SLEEP:", "OLLAMA_API:", "PYTHON_SCRIPT:", "MCP:", "DISCORD_API:"];
    for line in content.lines() {
        let line = line.trim();
        // Ollama sometimes replies with just "TASK_LIST" (no colon); treat as tool call with empty arg.
        if line.eq_ignore_ascii_case("TASK_LIST") {
            return Some(("TASK_LIST".to_string(), String::new()));
        }
        // Ollama sometimes echoes the plan format: "RECOMMEND: RUN_JS: ..." or "RECOMMEND: SCHEDULER: ...".
        let search = if line.to_uppercase().starts_with("RECOMMEND: ") {
            line[11..].trim()
        } else {
            line
        };
        for prefix in prefixes {
            if search.to_uppercase().starts_with(prefix) {
                let mut arg = search[prefix.len()..].trim().to_string();
                if arg.is_empty() {
                    continue;
                }
                let tool_name = prefix.trim_end_matches(':');
                // Normalize SCHEDULER -> SCHEDULE
                let tool_name = if tool_name.eq_ignore_ascii_case("SCHEDULER") {
                    "SCHEDULE".to_string()
                } else {
                    tool_name.to_string()
                };
                // Ollama sometimes concatenates multiple tools on one line; for FETCH_URL and BRAVE_SEARCH, stop at first ';'.
                if tool_name == "FETCH_URL" || tool_name == "BRAVE_SEARCH" {
                    if let Some(idx) = arg.find(';') {
                        arg = arg[..idx].trim().to_string();
                    }
                }
                if !arg.is_empty() || tool_name == "TASK_LIST" || tool_name == "TASK_SHOW" {
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
    "FETCH_URL:", "BRAVE_SEARCH:", "RUN_JS:", "SKILL:", "AGENT:", "RUN_CMD:", "SCHEDULE:", "SCHEDULER:", "REMOVE_SCHEDULE:",
    "TASK_LIST:", "TASK_SHOW:", "TASK_APPEND:", "TASK_STATUS:", "TASK_CREATE:", "TASK_ASSIGN:", "TASK_SLEEP:", "OLLAMA_API:", "MCP:", "PYTHON_SCRIPT:", "DISCORD_API:",
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
