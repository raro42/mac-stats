//! Agent session runner: run a specialist agent as an Ollama session with
//! tool-call loop (DISCORD_API, REDMINE_API) and runtime context injection.

use crate::commands::ollama::send_ollama_chat_messages;
use crate::commands::redmine_helpers::{
    extract_redmine_time_entries_summary_for_reply, question_explicitly_requests_json,
};
use crate::commands::tool_parsing::{
    normalize_inline_tool_sequences, parse_one_tool_at_line,
};
use tracing::info;

/// Build the runtime date/time context block injected into agent system prompts.
pub(crate) fn build_agent_runtime_context(now: chrono::DateTime<chrono::FixedOffset>) -> String {
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

/// Run an Ollama request for an LLM agent (soul+mood+skill as system prompt, task as user message).
/// Uses the agent's model if set; otherwise default. No conversation history. Logs agent name/id.
/// If the agent's response contains DISCORD_API: or REDMINE_API: tool calls, executes them and
/// feeds results back in a loop (up to max_tool_iterations) so agents can do multi-step API work.
/// Used by the tool loop (AGENT:) and by the agent-test CLI.
pub(crate) async fn run_agent_ollama_session(
    agent: &crate::agents::Agent,
    user_message: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
    include_global_memory: bool,
) -> Result<String, String> {
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

/// Execute a tool call found in an agent's response. Supports agent-safe APIs like DISCORD_API and REDMINE_API.
/// Returns Some(result_text) if a tool was executed, None if no tool call was found.
async fn execute_agent_tool_call(
    content: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> Option<String> {
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

/// Normalize Discord API path: strip model commentary after " — " so the path is valid for HTTP.
/// E.g. "/channels/123/messages?limit=10 — fetch the last 10 messages" -> "/channels/123/messages?limit=10"
pub(crate) fn normalize_discord_api_path(path_and_commentary: &str) -> String {
    let s = path_and_commentary.trim();
    let path_only = if let Some(idx) = s.find(" — ") {
        s[..idx].trim()
    } else {
        s
    };
    path_only.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
