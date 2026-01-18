const { invoke } = window.__TAURI__.core;

// Fetch app version once at startup (no polling for CPU efficiency)
let appVersion = null;

async function fetchAppVersion() {
  if (appVersion !== null) {
    return appVersion; // Already fetched, return cached value
  }
  
  try {
    appVersion = await invoke("get_app_version");
    // Set version in all footer elements
    const versionElements = document.querySelectorAll('.app-version, .theme-version, .arch-version');
    versionElements.forEach(el => {
      const themeName = el.textContent.split(' ')[0]; // Extract theme name if present
      if (themeName && el.textContent.includes('v')) {
        el.textContent = el.textContent.replace(/v[\d.]+/, `v${appVersion}`);
      } else {
        el.textContent = `v${appVersion}`;
      }
    });
    return appVersion;
  } catch (error) {
    console.error("Error fetching app version:", error);
    appVersion = "unknown";
    return appVersion;
  }
}

async function updateStats() {
  try {
    const metrics = await invoke("get_metrics");
    
    document.getElementById("cpu-value").textContent = `${Math.round(metrics.cpu)}%`;
    document.getElementById("gpu-value").textContent = `${Math.round(metrics.gpu)}%`;
    document.getElementById("ram-value").textContent = `${Math.round(metrics.ram)}%`;
    document.getElementById("disk-value").textContent = `${Math.round(metrics.disk)}%`;
  } catch (error) {
    console.error("Error fetching metrics:", error);
  }
}

window.addEventListener("DOMContentLoaded", () => {
  // Fetch version once at startup (no polling)
  fetchAppVersion();
  
  // Update immediately
  updateStats();
  
  // Update every second
  setInterval(updateStats, 1000);
});
