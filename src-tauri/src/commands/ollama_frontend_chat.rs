//! Frontend (CPU window) Ollama chat Tauri commands.
//!
//! These commands handle the in-app chat UI: `ollama_chat_with_execution` and
//! `ollama_chat_continue_with_result`. Extracted from `ollama.rs`.

use serde::{Deserialize, Serialize};

use crate::commands::ollama_chat::{
    ollama_chat, send_ollama_chat_messages, send_ollama_chat_messages_streaming,
};
use crate::commands::ollama_config::{default_non_agent_system_prompt, ChatRequest};
use crate::commands::tool_parsing::parse_fetch_url_from_response;

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
    use crate::metrics::format_metrics_for_ai_context;
    use tracing::info;

    ensure_cpu_window_open();

    info!(
        "Ollama Chat with Execution: Starting for question: {}",
        request.question
    );

    // Include gathered metrics so the model can answer accurately when the user asks about CPU, RAM, etc.
    let metrics_block = format_metrics_for_ai_context();
    let context_message = format!("{}\n\nUser question: {}", metrics_block, request.question);

    // Get system prompt: use soul.md (~/.mac-stats/agents/soul.md or bundled default) + tools when not overridden
    let system_prompt = request
        .system_prompt
        .unwrap_or_else(default_non_agent_system_prompt);

    // Build messages array with conversation history
    let mut messages = vec![crate::ollama::ChatMessage {
        role: "system".to_string(),
        content: system_prompt.clone(),
        images: None,
    }];

    // Add conversation history if provided (exclude system messages - we already have one)
    if let Some(ref history) = request.conversation_history {
        for msg in history {
            if msg.role == "user" || msg.role == "assistant" {
                messages.push(msg.clone());
            }
        }
        info!(
            "Ollama Chat with Execution: Added {} messages from conversation history",
            history
                .iter()
                .filter(|m| m.role == "user" || m.role == "assistant")
                .count()
        );
    }

    // Add current user message
    messages.push(crate::ollama::ChatMessage {
        role: "user".to_string(),
        content: context_message.clone(),
        images: None,
    });

    info!("Ollama Chat with Execution: Sending initial request to Ollama (stream={})", request.stream);
    let mut response = if request.stream {
        send_ollama_chat_messages_streaming(messages.clone(), None, None).await
    } else {
        send_ollama_chat_messages(messages.clone(), None, None).await
    }
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
        info!(
            "Ollama Chat with Execution: Fetched {} chars from {}",
            page_content.len(),
            url
        );

        // Build follow-up: current messages + assistant's FETCH_URL message + user with page content
        let mut follow_up_messages = messages.clone();
        follow_up_messages.push(crate::ollama::ChatMessage {
            role: "assistant".to_string(),
            content: response_content.clone(),
            images: None,
        });
        follow_up_messages.push(crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: format!(
                "Here is the page content:\n\n{}\n\nPlease answer the user's question based on this content.",
                page_content
            ),
            images: None,
        });

        let follow_up_request = ChatRequest {
            messages: follow_up_messages.clone(),
        };
        response = ollama_chat(follow_up_request)
            .await
            .map_err(|e| format!("Failed to send follow-up after fetch: {}", e))?;
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

    if let Some(code) = crate::commands::tool_parsing::detect_and_extract_js_code(&processed_content) {
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
            });
        }

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
    pub context_message: Option<String>,
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

    info!(
        "Ollama Chat Continue: Code executed, result: {}",
        execution_result
    );

    let system_prompt = system_prompt.unwrap_or_else(default_non_agent_system_prompt);

    let follow_up_message = format!(
        "I have executed your last codeblocks and the result is: {}\n\nCan you now answer the original question: {}?",
        execution_result, original_question
    );

    // Build messages array with conversation history
    let mut messages = vec![crate::ollama::ChatMessage {
        role: "system".to_string(),
        content: system_prompt.clone(),
        images: None,
    }];

    // Add conversation history if provided (exclude system messages - we already have one)
    if let Some(ref history) = conversation_history {
        for msg in history {
            if msg.role == "user" || msg.role == "assistant" {
                messages.push(msg.clone());
            }
        }
        info!(
            "Ollama Chat Continue: Added {} messages from conversation history",
            history
                .iter()
                .filter(|m| m.role == "user" || m.role == "assistant")
                .count()
        );
    }

    // Add the conversation flow for this code execution cycle
    messages.push(crate::ollama::ChatMessage {
        role: "user".to_string(),
        content: context_message.clone(),
        images: None,
    });
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
    let response = ollama_chat(chat_request)
        .await
        .map_err(|e| format!("Failed to send follow-up: {}", e))?;

    let response_content = response.message.content;
    info!(
        "Ollama Chat Continue: Received response ({} chars)",
        response_content.len()
    );

    // Process response content - handle escaped newlines
    let mut processed_content = response_content.replace("\\n", "\n");
    processed_content = processed_content.replace("javascript\n", "");

    if let Some(code) = crate::commands::tool_parsing::detect_and_extract_js_code(&processed_content) {
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
            });
        }

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
