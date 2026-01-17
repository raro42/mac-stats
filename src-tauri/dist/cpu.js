// Try multiple ways to get invoke function for Tauri v1
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

function formatUptime(seconds) {
  const hours = Math.floor(seconds / 3600);
  const days = Math.floor(hours / 24);
  if (days > 0) {
    return `${days}d ${hours % 24}h`;
  }
  return `${hours}h`;
}

// Store previous values for smooth transitions
let previousValues = {
  temperature: 0,
  usage: 0,
  frequency: 0,
  cpuPower: 0,
  gpuPower: 0,
  load1: 0,
  load5: 0,
  load15: 0
};

// SVG Ring Gauge Animation
const ringAnimations = new Map();
const CIRCUMFERENCE = 2 * Math.PI * 42; // radius = 42

function updateRingGauge(ringId, percent, key) {
  const clamped = Math.max(0, Math.min(100, percent));
  const progressEl = document.getElementById(ringId);
  if (!progressEl) return;
  
  const targetOffset = CIRCUMFERENCE - (clamped / 100) * CIRCUMFERENCE;
  
  // Simplified: update directly without animation to reduce CPU usage
  // Only animate if the change is significant (> 5%)
  if (!ringAnimations.has(key)) {
    ringAnimations.set(key, { current: CIRCUMFERENCE, target: targetOffset });
  }
  
  const anim = ringAnimations.get(key);
  const diff = Math.abs(anim.current - targetOffset);
  
  // If change is small, update directly; otherwise animate
  if (diff < (CIRCUMFERENCE * 0.05)) {
    anim.current = targetOffset;
    progressEl.style.strokeDashoffset = anim.current;
    ringAnimations.delete(key);
    return;
  }
  
  anim.target = targetOffset;
  
  if (anim.frameId) {
    cancelAnimationFrame(anim.frameId);
  }
  
  function animate() {
    const diff = anim.target - anim.current;
    if (Math.abs(diff) < 0.5) {
      anim.current = anim.target;
      progressEl.style.strokeDashoffset = anim.current;
      ringAnimations.delete(key);
      return;
    }
    
    // Faster animation (0.25 instead of 0.15) to complete sooner
    anim.current += diff * 0.25;
    progressEl.style.strokeDashoffset = anim.current;
    anim.frameId = requestAnimationFrame(animate);
  }
  
  animate();
}

// Simple value update (no tweening to save CPU)
function updateValue(element, newValue, previousValue, formatter) {
  const formatted = formatter(newValue);
  if (element.textContent !== formatted) {
    element.textContent = formatted;
    return true; // Changed
  }
  return false; // No change
}

// Update chip info from data
function updateChipInfo(chipInfo) {
  const chipInfoEl = document.getElementById('chip-info');
  if (chipInfoEl && chipInfo) {
    chipInfoEl.textContent = chipInfo;
  }
}

let refreshInterval = null;
let invoke = null;
let lastProcessUpdate = 0;

// Make refresh available globally for refresh button
window.refreshData = refresh;

async function refresh() {
  if (!invoke) {
    invoke = getInvoke();
    if (!invoke) {
      console.error("Cannot refresh: Tauri invoke not available");
      return;
    }
  }
  
  try {
    const data = await invoke("get_cpu_details");
    
    // Update chip info
    updateChipInfo(data.chip_info);
    
    // Update temperature
    const tempEl = document.getElementById("temperature-value");
    const tempHint = document.getElementById("temperature-hint");
    const tempSubtext = document.getElementById("temperature-subtext");
    const newTemp = Math.round(data.temperature);
    
    if (!data.can_read_temperature) {
      if (tempEl.textContent !== "—") {
        tempEl.textContent = "—";
        tempSubtext.textContent = "—";
        tempHint.style.display = "block";
      }
    } else {
      tempHint.style.display = "none";
      // Show temperature even if it's 0.0 (might be unsupported Mac model)
      // But show "—" if temperature is exactly 0.0 and we've been trying for a while
      if (newTemp === 0 && data.temperature === 0.0) {
        // Temperature is 0.0 - might be unsupported Mac model
        // Still show it as "0°" to indicate we're trying to read it
        const formatted = "0°";
        if (tempEl.textContent !== formatted) {
          tempEl.textContent = formatted;
          previousValues.temperature = 0;
        }
        if (tempSubtext.textContent !== "SMC: No data") {
          tempSubtext.textContent = "SMC: No data";
        }
      } else {
        const formatted = `${newTemp}°`;
        if (tempEl.textContent !== formatted) {
          tempEl.textContent = formatted;
          previousValues.temperature = newTemp;
        }
        // Thermal state subtext (only update if changed)
        let thermalText = "Thermal: Nominal";
        if (data.temperature >= 85) {
          thermalText = "Thermal: Critical";
        } else if (data.temperature >= 70) {
          thermalText = "Thermal: Serious";
        } else if (data.temperature >= 50) {
          thermalText = "Thermal: Fair";
        }
        if (tempSubtext.textContent !== thermalText) {
          tempSubtext.textContent = thermalText;
        }
      }
    }
    updateRingGauge("temperature-ring-progress", Math.min(100, data.temperature), 'temperature');

    // Update CPU usage
    const cpuUsageEl = document.getElementById("cpu-usage-value");
    const cpuUsageSubtext = document.getElementById("cpu-usage-subtext");
    const newUsage = Math.round(data.usage);
    const formatted = `${newUsage}%`;
    
    if (cpuUsageEl.textContent !== formatted) {
      cpuUsageEl.textContent = formatted;
      previousValues.usage = newUsage;
    }
    if (cpuUsageSubtext.textContent !== "Avg last 10s") {
      cpuUsageSubtext.textContent = "Avg last 10s";
    }
    updateRingGauge("cpu-usage-ring-progress", data.usage, 'usage');

    // Update frequency
    const freqEl = document.getElementById("frequency-value");
    const freqHint = document.getElementById("frequency-hint");
    const freqSubtext = document.getElementById("frequency-subtext");
    
    if (!data.can_read_frequency) {
      if (freqEl.textContent !== "—") {
        freqEl.textContent = "—";
        freqSubtext.textContent = "—";
        freqHint.style.display = "block";
      }
    } else {
      freqHint.style.display = "none";
      const formatted = data.frequency.toFixed(1);
      if (freqEl.textContent !== formatted) {
        freqEl.textContent = formatted;
        previousValues.frequency = data.frequency;
      }
      if (freqSubtext.textContent !== "GHz") {
        freqSubtext.textContent = "GHz";
      }
    }
    updateRingGauge("frequency-ring-progress", Math.min(100, (data.frequency / 5.0) * 100), 'frequency');

    // Update uptime
    document.getElementById("uptime-value").textContent = formatUptime(data.uptime_secs);

    // Update load averages (simple updates, no tweening)
    const load1El = document.getElementById("load-1");
    const newLoad1 = data.load_1.toFixed(2);
    if (load1El.textContent !== newLoad1) {
      load1El.textContent = newLoad1;
      previousValues.load1 = data.load_1;
    }

    const load5El = document.getElementById("load-5");
    const newLoad5 = data.load_5.toFixed(2);
    if (load5El.textContent !== newLoad5) {
      load5El.textContent = newLoad5;
      previousValues.load5 = data.load_5;
    }

    const load15El = document.getElementById("load-15");
    const newLoad15 = data.load_15.toFixed(2);
    if (load15El.textContent !== newLoad15) {
      load15El.textContent = newLoad15;
      previousValues.load15 = data.load_15;
    }

    // Update power consumption (simple updates)
    const cpuPowerEl = document.getElementById("cpu-power");
    const cpuPowerHint = document.getElementById("cpu-power-hint");
    if (!data.can_read_cpu_power) {
      if (cpuPowerEl.textContent !== "0.0 W") {
        cpuPowerEl.textContent = "0.0 W";
        cpuPowerHint.style.display = "block";
      }
    } else {
      cpuPowerHint.style.display = "none";
      const formatted = `${data.cpu_power.toFixed(2)} W`;
      if (cpuPowerEl.textContent !== formatted) {
        cpuPowerEl.textContent = formatted;
        previousValues.cpuPower = data.cpu_power;
      }
    }

    const gpuPowerEl = document.getElementById("gpu-power");
    const gpuPowerHint = document.getElementById("gpu-power-hint");
    if (!data.can_read_gpu_power) {
      if (gpuPowerEl.textContent !== "0.0 W") {
        gpuPowerEl.textContent = "0.0 W";
        gpuPowerHint.style.display = "block";
      }
    } else {
      gpuPowerHint.style.display = "none";
      const formatted = `${data.gpu_power.toFixed(2)} W`;
      if (gpuPowerEl.textContent !== formatted) {
        gpuPowerEl.textContent = formatted;
        previousValues.gpuPower = data.gpu_power;
      }
    }

    // Update process list only every 15 seconds to reduce CPU usage
    // Only update if we have processes (window must be visible for processes to be collected)
    const now = Date.now();
    if (now - lastProcessUpdate >= 15000 || lastProcessUpdate === 0) {
      lastProcessUpdate = now;
      
      const list = document.getElementById("process-list");
      list.innerHTML = "";
      
      if (data.top_processes && data.top_processes.length > 0) {
        data.top_processes.slice(0, 8).forEach((proc) => {
        const row = document.createElement("div");
        row.className = "process-row";
        
        const name = document.createElement("div");
        name.className = "process-name";
        name.textContent = proc.name;
        
        const usage = document.createElement("div");
        usage.className = "process-usage";
        
        const bar = document.createElement("div");
        bar.className = "process-bar";
        
        const barFill = document.createElement("div");
        barFill.className = "process-bar-fill";
        barFill.style.width = `${Math.min(100, proc.cpu)}%`;
        
        const percent = document.createElement("div");
        percent.className = "process-percent";
        percent.textContent = `${proc.cpu.toFixed(1)}%`;
        
        bar.appendChild(barFill);
        usage.appendChild(bar);
        usage.appendChild(percent);
        
        row.appendChild(name);
        row.appendChild(usage);
        list.appendChild(row);
        });
      } else {
        // No processes available (window might be closed, saving CPU)
        const emptyMsg = document.createElement("div");
        emptyMsg.className = "process-empty";
        emptyMsg.textContent = "No process data available";
        emptyMsg.style.textAlign = "center";
        emptyMsg.style.padding = "1rem";
        emptyMsg.style.color = "var(--text-secondary, #666)";
        list.appendChild(emptyMsg);
      }
    }
  } catch (error) {
    console.error("Failed to refresh CPU details", error);
  }
}

// Wait for Tauri to be available
function waitForTauri(callback, maxAttempts = 100) {
  const invokeFn = getInvoke();
  
  if (invokeFn) {
    callback(invokeFn);
    return;
  }
  
  if (maxAttempts > 0) {
    setTimeout(() => waitForTauri(callback, maxAttempts - 1), 50);
  } else {
    console.error("Tauri API not available after waiting");
  }
}

// Start refreshing when Tauri is ready
// Use 3 second interval to reduce CPU usage
function startRefresh() {
  refresh();
  if (refreshInterval) {
    clearInterval(refreshInterval);
  }
  refreshInterval = setInterval(refresh, 3000); // 3 seconds to reduce CPU usage
}

// Initialize when DOM and Tauri are ready
function init() {
  waitForTauri((invokeFn) => {
    invoke = invokeFn;
    startRefresh();
  });
}

// Initialize ring gauges
function initRingGauges() {
  const rings = ['temperature-ring-progress', 'cpu-usage-ring-progress', 'frequency-ring-progress'];
  rings.forEach(ringId => {
    const el = document.getElementById(ringId);
    if (el) {
      el.style.strokeDasharray = CIRCUMFERENCE;
      el.style.strokeDashoffset = CIRCUMFERENCE;
    }
  });
}

// Try multiple initialization strategies
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", () => {
    initRingGauges();
    init();
  });
} else {
  initRingGauges();
  init();
}

window.addEventListener("load", () => {
  if (!invoke) {
    init();
  }
});

document.addEventListener("visibilitychange", () => {
  if (!document.hidden && !refreshInterval && invoke) {
    startRefresh();
  } else if (!document.hidden && !invoke) {
    init();
  }
});
