//! MCP (Model Context Protocol) client for Ollama agent.
//!
//! Supports two transports: HTTP/SSE (remote) and stdio (local subprocess).
//! Config: MCP_SERVER_URL (HTTP/SSE) or MCP_SERVER_STDIO (e.g. npx|-y|@openbnb/mcp-server-airbnb).
//! See docs/010_mcp_agent.md.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use tracing::{info, warn};

/// One MCP tool (name + description for agent list).
#[derive(Debug, Clone)]
pub struct McpTool {
    pub name: String,
    pub description: Option<String>,
}

/// Read a key from .config.env (value after =).
fn mcp_value_from_config_env(path: &Path, key: &str, key_alt: &str) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let line = content.lines().find(|l| {
        let t = l.trim();
        t.starts_with(key) || t.starts_with(key_alt)
    })?;
    let (_, v) = line.split_once('=')?;
    let v = v.trim().to_string();
    if v.is_empty() {
        return None;
    }
    Some(v)
}

fn read_mcp_config_file(path: &Path) -> (Option<String>, Option<String>) {
    let url = mcp_value_from_config_env(path, "MCP_SERVER_URL=", "MCP-SERVER-URL=");
    let stdio = mcp_value_from_config_env(path, "MCP_SERVER_STDIO=", "MCP-SERVER-STDIO=");
    (url, stdio)
}

/// Get MCP server config: either stdio or HTTP/SSE.
/// Returns Some("stdio:cmd|arg1|arg2") if MCP_SERVER_STDIO is set, else Some(url) if MCP_SERVER_URL is set.
/// Checked: env MCP_SERVER_STDIO / MCP_SERVER_URL, then .config.env (cwd, src-tauri, ~/.mac-stats).
pub fn get_mcp_server_url() -> Option<String> {
    if let Ok(s) = std::env::var("MCP_SERVER_STDIO") {
        let s = s.trim().to_string();
        if !s.is_empty() {
            return Some(format!("stdio:{}", s));
        }
    }
    if let Ok(u) = std::env::var("MCP_SERVER_URL") {
        let u = u.trim().to_string();
        if !u.is_empty() {
            return Some(u);
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        let p = cwd.join(".config.env");
        if p.is_file() {
            let (url, stdio) = read_mcp_config_file(&p);
            if let Some(s) = stdio {
                return Some(format!("stdio:{}", s));
            }
            if let Some(u) = url {
                return Some(u);
            }
        }
        let p_src = cwd.join("src-tauri").join(".config.env");
        if p_src.is_file() {
            let (url, stdio) = read_mcp_config_file(&p_src);
            if let Some(s) = stdio {
                return Some(format!("stdio:{}", s));
            }
            if let Some(u) = url {
                return Some(u);
            }
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let p = Path::new(&home).join(".mac-stats").join(".config.env");
        if p.is_file() {
            let (url, stdio) = read_mcp_config_file(&p);
            if let Some(s) = stdio {
                return Some(format!("stdio:{}", s));
            }
            if let Some(u) = url {
                return Some(u);
            }
        }
    }
    None
}

/// Parse "stdio:cmd|arg1|arg2" into (command, args). Returns None if not a stdio spec.
fn parse_stdio_spec(server_config: &str) -> Option<(String, Vec<String>)> {
    let rest = server_config.strip_prefix("stdio:")?;
    let parts: Vec<String> = rest.split('|').map(|s| s.trim().to_string()).collect();
    if parts.is_empty() || parts[0].is_empty() {
        return None;
    }
    let cmd = parts[0].clone();
    let args = parts[1..].to_vec();
    Some((cmd, args))
}

/// Mask URL for logs (show host only).
fn mask_url_for_log(url: &str) -> String {
    if url.len() <= 40 {
        return url.to_string();
    }
    if let Ok(u) = url::Url::parse(url) {
        let host = u.host_str().unwrap_or("?");
        format!("{}... (host: {})", &url[..20.min(url.len())], host)
    } else {
        format!("{}...", &url[..24.min(url.len())])
    }
}

#[derive(Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    id: u32,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Clone)]
struct JsonRpcResponse {
    id: Option<u32>,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize, Clone)]
struct JsonRpcError {
    code: i32,
    message: String,
}

#[derive(Debug, Deserialize)]
struct ToolsListResult {
    tools: Vec<McpToolDef>,
    #[allow(dead_code)]
    next_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
struct McpToolDef {
    name: String,
    description: Option<String>,
    #[allow(dead_code)]
    input_schema: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct ToolCallResult {
    content: Option<Vec<ContentItem>>,
    #[serde(default)]
    is_error: bool,
}

#[derive(Debug, Deserialize)]
struct ContentItem {
    #[allow(dead_code)]
    r#type: Option<String>,
    text: Option<String>,
}

static RPC_ID: AtomicU32 = AtomicU32::new(1);

fn next_id() -> u32 {
    RPC_ID.fetch_add(1, Ordering::Relaxed)
}

/// Parse one SSE event from buffer (up to first "\n\n"). Returns (event_type, data, rest).
fn parse_one_sse_event(buf: &str) -> Option<(Option<String>, Option<String>, &str)> {
    let end = buf.find("\n\n")?;
    let block = &buf[..end];
    let rest = buf.get((end + 2)..).unwrap_or("");
    let mut event = None;
    let mut data_lines = Vec::new();
    for line in block.lines() {
        if line.starts_with("event:") {
            event = Some(line[6..].trim().to_string());
        } else if line.starts_with("data:") {
            data_lines.push(line[5..].trim());
        }
    }
    let data = if data_lines.is_empty() {
        None
    } else {
        Some(data_lines.join("\n"))
    };
    Some((event, data, rest))
}

/// Run MCP session: connect SSE, init, then run a single RPC (tools/list or tools/call) and return the JSON-RPC response.
async fn run_mcp_rpc(
    sse_url: &str,
    method: &str,
    params: Option<serde_json::Value>,
    client: &reqwest::Client,
) -> Result<JsonRpcResponse, String> {
    use futures_util::StreamExt;
    use tokio::sync::mpsc;

    info!(
        "MCP: connecting to SSE ({}) for {}",
        mask_url_for_log(sse_url),
        method
    );
    let resp = client
        .get(sse_url)
        .header("Accept", "text/event-stream")
        .send()
        .await
        .map_err(|e| format!("MCP SSE connect: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!(
            "MCP SSE HTTP {}: {}",
            resp.status().as_u16(),
            resp.status().canonical_reason().unwrap_or("")
        ));
    }
    let (tx_endpoint, mut rx_endpoint) = mpsc::unbounded_channel::<String>();
    let (tx_message, mut rx_message) = mpsc::unbounded_channel::<JsonRpcResponse>();
    let mut stream = resp.bytes_stream();
    let mut buf = String::new();
    let read_task = tokio::spawn(async move {
        while let Some(chunk) = stream.next().await {
            if let Ok(bytes) = chunk {
                if let Ok(s) = String::from_utf8(bytes.to_vec()) {
                    buf.push_str(&s);
                    while let Some((event, data, rest)) = parse_one_sse_event(&buf) {
                        buf = rest.to_string();
                        if event.as_deref() == Some("endpoint") {
                            let url = data.unwrap_or_default().trim().to_string();
                            if !url.is_empty() {
                                let _ = tx_endpoint.send(url);
                            }
                        } else if event.as_deref() == Some("message") {
                            if let Some(d) = data {
                                if !d.is_empty() {
                                    if let Ok(j) = serde_json::from_str::<JsonRpcResponse>(&d) {
                                        let _ = tx_message.send(j);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    let message_endpoint = tokio::time::timeout(Duration::from_secs(10), rx_endpoint.recv())
        .await
        .map_err(|_| "MCP: timeout waiting for endpoint event".to_string())?
        .ok_or_else(|| "MCP: endpoint channel closed".to_string())?;
    info!("MCP: got message endpoint, sending initialize");

    let init_id = next_id();
    let init_req = JsonRpcRequest {
        jsonrpc: "2.0",
        id: init_id,
        method: "initialize".to_string(),
        params: Some(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "mac-stats", "version": "0.1.0" }
        })),
    };
    let body = serde_json::to_string(&init_req).map_err(|e| format!("MCP JSON: {}", e))?;
    client
        .post(&message_endpoint)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|e| format!("MCP initialize POST: {}", e))?;

    let init_response = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let msg = rx_message.recv().await.ok_or("channel closed")?;
            if msg.id == Some(init_id) {
                return Ok::<_, String>(msg);
            }
        }
    })
    .await
    .map_err(|_| "MCP: timeout waiting for initialize response".to_string())??;
    if let Some(ref e) = init_response.error {
        return Err(format!("MCP initialize error: {} ({})", e.message, e.code));
    }

    let notif = serde_json::json!({"jsonrpc":"2.0","method":"notifications/initialized"});
    let _ = client
        .post(&message_endpoint)
        .header("Content-Type", "application/json")
        .body(notif.to_string())
        .send()
        .await;

    let rpc_id = next_id();
    let req = JsonRpcRequest {
        jsonrpc: "2.0",
        id: rpc_id,
        method: method.to_string(),
        params,
    };
    let body = serde_json::to_string(&req).map_err(|e| format!("MCP JSON: {}", e))?;
    info!("MCP: sending {} (id={})", method, rpc_id);
    client
        .post(&message_endpoint)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|e| format!("MCP {} POST: {}", method, e))?;

    let rpc_response = tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            let msg = rx_message.recv().await.ok_or("channel closed")?;
            if msg.id == Some(rpc_id) {
                return Ok::<_, String>(msg);
            }
        }
    })
    .await
    .map_err(|_| format!("MCP: timeout waiting for {} response", method))??;

    read_task.abort();
    Ok(rpc_response)
}

/// Run one JSON-RPC request against a stdio MCP server (spawn process, init, send one method, return response).
async fn run_mcp_stdio_rpc(
    command: &str,
    args: &[String],
    method: &str,
    params: Option<serde_json::Value>,
) -> Result<JsonRpcResponse, String> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::process::Command;

    info!("MCP stdio: spawning {} {:?} for {}", command, args, method);
    let mut child = Command::new(command)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("MCP stdio spawn: {}", e))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "MCP stdio: no stdin".to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "MCP stdio: no stdout".to_string())?;
    let mut reader = BufReader::new(stdout).lines();

    let init_id = next_id();
    let init_req = JsonRpcRequest {
        jsonrpc: "2.0",
        id: init_id,
        method: "initialize".to_string(),
        params: Some(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "mac-stats", "version": "0.1.0" }
        })),
    };
    let init_line = serde_json::to_string(&init_req).map_err(|e| format!("MCP JSON: {}", e))?;
    stdin
        .write_all(format!("{}\n", init_line).as_bytes())
        .await
        .map_err(|e| format!("MCP stdio write: {}", e))?;
    stdin.flush().await.map_err(|e| format!("MCP stdio flush: {}", e))?;

    let init_response = loop {
        let line = tokio::time::timeout(Duration::from_secs(15), reader.next_line())
            .await
            .map_err(|_| "MCP stdio: timeout waiting for initialize response".to_string())?
            .map_err(|e| format!("MCP stdio read: {}", e))?
            .ok_or_else(|| "MCP stdio: stdout closed".to_string())?;
        if let Ok(j) = serde_json::from_str::<JsonRpcResponse>(&line) {
            if j.id == Some(init_id) {
                break j;
            }
        }
    };
    if let Some(ref e) = init_response.error {
        return Err(format!("MCP initialize error: {} ({})", e.message, e.code));
    }

    let notif = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    stdin
        .write_all(format!("{}\n", notif).as_bytes())
        .await
        .map_err(|e| format!("MCP stdio write: {}", e))?;
    stdin.flush().await.map_err(|e| format!("MCP stdio flush: {}", e))?;

    let rpc_id = next_id();
    let req = JsonRpcRequest {
        jsonrpc: "2.0",
        id: rpc_id,
        method: method.to_string(),
        params,
    };
    let req_line = serde_json::to_string(&req).map_err(|e| format!("MCP JSON: {}", e))?;
    stdin
        .write_all(format!("{}\n", req_line).as_bytes())
        .await
        .map_err(|e| format!("MCP stdio write: {}", e))?;
    stdin.flush().await.map_err(|e| format!("MCP stdio flush: {}", e))?;

    let rpc_response = loop {
        let line = tokio::time::timeout(Duration::from_secs(20), reader.next_line())
            .await
            .map_err(|_| format!("MCP stdio: timeout waiting for {} response", method))?
            .map_err(|e| format!("MCP stdio read: {}", e))?
            .ok_or_else(|| "MCP stdio: stdout closed".to_string())?;
        if let Ok(j) = serde_json::from_str::<JsonRpcResponse>(&line) {
            if j.id == Some(rpc_id) {
                break j;
            }
        }
    };

    drop(stdin);
    let _ = child.wait().await;
    Ok(rpc_response)
}

/// List tools from the MCP server.
pub async fn list_tools(server_url: &str) -> Result<Vec<McpTool>, String> {
    if let Some((cmd, args)) = parse_stdio_spec(server_url) {
        info!("MCP: listing tools from stdio server {}", cmd);
        let response =
            run_mcp_stdio_rpc(&cmd, &args, "tools/list", Some(serde_json::json!({}))).await?;
        if let Some(ref e) = response.error {
            return Err(format!("MCP tools/list error: {} ({})", e.message, e.code));
        }
        let result = response
            .result
            .ok_or_else(|| "MCP: no result in tools/list".to_string())?;
        let list_result: ToolsListResult =
            serde_json::from_value(result.clone()).map_err(|e| format!("MCP tools/list result: {}", e))?;
        let tools: Vec<McpTool> = list_result
            .tools
            .into_iter()
            .map(|t| McpTool {
                name: t.name,
                description: t.description,
            })
            .collect();
        info!("MCP: listed {} tools (stdio)", tools.len());
        return Ok(tools);
    }

    info!("MCP: listing tools from {}", mask_url_for_log(server_url));
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("MCP HTTP client: {}", e))?;
    let response = run_mcp_rpc(server_url, "tools/list", Some(serde_json::json!({})), &client).await?;
    if let Some(ref e) = response.error {
        return Err(format!("MCP tools/list error: {} ({})", e.message, e.code));
    }
    let result = response
        .result
        .ok_or_else(|| "MCP: no result in tools/list".to_string())?;
    let list_result: ToolsListResult =
        serde_json::from_value(result.clone()).map_err(|e| format!("MCP tools/list result: {}", e))?;
    let tools: Vec<McpTool> = list_result
        .tools
        .into_iter()
        .map(|t| McpTool {
            name: t.name,
            description: t.description,
        })
        .collect();
    info!("MCP: listed {} tools", tools.len());
    Ok(tools)
}

/// Call an MCP tool by name with optional JSON arguments. Returns the tool result as text.
pub async fn call_tool(server_url: &str, tool_name: &str, arguments: Option<serde_json::Value>) -> Result<String, String> {
    let params = Some(serde_json::json!({
        "name": tool_name,
        "arguments": arguments.unwrap_or(serde_json::json!({}))
    }));

    if let Some((cmd, args)) = parse_stdio_spec(server_url) {
        info!("MCP: calling tool {} on stdio server {}", tool_name, cmd);
        let response = run_mcp_stdio_rpc(&cmd, &args, "tools/call", params).await?;
        if let Some(ref e) = response.error {
            return Err(format!("MCP tools/call error: {} ({})", e.message, e.code));
        }
        let result = response
            .result
            .ok_or_else(|| "MCP: no result in tools/call".to_string())?;
        let call_result: ToolCallResult =
            serde_json::from_value(result).map_err(|e| format!("MCP tools/call result: {}", e))?;
        if call_result.is_error {
            let msg = call_result
                .content
                .as_ref()
                .and_then(|c| c.first())
                .and_then(|i| i.text.as_deref())
                .unwrap_or("Unknown error");
            warn!("MCP: tool {} returned error: {}", tool_name, msg);
            return Err(msg.to_string());
        }
        let text = call_result
            .content
            .unwrap_or_default()
            .into_iter()
            .filter_map(|i| i.text)
            .collect::<Vec<_>>()
            .join("\n");
        info!("MCP: tool {} completed ({} chars, stdio)", tool_name, text.len());
        return Ok(if text.is_empty() { "(no output)".to_string() } else { text });
    }

    info!(
        "MCP: calling tool {} on {}",
        tool_name,
        mask_url_for_log(server_url)
    );
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("MCP HTTP client: {}", e))?;
    let response = run_mcp_rpc(server_url, "tools/call", params, &client).await?;
    if let Some(ref e) = response.error {
        return Err(format!("MCP tools/call error: {} ({})", e.message, e.code));
    }
    let result = response
        .result
        .ok_or_else(|| "MCP: no result in tools/call".to_string())?;
    let call_result: ToolCallResult =
        serde_json::from_value(result).map_err(|e| format!("MCP tools/call result: {}", e))?;
    if call_result.is_error {
        let msg = call_result
            .content
            .as_ref()
            .and_then(|c| c.first())
            .and_then(|i| i.text.as_deref())
            .unwrap_or("Unknown error");
        warn!("MCP: tool {} returned error: {}", tool_name, msg);
        return Err(msg.to_string());
    }
    let text = call_result
        .content
        .unwrap_or_default()
        .into_iter()
        .filter_map(|i| i.text)
        .collect::<Vec<_>>()
        .join("\n");
    info!("MCP: tool {} completed ({} chars)", tool_name, text.len());
    Ok(if text.is_empty() { "(no output)".to_string() } else { text })
}
