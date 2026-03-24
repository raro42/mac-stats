//! Tool execution loop extracted from the `answer_with_ollama_and_fetch` orchestrator.
//!
//! Runs parsed tool calls in sequence, dispatching each to the appropriate handler,
//! managing browser/dedup/budget guards, and feeding results back to Ollama until
//! the model stops emitting tool calls or hits the iteration cap.

use std::path::PathBuf;
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;

use crate::commands::discord_draft_stream::DiscordDraftHandle;
use crate::commands::loop_guard::ToolLoopGuard;
use crate::commands::ollama_chat::send_ollama_chat_messages;
use crate::commands::redmine_helpers::{
    extract_redmine_time_entries_summary_for_reply, grounded_redmine_time_entries_failure_reply,
    question_explicitly_requests_json,
};
use crate::commands::reply_helpers::final_reply_from_tool_results;
use crate::commands::tool_parsing::{
    normalize_browser_tool_arg, parse_all_tools_from_response, truncate_search_query_arg,
    MAX_BROWSER_TOOLS_PER_RUN,
};
use crate::ollama::{ChatMessage, ChatOptions};

/// Immutable parameters for the tool loop, set once before the loop starts.
pub(crate) struct ToolLoopParams {
    pub question: String,
    pub request_id: String,
    pub max_tool_iterations: u32,
    pub model_override: Option<String>,
    pub options_override: Option<ChatOptions>,
    pub status_tx: Option<UnboundedSender<String>>,
    pub discord_draft: Option<DiscordDraftHandle>,
    pub discord_reply_channel_id: Option<u64>,
    pub allow_schedule: bool,
    pub load_global_memory: bool,
    pub agent_descriptions_len: usize,
    pub model_context_size_tokens: u32,
    pub budget_warning_ratio: f64,
}

/// Mutable state accumulated during the tool loop.
pub(crate) struct ToolLoopState {
    pub tool_count: u32,
    pub attachment_paths: Vec<PathBuf>,
    pub screenshot_requested_by_tool_run: bool,
    pub agent_conversation: Vec<(String, String)>,
    pub last_successful_discord_call: Option<(String, String)>,
    pub last_run_cmd_arg: Option<String>,
    pub last_run_cmd_raw_output: Option<String>,
    pub current_task_path: Option<PathBuf>,
    pub exited_via_done: bool,
    pub last_browser_extract: Option<String>,
    pub browser_tool_count: u32,
    pub browser_tool_cap_reached: bool,
    pub last_browser_tool_arg: Option<(String, String)>,
    pub last_news_search_was_hub_only: Option<bool>,
    pub loop_guard: ToolLoopGuard,
}

impl ToolLoopState {
    pub fn new() -> Self {
        Self {
            tool_count: 0,
            attachment_paths: Vec::new(),
            screenshot_requested_by_tool_run: false,
            agent_conversation: Vec::new(),
            last_successful_discord_call: None,
            last_run_cmd_arg: None,
            last_run_cmd_raw_output: None,
            current_task_path: None,
            exited_via_done: false,
            last_browser_extract: None,
            browser_tool_count: 0,
            browser_tool_cap_reached: false,
            last_browser_tool_arg: None,
            last_news_search_was_hub_only: None,
            loop_guard: ToolLoopGuard::new(),
        }
    }
}

/// Result of `run_tool_loop`: the updated messages, final response text, and accumulated state.
pub(crate) struct ToolLoopResult {
    pub response_content: String,
    pub state: ToolLoopState,
}

fn send_status(tx: Option<&UnboundedSender<String>>, msg: &str) {
    if let Some(tx) = tx {
        let _ = tx.send(msg.to_string());
    }
}

const LOG_CONTENT_MAX: usize = 500;

fn log_content(content: &str, verbosity: u8) -> String {
    let n = content.chars().count();
    if verbosity >= 2 || n <= LOG_CONTENT_MAX {
        content.to_string()
    } else {
        crate::logging::ellipse(content, LOG_CONTENT_MAX)
    }
}

/// Run the tool execution loop: parse tools from `response_content`, dispatch each,
/// feed results back to Ollama, repeat until no more tool calls or iteration cap hit.
pub(crate) async fn run_tool_loop(
    params: &ToolLoopParams,
    messages: &mut Vec<ChatMessage>,
    initial_response: String,
) -> Result<ToolLoopResult, String> {
    let verbosity = crate::logging::VERBOSITY.load(std::sync::atomic::Ordering::Relaxed);
    let mut state = ToolLoopState::new();
    let mut response_content = initial_response;

    while state.tool_count < params.max_tool_iterations {
        let tools = parse_all_tools_from_response(&response_content);
        if tools.is_empty() {
            info!(
                "Agent router: no tool call in response ({} chars), treating as final answer: {}",
                response_content.chars().count(),
                log_content(&response_content, verbosity)
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
        let mut page_changed_this_turn = false;

        for (tool, arg) in tools {
            if state.tool_count >= params.max_tool_iterations {
                break;
            }
            state.tool_count += 1;

            // Handle DONE before dispatch so we can set `done_claimed` locally.
            if tool == "DONE" {
                let arg_lower = arg.trim().to_lowercase();
                let claimed_success = arg_lower.is_empty()
                    || arg_lower.contains("success")
                    || arg_lower.contains("yes")
                    || arg_lower.contains("true");
                done_claimed = Some(claimed_success);
                send_status(
                    params.status_tx.as_ref(),
                    if claimed_success {
                        "✅ Task marked done (success)."
                    } else {
                        "⏹\u{fe0f} Task marked done (could not complete)."
                    },
                );
                info!(
                    "Agent router: model called DONE (success={}), exiting tool loop",
                    claimed_success
                );
                tool_results.push(String::new());
                continue;
            }

            let mut executed_browser_tool_arg: Option<(String, String)> = None;

            let arg = if tool == "PERPLEXITY_SEARCH" || tool == "BRAVE_SEARCH" {
                truncate_search_query_arg(&arg)
            } else {
                arg
            };
            let arg_preview: String = arg.chars().take(80).collect();
            if arg.chars().count() > 80 {
                info!(
                    "Agent router: running tool {}/{} — {} (arg: {}...)",
                    state.tool_count, params.max_tool_iterations, tool, arg_preview
                );
            } else {
                info!(
                    "Agent router: running tool {}/{} — {} (arg: {})",
                    state.tool_count, params.max_tool_iterations, tool, arg_preview
                );
            }

            if let Some(ref draft) = params.discord_draft {
                draft.update(format!("Running {}…", tool));
            }

            // General loop guard: detect repeated tool calls and cycles across all tools.
            if let Some(reason) = state.loop_guard.record_and_check(&tool, &arg) {
                info!("Agent router: loop guard blocked {} — {}", tool, reason);
                tool_results.push(reason);
                continue;
            }

            // Cap browser tools per run to prevent runaway loops.
            let is_browser_tool = matches!(
                tool.as_str(),
                "BROWSER_NAVIGATE"
                    | "BROWSER_GO_BACK"
                    | "BROWSER_GO_FORWARD"
                    | "BROWSER_RELOAD"
                    | "BROWSER_CLICK"
                    | "BROWSER_INPUT"
                    | "BROWSER_KEYS"
                    | "BROWSER_SCROLL"
                    | "BROWSER_EXTRACT"
                    | "BROWSER_SEARCH_PAGE"
                    | "BROWSER_SCREENSHOT"
            );
            if is_browser_tool {
                let normalized_arg = normalize_browser_tool_arg(&tool, &arg);
                if state.last_browser_tool_arg.as_ref()
                    == Some(&(tool.clone(), normalized_arg.clone()))
                {
                    let msg = "Same browser action as previous step; use a different action or reply with DONE.".to_string();
                    tool_results.push(msg);
                    info!(
                        "Agent router: duplicate browser action skipped ({} with same arg)",
                        tool
                    );
                    continue;
                }
                if state.browser_tool_count >= MAX_BROWSER_TOOLS_PER_RUN {
                    state.browser_tool_cap_reached = true;
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
                if page_changed_this_turn {
                    tool_results.push(format!(
                        "Page changed by a previous navigation; {} was skipped because element indices are stale. Current page state is above — use it to plan new actions in the next turn.",
                        tool
                    ));
                    info!(
                        "Agent router: terminates-sequence — skipping {} after page-changing action this turn",
                        tool
                    );
                    continue;
                }
                state.browser_tool_count += 1;
                executed_browser_tool_arg = Some((tool.clone(), normalized_arg));
                info!(
                    "Agent router: browser tool #{}/{} this run",
                    state.browser_tool_count, MAX_BROWSER_TOOLS_PER_RUN
                );
            }
            if tool == "BROWSER_SCREENSHOT" {
                state.screenshot_requested_by_tool_run = true;
            }

            let user_message = dispatch_tool(
                &tool,
                &arg,
                params,
                &mut state,
                &response_content,
                messages,
                multi_tool_turn,
            )
            .await;

            let result_len = user_message.chars().count();
            info!(
                "Agent router: tool {} completed, sending result back to Ollama ({} chars): {}",
                tool,
                result_len,
                log_content(&user_message, verbosity)
            );

            // Track RUN_CMD state for TASK_APPEND dedup.
            if tool == "RUN_CMD" && user_message.starts_with("Here is the command output") {
                state.last_run_cmd_arg = Some(arg.clone());
            } else if tool != "RUN_CMD" {
                state.last_run_cmd_arg = None;
            }
            if tool != "TASK_APPEND" && tool != "RUN_CMD" {
                state.last_run_cmd_raw_output = None;
            }
            if tool == "BROWSER_EXTRACT"
                && !user_message.is_empty()
                && !user_message.contains("BROWSER_EXTRACT failed")
            {
                state.last_browser_extract = Some(user_message.clone());
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

            if multi_tool_turn
                && !is_browser_error
                && (tool == "BROWSER_NAVIGATE"
                    || tool == "BROWSER_GO_BACK"
                    || tool == "BROWSER_GO_FORWARD"
                    || tool == "BROWSER_RELOAD")
                && !user_message.starts_with("BROWSER_NAVIGATE requires")
            {
                page_changed_this_turn = true;
                info!(
                    "Agent router: {} terminates sequence — remaining browser actions this turn will be skipped",
                    tool
                );
            }

            tool_results.push(user_message);
            if let Some(pair) = executed_browser_tool_arg {
                state.last_browser_tool_arg = Some(pair);
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

        // Grounded Redmine failure short-circuit.
        if let Some(blocked_reply) =
            grounded_redmine_time_entries_failure_reply(&params.question, &user_message)
        {
            info!("Agent router: returning grounded Redmine blocked-state reply");
            response_content = blocked_reply;
            break;
        }
        if !question_explicitly_requests_json(&params.question) {
            if let Some(summary) = extract_redmine_time_entries_summary_for_reply(&user_message) {
                info!("Agent router: returning direct Redmine time-entry summary");
                response_content = summary;
                if done_claimed.is_some() {
                    state.exited_via_done = true;
                }
                break;
            }
        }
        if done_claimed.is_some() {
            if !user_message.trim().is_empty() {
                response_content = final_reply_from_tool_results(&params.question, &user_message);
            }
            state.exited_via_done = true;
            break;
        }

        messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: response_content.clone(),
            images: None,
        });
        let tool_result_role = if user_message.starts_with("Here is the command output") {
            "system"
        } else {
            "user"
        };
        messages.push(ChatMessage {
            role: tool_result_role.to_string(),
            content: user_message,
            images: None,
        });

        // Budget warning / last-iteration guidance.
        inject_budget_warnings(
            messages,
            state.tool_count,
            params.max_tool_iterations,
            params.budget_warning_ratio,
        );

        let follow_up = match send_ollama_chat_messages(
            messages.clone(),
            params.model_override.clone(),
            params.options_override.clone(),
        )
        .await
        {
            Ok(resp) => resp,
            Err(e)
                if crate::commands::content_reduction::is_context_overflow_error(&e)
                    && crate::config::Config::context_overflow_truncate_enabled() =>
            {
                let max_chars = crate::config::Config::context_overflow_max_result_chars();
                let n = crate::commands::content_reduction::truncate_oversized_tool_results(
                    messages, max_chars,
                );
                if n == 0 {
                    info!(
                        "Agent router: context overflow but no oversized tool results to truncate — returning error"
                    );
                    return Err(
                        crate::commands::content_reduction::sanitize_ollama_error_for_user(&e)
                            .unwrap_or(e),
                    );
                }
                info!(
                    "Agent router: context overflow recovery — truncated {} tool result(s) to {} chars, retrying",
                    n, max_chars
                );
                match send_ollama_chat_messages(
                    messages.clone(),
                    params.model_override.clone(),
                    params.options_override.clone(),
                )
                .await
                {
                    Ok(resp) => {
                        info!("Agent router: context overflow recovery succeeded after truncation");
                        resp
                    }
                    Err(retry_err) => {
                        info!(
                            "Agent router: context overflow recovery retry still failed: {}",
                            retry_err
                        );
                        return Err(format!(
                            "Context overflow: truncated {} tool result(s) and retried, but the request is still too large. Try starting a new topic or using a model with a larger context window.",
                            n
                        ));
                    }
                }
            }
            Err(e) => {
                return Err(
                    crate::commands::content_reduction::sanitize_ollama_error_for_user(&e)
                        .unwrap_or(e),
                );
            }
        };
        response_content = follow_up.message.content.clone();

        // Fallback: if Ollama returned empty after a successful tool result, use the raw tool output.
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
                    let cleaned = raw
                        .replace("\n\nUse this to answer the user's question.", "")
                        .replace("Here is the command output:\n\n", "")
                        .replace("Here is the page content:\n\n", "");
                    response_content = cleaned;
                }
            }
        }

        if state.tool_count >= params.max_tool_iterations {
            info!(
                "Agent router: max tool iterations reached ({}), using last response as final",
                params.max_tool_iterations
            );
        }
    }

    Ok(ToolLoopResult {
        response_content,
        state,
    })
}

fn inject_budget_warnings(
    messages: &mut Vec<ChatMessage>,
    tool_count: u32,
    max_tool_iterations: u32,
    budget_warning_ratio: f64,
) {
    if max_tool_iterations <= 1 || budget_warning_ratio <= 0.0 || budget_warning_ratio >= 1.0 {
        return;
    }
    let ratio = (tool_count as f64) / (max_tool_iterations as f64);
    if tool_count + 1 == max_tool_iterations {
        let msg = format!(
            "LAST ITERATION WARNING: You have used {}/{} tool iterations. This is your LAST tool iteration. \
             Reply with your final answer now. Summarize everything you have found so far. \
             Do NOT start a new tool chain — respond with your best answer or call DONE with your results.",
            tool_count, max_tool_iterations
        );
        info!(
            "Agent router: injecting last-iteration guidance (tool_count={}/{})",
            tool_count, max_tool_iterations
        );
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: msg,
            images: None,
        });
    } else if ratio >= budget_warning_ratio {
        let remaining = max_tool_iterations - tool_count;
        let pct = (ratio * 100.0) as u32;
        let msg = format!(
            "BUDGET WARNING: You have used {}/{} tool iterations ({}%). {} iterations remaining. \
             If the task cannot be completed in the remaining iterations, prioritize: \
             (1) consolidate your results so far, (2) reply with what you have or call DONE. \
             Partial results are far more valuable than exhausting all iterations with nothing saved.",
            tool_count, max_tool_iterations, pct, remaining
        );
        info!(
            "Agent router: injecting budget warning (tool_count={}/{}, ratio={:.2}, threshold={:.2})",
            tool_count, max_tool_iterations, ratio, budget_warning_ratio
        );
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: msg,
            images: None,
        });
    }
}

/// Dispatch a single tool call to its handler and return the result message.
async fn dispatch_tool(
    tool: &str,
    arg: &str,
    params: &ToolLoopParams,
    state: &mut ToolLoopState,
    response_content: &str,
    messages: &[ChatMessage],
    multi_tool_turn: bool,
) -> String {
    match tool {
        "FETCH_URL" if arg.contains("discord.com") => {
            crate::commands::network_tool_dispatch::handle_fetch_url_discord_redirect(
                arg,
                params.status_tx.as_ref(),
            )
            .await
        }
        "FETCH_URL" => {
            let estimated_used = messages.iter().map(|m| m.content.len()).sum::<usize>()
                + params.agent_descriptions_len;
            crate::commands::network_tool_dispatch::handle_fetch_url(
                arg,
                estimated_used,
                params.model_context_size_tokens,
                params.model_override.clone(),
                params.options_override.clone(),
                params.status_tx.as_ref(),
            )
            .await
            .unwrap_or_else(|e| format!("FETCH_URL error: {}", e))
        }
        "BROWSER_SCREENSHOT" => {
            let result = crate::commands::browser_tool_dispatch::handle_browser_screenshot(
                arg,
                &params.request_id,
                params.status_tx.as_ref(),
            )
            .await;
            if let Some(path) = result.attachment_path {
                state.attachment_paths.push(path);
            }
            result.message
        }
        "BROWSER_NAVIGATE" => {
            crate::commands::browser_tool_dispatch::handle_browser_navigate(
                arg,
                &params.request_id,
                params.status_tx.as_ref(),
            )
            .await
        }
        "BROWSER_GO_BACK" => {
            crate::commands::browser_tool_dispatch::handle_browser_go_back(
                &params.request_id,
                params.status_tx.as_ref(),
            )
            .await
        }
        "BROWSER_GO_FORWARD" => {
            crate::commands::browser_tool_dispatch::handle_browser_go_forward(
                &params.request_id,
                params.status_tx.as_ref(),
            )
            .await
        }
        "BROWSER_RELOAD" => {
            crate::commands::browser_tool_dispatch::handle_browser_reload(
                arg,
                &params.request_id,
                params.status_tx.as_ref(),
            )
            .await
        }
        "BROWSER_CLICK" => {
            crate::commands::browser_tool_dispatch::handle_browser_click(
                arg,
                params.status_tx.as_ref(),
            )
            .await
        }
        "BROWSER_INPUT" => {
            crate::commands::browser_tool_dispatch::handle_browser_input(
                arg,
                params.status_tx.as_ref(),
            )
            .await
        }
        "BROWSER_KEYS" => {
            crate::commands::browser_tool_dispatch::handle_browser_keys(
                arg,
                params.status_tx.as_ref(),
            )
            .await
        }
        "BROWSER_SCROLL" => {
            crate::commands::browser_tool_dispatch::handle_browser_scroll(
                arg,
                params.status_tx.as_ref(),
            )
            .await
        }
        "BROWSER_EXTRACT" => {
            send_status(params.status_tx.as_ref(), "📄 Extracting page text…");
            crate::commands::browser_tool_dispatch::handle_browser_extract().await
        }
        "BROWSER_SEARCH_PAGE" => {
            crate::commands::browser_tool_dispatch::handle_browser_search_page(
                arg,
                params.status_tx.as_ref(),
            )
            .await
        }
        "BRAVE_SEARCH" => {
            crate::commands::network_tool_dispatch::handle_brave_search(
                arg,
                params.status_tx.as_ref(),
            )
            .await
        }
        "PERPLEXITY_SEARCH" => {
            let result = crate::commands::perplexity_helpers::handle_perplexity_search(
                &params.question,
                arg,
                params.status_tx.as_ref(),
                &params.request_id,
            )
            .await;
            state.attachment_paths.extend(result.new_attachment_paths);
            if let Some(hub_only) = result.news_search_was_hub_only {
                state.last_news_search_was_hub_only = Some(hub_only);
            }
            result.text
        }
        "RUN_JS" => {
            crate::commands::delegation_tool_dispatch::handle_run_js(arg, params.status_tx.as_ref())
        }
        "SKILL" => {
            crate::commands::delegation_tool_dispatch::handle_skill(
                arg,
                &params.question,
                params.model_override.clone(),
                params.options_override.clone(),
                params.status_tx.as_ref(),
            )
            .await
        }
        "AGENT" => {
            let last_user_content = messages
                .last()
                .filter(|m| m.role == "user")
                .map(|m| m.content.as_str());
            let result = crate::commands::delegation_tool_dispatch::handle_agent(
                arg,
                &params.question,
                params.discord_reply_channel_id,
                params.status_tx.as_ref(),
                params.load_global_memory,
                last_user_content,
            )
            .await;
            if let Some(entry) = result.agent_conversation_entry {
                state.agent_conversation.push(entry);
            }
            result.message
        }
        "SCHEDULE" => crate::commands::task_tool_handlers::handle_schedule(
            arg,
            params.allow_schedule,
            params.discord_reply_channel_id,
            &params.status_tx,
        ),
        "REMOVE_SCHEDULE" => {
            crate::commands::task_tool_handlers::handle_remove_schedule(arg, &params.status_tx)
        }
        "LIST_SCHEDULES" => {
            crate::commands::task_tool_handlers::handle_list_schedules(&params.status_tx)
        }
        "RUN_CMD" => {
            let result = crate::commands::delegation_tool_dispatch::handle_run_cmd(
                arg,
                state.last_run_cmd_arg.as_deref(),
                multi_tool_turn,
                params.model_override.clone(),
                params.options_override.clone(),
                params.status_tx.as_ref(),
            )
            .await;
            if let Some(raw) = result.raw_output {
                state.last_run_cmd_raw_output = Some(raw);
            }
            result.message
        }
        "PYTHON_SCRIPT" => {
            crate::commands::delegation_tool_dispatch::handle_python_script(
                arg,
                response_content,
                params.status_tx.as_ref(),
            )
            .await
        }
        "DISCORD_API" => {
            let result = crate::commands::network_tool_dispatch::handle_discord_api(
                arg,
                state.last_successful_discord_call.as_ref(),
                params.status_tx.as_ref(),
            )
            .await;
            if let Some(call) = result.successful_call {
                state.last_successful_discord_call = Some(call);
            }
            result.message
        }
        "OLLAMA_API" => {
            crate::commands::misc_tool_dispatch::handle_ollama_api(arg, params.status_tx.as_ref())
                .await
        }
        "TASK_APPEND" => crate::commands::task_tool_handlers::handle_task_append(
            arg,
            &mut state.current_task_path,
            &mut state.last_run_cmd_raw_output,
            &params.status_tx,
        ),
        "TASK_STATUS" => crate::commands::task_tool_handlers::handle_task_status(
            arg,
            &mut state.current_task_path,
        ),
        "TASK_CREATE" => crate::commands::task_tool_handlers::handle_task_create(
            arg,
            params.discord_reply_channel_id,
            &mut state.current_task_path,
        ),
        "TASK_SHOW" => crate::commands::task_tool_handlers::handle_task_show(
            arg,
            &mut state.current_task_path,
            &params.status_tx,
        ),
        "TASK_ASSIGN" => crate::commands::task_tool_handlers::handle_task_assign(
            arg,
            &mut state.current_task_path,
            &params.status_tx,
        ),
        "TASK_SLEEP" => crate::commands::task_tool_handlers::handle_task_sleep(
            arg,
            &mut state.current_task_path,
            &params.status_tx,
        ),
        "TASK_LIST" => {
            crate::commands::task_tool_handlers::handle_task_list(arg, &params.status_tx)
        }
        "MCP" => {
            crate::commands::misc_tool_dispatch::handle_mcp(arg, params.status_tx.as_ref()).await
        }
        "CURSOR_AGENT" => {
            crate::commands::misc_tool_dispatch::handle_cursor_agent(arg, params.status_tx.as_ref())
                .await
        }
        "REDMINE_API" => {
            crate::commands::network_tool_dispatch::handle_redmine_api(
                arg,
                &params.question,
                &params.request_id,
                params.status_tx.as_ref(),
            )
            .await
        }
        "MASTODON_POST" => {
            crate::commands::misc_tool_dispatch::handle_mastodon_post(
                arg,
                params.status_tx.as_ref(),
            )
            .await
        }
        "MEMORY_APPEND" => crate::commands::misc_tool_dispatch::handle_memory_append(
            arg,
            params.discord_reply_channel_id,
        ),
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
    }
}
