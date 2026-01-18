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
  
  // Check if this is the first time we're updating this gauge
  const isFirstUpdate = !ringAnimations.has(key);
  
  if (isFirstUpdate) {
    // First update: initialize and paint immediately (no animation, no batching)
    ringAnimations.set(key, { current: targetOffset, target: targetOffset, lastFrameTime: null, frameId: null });
    // Paint immediately on first load - don't batch this, it needs to show right away
    progressEl.style.strokeDashoffset = targetOffset;
    return;
  }
  
  const anim = ringAnimations.get(key);
  const diff = Math.abs(anim.current - targetOffset);
  
  // STEP 7: If change is very small (<2% of gauge), skip update entirely
  // This prevents unnecessary WebKit rendering for imperceptible changes
  // BUT: Always allow updates if current value is at default (CIRCUMFERENCE) - means gauge wasn't painted yet
  if (diff < (CIRCUMFERENCE * 0.02) && anim.current !== CIRCUMFERENCE) {
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
let isWaitingForData = false; // Track if we're waiting for real data (non-zero usage)

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
    
    // CRITICAL: If we're waiting for real data and we got it, switch to normal interval
    // Match menu bar update frequency (1 second) for consistent CPU usage display
    if (isWaitingForData && data.usage > 0.0) {
      isWaitingForData = false;
      // Clear the fast polling interval
      if (refreshInterval) {
        clearInterval(refreshInterval);
      }
      // Start 1-second interval to match menu bar update frequency
      refreshInterval = setInterval(refresh, 1000);
      console.log("Got real data, switched to 1-second interval (matching menu bar)");
    }
    
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
      const currentDisplay = tempEl.textContent.replace(/°C/g, "").trim();
      if (currentDisplay !== "—") {
        scheduleDOMUpdate(() => {
          tempEl.innerHTML = "—";
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
        // Still show it as "0°C" to indicate we're trying to read it
        const numberText = "0";
        const currentText = tempEl.textContent.match(/^\d+/) ? tempEl.textContent.match(/^\d+/)[0] : "";
        
        if (currentText !== numberText) {
          scheduleDOMUpdate(() => {
            // Always rebuild with the correct structure: number + span
            tempEl.innerHTML = `${numberText}<span class="metric-unit">°C</span>`;
          });
          previousValues.temperature = 0;
        }
        if (tempSubtext.textContent !== "SMC: No data") {
          scheduleDOMUpdate(() => {
            tempSubtext.textContent = "SMC: No data";
          });
        }
      } else {
        const numberText = `${newTemp}`;
        // Get current number by extracting digits from textContent (ignoring °C)
        const currentText = tempEl.textContent.match(/^\d+/) ? tempEl.textContent.match(/^\d+/)[0] : "";
        
        if (currentText !== numberText) {
          scheduleDOMUpdate(() => {
            // Always rebuild with the correct structure: number + span
            tempEl.innerHTML = `${numberText}<span class="metric-unit">°C</span>`;
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
    // Always update ring gauge (it handles first paint and change detection internally)
    updateRingGauge("temperature-ring-progress", Math.min(100, data.temperature), 'temperature');

    // Update CPU usage
    const cpuUsageEl = document.getElementById("cpu-usage-value");
    const cpuUsageSubtext = document.getElementById("cpu-usage-subtext");
    // Always show usage as percentage, even if 0 (don't show "-")
    const newUsage = Math.max(0, Math.round(data.usage || 0));
    const numberText = `${newUsage}`;
    const currentText = cpuUsageEl.childNodes[0]?.textContent || "";
    
    if (currentText !== numberText) {
      scheduleDOMUpdate(() => {
        // Update the number part, keep the % span
        if (cpuUsageEl.childNodes[0]) {
          cpuUsageEl.childNodes[0].textContent = numberText;
        } else {
          cpuUsageEl.innerHTML = `${numberText}<span class="metric-unit">%</span>`;
        }
      });
      previousValues.usage = newUsage;
    }
    // Update usage subtext to show "Avg. 10s"
    const usageSubtext = "Avg. 10s";
    if (cpuUsageSubtext.textContent !== usageSubtext) {
      scheduleDOMUpdate(() => {
        cpuUsageSubtext.textContent = usageSubtext;
      });
    }
    
    // Always update ring gauge (it handles first paint and change detection internally)
    updateRingGauge("cpu-usage-ring-progress", data.usage, 'usage');

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
      // Display P-core and E-core frequencies if available (removed "GHz" to prevent flickering)
      // CRITICAL: Cache last known good values to prevent flickering when values temporarily become 0
      let subtext = freqSubtext.textContent || "—"; // Keep current value if no new valid data
      
      // Only update if we have valid P/E core frequencies (both > 0)
      if (data.p_core_frequency && data.p_core_frequency > 0 && data.e_core_frequency && data.e_core_frequency > 0) {
        subtext = `P: ${data.p_core_frequency.toFixed(1)} • E: ${data.e_core_frequency.toFixed(1)}`;
      } else if (data.p_core_frequency && data.p_core_frequency > 0) {
        // Only P-core available
        subtext = `P: ${data.p_core_frequency.toFixed(1)}`;
      } else if (data.e_core_frequency && data.e_core_frequency > 0) {
        // Only E-core available
        subtext = `E: ${data.e_core_frequency.toFixed(1)}`;
      }
      // If neither is available, keep the last known value (don't switch to "—" immediately)
      // Only update if subtext actually changed to prevent flickering
      if (freqSubtext.textContent !== subtext) {
        scheduleDOMUpdate(() => {
          freqSubtext.textContent = subtext;
        });
      }
    }
    // Always update ring gauge (it handles first paint and change detection internally)
    updateRingGauge("frequency-ring-progress", Math.min(100, (data.frequency / 5.0) * 100), 'frequency');

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
// CRITICAL: Keep trying even after maxAttempts - Tauri might not be ready when window first opens
function waitForTauri(callback, maxAttempts = 200) {
  const invokeFn = getInvoke();
  
  if (invokeFn) {
    callback(invokeFn);
    return;
  }
  
  if (maxAttempts > 0) {
    setTimeout(() => waitForTauri(callback, maxAttempts - 1), 50);
  } else {
    // Don't give up - keep trying every 100ms until Tauri is ready
    // This ensures we call refresh() as soon as Tauri becomes available
    console.warn("Tauri API not available yet, continuing to wait...");
    setTimeout(() => waitForTauri(callback, 0), 100);
  }
}

// Start refreshing when Tauri is ready
// CRITICAL: Poll every 1 second (matches menu bar update frequency)
// This ensures CPU usage gauge updates at same rate as menu bar
function startRefresh() {
  // Don't call refresh() here - it's already called in init() or visibilitychange
  // This prevents double-calling on startup
  if (refreshInterval) {
    clearInterval(refreshInterval);
  }
  
  // Check if we got real data on first call
  // If not, poll every 1 second until we do (SYSTEM might not be initialized yet)
  // Once we get real data (usage > 0), continue with 1-second interval (matches menu bar)
  isWaitingForData = true;
  refreshInterval = setInterval(refresh, 1000); // 1-second polling (matches menu bar frequency)
}

// Initialize when DOM and Tauri are ready
function init() {
  // Try to get Tauri immediately - don't wait if it's already available
  const immediateInvoke = getInvoke();
  if (immediateInvoke) {
    invoke = immediateInvoke;
    // Call refresh immediately - don't wait for interval
    refresh();
    startRefresh();
  } else {
    // Tauri not ready yet - wait for it
    waitForTauri((invokeFn) => {
      invoke = invokeFn;
      // Call refresh immediately when Tauri becomes available
      refresh();
      startRefresh();
    });
  }
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
  if (!document.hidden) {
    // Window became visible - refresh immediately
    if (invoke) {
      // Tauri is ready - refresh immediately and start interval
      refresh(); // Immediate refresh
      if (!refreshInterval) {
        startRefresh();
      }
    } else {
      // Tauri not ready - initialize (will keep trying until ready)
      init();
    }
  }
});
