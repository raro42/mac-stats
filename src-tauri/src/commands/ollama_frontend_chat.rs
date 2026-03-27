//! Frontend (CPU window) Ollama chat Tauri commands.
//!
//! These commands handle the in-app chat UI: `ollama_chat_with_execution` and
//! `ollama_chat_continue_with_result`. Extracted from `ollama.rs`.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::commands::content_reduction::CHARS_PER_TOKEN;
use crate::commands::context_assembler::{fragments, ContextAssembler, FrontendContextAssembler};
use crate::commands::ollama_chat::{
    ollama_chat, send_ollama_chat_messages, send_ollama_chat_messages_streaming, OllamaHttpQueue,
};
use crate::commands::ollama_config::{default_non_agent_system_prompt, ChatRequest};
use crate::commands::session_history::{
    prepare_conversation_history, CompactionLifecycleContext, CONVERSATION_HISTORY_CAP,
};
use crate::commands::tool_parsing::parse_fetch_url_from_response;

/// Primary in-app chat should see what the scheduler already posted to Discord (authoritative log + this block).
fn augment_cpu_system_with_scheduler_awareness(base: String) -> String {
    let block = crate::scheduler::delivery_awareness::format_for_chat_context();
    if block.is_empty() {
        base
    } else {
        debug!(
            target: "mac_stats::scheduler/delivery_awareness",
            block_chars = block.chars().count(),
            "CPU chat: prepending scheduler→Discord delivery awareness to system prompt"
        );
        format!("{}\n\n{}", base, block)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaChatWithExecutionRequest {
    pub question: String,
    pub system_prompt: Option<String>,
    pub conversation_history: Option<Vec<crate::ollama::ChatMessage>>,
    /// When true (default), stream response chunks to frontend via "ollama-chat-chunk" for better UX.
    #[serde(default = "default_stream_true")]
    pub stream: bool,
}

fn default_stream_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaChatWithExecutionResponse {
    pub needs_code_execution: bool,
    pub code: Option<String>,
    pub intermediate_response: Option<String>,
    pub final_answer: Option<String>,
    pub error: Option<String>,
    pub context_message: Option<String>,
    /// Paths the app would attach on remote/Discord-style runs (CPU chat path is usually empty).
    #[serde(default)]
    pub attachment_paths: Vec<String>,
}

/// If the CPU window is not open, schedule opening or showing it on the main thread so the user can see the chat.
fn ensure_cpu_window_open() {
    use crate::state::APP_HANDLE;
    use crate::ui::status_bar::create_cpu_window;
    use tauri::Manager;

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

    ensure_cpu_window_open();

    let q_lower = request.question.to_lowercase();
    let via_new_session_prefix = q_lower.trim_start().starts_with("new session:");
    let did_session_reset_phrase =
        crate::session_memory::user_wants_session_reset(&request.question);
    if via_new_session_prefix || did_session_reset_phrase {
        let coord_reset = crate::commands::turn_lifecycle::coordination_key(None);
        crate::commands::abort_cutoff::clear_cutoff(coord_reset);
    }

    let coord_ui = crate::commands::turn_lifecycle::coordination_key(None);
    let ui_event_ts = Utc::now();
    let ui_event_id = format!(
        "cpu-ui-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    if crate::commands::abort_cutoff::should_skip(coord_ui, &ui_event_id, ui_event_ts) {
        return Err(
            "Chat request skipped (stale vs a recent abort on the non-Discord session slot)."
                .to_string(),
        );
    }

    crate::keyed_queue::run_serial("ui-chat", async move {
    info!(
        "Ollama Chat with Execution: Starting for question: {}",
        request.question
    );

    let token_budget =
        crate::commands::context_assembler::resolve_default_chat_context_token_budget().await;

    // Include gathered metrics so the model can answer accurately when the user asks about CPU, RAM, etc.
    let context_message = fragments::cpu_window_user_turn_with_metrics(&request.question);

    // Get system prompt: use soul.md (~/.mac-stats/agents/soul.md or bundled default) + tools when not overridden
    let system_prompt = augment_cpu_system_with_scheduler_awareness(
        request
            .system_prompt
            .unwrap_or_else(default_non_agent_system_prompt),
    );

    let raw_prior: Vec<crate::ollama::ChatMessage> = request
        .conversation_history
        .as_ref()
        .map(|h| {
            h.iter()
                .filter(|m| m.role == "user" || m.role == "assistant")
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    let raw_prior = ContextAssembler::compact(&FrontendContextAssembler, raw_prior);

    let request_id = format!(
        "cpu-chat-{:08x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
            & 0xFFFF_FFFF
    );

    let compacted_prior = prepare_conversation_history(
        raw_prior,
        &request.question,
        false,
        None,
        &request_id,
        CompactionLifecycleContext {
            hook_source: "cpu".to_string(),
            hook_session_id: 0,
            emit_cpu_compaction_ui: true,
        },
    )
    .await;

    info!(
        "Ollama Chat with Execution: prior turns after prepare ({} messages, cap {})",
        compacted_prior.len(),
        CONVERSATION_HISTORY_CAP
    );

    let mut messages = ContextAssembler::assemble(
        &FrontendContextAssembler,
        &compacted_prior,
        &system_prompt,
        crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: context_message.clone(),
            images: None,
        },
        token_budget,
    );

    info!(
        "Ollama Chat with Execution: Sending initial request to Ollama (stream={})",
        request.stream
    );
    let cpu_q = OllamaHttpQueue::Acquire {
        key: "cpu_ui".to_string(),
        wait_hook: None,
    };
    let mut response = if request.stream {
        send_ollama_chat_messages_streaming(messages.clone(), None, None, cpu_q.clone()).await
    } else {
        send_ollama_chat_messages(messages.clone(), None, None, cpu_q).await
    }
    .map_err(|e| {
        crate::commands::content_reduction::sanitize_ollama_error_for_user(&e)
            .unwrap_or_else(|| format!("Failed to send chat request: {}", e))
    })?;

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

        let raw_content = crate::commands::browser::fetch_page_content_for_agent(&url)
            .map_err(|e| format!("Fetch page failed: {}", e))?;
        let original_len = raw_content.len();
        let page_content = crate::commands::html_cleaning::clean_html(&raw_content);
        let cleaned_len = page_content.len();
        if original_len > 0 {
            let ratio = (cleaned_len as f64 / original_len as f64 * 100.0) as u32;
            info!(
                "Ollama Chat: FETCH_URL HTML cleaned {} → {} bytes ({}%)",
                original_len, cleaned_len, ratio
            );
        }
        let page_content = if page_content.trim().is_empty() {
            "Page fetched but no readable text content found (page may require JavaScript rendering). Try BROWSER_NAVIGATE instead.".to_string()
        } else {
            crate::commands::text_normalize::apply_untrusted_homoglyph_normalization(page_content)
        };

        crate::commands::suspicious_patterns::log_untrusted_suspicious_scan(
            "fetched-page",
            &page_content,
        );

        let mut follow_up_messages = messages.clone();
        follow_up_messages.push(crate::ollama::ChatMessage {
            role: "assistant".to_string(),
            content: crate::commands::directive_tags::strip_inline_directive_tags_for_display(
                &response_content,
            ),
            images: None,
        });
        follow_up_messages.push(crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: format!(
                "Here is the page content:\n\n{}\n\nPlease answer the user's question based on this content.",
                crate::commands::untrusted_content::wrap_untrusted_content("fetched-page", &page_content)
            ),
            images: None,
        });

        let follow_up_request = ChatRequest {
            messages: follow_up_messages.clone(),
        };
        response = ollama_chat(follow_up_request).await.map_err(|e| {
            crate::commands::content_reduction::sanitize_ollama_error_for_user(&e)
                .unwrap_or_else(|| format!("Failed to send follow-up after fetch: {}", e))
        })?;
        response_content = response.message.content.clone();
        messages = follow_up_messages;
        info!(
            "Ollama Chat with Execution: Received response after fetch ({} chars)",
            response_content.len()
        );
    }

    info!(
        "Ollama Chat with Execution: Received response ({} chars)",
        response_content.len()
    );

    // Process response content - handle escaped newlines
    let mut processed_content = response_content.replace("\\n", "\n");
    processed_content = processed_content.replace("javascript\n", "");
    processed_content = crate::commands::directive_tags::strip_inline_directive_tags_for_display(
        &processed_content,
    );

    if let Some(code) =
        crate::commands::tool_parsing::detect_and_extract_js_code(&processed_content)
    {
        info!(
            "Ollama Chat with Execution: Extracted code ({} chars):\n{}",
            code.len(),
            code
        );

        if code.is_empty() {
            return Ok(OllamaChatWithExecutionResponse {
                needs_code_execution: false,
                code: None,
                intermediate_response: Some(processed_content),
                final_answer: None,
                error: Some("No code found in code-assistant response".to_string()),
                context_message: Some(context_message),
                attachment_paths: vec![],
            });
        }

        return Ok(OllamaChatWithExecutionResponse {
            needs_code_execution: true,
            code: Some(code),
            intermediate_response: Some(processed_content),
            final_answer: None,
            error: None,
            context_message: Some(context_message),
            attachment_paths: vec![],
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
        attachment_paths: vec![],
    })
    })
    .await
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaChatContinueResponse {
    pub needs_code_execution: bool,
    pub code: Option<String>,
    pub intermediate_response: Option<String>,
    pub final_answer: Option<String>,
    pub context_message: Option<String>,
    #[serde(default)]
    pub attachment_paths: Vec<String>,
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

    crate::keyed_queue::run_serial("ui-chat", async move {
    info!(
        "Ollama Chat Continue: Code executed, result: {}",
        execution_result
    );

    let token_budget =
        crate::commands::context_assembler::resolve_default_chat_context_token_budget().await;

    let system_prompt = augment_cpu_system_with_scheduler_awareness(
        system_prompt.unwrap_or_else(default_non_agent_system_prompt),
    );

    let follow_up_message = format!(
        "I have executed your last codeblocks and the result is: {}\n\nCan you now answer the original question: {}?",
        execution_result, original_question
    );
    let tail_extra_tokens = intermediate_response.chars().count() / CHARS_PER_TOKEN
        + follow_up_message.chars().count() / CHARS_PER_TOKEN
        + 128;
    let assemble_budget = token_budget.saturating_sub(tail_extra_tokens);

    let raw_prior: Vec<crate::ollama::ChatMessage> = conversation_history
        .as_ref()
        .map(|h| {
            h.iter()
                .filter(|m| m.role == "user" || m.role == "assistant")
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    let raw_prior = ContextAssembler::compact(&FrontendContextAssembler, raw_prior);

    let request_id = format!(
        "cpu-continue-{:08x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
            & 0xFFFF_FFFF
    );

    let compacted_prior = prepare_conversation_history(
        raw_prior,
        &original_question,
        false,
        None,
        &request_id,
        CompactionLifecycleContext {
            hook_source: "cpu".to_string(),
            hook_session_id: 0,
            emit_cpu_compaction_ui: true,
        },
    )
    .await;

    info!(
        "Ollama Chat Continue: prior turns after prepare ({} messages)",
        compacted_prior.len()
    );

    let mut messages = ContextAssembler::assemble(
        &FrontendContextAssembler,
        &compacted_prior,
        &system_prompt,
        crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: context_message.clone(),
            images: None,
        },
        assemble_budget.max(512),
    );
    messages.push(crate::ollama::ChatMessage {
        role: "assistant".to_string(),
        content: intermediate_response.clone(),
        images: None,
    });
    messages.push(crate::ollama::ChatMessage {
        role: "user".to_string(),
        content: follow_up_message,
        images: None,
    });

    let chat_request = ChatRequest { messages };

    info!("Ollama Chat Continue: Sending follow-up to Ollama");
    let response = ollama_chat(chat_request).await.map_err(|e| {
        crate::commands::content_reduction::sanitize_ollama_error_for_user(&e)
            .unwrap_or_else(|| format!("Failed to send follow-up: {}", e))
    })?;

    let response_content = response.message.content;
    info!(
        "Ollama Chat Continue: Received response ({} chars)",
        response_content.len()
    );

    // Process response content - handle escaped newlines
    let mut processed_content = response_content.replace("\\n", "\n");
    processed_content = processed_content.replace("javascript\n", "");
    processed_content = crate::commands::directive_tags::strip_inline_directive_tags_for_display(
        &processed_content,
    );

    if let Some(code) =
        crate::commands::tool_parsing::detect_and_extract_js_code(&processed_content)
    {
        info!(
            "Ollama Chat Continue: Extracted code for re-execution ({} chars):\n{}",
            code.len(),
            code
        );

        if code.is_empty() {
            return Ok(OllamaChatContinueResponse {
                needs_code_execution: false,
                code: None,
                intermediate_response: Some(processed_content),
                final_answer: None,
                context_message: Some(context_message),
                attachment_paths: vec![],
            });
        }

        return Ok(OllamaChatContinueResponse {
            needs_code_execution: true,
            code: Some(code),
            intermediate_response: Some(processed_content),
            final_answer: None,
            context_message: Some(context_message),
            attachment_paths: vec![],
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
        attachment_paths: vec![],
    })
    })
    .await
}
