//! Ollama Tauri commands

use crate::ollama::{OllamaClient, OllamaConfig, ChatMessage};
use serde::{Deserialize, Serialize};
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

/// Send chat message to Ollama (async, non-blocking)
#[tauri::command]
pub async fn ollama_chat(request: ChatRequest) -> Result<crate::ollama::ChatResponse, String> {
    use tracing::{debug, info};
    use serde_json;
    
    // Log raw request JSON
    let request_json = serde_json::to_string_pretty(&request)
        .unwrap_or_else(|_| "Failed to serialize request".to_string());
    info!("Ollama: Chat request JSON:\n{}", request_json);
    
    // Clone client data to avoid holding lock across await
    let (endpoint, model, api_key, messages) = {
        let client_guard = get_ollama_client().lock()
            .map_err(|e| e.to_string())?;

        let client = client_guard.as_ref()
            .ok_or_else(|| "Ollama not configured".to_string())?;
        
        (client.config.endpoint.clone(), 
         client.config.model.clone(),
         client.config.api_key.clone(),
         request.messages)
    };
    
    // Create a temporary client for this request (non-blocking)
    let temp_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let url = format!("{}/api/chat", endpoint);
    info!("Ollama: Using endpoint: {}", url);
    info!("Ollama: Streaming is disabled (stream: false)");
    let chat_request = crate::ollama::ChatRequest {
        model: model.clone(),
        messages: messages.clone(),
        stream: false,
    };
    
    let mut http_request = temp_client.post(&url).json(&chat_request);
    
    // Add API key if configured
    if let Some(keychain_account) = &api_key {
        if let Ok(Some(api_key_value)) = crate::security::get_credential(keychain_account) {
            let masked = crate::security::mask_credential(&api_key_value);
            http_request = http_request.header("Authorization", format!("Bearer {}", api_key_value));
            debug!("Ollama: Using API key for chat request (masked: {})", masked);
        }
    }

    let start_time = std::time::Instant::now();
    let response = http_request
        .send()
        .await
        .map_err(|e| {
            debug!("Ollama: Chat request failed: {}", e);
            format!("Failed to send chat request: {}", e)
        })?
        .json::<crate::ollama::ChatResponse>()
        .await
        .map_err(|e| {
            debug!("Ollama: Failed to parse response: {}", e);
            format!("Failed to parse response: {}", e)
        })?;
    let duration = start_time.elapsed();

    // Log raw response JSON
    let response_json = serde_json::to_string_pretty(&response)
        .unwrap_or_else(|_| "Failed to serialize response".to_string());
    info!("Ollama: Chat response JSON (duration: {:?}):\n{}", duration, response_json);

    Ok(response)
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

/// Unified Ollama chat command that handles code execution flow
/// This command:
/// 1. Gets system metrics
/// 2. Sends question to Ollama
/// 3. Detects if code needs execution
/// 4. Returns structured response
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
    
    // Get system prompt
    let system_prompt = request.system_prompt.unwrap_or_else(|| {
        "You are a general purpose AI. If you are asked for actual data like day or weather information, or flight information or stock information. Then we need to compile that information using speciallz crafted clients for doing so. You will put \"[variable-name]\" into the answer to signal that we need to go another step and ask and agent to fullfill the answer.\n\nWhenever asked with \"[variable-name]\", you must provide a javascript snipplet to be executed in the browser console to retrieve that information. Mark the answer to be executed as javascript. Do not put any other words around it. Do not insert formatting. Onlz return the code to be executed. This is needed for the next AI to understand and execute the same. When answering, use the role: code-assistant in the response. When you return executable code:\n- Start the response with: ROLE=code-assistant\n- On the next line, output ONLY executable JavaScript\n- Do not add explanations or formatting".to_string()
    });
    
    // Send initial request to Ollama
    let chat_request = ChatRequest {
        messages: vec![
            crate::ollama::ChatMessage {
                role: "system".to_string(),
                content: system_prompt.clone(),
            },
            crate::ollama::ChatMessage {
                role: "user".to_string(),
                content: context_message.clone(),
            },
        ],
    };
    
    info!("Ollama Chat with Execution: Sending initial request to Ollama");
    let response = ollama_chat(chat_request).await
        .map_err(|e| format!("Failed to send chat request: {}", e))?;
    
    let response_content = response.message.content;
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
    
    let chat_request = ChatRequest {
        messages: vec![
            crate::ollama::ChatMessage {
                role: "system".to_string(),
                content: system_prompt.clone(),
            },
            crate::ollama::ChatMessage {
                role: "user".to_string(),
                content: context_message.clone(),
            },
            crate::ollama::ChatMessage {
                role: "assistant".to_string(),
                content: intermediate_response.clone(),
            },
            crate::ollama::ChatMessage {
                role: "user".to_string(),
                content: follow_up_message,
            },
        ],
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
