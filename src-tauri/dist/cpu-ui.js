// Shared UI wiring for CPU window
// - Handles settings modal open/close
// - Handles theme selection + persistence
// - Triggers refresh via window.refreshData if available
//
// Theme-specific visuals live in each theme's cpu.html/cpu.css.

(function () {
  // Helper function to get Tauri invoke (same as in cpu.js)
  function getInvoke() {
    if (typeof window.__TAURI_INVOKE__ !== 'undefined') {
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
  function getSavedTheme() {
    return localStorage.getItem("theme") || "apple";
  }

  /** First-paint colors so theme navigation does not flash the WKWebView default white. */
  const THEME_BOOT_PAINT = {
    dark: ["#000000", "dark"],
    "data-poster": ["#0b0b10", "dark"],
    neon: ["#020302", "dark"],
    futuristic: ["#07070c", "dark"],
    apple: ["#e8e8ed", "light"],
    architect: ["#dfe6f0", "light"],
    light: ["#f2f2f7", "light"],
    material: ["#ebe4d9", "light"],
    "swiss-minimalistic": ["#ffffff", "light"],
  };

  function paintThemeBoot(theme) {
    const pair = THEME_BOOT_PAINT[theme] || THEME_BOOT_PAINT.apple;
    const bg = pair[0];
    const scheme = pair[1];
    try {
      document.documentElement.style.background = bg;
      document.documentElement.style.colorScheme = scheme;
      if (document.body) {
        document.body.style.background = bg;
      }
    } catch (_) {
      /* ignore */
    }
  }

  function getThemeBasePath() {
    // When opened via src-tauri/dist/cpu.html => base is "./themes"
    // When opened via src-tauri/dist/themes/<theme>/cpu.html => base is "../"
    const parts = window.location.pathname.split("/").filter(Boolean);
    const themesIndex = parts.lastIndexOf("themes");

    if (themesIndex !== -1) {
      // .../themes/<theme>/cpu.html
      return "../";
    }

    // .../cpu.html
    return "./themes/";
  }

  function themeAssetVersion() {
    try {
      return (
        localStorage.getItem("macStatsAssetVersion") ||
        new URLSearchParams(window.location.search).get("v") ||
        String(Date.now())
      );
    } catch (_) {
      return String(Date.now());
    }
  }

  function navigateToTheme(theme) {
    const base = getThemeBasePath();
    // base ends with / for root page; for themes page it's "../"
    const v = encodeURIComponent(themeAssetVersion());
    const url = `${base}${theme}/cpu.html?v=${v}`;
    const path = window.location.pathname;
    if (path.endsWith(`${theme}/cpu.html`) || path.endsWith(`${theme}/cpu.html/`)) {
      const cur = new URLSearchParams(window.location.search).get("v");
      if (cur === themeAssetVersion()) return;
    }
    window.location.href = url;
  }

  function syncThemeClass(theme) {
    // Allows theme-specific CSS that relies on body class.
    // Not required for navigation-based theming, but harmless.
    document.body.className = `theme-${theme}`;
  }

  let settingsFocusReturn = null;

  /** Modal header: title + close (toolbar keyboard parity). */
  function getModalHeaderToolbarItems(header, titleId, closeId) {
    if (!header) return [];
    return [titleId, closeId]
      .map((id) => document.getElementById(id))
      .filter((el) => {
        if (!el || !header.contains(el)) return false;
        if (el.hidden || el.disabled) return false;
        return el.getClientRects().length > 0 || header.contains(el);
      });
  }

  function refreshModalHeaderRovingTabindex(header, titleId, closeId, preferred) {
    const items = getModalHeaderToolbarItems(header, titleId, closeId);
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

  function ensureModalHeaderKbStyles() {
    if (document.getElementById("mac-stats-modal-header-kb-styles")) return;
    const style = document.createElement("style");
    style.id = "mac-stats-modal-header-kb-styles";
    style.textContent = `
      .modal-header-kb-hint,
      .settings-header-kb-hint {
        margin: 0;
        font-size: 11px;
        opacity: 0.72;
        flex: 1 1 auto;
        text-align: center;
      }
      .settings-header h2[tabindex]:focus {
        outline: 2px solid var(--focus-ring, rgba(0, 122, 255, 0.55));
        outline-offset: 2px;
        border-radius: 4px;
      }
    `;
    document.head.appendChild(style);
  }

  function ensureModalHeaderKbHint(header, titleId, closeId, closeSelector) {
    if (!header) return;
    let hint = header.querySelector(".modal-header-kb-hint, .settings-header-kb-hint");
    if (!hint) {
      hint = document.createElement("div");
      hint.className = "modal-header-kb-hint";
      hint.setAttribute("aria-hidden", "true");
      const closeBtn =
        (closeSelector && header.querySelector(closeSelector)) ||
        document.getElementById(closeId);
      if (closeBtn) header.insertBefore(hint, closeBtn);
      else header.appendChild(hint);
    }
    const items = getModalHeaderToolbarItems(header, titleId, closeId);
    hint.hidden = items.length < 2;
    hint.textContent =
      "← → / h l · Home/End move · Enter / Space on Close closes";
  }

  /**
   * Modal header toolbar keyboard — focus title · Close, then ←→ / h l / Home/End.
   */
  function wireModalHeaderToolbarKeyboard(header, options = {}) {
    if (!header) return;
    const titleId = options.titleId;
    const closeId = options.closeId;
    if (!titleId || !closeId) return;
    const ariaLabel = options.ariaLabel || "Modal header";
    const wireKey = options.wireKey || "modalHeaderToolbarKbWired";
    ensureModalHeaderKbStyles();
    ensureModalHeaderKbHint(header, titleId, closeId, options.closeSelector);
    refreshModalHeaderRovingTabindex(header, titleId, closeId);
    if (header.dataset[wireKey] === "1") return;
    header.dataset[wireKey] = "1";
    if (!header.getAttribute("role")) header.setAttribute("role", "toolbar");
    if (!header.getAttribute("aria-label")) {
      header.setAttribute("aria-label", ariaLabel);
    }
    header.addEventListener("focusin", (e) => {
      const items = getModalHeaderToolbarItems(header, titleId, closeId);
      if (items.includes(e.target)) {
        refreshModalHeaderRovingTabindex(header, titleId, closeId, e.target);
        ensureModalHeaderKbHint(header, titleId, closeId, options.closeSelector);
      }
    });
    header.addEventListener("keydown", (e) => {
      if (e.metaKey || e.ctrlKey || e.altKey) return;
      const items = getModalHeaderToolbarItems(header, titleId, closeId);
      if (!items.length) return;
      const idx = items.indexOf(document.activeElement);
      if (idx < 0) return;
      const active = items[idx];
      if (e.key === "Enter" || e.key === " ") {
        if (active?.id === titleId || active?.id === closeId) {
          return;
        }
      }
      let next = -1;
      if (
        e.key === "ArrowRight" ||
        e.key === "l" ||
        e.key === "ArrowDown" ||
        e.key === "j"
      ) {
        next = Math.min(idx + 1, items.length - 1);
      } else if (
        e.key === "ArrowLeft" ||
        e.key === "h" ||
        e.key === "ArrowUp" ||
        e.key === "k"
      ) {
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
      refreshModalHeaderRovingTabindex(header, titleId, closeId, items[next]);
      items[next].focus();
    });
  }

  /** Settings close/header toolbar keyboard (modal header parity). */
  function wireSettingsHeaderToolbarKeyboard(header) {
    wireModalHeaderToolbarKeyboard(header, {
      titleId: "settings-title",
      closeId: "close-settings",
      ariaLabel: "Settings header",
      wireKey: "settingsHeaderToolbarKbWired",
    });
  }

  function openSettingsModal() {
    const settingsModal = document.getElementById("settings-modal");
    if (!settingsModal) return;
    settingsFocusReturn = document.activeElement;
    settingsModal.style.display = "flex";
    settingsModal.setAttribute("aria-hidden", "false");
    settingsModal.setAttribute("role", "dialog");
    settingsModal.setAttribute("aria-modal", "true");
    if (!settingsModal.getAttribute("aria-labelledby")) {
      const title =
        settingsModal.querySelector("#settings-modal-title") ||
        settingsModal.querySelector("#settings-title") ||
        settingsModal.querySelector(".settings-header h2");
      if (title) {
        if (!title.id) title.id = "settings-modal-title";
        settingsModal.setAttribute("aria-labelledby", title.id);
      }
    }
    if (window.Discord?.refreshStatus) window.Discord.refreshStatus();
    if (window.Perplexity?.refreshStatus) window.Perplexity.refreshStatus();
    const perplexitySetting = document.getElementById("perplexity-setting");
    if (
      perplexitySetting &&
      typeof window.ensurePerplexitySettingsToolbarKeyboard === "function"
    ) {
      window.ensurePerplexitySettingsToolbarKeyboard(perplexitySetting);
    }
    const settingsHeader = settingsModal.querySelector(".settings-header");
    if (settingsHeader) wireSettingsHeaderToolbarKeyboard(settingsHeader);
    const credentialsSection = settingsModal.querySelector(
      'section[aria-labelledby="settings-credentials-heading"]'
    );
    if (credentialsSection) {
      wireCredentialsSectionToolbarKeyboard(credentialsSection);
    }
    requestAnimationFrame(() => {
      const closeBtn = document.getElementById("close-settings");
      if (closeBtn) {
        refreshModalHeaderRovingTabindex(
          settingsHeader,
          "settings-title",
          "close-settings",
          closeBtn
        );
        closeBtn.focus();
      }
    });
  }

  function closeSettingsModal() {
    const settingsModal = document.getElementById("settings-modal");
    if (!settingsModal) return;
    settingsModal.style.display = "none";
    settingsModal.setAttribute("aria-hidden", "true");
    const returnEl = settingsFocusReturn;
    settingsFocusReturn = null;
    if (returnEl && typeof returnEl.focus === "function") {
      try {
        returnEl.focus();
      } catch (_) {
        /* ignore */
      }
    }
  }

  function initSettingsModal() {
    const settingsBtn = document.getElementById("settings-btn");
    const settingsModal = document.getElementById("settings-modal");
    const closeSettings = document.getElementById("close-settings");

    if (settingsBtn && settingsModal) {
      settingsBtn.addEventListener("click", () => openSettingsModal());
    }

    if (closeSettings) {
      closeSettings.addEventListener("click", () => closeSettingsModal());
    }

    if (settingsModal) {
      settingsModal.addEventListener("click", (e) => {
        if (e.target === settingsModal) closeSettingsModal();
      });
      const settingsHeader = settingsModal.querySelector(".settings-header");
      if (settingsHeader) wireSettingsHeaderToolbarKeyboard(settingsHeader);
    }

    document.addEventListener("keydown", (e) => {
      if (e.key !== "Escape") return;
      const modal = document.getElementById("settings-modal");
      if (modal && modal.style.display !== "none") {
        closeSettingsModal();
      }
    });
  }

  // One-time inject of style for theme-switch fade-out (no per-theme CSS edits)
  function ensureThemeSwitchStyle() {
    if (document.getElementById("theme-switch-style")) return;
    const style = document.createElement("style");
    style.id = "theme-switch-style";
    style.textContent =
      "body.theme-switch-fade-out { opacity: 0; transition: opacity 0.2s ease-out; }";
    document.head.appendChild(style);
  }

  function applyTheme(theme) {
    localStorage.setItem("theme", theme);
    syncThemeClass(theme);
    // Paint destination color under the fade so opacity:0 does not show white WKWebView chrome.
    paintThemeBoot(theme);
    const v = encodeURIComponent(themeAssetVersion());
    const url = `${getThemeBasePath()}${theme}/cpu.html?v=${v}`;
    if (window.location.pathname.endsWith(theme + "/cpu.html")) {
      const cur = new URLSearchParams(window.location.search).get("v");
      if (cur === themeAssetVersion()) return;
    }

    ensureThemeSwitchStyle();
    document.body.classList.add("theme-switch-fade-out");

    let navigated = false;
    const done = () => {
      if (navigated) return;
      navigated = true;
      document.body.classList.remove("theme-switch-fade-out");
      window.location.href = url;
    };
    const fallback = setTimeout(done, 250);
    document.body.addEventListener(
      "transitionend",
      function onEnd(e) {
        if (e.target !== document.body || e.propertyName !== "opacity") return;
        document.body.removeEventListener("transitionend", onEnd);
        clearTimeout(fallback);
        done();
      },
      { once: true }
    );
  }

  /** Theme buttons in Settings Appearance (visible only). */
  function getThemeListButtons(themeList) {
    if (!themeList) return [];
    return Array.from(themeList.querySelectorAll("[data-theme]")).filter((el) => {
      if (!el || el.hidden || el.disabled) return false;
      return el.getClientRects().length > 0 || el.offsetParent !== null || themeList.contains(el);
    });
  }

  function refreshThemeListRovingTabindex(themeList, preferred) {
    const buttons = getThemeListButtons(themeList);
    if (!buttons.length) return;
    const focused = buttons.find((el) => el === document.activeElement);
    const current =
      (preferred && buttons.includes(preferred) && preferred) ||
      focused ||
      buttons.find((el) => el.getAttribute("aria-current") === "true") ||
      buttons.find((el) => el.tabIndex === 0) ||
      buttons[0];
    for (const el of buttons) {
      el.tabIndex = el === current ? 0 : -1;
    }
  }

  function ensureThemeListKbStyles() {
    if (document.getElementById("mac-stats-theme-list-kb-styles")) return;
    const style = document.createElement("style");
    style.id = "mac-stats-theme-list-kb-styles";
    style.textContent = `
      .theme-list-kb-hint {
        margin: 6px 0 0;
        font-size: 11px;
        opacity: 0.72;
        grid-column: 1 / -1;
      }
    `;
    document.head.appendChild(style);
  }

  /** Theme buttons + window-frame toggle in Settings Appearance (visible only). */
  function getAppearanceSettingControls(section) {
    if (!section) return [];
    const themeList = section.querySelector("#theme-list");
    const themes = themeList ? getThemeListButtons(themeList) : [];
    const frameToggle = section.querySelector("#window-decorations-toggle");
    const frame =
      frameToggle &&
      !frameToggle.hidden &&
      !frameToggle.disabled &&
      (frameToggle.getClientRects().length > 0 || section.contains(frameToggle))
        ? frameToggle
        : null;
    return frame ? themes.concat(frame) : themes;
  }

  function refreshAppearanceSettingRovingTabindex(section, preferred) {
    const controls = getAppearanceSettingControls(section);
    if (!controls.length) return;
    const focused = controls.find((el) => el === document.activeElement);
    const current =
      (preferred && controls.includes(preferred) && preferred) ||
      focused ||
      controls.find((el) => el.getAttribute?.("aria-current") === "true") ||
      controls.find((el) => el.tabIndex === 0) ||
      controls[0];
    for (const el of controls) {
      el.tabIndex = el === current ? 0 : -1;
    }
  }

  function ensureAppearanceSettingKbStyles() {
    ensureThemeListKbStyles();
    if (document.getElementById("mac-stats-appearance-setting-kb-styles")) return;
    const style = document.createElement("style");
    style.id = "mac-stats-appearance-setting-kb-styles";
    style.textContent = `
      .appearance-setting-kb-hint {
        margin: 8px 0 0;
        font-size: 11px;
        opacity: 0.72;
      }
    `;
    document.head.appendChild(style);
  }

  /**
   * Settings Appearance toolbar keyboard — theme list + window frame toggle;
   * ←→ / h l / Home/End (theme-list / header toolbar parity).
   */
  function wireAppearanceSettingToolbarKeyboard(section) {
    if (!section) return;
    ensureAppearanceSettingKbStyles();
    let hint = section.querySelector(":scope > .appearance-setting-kb-hint");
    if (!hint) {
      hint = document.createElement("div");
      hint.className = "appearance-setting-kb-hint";
      hint.setAttribute("aria-hidden", "true");
      section.appendChild(hint);
    }
    const controls = getAppearanceSettingControls(section);
    hint.hidden = controls.length < 2;
    hint.textContent =
      "← → / h l · Home/End move · Enter / Space applies theme or toggles frame · at end crosses to Product";
    refreshAppearanceSettingRovingTabindex(section);
    if (section.dataset.appearanceSettingKbWired === "1") return;
    section.dataset.appearanceSettingKbWired = "1";
    section.setAttribute("role", "toolbar");
    if (!section.getAttribute("aria-label")) {
      section.setAttribute("aria-label", "Appearance settings");
    }
    section.addEventListener("focusin", (e) => {
      const items = getAppearanceSettingControls(section);
      if (items.includes(e.target)) {
        refreshAppearanceSettingRovingTabindex(section, e.target);
        hint.hidden = items.length < 2;
      }
    });
    section.addEventListener("keydown", (e) => {
      if (e.metaKey || e.ctrlKey || e.altKey) return;
      const items = getAppearanceSettingControls(section);
      if (!items.length) return;
      const idx = items.indexOf(document.activeElement);
      if (idx < 0) return;
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
      let next = -1;
      if (forward) {
        if (idx === items.length - 1) {
          if (tryChainSettingsSectionFocus(section, 1)) {
            e.preventDefault();
            e.stopPropagation();
          }
          return;
        }
        next = idx + 1;
      } else if (back) {
        if (idx === 0) {
          if (tryChainSettingsSectionFocus(section, -1)) {
            e.preventDefault();
            e.stopPropagation();
          }
          return;
        }
        next = idx - 1;
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
      refreshAppearanceSettingRovingTabindex(section, items[next]);
      items[next].focus();
    });
  }

  /** Theme-list-only fallback when Appearance section markup is missing. */
  function wireThemeListToolbarKeyboard(themeList) {
    if (!themeList) return;
    ensureThemeListKbStyles();
    let hint = themeList.querySelector(":scope > .theme-list-kb-hint");
    if (!hint) {
      hint = document.createElement("div");
      hint.className = "theme-list-kb-hint";
      hint.setAttribute("aria-hidden", "true");
      themeList.appendChild(hint);
    }
    hint.textContent =
      "← → / h l · Home/End move · Enter / Space applies theme";
    refreshThemeListRovingTabindex(themeList);
    if (themeList.dataset.themeListKbWired === "1") return;
    themeList.dataset.themeListKbWired = "1";
    themeList.setAttribute("role", "toolbar");
    if (!themeList.getAttribute("aria-label")) {
      themeList.setAttribute("aria-label", "Theme");
    }
    themeList.addEventListener("focusin", (e) => {
      const buttons = getThemeListButtons(themeList);
      if (buttons.includes(e.target)) refreshThemeListRovingTabindex(themeList, e.target);
    });
    themeList.addEventListener("keydown", (e) => {
      if (e.metaKey || e.ctrlKey || e.altKey) return;
      const buttons = getThemeListButtons(themeList);
      if (!buttons.length) return;
      const idx = buttons.indexOf(document.activeElement);
      if (idx < 0) return;
      let next = -1;
      if (
        e.key === "ArrowRight" ||
        e.key === "l" ||
        e.key === "ArrowDown" ||
        e.key === "j"
      ) {
        next = Math.min(idx + 1, buttons.length - 1);
      } else if (
        e.key === "ArrowLeft" ||
        e.key === "h" ||
        e.key === "ArrowUp" ||
        e.key === "k"
      ) {
        next = Math.max(idx - 1, 0);
      } else if (e.key === "Home") {
        next = 0;
      } else if (e.key === "End") {
        next = buttons.length - 1;
      } else {
        return;
      }
      e.preventDefault();
      e.stopPropagation();
      if (next === idx) return;
      refreshThemeListRovingTabindex(themeList, buttons[next]);
      buttons[next].focus();
    });
  }

  /** Product toggles + Help / Reset in Settings (theme-list toolbar parity). */
  function getProductSettingControls(wrap) {
    if (!wrap) return [];
    const ids = [
      "ai-agent-enabled-toggle",
      "menu-bar-compact-toggle",
      "cpu-window-compact-toggle",
      "settings-help-btn",
      "settings-reset-defaults-btn",
    ];
    return ids
      .map((id) => document.getElementById(id))
      .filter((el) => {
        if (!el || !wrap.contains(el)) return false;
        if (el.hidden || el.disabled) return false;
        return el.getClientRects().length > 0 || el.offsetParent !== null;
      });
  }

  function refreshProductSettingRovingTabindex(wrap, preferred) {
    const controls = getProductSettingControls(wrap);
    if (!controls.length) return;
    const focused = controls.find((el) => el === document.activeElement);
    const current =
      (preferred && controls.includes(preferred) && preferred) ||
      focused ||
      controls.find((el) => el.tabIndex === 0) ||
      controls[0];
    for (const el of controls) {
      el.tabIndex = el === current ? 0 : -1;
    }
  }

  function ensureProductSettingKbStyles() {
    if (document.getElementById("mac-stats-product-setting-kb-styles")) return;
    const style = document.createElement("style");
    style.id = "mac-stats-product-setting-kb-styles";
    style.textContent = `
      .product-setting-kb-hint {
        margin: 8px 0 0;
        font-size: 11px;
        opacity: 0.72;
      }
    `;
    document.head.appendChild(style);
  }

  /**
   * Settings Product toolbar keyboard — focus a toggle or action, then ←→ /
   * h l / Home/End (theme-list / filter-chip parity). Space toggles checkboxes;
   * Enter/Space keep Help / Reset.
   */
  function wireProductSettingToolbarKeyboard(wrap) {
    if (!wrap) return;
    ensureProductSettingKbStyles();
    let hint = wrap.querySelector(":scope > .product-setting-kb-hint");
    if (!hint) {
      hint = document.createElement("div");
      hint.className = "product-setting-kb-hint";
      hint.setAttribute("aria-hidden", "true");
      wrap.appendChild(hint);
    }
    hint.textContent =
      "← → / h l · Home/End move · Space toggles · Enter / Space on Help / Reset · at ends crosses Appearance / Credentials";
    refreshProductSettingRovingTabindex(wrap);
    if (wrap.dataset.productSettingKbWired === "1") return;
    wrap.dataset.productSettingKbWired = "1";
    wrap.setAttribute("role", "toolbar");
    if (!wrap.getAttribute("aria-label")) {
      wrap.setAttribute("aria-label", "Product settings");
    }
    wrap.addEventListener("focusin", (e) => {
      const controls = getProductSettingControls(wrap);
      if (controls.includes(e.target)) {
        refreshProductSettingRovingTabindex(wrap, e.target);
      }
    });
    wrap.addEventListener("keydown", (e) => {
      if (e.metaKey || e.ctrlKey || e.altKey) return;
      const controls = getProductSettingControls(wrap);
      if (!controls.length) return;
      const idx = controls.indexOf(document.activeElement);
      if (idx < 0) return;
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
      let next = -1;
      if (forward) {
        if (idx === controls.length - 1) {
          if (tryChainSettingsSectionFocus(wrap.closest("section") || wrap, 1)) {
            e.preventDefault();
            e.stopPropagation();
          }
          return;
        }
        next = idx + 1;
      } else if (back) {
        if (idx === 0) {
          if (tryChainSettingsSectionFocus(wrap.closest("section") || wrap, -1)) {
            e.preventDefault();
            e.stopPropagation();
          }
          return;
        }
        next = idx - 1;
      } else if (e.key === "Home") {
        next = 0;
      } else if (e.key === "End") {
        next = controls.length - 1;
      } else {
        return;
      }
      e.preventDefault();
      e.stopPropagation();
      if (next === idx) return;
      refreshProductSettingRovingTabindex(wrap, controls[next]);
      controls[next].focus();
    });
  }

  /** Appearance → Product → Credentials in Settings modal (visible sections only). */
  function getSettingsModalSections() {
    const modal = document.getElementById("settings-modal");
    if (!modal) return [];
    const selectors = [
      'section[aria-labelledby="settings-appearance-heading"]',
      'section[aria-labelledby="settings-product-heading"]',
      'section[aria-labelledby="settings-credentials-heading"]',
    ];
    return selectors
      .map((sel) => modal.querySelector(sel))
      .filter((el) => {
        if (!el) return false;
        return el.getClientRects().length > 0 || modal.contains(el);
      });
  }

  function getSettingsSectionToolbarItems(section) {
    if (!section) return [];
    const headingId = section.getAttribute("aria-labelledby");
    if (headingId === "settings-appearance-heading") {
      return getAppearanceSettingControls(section);
    }
    if (headingId === "settings-product-heading") {
      const wrap = section.querySelector("#product-setting") || section;
      return getProductSettingControls(wrap);
    }
    if (headingId === "settings-credentials-heading") {
      return getCredentialsSectionToolbarItems(section);
    }
    return [];
  }

  function focusSettingsSectionToolbarItem(section, el) {
    if (!section || !el) return;
    const headingId = section.getAttribute("aria-labelledby");
    if (headingId === "settings-appearance-heading") {
      refreshAppearanceSettingRovingTabindex(section, el);
    } else if (headingId === "settings-product-heading") {
      const wrap = section.querySelector("#product-setting") || section;
      refreshProductSettingRovingTabindex(wrap, el);
    } else if (headingId === "settings-credentials-heading") {
      refreshCredentialsSectionRovingTabindex(section, el);
    }
    el.focus();
    if (
      (el.id === "discord-token-input" || el.id === "perplexity-api-key-input") &&
      typeof el.setSelectionRange === "function"
    ) {
      const len = (el.value || "").length;
      el.setSelectionRange(len, len);
    }
  }

  /** Jump to first/last control in adjacent Settings section (+1 forward, -1 back). */
  function tryChainSettingsSectionFocus(currentSection, direction) {
    const sections = getSettingsModalSections();
    const secIdx = sections.indexOf(currentSection);
    if (secIdx < 0) return false;
    const targetIdx = secIdx + direction;
    if (targetIdx < 0 || targetIdx >= sections.length) return false;
    const targetSection = sections[targetIdx];
    const items = getSettingsSectionToolbarItems(targetSection);
    if (!items.length) return false;
    const target = direction > 0 ? items[0] : items[items.length - 1];
    focusSettingsSectionToolbarItem(targetSection, target);
    return true;
  }

  function credentialsInputAtMoveBoundary(input, direction) {
    if (!input || input.tagName !== "INPUT") return true;
    if (direction > 0) {
      const len = (input.value || "").length;
      return input.selectionStart === len && input.selectionEnd === len;
    }
    return input.selectionStart === 0 && input.selectionEnd === 0;
  }

  /** Discord token + Perplexity key controls in Settings Credentials (visible only). */
  function getCredentialsSectionToolbarItems(section) {
    if (!section) return [];
    const ids = [
      "discord-token-input",
      "discord-save-token",
      "discord-clear-token",
      "view-debug-log",
      "perplexity-api-key-input",
      "perplexity-save-key",
      "perplexity-clear-key",
    ];
    return ids
      .map((id) => document.getElementById(id))
      .filter((el) => {
        if (!el || !section.contains(el)) return false;
        if (el.hidden || el.disabled) return false;
        return el.getClientRects().length > 0 || section.contains(el);
      });
  }

  function refreshCredentialsSectionRovingTabindex(section, preferred) {
    const items = getCredentialsSectionToolbarItems(section);
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

  function ensureCredentialsSectionKbStyles() {
    if (document.getElementById("mac-stats-credentials-section-kb-styles")) return;
    const style = document.createElement("style");
    style.id = "mac-stats-credentials-section-kb-styles";
    style.textContent = `
      .credentials-section-kb-hint {
        margin: 8px 0 0;
        font-size: 11px;
        opacity: 0.72;
      }
    `;
    document.head.appendChild(style);
  }

  /**
   * Settings Credentials section toolbar keyboard — focus Discord token · Save ·
   * Clear · View logs · Perplexity key · Save · Clear, then ←→ / h l / Home/End
   * (Discord / Perplexity subsection parity).
   */
  function wireCredentialsSectionToolbarKeyboard(section) {
    if (!section) return;
    ensureCredentialsSectionKbStyles();
    let hint = section.querySelector(":scope > .credentials-section-kb-hint");
    if (!hint) {
      hint = document.createElement("div");
      hint.className = "credentials-section-kb-hint";
      hint.setAttribute("aria-hidden", "true");
      section.appendChild(hint);
    }
    const items = getCredentialsSectionToolbarItems(section);
    hint.hidden = items.length < 2;
    hint.textContent =
      "← → / h l · Home/End move · arrows at token/key start/end · at start crosses to Product";
    refreshCredentialsSectionRovingTabindex(section);
    if (section.dataset.credentialsSectionKbWired === "1") return;
    section.dataset.credentialsSectionKbWired = "1";
    section.setAttribute("role", "toolbar");
    if (!section.getAttribute("aria-label")) {
      section.setAttribute("aria-label", "Credentials settings");
    }
    section.addEventListener("focusin", (e) => {
      const controls = getCredentialsSectionToolbarItems(section);
      if (controls.includes(e.target)) {
        refreshCredentialsSectionRovingTabindex(section, e.target);
        hint.hidden = controls.length < 2;
      }
    });
    section.addEventListener(
      "keydown",
      (e) => {
        if (e.metaKey || e.ctrlKey || e.altKey) return;
        const controls = getCredentialsSectionToolbarItems(section);
        if (!controls.length) return;
        const idx = controls.indexOf(document.activeElement);
        if (idx < 0) return;
        const active = controls[idx];
        if (e.key === "Enter" || e.key === " ") {
          if (
            active?.id === "discord-token-input" ||
            active?.id === "discord-save-token" ||
            active?.id === "discord-clear-token" ||
            active?.id === "view-debug-log" ||
            active?.id === "perplexity-api-key-input" ||
            active?.id === "perplexity-save-key" ||
            active?.id === "perplexity-clear-key"
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
            (active?.id === "discord-token-input" ||
              active?.id === "perplexity-api-key-input") &&
            !credentialsInputAtMoveBoundary(active, 1)
          ) {
            return;
          }
          if (idx === controls.length - 1) return;
          next = idx + 1;
        } else if (back) {
          if (
            (active?.id === "discord-token-input" ||
              active?.id === "perplexity-api-key-input") &&
            !credentialsInputAtMoveBoundary(active, -1)
          ) {
            return;
          }
          if (idx === 0) {
            if (tryChainSettingsSectionFocus(section, -1)) {
              e.preventDefault();
              e.stopPropagation();
            }
            return;
          }
          next = idx - 1;
        } else if (e.key === "Home") {
          next = 0;
        } else if (e.key === "End") {
          next = controls.length - 1;
        } else {
          return;
        }
        e.preventDefault();
        e.stopPropagation();
        if (next === idx) return;
        refreshCredentialsSectionRovingTabindex(section, controls[next]);
        controls[next].focus();
        if (
          (controls[next]?.id === "discord-token-input" ||
            controls[next]?.id === "perplexity-api-key-input") &&
          typeof controls[next].setSelectionRange === "function"
        ) {
          const len = (controls[next].value || "").length;
          controls[next].setSelectionRange(len, len);
        }
      },
      true
    );
  }

  function initThemePicker() {
    // New: one-click list of themes
    const themeList = document.getElementById("theme-list");
    if (themeList) {
      const savedTheme = getSavedTheme();
      const buttons = themeList.querySelectorAll("[data-theme]");
      buttons.forEach((btn) => {
        const theme = btn.getAttribute("data-theme");

        if (theme === savedTheme) {
          btn.setAttribute("aria-current", "true");
        } else {
          btn.removeAttribute("aria-current");
        }

        btn.addEventListener("click", () => {
          applyTheme(theme);
        });
      });
      const appearanceSection = themeList.closest(
        'section[aria-labelledby="settings-appearance-heading"]'
      );
      if (appearanceSection) {
        wireAppearanceSettingToolbarKeyboard(appearanceSection);
      } else {
        wireThemeListToolbarKeyboard(themeList);
      }
      return;
    }

    // Fallback: legacy select
    const themeSelect = document.getElementById("theme-select");
    if (!themeSelect) return;

    const savedTheme = getSavedTheme();
    themeSelect.value = savedTheme;

    themeSelect.addEventListener("change", (e) => {
      applyTheme(e.target.value);
    });
  }

  function initRefresh() {
    const refreshBtn = document.getElementById("refresh-btn");
    if (!refreshBtn) return;

    let metricsRefreshBusy = false;

    refreshBtn.addEventListener("click", async () => {
      if (
        metricsRefreshBusy ||
        refreshBtn.disabled ||
        refreshBtn.classList.contains("is-just-saved")
      ) {
        return;
      }
      if (typeof window.refreshData !== "function") return;

      metricsRefreshBusy = true;
      const idleLabel = refreshBtn._saveFlashOriginalLabel || refreshBtn.textContent || "↻";
      refreshBtn._saveFlashOriginalLabel = idleLabel;
      refreshBtn.disabled = true;
      refreshBtn.classList.remove("is-just-saved");
      refreshBtn.classList.add("is-refreshing");
      refreshBtn.title = "Refreshing…";
      refreshBtn.setAttribute("aria-busy", "true");

      let ok = false;
      try {
        await window.refreshData();
        ok = true;
      } catch (e) {
        console.error("metrics refresh failed", e);
      } finally {
        metricsRefreshBusy = false;
        refreshBtn.classList.remove("is-refreshing");
        refreshBtn.removeAttribute("aria-busy");
        refreshBtn.disabled = false;
        if (ok) {
          if (typeof window.flashSaveButton === "function") {
            window.flashSaveButton(refreshBtn, { savedLabel: "✓", durationMs: 1200 });
          } else {
            refreshBtn.classList.add("is-just-saved");
            refreshBtn.textContent = "✓";
            setTimeout(() => {
              refreshBtn.classList.remove("is-just-saved");
              refreshBtn.textContent = idleLabel;
              refreshBtn._saveFlashOriginalLabel = null;
            }, 1200);
          }
          refreshBtn.title = "Refreshed";
          setTimeout(() => {
            if (!refreshBtn.classList.contains("is-just-saved")) {
              refreshBtn.title = "Refresh";
            }
          }, 1300);
        } else {
          refreshBtn.textContent = idleLabel;
          refreshBtn._saveFlashOriginalLabel = null;
          refreshBtn.title = "Refresh";
        }
      }
    });
  }

  /** Brief Saved flash on a Settings toggle label (save-button feedback). */
  function flashToggleLabelSaved(toggle) {
    const label = toggle?.closest?.(".setting-toggle")?.querySelector(".toggle-label");
    if (!label || label.classList.contains("is-just-saved")) return;
    const original = label._saveFlashOriginalLabel || label.textContent || "";
    label._saveFlashOriginalLabel = original;
    label.classList.add("is-just-saved");
    label.textContent = "Saved";
    clearTimeout(label._saveFlashTimer);
    label._saveFlashTimer = setTimeout(() => {
      label.classList.remove("is-just-saved");
      label.textContent = original;
    }, 1600);
  }

  function initProductToggles() {
    const aiToggle = document.getElementById("ai-agent-enabled-toggle");
    const compactToggle = document.getElementById("menu-bar-compact-toggle");
    const cpuWindowCompactToggle = document.getElementById("cpu-window-compact-toggle");
    const helpBtn = document.getElementById("settings-help-btn");
    const resetBtn = document.getElementById("settings-reset-defaults-btn");
    const helpSheet = document.getElementById("settings-help-sheet");
    if (!aiToggle && !compactToggle && !cpuWindowCompactToggle && !helpBtn && !resetBtn) return;

    (async () => {
      try {
        const invoke = getInvoke();
        if (!invoke) return;
        if (aiToggle) aiToggle.checked = !!(await invoke("get_ai_agent_enabled"));
        if (compactToggle) compactToggle.checked = !!(await invoke("get_menu_bar_compact"));
        if (cpuWindowCompactToggle) {
          cpuWindowCompactToggle.checked = !!(await invoke("get_cpu_window_compact"));
        }
        applyAiUiVisibility(aiToggle ? aiToggle.checked : true);
      } catch (e) {
        console.warn("product toggles load", e);
      }
    })();

    if (aiToggle) {
      aiToggle.addEventListener("change", async () => {
        try {
          const invoke = getInvoke();
          if (!invoke) return;
          const v = await invoke("set_ai_agent_enabled", { enabled: aiToggle.checked });
          applyAiUiVisibility(!!v);
          flashToggleLabelSaved(aiToggle);
        } catch (e) {
          console.error(e);
          alert("Could not save aiAgentEnabled: " + e);
        }
      });
      // install.sh / hand-edit of config.json can flip AI without restart
      try {
        const { listen } = window.__TAURI__.event;
        listen("ai-agent-enabled-changed", (ev) => {
          const on = !!ev.payload;
          aiToggle.checked = on;
          applyAiUiVisibility(on);
        });
      } catch (_) {
        /* event bridge optional when not in Tauri */
      }
    }
    if (compactToggle) {
      compactToggle.addEventListener("change", async () => {
        try {
          const invoke = getInvoke();
          if (!invoke) return;
          await invoke("set_menu_bar_compact", { compact: compactToggle.checked });
          flashToggleLabelSaved(compactToggle);
        } catch (e) {
          console.error(e);
        }
      });
    }
    if (cpuWindowCompactToggle) {
      cpuWindowCompactToggle.addEventListener("change", async () => {
        try {
          const invoke = getInvoke();
          if (!invoke) return;
          const on = !!cpuWindowCompactToggle.checked;
          await invoke("set_cpu_window_compact", { compact: on });
          document.body.classList.toggle("cpu-window-compact", on);
          if (on && typeof window.applyCpuWindowCompactLayout === "function") {
            window.applyCpuWindowCompactLayout(true);
          }
          flashToggleLabelSaved(cpuWindowCompactToggle);
        } catch (e) {
          console.error(e);
        }
      });
    }
    if (helpBtn && helpSheet) {
      helpBtn.addEventListener("click", () => {
        if (helpBtn.disabled || helpBtn.classList.contains("is-just-saved")) return;
        const show = helpSheet.hasAttribute("hidden");
        if (show) {
          helpSheet.textContent = [
            "Menu bar: click to open window.",
            "CLI: mac_stats | mac_stats --cpu | mac_stats -vv  (logs: ~/.mac-stats/debug.log)",
            "Config: ~/.mac-stats/config.json  (aiAgentEnabled, menuBarCompact, cpuWindowCompact)",
            "Monitor-only: leave AI off. AI path: enable toggle + ollama pull llama3.2",
            "First AI ask: “What's my CPU temp?”",
            "Docs: docs/GETTING_STARTED.md",
          ].join("\n");
          helpSheet.removeAttribute("hidden");
          const originalLabel =
            helpBtn._saveFlashOriginalLabel || helpBtn.textContent || "Help / cheat sheet";
          helpBtn._saveFlashOriginalLabel = originalLabel;
          if (typeof window.flashSaveButton === "function") {
            window.flashSaveButton(helpBtn, { savedLabel: "Opened", durationMs: 1600 });
          } else {
            helpBtn.classList.add("is-just-saved");
            helpBtn.textContent = "Opened";
            setTimeout(() => {
              helpBtn.classList.remove("is-just-saved");
              helpBtn.textContent = originalLabel;
              helpBtn._saveFlashOriginalLabel = null;
            }, 1600);
          }
        } else {
          helpSheet.setAttribute("hidden", "");
        }
      });
    }
    if (resetBtn) {
      resetBtn.addEventListener("click", async () => {
        if (resetBtn.disabled || resetBtn.classList.contains("is-just-saved")) return;
        if (!confirm("Reset config toggles to monitor defaults? (Does not delete Keychain secrets.)")) return;
        const originalLabel =
          resetBtn._saveFlashOriginalLabel || resetBtn.textContent || "Reset to monitor defaults";
        resetBtn._saveFlashOriginalLabel = originalLabel;
        resetBtn.disabled = true;
        resetBtn.classList.remove("is-just-saved");
        resetBtn.textContent = "Resetting…";
        try {
          const invoke = getInvoke();
          if (!invoke) {
            resetBtn.disabled = false;
            resetBtn.textContent = originalLabel;
            resetBtn._saveFlashOriginalLabel = null;
            return;
          }
          const msg = await invoke("reset_config_to_monitor_defaults");
          if (aiToggle) aiToggle.checked = false;
          if (compactToggle) compactToggle.checked = true;
          if (cpuWindowCompactToggle) cpuWindowCompactToggle.checked = false;
          applyAiUiVisibility(false);
          resetBtn.disabled = false;
          if (typeof window.flashSaveButton === "function") {
            window.flashSaveButton(resetBtn, { savedLabel: "Reset", durationMs: 1600 });
          } else {
            resetBtn.classList.add("is-just-saved");
            resetBtn.textContent = "Reset";
            setTimeout(() => {
              resetBtn.classList.remove("is-just-saved");
              resetBtn.textContent = originalLabel;
              resetBtn._saveFlashOriginalLabel = null;
            }, 1600);
          }
          alert(msg || "Defaults applied. Restart recommended.");
        } catch (e) {
          resetBtn.disabled = false;
          resetBtn.classList.remove("is-just-saved");
          resetBtn.textContent = originalLabel;
          resetBtn._saveFlashOriginalLabel = null;
          alert(String(e));
        }
      });
    }

    const productSetting = document.getElementById("product-setting");
    if (productSetting) wireProductSettingToolbarKeyboard(productSetting);
    const credentialsSection = document.querySelector(
      'section[aria-labelledby="settings-credentials-heading"]'
    );
    if (credentialsSection) {
      wireCredentialsSectionToolbarKeyboard(credentialsSection);
    }
  }

  function applyAiUiVisibility(enabled) {
    const ids = [
      "ollama-section",
      "agent-ops-section",
      "icon-ollama",
      "icon-agent-ops",
      "icon-discord",
      "discord-setting",
      "perplexity-setting",
      "icon-perplexity",
      "perplexity-section",
    ];
    ids.forEach((id) => {
      const el = document.getElementById(id);
      if (!el) return;
      if (id.endsWith("-section")) {
        // sections also use .collapsed; keep icon row icons dimmed when off
        el.style.display = enabled ? "" : "none";
      } else if (el.classList.contains("icon-line-item")) {
        el.style.opacity = enabled ? "" : "0.55";
        el.style.pointerEvents = enabled ? "" : "none";
        el.title = enabled ? el.title.replace(/ \(AI off\)$/, "") : (el.getAttribute("data-title-base") || el.title) + " (AI off)";
        if (!el.getAttribute("data-title-base")) el.setAttribute("data-title-base", el.title.replace(/ \(AI off\)$/, ""));
      } else {
        el.style.display = enabled ? "" : "none";
      }
    });
    if (typeof window.refreshIconLineRovingTabindex === "function") {
      window.refreshIconLineRovingTabindex();
    }
  }

  function initWindowDecorations() {
    const toggle = document.getElementById("window-decorations-toggle");
    if (!toggle) return;

      // Load saved preference from Tauri command (reads from config file)
      async function loadPreference() {
        try {
          const invoke = getInvoke();
          if (invoke) {
            const decorations = await invoke("get_window_decorations");
            toggle.checked = decorations;
            // Also sync to localStorage for consistency
            localStorage.setItem("windowDecorations", decorations.toString());
          } else {
            // Fallback to localStorage if Tauri not available
            const saved = localStorage.getItem("windowDecorations");
            const decorations = saved !== null ? saved === "true" : true;
            toggle.checked = decorations;
          }
        } catch (err) {
          console.error("Failed to load window decorations preference:", err);
          // Fallback to localStorage
          const saved = localStorage.getItem("windowDecorations");
          const decorations = saved !== null ? saved === "true" : true;
          toggle.checked = decorations;
        }
      }

    // Save preference when toggled
    toggle.addEventListener("change", async (e) => {
      const enabled = e.target.checked;
      
      // Save to localStorage for immediate UI feedback
      localStorage.setItem("windowDecorations", enabled.toString());
      
      // Save to config file via Tauri command (works without recompiling)
      try {
        const invoke = getInvoke();
        if (invoke) {
          await invoke("set_window_decorations", { decorations: enabled });
          console.log(`Window decorations preference saved: ${enabled}`);
        }
      } catch (err) {
        console.error("Failed to save window decorations preference:", err);
      }
      
      // Show a message that the change will take effect on next window open
      const label = toggle.parentElement?.querySelector('.toggle-label');
      if (label) {
        const originalText = label.textContent;
        label.textContent = "Close & reopen window to apply";
        setTimeout(() => {
          label.textContent = originalText;
        }, 3000);
      } else {
        console.warn("Could not find toggle-label element to show message");
      }
    });

    loadPreference();
  }

  function initExternalLinks() {
    const githubLink = document.getElementById("github-link");
    if (!githubLink) return;

    let githubOpenBusy = false;
    if (!githubLink.getAttribute("title")) {
      githubLink.setAttribute("title", "GitHub");
    }
    if (!githubLink.getAttribute("aria-label")) {
      githubLink.setAttribute("aria-label", "Open mac-stats on GitHub");
    }

    githubLink.addEventListener("click", async (e) => {
      e.preventDefault();
      if (
        githubOpenBusy ||
        githubLink.getAttribute("aria-disabled") === "true" ||
        githubLink.classList.contains("is-just-saved")
      ) {
        return;
      }

      const url = githubLink.href;
      const idleTitle = githubLink.getAttribute("title") || "GitHub";
      githubOpenBusy = true;
      githubLink.setAttribute("aria-disabled", "true");
      githubLink.setAttribute("aria-busy", "true");
      githubLink.classList.remove("is-just-saved");
      githubLink.setAttribute("title", "Opening…");

      let ok = false;
      const invoke = window.__TAURI__?.core?.invoke;

      try {
        // Tauri 2: shell plugin IPC
        if (invoke) {
          try {
            await invoke("plugin:shell|open", { path: url });
            ok = true;
          } catch (err) {
            console.warn("plugin:shell|open failed, trying legacy/fallback", err);
          }
        }

        if (!ok && window.__TAURI__?.shell?.open) {
          await window.__TAURI__.shell.open(url);
          ok = true;
        } else if (!ok && window.__TAURI__?.tauri?.shell?.open) {
          await window.__TAURI__.tauri.shell.open(url);
          ok = true;
        } else if (!ok) {
          window.open(url, "_blank", "noopener,noreferrer");
          ok = true;
        }
      } catch (err) {
        console.error("Failed to open GitHub URL:", err);
        try {
          window.open(url, "_blank", "noopener,noreferrer");
          ok = true;
        } catch (_) {
          ok = false;
        }
      } finally {
        githubOpenBusy = false;
        githubLink.removeAttribute("aria-busy");
        githubLink.removeAttribute("aria-disabled");
        // Keep the local SVG icon — do not replace link contents with text.
        if (ok) {
          githubLink.classList.add("is-just-saved");
          githubLink.setAttribute("title", "Opened");
          setTimeout(() => {
            githubLink.classList.remove("is-just-saved");
            githubLink.setAttribute("title", idleTitle);
          }, 1600);
        } else {
          githubLink.setAttribute("title", idleTitle);
        }
      }
    });
  }

  // Simple markdown to HTML converter for changelog format
  // Handles nested lists properly by detecting indentation
  function convertMarkdownToHtml(markdown) {
    const lines = markdown.split('\n');
    const result = [];
    let i = 0;
    
    while (i < lines.length) {
      const line = lines[i];
      
      // Headers
      if (line.match(/^### /)) {
        result.push(`<h3 class="changelog-h3">${line.replace(/^### /, '')}</h3>`);
        i++;
      } else if (line.match(/^## \[(.*?)\] - (.*)$/)) {
        const match = line.match(/^## \[(.*?)\] - (.*)$/);
        result.push(`<h2 class="changelog-h2"><span class="changelog-version">${match[1]}</span> <span class="changelog-date">${match[2]}</span></h2>`);
        i++;
      } else if (line.match(/^## /)) {
        result.push(`<h2 class="changelog-h2">${line.replace(/^## /, '')}</h2>`);
        i++;
      } else if (line.match(/^- /)) {
        // Process list items (including nested)
        const listResult = processListItems(lines, i);
        result.push(listResult.html);
        i = listResult.nextIndex;
      } else if (line.trim() === '') {
        // Empty line - add paragraph break if needed
        if (result.length > 0 && !result[result.length - 1].endsWith('</p>') && 
            !result[result.length - 1].endsWith('</ul>') && 
            !result[result.length - 1].endsWith('</h2>') && 
            !result[result.length - 1].endsWith('</h3>')) {
          result.push('</p><p class="changelog-paragraph">');
        }
        i++;
      } else {
        // Regular paragraph text
        if (result.length === 0 || result[result.length - 1].endsWith('</h2>') || 
            result[result.length - 1].endsWith('</h3>') || 
            result[result.length - 1].endsWith('</ul>')) {
          result.push('<p class="changelog-paragraph">');
        }
        let text = line;
        // Process inline formatting
        text = text.replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>');
        text = text.replace(/`([^`]+)`/g, '<code class="changelog-code">$1</code>');
        result.push(text);
        i++;
      }
    }
    
    let html = result.join('\n');
    
    // Close any open paragraphs
    html = html.replace(/([^>])(\n|$)/g, '$1<br>$2');
    html = html.replace(/<br>\n/g, '\n');
    
    // Wrap unclosed paragraphs
    if (!html.includes('<p class="changelog-paragraph">') && 
        !html.match(/^<h[23]|^<ul/)) {
      html = '<p class="changelog-paragraph">' + html + '</p>';
    }
    
    // Clean up empty paragraphs and fix structure
    html = html.replace(/<p class="changelog-paragraph"><\/p>/g, '');
    html = html.replace(/<p class="changelog-paragraph">(<h[23])/g, '$1');
    html = html.replace(/(<\/h[23]>)<p class="changelog-paragraph">/g, '$1');
    html = html.replace(/<p class="changelog-paragraph">(<ul)/g, '$1');
    html = html.replace(/(<\/ul>)<p class="changelog-paragraph">/g, '$1');
    html = html.replace(/(<\/ul>)\n*(<ul)/g, '$1$2');
    
    // Close any unclosed paragraphs at the end
    if (html.includes('<p class="changelog-paragraph">') && !html.endsWith('</p>')) {
      html += '</p>';
    }
    
    return html;
  }
  
  // Process list items with proper nesting detection
  function processListItems(lines, startIndex) {
    const items = [];
    let i = startIndex;
    let currentLevel = 0;
    
    // Find all consecutive list items
    while (i < lines.length) {
      const line = lines[i];
      const trimmed = line.trim();
      
      if (trimmed === '' && items.length > 0) {
        // Empty line after list items - check if next line is also a list item
        if (i + 1 < lines.length && lines[i + 1].trim().match(/^- /)) {
          i++;
          continue;
        } else {
          break;
        }
      }
      
      const listMatch = line.match(/^(\s*)- (.*)$/);
      if (listMatch) {
        const indent = listMatch[1].length;
        const content = listMatch[2];
        items.push({ indent, content, originalLine: line });
        i++;
      } else if (trimmed === '' && items.length === 0) {
        i++;
      } else {
        break;
      }
    }
    
    if (items.length === 0) {
      return { html: '', nextIndex: startIndex + 1 };
    }
    
    // Build nested HTML structure
    const html = buildNestedList(items, 0, 0);
    return { html, nextIndex: i };
  }
  
  // Build nested list HTML from items array
  function buildNestedList(items, startIdx, baseIndent) {
    if (startIdx >= items.length) return '';
    
    let html = '<ul class="changelog-list">';
    let i = startIdx;
    
    while (i < items.length) {
      const item = items[i];
      const indent = item.indent;
      
      // If this item is at the same or higher level, we're done with this list
      if (indent < baseIndent) {
        break;
      }
      
      // If this item is nested deeper, process it as a nested list
      if (indent > baseIndent) {
        const nested = buildNestedList(items, i, indent);
        // Find where nested list ends
        let nestedEnd = i;
        while (nestedEnd < items.length && items[nestedEnd].indent >= indent) {
          nestedEnd++;
        }
        // Add nested list to previous item if it exists
        if (html.endsWith('</li>')) {
          html = html.slice(0, -5) + nested + '</li>';
        } else {
          html += nested;
        }
        i = nestedEnd;
        continue;
      }
      
      // Process inline formatting
      let content = item.content;
      content = content.replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>');
      content = content.replace(/`([^`]+)`/g, '<code class="changelog-code">$1</code>');
      
      html += `<li class="changelog-item">${content}`;
      
      // Check if next item is nested
      if (i + 1 < items.length && items[i + 1].indent > indent) {
        // Process nested items
        const nested = buildNestedList(items, i + 1, items[i + 1].indent);
        html += nested;
        // Skip nested items
        while (i + 1 < items.length && items[i + 1].indent > indent) {
          i++;
        }
      }
      
      html += '</li>';
      i++;
    }
    
    html += '</ul>';
    return html;
  }

  // Make loadChangelog function accessible for version click handlers
  // This needs to be defined before injectAppVersion and initChangelogModal
  function loadChangelogForModal(changelogBody, changelogModal) {
    changelogBody.innerHTML = '<div class="changelog-loading">Loading changelog...</div>';
    
    (async () => {
      try {
        const invoke = getInvoke();
        if (!invoke) {
          console.error("Tauri invoke not available. window.__TAURI__:", window.__TAURI__);
          changelogBody.innerHTML = '<div class="changelog-error">Tauri API not available. Please ensure the app is running in Tauri.</div>';
          return;
        }

        console.log("Calling get_changelog Tauri command...");
        const changelogText = await invoke("get_changelog");
        console.log("Changelog received, length:", changelogText?.length || 0);
        
        if (!changelogText || changelogText.trim().length === 0) {
          changelogBody.innerHTML = '<div class="changelog-error">Changelog is empty. Please rebuild the app to include the changelog.</div>';
          return;
        }
        
        // Convert markdown to HTML (simple conversion for changelog format)
        const html = convertMarkdownToHtml(changelogText);
        changelogBody.innerHTML = html;
      } catch (error) {
        console.error("Failed to load changelog:", error);
        const errorMessage = error?.toString() || String(error) || "Unknown error";
        changelogBody.innerHTML = `<div class="changelog-error">Failed to load changelog:<br><br>${errorMessage}<br><br>Please ensure the app has been rebuilt after adding the changelog feature.</div>`;
      }
    })();
  }

  async function injectAppVersion() {
    // OPTIMIZATION Phase 2: Cache app version in localStorage
    // Fetch app version from Rust backend and inject into all version elements
    try {
      let version = localStorage.getItem('appVersion');

      // If not cached, fetch from backend
      if (!version) {
        const invoke = getInvoke();
        if (!invoke) {
          console.warn("Tauri invoke not available, skipping version injection");
          return;
        }

        version = await invoke("get_app_version");

        // Cache for future loads
        try {
          localStorage.setItem('appVersion', version);
        } catch (e) {
          console.warn("Failed to cache version:", e);
        }
      }

      // Update all version elements (theme name varies per theme)
      // .theme-version, .arch-version, etc.
      const versionElements = document.querySelectorAll(
        "[class*='version'], .theme-version, .arch-version"
      );

      versionElements.forEach((el) => {
        const themeName = el.textContent.split(" v")[0].trim();
        if (themeName) {
          el.textContent = `${themeName} v${version}`;
        } else {
          el.textContent = `v${version}`;
        }
        
        // Make version clickable
        el.style.cursor = "pointer";
        el.setAttribute("title", "Click to view changelog");
        el.classList.add("version-clickable");
        
        // Ensure click handler is attached (in case initChangelogModal ran before this)
        if (!el.dataset.changelogHandler) {
          el.dataset.changelogHandler = "true";
          const changelogModal = document.getElementById("changelog-modal");
          const changelogBody = document.getElementById("changelog-body");
          
          if (changelogModal && changelogBody) {
            el.addEventListener("click", (e) => {
              e.preventDefault();
              e.stopPropagation();
              if (el.classList.contains("is-just-saved")) return;
              console.log("Version clicked (from injectAppVersion), opening changelog modal");
              openChangelogModal(changelogModal, changelogBody, el);
            });
          }
        }
      });

      console.log(`App version injected: v${version}`);
    } catch (err) {
      console.error("Failed to fetch app version:", err);
    }
  }


  let changelogFocusReturn = null;

  /** Version label → changelog: Opened flash + block double open while flashing. */
  function flashChangelogOpened(triggerEl) {
    if (!triggerEl) return;
    const idleLabel =
      triggerEl._saveFlashOriginalLabel || triggerEl.textContent || "v?";
    triggerEl._saveFlashOriginalLabel = idleLabel;
    if (typeof window.flashSaveButton === "function") {
      window.flashSaveButton(triggerEl, { savedLabel: "Opened", durationMs: 1600 });
    } else {
      triggerEl.classList.add("is-just-saved");
      triggerEl.textContent = "Opened";
      setTimeout(() => {
        triggerEl.classList.remove("is-just-saved");
        triggerEl.textContent = idleLabel;
        triggerEl._saveFlashOriginalLabel = null;
      }, 1600);
    }
  }

  function openChangelogModal(changelogModal, changelogBody, triggerEl) {
    if (!changelogModal || !changelogBody) return;
    if (triggerEl && triggerEl.classList.contains("is-just-saved")) return;
    changelogFocusReturn = document.activeElement;
    changelogModal.style.display = "flex";
    changelogModal.setAttribute("aria-hidden", "false");
    changelogModal.setAttribute("role", "dialog");
    changelogModal.setAttribute("aria-modal", "true");
    const title =
      changelogModal.querySelector("#changelog-modal-title") ||
      changelogModal.querySelector(".settings-header h2");
    if (title) {
      if (!title.id) title.id = "changelog-modal-title";
      changelogModal.setAttribute("aria-labelledby", title.id);
    }
    const changelogHeader = changelogModal.querySelector(".settings-header");
    if (changelogHeader) wireChangelogHeaderToolbarKeyboard(changelogHeader);
    loadChangelogForModal(changelogBody, changelogModal);
    if (triggerEl) flashChangelogOpened(triggerEl);
    requestAnimationFrame(() => {
      const closeBtn = document.getElementById("close-changelog");
      if (closeBtn && changelogHeader) {
        refreshModalHeaderRovingTabindex(
          changelogHeader,
          "changelog-modal-title",
          "close-changelog",
          closeBtn
        );
        closeBtn.focus();
      } else {
        closeBtn?.focus();
      }
    });
  }

  function closeChangelogModal(changelogModal) {
    if (!changelogModal) return;
    changelogModal.style.display = "none";
    changelogModal.setAttribute("aria-hidden", "true");
    const returnEl = changelogFocusReturn;
    changelogFocusReturn = null;
    if (returnEl && typeof returnEl.focus === "function") {
      try {
        returnEl.focus();
      } catch (_) {
        /* ignore */
      }
    }
  }

  /** Changelog modal header toolbar keyboard (Settings header parity). */
  function wireChangelogHeaderToolbarKeyboard(header) {
    wireModalHeaderToolbarKeyboard(header, {
      titleId: "changelog-modal-title",
      closeId: "close-changelog",
      ariaLabel: "Changelog header",
      wireKey: "changelogHeaderToolbarKbWired",
    });
  }

  function initChangelogModal() {
    const changelogModal = document.getElementById("changelog-modal");
    const closeChangelog = document.getElementById("close-changelog");
    const changelogBody = document.getElementById("changelog-body");

    if (!changelogModal || !changelogBody) {
      console.warn("Changelog modal elements not found");
      return;
    }

    const changelogHeader = changelogModal.querySelector(".settings-header");
    if (changelogHeader) wireChangelogHeaderToolbarKeyboard(changelogHeader);
    
    // Get version elements - try multiple selectors to catch all cases
    const versionElements = document.querySelectorAll(
      ".app-version, .theme-version, .arch-version, [class*='version']"
    );
    
    console.log(`Found ${versionElements.length} version elements for changelog`);

    // Open modal when version is clicked
    versionElements.forEach((el) => {
      if (el.dataset.changelogHandler) return;
      el.dataset.changelogHandler = "true";
      el.style.cursor = "pointer";
      el.setAttribute("title", "Click to view changelog");
      el.classList.add("version-clickable");
      console.log("Adding click handler to version element:", el.className, el.textContent);
      el.addEventListener("click", (e) => {
        e.preventDefault();
        e.stopPropagation();
        if (el.classList.contains("is-just-saved")) return;
        console.log("Version clicked, opening changelog modal");
        openChangelogModal(changelogModal, changelogBody, el);
      });
    });
    
    // Also set up click handler on version elements that might be added later
    // This ensures version elements added by injectAppVersion() are also clickable
    const observer = new MutationObserver((mutations) => {
      const newVersionElements = document.querySelectorAll(
        ".app-version, .theme-version, .arch-version, [class*='version']"
      );
      newVersionElements.forEach((el) => {
        if (!el.dataset.changelogHandler) {
          el.dataset.changelogHandler = "true";
          el.style.cursor = "pointer";
          el.setAttribute("title", "Click to view changelog");
          el.addEventListener("click", (e) => {
            e.preventDefault();
            e.stopPropagation();
            if (el.classList.contains("is-just-saved")) return;
            console.log("Version clicked (from observer), opening changelog modal");
            openChangelogModal(changelogModal, changelogBody, el);
          });
        }
      });
    });
    
    // Observe the document body for new version elements
    observer.observe(document.body, {
      childList: true,
      subtree: true
    });

    // Close modal handlers
    if (closeChangelog) {
      closeChangelog.addEventListener("click", () => {
        closeChangelogModal(changelogModal);
      });
    }

    changelogModal.addEventListener("click", (e) => {
      if (e.target === changelogModal) {
        closeChangelogModal(changelogModal);
      }
    });

    // ESC key to close
    document.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && changelogModal.style.display !== "none") {
        closeChangelogModal(changelogModal);
      }
    });
  }

  function bootstrap() {
    const savedTheme = getSavedTheme();
    paintThemeBoot(savedTheme);
    syncThemeClass(savedTheme);
    initSettingsModal();
    initThemePicker();
    initRefresh();
    initExternalLinks();
    initWindowDecorations();
    initProductToggles();
    // Initialize changelog modal first, then inject version (so version elements are ready)
    initChangelogModal();
    injectAppVersion();
  }

  window.wireModalHeaderToolbarKeyboard = wireModalHeaderToolbarKeyboard;

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", bootstrap);
  } else {
    bootstrap();
  }
})();
