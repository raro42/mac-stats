// Discord bot token configuration (Settings modal).
// Shared by all themes; expects elements: #discord-status, #discord-token-input, #discord-save-token, #discord-clear-token, #settings-modal.
// Call window.Discord.refreshStatus() when opening Settings to update status (e.g. from cpu-ui.js).

(function () {
  // Match cpu.js getInvoke order exactly (Tauri 1 inject)
  function getInvoke() {
    if (typeof window.__TAURI_INVOKE__ !== "undefined") {
      return window.__TAURI_INVOKE__;
    }
    if (window.__TAURI__?.core?.invoke) {
      return window.__TAURI__.core.invoke;
    }
    if (window.__TAURI__?.tauri?.invoke) {
      return window.__TAURI__.tauri.invoke;
    }
    if (window.__TAURI__?.invoke) {
      return window.__TAURI__.invoke;
    }
    const internals = window.__TAURI_INTERNALS__;
    if (internals && typeof internals.invoke === 'function') {
      return internals.invoke.bind(internals);
    }
    return null;
  }

  function showFeedback(message, isSuccess) {
    const statusEl = document.getElementById("discord-status");
    if (!statusEl) return;
    statusEl.textContent = message;
    statusEl.style.color = isSuccess ? "" : "inherit";
    statusEl.style.fontWeight = isSuccess ? "600" : "inherit";
  }

  async function refreshStatus() {
    const statusEl = document.getElementById("discord-status");
    if (!statusEl) return;
    const invoke = getInvoke();
    if (!invoke) {
      statusEl.textContent = "—";
      return;
    }
    try {
      const configured = await invoke("is_discord_configured");
      statusEl.textContent = configured
        ? "Configured"
        : "Not configured";
      statusEl.style.fontWeight = "";
      statusEl.style.color = "";
    } catch (_) {
      statusEl.textContent = "—";
    }
  }

  function clearInput() {
    const input = document.getElementById("discord-token-input");
    if (input) input.value = "";
  }

  function doSaveToken() {
    const invoke = getInvoke();
    if (!invoke) {
      window.alert("App not ready. Try again in a moment.");
      return;
    }
    const tokenInput = document.getElementById("discord-token-input");
    const trimmed = tokenInput ? tokenInput.value.trim() : "";
    (async function () {
      try {
        await invoke("configure_discord", { token: trimmed || null });
        clearInput();
        if (trimmed) {
          showFeedback("Token saved. Connecting…", true);
          setTimeout(refreshStatus, 4000);
        } else {
          await refreshStatus();
        }
      } catch (err) {
        showFeedback("Failed: " + String(err), false);
        setTimeout(refreshStatus, 4000);
      }
    })();
  }

  function doClearToken() {
    const invoke = getInvoke();
    if (!invoke) {
      window.alert("App not ready. Try again in a moment.");
      return;
    }
    (async function () {
      try {
        await invoke("configure_discord", { token: null });
        clearInput();
        showFeedback("Token cleared. Restart the app to disconnect.", true);
        setTimeout(refreshStatus, 4000);
      } catch (err) {
        showFeedback("Failed: " + String(err), false);
        setTimeout(refreshStatus, 4000);
      }
    })();
  }

  async function doViewLogs() {
    const invoke = getInvoke();
    if (!invoke) {
      window.alert("App not ready. Try again in a moment.");
      return;
    }
    try {
      await invoke("open_debug_log");
    } catch (err) {
      const path = await invoke("get_debug_log_path").catch(() => null);
      const msg = path
        ? "Could not open log file. Path: " + path
        : "Could not open log file: " + String(err);
      window.alert(msg);
    }
  }

  function init() {
    const settingsModal = document.getElementById("settings-modal");
    const saveBtn = document.getElementById("discord-save-token");
    const clearBtn = document.getElementById("discord-clear-token");
    if (saveBtn) {
      saveBtn.disabled = false;
      saveBtn.addEventListener("click", function (e) {
        e.preventDefault();
        e.stopPropagation();
        doSaveToken();
      });
    }
    if (clearBtn) {
      clearBtn.disabled = false;
      clearBtn.addEventListener("click", function (e) {
        e.preventDefault();
        e.stopPropagation();
        doClearToken();
      });
    }
    const viewLogsBtn = document.getElementById("view-debug-log");
    if (viewLogsBtn) {
      viewLogsBtn.disabled = false;
      viewLogsBtn.addEventListener("click", function (e) {
        e.preventDefault();
        e.stopPropagation();
        doViewLogs();
      });
    }
  }

  window.Discord = { refreshStatus: refreshStatus };

  function runInit() {
    init();
  }

  // Defer init so Tauri inject runs first (theme page loads after redirect)
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () {
      setTimeout(runInit, 100);
    });
  } else {
    setTimeout(runInit, 100);
  }
})();
