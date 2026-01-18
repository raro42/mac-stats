// Shared UI wiring for CPU window
// - Handles settings modal open/close
// - Handles theme selection + persistence
// - Triggers refresh via window.refreshData if available
//
// Theme-specific visuals live in each theme's cpu.html/cpu.css.

(function () {
  function getSavedTheme() {
    return localStorage.getItem("theme") || "apple";
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

  function navigateToTheme(theme) {
    const base = getThemeBasePath();
    // base ends with / for root page; for themes page it's "../"
    const url = `${base}${theme}/cpu.html`;
    if (window.location.pathname.endsWith(url)) return;
    window.location.href = url;
  }

  function syncThemeClass(theme) {
    // Allows theme-specific CSS that relies on body class.
    // Not required for navigation-based theming, but harmless.
    document.body.className = `theme-${theme}`;
  }

  function initSettingsModal() {
    const settingsBtn = document.getElementById("settings-btn");
    const settingsModal = document.getElementById("settings-modal");
    const closeSettings = document.getElementById("close-settings");

    if (settingsBtn && settingsModal) {
      settingsBtn.addEventListener("click", () => {
        settingsModal.style.display = "flex";
      });
    }

    if (closeSettings && settingsModal) {
      closeSettings.addEventListener("click", () => {
        settingsModal.style.display = "none";
      });
    }

    if (settingsModal) {
      settingsModal.addEventListener("click", (e) => {
        if (e.target === settingsModal) {
          settingsModal.style.display = "none";
        }
      });
    }
  }

  function applyTheme(theme) {
    localStorage.setItem("theme", theme);
    syncThemeClass(theme);
    navigateToTheme(theme);
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

    refreshBtn.addEventListener("click", () => {
      if (window.refreshData) {
        window.refreshData();
      }
    });
  }

  function initWindowDecorations() {
    const toggle = document.getElementById("window-decorations-toggle");
    if (!toggle) return;

    // Load saved preference from Tauri command (reads from config file)
    async function loadPreference() {
      try {
        if (window.__TAURI__?.invoke) {
          const decorations = await window.__TAURI__.invoke("get_window_decorations");
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
        if (window.__TAURI__?.invoke) {
          await window.__TAURI__.invoke("set_window_decorations", { decorations: enabled });
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

    githubLink.addEventListener("click", (e) => {
      e.preventDefault();
      const url = githubLink.href;

      // Try to use Tauri shell API if available (Tauri v1)
      if (window.__TAURI__?.shell?.open) {
        window.__TAURI__.shell.open(url).catch((err) => {
          console.error("Failed to open URL with Tauri shell:", err);
          // Fallback to window.open
          window.open(url, "_blank", "noopener,noreferrer");
        });
      } else if (window.__TAURI__?.tauri?.shell?.open) {
        window.__TAURI__.tauri.shell.open(url).catch((err) => {
          console.error("Failed to open URL with Tauri shell:", err);
          window.open(url, "_blank", "noopener,noreferrer");
        });
      } else {
        // Fallback to window.open if Tauri API is not available
        window.open(url, "_blank", "noopener,noreferrer");
      }
    });
  }

  async function injectAppVersion() {
    // Fetch app version from Rust backend and inject into all version elements
    try {
      if (!window.__TAURI__?.invoke) {
        console.warn("Tauri invoke not available, skipping version injection");
        return;
      }

      const version = await window.__TAURI__.invoke("get_app_version");

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
      });

      console.log(`App version injected: v${version}`);
    } catch (err) {
      console.error("Failed to fetch app version:", err);
    }
  }

  function bootstrap() {
    const savedTheme = getSavedTheme();
    syncThemeClass(savedTheme);
    initSettingsModal();
    initThemePicker();
    initRefresh();
    initExternalLinks();
    initWindowDecorations();
    injectAppVersion();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", bootstrap);
  } else {
    bootstrap();
  }
})();
