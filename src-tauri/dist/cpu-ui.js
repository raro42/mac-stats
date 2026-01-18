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

  function bootstrap() {
    const savedTheme = getSavedTheme();
    syncThemeClass(savedTheme);
    initSettingsModal();
    initThemePicker();
    initRefresh();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", bootstrap);
  } else {
    bootstrap();
  }
})();
