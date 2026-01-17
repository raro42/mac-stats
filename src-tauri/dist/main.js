const { invoke } = window.__TAURI__.core;

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
  // Update immediately
  updateStats();
  
  // Update every second
  setInterval(updateStats, 1000);
});
