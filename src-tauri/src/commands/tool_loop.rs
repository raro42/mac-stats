//! Tool execution loop extracted from the `answer_with_ollama_and_fetch` orchestrator.
//!
//! Runs parsed tool calls in sequence, dispatching each to the appropriate handler,
//! managing browser/dedup/budget guards, and feeding results back to Ollama until
//! the model stops emitting tool calls or hits the iteration cap.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;

use crate::commands::discord_draft_stream::DiscordDraftHandle;
use crate::commands::loop_guard::{ToolLoopAfterResult, ToolLoopGuard};
use crate::commands::ollama_chat::{send_ollama_chat_messages, OllamaHttpQueue};
use crate::commands::redmine_helpers::{
    extract_redmine_time_entries_summary_for_reply, grounded_redmine_time_entries_failure_reply,
    question_explicitly_requests_json,
};
use crate::commands::reply_helpers::final_reply_from_tool_results;
use crate::commands::tool_parsing::{
    normalize_browser_tool_arg, parse_all_tools_from_response, truncate_search_query_arg,
    MAX_BROWSER_TOOLS_PER_RUN,
};
use crate::commands::turn_lifecycle::TurnOutputGate;
use crate::ollama::{ChatMessage, ChatOptions};

/// One executed (or skipped) tool invocation in the agent router for observability and summaries.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields used in tests and for future structured export; summary log uses counts only.
pub(crate) struct ToolStep {
    pub step_index: u32,
    pub tool_name: String,
    pub input_summary: String,
    pub result_summary: String,
    pub ok: bool,
    pub duration_ms: u128,
    pub is_browser_action: bool,
}

fn is_browser_tool_name(tool: &str) -> bool {
    matches!(
        tool,
        "BROWSER_NAVIGATE"
            | "BROWSER_GO_BACK"
            | "BROWSER_GO_FORWARD"
            | "BROWSER_RELOAD"
            | "BROWSER_CLEAR_COOKIES"
            | "BROWSER_SWITCH_TAB"
            | "BROWSER_CLOSE_TAB"
            | "BROWSER_CLICK"
            | "BROWSER_HOVER"
            | "BROWSER_DRAG"
            | "BROWSER_INPUT"
            | "BROWSER_UPLOAD"
            | "BROWSER_KEYS"
            | "BROWSER_SCROLL"
            | "BROWSER_EXTRACT"
            | "BROWSER_SEARCH_PAGE"
            | "BROWSER_QUERY"
            | "BROWSER_DOWNLOAD"
            | "BROWSER_SCREENSHOT"
            | "BROWSER_SAVE_PDF"
    )
}

/// Whether the message returned from dispatch indicates a failed tool execution (for failure budget).
pub(crate) fn tool_dispatch_message_is_error(tool: &str, message: &str) -> bool {
    if tool == "DONE" {
        return false;
    }
    // FETCH_URL: `handle_fetch_url` usually returns Ok() with user-facing text; only rare paths use
    // `FETCH_URL error:` (e.g. `reduce_fetched_content_to_fit` Err). SSRF blocks and HTTP failures
    // surface as "That URL could not be fetched…" — those must count toward the failure budget.
    if tool == "FETCH_URL" {
        let m = message.trim_start();
        return m.starts_with("FETCH_URL error:")
            || m.starts_with("That URL could not be fetched")
            || m.starts_with("That URL returned 401 Unauthorized")
            || m.starts_with("Discord API failed")
            || m.starts_with("Cannot fetch discord.com pages directly");
    }
    if tool == "RUN_CMD"
        && (message.starts_with("RUN_CMD failed")
            || message.starts_with("RUN_CMD failed in a multi-step plan"))
    {
        return true;
    }
    if tool == "BROWSER_SCREENSHOT" {
        return message.starts_with("Screenshot of current page failed")
            || message.starts_with("Screenshot task error");
    }
    if tool == "BROWSER_SAVE_PDF" {
        return message.starts_with("PDF export of current page failed")
            || message.starts_with("PDF export task error");
    }
    if tool.starts_with("BROWSER_") {
        return message.starts_with(&format!("{} failed", tool))
            || message.starts_with(&format!("{} task error", tool))
            || message.starts_with(&format!("{} HTTP fallback task error", tool))
            || message.starts_with(&format!("{} CDP retry task error", tool));
    }
    if message.contains("Unknown tool \"") && message.contains("available tools") {
        return true;
    }
    false
}

fn summarize_arg(arg: &str) -> String {
    let t: String = arg.chars().take(120).collect();
    if arg.chars().count() > 120 {
        format!("{}…", t)
    } else {
        t
    }
}

fn summarize_result(message: &str) -> String {
    let one_line = message.lines().next().unwrap_or(message);
    let t: String = one_line.chars().take(160).collect();
    if one_line.chars().count() > 160 {
        format!("{}…", t)
    } else {
        t
    }
}

/// After a page-changing browser action in a multi-tool turn, later tools in the batch are skipped
/// so the model cannot use stale element indices — except **screenshot** and **save PDF**, which
/// only read the current document and remain valid on the post-navigation page.
pub(crate) fn stale_batch_guard_should_skip_tool(
    multi_tool_turn: bool,
    page_changed_this_turn: bool,
    batch_idx: usize,
    tool: &str,
) -> bool {
    multi_tool_turn
        && page_changed_this_turn
        && batch_idx > 0
        && !matches!(tool, "BROWSER_SCREENSHOT" | "BROWSER_SAVE_PDF")
}

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
    /// When `Some`, replaces legacy same-call limits with hash-based repeat detection.
    pub loop_detection: Option<crate::config::ToolLoopDetectionConfig>,
    /// Stop after this many consecutive tool or follow-up LLM failures (OpenClaw-style budget).
    pub max_consecutive_failures: u32,
    /// When set, records tool names and short arg summaries for timeout / error messages (no raw results).
    pub partial_progress_capture: Option<crate::commands::partial_progress::PartialProgressCapture>,
    /// When set, status lines and draft updates are suppressed after a wall-clock turn timeout.
    pub output_gate: Option<TurnOutputGate>,
    /// Set to `true` after tool results are merged into the chat for the next model turn (user-visible pipeline).
    pub forward_substantive_output: Option<Arc<AtomicBool>>,
}

/// Mutable state accumulated during the tool loop.
pub(crate) struct ToolLoopState {
    pub tool_count: u32,
    pub attachment_paths: Vec<PathBuf>,
    /// Latest successful `BROWSER_SCREENSHOT` path this run (for `[[attach_screenshot]]`).
    pub last_browser_screenshot_path: Option<PathBuf>,
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
    /// Set after a `BROWSER_*` tool is actually dispatched in this agent request (success or error),
    /// so the next browser tool can optionally sleep first (`browserWaitBetweenActionsSecs`).
    pub had_browser_tool_dispatch_in_request: bool,
    pub consecutive_failures: u32,
    pub tool_steps: Vec<ToolStep>,
    pub stopped_due_to_failure_budget: bool,
    /// True when the tool loop exited because `tool_count` reached `max_tool_iterations` (not DONE / empty tools).
    pub hit_max_tool_iterations: bool,
    pub step_sequence: u32,
}

impl ToolLoopState {
    pub fn new(loop_detection: Option<crate::config::ToolLoopDetectionConfig>) -> Self {
        Self {
            tool_count: 0,
            attachment_paths: Vec::new(),
            last_browser_screenshot_path: None,
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
            loop_guard: ToolLoopGuard::new(loop_detection),
            had_browser_tool_dispatch_in_request: false,
            consecutive_failures: 0,
            tool_steps: Vec::new(),
            stopped_due_to_failure_budget: false,
            hit_max_tool_iterations: false,
            step_sequence: 0,
        }
    }
}

/// Result of `run_tool_loop`: the updated messages, final response text, and accumulated state.
pub(crate) struct ToolLoopResult {
    pub response_content: String,
    pub state: ToolLoopState,
    /// Optional markdown line for Discord/scheduler replies (tool run summary).
    pub user_visible_footer: Option<String>,
}

fn send_status(gate: Option<&TurnOutputGate>, tx: Option<&UnboundedSender<String>>, msg: &str) {
    if let Some(g) = gate {
        if !crate::commands::turn_lifecycle::gate_allows_send(g) {
            return;
        }
    }
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
    let _agent_status_guard =
        crate::events::push_agent_status_tx(params.status_tx.clone(), params.output_gate.clone());
    let verbosity = crate::logging::VERBOSITY.load(std::sync::atomic::Ordering::Relaxed);
    let mut state = ToolLoopState::new(params.loop_detection.clone());
    let mut response_content = initial_response;
    if let Some(ref cap) = params.partial_progress_capture {
        cap.set_last_assistant_text(&response_content);
    }

    'agent_tool_loop: while state.tool_count < params.max_tool_iterations {
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
        let mut critical_loop_abort: Option<String> = None;
        let mut failure_budget_break = false;

        let bump_failure_budget = |state: &mut ToolLoopState,
                                   tool_results: &[String],
                                   response_content: &mut String|
         -> bool {
            if state.consecutive_failures < params.max_consecutive_failures {
                return false;
            }
            state.stopped_due_to_failure_budget = true;
            let joined = tool_results.join("\n\n---\n\n");
            *response_content = format!(
                "{}\n\n---\n\n**Limit: consecutive tool/LLM failures** — stopped after {} consecutive errors (limit {}). Partial tool output is preserved above.",
                joined.trim_end(),
                state.consecutive_failures,
                params.max_consecutive_failures
            );
            info!(
                "Agent router: failure budget exceeded (consecutive_failures={}, limit={}) — returning partial results",
                state.consecutive_failures, params.max_consecutive_failures
            );
            true
        };

        for (batch_idx, (tool, arg)) in tools.into_iter().enumerate() {
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
                    params.output_gate.as_ref(),
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
                state.consecutive_failures = 0;
                state.step_sequence += 1;
                state.tool_steps.push(ToolStep {
                    step_index: state.step_sequence,
                    tool_name: tool.clone(),
                    input_summary: summarize_arg(&arg),
                    result_summary: "DONE".to_string(),
                    ok: true,
                    duration_ms: 0,
                    is_browser_action: false,
                });
                if let Some(ref cap) = params.partial_progress_capture {
                    cap.record_tool_run(&tool, &summarize_arg(&arg), true);
                }
                tool_results.push(String::new());
                continue;
            }

            let mut executed_browser_tool_arg: Option<(String, String)> = None;

            let arg = if tool == "PERPLEXITY_SEARCH" || tool == "BRAVE_SEARCH" {
                truncate_search_query_arg(&arg)
            } else {
                arg
            };

            // Stale target guard (browser-use style): after a successful navigation in this batch,
            // skip remaining index-based browser tools so the model is re-invoked with fresh state.
            // Screenshot / PDF export still run — they capture the page as it is after navigation.
            if stale_batch_guard_should_skip_tool(
                multi_tool_turn,
                page_changed_this_turn,
                batch_idx,
                &tool,
            ) {
                let msg = format!(
                    "Skipped ({tool}): browser page or target changed earlier in this batch; continue in the next model turn with updated state instead of chaining more tools here."
                );
                info!(
                    "Agent router: stale-batch skip — skipping {} after page-changing action this turn",
                    tool
                );
                state.step_sequence += 1;
                state.tool_steps.push(ToolStep {
                    step_index: state.step_sequence,
                    tool_name: tool.clone(),
                    input_summary: summarize_arg(&arg),
                    result_summary: "skipped (stale batch)".to_string(),
                    ok: true,
                    duration_ms: 0,
                    is_browser_action: is_browser_tool_name(&tool),
                });
                if let Some(ref cap) = params.partial_progress_capture {
                    cap.record_tool_run(&tool, &summarize_arg(&arg), true);
                }
                tool_results.push(msg);
                continue;
            }

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
                if params.output_gate.as_ref().map_or(true, |g| {
                    crate::commands::turn_lifecycle::gate_allows_send(g)
                }) {
                    draft.update(format!("Running {}…", tool));
                }
            }

            let is_browser_tool = is_browser_tool_name(&tool);
            if is_browser_tool {
                let normalized_arg = normalize_browser_tool_arg(&tool, &arg);
                if state.last_browser_tool_arg.as_ref()
                    == Some(&(tool.clone(), normalized_arg.clone()))
                {
                    let msg = "Same browser action as previous step; use a different action or reply with DONE.".to_string();
                    state.consecutive_failures += 1;
                    state.step_sequence += 1;
                    state.tool_steps.push(ToolStep {
                        step_index: state.step_sequence,
                        tool_name: tool.clone(),
                        input_summary: summarize_arg(&arg),
                        result_summary: summarize_result(&msg),
                        ok: false,
                        duration_ms: 0,
                        is_browser_action: true,
                    });
                    if let Some(ref cap) = params.partial_progress_capture {
                        cap.record_tool_run(&tool, &summarize_arg(&arg), false);
                    }
                    tool_results.push(msg);
                    info!(
                        "Agent router: duplicate browser action skipped ({} with same arg)",
                        tool
                    );
                    if bump_failure_budget(&mut state, &tool_results, &mut response_content) {
                        failure_budget_break = true;
                        break;
                    }
                    continue;
                }
                if state.browser_tool_count >= MAX_BROWSER_TOOLS_PER_RUN {
                    state.browser_tool_cap_reached = true;
                    let msg = format!(
                        "Maximum browser actions per run reached ({}). Reply with your answer or DONE: success / DONE: no.",
                        MAX_BROWSER_TOOLS_PER_RUN
                    );
                    state.consecutive_failures += 1;
                    state.step_sequence += 1;
                    state.tool_steps.push(ToolStep {
                        step_index: state.step_sequence,
                        tool_name: tool.clone(),
                        input_summary: summarize_arg(&arg),
                        result_summary: summarize_result(&msg),
                        ok: false,
                        duration_ms: 0,
                        is_browser_action: true,
                    });
                    if let Some(ref cap) = params.partial_progress_capture {
                        cap.record_tool_run(&tool, &summarize_arg(&arg), false);
                    }
                    tool_results.push(msg);
                    info!(
                        "Agent router: browser tool cap reached ({}), skipping {}",
                        MAX_BROWSER_TOOLS_PER_RUN, tool
                    );
                    if bump_failure_budget(&mut state, &tool_results, &mut response_content) {
                        failure_budget_break = true;
                        break;
                    }
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

            // Loop guard after preflight skips so we only record tools that will actually run.
            if let Some(reason) = state.loop_guard.before_tool_execute(&tool, &arg) {
                info!("Agent router: loop guard blocked {} — {}", tool, reason);
                state.consecutive_failures += 1;
                state.step_sequence += 1;
                state.tool_steps.push(ToolStep {
                    step_index: state.step_sequence,
                    tool_name: tool.clone(),
                    input_summary: summarize_arg(&arg),
                    result_summary: summarize_result(&reason),
                    ok: false,
                    duration_ms: 0,
                    is_browser_action: is_browser_tool,
                });
                if let Some(ref cap) = params.partial_progress_capture {
                    cap.record_tool_run(&tool, &summarize_arg(&arg), false);
                }
                tool_results.push(reason);
                if bump_failure_budget(&mut state, &tool_results, &mut response_content) {
                    failure_budget_break = true;
                    break;
                }
                continue;
            }

            if is_browser_tool {
                let wait = crate::config::Config::browser_wait_between_actions_secs();
                if wait > 0.0 && state.had_browser_tool_dispatch_in_request {
                    crate::mac_stats_debug!(
                        "browser/between_actions",
                        "Browser agent: waiting {:.3}s between actions (browserWaitBetweenActionsSecs)",
                        wait
                    );
                    tokio::time::sleep(std::time::Duration::from_secs_f64(wait)).await;
                }
            }

            let started = Instant::now();
            let mut user_message = dispatch_tool(
                &tool,
                &arg,
                params,
                &mut state,
                &response_content,
                messages,
                multi_tool_turn,
            )
            .await;
            let duration_ms = started.elapsed().as_millis();

            if is_browser_tool {
                state.had_browser_tool_dispatch_in_request = true;
            }

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

            let is_browser_error =
                is_browser_tool && tool_dispatch_message_is_error(&tool, &user_message);
            let is_multi_tool_run_cmd_error = tool == "RUN_CMD"
                && user_message.starts_with("RUN_CMD failed in a multi-step plan");

            if multi_tool_turn
                && !is_browser_error
                && (tool == "BROWSER_NAVIGATE"
                    || tool == "BROWSER_GO_BACK"
                    || tool == "BROWSER_GO_FORWARD"
                    || tool == "BROWSER_RELOAD"
                    || tool == "BROWSER_SWITCH_TAB"
                    || tool == "BROWSER_CLOSE_TAB")
                && !user_message.starts_with("BROWSER_NAVIGATE requires")
            {
                page_changed_this_turn = true;
                info!(
                    "Agent router: {} completed — stale-batch guard applies to later index-based browser tools this batch (BROWSER_SCREENSHOT / BROWSER_SAVE_PDF still run)",
                    tool
                );
            }

            match state
                .loop_guard
                .after_tool_result(&tool, &arg, &user_message)
            {
                ToolLoopAfterResult::None => {}
                ToolLoopAfterResult::Warning(s) => user_message.push_str(&s),
                ToolLoopAfterResult::Critical(m) => {
                    let dispatch_failed_crit = tool_dispatch_message_is_error(&tool, &user_message);
                    crate::events::emit(
                        "tool:invoked",
                        crate::events::EventPayload::ToolInvoked {
                            tool_name: tool.clone(),
                            success: !dispatch_failed_crit,
                            duration_ms,
                        },
                    );
                    info!(
                        "Agent router: tool-loop detection critical for {} — stopping run",
                        tool
                    );
                    state.consecutive_failures += 1;
                    state.step_sequence += 1;
                    state.tool_steps.push(ToolStep {
                        step_index: state.step_sequence,
                        tool_name: tool.clone(),
                        input_summary: summarize_arg(&arg),
                        result_summary: summarize_result(&user_message),
                        ok: false,
                        duration_ms,
                        is_browser_action: is_browser_tool,
                    });
                    if let Some(ref cap) = params.partial_progress_capture {
                        cap.record_tool_run(&tool, &summarize_arg(&arg), false);
                    }
                    tool_results.push(user_message);
                    if let Some(pair) = executed_browser_tool_arg {
                        state.last_browser_tool_arg = Some(pair);
                    }
                    critical_loop_abort = Some(m);
                    break;
                }
            }

            let dispatch_failed = tool_dispatch_message_is_error(&tool, &user_message);
            crate::events::emit(
                "tool:invoked",
                crate::events::EventPayload::ToolInvoked {
                    tool_name: tool.clone(),
                    success: !dispatch_failed,
                    duration_ms,
                },
            );
            if dispatch_failed {
                state.consecutive_failures += 1;
            } else {
                state.consecutive_failures = 0;
            }

            state.step_sequence += 1;
            state.tool_steps.push(ToolStep {
                step_index: state.step_sequence,
                tool_name: tool.clone(),
                input_summary: summarize_arg(&arg),
                result_summary: summarize_result(&user_message),
                ok: !dispatch_failed,
                duration_ms,
                is_browser_action: is_browser_tool,
            });
            if let Some(ref cap) = params.partial_progress_capture {
                cap.record_tool_run(&tool, &summarize_arg(&arg), !dispatch_failed);
            }

            tool_results.push(user_message.clone());
            if let Some(pair) = executed_browser_tool_arg {
                state.last_browser_tool_arg = Some(pair);
            }

            if bump_failure_budget(&mut state, &tool_results, &mut response_content) {
                failure_budget_break = true;
                break;
            }

            if is_browser_error || is_multi_tool_run_cmd_error {
                info!(
                    "Agent router: {} returned an error, aborting remaining tools in this turn",
                    tool
                );
                break;
            }
        }

        if failure_budget_break {
            break 'agent_tool_loop;
        }

        if let Some(msg) = critical_loop_abort {
            response_content = msg;
            break;
        }

        let user_message = tool_results.join("\n\n---\n\n");
        let user_message_snapshot_for_llm_fail = user_message.clone();

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

        if let Some(f) = params.forward_substantive_output.as_ref() {
            f.store(true, Ordering::Release);
        }

        messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: crate::commands::directive_tags::strip_inline_directive_tags_for_display(
                &response_content,
            ),
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

        let n_proactive =
            crate::commands::content_reduction::proactively_compact_tool_results_for_context_budget(
                messages.as_mut_slice(),
                params.model_context_size_tokens,
            );
        if n_proactive > 0 {
            info!(
                "Agent router: proactive context budget — {} tool-result compaction step(s) before follow-up Ollama call",
                n_proactive
            );
        }

        let follow_up = match send_ollama_chat_messages(
            messages.clone(),
            params.model_override.clone(),
            params.options_override.clone(),
            OllamaHttpQueue::Nested,
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
                    OllamaHttpQueue::Nested,
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
                        state.consecutive_failures += 1;
                        info!(
                            "Agent router: follow-up LLM failure after overflow recovery (consecutive_failures={}/{})",
                            state.consecutive_failures, params.max_consecutive_failures
                        );
                        if state.consecutive_failures >= params.max_consecutive_failures {
                            state.stopped_due_to_failure_budget = true;
                            response_content = format!(
                                "{}\n\n---\n\n**Limit: consecutive tool/LLM failures** — {} consecutive errors (limit {}). Context overflow recovery still failed; partial tool output is above.",
                                user_message_snapshot_for_llm_fail.trim_end(),
                                state.consecutive_failures,
                                params.max_consecutive_failures
                            );
                            break 'agent_tool_loop;
                        }
                        return Err(format!(
                            "Context overflow: truncated {} tool result(s) and retried, but the request is still too large. Try starting a new topic or using a model with a larger context window.",
                            n
                        ));
                    }
                }
            }
            Err(e) => {
                state.consecutive_failures += 1;
                let sanitized =
                    crate::commands::content_reduction::sanitize_ollama_error_for_user(&e)
                        .unwrap_or_else(|| e.clone());
                info!(
                    "Agent router: follow-up LLM call failed (consecutive_failures={}/{}): {}",
                    state.consecutive_failures, params.max_consecutive_failures, sanitized
                );
                if state.consecutive_failures >= params.max_consecutive_failures {
                    state.stopped_due_to_failure_budget = true;
                    response_content = format!(
                        "{}\n\n---\n\n**Limit: consecutive tool/LLM failures** — {} consecutive errors (limit {}). Last error: {}",
                        user_message_snapshot_for_llm_fail.trim_end(),
                        state.consecutive_failures,
                        params.max_consecutive_failures,
                        sanitized
                    );
                    break 'agent_tool_loop;
                }
                return Err(sanitized);
            }
        };
        response_content = follow_up.message.content.clone();
        if let Some(ref cap) = params.partial_progress_capture {
            cap.set_last_assistant_text(&response_content);
        }

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
            state.hit_max_tool_iterations = true;
            info!(
                "Agent router: limit=max_tool_iterations — cap {} reached, using last model response as final",
                params.max_tool_iterations
            );
        }
    }

    let total_ms: u128 = state.tool_steps.iter().map(|s| s.duration_ms).sum();
    let error_steps = state.tool_steps.iter().filter(|s| !s.ok).count();
    let mut tools_used: Vec<&str> = state
        .tool_steps
        .iter()
        .map(|s| s.tool_name.as_str())
        .collect();
    tools_used.sort_unstable();
    tools_used.dedup();
    info!(
        "Agent router [{}]: tool-step summary steps={} error_steps={} total_ms={} tools_used=[{}] stopped_failure_budget={}",
        params.request_id,
        state.tool_steps.len(),
        error_steps,
        total_ms,
        tools_used.join(", "),
        state.stopped_due_to_failure_budget
    );

    let user_visible_footer = build_tool_run_footer(&state);

    Ok(ToolLoopResult {
        response_content,
        state,
        user_visible_footer,
    })
}

fn build_tool_run_footer(state: &ToolLoopState) -> Option<String> {
    if state.tool_steps.is_empty() {
        return None;
    }
    let n = state.tool_steps.len();
    let fails = state.tool_steps.iter().filter(|s| !s.ok).count();
    let total_ms: u128 = state.tool_steps.iter().map(|s| s.duration_ms).sum();
    let mut names: Vec<&str> = state
        .tool_steps
        .iter()
        .map(|s| s.tool_name.as_str())
        .collect();
    names.sort_unstable();
    names.dedup();
    let relevant =
        state.stopped_due_to_failure_budget || state.hit_max_tool_iterations || n >= 2 || fails > 0;
    if !relevant {
        return None;
    }
    let mut line = format!(
        "*Agent tool run: {} step(s), {} with errors, ~{} ms total; tools: {}.*",
        n,
        fails,
        total_ms,
        names.join(", ")
    );
    if state.stopped_due_to_failure_budget {
        line.push_str(" Stopped early due to repeated failures (maxConsecutiveToolFailures).");
    }
    if state.hit_max_tool_iterations {
        line.push_str(" **Limit: tool iteration cap** — reached max tool rounds (agentRouterMaxToolIterations* or agent max_tool_iterations).");
    }
    Some(format!("\n\n{}", line))
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
                state.last_browser_screenshot_path = Some(path.clone());
                state.attachment_paths.push(path);
            }
            result.message
        }
        "BROWSER_SAVE_PDF" => {
            let result = crate::commands::browser_tool_dispatch::handle_browser_save_pdf(
                arg,
                &params.request_id,
                params.status_tx.as_ref(),
            )
            .await;
            if let Some(path) = result.attachment_path {
                if !state.attachment_paths.iter().any(|x| x == &path) {
                    state.attachment_paths.push(path);
                }
            }
            result.message
        }
        "BROWSER_NAVIGATE" => {
            let (msg, dls) = crate::commands::browser_tool_dispatch::handle_browser_navigate(
                arg,
                &params.request_id,
                params.status_tx.as_ref(),
            )
            .await;
            for p in dls {
                if !state.attachment_paths.iter().any(|x| x == &p) {
                    state.attachment_paths.push(p);
                }
            }
            msg
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
        "BROWSER_CLEAR_COOKIES" => {
            crate::commands::browser_tool_dispatch::handle_browser_clear_cookies(
                &params.request_id,
                params.status_tx.as_ref(),
            )
            .await
        }
        "BROWSER_SWITCH_TAB" => {
            crate::commands::browser_tool_dispatch::handle_browser_switch_tab(
                arg,
                &params.request_id,
                params.status_tx.as_ref(),
            )
            .await
        }
        "BROWSER_CLOSE_TAB" => {
            crate::commands::browser_tool_dispatch::handle_browser_close_tab(
                arg,
                &params.request_id,
                params.status_tx.as_ref(),
            )
            .await
        }
        "BROWSER_CLICK" => {
            let (msg, dls) = crate::commands::browser_tool_dispatch::handle_browser_click(
                arg,
                &params.request_id,
                params.status_tx.as_ref(),
            )
            .await;
            for p in dls {
                if !state.attachment_paths.iter().any(|x| x == &p) {
                    state.attachment_paths.push(p);
                }
            }
            msg
        }
        "BROWSER_HOVER" => {
            let (msg, dls) = crate::commands::browser_tool_dispatch::handle_browser_hover(
                arg,
                &params.request_id,
                params.status_tx.as_ref(),
            )
            .await;
            for p in dls {
                if !state.attachment_paths.iter().any(|x| x == &p) {
                    state.attachment_paths.push(p);
                }
            }
            msg
        }
        "BROWSER_DRAG" => {
            let (msg, dls) = crate::commands::browser_tool_dispatch::handle_browser_drag(
                arg,
                &params.request_id,
                params.status_tx.as_ref(),
            )
            .await;
            for p in dls {
                if !state.attachment_paths.iter().any(|x| x == &p) {
                    state.attachment_paths.push(p);
                }
            }
            msg
        }
        "BROWSER_DOWNLOAD" => {
            let (msg, dls) = crate::commands::browser_tool_dispatch::handle_browser_download(
                arg,
                &params.request_id,
                params.status_tx.as_ref(),
            )
            .await;
            for p in dls {
                if !state.attachment_paths.iter().any(|x| x == &p) {
                    state.attachment_paths.push(p);
                }
            }
            msg
        }
        "BROWSER_INPUT" => {
            crate::commands::browser_tool_dispatch::handle_browser_input(
                arg,
                params.status_tx.as_ref(),
            )
            .await
        }
        "BROWSER_UPLOAD" => {
            crate::commands::browser_tool_dispatch::handle_browser_upload(
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
            send_status(
                params.output_gate.as_ref(),
                params.status_tx.as_ref(),
                "📄 Extracting page as markdown…",
            );
            crate::commands::browser_tool_dispatch::handle_browser_extract(arg).await
        }
        "BROWSER_SEARCH_PAGE" => {
            crate::commands::browser_tool_dispatch::handle_browser_search_page(
                arg,
                params.status_tx.as_ref(),
            )
            .await
        }
        "BROWSER_QUERY" => {
            crate::commands::browser_tool_dispatch::handle_browser_query(
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

#[cfg(test)]
mod tool_step_tests {
    use super::{stale_batch_guard_should_skip_tool, tool_dispatch_message_is_error};

    #[test]
    fn stale_batch_skips_click_after_navigate_in_multi_tool_turn() {
        assert!(stale_batch_guard_should_skip_tool(
            true,
            true,
            1,
            "BROWSER_CLICK"
        ));
    }

    #[test]
    fn stale_batch_allows_screenshot_after_navigate_in_multi_tool_turn() {
        assert!(!stale_batch_guard_should_skip_tool(
            true,
            true,
            1,
            "BROWSER_SCREENSHOT"
        ));
    }

    #[test]
    fn stale_batch_allows_save_pdf_after_navigate_in_multi_tool_turn() {
        assert!(!stale_batch_guard_should_skip_tool(
            true,
            true,
            1,
            "BROWSER_SAVE_PDF"
        ));
    }

    #[test]
    fn stale_batch_first_tool_never_skipped_by_guard() {
        assert!(!stale_batch_guard_should_skip_tool(
            true,
            true,
            0,
            "BROWSER_CLICK"
        ));
    }

    #[test]
    fn fetch_url_error_prefix_is_failure() {
        assert!(tool_dispatch_message_is_error(
            "FETCH_URL",
            "FETCH_URL error: connection refused"
        ));
    }

    #[test]
    fn fetch_url_user_visible_soft_fail_counts() {
        assert!(tool_dispatch_message_is_error(
            "FETCH_URL",
            "That URL could not be fetched (connection or server error). Answer without that page."
        ));
    }

    #[test]
    fn fetch_url_401_short_circuit_counts() {
        assert!(tool_dispatch_message_is_error(
            "FETCH_URL",
            "That URL returned 401 Unauthorized. Do not try another URL. Answer based on what you know."
        ));
    }

    #[test]
    fn fetch_url_success_not_failure() {
        assert!(!tool_dispatch_message_is_error(
            "FETCH_URL",
            "Here is the page content:\n\ntitle"
        ));
    }

    #[test]
    fn browser_navigate_failed_is_failure() {
        assert!(tool_dispatch_message_is_error(
            "BROWSER_NAVIGATE",
            "BROWSER_NAVIGATE failed: bad URL"
        ));
    }

    #[test]
    fn unknown_tool_hint_is_failure() {
        assert!(tool_dispatch_message_is_error(
            "FOO",
            "Unknown tool \"FOO\". Use one of the available tools from the agent list."
        ));
    }
}
