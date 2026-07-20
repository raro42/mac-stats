//! Miscellaneous tool dispatch handlers for the agent router tool loop.
//!
//! Contains: OLLAMA_API, MCP, CURSOR_AGENT, MASTODON_POST.
//! MEMORY lives in `curated_memory` (Hermes-style add/replace/remove).
//! Extracted from `commands/ollama.rs` to keep modules small and cohesive.

use tracing::info;

use crate::commands::ollama_models::{
    delete_ollama_model, get_ollama_version, list_ollama_models_full, list_ollama_running_models,
    load_ollama_model, ollama_embeddings, pull_ollama_model, unload_ollama_model,
};
use crate::commands::reply_helpers::mastodon_post;

fn send_status(tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>, msg: &str) {
    if let Some(tx) = tx {
        let _ = tx.send(msg.to_string());
    }
}

/// Short, non-leaky hints when stdio MCP fails (e.g. missing `ori`, timeouts).
fn mcp_stdio_troubleshooting_hint(server_url: &str, err: &str) -> &'static str {
    if !server_url.starts_with("stdio:") {
        return "";
    }
    let lower = err.to_lowercase();
    if lower.contains("mcp stdio spawn") || lower.contains("no such file") {
        return " Hint: ensure the MCP command is on PATH when mac-stats starts, or use an absolute path in MCP_SERVER_STDIO. For Ori, run `ori health` in Terminal. See docs/038_ori_mnemos_mcp.md.";
    }
    if lower.contains("timeout") {
        return " Hint: stdio MCP starts a new process per call; slow servers or large vaults may time out. See docs/038_ori_mnemos_mcp.md.";
    }
    if lower.contains("initialize error")
        || lower.contains("tools/list error")
        || lower.contains("tools/call error")
    {
        return " Hint: check MCP_SERVER_STDIO and vault path; see docs/038_ori_mnemos_mcp.md troubleshooting.";
    }
    ""
}

pub(crate) async fn handle_ollama_api(
    arg: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
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
    send_status(status_tx, &status_detail);
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

pub(crate) async fn handle_mcp(
    arg: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    send_status(status_tx, "Calling MCP tool…");
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
                (arg.to_string(), None)
            };
            match crate::mcp::call_tool(&server_url, &mcp_tool_name, mcp_args).await {
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
                    info!("Agent router: MCP tool {} failed: {}", mcp_tool_name, e);
                    let hint = mcp_stdio_troubleshooting_hint(&server_url, &e);
                    format!(
                        "MCP tool \"{}\" failed: {}. Answer the user without this result.{}",
                        mcp_tool_name, e, hint
                    )
                }
            }
        }
        None => {
            info!("Agent router: MCP not configured (no MCP_SERVER_URL or MCP_SERVER_STDIO)");
            "MCP is not configured (set MCP_SERVER_URL for HTTP/SSE or MCP_SERVER_STDIO for a local server in env or ~/.mac-stats/.config.env). Answer without using MCP.".to_string()
        }
    }
}

pub(crate) async fn handle_cursor_agent(
    arg: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    if !crate::commands::cursor_agent::is_cursor_agent_available() {
        return "CURSOR_AGENT is not available (cursor-agent CLI not found on PATH). Answer without it.".to_string();
    }
    let prompt = arg.trim().to_string();
    if prompt.is_empty() {
        return "CURSOR_AGENT requires a prompt: CURSOR_AGENT: <detailed coding task>".to_string();
    }
    let preview: String = prompt.chars().take(80).collect();
    send_status(status_tx, &format!("Running Cursor Agent: {}…", preview));
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

pub(crate) async fn handle_mastodon_post(
    arg: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> String {
    let arg = arg.trim();
    if arg.is_empty() {
        return "MASTODON_POST requires text. Usage: MASTODON_POST: <text to post>. Optional visibility prefix: MASTODON_POST: unlisted: <text> (default: public).".to_string();
    }
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
    send_status(status_tx, &format!("Posting to Mastodon ({})…", visibility));
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
