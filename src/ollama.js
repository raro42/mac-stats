//! Ollama integration module
//! Handles all Ollama chat communication, code execution, and UI management

// Get invoke function safely (Tauri may not be ready when module loads)
function getInvoke() {
  if (typeof window.__TAURI__ !== 'undefined' && window.__TAURI__.core?.invoke) {
    return window.__TAURI__.core.invoke;
  }
  // Fallback for different Tauri versions
  if (typeof window.__TAURI_INVOKE__ !== 'undefined') {
    return window.__TAURI_INVOKE__;
  }
  throw new Error('Tauri invoke not available');
}

const invoke = (...args) => getInvoke()(...args);

// ============================================================================
// Configuration & State
// ============================================================================

function getOllamaEndpoint() {
  const saved = localStorage.getItem('ollama_endpoint');
  return saved || 'http://localhost:11434';
}

function saveOllamaEndpoint(endpoint) {
  localStorage.setItem('ollama_endpoint', endpoint);
}

function getDefaultModel() {
  return 'llama2';
}

function getSystemPrompt() {
  const saved = localStorage.getItem('ollama_system_prompt');
  return saved || 'You are a helpful assistant that answers questions about system metrics and monitoring.';
}

/**
 * Sanitize string for safe logging
 * - Truncates to specified max length (default 100)
 * - Removes/replaces dangerous characters (quotes, backticks, newlines, control chars)
 * - Prevents breaking log format or system
 */
function sanitizeForLogging(str, maxLength = 100) {
  if (str === null || str === undefined) {
    return String(str);
  }
  
  let sanitized = String(str);
  
  // Replace newlines and carriage returns with spaces
  sanitized = sanitized.replace(/[\r\n]+/g, ' ');
  
  // Replace tabs with spaces
  sanitized = sanitized.replace(/\t/g, ' ');
  
  // Remove control characters (except space)
  sanitized = sanitized.replace(/[\x00-\x1F\x7F]/g, '');
  
  // Replace dangerous quotes (double quotes, single quotes, backticks) with single quotes
  sanitized = sanitized.replace(/["'`]/g, "'");
  
  // Collapse multiple spaces
  sanitized = sanitized.replace(/\s+/g, ' ').trim();
  
  // Truncate to max length
  if (sanitized.length > maxLength) {
    sanitized = sanitized.substring(0, maxLength - 3) + '...';
  }
  
  return sanitized;
}

/**
 * Sanitize result string for safe logging (100 chars)
 */
function sanitizeResultForLogging(result) {
  return sanitizeForLogging(result, 100);
}

/**
 * Safely log JavaScript execution to Rust backend
 * Catches all errors and never throws to prevent breaking execution flow
 */
async function safeLogExecution(logData) {
  try {
    // Sanitize all string fields to prevent breaking the system
    const sanitizedLog = {
      code: sanitizeForLogging(logData.code || '', 200), // Code can be longer, but still limit it
      result: sanitizeResultForLogging(logData.result || ''),
      result_type: sanitizeForLogging(logData.result_type || 'unknown', 50),
      is_undefined: Boolean(logData.is_undefined),
      success: Boolean(logData.success),
      error_name: logData.error_name ? sanitizeForLogging(logData.error_name, 100) : null,
      error_message: logData.error_message ? sanitizeForLogging(logData.error_message, 200) : null,
      error_stack: logData.error_stack ? sanitizeForLogging(logData.error_stack, 500) : null
    };
    
    await invoke('log_ollama_js_execution', sanitizedLog);
  } catch (logErr) {
    // Silently catch and log to console only - never throw
    // This ensures logging failures never break the main execution flow
    console.warn('[Ollama] Failed to log execution (non-fatal):', logErr?.message || logErr);
  }
}

// ============================================================================
// Connection Management
// ============================================================================

/**
 * Auto-configure Ollama with default endpoint and model if not already configured
 */
async function autoConfigureOllama() {
  const endpoint = getOllamaEndpoint();
  const defaultModel = getDefaultModel();
  
  console.log('[Ollama] Auto-configuring with endpoint:', endpoint, 'model:', defaultModel);
  
  try {
    await configureOllama(endpoint, defaultModel);
    console.log('[Ollama] Auto-configuration successful');
    return true;
  } catch (err) {
    console.error('[Ollama] Failed to auto-configure:', err);
    return false;
  }
}

/**
 * Check Ollama connection status and update UI
 * Auto-configures Ollama if not already configured
 */
async function checkOllamaConnection() {
  const statusEl = document.getElementById('ollama-status');
  const connectionIndicator = document.getElementById('ollama-connection-indicator');
  const chat = document.getElementById('ollama-chat');
  
  if (!statusEl && !connectionIndicator) return;

  try {
    // First check if Ollama is configured by trying to check connection
    let connected = false;
    let connectionError = null;
    let connectionFailed = false; // Track if connection actually failed (not just not configured)
    
    try {
      connected = await invoke('check_ollama_connection');
    } catch (err) {
      // If invoke fails, it could be:
      // 1. Ollama not configured (returns false, not an error)
      // 2. Connection failed (network error, Ollama not running)
      connectionError = err;
      connectionFailed = true; // This is a real error, not just "not configured"
      console.log('[Ollama] Connection check failed with error:', err);
    }
    
    // If not connected and no error, it might be because Ollama isn't configured yet
    // Try to auto-configure and then check again
    if (!connected && !connectionFailed) {
      console.log('[Ollama] Connection check returned false, attempting auto-configuration...');
      try {
        const configured = await autoConfigureOllama();
        if (configured) {
          // Wait a bit for configuration to take effect
          await new Promise(resolve => setTimeout(resolve, 200));
          // Check connection again after auto-configuration
          try {
            connected = await invoke('check_ollama_connection');
            if (!connected) {
              // Configuration succeeded but connection still fails - Ollama not running
              connectionFailed = true;
            }
          } catch (checkErr) {
            connectionError = checkErr;
            connectionFailed = true; // Connection failed after config - Ollama not running
            console.error('[Ollama] Connection check failed after auto-config:', checkErr);
          }
        } else {
          // Auto-configuration failed - likely Ollama not running/not installed
          connectionFailed = true;
        }
      } catch (configErr) {
        connectionError = configErr;
        connectionFailed = true; // Auto-config failed - Ollama not available
        console.error('[Ollama] Auto-configuration failed:', configErr);
      }
    }
    
    // Update status element (dashboard.js style)
    if (statusEl) {
      if (connected) {
        statusEl.textContent = 'Connected';
        statusEl.classList.add('connected');
        if (chat) chat.style.display = 'block';
      } else if (connectionFailed) {
        statusEl.textContent = 'Error: Ollama not available - Check if Ollama is running';
        statusEl.classList.remove('connected');
      } else {
        statusEl.textContent = 'Not connected - Configure in settings';
        statusEl.classList.remove('connected');
      }
    }
    
    // Update connection indicator (cpu.js style)
    if (connectionIndicator) {
      if (connected) {
        connectionIndicator.classList.add('connected');
        connectionIndicator.title = 'Connected';
        if (chat) chat.style.display = 'block';
      } else if (connectionFailed) {
        connectionIndicator.classList.remove('connected');
        connectionIndicator.title = 'Error: Ollama not available - Check if Ollama is running';
      } else {
        connectionIndicator.classList.remove('connected');
        connectionIndicator.title = 'Not connected - Click to configure Ollama URL';
      }
    }
    
    // Update icon status: connected (green), error (yellow), or unknown (grey)
    if (typeof window.updateOllamaIconStatus === 'function') {
      if (connected) {
        window.updateOllamaIconStatus('connected');
      } else if (connectionFailed) {
        window.updateOllamaIconStatus('error');
      } else {
        window.updateOllamaIconStatus('unknown');
      }
    }
    
    return connected;
  } catch (err) {
    console.error('[Ollama] Failed to check connection:', err);
    
    // Set error state on icon if available (for CPU window) - yellow
    if (typeof window.updateOllamaIconStatus === 'function') {
      window.updateOllamaIconStatus('error');
    }
    
    if (statusEl) {
      statusEl.textContent = 'Error: Ollama not available - Check if Ollama is running';
      statusEl.classList.remove('connected');
    }
    if (connectionIndicator) {
      connectionIndicator.classList.remove('connected');
      connectionIndicator.title = 'Error: Ollama not available - Check if Ollama is running';
    }
    return false;
  }
}

/**
 * Configure Ollama endpoint and model
 */
async function configureOllama(endpoint, model, apiKeyKeychainAccount = null) {
  try {
    await invoke('configure_ollama', {
      config: {
        endpoint: endpoint || getOllamaEndpoint(),
        model: model || getDefaultModel(),
        api_key_keychain_account: apiKeyKeychainAccount
      }
    });
    if (endpoint) {
      saveOllamaEndpoint(endpoint);
    }
    return true;
  } catch (err) {
    console.error('[Ollama] Failed to configure:', err);
    throw err;
  }
}

/**
 * Show dialog to configure Ollama URL
 */
async function showOllamaUrlDialog() {
  const currentEndpoint = getOllamaEndpoint();
  const url = prompt('Enter Ollama endpoint URL:', currentEndpoint);
  
  if (!url) return; // User cancelled

  // Validate URL format
  try {
    new URL(url);
  } catch (e) {
    alert('Invalid URL format. Please enter a valid URL (e.g., http://localhost:11434)');
    return;
  }

  // Save and reconfigure
  saveOllamaEndpoint(url);
  const defaultModel = getDefaultModel();
  
  try {
    await configureOllama(url, defaultModel);
    await checkOllamaConnection();
  } catch (err) {
    alert(`Failed to configure Ollama: ${err}`);
  }
}

// ============================================================================
// Model Management
// ============================================================================

/**
 * Update Ollama model
 */
async function updateOllamaModel(model) {
  if (!model) return;
  
  const endpoint = getOllamaEndpoint();
  console.log('[Ollama] Updating model to:', model);
  
  try {
    await configureOllama(endpoint, model);
    console.log('[Ollama] Model updated successfully');
    return true;
  } catch (err) {
    console.error('[Ollama] Failed to update model:', err);
    return false;
  }
}

/**
 * Load available models from Ollama
 */
async function loadAvailableModels() {
  try {
    const models = await invoke('list_ollama_models');
    return models;
  } catch (err) {
    console.error('[Ollama] Failed to load models:', err);
    return [];
  }
}

// ============================================================================
// Chat Message Handling
// ============================================================================

// Conversation history storage (in-memory, per session)
let conversationHistory = [];

/**
 * Get conversation history
 */
function getConversationHistory() {
  return conversationHistory;
}

/**
 * Add message to conversation history
 */
function addToHistory(role, content) {
  conversationHistory.push({ role, content });
  // Limit history to last 20 messages to avoid token limits
  if (conversationHistory.length > 20) {
    conversationHistory = conversationHistory.slice(-20);
  }
}

/**
 * Clear conversation history
 */
function clearConversationHistory() {
  conversationHistory = [];
}

/**
 * Send chat message to Ollama using unified command
 */
async function sendChatMessage() {
  const input = document.getElementById('chat-input');
  const message = input?.value.trim();
  
  if (!message || !input) {
    console.log('[Ollama] Empty message or input not found');
    return;
  }

  console.log('[Ollama] ========== sendChatMessage() CALLED ==========');
  console.log('[Ollama] Message:', message);
  console.log('[Ollama] Conversation history length:', conversationHistory.length);

  // Add user message to chat
  addChatMessage('user', message);
  input.value = '';

  // Add user message to conversation history
  addToHistory('user', message);

  try {
    // Get system prompt
    const systemPrompt = getSystemPrompt();
    
    // Get conversation history (excluding the current message - it's in question)
    const history = getConversationHistory().slice(0, -1);
    
    // Use unified command that handles everything
    console.log('[Ollama] Sending request via unified command...');
    const response = await invoke('ollama_chat_with_execution', {
      request: {
        question: message,
        system_prompt: systemPrompt,
        conversation_history: history.length > 0 ? history : null
      }
    });
    
    console.log('[Ollama] ✅ Response received:', response);
    
    if (response.error) {
      addChatMessage('assistant', `Error: ${response.error}`);
      return;
    }
    
    if (response.needs_code_execution && response.code) {
      // Code needs to be executed - handle ping-pong (recursive execution)
      await executeCodeAndContinue(response, message, systemPrompt, 0);
    } else if (response.final_answer) {
      // Direct answer, no code execution needed
      addChatMessage('assistant', response.final_answer);
      // Add assistant response to history
      addToHistory('assistant', response.final_answer);
    } else {
      addChatMessage('assistant', 'Received unexpected response format');
    }
    
  } catch (err) {
    console.error('[Ollama] Failed to send chat message:', err);
    addChatMessage('assistant', `Error: ${err}`);
  }
}

/**
 * Execute code and continue with Ollama (handles ping-pong/recursive execution)
 * @param {Object} response - Response from ollama_chat_with_execution or ollama_chat_continue_with_result
 * @param {string} originalQuestion - Original user question
 * @param {string} systemPrompt - System prompt
 * @param {number} iteration - Current iteration (max 5 to prevent infinite loops)
 */
async function executeCodeAndContinue(response, originalQuestion, systemPrompt, iteration = 0) {
  const MAX_ITERATIONS = 5;
  
  if (iteration >= MAX_ITERATIONS) {
    addChatMessage('assistant', 'Error: Maximum code execution iterations reached. Ollama keeps requesting code execution.');
    return;
  }
  
  console.log(`[Ollama] Code execution iteration ${iteration + 1}/${MAX_ITERATIONS}`);
  console.log('[Ollama] Code to execute:', response.code);
  
  addChatMessage('assistant', `<em style="color: #666;">Executing code (step ${iteration + 1})...</em>`, true);
  
  try {
    // Execute the code
    const executionResult = await executeJavaScriptCode(response.code);
    
    // Format result
    // For strings, don't use JSON.stringify (it adds quotes)
    // For other types, use JSON.stringify for proper serialization
    let resultString;
    if (executionResult === undefined) {
      resultString = 'undefined';
    } else if (typeof executionResult === 'string') {
      // For strings, use directly (no quotes)
      resultString = executionResult;
    } else {
      try {
        resultString = JSON.stringify(executionResult);
      } catch (stringifyError) {
        resultString = String(executionResult);
      }
    }
    
    console.log(`[Ollama] Code executed (iteration ${iteration + 1}), result:`, resultString);
    
    // Log execution to Rust (safe logging - never throws)
    await safeLogExecution({
      code: response.code,
      result: resultString,
      result_type: typeof executionResult,
      is_undefined: executionResult === undefined,
      success: true,
      error_name: null,
      error_message: null,
      error_stack: null
    });
    
    // Update UI with code and result
    const messagesContainer = document.getElementById('chat-messages');
    const lastMessage = messagesContainer?.lastElementChild;
    if (lastMessage && lastMessage.textContent.includes('Executing code')) {
      const stepText = iteration > 0 ? ` (step ${iteration + 1})` : '';
      lastMessage.innerHTML = `<div style="margin: 8px 0; padding: 8px; background: #f9f9f9; border-left: 3px solid #4CAF50; border-radius: 4px;">
        <div style="font-size: 0.9em; color: #666; margin-bottom: 4px;">📝 Code executed${stepText}:</div>
        <pre style="background: #fff; padding: 6px; border-radius: 3px; margin: 4px 0; overflow-x: auto; font-size: 0.85em;"><code>${escapeHtml(response.code)}</code></pre>
        <div style="font-size: 0.9em; color: #666; margin-top: 8px; margin-bottom: 4px;">✅ Result:</div>
        <div><strong style="color: #4CAF50;">${escapeHtml(resultString)}</strong></div>
        <div style="font-size: 0.9em; color: #666; margin-top: 8px;">⏳ Getting response from AI...</div>
      </div>`;
    }
    
    // Continue with result (may trigger another code execution - ping-pong)
    console.log(`[Ollama] Sending execution result to Ollama (iteration ${iteration + 1})...`);
    try {
      // Get conversation history for context
      const history = getConversationHistory();
      
      const continueResponse = await invoke('ollama_chat_continue_with_result', {
        code: response.code,
        executionResult: resultString,
        originalQuestion: originalQuestion,
        contextMessage: response.context_message || '',
        intermediateResponse: response.intermediate_response || '',
        systemPrompt: systemPrompt,
        conversationHistory: history.length > 0 ? history : null
      });
      
      console.log(`[Ollama] Continue response (iteration ${iteration + 1}):`, continueResponse);
      
      // Check if Ollama wants more code execution (ping-pong)
      if (continueResponse.needs_code_execution && continueResponse.code) {
        console.log(`[Ollama] Ping-pong detected! Ollama wants more code execution (iteration ${iteration + 1})`);
        
        // Remove "Getting response" message
        if (lastMessage && lastMessage.textContent.includes('Getting response')) {
          messagesContainer.removeChild(lastMessage);
        }
        
        // Recursively execute the new code
        await executeCodeAndContinue(continueResponse, originalQuestion, systemPrompt, iteration + 1);
      } else if (continueResponse.final_answer) {
        // Final answer received
        console.log(`[Ollama] Final answer received after ${iteration + 1} iteration(s)`);
        
        // Remove intermediate message and show final answer
        if (lastMessage && lastMessage.textContent.includes('Getting response')) {
          messagesContainer.removeChild(lastMessage);
        }
        
        addChatMessage('assistant', continueResponse.final_answer);
        // Add assistant response to history
        addToHistory('assistant', continueResponse.final_answer);
      } else {
        // Unexpected response format
        console.warn('[Ollama] Unexpected continue response format:', continueResponse);
        addChatMessage('assistant', 'Received unexpected response format from Ollama.');
      }
    } catch (continueError) {
      console.error('[Ollama] Error in continue_with_result:', continueError);
      const errorMsg = continueError?.message || continueError?.toString() || String(continueError) || 'Unknown error';
      addChatMessage('assistant', `Error getting response: ${errorMsg}`);
      throw continueError; // Re-throw to be caught by outer catch
    }
    
  } catch (error) {
    console.error(`[Ollama] Error in code execution (iteration ${iteration + 1}):`, error);
    const errorMsg = error?.message || error?.toString() || String(error) || 'Unknown error';
    addChatMessage('assistant', `Error executing code: ${errorMsg}`);
    
    // Log error to Rust (safe logging - never throws)
    await safeLogExecution({
      code: response.code,
      result: `ERROR: ${error.name || 'Error'}: ${error.message || errorMsg}`,
      result_type: 'error',
      is_undefined: false,
      success: false,
      error_name: error.name || null,
      error_message: error.message || null,
      error_stack: error.stack || null
    });
  }
}

/**
 * Process Ollama response - handle ROLE=code-assistant and code execution
 */
async function processOllamaResponse(response, originalMessage, contextMessage) {
  console.log('[Ollama] ========== RAW RESPONSE RECEIVED ==========');
  console.log('[Ollama] Full response object:', response);
  
  let responseContent = response.message.content;
  
  if (!responseContent) {
    console.error('[Ollama] ERROR: response.message.content is null/undefined!');
    addChatMessage('assistant', 'Error: Received empty response from Ollama.');
    return;
  }
  
  // Handle escaped newlines (from JSON stringification)
  const originalContent = responseContent;
  responseContent = responseContent.replace(/\\n/g, '\n');
  
  // Remove "javascript\n" if present (Ollama sometimes includes this as text)
  responseContent = responseContent.replace(/javascript\n/gi, '');
  
  console.log('[Ollama] ========== PARSING RESPONSE ==========');
  console.log('[Ollama] Original content:', JSON.stringify(originalContent));
  console.log('[Ollama] Processed content:', JSON.stringify(responseContent));
  
  // Check if Ollama is asking us to execute code (ROLE=code-assistant pattern)
  const trimmedContent = responseContent.trim();
  const isCodeAssistant = trimmedContent.startsWith("ROLE=code-assistant") || 
                          /^ROLE=code-assistant/i.test(trimmedContent);
  
  console.log('[Ollama] Is code-assistant response?', isCodeAssistant);
  
  if (isCodeAssistant) {
    console.log('[Ollama JS Execution] ✅✅✅ DETECTED ROLE=code-assistant response ✅✅✅');
    
    // Show intermediate message
    addChatMessage('assistant', '<em style="color: #666;">Executing code to gather information...</em>', true);
    
    // Extract code (everything after the first line)
    const lines = responseContent.split(/\r?\n/);
    let code;
    
    if (lines.length >= 2) {
      code = lines.slice(1).join('\n').trim();
    } else {
      code = responseContent.replace(/^ROLE=code-assistant\s*/i, '').trim();
    }
    
    // Remove markdown code block markers (```javascript, ```, etc.)
    code = code.replace(/^```[\w]*\n?/g, '').replace(/\n?```$/g, '').trim();
    
    console.log('[Ollama JS Execution] Extracted code:', code);
    
    if (!code || code.length === 0) {
      console.error('[Ollama JS Execution] ERROR: No code found to execute');
      addChatMessage('assistant', 'Error: No code found in code-assistant response.');
      return;
    }
    
    // Log to Rust
    try {
      await invoke('log_ollama_js_check', {
        responseContent: responseContent,
        responseLength: responseContent.length
      });
      await invoke('log_ollama_js_extraction', {
        foundBlocks: 1,
        blocks: [code]
      });
    } catch (logErr) {
      console.warn('[Ollama JS Execution] Failed to log to backend:', logErr);
    }
    
    // Execute the code
    try {
      console.log('[Ollama JS Execution] Executing code now...');
      const executionResult = await executeJavaScriptCode(code);
      
      // Format result as string
      // For strings, don't use JSON.stringify (it adds quotes)
      // For other types, use JSON.stringify for proper serialization
      let resultString;
      if (executionResult === undefined) {
        resultString = 'undefined';
      } else if (typeof executionResult === 'string') {
        // For strings, use directly (no quotes)
        resultString = executionResult;
      } else {
        try {
          resultString = JSON.stringify(executionResult);
        } catch (stringifyError) {
          resultString = String(executionResult);
        }
      }
      
      console.log('[Ollama JS Execution] Code executed successfully, result:', resultString);
      console.log('[Ollama JS Execution] Logging execution result to Rust backend...');
      
      // Log execution to Rust (safe logging - never throws)
      await safeLogExecution({
        code: code,
        result: resultString,
        result_type: typeof executionResult,
        is_undefined: executionResult === undefined,
        success: true,
        error_name: null,
        error_message: null,
        error_stack: null
      });
      console.log('[Ollama JS Execution] Logged execution result to Rust backend');
      
      // Send result back to Ollama with original question
      const followUpMessage = `I have executed your last codeblocks and the result is: ${resultString}

Can you now answer the original question: ${originalMessage}?`;
      
      console.log('[Ollama JS Execution] Sending follow-up to Ollama with result');
      
      // Update the intermediate message to show we got the result
      const messagesContainer = document.getElementById('chat-messages');
      const lastMessage = messagesContainer?.lastElementChild;
      if (lastMessage && lastMessage.textContent.includes('Executing code')) {
        lastMessage.innerHTML = `<div style="margin: 8px 0; padding: 8px; background: #f9f9f9; border-left: 3px solid #4CAF50; border-radius: 4px;">
          <div style="font-size: 0.9em; color: #666; margin-bottom: 4px;">📝 Code executed:</div>
          <pre style="background: #fff; padding: 6px; border-radius: 3px; margin: 4px 0; overflow-x: auto; font-size: 0.85em;"><code>${escapeHtml(code)}</code></pre>
          <div style="font-size: 0.9em; color: #666; margin-top: 8px; margin-bottom: 4px;">✅ Result:</div>
          <div><strong style="color: #4CAF50;">${escapeHtml(resultString)}</strong></div>
          <div style="font-size: 0.9em; color: #666; margin-top: 8px;">⏳ Getting final answer from AI...</div>
        </div>`;
      }
      
      // Send follow-up to Ollama
      const systemPrompt = getSystemPrompt();
      const followUpResponse = await invoke('ollama_chat', {
        request: {
          messages: [
            { role: 'system', content: systemPrompt },
            { role: 'user', content: contextMessage },
            { role: 'assistant', content: responseContent },
            { role: 'user', content: followUpMessage }
          ]
        }
      });
      
      // Display the final answer
      const finalAnswer = followUpResponse.message.content;
      
      // Remove the intermediate message and add the final answer
      if (lastMessage && lastMessage.textContent.includes('Getting final answer')) {
        messagesContainer.removeChild(lastMessage);
      }
      
      addChatMessage('assistant', finalAnswer);
      
    } catch (error) {
      console.error('[Ollama JS Execution] ERROR executing code:', error);
      
      // Log error to Rust (safe logging - never throws)
      await safeLogExecution({
        code: code,
        result: `ERROR: ${error.name}: ${error.message}`,
        result_type: 'error',
        is_undefined: false,
        success: false,
        error_name: error.name || null,
        error_message: error.message || null,
        error_stack: error.stack || null
      });
      
      // Show error to user
      addChatMessage('assistant', `Error executing code: ${error.name}: ${error.message}`);
    }
    
    return; // Exit early, we've handled the code-assistant response
  }
  
  // No code execution needed, just display the response
  console.log('[Ollama] Regular response (no code execution)');
  addChatMessage('assistant', responseContent);
}

// ============================================================================
// JavaScript Code Execution
// ============================================================================

/**
 * Execute JavaScript code safely and return the result
 * Handles both sync and async code, expressions and statements
 */
async function executeJavaScriptCode(code) {
  const trimmedCode = code.trim();
  
  console.log('[Ollama JS Execution] Executing code:', trimmedCode);
  
  try {
    let result;
    
    // First, try to execute as an expression (most common case for single-line code)
    // Wrap in return statement to capture the value
    try {
      // Try as direct expression with return
      const func = new Function('return (' + trimmedCode + ')');
      result = func();
      console.log('[Ollama JS Execution] Executed as expression, result:', result, 'type:', typeof result);
    } catch (exprError) {
      console.log('[Ollama JS Execution] Expression execution failed, trying as statement with return wrapper');
      // If that fails, try wrapping the entire code in a return statement
      // This handles cases like: new Date().toLocaleDateString()
      try {
        // Remove trailing semicolon if present
        const codeWithoutSemicolon = trimmedCode.replace(/;+$/, '');
        const func = new Function('return ' + codeWithoutSemicolon);
        result = func();
        console.log('[Ollama JS Execution] Executed with return wrapper, result:', result, 'type:', typeof result);
      } catch (returnError) {
        // If that also fails, try as a statement (but this won't return a value)
        console.log('[Ollama JS Execution] Return wrapper failed, trying as statement');
        const func = new Function(trimmedCode);
        result = func();
        console.log('[Ollama JS Execution] Executed as statement, result:', result, 'type:', typeof result);
        
        // If result is undefined, the code executed but didn't return anything
        // This is a problem - we need to evaluate it as an expression
        if (result === undefined) {
          console.warn('[Ollama JS Execution] Statement execution returned undefined, trying eval as fallback');
          // Last resort: use eval to evaluate as expression
          try {
            result = eval('(' + trimmedCode + ')');
            console.log('[Ollama JS Execution] Eval succeeded, result:', result, 'type:', typeof result);
          } catch (evalError) {
            throw new Error(`Code executed but returned undefined. The code may need to be an expression that returns a value. Error: ${evalError.message}`);
          }
        }
      }
    }
    
    // If result is a Promise, await it
    if (result instanceof Promise) {
      console.log('[Ollama JS Execution] Result is a Promise, awaiting...');
      result = await result;
      console.log('[Ollama JS Execution] Promise resolved, result:', result);
    }
    
    console.log('[Ollama JS Execution] Final result:', result, 'type:', typeof result, 'isUndefined:', result === undefined);
    return result;
  } catch (error) {
    console.error('[Ollama JS Execution] Execution failed:', error);
    throw new Error(`Failed to execute JavaScript code: ${error.message}\nCode: ${trimmedCode}`);
  }
}

// ============================================================================
// UI Helpers
// ============================================================================

/**
 * Add a chat message to the chat container
 */
function addChatMessage(role, content, isHtml = false) {
  const messagesContainer = document.getElementById('chat-messages');
  if (!messagesContainer) {
    console.warn('[Ollama] chat-messages container not found');
    return;
  }
  
  const messageDiv = document.createElement('div');
  messageDiv.className = `chat-message ${role}`;
  
  if (isHtml) {
    messageDiv.innerHTML = content;
  } else {
    messageDiv.textContent = content;
  }
  
  messagesContainer.appendChild(messageDiv);
  messagesContainer.scrollTop = messagesContainer.scrollHeight;
}

/**
 * Escape HTML special characters
 */
function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

// ============================================================================
// Event Listeners Setup
// ============================================================================

/**
 * Initialize Ollama chat event listeners
 */
function initOllamaChatListeners() {
  const chatInput = document.getElementById('chat-input');
  const chatSendBtn = document.getElementById('chat-send-btn');
  
  if (!chatInput || !chatSendBtn) {
    console.warn('[Ollama] Chat input or send button not found');
    return;
  }
  
  // Send button click
  chatSendBtn.addEventListener('click', () => {
    console.log('[Ollama] Send button clicked');
    sendChatMessage();
  });
  
  // Enter key in input
  chatInput.addEventListener('keypress', (e) => {
    if (e.key === 'Enter') {
      console.log('[Ollama] Enter key pressed');
      sendChatMessage();
    }
  });
  
  console.log('[Ollama] Chat event listeners initialized');
}

// ============================================================================
// Auto-initialize on module load
// ============================================================================
// Auto-configure Ollama when the module loads (if DOM is ready)
async function initializeOllama() {
  // Auto-configure after a short delay to ensure everything is ready
  console.log('[Ollama] Module loaded, auto-configuring...');
  try {
    // Always auto-configure the backend, regardless of DOM elements
    await autoConfigureOllama();
    // Check connection after auto-configuration (this will update UI if elements exist)
    setTimeout(() => {
      checkOllamaConnection();
    }, 200);
  } catch (err) {
    console.error('[Ollama] Failed to initialize:', err);
  }
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', () => {
    setTimeout(initializeOllama, 100);
  });
} else {
  // DOM already loaded, initialize immediately
  setTimeout(initializeOllama, 100);
}

// ============================================================================
// Exports
// ============================================================================
// Export functions for use in other modules
window.Ollama = {
  // Connection
  checkConnection: checkOllamaConnection,
  configure: configureOllama,
  showUrlDialog: showOllamaUrlDialog,
  autoConfigure: autoConfigureOllama,
  
  // Models
  updateModel: updateOllamaModel,
  loadModels: loadAvailableModels,
  
  // Chat
  sendMessage: sendChatMessage,
  processResponse: processOllamaResponse,
  getHistory: getConversationHistory,
  clearHistory: clearConversationHistory,
  
  // Code execution
  executeCode: executeJavaScriptCode,
  
  // UI
  addMessage: addChatMessage,
  initListeners: initOllamaChatListeners,
  
  // Utils
  getEndpoint: getOllamaEndpoint,
  getSystemPrompt: getSystemPrompt,
  escapeHtml: escapeHtml
};
