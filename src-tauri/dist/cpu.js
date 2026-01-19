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
  
  // STEP 7: If change is very small (<5% of gauge), skip update entirely
  // OPTIMIZATION Phase 1: Increased from 2% to 5% (human perception threshold)
  // This prevents unnecessary WebKit rendering for imperceptible changes
  // BUT: Always allow updates if current value is at default (CIRCUMFERENCE) - means gauge wasn't painted yet
  if (diff < (CIRCUMFERENCE * 0.05) && anim.current !== CIRCUMFERENCE) {
    // Change is too small to be visible - skip update
    return;
  }
  
  // STEP 7: If change is small but visible, update directly without animation
  // OPTIMIZATION Phase 1: Increased threshold from 15% to 20% to reduce animation frequency
  if (diff < (CIRCUMFERENCE * 0.20)) {
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
    // Force process update on first call (when lastProcessUpdate is 0)
    const isFirstCall = lastProcessUpdate === 0;
    if (isFirstCall) {
      window._forceProcessUpdate = true;
    }
    
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
            // OPTIMIZATION Phase 2: Update first text node instead of innerHTML rebuild
            if (tempEl.firstChild && tempEl.firstChild.nodeType === 3) {
              tempEl.firstChild.textContent = numberText;
            } else {
              tempEl.innerHTML = `${numberText}<span class="metric-unit">°C</span>`;
            }
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
            // OPTIMIZATION Phase 2: Update first text node instead of innerHTML rebuild
            if (tempEl.firstChild && tempEl.firstChild.nodeType === 3) {
              tempEl.firstChild.textContent = numberText;
            } else {
              tempEl.innerHTML = `${numberText}<span class="metric-unit">°C</span>`;
            }
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
    
    // Update data-poster charts if available
    if (window.posterCharts && data.can_read_temperature && data.temperature > 0) {
      window.posterCharts.updateTemperature(data.temperature);
    }
    
    // Update dark theme history charts if available
    if (window.darkHistory && data.can_read_temperature && data.temperature > 0) {
      window.darkHistory.updateTemperature(data.temperature);
    }
    
    // Update light theme history charts if available
    if (window.lightHistory && data.can_read_temperature && data.temperature > 0) {
      window.lightHistory.updateTemperature(data.temperature);
    }
    
    // Update futuristic theme history charts if available
    if (window.futuristicHistory && data.can_read_temperature && data.temperature > 0) {
      window.futuristicHistory.updateTemperature(data.temperature);
    }
    
    // Update material theme history charts if available
    if (window.materialHistory && data.can_read_temperature && data.temperature > 0) {
      window.materialHistory.updateTemperature(data.temperature);
    }
    
    // Update neon theme history charts if available
    if (window.neonHistory && data.can_read_temperature && data.temperature > 0) {
      window.neonHistory.updateTemperature(data.temperature);
    }
    
    // Update swiss theme history charts if available
    if (window.swissHistory && data.can_read_temperature && data.temperature > 0) {
      window.swissHistory.updateTemperature(data.temperature);
    }
    
    // Update architect theme history charts if available
    if (window.architectHistory && data.can_read_temperature && data.temperature > 0) {
      window.architectHistory.updateTemperature(data.temperature);
    }

    // Update CPU usage
    const cpuUsageEl = document.getElementById("cpu-usage-value");
    const cpuUsageSubtext = document.getElementById("cpu-usage-subtext");
    // Always show usage as percentage, even if 0 (don't show "-")
    const newUsage = Math.max(0, Math.round(data.usage || 0));
    const numberText = `${newUsage}`;
    // Check if we need to update (extract number from current content)
    const currentText = cpuUsageEl.textContent.match(/^\d+/) ? cpuUsageEl.textContent.match(/^\d+/)[0] : "";
    
    if (currentText !== numberText) {
      scheduleDOMUpdate(() => {
        // OPTIMIZATION Phase 2: Update first text node instead of innerHTML rebuild
        if (cpuUsageEl.firstChild && cpuUsageEl.firstChild.nodeType === 3) {
          cpuUsageEl.firstChild.textContent = numberText;
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
    
    // Update data-poster charts if available
    if (window.posterCharts) {
      window.posterCharts.updateUsage(data.usage);
    }
    
    // Update dark theme history charts if available
    if (window.darkHistory) {
      window.darkHistory.updateUsage(data.usage);
    }
    
    // Update light theme history charts if available
    if (window.lightHistory) {
      window.lightHistory.updateUsage(data.usage);
    }
    
    // Update futuristic theme history charts if available
    if (window.futuristicHistory) {
      window.futuristicHistory.updateUsage(data.usage);
    }
    
    // Update material theme history charts if available
    if (window.materialHistory) {
      window.materialHistory.updateUsage(data.usage);
    }
    
    // Update neon theme history charts if available
    if (window.neonHistory) {
      window.neonHistory.updateUsage(data.usage);
    }
    
    // Update swiss theme history charts if available
    if (window.swissHistory) {
      window.swissHistory.updateUsage(data.usage);
    }
    
    // Update architect theme history charts if available
    if (window.architectHistory) {
      window.architectHistory.updateUsage(data.usage);
    }

    // Update frequency
    const freqEl = document.getElementById("frequency-value");
    const freqHint = document.getElementById("frequency-hint");
    const freqSubtext = document.getElementById("frequency-subtext");
    
    if (!data.can_read_frequency) {
      failedAttempts.frequency++;
      if (!freqEl.textContent.includes("—")) {
        scheduleDOMUpdate(() => {
          freqEl.innerHTML = "—<span class=\"metric-unit\">GHz</span>";
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
      // Check if we need to update (extract number from current content)
      const currentFreqText = freqEl.textContent.match(/[\d.]+/) ? freqEl.textContent.match(/[\d.]+/)[0] : "";
      
      if (currentFreqText !== formatted) {
        scheduleDOMUpdate(() => {
          // OPTIMIZATION Phase 2: Update first text node instead of innerHTML rebuild
          if (freqEl.firstChild && freqEl.firstChild.nodeType === 3) {
            freqEl.firstChild.textContent = formatted;
          } else {
            freqEl.innerHTML = `${formatted}<span class="metric-unit">GHz</span>`;
          }
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
    
      // Update data-poster charts if available
      if (window.posterCharts && data.frequency > 0) {
        window.posterCharts.updateFrequency(data.frequency);
      }
      
      // Update dark theme history charts if available
      if (window.darkHistory && data.frequency > 0) {
        window.darkHistory.updateFrequency(data.frequency);
      }
      
      // Update light theme history charts if available
      if (window.lightHistory && data.frequency > 0) {
        window.lightHistory.updateFrequency(data.frequency);
      }
      
      // Update futuristic theme history charts if available
      if (window.futuristicHistory && data.frequency > 0) {
        window.futuristicHistory.updateFrequency(data.frequency);
      }
      
      // Update material theme history charts if available
      if (window.materialHistory && data.frequency > 0) {
        window.materialHistory.updateFrequency(data.frequency);
      }
      
      // Update neon theme history charts if available
      if (window.neonHistory && data.frequency > 0) {
        window.neonHistory.updateFrequency(data.frequency);
      }
      
      // Update swiss theme history charts if available
      if (window.swissHistory && data.frequency > 0) {
        window.swissHistory.updateFrequency(data.frequency);
      }
      
      // Update architect theme history charts if available
      if (window.architectHistory && data.frequency > 0) {
        window.architectHistory.updateFrequency(data.frequency);
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
    if (!data.can_read_cpu_power) {
      failedAttempts.cpuPower++;
      // Only show hint after multiple failed attempts
      const shouldShowHint = failedAttempts.cpuPower >= FAILED_ATTEMPTS_THRESHOLD;
      const displayText = shouldShowHint ? "Requires root privileges" : "0.0 W";
      if (cpuPowerEl.textContent !== displayText) {
        scheduleDOMUpdate(() => {
          cpuPowerEl.textContent = displayText;
        });
      }
    } else {
      failedAttempts.cpuPower = 0;
      const formatted = `${data.cpu_power.toFixed(2)} W`;
      if (cpuPowerEl.textContent !== formatted) {
        scheduleDOMUpdate(() => {
          cpuPowerEl.textContent = formatted;
        });
        previousValues.cpuPower = data.cpu_power;
      }
    }

    const gpuPowerEl = document.getElementById("gpu-power");
    if (!data.can_read_gpu_power) {
      failedAttempts.gpuPower++;
      // Only show hint after multiple failed attempts
      const shouldShowHint = failedAttempts.gpuPower >= FAILED_ATTEMPTS_THRESHOLD;
      const displayText = shouldShowHint ? "Requires root privileges" : "0.0 W";
      if (gpuPowerEl.textContent !== displayText) {
        scheduleDOMUpdate(() => {
          gpuPowerEl.textContent = displayText;
        });
      }
    } else {
      failedAttempts.gpuPower = 0;
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
    // But allow forced immediate updates when needed (e.g., after force quit, or on initial load)
    const now = Date.now();
    const forceUpdate = window._forceProcessUpdate === true;
    const isInitialLoad = lastProcessUpdate === 0;
    if (forceUpdate || isInitialLoad || now - lastProcessUpdate >= 15000) {
      lastProcessUpdate = now;
      window._forceProcessUpdate = false; // Reset flag after use
      
      const list = document.getElementById("process-list");
      
      // STEP 7: Use document fragment to batch all DOM updates
      // This reduces WebKit reflows from multiple appendChild calls
      const fragment = document.createDocumentFragment();
      
      if (data.top_processes && data.top_processes.length > 0) {
        data.top_processes.slice(0, 8).forEach((proc) => {
          const row = document.createElement("div");
          row.className = "process-row";
          row.setAttribute("data-pid", proc.pid);
          row.style.cursor = "pointer";
          row.title = "Click for details";
          
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
          
          // Add click handler for process details
          row.addEventListener("click", () => {
            showProcessDetails(proc.pid);
          });
          
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
  // Force immediate process update on initial load
  window._forceProcessUpdate = true;
  
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
  // Fetch app version once at startup (no polling for CPU efficiency)
  let appVersion = null;
  
  async function fetchAppVersion() {
    if (appVersion !== null) {
      return appVersion; // Already fetched, return cached value
    }
    
    const invoke = getInvoke();
    if (!invoke) {
      appVersion = "unknown";
      return appVersion;
    }
    
    try {
      appVersion = await invoke("get_app_version");
      // Set version in all footer elements
      const versionElements = document.querySelectorAll('.app-version, .theme-version, .arch-version');
      versionElements.forEach(el => {
        const text = el.textContent;
        // Preserve theme name if present (e.g., "Apple v0.0.3" -> "Apple v0.0.4")
        if (text.includes('v')) {
          const parts = text.split('v');
          const themeName = parts[0].trim();
          if (themeName) {
            el.textContent = `${themeName} v${appVersion}`;
          } else {
            el.textContent = `v${appVersion}`;
          }
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

  document.addEventListener("DOMContentLoaded", () => {
    // Fetch version once at startup (no polling)
    fetchAppVersion();
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
    // Window became visible - refresh immediately and force process update
    window._forceProcessUpdate = true; // Force immediate process list update
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

// Process details popover
let processDetailsModal = null;
let currentProcessPid = null;
let processDetailsRefreshInterval = null;

function formatBytes(bytes) {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + " " + sizes[i];
}

function formatTime(seconds) {
  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  return `${hours}h ${minutes}m`;
}

function formatDate(timestamp) {
  const date = new Date(timestamp * 1000);
  const now = new Date();
  const diffMs = now - date;
  const diffSeconds = Math.floor(diffMs / 1000);
  const diffMinutes = Math.floor(diffSeconds / 60);
  const diffHours = Math.floor(diffMinutes / 60);
  const diffDays = Math.floor(diffHours / 24);
  
  // Format the date nicely: "18th January 2026, 3:45 PM"
  const day = date.getDate();
  const daySuffix = getDaySuffix(day);
  const monthNames = ["January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December"];
  const month = monthNames[date.getMonth()];
  const year = date.getFullYear();
  
  let hours = date.getHours();
  const minutes = date.getMinutes();
  const ampm = hours >= 12 ? 'PM' : 'AM';
  hours = hours % 12;
  hours = hours ? hours : 12; // the hour '0' should be '12'
  const minutesStr = minutes < 10 ? '0' + minutes : minutes;
  
  const formattedDate = `${day}${daySuffix} ${month} ${year}, ${hours}:${minutesStr} ${ampm}`;
  
  // Calculate relative time: "1 day 15h ago"
  let relativeTime = "";
  if (diffDays > 0) {
    const remainingHours = diffHours % 24;
    if (remainingHours > 0) {
      relativeTime = `${diffDays} day${diffDays > 1 ? 's' : ''} ${remainingHours}h ago`;
    } else {
      relativeTime = `${diffDays} day${diffDays > 1 ? 's' : ''} ago`;
    }
  } else if (diffHours > 0) {
    relativeTime = `${diffHours}h ago`;
  } else if (diffMinutes > 0) {
    relativeTime = `${diffMinutes}m ago`;
  } else {
    relativeTime = `${diffSeconds}s ago`;
  }
  
  return `${formattedDate} - ${relativeTime}`;
}

function getDaySuffix(day) {
  if (day > 3 && day < 21) return 'th';
  switch (day % 10) {
    case 1: return 'st';
    case 2: return 'nd';
    case 3: return 'rd';
    default: return 'th';
  }
}

async function updateProcessDetailsContent(pid) {
  // CRITICAL: Only refresh if modal is actually visible
  // This prevents unnecessary backend calls when modal is closed
  if (!processDetailsModal || processDetailsModal.style.display === "none") {
    // Modal is not visible, don't refresh
    return;
  }
  
  if (!invoke) {
    invoke = getInvoke();
    if (!invoke) {
      console.error("Cannot refresh process details: Tauri invoke not available");
      return;
    }
  }
  
  try {
    const details = await invoke("get_process_details", { pid });
    
    // Double-check modal is still visible after async call (might have been closed)
    if (!processDetailsModal || processDetailsModal.style.display === "none") {
      return;
    }
    
    const body = document.getElementById("process-details-body");
    if (!body) return;
    
    populateProcessDetailsBody(body, details, pid);
  } catch (error) {
    console.error("Failed to refresh process details:", error);
    // Don't show alert on auto-refresh failures, only log
  }
}

function populateProcessDetailsBody(body, details, pid) {
    const startDate = formatDate(details.start_time);
    const cpuTimeFormatted = formatTime(Math.floor(details.total_cpu_time / 1000));
    const memoryFormatted = formatBytes(details.memory);
    const virtualMemoryFormatted = formatBytes(details.virtual_memory);
    const diskReadFormatted = formatBytes(details.disk_read);
    const diskWrittenFormatted = formatBytes(details.disk_written);
    
    body.innerHTML = `
      <div class="process-detail-row">
        <span class="process-detail-label">Name</span>
        <span class="process-detail-value">${details.name}</span>
      </div>
      <div class="process-detail-row">
        <span class="process-detail-label">PID</span>
        <span class="process-detail-value">${details.pid}</span>
      </div>
      <div class="process-detail-row">
        <span class="process-detail-label">Current CPU</span>
        <span class="process-detail-value">${details.cpu.toFixed(1)}%</span>
      </div>
      <div class="process-detail-row">
        <span class="process-detail-label">Total CPU Time</span>
        <span class="process-detail-value">${cpuTimeFormatted}</span>
      </div>
      <div class="process-detail-row">
        <span class="process-detail-label">Parent Process</span>
        <span class="process-detail-value">${details.parent_name ? `${details.parent_name} (PID: ${details.parent_pid})` : "—"}</span>
      </div>
      <div class="process-detail-row">
        <span class="process-detail-label">Started</span>
        <span class="process-detail-value">${startDate}</span>
      </div>
      <div class="process-detail-row">
        <span class="process-detail-label">User</span>
        <span class="process-detail-value">${details.user_name ? `${details.user_name} (${details.user_id})` : (details.user_id || "—")}</span>
      </div>
      <div class="process-detail-row">
        <span class="process-detail-label">Effective User</span>
        <span class="process-detail-value">${details.effective_user_name ? `${details.effective_user_name} (${details.effective_user_id})` : (details.effective_user_id || "—")}</span>
      </div>
      <div class="process-detail-row-group">
        <div class="process-detail-row">
          <span class="process-detail-label">Memory</span>
          <span class="process-detail-value">${memoryFormatted}</span>
        </div>
        <div class="process-detail-row">
          <span class="process-detail-label">Virtual Memory</span>
          <span class="process-detail-value">${virtualMemoryFormatted}</span>
        </div>
      </div>
      <div class="process-detail-row-group">
        <div class="process-detail-row">
          <span class="process-detail-label">Disk Read</span>
          <span class="process-detail-value">${diskReadFormatted}</span>
        </div>
        <div class="process-detail-row">
          <span class="process-detail-label">Disk Written</span>
          <span class="process-detail-value">${diskWrittenFormatted}</span>
        </div>
      </div>
      <div class="force-quit-section">
        <button id="force-quit-process-btn" class="force-quit-btn">Force Quit Process</button>
      </div>
    `;
    
    // Set up force quit button handler (remove old listeners first by cloning)
    const forceQuitBtn = document.getElementById("force-quit-process-btn");
    if (forceQuitBtn) {
      // Clone and replace to remove old event listeners when refreshing
      const newBtn = forceQuitBtn.cloneNode(true);
      forceQuitBtn.parentNode.replaceChild(newBtn, forceQuitBtn);
      
      newBtn.addEventListener("click", async () => {
        if (!confirm(`Are you sure you want to force quit "${details.name}" (PID: ${pid})? This action cannot be undone.`)) {
          return;
        }
        
        try {
          if (!invoke) {
            invoke = getInvoke();
            if (!invoke) {
              alert("Cannot force quit: Tauri invoke not available");
              return;
            }
          }
          
          await invoke("force_quit_process", { pid });
          
          // Clear refresh interval and close modal
          if (processDetailsRefreshInterval) {
            clearInterval(processDetailsRefreshInterval);
            processDetailsRefreshInterval = null;
          }
          currentProcessPid = null;
          processDetailsModal.style.display = "none";
          
          // Force immediate refresh of process list (bypass 15-second throttle)
          window._forceProcessUpdate = true;
          if (window.refreshData) {
            // Refresh immediately to show updated process list
            await window.refreshData();
          }
          
          alert(`Process "${details.name}" has been force quit.`);
        } catch (error) {
          console.error("Failed to force quit process:", error);
          alert(`Failed to force quit process: ${error}`);
        }
      });
    }
}

async function showProcessDetails(pid) {
  if (!invoke) {
    invoke = getInvoke();
    if (!invoke) {
      console.error("Cannot show process details: Tauri invoke not available");
      return;
    }
  }
  
  try {
    const details = await invoke("get_process_details", { pid });
    
    // Use existing modal from HTML or create it
    processDetailsModal = document.getElementById("process-details-modal");
    if (!processDetailsModal) {
      // Create modal if it doesn't exist in HTML
      processDetailsModal = document.createElement("div");
      processDetailsModal.id = "process-details-modal";
      processDetailsModal.className = "settings-modal";
      processDetailsModal.style.display = "none";
      processDetailsModal.innerHTML = `
        <div class="settings-card">
          <div class="settings-header">
            <h2>Process Details</h2>
            <button id="close-process-details" class="icon-btn" aria-label="Close">×</button>
          </div>
          <div class="settings-body" id="process-details-body"></div>
        </div>
      `;
      document.body.appendChild(processDetailsModal);
    }
    
    // Set up close handlers (only once)
    if (!processDetailsModal.dataset.handlersSetup) {
      const closeBtn = processDetailsModal.querySelector("#close-process-details");
      if (closeBtn) {
        closeBtn.addEventListener("click", () => {
          // Clear refresh interval when closing
          if (processDetailsRefreshInterval) {
            clearInterval(processDetailsRefreshInterval);
            processDetailsRefreshInterval = null;
          }
          currentProcessPid = null;
          processDetailsModal.style.display = "none";
        });
      }
      
      // Click outside to close
      processDetailsModal.addEventListener("click", (e) => {
        if (e.target === processDetailsModal) {
          // Clear refresh interval when closing
          if (processDetailsRefreshInterval) {
            clearInterval(processDetailsRefreshInterval);
            processDetailsRefreshInterval = null;
          }
          currentProcessPid = null;
          processDetailsModal.style.display = "none";
        }
      });
      
      // ESC key to close
      const escHandler = (e) => {
        if (e.key === "Escape" && processDetailsModal.style.display !== "none") {
          // Clear refresh interval when closing
          if (processDetailsRefreshInterval) {
            clearInterval(processDetailsRefreshInterval);
            processDetailsRefreshInterval = null;
          }
          currentProcessPid = null;
          processDetailsModal.style.display = "none";
        }
      };
      document.addEventListener("keydown", escHandler);
      processDetailsModal.dataset.handlersSetup = "true";
    }
    
    // Store current PID for refresh functionality
    currentProcessPid = pid;
    
    // Clear any existing refresh interval
    if (processDetailsRefreshInterval) {
      clearInterval(processDetailsRefreshInterval);
      processDetailsRefreshInterval = null;
    }
    
    // Populate details
    const body = document.getElementById("process-details-body");
    populateProcessDetailsBody(body, details, pid);
    
    // Show modal (using same display style as settings modal)
    processDetailsModal.style.display = "flex";
    
    // Start auto-refresh every 2 seconds while modal is open
    // CRITICAL: Only refresh if modal is actually visible (checked in updateProcessDetailsContent too)
    processDetailsRefreshInterval = setInterval(() => {
      // Check if modal is visible before refreshing
      if (currentProcessPid !== null && 
          processDetailsModal && 
          processDetailsModal.style.display !== "none") {
        updateProcessDetailsContent(currentProcessPid);
      } else {
        // Modal closed or not visible, clear interval to stop refreshing
        if (processDetailsRefreshInterval) {
          clearInterval(processDetailsRefreshInterval);
          processDetailsRefreshInterval = null;
        }
        currentProcessPid = null;
      }
    }, 2000);
  } catch (error) {
    console.error("Failed to fetch process details:", error);
    alert(`Failed to fetch process details: ${error}`);
  }
}

// OPTIMIZATION Phase 2: Cleanup on window unload
window.addEventListener('beforeunload', () => {
  ringAnimations.clear();  // Clear animation state map
  pendingDOMUpdates = [];  // Clear pending updates
  if (refreshInterval) {
    clearInterval(refreshInterval);
  }
  if (processDetailsRefreshInterval) {
    clearInterval(processDetailsRefreshInterval);
    processDetailsRefreshInterval = null;
  }
  console.log('Cleaned up animation state on window close');
});
