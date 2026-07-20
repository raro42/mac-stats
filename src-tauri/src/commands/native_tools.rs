//! Native Ollama / OpenAI-compatible tool schemas (Hermes/OpenClaw style).
//!
//! When enabled, execute-loop `/api/chat` requests include real `tools` JSON so the model
//! emits structured `tool_calls` instead of free-text `TOOL: arg` lines. We still synthesize
//! text lines for the existing dispatch path so handlers stay unchanged.

use serde_json::{json, Value};

use crate::commands::tool_registry::TOOLS;
use crate::ollama::{ChatMessage, ChatResponse, OllamaToolCall};

/// OpenAI/Ollama function-tool schemas derived from [`TOOLS`].
pub(crate) fn ollama_tool_schemas() -> Vec<Value> {
    let mut out = Vec::with_capacity(TOOLS.len() + 2);
    for t in TOOLS.iter() {
        if t.name == "SCHEDULER" {
            continue;
        }
        let (param_name, param_desc) = primary_param(t.name);
        let parameters = if t.accepts_argument {
            json!({
                "type": "object",
                "properties": {
                    param_name: {
                        "type": "string",
                        "description": param_desc
                    }
                },
                "required": [param_name]
            })
        } else {
            json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            })
        };
        out.push(json!({
            "type": "function",
            "function": {
                "name": t.name,
                "description": t.description,
                "parameters": parameters
            }
        }));
    }
    // Explicit completion signals (also available as text DONE:)
    out.push(json!({
        "type": "function",
        "function": {
            "name": "DONE",
            "description": "End the tool loop when the user request is fully handled (or cannot be).",
            "parameters": {
                "type": "object",
                "properties": {
                    "status": {
                        "type": "string",
                        "enum": ["success", "no"],
                        "description": "success if completed; no if blocked/failed"
                    }
                },
                "required": ["status"]
            }
        }
    }));
    out
}

fn primary_param(name: &str) -> (&'static str, &'static str) {
    match name {
        "BRAVE_SEARCH" | "PERPLEXITY_SEARCH" => ("query", "Search query"),
        "FETCH_URL" | "BROWSER_NAVIGATE" | "BROWSER_SCREENSHOT" => {
            ("url_or_arg", "URL, or 'current' for screenshot of the focused tab")
        }
        "RUN_CMD" => ("command", "Allowlisted shell command"),
        "RUN_JS" => ("code", "JavaScript source to run"),
        "CURSOR_AGENT" => ("prompt", "Task prompt for Cursor Agent"),
        "AGENT" | "SKILL" | "SKILL_VIEW" => ("target", "Agent/skill id or slug, optional task after space"),
        "TODO" => ("spec", "read | add <id> | <content> | done <id> | set <json> | clear"),
        "MEMORY" | "MEMORY_APPEND" => (
            "text",
            "add <text> | replace <old> => <new> | remove <substr> | read (MEMORY_APPEND = add)",
        ),
        "REDMINE_API" | "DISCORD_API" | "OLLAMA_API" | "MCP" => {
            ("request", "Method/path or tool invocation line")
        }
        "SCHEDULE" | "REMOVE_SCHEDULE" => ("spec", "Schedule specification or id"),
        _ => ("argument", "Tool argument string"),
    }
}

/// Convert native `tool_calls` into `TOOL: arg` lines and merge into message content so
/// [`crate::commands::tool_parsing::parse_all_tools_from_response`] keeps working.
pub(crate) fn synthesize_text_tools_from_native(resp: &mut ChatResponse) {
    let Some(calls) = resp.message.tool_calls.as_ref() else {
        return;
    };
    if calls.is_empty() {
        return;
    }
    let lines: Vec<String> = calls.iter().filter_map(tool_call_to_line).collect();
    if lines.is_empty() {
        return;
    }
    let synthesized = lines.join("\n");
    let existing = resp.message.content.trim();
    if existing.is_empty() {
        resp.message.content = synthesized;
    } else if !crate::commands::tool_parsing::parse_all_tools_from_response(existing).is_empty() {
        // Prefer existing text-line tools; keep native as fallback only if parse found nothing.
    } else {
        resp.message.content = format!("{}\n\n{}", existing, synthesized);
    }
    crate::mac_stats_info!(
        "ollama/chat",
        "Native tool_calls → synthesized {} tool line(s)",
        lines.len()
    );
}

fn tool_call_to_line(call: &OllamaToolCall) -> Option<String> {
    let name = call
        .function
        .name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())?;
    let name_upper = name.to_uppercase();
    let arg = arguments_to_arg_string(name, &call.function.arguments);
    if name_upper == "DONE" {
        let status = if arg.is_empty() { "success" } else { arg.as_str() };
        return Some(format!("DONE: {}", status));
    }
    if arg.is_empty() {
        Some(format!("{}:", name_upper))
    } else {
        Some(format!("{}: {}", name_upper, arg))
    }
}

fn arguments_to_arg_string(tool_name: &str, args: &Value) -> String {
    if args.is_null() {
        return String::new();
    }
    if let Some(s) = args.as_str() {
        let trimmed = s.trim();
        if trimmed.starts_with('{') {
            if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
                return arguments_to_arg_string(tool_name, &v);
            }
        }
        return trimmed.to_string();
    }
    let obj = match args.as_object() {
        Some(o) => o,
        None => return args.to_string(),
    };
    if obj.is_empty() {
        return String::new();
    }
    let (primary, _) = primary_param(&tool_name.to_uppercase());
    if let Some(v) = obj.get(primary).and_then(|x| x.as_str()) {
        return v.trim().to_string();
    }
    // Common aliases
    for key in [
        "argument",
        "query",
        "url",
        "url_or_arg",
        "command",
        "code",
        "prompt",
        "target",
        "request",
        "text",
        "spec",
        "status",
        "input",
    ] {
        if let Some(v) = obj.get(key).and_then(|x| x.as_str()) {
            return v.trim().to_string();
        }
    }
    // Single-key object → that value
    if obj.len() == 1 {
        if let Some((_, v)) = obj.iter().next() {
            if let Some(s) = v.as_str() {
                return s.trim().to_string();
            }
            return v.to_string();
        }
    }
    // Multi-arg tools (e.g. BROWSER_INPUT): join values in a stable-ish order
    obj.values()
        .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Compact system-prompt tooling section when native schemas are also sent (schemas carry detail).
pub(crate) fn compact_native_tools_prompt_section() -> String {
    let mut s = String::from(
        "## Tools\n\
You have structured tools available via the API (preferred) and equivalent text lines.\n\
Prefer native function/tool calls. Text fallback: one line `TOOL_NAME: <argument>`.\n\
When finished, call DONE with status success or no — or reply with the final answer only.\n\
Silent default: do not narrate every tool; act, then answer briefly.\n\n\
Available tools:\n",
    );
    s.push_str(&crate::commands::tool_registry::tool_descriptions_for_prompt());
    s
}

/// Helper for building assistant/user messages without tool_calls.
#[allow(dead_code)]
pub(crate) fn msg(role: &str, content: impl Into<String>) -> ChatMessage {
    ChatMessage {
        role: role.to_string(),
        content: content.into(),
        images: None,
        tool_calls: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ollama::{OllamaFunctionCall, OllamaToolCall};

    #[test]
    fn schemas_include_brave_and_done() {
        let schemas = ollama_tool_schemas();
        assert!(schemas.len() > 10);
        let names: Vec<String> = schemas
            .iter()
            .filter_map(|s| {
                s.get("function")
                    .and_then(|f| f.get("name"))
                    .and_then(|n| n.as_str())
                    .map(|s| s.to_string())
            })
            .collect();
        assert!(names.iter().any(|n| n == "BRAVE_SEARCH"));
        assert!(names.iter().any(|n| n == "DONE"));
        assert!(!names.iter().any(|n| n == "SCHEDULER"));
    }

    #[test]
    fn synthesize_brave_query() {
        let mut resp = ChatResponse {
            message: ChatMessage {
                role: "assistant".into(),
                content: String::new(),
                images: None,
                tool_calls: Some(vec![OllamaToolCall {
                    id: None,
                    function: OllamaFunctionCall {
                        name: Some("BRAVE_SEARCH".into()),
                        index: None,
                        arguments: json!({"query": "Ralf Roeber"}),
                    },
                }]),
            },
            done: true,
        };
        synthesize_text_tools_from_native(&mut resp);
        assert_eq!(resp.message.content.trim(), "BRAVE_SEARCH: Ralf Roeber");
    }

    #[test]
    fn synthesize_done_status() {
        let mut resp = ChatResponse {
            message: ChatMessage {
                role: "assistant".into(),
                content: String::new(),
                images: None,
                tool_calls: Some(vec![OllamaToolCall {
                    id: None,
                    function: OllamaFunctionCall {
                        name: Some("DONE".into()),
                        index: None,
                        arguments: json!({"status": "success"}),
                    },
                }]),
            },
            done: true,
        };
        synthesize_text_tools_from_native(&mut resp);
        assert_eq!(resp.message.content.trim(), "DONE: success");
    }
}
