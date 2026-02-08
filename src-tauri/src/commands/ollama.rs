//! Ollama Tauri commands

use crate::ollama::{OllamaClient, OllamaConfig, ChatMessage};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::process::Command;
use std::sync::Mutex;
use std::sync::OnceLock;

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

/// Internal: send messages to Ollama and return the chat response.
/// Used by the ollama_chat command and by answer_with_ollama_and_fetch (Discord / agent).
pub async fn send_ollama_chat_messages(
    messages: Vec<crate::ollama::ChatMessage>,
) -> Result<crate::ollama::ChatResponse, String> {
    use tracing::{debug, info};

    let (endpoint, model, api_key) = {
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
        )
    };

    let temp_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let url = format!("{}/api/chat", endpoint);
    let chat_request = crate::ollama::ChatRequest {
        model: model.clone(),
        messages,
        stream: false,
    };

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
        .map_err(|e| format!("Failed to send chat request: {}", e))?
        .json::<crate::ollama::ChatResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    let content = &response.message.content;
    let n = content.chars().count();
    const LOG_MAX: usize = 500;
    if n <= LOG_MAX {
        info!("Ollama: Chat response received ({} chars): {}", n, content);
    } else {
        let head: String = content.chars().take(LOG_MAX).collect();
        info!("Ollama: Chat response received ({} chars): {}...", n, head);
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

    send_ollama_chat_messages(request.messages).await
}

/// Base agent descriptions (without MCP). Includes RUN_JS, FETCH_URL, BRAVE_SEARCH, SCHEDULE.
const AGENT_DESCRIPTIONS_BASE: &str = r#"We have 4 agents available:

1. **RUN_JS** (JavaScript superpowers): Execute JavaScript in the app context (e.g. browser console). Use for: dynamic data, DOM inspection, client-side state. To invoke: reply with exactly one line: RUN_JS: <JavaScript code>. Note: In some contexts (e.g. Discord) JS is not executed; then answer without running code.

2. **FETCH_URL**: Fetch the full text of a web page. Use for: reading a specific URL's content. To invoke: reply with exactly one line: FETCH_URL: <full URL> (e.g. FETCH_URL: https://www.example.com). The app will return the page text.

3. **BRAVE_SEARCH**: Web search via Brave Search API. Use for: finding current info, facts, multiple sources. To invoke: reply with exactly one line: BRAVE_SEARCH: <search query>. The app will return search results.

4. **SCHEDULE** (scheduler): Add a task to run at scheduled times. Use when the user wants something to run later or repeatedly (e.g. every 5 minutes, daily). To invoke: reply with exactly one line: SCHEDULE: every N minutes <task description> (e.g. SCHEDULE: every 5 minutes Execute RUN_JS to fetch CPU and RAM). Or SCHEDULE: <cron expression> <task>. We will add it to ~/.mac-stats/schedules.json and confirm to the user."#;

/// RUN_CMD agent description (appended when ALLOW_LOCAL_CMD is not 0).
const RUN_CMD_DESCRIPTION: &str = r#"

5. **RUN_CMD** (local read-only): Run a restricted local command to read app data under ~/.mac-stats. Use for: reading schedules.json, config, or other files in ~/.mac-stats. To invoke: reply with exactly one line: RUN_CMD: <command> [args] (e.g. RUN_CMD: cat ~/.mac-stats/schedules.json or RUN_CMD: ls ~/.mac-stats). Only cat, head, tail, ls are allowed; paths must be under ~/.mac-stats."#;

/// Build agent descriptions string: base, optional RUN_CMD when allowed, then MCP tools when configured.
async fn build_agent_descriptions() -> String {
    use tracing::info;
    let mut base = AGENT_DESCRIPTIONS_BASE.to_string();
    if crate::commands::run_cmd::is_local_cmd_allowed() {
        base.push_str(RUN_CMD_DESCRIPTION);
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
            let mcp_num = if crate::commands::run_cmd::is_local_cmd_allowed() {
                6
            } else {
                5
            };
            let mut mcp_section = format!("\n\n{}. **MCP** (tools from configured MCP server, {} tools): Use when the task matches a tool below. To invoke: reply with exactly one line: MCP: <tool_name> <arguments>. Arguments can be JSON (e.g. MCP: get_weather {{\"location\": \"NYC\"}}) or plain text (e.g. MCP: fetch_url https://example.com).\n\nAvailable MCP tools:\n", mcp_num, tools.len());
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

/// Shared API for Discord (and other agents): ask Ollama how to solve, then run agents (FETCH_URL, BRAVE_SEARCH, RUN_JS).
/// 1) Planning: send user question + agent list, get RECOMMEND: plan.
/// 2) Execution: send plan + "now answer using agents", loop on FETCH_URL / BRAVE_SEARCH / RUN_JS (max 5 tool calls).
/// If `status_tx` is provided (e.g. from Discord), short status messages are sent so the user sees we're still working.
pub async fn answer_with_ollama_and_fetch(
    question: &str,
    status_tx: Option<tokio::sync::mpsc::UnboundedSender<String>>,
) -> Result<String, String> {
    use tracing::info;

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
    send_status("Thinking…");

    let agent_descriptions = build_agent_descriptions().await;
    info!("Agent router: agent list built ({} chars)", agent_descriptions.len());

    // --- Planning step: ask Ollama how it would solve the question ---
    info!("Agent router: planning step — asking Ollama for RECOMMEND");
    const PLANNING_PROMPT: &str = "You are a helpful assistant. We will give you a user question and a list of available agents. Reply with your recommended approach in this exact format: RECOMMEND: <your plan in one or two sentences: which agents to use and in what order>. Do not execute anything yet.";
    let planning_messages = vec![
        crate::ollama::ChatMessage {
            role: "system".to_string(),
            content: format!("{}\n\n{}\n\nUser question: {}", PLANNING_PROMPT, agent_descriptions, question),
        },
        crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: "Reply with RECOMMEND: your plan.".to_string(),
        },
    ];
    let plan_response = send_ollama_chat_messages(planning_messages).await?;
    let recommendation = plan_response.message.content.trim().to_string();
    info!("Agent router: understood plan — RECOMMEND: {}", recommendation.chars().take(200).collect::<String>());
    send_status("Working on it…");

    // --- Execution: system prompt with agents + plan, then tool loop ---
    info!("Agent router: execution step — sending plan + question, starting tool loop (max 5 tools)");
    const EXECUTION_PROMPT: &str = "You are a helpful assistant. Use the agents when needed. When you need an agent, output exactly one line: FETCH_URL: <url> or BRAVE_SEARCH: <query> or RUN_JS: <code> or RUN_CMD: <command and args> or SCHEDULE: every N minutes <task> or MCP: <tool_name> <arguments> (if MCP tools are listed). We will run it and give you the result. Then continue with your answer. Answer concisely.";
    let mut messages = vec![
        crate::ollama::ChatMessage {
            role: "system".to_string(),
            content: format!(
                "{}\n\n{}\n\nYour plan: {}",
                EXECUTION_PROMPT,
                agent_descriptions,
                recommendation
            ),
        },
        crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: question.to_string(),
        },
    ];

    /// Log content in full if ≤500 chars, else first 500 + truncation note.
    const LOG_CONTENT_MAX: usize = 500;
    let log_content = |content: &str| {
        let n = content.chars().count();
        if n <= LOG_CONTENT_MAX {
            content.to_string()
        } else {
            format!(
                "{}... [truncated, {} chars total]",
                content.chars().take(LOG_CONTENT_MAX).collect::<String>(),
                n
            )
        }
    };

    const MAX_TOOL_ITERATIONS: u32 = 5;
    let mut tool_count: u32 = 0;
    let mut response = send_ollama_chat_messages(messages.clone()).await?;
    let mut response_content = response.message.content.clone();
    let n_first = response_content.chars().count();
    info!("Agent router: first response received ({} chars): {}", n_first, log_content(&response_content));

    while tool_count < MAX_TOOL_ITERATIONS {
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
        info!("Agent router: running tool {}/{} — {}", tool_count, MAX_TOOL_ITERATIONS, tool);

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
                    Ok(body) => format!(
                        "Here is the page content:\n\n{}\n\nPlease answer the user's question based on this content.",
                        body
                    ),
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
                send_status("Running code…");
                info!("Discord/Ollama: RUN_JS requested: {}... [{} chars]", arg.chars().take(60).collect::<String>(), arg.len());
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
            "SCHEDULE" => {
                send_status("Scheduling…");
                info!("Agent router: SCHEDULE requested (arg len={})", arg.chars().count());
                match parse_schedule_arg(&arg) {
                    Ok((cron_str, task)) => {
                        let id = format!("discord-{}", chrono::Utc::now().timestamp());
                        match crate::scheduler::add_schedule(id.clone(), cron_str.clone(), task.clone()) {
                            Ok(()) => {
                                info!("Agent router: SCHEDULE added (id={}, cron={})", id, cron_str);
                                let task_preview: String = task.chars().take(100).collect();
                                format!(
                                    "Schedule added successfully. The scheduler will run this task (cron: {}): \"{}\". Tell the user in your reply that it is scheduled and they will see it in the scheduler.",
                                    cron_str,
                                    task_preview.trim()
                                )
                            }
                            Err(e) => {
                                info!("Agent router: SCHEDULE failed: {}", e);
                                format!("Failed to add schedule: {}. Tell the user and suggest they check ~/.mac-stats/schedules.json.", e)
                            }
                        }
                    }
                    Err(e) => {
                        info!("Agent router: SCHEDULE parse failed: {}", e);
                        format!("Could not parse schedule (expected e.g. \"every 5 minutes <task>\"): {}. Ask the user to rephrase.", e)
                    }
                }
            }
            "RUN_CMD" => {
                if !crate::commands::run_cmd::is_local_cmd_allowed() {
                    "RUN_CMD is not available (disabled by ALLOW_LOCAL_CMD=0). Answer without running local commands.".to_string()
                } else {
                    let status_preview: String = arg.chars().take(40).collect();
                    let status_msg = if arg.chars().count() > 40 {
                        format!("Running local command: {}…", status_preview)
                    } else {
                        format!("Running local command: {}", arg)
                    };
                    send_status(&status_msg);
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
        messages.push(crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: user_message,
        });

        response = send_ollama_chat_messages(messages.clone()).await?;
        response_content = response.message.content.clone();
        if tool_count >= MAX_TOOL_ITERATIONS {
            info!("Agent router: max tool iterations reached ({}), using last response as final", MAX_TOOL_ITERATIONS);
        }
    }

    let final_len = response_content.chars().count();
    info!("Agent router: done after {} tool(s), returning final response ({} chars): {}", tool_count, final_len, log_content(&response_content));
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

/// Parse SCHEDULE argument into (cron_str, task). Supports "every N minutes <task>" (case insensitive).
/// Returns (cron_expression, task_text). Task is the full arg so the scheduler runs it as the Ollama question.
fn parse_schedule_arg(arg: &str) -> Result<(String, String), String> {
    let lower = arg.to_lowercase();
    let rest = lower.trim_start();
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
        // Cron: every N minutes = "0 */N * * * *" (second 0, every N minutes)
        let cron_str = format!("0 */{} * * * *", n);
        let task = arg.trim().to_string();
        Ok((cron_str, task))
    } else {
        // Optional: allow raw cron at start (e.g. "0 */5 * * * * Task here"). For now only "every N minutes".
        Err("expected 'every N minutes' (e.g. SCHEDULE: every 5 minutes Execute RUN_JS to fetch CPU)".to_string())
    }
}

/// Parse one of FETCH_URL:, BRAVE_SEARCH:, RUN_JS:, SCHEDULE:/SCHEDULER:, MCP: from assistant content.
/// Also accepts lines starting with "RECOMMEND: " (e.g. "RECOMMEND: SCHEDULER: Every 5 minutes...").
/// Returns (tool_name, argument) or None.
fn parse_tool_from_response(content: &str) -> Option<(String, String)> {
    let prefixes = ["FETCH_URL:", "BRAVE_SEARCH:", "RUN_JS:", "RUN_CMD:", "SCHEDULE:", "SCHEDULER:", "MCP:"];
    for line in content.lines() {
        let line = line.trim();
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
                if !arg.is_empty() {
                    return Some((tool_name, arg));
                }
            }
        }
    }
    None
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
    info!("Ollama JS Execution: Response content (first 500 chars):\n{}", 
          response_content.chars().take(500).collect::<String>());
    
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
    
    // Get system prompt (includes FETCH_URL tool for web navigation)
    let system_prompt = request.system_prompt.unwrap_or_else(|| {
        let base = "You are a general purpose AI. If you are asked for actual data like day or weather information, or flight information or stock information. Then we need to compile that information using speciallz crafted clients for doing so. You will put \"[variable-name]\" into the answer to signal that we need to go another step and ask and agent to fullfill the answer.\n\nWhenever asked with \"[variable-name]\", you must provide a javascript snipplet to be executed in the browser console to retrieve that information. Mark the answer to be executed as javascript. Do not put any other words around it. Do not insert formatting. Onlz return the code to be executed. This is needed for the next AI to understand and execute the same. When answering, use the role: code-assistant in the response. When you return executable code:\n- Start the response with: ROLE=code-assistant\n- On the next line, output ONLY executable JavaScript\n- Do not add explanations or formatting";
        let fetch_tool = "\n\nFor web pages: To fetch a page and use its content (e.g. \"navigate to X and get Y\"), reply with exactly one line: FETCH_URL: <full URL> (e.g. FETCH_URL: https://www.example.com). The app will fetch the page and give you the text; then answer the user based on that.";
        format!("{}{}", base, fetch_tool)
    });
    
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
    
    info!("Ollama Chat Continue: Code executed, result: {}", execution_result);
    
    let system_prompt = system_prompt.unwrap_or_else(|| {
        "You are a helpful assistant that answers questions about system metrics and monitoring.".to_string()
    });
    
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
