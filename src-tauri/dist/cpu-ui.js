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
    return null;
  }
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
        if (window.Discord?.refreshStatus) window.Discord.refreshStatus();
        if (window.Perplexity?.refreshStatus) window.Perplexity.refreshStatus();
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
            // Get the loadChangelog function from the closure
            // We'll need to access it via a global or re-implement the click handler
            el.addEventListener("click", (e) => {
              e.preventDefault();
              e.stopPropagation();
              console.log("Version clicked (from injectAppVersion), opening changelog modal");
              
              // Trigger changelog modal opening
              changelogModal.style.display = "flex";
              
              // Load changelog
              loadChangelogForModal(changelogBody, changelogModal);
            });
          }
        }
      });

      console.log(`App version injected: v${version}`);
    } catch (err) {
      console.error("Failed to fetch app version:", err);
    }
  }


  function initChangelogModal() {
    const changelogModal = document.getElementById("changelog-modal");
    const closeChangelog = document.getElementById("close-changelog");
    const changelogBody = document.getElementById("changelog-body");

    if (!changelogModal || !changelogBody) {
      console.warn("Changelog modal elements not found");
      return;
    }
    
    // Get version elements - try multiple selectors to catch all cases
    const versionElements = document.querySelectorAll(
      ".app-version, .theme-version, .arch-version, [class*='version']"
    );
    
    console.log(`Found ${versionElements.length} version elements for changelog`);

    // Function to load and display changelog (uses the global function)
    function loadChangelog() {
      loadChangelogForModal(changelogBody, changelogModal);
    }

    // Open modal when version is clicked
    versionElements.forEach((el) => {
      console.log("Adding click handler to version element:", el.className, el.textContent);
      el.addEventListener("click", (e) => {
        e.preventDefault();
        e.stopPropagation();
        console.log("Version clicked, opening changelog modal");
        changelogModal.style.display = "flex";
        loadChangelogForModal(changelogBody, changelogModal);
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
            console.log("Version clicked (from observer), opening changelog modal");
            changelogModal.style.display = "flex";
            loadChangelogForModal(changelogBody, changelogModal);
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
        changelogModal.style.display = "none";
      });
    }

    changelogModal.addEventListener("click", (e) => {
      if (e.target === changelogModal) {
        changelogModal.style.display = "none";
      }
    });

    // ESC key to close
    document.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && changelogModal.style.display !== "none") {
        changelogModal.style.display = "none";
      }
    });
  }

  function bootstrap() {
    const savedTheme = getSavedTheme();
    syncThemeClass(savedTheme);
    initSettingsModal();
    initThemePicker();
    initRefresh();
    initExternalLinks();
    initWindowDecorations();
    // Initialize changelog modal first, then inject version (so version elements are ready)
    initChangelogModal();
    injectAppVersion();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", bootstrap);
  } else {
    bootstrap();
  }
})();
