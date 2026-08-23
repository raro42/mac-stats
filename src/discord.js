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

  let discordTokenBusy = false;

  function setDiscordTokenBusy(busy, which) {
    discordTokenBusy = !!busy;
    const saveBtn = document.getElementById("discord-save-token");
    const clearBtn = document.getElementById("discord-clear-token");
    if (saveBtn) {
      saveBtn.disabled = !!busy;
      if (busy && which === "save") {
        saveBtn.classList.remove("is-just-saved");
        if (saveBtn._saveFlashOriginalLabel == null) {
          saveBtn._saveFlashOriginalLabel = saveBtn.textContent;
        }
        saveBtn.textContent = "Saving…";
      } else if (!busy && !saveBtn.classList.contains("is-just-saved")) {
        saveBtn.textContent =
          saveBtn._saveFlashOriginalLabel || "Save token";
        saveBtn._saveFlashOriginalLabel = null;
      }
    }
    if (clearBtn) {
      clearBtn.disabled = !!busy;
      if (busy && which === "clear") {
        clearBtn.classList.remove("is-just-saved");
        if (clearBtn._saveFlashOriginalLabel == null) {
          clearBtn._saveFlashOriginalLabel = clearBtn.textContent;
        }
        clearBtn.textContent = "Clearing…";
      } else if (!busy && !clearBtn.classList.contains("is-just-saved")) {
        clearBtn.textContent =
          clearBtn._saveFlashOriginalLabel || "Clear token";
        clearBtn._saveFlashOriginalLabel = null;
      }
    }
  }

  function flashDiscordBtn(btn, savedLabel) {
    if (typeof window.flashSaveButton === "function") {
      window.flashSaveButton(btn, { savedLabel: savedLabel, durationMs: 1600 });
      return;
    }
    if (!btn) return;
    const prev = btn.textContent;
    btn.classList.add("is-just-saved");
    btn.textContent = savedLabel;
    setTimeout(function () {
      btn.classList.remove("is-just-saved");
      btn.textContent = prev;
    }, 1600);
  }

  function doSaveToken() {
    if (discordTokenBusy) return;
    const invoke = getInvoke();
    if (!invoke) {
      window.alert("App not ready. Try again in a moment.");
      return;
    }
    const tokenInput = document.getElementById("discord-token-input");
    const trimmed = tokenInput ? tokenInput.value.trim() : "";
    const saveBtn = document.getElementById("discord-save-token");
    (async function () {
      setDiscordTokenBusy(true, "save");
      try {
        await invoke("configure_discord", { token: trimmed || null });
        clearInput();
        setDiscordTokenBusy(false);
        if (trimmed) {
          showFeedback("Token saved. Connecting…", true);
          flashDiscordBtn(saveBtn, "Saved");
          setTimeout(refreshStatus, 4000);
        } else {
          await refreshStatus();
          flashDiscordBtn(saveBtn, "Saved");
        }
      } catch (err) {
        setDiscordTokenBusy(false);
        showFeedback("Failed: " + String(err), false);
        setTimeout(refreshStatus, 4000);
      }
    })();
  }

  function doClearToken() {
    if (discordTokenBusy) return;
    const invoke = getInvoke();
    if (!invoke) {
      window.alert("App not ready. Try again in a moment.");
      return;
    }
    const clearBtn = document.getElementById("discord-clear-token");
    (async function () {
      setDiscordTokenBusy(true, "clear");
      try {
        await invoke("configure_discord", { token: null });
        clearInput();
        setDiscordTokenBusy(false);
        showFeedback("Token cleared. Restart the app to disconnect.", true);
        flashDiscordBtn(clearBtn, "Cleared");
        setTimeout(refreshStatus, 4000);
      } catch (err) {
        setDiscordTokenBusy(false);
        showFeedback("Failed: " + String(err), false);
        setTimeout(refreshStatus, 4000);
      }
    })();
  }

  async function doViewLogs() {
    const viewLogsBtn = document.getElementById("view-debug-log");
    if (viewLogsBtn && (viewLogsBtn.disabled || viewLogsBtn.classList.contains("is-just-saved"))) {
      return;
    }
    const invoke = getInvoke();
    if (!invoke) {
      window.alert("App not ready. Try again in a moment.");
      return;
    }
    const originalLabel =
      (viewLogsBtn &&
        (viewLogsBtn._saveFlashOriginalLabel || viewLogsBtn.textContent)) ||
      "View logs";
    if (viewLogsBtn) {
      viewLogsBtn._saveFlashOriginalLabel = originalLabel;
      viewLogsBtn.disabled = true;
      viewLogsBtn.classList.remove("is-just-saved");
      viewLogsBtn.textContent = "Opening…";
    }
    try {
      await invoke("open_debug_log");
      if (viewLogsBtn) {
        viewLogsBtn.disabled = false;
        if (typeof window.flashSaveButton === "function") {
          window.flashSaveButton(viewLogsBtn, {
            savedLabel: "Opened",
            durationMs: 1600,
          });
        } else {
          flashDiscordBtn(viewLogsBtn, "Opened");
        }
      }
    } catch (err) {
      if (viewLogsBtn) {
        viewLogsBtn.disabled = false;
        viewLogsBtn.classList.remove("is-just-saved");
        viewLogsBtn.textContent = originalLabel;
        viewLogsBtn._saveFlashOriginalLabel = null;
      }
      const path = await invoke("get_debug_log_path").catch(() => null);
      const msg = path
        ? "Could not open log file. Path: " + path
        : "Could not open log file: " + String(err);
      window.alert(msg);
    }
  }

  function discordInputAtMoveBoundary(input, direction) {
    if (!input || input.tagName !== "INPUT") return true;
    if (direction > 0) {
      const len = (input.value || "").length;
      return input.selectionStart === len && input.selectionEnd === len;
    }
    return input.selectionStart === 0 && input.selectionEnd === 0;
  }

  /** Focusable Discord settings toolbar items (token · Save · Clear · View logs). */
  function getDiscordToolbarItems(wrap) {
    const container =
      wrap || document.getElementById("discord-setting");
    if (!container) return [];
    const ids = [
      "discord-token-input",
      "discord-save-token",
      "discord-clear-token",
      "view-debug-log",
    ];
    return ids
      .map((id) => document.getElementById(id))
      .filter((el) => {
        if (!el || !container.contains(el)) return false;
        if (el.hidden || el.disabled) return false;
        return el.getClientRects().length > 0 || container.contains(el);
      });
  }

  function refreshDiscordToolbarRovingTabindex(wrap, preferred) {
    const container = wrap || document.getElementById("discord-setting");
    const items = getDiscordToolbarItems(container);
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

  function ensureDiscordToolbarKbHint(wrap) {
    const container = wrap || document.getElementById("discord-setting");
    if (!container) return;
    const actions = container.querySelector(".discord-actions");
    if (!actions) return;
    let hint = actions.querySelector(".discord-toolbar-kb-hint");
    if (!hint) {
      hint = document.createElement("div");
      hint.className = "discord-toolbar-kb-hint";
      hint.setAttribute("aria-hidden", "true");
      actions.appendChild(hint);
    }
    const items = getDiscordToolbarItems(container);
    hint.hidden = items.length < 2;
    hint.textContent =
      "← → / h l · Home/End move · Enter saves from token · buttons keep activate";
  }

  /**
   * Discord settings toolbar keyboard — focus token · Save · Clear · View logs,
   * then ←→ / h l / Home/End (Monitors detail action toolbar parity).
   */
  function wireDiscordToolbarKeyboard(wrap) {
    const container = wrap || document.getElementById("discord-setting");
    if (!container) return;
    ensureDiscordToolbarKbHint(container);
    refreshDiscordToolbarRovingTabindex(container);
    if (container.dataset.discordToolbarKbWired === "1") return;
    container.dataset.discordToolbarKbWired = "1";
    if (!container.getAttribute("role")) container.setAttribute("role", "toolbar");
    if (!container.getAttribute("aria-label")) {
      container.setAttribute("aria-label", "Discord bot token");
    }
    container.addEventListener("focusin", (e) => {
      const items = getDiscordToolbarItems(container);
      if (items.includes(e.target)) {
        refreshDiscordToolbarRovingTabindex(container, e.target);
        ensureDiscordToolbarKbHint(container);
      }
    });
    container.addEventListener("keydown", (e) => {
      if (e.metaKey || e.ctrlKey || e.altKey) return;
      const items = getDiscordToolbarItems(container);
      if (!items.length) return;
      const idx = items.indexOf(document.activeElement);
      if (idx < 0) return;
      const active = items[idx];
      if (e.key === "Enter" || e.key === " ") {
        if (
          active?.id === "discord-token-input" ||
          active?.id === "discord-save-token" ||
          active?.id === "discord-clear-token" ||
          active?.id === "view-debug-log"
        ) {
          return;
        }
      }
      let next = -1;
      const forward =
        e.key === "ArrowRight" ||
        e.key === "l" ||
        e.key === "ArrowDown" ||
        e.key === "j";
      const back =
        e.key === "ArrowLeft" ||
        e.key === "h" ||
        e.key === "ArrowUp" ||
        e.key === "k";
      if (forward) {
        if (
          active?.id === "discord-token-input" &&
          !discordInputAtMoveBoundary(active, 1)
        ) {
          return;
        }
        next = Math.min(idx + 1, items.length - 1);
      } else if (back) {
        if (
          active?.id === "discord-token-input" &&
          !discordInputAtMoveBoundary(active, -1)
        ) {
          return;
        }
        next = Math.max(idx - 1, 0);
      } else if (e.key === "Home") {
        next = 0;
      } else if (e.key === "End") {
        next = items.length - 1;
      } else {
        return;
      }
      e.preventDefault();
      e.stopPropagation();
      if (next === idx) return;
      refreshDiscordToolbarRovingTabindex(container, items[next]);
      items[next].focus();
      if (
        items[next]?.id === "discord-token-input" &&
        typeof items[next].setSelectionRange === "function"
      ) {
        const len = (items[next].value || "").length;
        items[next].setSelectionRange(len, len);
      }
    });
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
    wireDiscordToolbarKeyboard();
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
