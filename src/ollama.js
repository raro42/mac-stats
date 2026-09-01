//! Ollama integration module
//! Handles all Ollama chat communication, code execution, and UI management

// Get invoke function safely (Tauri may not be ready when module loads)
function getInvoke() {
  if (typeof window.__TAURI__ !== 'undefined' && window.__TAURI__.core?.invoke) {
    return window.__TAURI__.core.invoke;
  }
  // Tauri 2 can expose IPC here when `withGlobalTauri` is false (not recommended for this app).
  const internals = window.__TAURI_INTERNALS__;
  if (internals && typeof internals.invoke === 'function') {
    return internals.invoke.bind(internals);
  }
  // Fallback for different Tauri versions
  if (typeof window.__TAURI_INVOKE__ !== 'undefined') {
    return window.__TAURI_INVOKE__;
  }
  throw new Error('Tauri invoke not available');
}

const invoke = (...args) => getInvoke()(...args);

/** Get Tauri event listen if available (for streaming chat chunks). */
function getListen() {
  if (typeof window.__TAURI__ !== 'undefined' && window.__TAURI__.event?.listen) {
    return window.__TAURI__.event.listen;
  }
  return null;
}

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

/** Prefer gemma4 (etc.) over whatever Ollama returns first in /api/tags (often ornith). */
function pickPreferredModel(models) {
  if (!models || models.length === 0) return null;
  const prefs = ['gemma4:latest', 'gemma4', 'qwen3:latest', 'qwen3', 'qwen2.5-coder:latest'];
  for (const p of prefs) {
    if (models.includes(p)) return p;
  }
  for (const p of prefs) {
    const base = p.split(':')[0];
    const hit = models.find((m) => m === base || m.startsWith(`${base}:`));
    if (hit) return hit;
  }
  return models[0];
}

async function getDefaultModel() {
  const saved = localStorage.getItem('ollama_model');
  try {
    const models = await invoke('list_ollama_models');
    if (models && models.length > 0) {
      console.log('[Ollama] Detected models:', models.join(', '));
      const preferred = pickPreferredModel(models);
      // Keep an explicit user choice, but do not stick on tags[0] (e.g. ornith) when
      // a preferred model is installed — that was auto-picked, not selected.
      if (saved && models.includes(saved)) {
        if (preferred && saved === models[0] && saved !== preferred) {
          console.log('[Ollama] Upgrading auto-picked', saved, '→', preferred);
          localStorage.setItem('ollama_model', preferred);
          return preferred;
        }
        return saved;
      }
      if (saved) {
        console.warn(
          '[Ollama] Saved model not installed, ignoring:',
          saved,
          '→ using',
          preferred
        );
      }
      localStorage.setItem('ollama_model', preferred);
      return preferred;
    }
  } catch (_) { /* Ollama may not be reachable yet */ }
  return saved || 'gemma4:latest';
}

/** Returns custom system prompt from localStorage, or null if not set (backend then uses soul.md from ~/.mac-stats/agents/soul.md). */
function getSystemPrompt() {
  const saved = localStorage.getItem('ollama_system_prompt');
  return saved || null;
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
  const defaultModel = await getDefaultModel();
  
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

    const circuitOpen = await readOllamaCircuitOpen();
    // Circuit open blocks chat even when /api/tags still answers — prefer circuit cue.
    if (circuitOpen) {
      if (typeof window.updateOllamaIconStatus === 'function') {
        window.updateOllamaIconStatus('error');
      }
      if (statusEl) {
        statusEl.textContent = 'Ollama circuit open — retry soon';
        statusEl.classList.remove('connected');
      }
      if (connectionIndicator) {
        connectionIndicator.classList.remove('connected');
        connectionIndicator.title = 'Ollama circuit open — chat paused; retry soon';
      }
      chatModelGlanceState = {
        status: 'error',
        model: String(localStorage.getItem('ollama_model') || '').trim(),
        circuitOpen: true,
      };
      applyChatModelGlanceState();
      return false;
    }
    chatModelGlanceState = {
      status: connected ? 'connected' : connectionFailed ? 'error' : 'unknown',
      model: String(localStorage.getItem('ollama_model') || '').trim(),
      circuitOpen: false,
    };
    applyChatModelGlanceState();
    
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
    const circuitOpen = await readOllamaCircuitOpen();
    chatModelGlanceState = {
      status: 'error',
      model: String(localStorage.getItem('ollama_model') || '').trim(),
      circuitOpen,
    };
    applyChatModelGlanceState();
    return false;
  }
}

/**
 * Configure Ollama endpoint and model
 */
async function configureOllama(endpoint, model, apiKeyKeychainAccount = null) {
  try {
    let resolvedModel = model;
    if (!resolvedModel || typeof resolvedModel !== 'string') {
      resolvedModel = await getDefaultModel();
    } else {
      // Never leave a missing/stale model configured — chat would fail while
      // check_ollama_connection still returns true (it only probes /api/tags).
      try {
        const models = await invoke('list_ollama_models');
        if (models && models.length > 0 && !models.includes(resolvedModel)) {
          const fallback = pickPreferredModel(models) || models[0];
          console.warn(
            '[Ollama] Model not installed:',
            resolvedModel,
            '→ falling back to',
            fallback
          );
          resolvedModel = fallback;
          localStorage.setItem('ollama_model', resolvedModel);
        }
      } catch (_) { /* list may fail if endpoint down; keep requested model */ }
    }
    await invoke('configure_ollama', {
      config: {
        endpoint: endpoint || getOllamaEndpoint(),
        model: resolvedModel,
        api_key_keychain_account: apiKeyKeychainAccount
      }
    });
    if (endpoint) {
      saveOllamaEndpoint(endpoint);
    }
    if (resolvedModel) {
      localStorage.setItem('ollama_model', resolvedModel);
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
  const defaultModel = await getDefaultModel();
  
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
    if (chatModelGlanceState.status === 'connected') {
      chatModelGlanceState.model = String(model || '').trim();
    }
    applyChatModelGlanceState();
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
 * Add message to conversation history.
 * @param {string} role - 'user' | 'assistant'
 * @param {string} content - message text
 * @param {string[]|null|undefined} attachmentPaths - optional screenshot paths; recorded as `[screenshot: …]` lines (same convention as agent router)
 */
function addToHistory(role, content, attachmentPaths) {
  let stored = content;
  if (attachmentPaths && attachmentPaths.length > 0) {
    const lines = attachmentPaths.map((p) => `[screenshot: ${p}]`).join('\n');
    const sep =
      stored && !stored.endsWith('\n') ? '\n\n' : stored ? '\n' : '';
    stored = `${stored}${sep}${lines}`;
  }
  conversationHistory.push({ role, content: stored });
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
  chatFilterMode = 'all';
  const container = document.getElementById('chat-messages');
  if (container) {
    container.innerHTML = '';
    ensureChatEmptyHint();
    ensureChatMessagesKbHint(container, false);
  }
  applyChatListFilter();
  updateChatClearButton();
}

/** Transcript role filter (Monitors All/Up/Down/Slow parity). */
let chatFilterMode = 'all';

/** True when assistant bubble text is a failed turn (send / continue / JS). */
function isChatErrorText(text) {
  const t = String(text ?? '').trim();
  if (!t) return false;
  return (
    /^Error:/i.test(t) ||
    /^Error getting response:/i.test(t) ||
    /^Error executing code:/i.test(t)
  );
}

/** Mark (or clear) error wash on a chat bubble from class or copy text. */
function syncChatMessageErrorClass(el) {
  if (!el || !el.classList.contains('assistant')) {
    el?.classList.remove('is-error');
    return false;
  }
  if (el.classList.contains('thinking')) {
    el.classList.remove('is-error');
    return false;
  }
  const copy =
    (el.dataset && el.dataset.copyText) ||
    String(el.innerText || el.textContent || '').trim();
  const isErr = isChatErrorText(copy);
  el.classList.toggle('is-error', isErr);
  return isErr;
}

/** All · You · Assistant · Errors chips above the message list. */
function ensureChatFilterChips() {
  const chat = document.getElementById('ollama-chat');
  const messages = document.getElementById('chat-messages');
  if (!chat || !messages || !messages.parentNode) return;
  let wrap = document.getElementById('chat-filter-chips');
  if (!wrap) {
    wrap = document.createElement('div');
    wrap.id = 'chat-filter-chips';
    wrap.className = 'chat-filter-chips';
    wrap.setAttribute('role', 'group');
    wrap.setAttribute('aria-label', 'Chat message filter');
    wrap.hidden = true;
    wrap.innerHTML =
      '<button type="button" class="chat-filter-chip is-active" data-chat-filter="all" aria-pressed="true" title="Show every message">All</button>' +
      '<button type="button" class="chat-filter-chip" data-chat-filter="you" aria-pressed="false" title="Show your messages only">You <span class="chat-filter-count" data-chat-filter-count="you">0</span></button>' +
      '<button type="button" class="chat-filter-chip" data-chat-filter="assistant" aria-pressed="false" title="Show assistant replies only">Assistant <span class="chat-filter-count" data-chat-filter-count="assistant">0</span></button>' +
      '<button type="button" class="chat-filter-chip" data-chat-filter="errors" aria-pressed="false" title="Show failed turns only (Error: …)">Errors <span class="chat-filter-count" data-chat-filter-count="errors">0</span></button>';
    messages.parentNode.insertBefore(wrap, messages);
    wrap.addEventListener('click', (e) => {
      const btn = e.target && e.target.closest && e.target.closest('[data-chat-filter]');
      if (!btn || !wrap.contains(btn)) return;
      e.preventDefault();
      e.stopPropagation();
      setChatFilterMode(btn.getAttribute('data-chat-filter') || 'all');
    });
  } else if (!wrap.querySelector('[data-chat-filter="errors"]')) {
    const errBtn = document.createElement('button');
    errBtn.type = 'button';
    errBtn.className = 'chat-filter-chip';
    errBtn.setAttribute('data-chat-filter', 'errors');
    errBtn.setAttribute('aria-pressed', 'false');
    errBtn.title = 'Show failed turns only (Error: …)';
    errBtn.innerHTML =
      'Errors <span class="chat-filter-count" data-chat-filter-count="errors">0</span>';
    wrap.appendChild(errBtn);
  }
  if (typeof window.wireFilterChipToolbarKeyboard === 'function') {
    window.wireFilterChipToolbarKeyboard(wrap);
  }
}

function normalizeChatFilterMode(mode) {
  if (mode === 'you' || mode === 'assistant' || mode === 'errors') return mode;
  return 'all';
}

function setChatFilterMode(mode) {
  const next = normalizeChatFilterMode(mode);
  chatFilterMode = next;
  document.querySelectorAll('#chat-filter-chips [data-chat-filter]').forEach((btn) => {
    const on = btn.getAttribute('data-chat-filter') === next;
    btn.classList.toggle('is-active', on);
    btn.setAttribute('aria-pressed', on ? 'true' : 'false');
  });
  applyChatListFilter();
}

function chatFilterMissHint() {
  if (chatFilterMode === 'errors') {
    return 'No failed turns in this chat right now.';
  }
  if (chatFilterMode === 'you') {
    return 'No messages from you yet.';
  }
  if (chatFilterMode === 'assistant') {
    return 'No assistant replies yet.';
  }
  return 'Try All, or clear the role filter.';
}

function ensureChatFilterMissState(container, show) {
  if (!container) return;
  const existing = container.querySelector('.chat-filter-miss');
  if (!show) {
    existing?.remove();
    return;
  }
  let wrap = existing;
  if (!wrap) {
    wrap = document.createElement('div');
    wrap.className = 'chat-empty chat-filter-miss';
    wrap.setAttribute('role', 'status');
    wrap.innerHTML =
      `<div class="chat-empty-copy chat-filter-miss-msg">Nothing matches this filter</div>` +
      `<div class="chat-filter-miss-hint"></div>` +
      `<button type="button" class="chat-filter-miss-cta chat-clear-filter">Clear filter</button>`;
    container.appendChild(wrap);
    wrap.querySelector('.chat-clear-filter')?.addEventListener('click', (e) => {
      e.preventDefault();
      e.stopPropagation();
      setChatFilterMode('all');
    });
  }
  const hint = wrap.querySelector('.chat-filter-miss-hint');
  if (hint) hint.textContent = chatFilterMissHint();
}

function applyChatListFilter() {
  ensureChatFilterChips();
  const chips = document.getElementById('chat-filter-chips');
  const container = document.getElementById('chat-messages');
  if (!container) return;

  const items = Array.from(container.querySelectorAll('.chat-message'));
  const trueEmpty = !!container.querySelector('.chat-empty:not(.chat-filter-miss)');
  if (chips) chips.hidden = trueEmpty || items.length === 0;

  let youCount = 0;
  let assistantCount = 0;
  let errorsCount = 0;
  items.forEach((el) => {
    if (el.classList.contains('user')) youCount++;
    else if (el.classList.contains('assistant')) {
      assistantCount++;
      if (syncChatMessageErrorClass(el)) errorsCount++;
    }
  });

  const youEl = document.querySelector('[data-chat-filter-count="you"]');
  const asstEl = document.querySelector('[data-chat-filter-count="assistant"]');
  const errEl = document.querySelector('[data-chat-filter-count="errors"]');
  if (youEl) youEl.textContent = String(youCount);
  if (asstEl) asstEl.textContent = String(assistantCount);
  if (errEl) errEl.textContent = String(errorsCount);
  document.querySelectorAll('#chat-filter-chips [data-chat-filter]').forEach((btn) => {
    const key = btn.getAttribute('data-chat-filter');
    btn.classList.toggle(
      'has-hits',
      key === 'you'
        ? youCount > 0
        : key === 'assistant'
          ? assistantCount > 0
          : key === 'errors'
            ? errorsCount > 0
            : false
    );
  });

  if (trueEmpty || items.length === 0) {
    ensureChatFilterMissState(container, false);
    applyChatErrorsGlanceState();
    return;
  }

  let visible = 0;
  items.forEach((el) => {
    const isYou = el.classList.contains('user');
    const isAsst = el.classList.contains('assistant');
    const isErr = el.classList.contains('is-error');
    let show = true;
    if (chatFilterMode === 'you') show = isYou;
    else if (chatFilterMode === 'assistant') show = isAsst;
    else if (chatFilterMode === 'errors') show = isErr;
    el.style.display = show ? '' : 'none';
    if (show) visible++;
  });

  ensureChatFilterMissState(container, visible === 0);
  // Drop selection if the active bubble was filtered out.
  const selected = container.querySelector('.chat-message.is-selected');
  if (selected && selected.style.display === 'none') {
    clearChatMessageSelection(container);
  } else if (visible > 0) {
    syncChatMessagesTabOrder(
      container,
      selected && selected.style.display !== 'none' ? selected : null
    );
  } else {
    ensureChatMessagesKbHint(container, false);
  }
  applyChatErrorsGlanceState();
}

function getChatClearButton() {
  return document.getElementById('chat-clear-btn');
}

/** Truncate glance preview (strip markdown noise, cap length). */
function truncateChatGlancePreview(text, maxLen = 48) {
  const raw = String(text || '')
    .replace(/\[screenshot:[^\]]+\]/gi, '')
    .replace(/\s+/g, ' ')
    .trim();
  if (!raw) return '';
  if (raw.length <= maxLen) return raw;
  return `${raw.slice(0, maxLen - 1).trim()}…`;
}

/** Connection snapshot for the model glance (updated by checkOllamaConnection). */
let chatModelGlanceState = { status: 'unknown', model: '', circuitOpen: false };

async function readOllamaCircuitOpen() {
  try {
    return !!(await invoke('ollama_circuit_is_open'));
  } catch (_) {
    return false;
  }
}

function shortChatModelName(modelName, maxLen = 28) {
  const name = String(modelName || '').trim();
  if (!name) return '';
  if (name.length <= maxLen) return name;
  return `${name.slice(0, maxLen - 1).trim()}…`;
}

function getChatModelGlanceLabel() {
  const modelText = document.getElementById('ollama-model-text');
  const fromUi = modelText && modelText.style.display !== 'none'
    ? String(modelText.textContent || '').trim()
    : '';
  const fromStore = String(localStorage.getItem('ollama_model') || '').trim();
  return shortChatModelName(fromUi || fromStore || chatModelGlanceState.model || '');
}

/** True when AI Chat content is collapsed (header may still show). */
function isOllamaSectionCollapsed() {
  const content = document.getElementById('ollama-content');
  return !!(content && content.classList.contains('collapsed'));
}

/** Expand AI Chat if collapsed (model glance / Offline CTA / collapsed glance). */
function ensureOllamaSectionExpanded() {
  const content = document.getElementById('ollama-content');
  const section = document.querySelector('.ollama-section');
  const header = document.getElementById('ollama-header');
  if (!content) return;
  if (!content.classList.contains('collapsed') && content.style.display !== 'none') return;
  if (section) {
    section.style.display = '';
    section.classList.remove('collapsed');
    section.removeAttribute('aria-hidden');
  }
  content.classList.remove('collapsed');
  content.style.display = '';
  const divider = document.getElementById('monitors-ollama-divider');
  if (divider) divider.style.display = '';
  if (typeof window.setSectionCollapsed === 'function') {
    window.setSectionCollapsed('ollama_collapsed', false);
  }
  if (typeof window.__setOllamaCollapsed === 'function') {
    window.__setOllamaCollapsed(false);
  }
  header?.setAttribute('aria-expanded', 'true');
  if (typeof header?._syncCollapseA11y === 'function') header._syncCollapseA11y();
  if (typeof window.syncSectionIcon === 'function') {
    window.syncSectionIcon('icon-ollama', true);
  }
  const iconEl = document.getElementById('icon-ollama');
  if (iconEl) {
    iconEl.title = iconEl.getAttribute('data-title-base') || 'Hide AI Chat';
  }
  const chat = document.getElementById('ollama-chat');
  if (chat) chat.style.display = 'block';
  const menuCollapse = document.getElementById('ollama-menu-collapse');
  if (menuCollapse) menuCollapse.textContent = 'Collapse';
}

/** Collapsed-section glance under AI Chat header (Monitors / Disk Cleanup parity). */
function ensureOllamaCollapsedGlance() {
  const header = document.getElementById('ollama-header');
  if (!header) return null;
  let glance = document.getElementById('ollama-collapsed-glance');
  if (!glance) {
    glance = document.createElement('div');
    glance.id = 'ollama-collapsed-glance';
    glance.className = 'ollama-collapsed-glance';
    glance.hidden = true;
    glance.innerHTML = '<span id="ollama-collapsed-glance-text"></span>';
    header.insertAdjacentElement('afterend', glance);
    wireOllamaCollapsedGlanceClick(glance);
  }
  return glance;
}

function syncOllamaCollapsedGlance() {
  const glance = ensureOllamaCollapsedGlance();
  if (!glance) return;
  const glanceText = document.getElementById('ollama-collapsed-glance-text');
  if (!isOllamaSectionCollapsed()) {
    glance.hidden = true;
    // Re-apply expanded glances (header toggle / icon open).
    applyChatModelGlanceState();
    applyChatTurnGlanceState();
    applyChatAnswerGlanceState();
    applyChatErrorsGlanceState();
    return;
  }
  // One strip while collapsed — hide the expanded model/turn/answer/errors glances.
  const model = document.getElementById('chat-model-glance');
  const turn = document.getElementById('chat-turn-glance');
  const answer = document.getElementById('chat-answer-glance');
  const errors = document.getElementById('chat-errors-glance');
  if (model) model.hidden = true;
  if (turn) turn.hidden = true;
  if (answer) answer.hidden = true;
  if (errors) errors.hidden = true;
  const offlineAttention = document.getElementById('chat-offline-attention-glance');
  if (offlineAttention) offlineAttention.hidden = true;

  glance.hidden = false;
  const status = chatModelGlanceState.status || 'unknown';
  const modelName = getChatModelGlanceLabel();
  const turns = countChatTurns();
  const errCount = countChatErrors();
  const preview = getChatTurnGlancePreview();
  let line = 'AI Chat';
  let wash = 'is-empty';
  if (status === 'error') {
    line = chatModelGlanceState.circuitOpen
      ? 'Circuit open · retry soon'
      : 'Offline · check Ollama';
    wash = 'is-offline';
  } else if (status === 'unknown') {
    line = 'Not set · configure URL';
    wash = 'is-offline';
  } else if (chatSendInFlight) {
    line = preview ? `Sending · ${preview}` : 'Sending · wait';
    wash = 'is-active';
  } else if (errCount > 0) {
    const errLabel = errCount === 1 ? '1 failed' : `${errCount} failed`;
    if (turns && preview) {
      line = `${errLabel} · ${preview}`;
    } else {
      line = `Errors · ${errLabel}`;
    }
    wash = 'has-errors';
  } else if (turns && preview) {
    const turnLabel = turns === 1 ? '1 turn' : `${turns} turns`;
    line = `${turnLabel} · ${preview}`;
    wash = 'is-online';
  } else if (modelName) {
    line = `Ready · try a starter · ${modelName}`;
    wash = 'is-online';
  } else {
    line = 'Ready · pick a model';
    wash = 'is-online';
  }
  if (glanceText) glanceText.textContent = line;
  glance.classList.toggle('is-online', wash === 'is-online');
  glance.classList.toggle('is-offline', wash === 'is-offline');
  glance.classList.toggle('is-active', wash === 'is-active');
  glance.classList.toggle('has-errors', wash === 'has-errors');
  glance.classList.toggle('is-empty', wash === 'is-empty');
  glance.setAttribute('role', 'button');
  glance.tabIndex = 0;
  const chainHint = '↑ → AI Chat icon · ↓ → footer';
  if (wash === 'is-offline') {
    const circuit = !!chatModelGlanceState.circuitOpen;
    glance.title = circuit
      ? `Open AI Chat — circuit open, retry soon · ${chainHint}`
      : `Open AI Chat — configure Ollama · ${chainHint}`;
    glance.setAttribute(
      'aria-label',
      circuit
        ? `${line} — click to expand · ↑ AI Chat icon · ↓ footer`
        : `${line} — click to configure · ↑ AI Chat icon · ↓ footer`
    );
  } else if (wash === 'is-active' && chatSendInFlight) {
    glance.title = `Open AI Chat — reply in flight · ${chainHint}`;
    glance.setAttribute(
      'aria-label',
      `${line} — click to expand and watch the reply · ↑ AI Chat icon · ↓ footer`
    );
  } else if (wash === 'has-errors') {
    glance.title = `Show AI Chat Errors filter · ${chainHint}`;
    glance.setAttribute(
      'aria-label',
      `${line} — click to expand and show Errors · ↑ AI Chat icon · ↓ footer`
    );
  } else if (turns && preview) {
    glance.title = `Show AI Chat and focus composer · ${chainHint}`;
    glance.setAttribute(
      'aria-label',
      `${line} — click to expand and focus composer · ↑ AI Chat icon · ↓ footer`
    );
  } else if (modelName && !turns) {
    glance.title = `Show AI Chat and focus a starter · ${chainHint}`;
    glance.setAttribute(
      'aria-label',
      `${line} — click to expand and try a starter · ↑ AI Chat icon · ↓ footer`
    );
  } else {
    glance.title = `Show AI Chat · ${chainHint}`;
    glance.setAttribute(
      'aria-label',
      `${line} — click to expand · ↑ AI Chat icon · ↓ footer`
    );
  }
}

function activateOllamaCollapsedGlance() {
  const status = chatModelGlanceState.status || 'unknown';
  const errCount = countChatErrors();
  ensureOllamaSectionExpanded();
  syncOllamaCollapsedGlance();
  applyChatModelGlanceState();
  applyChatTurnGlanceState();
  applyChatAnswerGlanceState();
  applyChatErrorsGlanceState();
  if (status !== 'connected') {
    if (!chatModelGlanceState.circuitOpen) {
      showOllamaUrlDialog();
    } else {
      document.getElementById('chat-input')?.focus();
    }
    return;
  }
  if (chatSendInFlight) {
    const container = document.getElementById('chat-messages');
    if (container) {
      container.scrollTop = container.scrollHeight;
      const last = container.querySelector('.chat-message:last-child');
      if (last && typeof last.scrollIntoView === 'function') {
        last.scrollIntoView({ block: 'nearest' });
      }
    }
    document.getElementById('chat-input')?.focus();
    return;
  }
  if (errCount > 0) {
    setChatFilterMode('errors');
    const firstErr = document.querySelector(
      '#chat-messages .chat-message.assistant.is-error'
    );
    if (firstErr && typeof firstErr.scrollIntoView === 'function') {
      firstErr.scrollIntoView({ block: 'nearest' });
      if (typeof firstErr.focus === 'function') firstErr.focus();
    }
    return;
  }
  if (isChatTrulyEmpty()) {
    ensureChatEmptyHint();
    applyChatOfflineAttentionGlanceState();
    if (focusChatEmptySuggestionFirst()) return;
    document.getElementById('chat-input')?.focus();
    return;
  }
  const container = document.getElementById('chat-messages');
  if (container) {
    container.scrollTop = container.scrollHeight;
    const last = container.querySelector('.chat-message:last-child');
    if (last && typeof last.scrollIntoView === 'function') {
      last.scrollIntoView({ block: 'nearest' });
    }
  }
  document.getElementById('chat-input')?.focus();
}

function wireOllamaCollapsedGlanceClick(glance) {
  if (!glance || glance.dataset.ollamaCollapsedGlanceWired === '1') return;
  glance.dataset.ollamaCollapsedGlanceWired = '1';
  const activate = () => {
    activateOllamaCollapsedGlance();
  };
  glance.addEventListener('click', (e) => {
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
  glance.addEventListener('keydown', (e) => {
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      e.stopPropagation();
      activate();
      return;
    }
    if (e.key === 'ArrowUp' || e.key === 'k') {
      if (
        typeof window.tryChainOllamaGlanceToIconLine === 'function' &&
        window.tryChainOllamaGlanceToIconLine()
      ) {
        e.preventDefault();
        e.stopPropagation();
      }
      return;
    }
    if (e.key === 'ArrowDown' || e.key === 'j') {
      if (
        typeof window.tryChainOllamaGlanceToFooter === 'function' &&
        window.tryChainOllamaGlanceToFooter()
      ) {
        e.preventDefault();
        e.stopPropagation();
      }
    }
  });
}

/** Model / connection glance under AI Chat header (Debug Log / turn-glance parity). */
function ensureChatModelGlance() {
  const header = document.getElementById('ollama-header');
  if (!header) return null;
  let glance = document.getElementById('chat-model-glance');
  if (!glance) {
    glance = document.createElement('div');
    glance.id = 'chat-model-glance';
    glance.className = 'chat-model-glance';
    glance.hidden = true;
    glance.innerHTML = '<span id="chat-model-glance-text"></span>';
    const collapsed = ensureOllamaCollapsedGlance();
    const anchor = collapsed || header;
    anchor.insertAdjacentElement('afterend', glance);
    wireChatModelGlanceClick(glance);
  }
  return glance;
}

function applyChatModelGlanceState() {
  if (isOllamaSectionCollapsed()) {
    syncOllamaCollapsedGlance();
    return;
  }
  const collapsedGlance = document.getElementById('ollama-collapsed-glance');
  if (collapsedGlance) collapsedGlance.hidden = true;
  const glance = ensureChatModelGlance();
  if (!glance) return;
  const text = document.getElementById('chat-model-glance-text');
  const status = chatModelGlanceState.status || 'unknown';
  const model = getChatModelGlanceLabel();
  glance.hidden = false;
  glance.classList.toggle('is-online', status === 'connected');
  glance.classList.toggle('is-offline', status === 'error' || status === 'unknown');
  glance.setAttribute('role', 'button');
  glance.tabIndex = 0;
  if (status === 'connected') {
    glance.classList.remove('is-circuit');
    const label = model ? `Model · ${model}` : 'Model · pick one';
    if (text) text.textContent = label;
    glance.classList.toggle('is-no-model', !model);
    glance.title = model
      ? `Change model (${model})`
      : 'Choose an Ollama model';
    glance.setAttribute(
      'aria-label',
      model ? `Connected — model ${model}. Click to change.` : 'Connected — choose a model'
    );
  } else if (status === 'error') {
    glance.classList.remove('is-no-model');
    const circuit = !!chatModelGlanceState.circuitOpen;
    glance.classList.toggle('is-circuit', circuit);
    if (text) {
      text.textContent = circuit
        ? 'Offline · circuit open'
        : 'Offline · check Ollama';
    }
    glance.title = circuit
      ? 'Ollama circuit open — chat paused; retry soon'
      : 'Ollama is not available — click to set the URL';
    glance.setAttribute(
      'aria-label',
      circuit
        ? 'Ollama circuit open — retry soon'
        : 'Ollama offline — click to configure URL'
    );
  } else {
    glance.classList.remove('is-no-model', 'is-circuit');
    if (text) text.textContent = 'Not set · configure URL';
    glance.title = 'Click to configure the Ollama URL';
    glance.setAttribute('aria-label', 'Ollama not configured — click to set URL');
  }
  applyChatOfflineAttentionGlanceState();
}

function wireChatModelGlanceClick(glance) {
  if (!glance || glance.dataset.modelGlanceWired === '1') return;
  glance.dataset.modelGlanceWired = '1';
  const activate = () => {
    ensureOllamaSectionExpanded();
    const status = chatModelGlanceState.status || 'unknown';
    if (status === 'connected') {
      const modelText = document.getElementById('ollama-model-text');
      if (modelText && typeof modelText.click === 'function') {
        modelText.click();
        return;
      }
      document.getElementById('chat-input')?.focus();
      return;
    }
    if (chatModelGlanceState.circuitOpen) {
      document.getElementById('chat-input')?.focus();
      return;
    }
    showOllamaUrlDialog();
  };
  glance.addEventListener('click', (e) => {
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
  glance.addEventListener('keydown', (e) => {
    if (e.key !== 'Enter' && e.key !== ' ') return;
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
}

/** Last user turn preview for the turn glance strip (Top CPU / Monitors parity). */
function getChatTurnGlancePreview() {
  for (let i = conversationHistory.length - 1; i >= 0; i--) {
    if (conversationHistory[i]?.role === 'user') {
      return truncateChatGlancePreview(conversationHistory[i].content);
    }
  }
  const nodes = document.querySelectorAll('#chat-messages .chat-message.user');
  const last = nodes.length ? nodes[nodes.length - 1] : null;
  if (last) return truncateChatGlancePreview(last.textContent || '');
  return '';
}

function countChatTurns() {
  let turns = conversationHistory.filter((m) => m.role === 'user').length;
  if (!turns) {
    turns = document.querySelectorAll('#chat-messages .chat-message.user').length;
  }
  return turns;
}

/** Count failed assistant turns (Error: …) for the Errors glance / filter chip. */
function countChatErrors() {
  const nodes = document.querySelectorAll(
    '#chat-messages .chat-message.assistant:not(.thinking)'
  );
  let n = 0;
  nodes.forEach((el) => {
    if (syncChatMessageErrorClass(el)) n++;
  });
  return n;
}

/** Turn glance under AI Chat header — scroll to latest + focus composer. */
function ensureChatTurnGlance() {
  const header = document.getElementById('ollama-header');
  if (!header) return null;
  let glance = document.getElementById('chat-turn-glance');
  if (!glance) {
    glance = document.createElement('div');
    glance.id = 'chat-turn-glance';
    glance.className = 'chat-turn-glance';
    glance.hidden = true;
    glance.innerHTML = '<span id="chat-turn-glance-text"></span>';
    const model = ensureChatModelGlance();
    const anchor = model || header;
    anchor.insertAdjacentElement('afterend', glance);
    wireChatTurnGlanceClick(glance);
  }
  return glance;
}

function applyChatTurnGlanceState() {
  if (isOllamaSectionCollapsed()) {
    syncOllamaCollapsedGlance();
    return;
  }
  const glance = ensureChatTurnGlance();
  if (!glance) return;
  const text = document.getElementById('chat-turn-glance-text');
  const turns = countChatTurns();
  const preview = getChatTurnGlancePreview();
  if (!turns || !preview) {
    glance.hidden = true;
    glance.classList.remove('is-active');
    return;
  }
  glance.hidden = false;
  const turnLabel = turns === 1 ? '1 turn' : `${turns} turns`;
  if (text) {
    text.textContent = chatSendInFlight
      ? `Sending · ${preview}`
      : `${turnLabel} · ${preview}`;
  }
  glance.classList.toggle('is-active', chatSendInFlight);
  glance.setAttribute('role', 'button');
  glance.tabIndex = 0;
  glance.title = chatSendInFlight
    ? 'Reply in flight — scroll to latest'
    : 'Scroll to latest message and focus composer';
  glance.setAttribute(
    'aria-label',
    chatSendInFlight
      ? `Sending reply for "${preview}" — scroll to latest`
      : `${turnLabel} — last question "${preview}" — scroll to latest`
  );
}

function wireChatTurnGlanceClick(glance) {
  if (!glance || glance.dataset.turnGlanceWired === '1') return;
  glance.dataset.turnGlanceWired = '1';
  const activate = () => {
    const container = document.getElementById('chat-messages');
    if (container) {
      container.scrollTop = container.scrollHeight;
      const last = container.querySelector('.chat-message:last-child');
      if (last && typeof last.scrollIntoView === 'function') {
        last.scrollIntoView({ block: 'nearest' });
      }
    }
    document.getElementById('chat-input')?.focus();
  };
  glance.addEventListener('click', (e) => {
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
  glance.addEventListener('keydown', (e) => {
    if (e.key !== 'Enter' && e.key !== ' ') return;
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
}

/** Last assistant reply text for the answer glance (history first, then DOM). */
function getLastAssistantAnswerText() {
  for (let i = conversationHistory.length - 1; i >= 0; i--) {
    const m = conversationHistory[i];
    if (m?.role === 'assistant' && typeof m.content === 'string') {
      const t = m.content.trim();
      if (t) return t;
    }
  }
  const nodes = document.querySelectorAll(
    '#chat-messages .chat-message.assistant:not(.thinking)'
  );
  const last = nodes.length ? nodes[nodes.length - 1] : null;
  if (!last) return '';
  return String(last.innerText || last.textContent || '').trim();
}

/** Truncate last-answer glance preview. */
function getChatAnswerGlancePreview() {
  return truncateChatGlancePreview(getLastAssistantAnswerText(), 52);
}

async function copyChatTextToClipboard(text) {
  const value = String(text || '').trim();
  if (!value) return false;
  if (typeof window.copyTextToClipboard === 'function') {
    return window.copyTextToClipboard(value);
  }
  try {
    if (navigator.clipboard && typeof navigator.clipboard.writeText === 'function') {
      await navigator.clipboard.writeText(value);
      return true;
    }
  } catch (_) {
    /* fall through */
  }
  try {
    const ta = document.createElement('textarea');
    ta.value = value;
    ta.setAttribute('readonly', '');
    ta.style.position = 'fixed';
    ta.style.left = '-9999px';
    document.body.appendChild(ta);
    ta.select();
    const ok = document.execCommand('copy');
    document.body.removeChild(ta);
    return !!ok;
  } catch (_) {
    return false;
  }
}

function flashChatAnswerGlanceCopied(glance) {
  if (!glance) return;
  const text = document.getElementById('chat-answer-glance-text');
  if (glance._answerCopiedTimer) {
    clearTimeout(glance._answerCopiedTimer);
    glance._answerCopiedTimer = null;
  }
  const prev = text ? text.textContent : '';
  glance.classList.add('is-just-copied');
  if (text) text.textContent = 'Copied';
  glance.title = 'Copied';
  glance.setAttribute('aria-label', 'Copied last answer');
  glance._answerCopiedTimer = setTimeout(() => {
    glance.classList.remove('is-just-copied');
    glance._answerCopiedTimer = null;
    applyChatAnswerGlanceState();
    if (text && !text.textContent) text.textContent = prev;
  }, 1600);
}

/** Plain text for a chat bubble (stored at create time; falls back to visible text). */
function getChatMessageCopyText(el) {
  if (!el) return '';
  const stored = (el.dataset && el.dataset.copyText) || '';
  if (stored.trim()) return stored.trim();
  return String(el.innerText || el.textContent || '')
    .replace(/\s+/g, ' ')
    .trim();
}

function flashChatMessageCopied(el) {
  if (!el) return;
  if (el._msgCopiedTimer) {
    clearTimeout(el._msgCopiedTimer);
    el._msgCopiedTimer = null;
  }
  el.classList.add('is-just-copied');
  const prevTitle = el.title || '';
  el.title = 'Copied';
  el.setAttribute('aria-label', 'Copied');
  el._msgCopiedTimer = setTimeout(() => {
    el.classList.remove('is-just-copied');
    el._msgCopiedTimer = null;
    el.title = prevTitle || 'Click to copy · ↑↓ / j k to move · Esc clears';
    const role = el.classList.contains('user') ? 'your' : 'assistant';
    el.setAttribute('aria-label', `${role} message — copy with Enter or c`);
  }, 1600);
}

async function copyChatMessageFromUi(el) {
  if (!el || el.classList.contains('is-just-copied') || chatSendInFlight) return false;
  // Let the user finish a drag-select without stealing the clipboard.
  try {
    const sel = window.getSelection && window.getSelection();
    if (sel && sel.rangeCount > 0 && String(sel.toString() || '').trim()) {
      const anchor = sel.anchorNode;
      if (anchor && el.contains(anchor)) return false;
    }
  } catch (_) {
    /* ignore */
  }
  const value = getChatMessageCopyText(el);
  if (!value) return false;
  const ok = await copyChatTextToClipboard(value);
  if (ok) flashChatMessageCopied(el);
  return ok;
}

/** Visible (non-thinking) chat bubbles for keyboard nav. */
function visibleChatMessages(container) {
  if (!container) return [];
  return Array.from(container.querySelectorAll('.chat-message')).filter((el) => {
    if (el.classList.contains('thinking')) return false;
    if (el.style.display === 'none') return false;
    return true;
  });
}

function focusChatMessagesLast(container) {
  const items = visibleChatMessages(container);
  if (!items.length) return false;
  syncChatMessagesTabOrder(container, items[items.length - 1]);
  items[items.length - 1].focus();
  try {
    items[items.length - 1].scrollIntoView({ block: 'nearest', behavior: 'smooth' });
  } catch (_) {
    /* ignore */
  }
  return true;
}
window.focusChatMessagesLast = focusChatMessagesLast;

/** Hint above the message list (Monitors / Top Processes kb-hint parity). */
function ensureChatMessagesKbHint(container, show) {
  if (!container || !container.parentNode) return;
  let hint = document.getElementById('chat-kb-hint');
  if (!show) {
    hint?.remove();
    return;
  }
  if (!hint) {
    hint = document.createElement('div');
    hint.className = 'chat-kb-hint';
    hint.id = 'chat-kb-hint';
    container.parentNode.insertBefore(hint, container);
  }
  hint.textContent =
    'All · You · Assistant filters · focus list then ↑↓ / j k / Home / End · last ↓ → composer · click / Enter / Space / c copies · PgUp/PgDn · Esc clears';
}

/** Keep one selected + tabbable bubble (Monitors listbox parity). */
function syncChatMessagesTabOrder(container, preferEl) {
  const items = visibleChatMessages(container);
  ensureChatMessagesKbHint(container, items.length > 0);
  if (!items.length) {
    container?.querySelectorAll('.chat-message.is-selected').forEach((el) => {
      el.classList.remove('is-selected');
      el.setAttribute('aria-selected', 'false');
    });
    return;
  }
  let activeIdx = preferEl ? items.indexOf(preferEl) : -1;
  if (activeIdx < 0) {
    activeIdx = items.findIndex((el) => el.classList.contains('is-selected'));
  }
  if (activeIdx < 0) {
    const focused = document.activeElement;
    activeIdx = focused && items.includes(focused) ? items.indexOf(focused) : 0;
  }
  items.forEach((el, i) => {
    const on = i === activeIdx;
    el.classList.toggle('is-selected', on);
    el.setAttribute('aria-selected', on ? 'true' : 'false');
    el.tabIndex = on ? 0 : -1;
  });
}

function clearChatMessageSelection(container) {
  if (!container) return;
  container.querySelectorAll('.chat-message.is-selected').forEach((el) => {
    el.classList.remove('is-selected');
    el.setAttribute('aria-selected', 'false');
  });
  const items = visibleChatMessages(container);
  items.forEach((el, i) => {
    el.tabIndex = i === 0 ? 0 : -1;
  });
  if (document.activeElement && container.contains(document.activeElement)) {
    document.activeElement.blur();
  }
}

/** Wire click / Enter / Space / c copy + j/k list nav on chat bubbles. */
function wireChatMessagesCopy(container) {
  if (!container || container.dataset.copyWired === '1') return;
  container.dataset.copyWired = '1';
  container.setAttribute('role', 'listbox');
  container.setAttribute('aria-label', 'Chat messages');
  if (!container.hasAttribute('tabindex')) {
    container.setAttribute('tabindex', '0');
  }
  const activate = (msg) => {
    if (!msg || msg.classList.contains('thinking')) return;
    void copyChatMessageFromUi(msg);
  };
  const focusMessage = (next) => {
    if (!next) return;
    syncChatMessagesTabOrder(container, next);
    next.focus();
    try {
      next.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    } catch (_) {
      /* ignore */
    }
  };
  container.addEventListener('click', (e) => {
    const msg = e.target && e.target.closest && e.target.closest('.chat-message');
    if (!msg || !container.contains(msg)) return;
    if (e.target.closest('a, button, input, textarea, select')) return;
    syncChatMessagesTabOrder(container, msg);
    msg.focus();
    activate(msg);
  });
  container.addEventListener('keydown', (e) => {
    const msg = e.target && e.target.closest && e.target.closest('.chat-message');
    if (!msg || !container.contains(msg)) {
      // First arrow/j from listbox chrome focuses first/last message (Perplexity / Monitors parity).
      if (e.target !== container) return;
      if (
        (e.key === 'ArrowUp' || e.key === 'k' || e.key === 'K') &&
        typeof window.tryChainListboxToFilterChips === 'function' &&
        window.tryChainListboxToFilterChips(container)
      ) {
        return;
      }
      const items = visibleChatMessages(container);
      if (!items.length) return;
      let next = -1;
      if (e.key === 'ArrowDown' || e.key === 'j' || e.key === 'Home') next = 0;
      else if (e.key === 'ArrowUp' || e.key === 'k' || e.key === 'End') next = items.length - 1;
      else return;
      e.preventDefault();
      focusMessage(items[next]);
      return;
    }
    if (msg.style.display === 'none' || msg.classList.contains('thinking')) return;
    const items = visibleChatMessages(container);
    const idx = items.indexOf(msg);
    if (idx < 0) return;

    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      activate(msg);
      return;
    }

    if (
      (e.key === 'c' || e.key === 'C') &&
      !e.metaKey &&
      !e.ctrlKey &&
      !e.altKey
    ) {
      e.preventDefault();
      activate(msg);
      return;
    }

    if (e.key === 'Escape') {
      e.preventDefault();
      clearChatMessageSelection(container);
      return;
    }

    const move = (nextIdx) => {
      if (nextIdx < 0 || nextIdx >= items.length) return;
      e.preventDefault();
      focusMessage(items[nextIdx]);
    };

    if (e.key === 'ArrowDown' || e.key === 'j' || e.key === 'J') {
      if (idx === items.length - 1 && focusChatComposerFirst()) {
        e.preventDefault();
        e.stopPropagation();
        return;
      }
      if (
        idx === items.length - 1 &&
        typeof window.tryChainSectionContentToFooter === 'function' &&
        window.tryChainSectionContentToFooter(container)
      ) {
        return;
      }
      move(idx + 1);
      return;
    }
    if (e.key === 'ArrowUp' || e.key === 'k' || e.key === 'K') {
      if (
        idx === 0 &&
        typeof window.tryChainListboxToFilterChips === 'function' &&
        window.tryChainListboxToFilterChips(container)
      ) {
        return;
      }
      move(idx - 1);
      return;
    }
    if (e.key === 'PageDown') {
      move(Math.min(items.length - 1, idx + 5));
      return;
    }
    if (e.key === 'PageUp') {
      move(Math.max(0, idx - 5));
      return;
    }
    if (e.key === 'Home') {
      move(0);
      return;
    }
    if (e.key === 'End') {
      move(items.length - 1);
    }
  });
}

function decorateChatMessageForCopy(messageDiv, role, plainText) {
  if (!messageDiv) return;
  const text = String(plainText ?? '').trim();
  if (text) messageDiv.dataset.copyText = text;
  messageDiv.setAttribute('role', 'option');
  if (!messageDiv.hasAttribute('tabindex')) messageDiv.tabIndex = -1;
  messageDiv.setAttribute('aria-selected', 'false');
  messageDiv.title =
    'Click to copy · focus list then ↑↓ / j k / Home / End · Esc clears';
  const who = role === 'user' ? 'your' : 'assistant';
  messageDiv.setAttribute('aria-label', `${who} message — copy with Enter or c`);
}

/** Last-answer glance under AI Chat — click copies reply (Top Processes / Monitors parity). */
function ensureChatAnswerGlance() {
  const turn = ensureChatTurnGlance();
  const model = ensureChatModelGlance();
  const header = document.getElementById('ollama-header');
  const anchor = turn || model || header;
  if (!anchor) return null;
  let glance = document.getElementById('chat-answer-glance');
  if (!glance) {
    glance = document.createElement('div');
    glance.id = 'chat-answer-glance';
    glance.className = 'chat-answer-glance';
    glance.hidden = true;
    glance.innerHTML = '<span id="chat-answer-glance-text"></span>';
    anchor.insertAdjacentElement('afterend', glance);
    wireChatAnswerGlanceClick(glance);
  }
  return glance;
}

function applyChatAnswerGlanceState() {
  if (isOllamaSectionCollapsed()) {
    syncOllamaCollapsedGlance();
    return;
  }
  const glance = ensureChatAnswerGlance();
  if (!glance) return;
  if (glance.classList.contains('is-just-copied') && glance._answerCopiedTimer) return;
  const text = document.getElementById('chat-answer-glance-text');
  const answer = getLastAssistantAnswerText();
  const preview = getChatAnswerGlancePreview();
  if (!answer || !preview || chatSendInFlight) {
    glance.hidden = true;
    glance.classList.remove('has-answer', 'has-errors');
    return;
  }
  const isErr = isChatErrorText(answer);
  glance.hidden = false;
  glance.classList.toggle('has-answer', !isErr);
  glance.classList.toggle('has-errors', isErr);
  if (text) {
    text.textContent = isErr ? `Last error · ${preview}` : `Last answer · ${preview}`;
  }
  glance.setAttribute('role', 'button');
  glance.tabIndex = 0;
  if (isErr) {
    glance.title = 'Show failed turns (Errors filter)';
    glance.setAttribute('aria-label', `Last error: ${preview} — click to show Errors`);
  } else {
    glance.title = 'Copy last answer';
    glance.setAttribute('aria-label', `Copy last answer: ${preview}`);
  }
}

function wireChatAnswerGlanceClick(glance) {
  if (!glance || glance.dataset.answerGlanceWired === '1') return;
  glance.dataset.answerGlanceWired = '1';
  const activate = async () => {
    if (chatSendInFlight) return;
    const answer = getLastAssistantAnswerText();
    if (!answer) return;
    if (isChatErrorText(answer)) {
      ensureOllamaSectionExpanded();
      setChatFilterMode('errors');
      const firstErr = document.querySelector(
        '#chat-messages .chat-message.assistant.is-error'
      );
      if (firstErr && typeof firstErr.scrollIntoView === 'function') {
        firstErr.scrollIntoView({ block: 'nearest' });
        firstErr.focus?.();
      }
      return;
    }
    const ok = await copyChatTextToClipboard(answer);
    if (ok) flashChatAnswerGlanceCopied(glance);
  };
  glance.addEventListener('click', (e) => {
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
  glance.addEventListener('keydown', (e) => {
    if (e.key !== 'Enter' && e.key !== ' ') return;
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
}

/** Errors glance under AI Chat — Debug Log error/warn glance parity (failed turns). */
function ensureChatErrorsGlance() {
  const answer = ensureChatAnswerGlance();
  const turn = ensureChatTurnGlance();
  const model = ensureChatModelGlance();
  const header = document.getElementById('ollama-header');
  const anchor = answer || turn || model || header;
  if (!anchor) return null;
  let glance = document.getElementById('chat-errors-glance');
  if (!glance) {
    glance = document.createElement('div');
    glance.id = 'chat-errors-glance';
    glance.className = 'chat-errors-glance';
    glance.hidden = true;
    glance.innerHTML = '<span id="chat-errors-glance-text"></span>';
    anchor.insertAdjacentElement('afterend', glance);
    wireChatErrorsGlanceClick(glance);
  }
  return glance;
}

function applyChatErrorsGlanceState() {
  if (isOllamaSectionCollapsed()) {
    const glance = document.getElementById('chat-errors-glance');
    if (glance) glance.hidden = true;
    syncOllamaCollapsedGlance();
    return;
  }
  const glance = ensureChatErrorsGlance();
  if (!glance) return;
  const text = document.getElementById('chat-errors-glance-text');
  const n = countChatErrors();
  if (n <= 0) {
    glance.hidden = true;
    glance.classList.remove('has-errors');
    applyChatOfflineAttentionGlanceState();
    return;
  }
  glance.hidden = false;
  glance.classList.add('has-errors');
  const label = n === 1 ? '1 failed turn' : `${n} failed turns`;
  if (text) text.textContent = `Errors · ${label}`;
  glance.setAttribute('role', 'button');
  glance.tabIndex = 0;
  glance.title = 'Show failed turns only (Errors filter)';
  glance.setAttribute(
    'aria-label',
    `AI Chat has ${label} — click to show Errors filter`
  );
  applyChatOfflineAttentionGlanceState();
}

/** True when the chat pane has no message bubbles (empty Ready / starter cue). */
function isChatTrulyEmpty() {
  const container = document.getElementById('chat-messages');
  if (!container) return true;
  return !container.querySelector('.chat-message');
}

function ensureChatOfflineAttentionGlance() {
  ensureChatFilterChips();
  const chips = document.getElementById('chat-filter-chips');
  const chat = document.getElementById('ollama-chat');
  let glance = document.getElementById('chat-offline-attention-glance');
  if (!glance) {
    glance = document.createElement('div');
    glance.id = 'chat-offline-attention-glance';
    glance.className = 'chat-offline-attention-glance';
    glance.hidden = true;
    glance.innerHTML = '<span id="chat-offline-attention-glance-text"></span>';
    if (chips) {
      chips.insertAdjacentElement('beforebegin', glance);
    } else if (chat) {
      chat.insertAdjacentElement('afterbegin', glance);
    } else {
      return null;
    }
    wireChatOfflineAttentionGlanceClick(glance);
  } else if (chips && glance.nextElementSibling !== chips) {
    chips.insertAdjacentElement('beforebegin', glance);
  }
  return glance;
}

/**
 * Connection / model attention glance above AI Chat filters (Errors / Offline parity).
 * Visible when the section is open and Ollama is offline, not configured,
 * connected with no model selected, Ready with an empty chat (try a starter),
 * Continue with history (ask another), or a reply is in flight (Sending · wait).
 */
function applyChatOfflineAttentionGlanceState() {
  if (isOllamaSectionCollapsed()) {
    const glance = document.getElementById('chat-offline-attention-glance');
    if (glance) glance.hidden = true;
    return;
  }
  const glance = ensureChatOfflineAttentionGlance();
  if (!glance) return;
  const text = document.getElementById('chat-offline-attention-glance-text');
  const status = chatModelGlanceState.status || 'unknown';
  const model = getChatModelGlanceLabel();
  glance.setAttribute('role', 'button');
  glance.tabIndex = 0;

  const clearModeClasses = () => {
    glance.classList.remove(
      'is-offline',
      'is-not-set',
      'is-no-model',
      'is-circuit',
      'is-ready',
      'is-continue',
      'is-sending'
    );
  };

  if (chatSendInFlight) {
    glance.hidden = false;
    clearModeClasses();
    glance.classList.add('is-sending');
    if (text) text.textContent = 'Chat · Sending · wait';
    glance.title = 'Reply in flight — click to scroll to latest';
    glance.setAttribute(
      'aria-label',
      'AI Chat is sending — click to watch the reply'
    );
    return;
  }

  if (status === 'connected') {
    if (model) {
      if (isChatTrulyEmpty()) {
        glance.hidden = false;
        clearModeClasses();
        glance.classList.add('is-ready');
        if (text) text.textContent = 'Chat · Ready · try a starter';
        glance.title = 'Ready — click to focus a starter prompt';
        glance.setAttribute(
          'aria-label',
          'AI Chat ready — click to try a starter'
        );
        return;
      }
      const turns = countChatTurns();
      const turnLabel = turns === 1 ? '1 turn' : `${turns} turns`;
      glance.hidden = false;
      clearModeClasses();
      glance.classList.add('is-continue');
      if (text) text.textContent = 'Chat · Continue · ask another';
      glance.title = `${turnLabel} — click to focus the composer`;
      glance.setAttribute(
        'aria-label',
        `AI Chat has ${turnLabel} — click to ask another`
      );
      return;
    }
    glance.hidden = false;
    clearModeClasses();
    glance.classList.add('is-no-model');
    if (text) text.textContent = 'Chat · No model · pick one';
    glance.title = 'Connected — click to choose an Ollama model';
    glance.setAttribute(
      'aria-label',
      'AI Chat has no model — click to pick one'
    );
    return;
  }
  glance.hidden = false;
  clearModeClasses();
  const circuit = status === 'error' && !!chatModelGlanceState.circuitOpen;
  glance.classList.toggle('is-circuit', circuit);
  glance.classList.toggle('is-offline', status === 'error' && !circuit);
  glance.classList.toggle('is-not-set', status === 'unknown');
  if (status === 'error') {
    if (circuit) {
      if (text) text.textContent = 'Chat · Circuit open · retry soon';
      glance.title = 'Ollama circuit open — chat paused; retry soon';
      glance.setAttribute(
        'aria-label',
        'AI Chat paused — Ollama circuit open; retry soon'
      );
    } else {
      if (text) text.textContent = 'Chat · Offline · check Ollama';
      glance.title = 'Ollama is not available — click to set the URL';
      glance.setAttribute(
        'aria-label',
        'AI Chat offline — click to configure Ollama URL'
      );
    }
    return;
  }
  if (text) text.textContent = 'Chat · Not set · configure URL';
  glance.title = 'Click to configure the Ollama URL';
  glance.setAttribute(
    'aria-label',
    'Ollama not configured — click to set URL'
  );
}

function wireChatOfflineAttentionGlanceClick(glance) {
  if (!glance || glance.dataset.chatOfflineAttentionWired === '1') return;
  glance.dataset.chatOfflineAttentionWired = '1';
  const activate = () => {
    ensureOllamaSectionExpanded();
    if (chatSendInFlight) {
      const container = document.getElementById('chat-messages');
      if (container) {
        container.scrollTop = container.scrollHeight;
        const last = container.querySelector('.chat-message:last-child');
        if (last && typeof last.scrollIntoView === 'function') {
          last.scrollIntoView({ block: 'nearest' });
        }
      }
      document.getElementById('chat-input')?.focus();
      return;
    }
    const status = chatModelGlanceState.status || 'unknown';
    if (status === 'connected' && !getChatModelGlanceLabel()) {
      const modelText = document.getElementById('ollama-model-text');
      if (modelText && typeof modelText.click === 'function') {
        modelText.click();
        return;
      }
      document.getElementById('chat-input')?.focus();
      return;
    }
    if (status === 'connected' && isChatTrulyEmpty()) {
      ensureChatEmptyHint();
      if (focusChatEmptySuggestionFirst()) return;
      document.getElementById('chat-input')?.focus();
      return;
    }
    if (status === 'connected' && getChatModelGlanceLabel() && !isChatTrulyEmpty()) {
      const container = document.getElementById('chat-messages');
      if (container) {
        container.scrollTop = container.scrollHeight;
        const last = container.querySelector('.chat-message:last-child');
        if (last && typeof last.scrollIntoView === 'function') {
          last.scrollIntoView({ block: 'nearest' });
        }
      }
      document.getElementById('chat-input')?.focus();
      return;
    }
    if (chatModelGlanceState.circuitOpen) {
      document.getElementById('chat-input')?.focus();
      return;
    }
    showOllamaUrlDialog();
  };
  glance.addEventListener('click', (e) => {
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
  glance.addEventListener('keydown', (e) => {
    if (e.key !== 'Enter' && e.key !== ' ') return;
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
}

function wireChatErrorsGlanceClick(glance) {
  if (!glance || glance.dataset.chatErrorsGlanceWired === '1') return;
  glance.dataset.chatErrorsGlanceWired = '1';
  const activate = () => {
    ensureOllamaSectionExpanded();
    setChatFilterMode('errors');
    const firstErr = document.querySelector(
      '#chat-messages .chat-message.assistant.is-error'
    );
    if (firstErr && typeof firstErr.scrollIntoView === 'function') {
      firstErr.scrollIntoView({ block: 'nearest' });
      if (typeof firstErr.focus === 'function') firstErr.focus();
    }
  };
  glance.addEventListener('click', (e) => {
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
  glance.addEventListener('keydown', (e) => {
    if (e.key !== 'Enter' && e.key !== ' ') return;
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
}

/** Enable Clear only when history exists and Send is not in flight. */
function updateChatClearButton() {
  const btn = getChatClearButton();
  if (!btn) return;
  const hasUi = !!document.querySelector('#chat-messages .chat-message');
  const hasSomething = conversationHistory.length > 0 || hasUi;
  btn.disabled = !hasSomething || chatSendInFlight;
  btn.title = chatSendInFlight
    ? 'Wait for the reply to finish'
    : hasSomething
      ? 'Clear this chat'
      : 'No messages to clear';
  applyChatModelGlanceState();
  applyChatTurnGlanceState();
  applyChatAnswerGlanceState();
  applyChatErrorsGlanceState();
  applyChatListFilter();
  ensureChatComposerToolbarKeyboard();
  refreshChatComposerRovingTabindex();
  ensureChatComposerKbHint();
}

/**
 * Clear chat with visible confirmation on the Clear control (save-button feedback).
 */
function clearChatWithFeedback() {
  if (chatSendInFlight) return;
  const hasUi = !!document.querySelector('#chat-messages .chat-message');
  if (!conversationHistory.length && !hasUi) return;
  const btn = getChatClearButton();
  clearConversationHistory();
  const input = document.getElementById('chat-input');
  if (input) {
    input.value = '';
    input.focus();
  }
  if (!btn) return;
  if (btn._clearFlashTimer) {
    clearTimeout(btn._clearFlashTimer);
    btn._clearFlashTimer = null;
  }
  const idle = btn.dataset.idleLabel || 'Clear';
  btn.dataset.idleLabel = idle;
  btn.classList.add('is-just-saved');
  btn.textContent = 'Cleared';
  btn.disabled = true;
  btn._clearFlashTimer = setTimeout(() => {
    btn.classList.remove('is-just-saved');
    btn.textContent = idle;
    btn._clearFlashTimer = null;
    updateChatClearButton();
  }, 1600);
}

/** Starter prompts for an empty chat (fill composer; do not auto-send). */
const CHAT_EMPTY_SUGGESTIONS = [
  { label: "What's using CPU?", prompt: "What's using CPU?" },
  { label: "What's scheduled?", prompt: "What's scheduled next?" },
  { label: 'Any sites down?', prompt: 'Are any website monitors down?' },
];

/** Brief flash on a starter chip (Load into AI Chat / save-button-feedback parity). */
function flashChatEmptyChip(btn) {
  if (!btn) return;
  if (btn._chipFlashTimer) {
    clearTimeout(btn._chipFlashTimer);
    btn._chipFlashTimer = null;
  }
  const idle = btn.dataset.idleLabel || btn.textContent || '';
  btn.dataset.idleLabel = idle;
  btn.classList.add('is-just-saved');
  btn.textContent = 'In composer';
  btn._chipFlashTimer = setTimeout(() => {
    btn.classList.remove('is-just-saved');
    btn.textContent = idle;
    btn._chipFlashTimer = null;
  }, 1600);
}

/**
 * Put a starter prompt in the composer (Load into AI Chat parity — user hits Enter).
 * @param {string} prompt
 * @param {HTMLButtonElement | null | undefined} chipBtn
 */
function applyChatEmptySuggestion(prompt, chipBtn) {
  const input = document.getElementById('chat-input');
  const text = String(prompt || '').trim();
  if (!input || !text || chatSendInFlight) return;
  input.value = text;
  input.focus();
  try {
    const len = input.value.length;
    input.setSelectionRange(len, len);
  } catch (_) {
    /* ignore */
  }
  flashChatEmptyChip(chipBtn);
}

function getChatEmptySuggestionChips() {
  const row = document.querySelector(
    '#chat-messages .chat-empty:not(.chat-filter-miss) .chat-empty-suggestions'
  );
  if (!row) return [];
  return Array.from(row.querySelectorAll('.chat-empty-chip')).filter((el) => {
    if (!el || el.hidden || el.disabled) return false;
    return el.getClientRects().length > 0 || row.contains(el);
  });
}

function refreshChatEmptySuggestionRovingTabindex(preferred) {
  const chips = getChatEmptySuggestionChips();
  if (!chips.length) return;
  const focused = chips.find((el) => el === document.activeElement);
  const current =
    (preferred && chips.includes(preferred) && preferred) ||
    focused ||
    chips.find((el) => el.tabIndex === 0) ||
    chips[0];
  for (const el of chips) {
    el.tabIndex = el === current ? 0 : -1;
  }
}

function focusChatEmptySuggestionFirst() {
  const chips = getChatEmptySuggestionChips();
  if (!chips.length) return false;
  refreshChatEmptySuggestionRovingTabindex(chips[0]);
  chips[0].focus();
  return true;
}

function focusChatEmptySuggestionLast() {
  const chips = getChatEmptySuggestionChips();
  if (!chips.length) return false;
  const last = chips[chips.length - 1];
  refreshChatEmptySuggestionRovingTabindex(last);
  last.focus();
  return true;
}

function ensureChatEmptySuggestionKbHint(row) {
  if (!row) return;
  let hint = row.querySelector('.chat-empty-kb-hint');
  if (!hint) {
    hint = document.createElement('div');
    hint.className = 'chat-empty-kb-hint';
    hint.setAttribute('aria-hidden', 'true');
    row.appendChild(hint);
  }
  const chips = getChatEmptySuggestionChips();
  hint.hidden = chips.length < 2;
  hint.textContent =
    '← → / h l · Home/End move · Enter / Space puts prompt in composer · at end crosses to composer';
}

/**
 * Starter-chip toolbar keyboard — ←→ / h l / Home/End; Enter/Space fills composer;
 * last chip → composer input (filter-chip / Help-sheet parity).
 */
function ensureChatEmptySuggestionsToolbarKeyboard(row) {
  if (!row) return;
  ensureChatEmptySuggestionKbHint(row);
  refreshChatEmptySuggestionRovingTabindex();
  if (row.dataset.chatEmptyKbWired === '1') return;
  row.dataset.chatEmptyKbWired = '1';
  if (!row.getAttribute('role')) row.setAttribute('role', 'toolbar');
  if (!row.getAttribute('aria-label')) {
    row.setAttribute('aria-label', 'AI Chat starter prompts');
  }
  row.addEventListener('focusin', (e) => {
    const chips = getChatEmptySuggestionChips();
    if (chips.includes(e.target)) {
      refreshChatEmptySuggestionRovingTabindex(e.target);
      ensureChatEmptySuggestionKbHint(row);
    }
  });
  row.addEventListener('keydown', (e) => {
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    const chips = getChatEmptySuggestionChips();
    if (!chips.length) return;
    const idx = chips.indexOf(document.activeElement);
    if (idx < 0) return;
    const active = chips[idx];
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      e.stopPropagation();
      const prompt = active?.dataset?.prompt || active?.dataset?.idleLabel || '';
      applyChatEmptySuggestion(prompt, active);
      return;
    }
    let next = -1;
    const forward =
      e.key === 'ArrowRight' ||
      e.key === 'l' ||
      e.key === 'ArrowDown' ||
      e.key === 'j';
    const back =
      e.key === 'ArrowLeft' ||
      e.key === 'h' ||
      e.key === 'ArrowUp' ||
      e.key === 'k';
    if (forward && idx === chips.length - 1) {
      const input = document.getElementById('chat-input');
      if (input) {
        e.preventDefault();
        e.stopPropagation();
        if (typeof window.refreshChatComposerRovingTabindex === 'function') {
          window.refreshChatComposerRovingTabindex(null, input);
        }
        input.focus();
        try {
          const len = (input.value || '').length;
          input.setSelectionRange(len, len);
        } catch (_) {
          /* ignore */
        }
      }
      return;
    }
    if (forward) next = Math.min(idx + 1, chips.length - 1);
    else if (back) next = Math.max(idx - 1, 0);
    else if (e.key === 'Home') next = 0;
    else if (e.key === 'End') next = chips.length - 1;
    else return;
    e.preventDefault();
    e.stopPropagation();
    if (next === idx) return;
    refreshChatEmptySuggestionRovingTabindex(chips[next]);
    chips[next].focus();
  });
}

/**
 * Show a calm empty-state hint when the chat pane has no messages yet.
 */
function ensureChatEmptyHint() {
  const container = document.getElementById('chat-messages');
  if (!container) return;
  if (container.querySelector('.chat-message')) {
    container.querySelector('.chat-empty:not(.chat-filter-miss)')?.remove();
    applyChatListFilter();
    applyChatOfflineAttentionGlanceState();
    return;
  }
  ensureChatFilterMissState(container, false);
  const existing = container.querySelector('.chat-empty:not(.chat-filter-miss)');
  if (existing) {
    const row = existing.querySelector('.chat-empty-suggestions');
    if (row) ensureChatEmptySuggestionsToolbarKeyboard(row);
    applyChatListFilter();
    applyChatOfflineAttentionGlanceState();
    return;
  }
  const empty = document.createElement('div');
  empty.className = 'chat-empty';
  empty.setAttribute('role', 'status');

  const title = document.createElement('p');
  title.className = 'chat-empty-title';
  title.textContent = 'Nothing in this chat yet';
  empty.appendChild(title);

  const copy = document.createElement('p');
  copy.className = 'chat-empty-copy';
  copy.textContent =
    'Ask about CPU, RAM, schedules, or tasks — answers stay on this Mac.';
  empty.appendChild(copy);

  const row = document.createElement('div');
  row.className = 'chat-empty-suggestions';
  CHAT_EMPTY_SUGGESTIONS.forEach((item) => {
    const btn = document.createElement('button');
    btn.type = 'button';
    btn.className = 'chat-empty-chip';
    btn.textContent = item.label;
    btn.dataset.prompt = item.prompt;
    btn.dataset.idleLabel = item.label;
    btn.title = 'Put this in the composer — then Send or Enter';
    btn.addEventListener('click', (e) => {
      e.preventDefault();
      e.stopPropagation();
      applyChatEmptySuggestion(item.prompt, btn);
    });
    row.appendChild(btn);
  });
  empty.appendChild(row);

  empty.addEventListener('click', (e) => {
    if (e.target.closest('.chat-empty-chip')) return;
    document.getElementById('chat-input')?.focus();
  });

  container.appendChild(empty);
  ensureChatEmptySuggestionsToolbarKeyboard(row);
  applyChatListFilter();
  applyChatOfflineAttentionGlanceState();
}

function clearChatEmptyHint() {
  document
    .getElementById('chat-messages')
    ?.querySelector('.chat-empty:not(.chat-filter-miss)')
    ?.remove();
  applyChatOfflineAttentionGlanceState();
}

/**
 * Replace history (e.g. Agent Ops session resume). Caps at last 20 turns.
 * Also rebuilds the chat message UI when present.
 * @param {{role: string, content: string}[]} messages
 */
function replaceConversationHistory(messages) {
  const list = Array.isArray(messages) ? messages : [];
  conversationHistory = list
    .filter((m) => m && (m.role === 'user' || m.role === 'assistant') && typeof m.content === 'string')
    .map((m) => ({ role: m.role, content: m.content }))
    .slice(-20);
  const container = document.getElementById('chat-messages');
  if (container) {
    container.innerHTML = '';
    for (const m of conversationHistory) {
      addChatMessage(m.role, m.content);
    }
    if (!conversationHistory.length) ensureChatEmptyHint();
  }
  const chat = document.getElementById('ollama-chat');
  if (chat && conversationHistory.length) {
    chat.style.display = '';
  }
  console.log('[Ollama] Replaced conversation history:', conversationHistory.length, 'messages');
  applyChatListFilter();
  updateChatClearButton();
}

/**
 * Send chat message to Ollama using unified command
 */
/** In-flight chat send: blocks double Enter/click and labels the Send button. */
let chatSendInFlight = false;
let chatSendFlashTimer = null;

function getChatSendButton() {
  return document.getElementById('chat-send-btn');
}

function setChatSendBusy(busy) {
  const btn = getChatSendButton();
  chatSendInFlight = !!busy;
  if (btn) {
    if (chatSendFlashTimer) {
      clearTimeout(chatSendFlashTimer);
      chatSendFlashTimer = null;
    }
    btn.classList.remove('is-just-saved');
    if (busy) {
      if (btn.dataset.idleLabel == null) btn.dataset.idleLabel = btn.textContent || 'Send';
      btn.disabled = true;
      btn.textContent = 'Sending…';
    } else {
      btn.disabled = false;
      btn.textContent = btn.dataset.idleLabel || 'Send';
    }
  }
  updateChatClearButton();
  applyChatOfflineAttentionGlanceState();
  syncOllamaCollapsedGlance();
}

/** Brief success flash on Send (save-button-feedback parity). */
function flashChatSendSent() {
  const btn = getChatSendButton();
  if (!btn) return;
  if (chatSendFlashTimer) {
    clearTimeout(chatSendFlashTimer);
    chatSendFlashTimer = null;
  }
  const idle = btn.dataset.idleLabel || 'Send';
  btn.disabled = false;
  btn.classList.add('is-just-saved');
  btn.textContent = 'Sent';
  chatSendFlashTimer = setTimeout(() => {
    btn.classList.remove('is-just-saved');
    btn.textContent = idle;
    chatSendFlashTimer = null;
  }, 1600);
}

async function sendChatMessage() {
  const input = document.getElementById('chat-input');
  const message = input?.value.trim();
  
  if (!message || !input) {
    console.log('[Ollama] Empty message or input not found');
    return;
  }
  if (chatSendInFlight) {
    console.log('[Ollama] Send ignored — already in flight');
    return;
  }

  console.log('[Ollama] ========== sendChatMessage() CALLED ==========');
  console.log('[Ollama] Message:', message);
  console.log('[Ollama] Conversation history length:', conversationHistory.length);

  input.value = '';
  setChatSendBusy(true);
  let sendOk = false;

  try {
    // Reserved words: meta-commands only — no user bubble, no history, no Ollama (022 §F8).
    if (message === '--cpu') {
      try {
        await invoke('toggle_cpu_window');
        addChatMessage('assistant', 'CPU window toggled.');
        sendOk = true;
      } catch (err) {
        addChatMessage('assistant', `Error: ${err}`);
      }
      return;
    }
    if (message === '-v' || message === '-vv' || message === '-vvv') {
      const level = message.length - 1; // -v=1, -vv=2, -vvv=3
      try {
        await invoke('set_chat_verbosity', { level });
        const labels = { 1: 'warn (-v)', 2: 'debug (-vv)', 3: 'trace (-vvv)' };
        addChatMessage('assistant', `Verbosity set to ${labels[level]}.`);
        sendOk = true;
      } catch (err) {
        addChatMessage('assistant', `Error: ${err}`);
      }
      return;
    }

    addChatMessage('user', message);
    addToHistory('user', message);

    const systemPrompt = getSystemPrompt();
    const history = getConversationHistory().slice(0, -1);

    let useStreaming = false;
    let unlistenChunk = null;
    const listenFn = getListen();
    if (listenFn) {
      try {
        addChatMessage('assistant', '');
        unlistenChunk = await listenFn('ollama-chat-chunk', (event) => {
          const content = event?.payload?.content;
          if (typeof content === 'string') appendToLastAssistantMessage(content);
        });
        useStreaming = true;
      } catch (_) {
        const container = document.getElementById('chat-messages');
        const assistantMessages = container?.querySelectorAll('.chat-message.assistant');
        if (assistantMessages?.length) assistantMessages[assistantMessages.length - 1].remove();
      }
    }

    try {
      console.log('[Ollama] Sending request via unified command (stream=', useStreaming, ')...');
      const response = await invoke('ollama_chat_with_execution', {
        request: {
          question: message,
          ...(systemPrompt ? { system_prompt: systemPrompt } : {}),
          conversation_history: history.length > 0 ? history : null,
          stream: useStreaming
        }
      });

      if (unlistenChunk && typeof unlistenChunk === 'function') {
        unlistenChunk();
        unlistenChunk = null;
      }

      console.log('[Ollama] ✅ Response received:', response);

      if (response.error) {
        const errText = `Error: ${response.error}`;
        if (useStreaming) {
          const container = document.getElementById('chat-messages');
          const assistantMessages = container?.querySelectorAll('.chat-message.assistant');
          const last = assistantMessages?.[assistantMessages.length - 1];
          if (last && !last.textContent.trim()) {
            last.textContent = errText;
            if (last.dataset) last.dataset.copyText = errText;
            last.classList.add('is-error');
            applyChatListFilter();
          } else {
            addChatMessage('assistant', errText);
          }
        } else {
          addChatMessage('assistant', errText);
        }
        return;
      }

      if (response.needs_code_execution && response.code) {
        await executeCodeAndContinue(response, message, systemPrompt, 0);
        sendOk = true;
      } else if (response.final_answer) {
        // Instant / non-stream replies still set stream=true in the UI, which creates an
        // empty assistant bubble and never emits chunks — fill that bubble instead of skipping.
        if (useStreaming) {
          const container = document.getElementById('chat-messages');
          const assistantMessages = container?.querySelectorAll('.chat-message.assistant');
          const last = assistantMessages?.[assistantMessages.length - 1];
          if (last) {
            // Instant (empty bubble) or streamed text → render Markdown for readable layout.
            setAssistantMessageContent(last, response.final_answer);
          } else {
            addChatMessage('assistant', response.final_answer);
          }
        } else {
          addChatMessage('assistant', response.final_answer);
        }
        addToHistory('assistant', response.final_answer, response.attachment_paths);
        sendOk = true;
      } else {
        if (!useStreaming) addChatMessage('assistant', 'Received unexpected response format');
        else addChatMessage('assistant', 'Received unexpected response format');
      }
    } catch (err) {
      if (unlistenChunk && typeof unlistenChunk === 'function') unlistenChunk();
      console.error('[Ollama] Failed to send chat message:', err);
      if (useStreaming) {
        const container = document.getElementById('chat-messages');
        const assistantMessages = container?.querySelectorAll('.chat-message.assistant');
        const last = assistantMessages?.[assistantMessages.length - 1];
        if (last && !last.textContent.trim()) last.textContent = `Error: ${err}`;
        else addChatMessage('assistant', `Error: ${err}`);
      } else {
        addChatMessage('assistant', `Error: ${err}`);
      }
    }
  } finally {
    setChatSendBusy(false);
    if (sendOk) flashChatSendSent();
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
  
  addChatMessage('assistant', `<span class="chat-status">Executing code (step ${iteration + 1})…</span>`, true);
  
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
      lastMessage.innerHTML = `<div class="chat-exec-card">
        <div class="chat-exec-label">Code executed${stepText}</div>
        <pre class="chat-exec-code"><code>${escapeHtml(response.code)}</code></pre>
        <div class="chat-exec-label">Result</div>
        <div class="chat-exec-result">${escapeHtml(resultString)}</div>
        <div class="chat-status">Getting response from AI…</div>
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
        system_prompt: systemPrompt ?? undefined,
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
        
        // Remove intermediate message and show answer(s)
        if (lastMessage && lastMessage.textContent.includes('Getting response')) {
          messagesContainer.removeChild(lastMessage);
        }
        
        // Show both intermediate and final when we have both (so user can see if intermediate was correct)
        const intermediate = (response.intermediate_response || '').trim();
        const finalText = (continueResponse.final_answer || '').trim();
        const displayText = addIntermediateFinalAnswers(intermediate, finalText);
        addToHistory('assistant', displayText, continueResponse.attachment_paths);
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
    addChatMessage('assistant', '<span class="chat-status">Executing code to gather information…</span>', true);
    
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
        lastMessage.innerHTML = `<div class="chat-exec-card">
          <div class="chat-exec-label">Code executed</div>
          <pre class="chat-exec-code"><code>${escapeHtml(code)}</code></pre>
          <div class="chat-exec-label">Result</div>
          <div class="chat-exec-result">${escapeHtml(resultString)}</div>
          <div class="chat-status">Getting final answer from AI…</div>
        </div>`;
      }
      
      // Send follow-up to Ollama (use soul-based default when no custom system prompt)
      const systemPrompt = getSystemPrompt() || await invoke('get_default_ollama_system_prompt');
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
      
      // Display both intermediate and final so user can see if intermediate was correct
      const finalAnswer = (followUpResponse.message.content || '').trim();
      const intermediateContent = (responseContent || '').trim();
      
      // Remove the intermediate message and add the combined answer(s)
      if (lastMessage && lastMessage.textContent.includes('Getting final answer')) {
        messagesContainer.removeChild(lastMessage);
      }
      
      addIntermediateFinalAnswers(intermediateContent, finalAnswer);
      
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
 * Render assistant markdown to HTML (escaped fallback).
 */
function renderMarkdownHtml(text) {
  if (typeof marked !== 'undefined') {
    try {
      marked.setOptions({ breaks: true, gfm: true });
      return marked.parse(String(text ?? ''));
    } catch (_) {
      /* fall through */
    }
  }
  return escapeHtml(text);
}

/**
 * Show intermediate + final code-exec answers as glass-labeled parts.
 * Returns plain text for conversation history.
 */
function addIntermediateFinalAnswers(intermediate, finalText) {
  const inter = String(intermediate || '').trim();
  const fin = String(finalText || '').trim();
  if (!inter) {
    addChatMessage('assistant', fin);
    return fin;
  }
  const html = `<div class="chat-answer-stack">
      <div class="chat-answer-part">
        <div class="chat-exec-label">Intermediate</div>
        <div class="markdown">${renderMarkdownHtml(inter)}</div>
      </div>
      <div class="chat-answer-part chat-answer-final">
        <div class="chat-exec-label">Final</div>
        <div class="markdown">${renderMarkdownHtml(fin)}</div>
      </div>
    </div>`;
  addChatMessage('assistant', html, true);
  return `--- Intermediate answer ---\n\n${inter}\n\n--- Final answer ---\n\n${fin}`;
}

/**
 * Add a chat message to the chat container.
 * Assistant replies use Markdown when `marked` is available (readable lists/paragraphs).
 */
function addChatMessage(role, content, isHtml = false) {
  const messagesContainer = document.getElementById('chat-messages');
  if (!messagesContainer) {
    console.warn('[Ollama] chat-messages container not found');
    return;
  }

  clearChatEmptyHint();

  const messageDiv = document.createElement('div');
  messageDiv.className = `chat-message ${role}`;
  if (role === 'assistant' && !isHtml && isChatErrorText(content)) {
    messageDiv.classList.add('is-error');
  }

  if (isHtml) {
    messageDiv.innerHTML = content;
    decorateChatMessageForCopy(
      messageDiv,
      role,
      String(messageDiv.innerText || messageDiv.textContent || '').trim()
    );
    syncChatMessageErrorClass(messageDiv);
  } else if (role === 'assistant' && typeof marked !== 'undefined' && !isChatErrorText(content)) {
    try {
      marked.setOptions({ breaks: true, gfm: true });
      const markdownWrapper = document.createElement('div');
      markdownWrapper.className = 'markdown';
      markdownWrapper.innerHTML = marked.parse(String(content ?? ''));
      messageDiv.appendChild(markdownWrapper);
      decorateChatMessageForCopy(messageDiv, role, content);
    } catch (err) {
      console.warn('[Ollama] markdown render failed, falling back to text', err);
      messageDiv.textContent = content;
      decorateChatMessageForCopy(messageDiv, role, content);
    }
  } else {
    messageDiv.textContent = content;
    decorateChatMessageForCopy(messageDiv, role, content);
  }

  wireChatMessagesCopy(messagesContainer);
  messagesContainer.appendChild(messageDiv);
  applyChatListFilter();
  if (messageDiv.style.display !== 'none') {
    messagesContainer.scrollTop = messagesContainer.scrollHeight;
  }
  updateChatClearButton();
}

/**
 * Replace an assistant bubble's content (used for instant replies into an empty stream bubble).
 */
function setAssistantMessageContent(el, content) {
  if (!el) return;
  el.replaceChildren();
  el.classList.toggle('is-error', isChatErrorText(content));
  if (typeof marked !== 'undefined' && !isChatErrorText(content)) {
    try {
      marked.setOptions({ breaks: true, gfm: true });
      const markdownWrapper = document.createElement('div');
      markdownWrapper.className = 'markdown';
      markdownWrapper.innerHTML = marked.parse(String(content ?? ''));
      el.appendChild(markdownWrapper);
      decorateChatMessageForCopy(el, 'assistant', content);
      applyChatListFilter();
      return;
    } catch (_) {
      /* fall through */
    }
  }
  el.textContent = content;
  decorateChatMessageForCopy(el, 'assistant', content);
  applyChatListFilter();
}

/**
 * Append text to the last assistant message (for streaming). Safe: appends as text node.
 */
function appendToLastAssistantMessage(text) {
  const messagesContainer = document.getElementById('chat-messages');
  if (!messagesContainer) return;
  const assistantMessages = messagesContainer.querySelectorAll('.chat-message.assistant');
  const last = assistantMessages[assistantMessages.length - 1];
  if (!last) return;
  last.appendChild(document.createTextNode(text));
  const prev = (last.dataset && last.dataset.copyText) || '';
  last.dataset.copyText = `${prev}${text}`;
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

/** Focusable composer items in DOM order (input · Clear when enabled · Send). */
function getChatComposerItems(container) {
  const row = container || document.querySelector('.chat-input-container');
  if (!row) return [];
  const items = [];
  const input = document.getElementById('chat-input');
  const clearBtn = getChatClearButton();
  const sendBtn = document.getElementById('chat-send-btn');
  if (input && row.contains(input) && !input.hidden) items.push(input);
  if (clearBtn && row.contains(clearBtn) && !clearBtn.hidden && !clearBtn.disabled) {
    items.push(clearBtn);
  }
  if (sendBtn && row.contains(sendBtn) && !sendBtn.hidden) items.push(sendBtn);
  return items.filter((el) => {
    if (!el || el.hidden) return false;
    return el.getClientRects().length > 0 || row.contains(el);
  });
}

function chatComposerInputAtMoveBoundary(input, direction) {
  if (!input || input.tagName !== 'INPUT') return true;
  if (direction > 0) {
    const len = (input.value || '').length;
    return input.selectionStart === len && input.selectionEnd === len;
  }
  return input.selectionStart === 0 && input.selectionEnd === 0;
}

function refreshChatComposerRovingTabindex(container, preferred) {
  const row = container || document.querySelector('.chat-input-container');
  const items = getChatComposerItems(row);
  if (!items.length) return;
  const focused = items.find((el) => el === document.activeElement);
  const current =
    (preferred && items.includes(preferred) && preferred) ||
    focused ||
    items.find((el) => el.tabIndex === 0) ||
    items[0];
  for (const el of items) {
    el.tabIndex = el === current ? 0 : -1;
  }
}

function isChatComposerRowVisible(row) {
  if (!row || row.hidden) return false;
  try {
    return row.getClientRects().length > 0;
  } catch (_) {
    return false;
  }
}

/** Focus first composer control (input). Used by message-list → composer chain. */
function focusChatComposerFirst() {
  const row = document.querySelector('.chat-input-container');
  if (!isChatComposerRowVisible(row)) return false;
  const items = getChatComposerItems(row);
  if (!items.length) return false;
  refreshChatComposerRovingTabindex(row, items[0]);
  items[0].focus();
  if (items[0]?.id === 'chat-input' && typeof items[0].setSelectionRange === 'function') {
    try {
      const len = (items[0].value || '').length;
      items[0].setSelectionRange(len, len);
    } catch (_) {
      /* ignore */
    }
  }
  return true;
}

/** Focus last composer control (Send). Used by footer ← composer chain. */
function focusChatComposerLast() {
  const row = document.querySelector('.chat-input-container');
  if (!isChatComposerRowVisible(row)) return false;
  const items = getChatComposerItems(row);
  if (!items.length) return false;
  const target = items[items.length - 1];
  refreshChatComposerRovingTabindex(row, target);
  target.focus();
  return true;
}

function ensureChatComposerKbHint(container) {
  const row = container || document.querySelector('.chat-input-container');
  if (!row) return;
  let hint = row.querySelector('.chat-composer-kb-hint');
  if (!hint) {
    hint = document.createElement('div');
    hint.className = 'chat-composer-kb-hint';
    hint.setAttribute('aria-hidden', 'true');
    row.appendChild(hint);
  }
  const items = getChatComposerItems(row);
  hint.hidden = items.length < 2;
  const hasStarters = getChatEmptySuggestionChips().length > 0;
  const hasMessages = visibleChatMessages(document.getElementById('chat-messages')).length > 0;
  if (hasStarters) {
    hint.textContent =
      '← → / h l · Home/End move · Enter sends · at start crosses to starter chips · last → footer · Clear / Send on button';
  } else if (hasMessages) {
    hint.textContent =
      '← → / h l · Home/End move · Enter sends · at start crosses to last message · last → footer · Clear / Send on button';
  } else {
    hint.textContent =
      '← → / h l · Home/End move · Enter sends from input · Clear / Send on button';
  }
}

/**
 * AI Chat composer toolbar keyboard — focus input · Clear · Send, then ←→ / h l /
 * Home/End (filter-row parity). Input keeps normal typing; arrows move only at
 * text start/end. One Tab stop via roving tabindex.
 */
function ensureChatComposerToolbarKeyboard() {
  const row = document.querySelector('.chat-input-container');
  if (!row) return;
  ensureChatComposerKbHint(row);
  refreshChatComposerRovingTabindex(row);
  if (row.dataset.chatComposerKbWired === '1') return;
  row.dataset.chatComposerKbWired = '1';
  if (!row.getAttribute('role')) row.setAttribute('role', 'toolbar');
  if (!row.getAttribute('aria-label')) row.setAttribute('aria-label', 'AI Chat composer');
  row.addEventListener('focusin', (e) => {
    const items = getChatComposerItems(row);
    if (items.includes(e.target)) {
      refreshChatComposerRovingTabindex(row, e.target);
      ensureChatComposerKbHint(row);
    }
  });
  row.addEventListener('keydown', (e) => {
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    const items = getChatComposerItems(row);
    if (!items.length) return;
    const idx = items.indexOf(document.activeElement);
    if (idx < 0) return;
    const active = items[idx];
    if (e.key === 'Enter' || e.key === ' ') {
      if (active === document.getElementById('chat-input')) return;
      if (active?.id === 'chat-clear-btn' || active?.id === 'chat-send-btn') return;
    }
    let next = -1;
    const forward =
      e.key === 'ArrowRight' ||
      e.key === 'l' ||
      e.key === 'ArrowDown' ||
      e.key === 'j';
    const back =
      e.key === 'ArrowLeft' ||
      e.key === 'h' ||
      e.key === 'ArrowUp' ||
      e.key === 'k';
    if (forward) {
      if (active?.id === 'chat-input' && !chatComposerInputAtMoveBoundary(active, 1)) {
        return;
      }
      if (
        idx === items.length - 1 &&
        typeof window.tryChainFilterChipToFooterFirst === 'function' &&
        window.tryChainFilterChipToFooterFirst()
      ) {
        e.preventDefault();
        e.stopPropagation();
        return;
      }
      next = Math.min(idx + 1, items.length - 1);
    } else if (back) {
      if (active?.id === 'chat-input' && !chatComposerInputAtMoveBoundary(active, -1)) {
        return;
      }
      if (idx === 0 && focusChatEmptySuggestionLast()) {
        e.preventDefault();
        e.stopPropagation();
        return;
      }
      if (
        idx === 0 &&
        focusChatMessagesLast(document.getElementById('chat-messages'))
      ) {
        e.preventDefault();
        e.stopPropagation();
        return;
      }
      next = Math.max(idx - 1, 0);
    } else if (e.key === 'Home') {
      next = 0;
    } else if (e.key === 'End') {
      next = items.length - 1;
    } else {
      return;
    }
    e.preventDefault();
    e.stopPropagation();
    if (next === idx) return;
    refreshChatComposerRovingTabindex(row, items[next]);
    items[next].focus();
    if (items[next]?.id === 'chat-input' && typeof items[next].select === 'function') {
      const len = (items[next].value || '').length;
      items[next].setSelectionRange(len, len);
    }
  });
}

// ============================================================================
// Event Listeners Setup
// ============================================================================

/**
 * Initialize Ollama chat event listeners
 */
let compactionStatusDismissTimer = null;

function setChatCompactionStatus(html, showSpinner) {
  const el = document.getElementById('chat-compaction-status');
  if (!el) return;
  if (!html) {
    el.hidden = true;
    el.innerHTML = '';
    return;
  }
  el.hidden = false;
  if (showSpinner) {
    el.innerHTML = `<span class="compaction-spinner" aria-hidden="true"></span><span>${html}</span>`;
  } else {
    el.innerHTML = `<span>${html}</span>`;
  }
}

async function setupCompactionStatusListener() {
  const listenFn = getListen();
  if (!listenFn) return;
  try {
    await listenFn('mac-stats-compaction', (event) => {
      const data = event?.payload?.data;
      if (!data || typeof data !== 'object') return;
      const phase = data.phase;
      if (phase === 'start') {
        if (compactionStatusDismissTimer) {
          clearTimeout(compactionStatusDismissTimer);
          compactionStatusDismissTimer = null;
        }
        setChatCompactionStatus('Compacting context…', true);
      } else if (phase === 'end') {
        if (compactionStatusDismissTimer) {
          clearTimeout(compactionStatusDismissTimer);
          compactionStatusDismissTimer = null;
        }
        if (data.ok === true) {
          setChatCompactionStatus('Context compacted', false);
          compactionStatusDismissTimer = setTimeout(() => {
            setChatCompactionStatus('', false);
            compactionStatusDismissTimer = null;
          }, 4000);
        } else {
          setChatCompactionStatus('', false);
        }
      }
    });
  } catch (e) {
    console.warn('[Ollama] mac-stats-compaction listener failed:', e?.message || e);
  }
}

function initOllamaChatListeners() {
  const chatInput = document.getElementById('chat-input');
  const chatSendBtn = document.getElementById('chat-send-btn');
  const chatClearBtn = document.getElementById('chat-clear-btn');
  
  if (!chatInput || !chatSendBtn) {
    console.warn('[Ollama] Chat input or send button not found');
    return;
  }

  void setupCompactionStatusListener();

  if (!chatInput.placeholder || chatInput.placeholder.includes('system metrics')) {
    chatInput.placeholder = 'Ask about metrics, tasks, or the web…';
  }
  wireChatMessagesCopy(document.getElementById('chat-messages'));
  ensureChatEmptyHint();
  updateChatClearButton();
  ensureChatComposerToolbarKeyboard();
  
  // Send button click
  chatSendBtn.addEventListener('click', () => {
    console.log('[Ollama] Send button clicked');
    sendChatMessage();
  });

  if (chatClearBtn) {
    if (chatClearBtn.dataset.idleLabel == null) {
      chatClearBtn.dataset.idleLabel = chatClearBtn.textContent || 'Clear';
    }
    chatClearBtn.addEventListener('click', () => {
      console.log('[Ollama] Clear button clicked');
      clearChatWithFeedback();
    });
  }
  
  // Enter sends (ignore while in flight; Shift+Enter left for future multiline)
  chatInput.addEventListener('keydown', (e) => {
    if (e.key !== 'Enter' || e.shiftKey) return;
    e.preventDefault();
    if (chatSendInFlight) return;
    console.log('[Ollama] Enter key pressed');
    sendChatMessage();
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
  ensureOllamaCollapsedGlance();
  ensureChatModelGlance();
  ensureChatTurnGlance();
  applyChatModelGlanceState();
  syncOllamaCollapsedGlance();
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
  getDefaultModel: getDefaultModel,
  
  // Chat
  sendMessage: sendChatMessage,
  processResponse: processOllamaResponse,
  getHistory: getConversationHistory,
  clearHistory: clearConversationHistory,
  clearChat: clearChatWithFeedback,
  replaceHistory: replaceConversationHistory,
  
  // Code execution
  executeCode: executeJavaScriptCode,
  
  // UI
  addMessage: addChatMessage,
  setAssistantMessageContent: setAssistantMessageContent,
  initListeners: initOllamaChatListeners,
  syncCollapsedGlance: syncOllamaCollapsedGlance,
  focusEmptySuggestionFirst: focusChatEmptySuggestionFirst,
  focusEmptySuggestionLast: focusChatEmptySuggestionLast,
  
  // Utils
  getEndpoint: getOllamaEndpoint,
  saveOllamaEndpoint: saveOllamaEndpoint,
  getSystemPrompt: getSystemPrompt,
  escapeHtml: escapeHtml
};

window.focusChatEmptySuggestionFirst = focusChatEmptySuggestionFirst;
window.focusChatEmptySuggestionLast = focusChatEmptySuggestionLast;
window.focusChatComposerFirst = focusChatComposerFirst;
window.focusChatComposerLast = focusChatComposerLast;
window.refreshChatComposerRovingTabindex = refreshChatComposerRovingTabindex;
