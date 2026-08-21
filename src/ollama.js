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

    chatModelGlanceState = {
      status: connected ? 'connected' : connectionFailed ? 'error' : 'unknown',
      model: String(localStorage.getItem('ollama_model') || '').trim(),
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
    chatModelGlanceState = { status: 'error', model: '' };
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
  }
  applyChatListFilter();
  updateChatClearButton();
}

/** Transcript role filter (Monitors All/Up/Down parity). */
let chatFilterMode = 'all';

/** All · You · Assistant chips above the message list. */
function ensureChatFilterChips() {
  const chat = document.getElementById('ollama-chat');
  const messages = document.getElementById('chat-messages');
  if (!chat || !messages || !messages.parentNode) return;
  if (document.getElementById('chat-filter-chips')) return;
  const wrap = document.createElement('div');
  wrap.id = 'chat-filter-chips';
  wrap.className = 'chat-filter-chips';
  wrap.setAttribute('role', 'group');
  wrap.setAttribute('aria-label', 'Chat message filter');
  wrap.hidden = true;
  wrap.innerHTML =
    '<button type="button" class="chat-filter-chip is-active" data-chat-filter="all" aria-pressed="true" title="Show every message">All</button>' +
    '<button type="button" class="chat-filter-chip" data-chat-filter="you" aria-pressed="false" title="Show your messages only">You <span class="chat-filter-count" data-chat-filter-count="you">0</span></button>' +
    '<button type="button" class="chat-filter-chip" data-chat-filter="assistant" aria-pressed="false" title="Show assistant replies only">Assistant <span class="chat-filter-count" data-chat-filter-count="assistant">0</span></button>';
  messages.parentNode.insertBefore(wrap, messages);
  wrap.addEventListener('click', (e) => {
    const btn = e.target && e.target.closest && e.target.closest('[data-chat-filter]');
    if (!btn || !wrap.contains(btn)) return;
    e.preventDefault();
    e.stopPropagation();
    setChatFilterMode(btn.getAttribute('data-chat-filter') || 'all');
  });
}

function setChatFilterMode(mode) {
  const next = mode === 'you' || mode === 'assistant' ? mode : 'all';
  chatFilterMode = next;
  document.querySelectorAll('#chat-filter-chips [data-chat-filter]').forEach((btn) => {
    const on = btn.getAttribute('data-chat-filter') === next;
    btn.classList.toggle('is-active', on);
    btn.setAttribute('aria-pressed', on ? 'true' : 'false');
  });
  applyChatListFilter();
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
      `<div class="chat-filter-miss-hint">Try All, or clear the role filter.</div>` +
      `<button type="button" class="chat-filter-miss-cta chat-clear-filter">Clear filter</button>`;
    container.appendChild(wrap);
    wrap.querySelector('.chat-clear-filter')?.addEventListener('click', (e) => {
      e.preventDefault();
      e.stopPropagation();
      setChatFilterMode('all');
    });
  }
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
  items.forEach((el) => {
    if (el.classList.contains('user')) youCount++;
    else if (el.classList.contains('assistant')) assistantCount++;
  });

  const youEl = document.querySelector('[data-chat-filter-count="you"]');
  const asstEl = document.querySelector('[data-chat-filter-count="assistant"]');
  if (youEl) youEl.textContent = String(youCount);
  if (asstEl) asstEl.textContent = String(assistantCount);
  document.querySelectorAll('#chat-filter-chips [data-chat-filter]').forEach((btn) => {
    const key = btn.getAttribute('data-chat-filter');
    btn.classList.toggle(
      'has-hits',
      key === 'you' ? youCount > 0 : key === 'assistant' ? assistantCount > 0 : false
    );
  });

  if (trueEmpty || items.length === 0) {
    ensureChatFilterMissState(container, false);
    return;
  }

  let visible = 0;
  items.forEach((el) => {
    const isYou = el.classList.contains('user');
    const isAsst = el.classList.contains('assistant');
    let show = true;
    if (chatFilterMode === 'you') show = isYou;
    else if (chatFilterMode === 'assistant') show = isAsst;
    el.style.display = show ? '' : 'none';
    if (show) visible++;
  });

  ensureChatFilterMissState(container, visible === 0);
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
let chatModelGlanceState = { status: 'unknown', model: '' };

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
  if (!content.classList.contains('collapsed')) return;
  content.classList.remove('collapsed');
  section?.classList.remove('collapsed');
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
    return;
  }
  // One strip while collapsed — hide the expanded model/turn/answer glances.
  const model = document.getElementById('chat-model-glance');
  const turn = document.getElementById('chat-turn-glance');
  const answer = document.getElementById('chat-answer-glance');
  if (model) model.hidden = true;
  if (turn) turn.hidden = true;
  if (answer) answer.hidden = true;

  glance.hidden = false;
  const status = chatModelGlanceState.status || 'unknown';
  const modelName = getChatModelGlanceLabel();
  const turns = countChatTurns();
  const preview = getChatTurnGlancePreview();
  let line = 'AI Chat';
  let wash = 'is-empty';
  if (status === 'error') {
    line = 'Offline · check Ollama';
    wash = 'is-offline';
  } else if (status === 'unknown') {
    line = 'Not set · configure URL';
    wash = 'is-offline';
  } else if (turns && preview) {
    const turnLabel = turns === 1 ? '1 turn' : `${turns} turns`;
    line = `${turnLabel} · ${preview}`;
    wash = chatSendInFlight ? 'is-active' : 'is-online';
  } else if (modelName) {
    line = `Ready · ${modelName}`;
    wash = 'is-online';
  } else {
    line = 'Ready · pick a model';
    wash = 'is-online';
  }
  if (glanceText) glanceText.textContent = line;
  glance.classList.toggle('is-online', wash === 'is-online');
  glance.classList.toggle('is-offline', wash === 'is-offline');
  glance.classList.toggle('is-active', wash === 'is-active');
  glance.classList.toggle('is-empty', wash === 'is-empty');
  glance.setAttribute('role', 'button');
  glance.tabIndex = 0;
  if (wash === 'is-offline') {
    glance.title = 'Open AI Chat — configure Ollama';
    glance.setAttribute('aria-label', `${line} — click to configure`);
  } else if (turns && preview) {
    glance.title = 'Show AI Chat and focus composer';
    glance.setAttribute(
      'aria-label',
      `${line} — click to expand and focus composer`
    );
  } else {
    glance.title = 'Show AI Chat';
    glance.setAttribute('aria-label', `${line} — click to expand`);
  }
}

function activateOllamaCollapsedGlance() {
  const status = chatModelGlanceState.status || 'unknown';
  ensureOllamaSectionExpanded();
  syncOllamaCollapsedGlance();
  applyChatModelGlanceState();
  applyChatTurnGlanceState();
  applyChatAnswerGlanceState();
  if (status !== 'connected') {
    showOllamaUrlDialog();
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
    if (e.key !== 'Enter' && e.key !== ' ') return;
    e.preventDefault();
    e.stopPropagation();
    activate();
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
    const label = model ? `Model · ${model}` : 'Model · pick one';
    if (text) text.textContent = label;
    glance.title = model
      ? `Change model (${model})`
      : 'Choose an Ollama model';
    glance.setAttribute(
      'aria-label',
      model ? `Connected — model ${model}. Click to change.` : 'Connected — choose a model'
    );
    return;
  }
  if (status === 'error') {
    if (text) text.textContent = 'Offline · check Ollama';
    glance.title = 'Ollama is not available — click to set the URL';
    glance.setAttribute('aria-label', 'Ollama offline — click to configure URL');
    return;
  }
  if (text) text.textContent = 'Not set · configure URL';
  glance.title = 'Click to configure the Ollama URL';
  glance.setAttribute('aria-label', 'Ollama not configured — click to set URL');
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
  if (text) text.textContent = `${turnLabel} · ${preview}`;
  glance.classList.toggle('is-active', chatSendInFlight);
  glance.setAttribute('role', 'button');
  glance.tabIndex = 0;
  glance.title = 'Scroll to latest message and focus composer';
  glance.setAttribute(
    'aria-label',
    `${turnLabel} — last question "${preview}" — scroll to latest`
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
    el.title = prevTitle || 'Click to copy message';
    const role = el.classList.contains('user') ? 'your' : 'assistant';
    el.setAttribute('aria-label', `Copy ${role} message`);
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

/** Wire click / Enter / Space copy on chat bubbles (last-answer glance parity). */
function wireChatMessagesCopy(container) {
  if (!container || container.dataset.copyWired === '1') return;
  container.dataset.copyWired = '1';
  const activate = (msg) => {
    if (!msg || msg.classList.contains('thinking')) return;
    void copyChatMessageFromUi(msg);
  };
  container.addEventListener('click', (e) => {
    const msg = e.target && e.target.closest && e.target.closest('.chat-message');
    if (!msg || !container.contains(msg)) return;
    if (e.target.closest('a, button, input, textarea, select')) return;
    activate(msg);
  });
  container.addEventListener('keydown', (e) => {
    if (e.key !== 'Enter' && e.key !== ' ') return;
    const msg = e.target && e.target.closest && e.target.closest('.chat-message');
    if (!msg || !container.contains(msg)) return;
    e.preventDefault();
    activate(msg);
  });
}

function decorateChatMessageForCopy(messageDiv, role, plainText) {
  if (!messageDiv) return;
  const text = String(plainText ?? '').trim();
  if (text) messageDiv.dataset.copyText = text;
  messageDiv.setAttribute('role', 'button');
  messageDiv.tabIndex = 0;
  messageDiv.title = 'Click to copy message';
  const who = role === 'user' ? 'your' : 'assistant';
  messageDiv.setAttribute('aria-label', `Copy ${who} message`);
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
    glance.classList.remove('has-answer');
    return;
  }
  glance.hidden = false;
  glance.classList.add('has-answer');
  if (text) text.textContent = `Last answer · ${preview}`;
  glance.setAttribute('role', 'button');
  glance.tabIndex = 0;
  glance.title = 'Copy last answer';
  glance.setAttribute('aria-label', `Copy last answer: ${preview}`);
}

function wireChatAnswerGlanceClick(glance) {
  if (!glance || glance.dataset.answerGlanceWired === '1') return;
  glance.dataset.answerGlanceWired = '1';
  const activate = async () => {
    if (chatSendInFlight) return;
    const answer = getLastAssistantAnswerText();
    if (!answer) return;
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
  applyChatListFilter();
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

/**
 * Show a calm empty-state hint when the chat pane has no messages yet.
 */
function ensureChatEmptyHint() {
  const container = document.getElementById('chat-messages');
  if (!container) return;
  if (container.querySelector('.chat-message')) {
    container.querySelector('.chat-empty:not(.chat-filter-miss)')?.remove();
    applyChatListFilter();
    return;
  }
  ensureChatFilterMissState(container, false);
  if (container.querySelector('.chat-empty:not(.chat-filter-miss)')) {
    applyChatListFilter();
    return;
  }
  const empty = document.createElement('div');
  empty.className = 'chat-empty';
  empty.setAttribute('role', 'status');

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
  applyChatListFilter();
}

function clearChatEmptyHint() {
  document
    .getElementById('chat-messages')
    ?.querySelector('.chat-empty:not(.chat-filter-miss)')
    ?.remove();
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
        if (useStreaming) {
          const container = document.getElementById('chat-messages');
          const assistantMessages = container?.querySelectorAll('.chat-message.assistant');
          const last = assistantMessages?.[assistantMessages.length - 1];
          if (last && !last.textContent.trim()) last.textContent = `Error: ${response.error}`;
          else addChatMessage('assistant', `Error: ${response.error}`);
        } else {
          addChatMessage('assistant', `Error: ${response.error}`);
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

  if (isHtml) {
    messageDiv.innerHTML = content;
    decorateChatMessageForCopy(
      messageDiv,
      role,
      String(messageDiv.innerText || messageDiv.textContent || '').trim()
    );
  } else if (role === 'assistant' && typeof marked !== 'undefined') {
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
  if (typeof marked !== 'undefined') {
    try {
      marked.setOptions({ breaks: true, gfm: true });
      const markdownWrapper = document.createElement('div');
      markdownWrapper.className = 'markdown';
      markdownWrapper.innerHTML = marked.parse(String(content ?? ''));
      el.appendChild(markdownWrapper);
      decorateChatMessageForCopy(el, 'assistant', content);
      return;
    } catch (_) {
      /* fall through */
    }
  }
  el.textContent = content;
  decorateChatMessageForCopy(el, 'assistant', content);
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
  
  // Utils
  getEndpoint: getOllamaEndpoint,
  saveOllamaEndpoint: saveOllamaEndpoint,
  getSystemPrompt: getSystemPrompt,
  escapeHtml: escapeHtml
};
