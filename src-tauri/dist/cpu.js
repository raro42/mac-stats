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

// STEP 7: Batch DOM updates to reduce WebKit rendering
// Collect all DOM changes and apply them in a single requestAnimationFrame
let pendingDOMUpdates = [];
let domUpdateScheduled = false;

function scheduleDOMUpdate(updateFn) {
  pendingDOMUpdates.push(updateFn);
  if (!domUpdateScheduled) {
    domUpdateScheduled = true;
    requestAnimationFrame(() => {
      // Apply all pending updates in one batch
      pendingDOMUpdates.forEach(fn => fn());
      pendingDOMUpdates = [];
      domUpdateScheduled = false;
    });
  }
}

// Track failed attempts before showing "Requires root privileges" hint
const failedAttempts = {
  temperature: 0,
  frequency: 0,
  cpuPower: 0,
  gpuPower: 0
};

// Number of consecutive failures before showing the hint
const FAILED_ATTEMPTS_THRESHOLD = 3;

// SVG Ring Gauge Animation
const ringAnimations = new Map();
const CIRCUMFERENCE = 2 * Math.PI * 42; // radius = 42

function updateRingGauge(ringId, percent, key) {
  const clamped = Math.max(0, Math.min(100, percent));
  const progressEl = document.getElementById(ringId);
  if (!progressEl) return;
  
  const targetOffset = CIRCUMFERENCE - (clamped / 100) * CIRCUMFERENCE;
  
  // STEP 7: Only update if change is significant (>2% of gauge) to reduce WebKit rendering
  // Skip updates for tiny changes - they're not visible anyway
  if (!ringAnimations.has(key)) {
    ringAnimations.set(key, { current: CIRCUMFERENCE, target: targetOffset, lastFrameTime: null });
  }
  
  const anim = ringAnimations.get(key);
  const diff = Math.abs(anim.current - targetOffset);
  
  // STEP 7: If change is very small (<2% of gauge), skip update entirely
  // This prevents unnecessary WebKit rendering for imperceptible changes
  if (diff < (CIRCUMFERENCE * 0.02)) {
    // Change is too small to be visible - skip update
    return;
  }
  
  // STEP 7: If change is small but visible, update directly without animation
  // Increased threshold from 10% to 15% to reduce animation frequency
  if (diff < (CIRCUMFERENCE * 0.15)) {
    anim.current = targetOffset;
    // Batch this DOM update
    scheduleDOMUpdate(() => {
      progressEl.style.strokeDashoffset = anim.current;
    });
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
    
    // STEP 6: Throttle to 20fps (update every ~50ms) to reduce Graphics/Media CPU usage
    // CRITICAL: Only call requestAnimationFrame if we're actually updating
    // This prevents WebKit from processing unnecessary display link callbacks
    const now = performance.now();
    if (!anim.lastFrameTime) {
      anim.lastFrameTime = now;
    }
    const elapsed = now - anim.lastFrameTime;
    
    if (elapsed >= 50) { // 20fps = 50ms per frame (reduced from 30fps to save CPU)
      // Faster animation (0.35 instead of 0.3) to complete sooner with fewer frames
      anim.current += diff * 0.35;
      // Batch DOM update
      scheduleDOMUpdate(() => {
        progressEl.style.strokeDashoffset = anim.current;
      });
      anim.lastFrameTime = now;
      
      // Only schedule next frame if we're not done
      if (Math.abs(anim.target - anim.current) >= 0.5) {
        anim.frameId = requestAnimationFrame(animate);
      } else {
        anim.current = anim.target;
        scheduleDOMUpdate(() => {
          progressEl.style.strokeDashoffset = anim.current;
        });
        ringAnimations.delete(key);
      }
    } else {
      // Not time to update yet - schedule next check but don't update DOM
      // This reduces WebKit rendering work
      anim.frameId = requestAnimationFrame(animate);
    }
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
function updateChipInfo(chipInfo, uptimeSecs) {
  const chipInfoEl = document.getElementById('chip-info');
  if (chipInfoEl && chipInfo) {
    let displayText = chipInfo;
    if (uptimeSecs !== undefined && uptimeSecs > 0) {
      const uptimeFormatted = formatUptime(uptimeSecs);
      displayText = `${chipInfo} · ${uptimeFormatted}`;
    }
    chipInfoEl.textContent = displayText;
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
    
    // STEP 7: Batch all DOM updates to reduce WebKit rendering
    // Collect all changes first, then apply in one batch
    
    // Update chip info with uptime
    updateChipInfo(data.chip_info, data.uptime_secs);
    
    // Update temperature
    const tempEl = document.getElementById("temperature-value");
    const tempHint = document.getElementById("temperature-hint");
    const tempSubtext = document.getElementById("temperature-subtext");
    const newTemp = Math.round(data.temperature);
    
    if (!data.can_read_temperature) {
      failedAttempts.temperature++;
      if (tempEl.textContent !== "—") {
        scheduleDOMUpdate(() => {
          tempEl.textContent = "—";
          tempSubtext.textContent = "—";
        });
      }
      // Only show hint after multiple failed attempts
      const shouldShowHint = failedAttempts.temperature >= FAILED_ATTEMPTS_THRESHOLD;
      if (tempHint.style.display !== (shouldShowHint ? "block" : "none")) {
        scheduleDOMUpdate(() => {
          tempHint.style.display = shouldShowHint ? "block" : "none";
        });
      }
    } else {
      failedAttempts.temperature = 0;
      if (tempHint.style.display !== "none") {
        scheduleDOMUpdate(() => {
          tempHint.style.display = "none";
        });
      }
      // Show temperature even if it's 0.0 (might be unsupported Mac model)
      // But show "—" if temperature is exactly 0.0 and we've been trying for a while
      if (newTemp === 0 && data.temperature === 0.0) {
        // Temperature is 0.0 - might be unsupported Mac model
        // Still show it as "0°" to indicate we're trying to read it
        const formatted = "0°";
        if (tempEl.textContent !== formatted) {
          scheduleDOMUpdate(() => {
            tempEl.textContent = formatted;
          });
          previousValues.temperature = 0;
        }
        if (tempSubtext.textContent !== "SMC: No data") {
          scheduleDOMUpdate(() => {
            tempSubtext.textContent = "SMC: No data";
          });
        }
      } else {
        const formatted = `${newTemp}°`;
        if (tempEl.textContent !== formatted) {
          scheduleDOMUpdate(() => {
            tempEl.textContent = formatted;
          });
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
          scheduleDOMUpdate(() => {
            tempSubtext.textContent = thermalText;
          });
        }
      }
    }
    // STEP 7: Only update ring gauge if temperature changed significantly (>1°C)
    if (Math.abs(data.temperature - previousValues.temperature) > 1.0) {
      updateRingGauge("temperature-ring-progress", Math.min(100, data.temperature), 'temperature');
    }

    // Update CPU usage
    const cpuUsageEl = document.getElementById("cpu-usage-value");
    const cpuUsageSubtext = document.getElementById("cpu-usage-subtext");
    const newUsage = Math.round(data.usage);
    const formatted = `${newUsage}%`;
    
    if (cpuUsageEl.textContent !== formatted) {
      scheduleDOMUpdate(() => {
        cpuUsageEl.textContent = formatted;
      });
      previousValues.usage = newUsage;
    }
    // STEP 7: Only update subtext if it actually changed (it's static, so skip)
    // Removed unnecessary subtext update - it's always "Avg last 10s"
    
    // STEP 7: Only update ring gauge if usage changed significantly (>0.5%)
    if (Math.abs(data.usage - previousValues.usage) > 0.5) {
      updateRingGauge("cpu-usage-ring-progress", data.usage, 'usage');
    }

    // Update frequency
    const freqEl = document.getElementById("frequency-value");
    const freqHint = document.getElementById("frequency-hint");
    const freqSubtext = document.getElementById("frequency-subtext");
    
    if (!data.can_read_frequency) {
      failedAttempts.frequency++;
      if (freqEl.textContent !== "—") {
        scheduleDOMUpdate(() => {
          freqEl.textContent = "—";
          freqSubtext.textContent = "—";
        });
      }
      // Only show hint after multiple failed attempts
      const shouldShowHint = failedAttempts.frequency >= FAILED_ATTEMPTS_THRESHOLD;
      if (freqHint.style.display !== (shouldShowHint ? "block" : "none")) {
        scheduleDOMUpdate(() => {
          freqHint.style.display = shouldShowHint ? "block" : "none";
        });
      }
    } else {
      failedAttempts.frequency = 0;
      if (freqHint.style.display !== "none") {
        scheduleDOMUpdate(() => {
          freqHint.style.display = "none";
        });
      }
      const formatted = data.frequency.toFixed(1);
      if (freqEl.textContent !== formatted) {
        scheduleDOMUpdate(() => {
          freqEl.textContent = formatted;
        });
        previousValues.frequency = data.frequency;
      }
      // Display P-core and E-core frequencies if available
      let subtext = "GHz";
      if (data.p_core_frequency && data.p_core_frequency > 0 && data.e_core_frequency && data.e_core_frequency > 0) {
        subtext = `P: ${data.p_core_frequency.toFixed(1)} • E: ${data.e_core_frequency.toFixed(1)}`;
      } else if (data.p_core_frequency && data.p_core_frequency > 0) {
        subtext = `P: ${data.p_core_frequency.toFixed(1)}`;
      } else if (data.e_core_frequency && data.e_core_frequency > 0) {
        subtext = `E: ${data.e_core_frequency.toFixed(1)}`;
      }
      if (freqSubtext.textContent !== subtext) {
        scheduleDOMUpdate(() => {
          freqSubtext.textContent = subtext;
        });
      }
    }
    // STEP 7: Only update ring gauge if frequency changed significantly (>0.1 GHz)
    if (Math.abs(data.frequency - previousValues.frequency) > 0.1) {
      updateRingGauge("frequency-ring-progress", Math.min(100, (data.frequency / 5.0) * 100), 'frequency');
    }

    // Update uptime
    const uptimeEl = document.getElementById("uptime-value");
    const uptimeFormatted = formatUptime(data.uptime_secs);
    if (uptimeEl.textContent !== uptimeFormatted) {
      scheduleDOMUpdate(() => {
        uptimeEl.textContent = uptimeFormatted;
      });
    }

    // Update load averages (simple updates, no tweening)
    const load1El = document.getElementById("load-1");
    const newLoad1 = data.load_1.toFixed(2);
    if (load1El.textContent !== newLoad1) {
      scheduleDOMUpdate(() => {
        load1El.textContent = newLoad1;
      });
      previousValues.load1 = data.load_1;
    }

    const load5El = document.getElementById("load-5");
    const newLoad5 = data.load_5.toFixed(2);
    if (load5El.textContent !== newLoad5) {
      scheduleDOMUpdate(() => {
        load5El.textContent = newLoad5;
      });
      previousValues.load5 = data.load_5;
    }

    const load15El = document.getElementById("load-15");
    const newLoad15 = data.load_15.toFixed(2);
    if (load15El.textContent !== newLoad15) {
      scheduleDOMUpdate(() => {
        load15El.textContent = newLoad15;
      });
      previousValues.load15 = data.load_15;
    }

    // Update power consumption (simple updates)
    const cpuPowerEl = document.getElementById("cpu-power");
    const cpuPowerHint = document.getElementById("cpu-power-hint");
    if (!data.can_read_cpu_power) {
      failedAttempts.cpuPower++;
      if (cpuPowerEl.textContent !== "0.0 W") {
        scheduleDOMUpdate(() => {
          cpuPowerEl.textContent = "0.0 W";
        });
      }
      // Only show hint after multiple failed attempts
      const shouldShowHint = failedAttempts.cpuPower >= FAILED_ATTEMPTS_THRESHOLD;
      if (cpuPowerHint.style.display !== (shouldShowHint ? "block" : "none")) {
        scheduleDOMUpdate(() => {
          cpuPowerHint.style.display = shouldShowHint ? "block" : "none";
        });
      }
    } else {
      failedAttempts.cpuPower = 0;
      if (cpuPowerHint.style.display !== "none") {
        scheduleDOMUpdate(() => {
          cpuPowerHint.style.display = "none";
        });
      }
      const formatted = `${data.cpu_power.toFixed(2)} W`;
      if (cpuPowerEl.textContent !== formatted) {
        scheduleDOMUpdate(() => {
          cpuPowerEl.textContent = formatted;
        });
        previousValues.cpuPower = data.cpu_power;
      }
    }

    const gpuPowerEl = document.getElementById("gpu-power");
    const gpuPowerHint = document.getElementById("gpu-power-hint");
    if (!data.can_read_gpu_power) {
      failedAttempts.gpuPower++;
      if (gpuPowerEl.textContent !== "0.0 W") {
        scheduleDOMUpdate(() => {
          gpuPowerEl.textContent = "0.0 W";
        });
      }
      // Only show hint after multiple failed attempts
      const shouldShowHint = failedAttempts.gpuPower >= FAILED_ATTEMPTS_THRESHOLD;
      if (gpuPowerHint.style.display !== (shouldShowHint ? "block" : "none")) {
        scheduleDOMUpdate(() => {
          gpuPowerHint.style.display = shouldShowHint ? "block" : "none";
        });
      }
    } else {
      failedAttempts.gpuPower = 0;
      if (gpuPowerHint.style.display !== "none") {
        scheduleDOMUpdate(() => {
          gpuPowerHint.style.display = "none";
        });
      }
      const formatted = `${data.gpu_power.toFixed(2)} W`;
      if (gpuPowerEl.textContent !== formatted) {
        scheduleDOMUpdate(() => {
          gpuPowerEl.textContent = formatted;
        });
        previousValues.gpuPower = data.gpu_power;
      }
    }

    // STEP 7: Update process list only every 15 seconds to reduce CPU usage
    // Use document fragment to batch DOM updates and reduce WebKit reflows
    const now = Date.now();
    if (now - lastProcessUpdate >= 15000 || lastProcessUpdate === 0) {
      lastProcessUpdate = now;
      
      const list = document.getElementById("process-list");
      
      // STEP 7: Use document fragment to batch all DOM updates
      // This reduces WebKit reflows from multiple appendChild calls
      const fragment = document.createDocumentFragment();
      
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
          fragment.appendChild(row);
        });
      } else {
        // No processes available (window might be closed, saving CPU)
        const emptyMsg = document.createElement("div");
        emptyMsg.className = "process-empty";
        emptyMsg.textContent = "No process data available";
        emptyMsg.style.textAlign = "center";
        emptyMsg.style.padding = "1rem";
        emptyMsg.style.color = "var(--text-secondary, #666)";
        fragment.appendChild(emptyMsg);
      }
      
      // STEP 7: Batch the innerHTML clear and fragment append in one DOM update
      scheduleDOMUpdate(() => {
        list.innerHTML = "";
        list.appendChild(fragment);
      });
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
  refreshInterval = setInterval(refresh, 8000); // STEP 4: 8 seconds (increased from 5s) to reduce CPU usage
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
