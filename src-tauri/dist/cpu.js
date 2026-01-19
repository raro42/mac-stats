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
    
    // Update battery/power with the data we just fetched
    updateBatteryPower(data);
    
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

    // Update power consumption (with caching to prevent flickering)
    const cpuPowerEl = document.getElementById("cpu-power");
    if (!data.can_read_cpu_power) {
      failedAttempts.cpuPower++;
      // Only show hint after multiple failed attempts
      const shouldShowHint = failedAttempts.cpuPower >= FAILED_ATTEMPTS_THRESHOLD;
      const displayText = shouldShowHint ? "Requires root privileges" : "0 W";
      if (cpuPowerEl.textContent !== displayText) {
        scheduleDOMUpdate(() => {
          cpuPowerEl.textContent = displayText;
        });
      }
    } else {
      failedAttempts.cpuPower = 0;
      // CRITICAL: Cache last known good value to prevent flickering when value temporarily becomes 0
      let cpuPowerValue = previousValues.cpuPower || 0; // Keep current value if no new valid data
      
      // Only update if we have a valid non-zero value
      if (data.cpu_power && data.cpu_power > 0) {
        cpuPowerValue = data.cpu_power;
        previousValues.cpuPower = data.cpu_power; // Update cache
      }
      // If value is 0, keep the last known good value (don't update)
      
      const formatted = `${Math.round(cpuPowerValue)} W`;
      // Only update if value actually changed to prevent unnecessary DOM updates
      if (cpuPowerEl.textContent !== formatted) {
        scheduleDOMUpdate(() => {
          cpuPowerEl.textContent = formatted;
        });
      }
    }

    const gpuPowerEl = document.getElementById("gpu-power");
    if (!data.can_read_gpu_power) {
      failedAttempts.gpuPower++;
      // Only show hint after multiple failed attempts
      const shouldShowHint = failedAttempts.gpuPower >= FAILED_ATTEMPTS_THRESHOLD;
      const displayText = shouldShowHint ? "Requires root privileges" : "0 W";
      if (gpuPowerEl.textContent !== displayText) {
        scheduleDOMUpdate(() => {
          gpuPowerEl.textContent = displayText;
        });
      }
    } else {
      failedAttempts.gpuPower = 0;
      // CRITICAL: Cache last known good value to prevent flickering when value temporarily becomes 0
      let gpuPowerValue = previousValues.gpuPower || 0; // Keep current value if no new valid data
      
      // Only update if we have a valid non-zero value
      if (data.gpu_power && data.gpu_power > 0) {
        gpuPowerValue = data.gpu_power;
        previousValues.gpuPower = data.gpu_power; // Update cache
      }
      // If value is 0, keep the last known good value (don't update)
      
      const formatted = `${Math.round(gpuPowerValue)} W`;
      // Only update if value actually changed to prevent unnecessary DOM updates
      if (gpuPowerEl.textContent !== formatted) {
        scheduleDOMUpdate(() => {
          gpuPowerEl.textContent = formatted;
        });
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

// ============================================================================
// Monitoring Features (v0.1.0)
// ============================================================================

// Battery & Power Status Strip
function updateBatteryPower(cpuDetails) {
  const batteryLevel = document.getElementById('battery-level');
  const batteryStatus = document.getElementById('battery-status');
  const batteryIcon = document.getElementById('battery-icon');
  const powerValue = document.getElementById('power-value');
  const timeRemaining = document.getElementById('time-remaining');

  if (!batteryLevel) {
    console.warn('Battery level element not found - battery-power-strip might not exist in this theme');
    return; // Element might not exist in all themes
  }

  console.log('[Battery] Updating battery/power:', {
    has_battery: cpuDetails.has_battery,
    battery_level: cpuDetails.battery_level,
    is_charging: cpuDetails.is_charging,
    cpu_power: cpuDetails.cpu_power,
    gpu_power: cpuDetails.gpu_power
  });

  if (cpuDetails.has_battery) {
    const level = cpuDetails.battery_level || 0;
    const isCharging = cpuDetails.is_charging || false;
    
    if (batteryLevel) batteryLevel.textContent = `${level.toFixed(0)}%`;
    if (batteryStatus) batteryStatus.textContent = isCharging ? 'Charging' : 'Discharging';
    
    // Update battery icon SVG for charging state
    if (batteryIcon && batteryIcon.tagName === 'svg') {
      // Battery icon is now an SVG, we can update its appearance via CSS class
      if (isCharging) {
        batteryIcon.classList.add('charging');
        batteryIcon.setAttribute('title', 'Charging');
      } else {
        batteryIcon.classList.remove('charging');
        batteryIcon.setAttribute('title', 'Battery');
      }
      
      // Update battery fill level visually (optional - can show battery level in icon)
      const batteryRect = batteryIcon.querySelector('rect');
      if (batteryRect) {
        // Calculate fill width based on battery level (16px total width, 2px padding)
        const fillWidth = (level / 100) * 12; // 12px usable width (16 - 2*2 padding)
        // We could add a fill rectangle here if desired, but keeping it minimal for now
      }
    }
    
    const totalPower = (cpuDetails.cpu_power || 0) + (cpuDetails.gpu_power || 0);
    if (powerValue) {
      if (totalPower > 0) {
        powerValue.textContent = `${totalPower.toFixed(1)} W`;
      } else {
        powerValue.textContent = '-- W';
      }
    }
    
    if (timeRemaining && !isCharging && level > 0 && totalPower > 0) {
      const hours = (level / 100) * 10 / (totalPower / 20); // Simplified estimate
      timeRemaining.textContent = `~${hours.toFixed(1)}h remaining`;
    } else if (timeRemaining) {
      timeRemaining.textContent = '';
    }
  } else {
    // No battery (desktop Mac)
    if (batteryLevel) batteryLevel.textContent = 'N/A';
    if (batteryStatus) batteryStatus.textContent = 'No battery';
    if (batteryIcon && batteryIcon.tagName === 'svg') {
      batteryIcon.classList.add('no-battery');
      batteryIcon.setAttribute('title', 'No battery');
    }
    const totalPower = (cpuDetails.cpu_power || 0) + (cpuDetails.gpu_power || 0);
    if (powerValue) {
      if (totalPower > 0) {
        powerValue.textContent = `${totalPower.toFixed(1)} W`;
      } else {
        powerValue.textContent = '-- W';
      }
    }
    if (timeRemaining) timeRemaining.textContent = '';
  }
}

// Monitors Section
let monitorsCollapsed = true;
let monitorsUpdateInterval = null;

// Cache for monitor status data (to avoid polling backend when opening settings)
const monitorStatusCache = new Map(); // Map<monitorId, {is_up, response_time_ms, error, checked_at}>

// Monitor history storage (last 24 hours)
// Structure: Map<monitorId, Array<{timestamp: number, is_up: boolean}>>
const monitorHistory = new Map();

// Initialize monitor history from localStorage
function initMonitorHistory() {
  try {
    const stored = localStorage.getItem('monitor_history');
    if (stored) {
      const parsed = JSON.parse(stored);
      const now = Date.now();
      const twentyFourHoursAgo = now - (24 * 60 * 60 * 1000);
      
      // Filter out entries older than 24 hours
      for (const [monitorId, history] of Object.entries(parsed)) {
        const filtered = history.filter(entry => entry.timestamp >= twentyFourHoursAgo);
        if (filtered.length > 0) {
          monitorHistory.set(monitorId, filtered);
        }
      }
    }
  } catch (err) {
    console.error('Failed to load monitor history:', err);
  }
}

// Save monitor history to localStorage
function saveMonitorHistory() {
  try {
    const now = Date.now();
    const twentyFourHoursAgo = now - (24 * 60 * 60 * 1000);
    
    // Clean up old entries before saving
    const toSave = {};
    for (const [monitorId, history] of monitorHistory.entries()) {
      const filtered = history.filter(entry => entry.timestamp >= twentyFourHoursAgo);
      if (filtered.length > 0) {
        toSave[monitorId] = filtered;
      }
    }
    
    localStorage.setItem('monitor_history', JSON.stringify(toSave));
  } catch (err) {
    console.error('Failed to save monitor history:', err);
  }
}

// Add a history entry for a monitor
function addMonitorHistoryEntry(monitorId, isUp) {
  const now = Date.now();
  const twentyFourHoursAgo = now - (24 * 60 * 60 * 1000);
  
  if (!monitorHistory.has(monitorId)) {
    monitorHistory.set(monitorId, []);
  }
  
  const history = monitorHistory.get(monitorId);
  history.push({ timestamp: now, is_up: isUp });
  
  // Remove entries older than 24 hours
  const filtered = history.filter(entry => entry.timestamp >= twentyFourHoursAgo);
  monitorHistory.set(monitorId, filtered);
  
  // Save to localStorage (throttled)
  if (!window._monitorHistorySaveTimeout) {
    window._monitorHistorySaveTimeout = setTimeout(() => {
      saveMonitorHistory();
      window._monitorHistorySaveTimeout = null;
    }, 1000);
  }
}

// Get monitor history for the last 24 hours
function getMonitorHistory(monitorId) {
  const now = Date.now();
  const twentyFourHoursAgo = now - (24 * 60 * 60 * 1000);
  
  const history = monitorHistory.get(monitorId) || [];
  return history.filter(entry => entry.timestamp >= twentyFourHoursAgo);
}

function getMonitorsCollapsedState() {
  // Get saved state from localStorage, default to true (collapsed)
  const saved = localStorage.getItem('monitors_collapsed');
  return saved !== null ? saved === 'true' : true;
}

function saveMonitorsCollapsedState(collapsed) {
  localStorage.setItem('monitors_collapsed', collapsed.toString());
}

function initMonitorsSection() {
  const header = document.getElementById('monitors-header');
  const content = document.getElementById('monitors-content');
  const section = document.querySelector('.monitors-section');

  if (!header || !content) {
    console.warn('Monitors section elements not found');
    return;
  }
  
  console.log('Initializing monitors section', { header: !!header, content: !!content });

  // Initialize monitor history from localStorage
  initMonitorHistory();

  // Always load monitors to calculate height, even when collapsed
  loadMonitors().then(() => {
    updateMonitorsHeight();
  });
  updateMonitorsSummary();
  
  // Restore saved state
  monitorsCollapsed = getMonitorsCollapsedState();
  updateMonitorsStatusDot();
  const divider = document.getElementById('monitors-ollama-divider');
  if (monitorsCollapsed) {
    content.classList.add('collapsed');
    if (section) {
      section.classList.add('collapsed');
    }
    if (divider) {
      divider.style.display = 'none';
    }
  } else {
    content.classList.remove('collapsed');
    if (section) {
      section.classList.remove('collapsed');
    }
    if (divider) {
      divider.style.display = '';
    }
    // Update monitors every 30 seconds
    if (!monitorsUpdateInterval) {
      monitorsUpdateInterval = setInterval(() => {
        updateMonitorsSummary();
        loadMonitors().then(() => {
          updateMonitorsHeight();
        });
      }, 30000);
    }
  }

  // Make header clickable to toggle collapse/expand
  header.addEventListener('click', (e) => {
    // Don't toggle if clicking on menu button (it opens settings)
    const menuBtn = document.getElementById('monitors-menu-btn');
    
    // Check if click originated from within the menu button
    const clickedElement = e.target;
    if (menuBtn && (clickedElement === menuBtn || clickedElement.closest && clickedElement.closest('#monitors-menu-btn'))) {
      return; // Let menu button handle its own click (opens settings)
    }
    
    // Toggle collapse state when clicking anywhere else on the header (including title text)
    e.stopPropagation(); // Prevent any parent handlers
    monitorsCollapsed = !monitorsCollapsed;
    saveMonitorsCollapsedState(monitorsCollapsed);
    
    const section = document.querySelector('.monitors-section');
    const divider = document.getElementById('monitors-ollama-divider');
    
    if (monitorsCollapsed) {
      content.classList.add('collapsed');
      if (section) {
        section.classList.add('collapsed');
      }
      if (divider) {
        divider.style.display = 'none';
      }
      if (monitorsUpdateInterval) {
        clearInterval(monitorsUpdateInterval);
        monitorsUpdateInterval = null;
      }
    } else {
      content.classList.remove('collapsed');
      if (section) {
        section.classList.remove('collapsed');
      }
      if (divider) {
        divider.style.display = '';
      }
      // Just update height based on existing content - don't trigger backend calls
      // The interval will handle data updates
      updateMonitorsHeight();
      
      // Start interval if not already running (but don't call immediately)
      if (!monitorsUpdateInterval) {
        monitorsUpdateInterval = setInterval(() => {
          updateMonitorsSummary();
          loadMonitors().then(() => {
            updateMonitorsHeight();
          });
        }, 30000);
      }
    }
    updateMonitorsStatusDot();
    
  });

  // Initialize menu button - directly opens settings
  const menuBtn = document.getElementById('monitors-menu-btn');
  
  if (menuBtn) {
    // Clicking "..." button directly opens settings
    menuBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      e.preventDefault();
      showMonitorsSettings();
    });
  }
  
  // Initialize settings popover
  const settingsPopover = document.getElementById('monitors-settings-popover');
  const settingsClose = document.getElementById('monitors-settings-close');
  const settingsList = document.getElementById('monitors-settings-list');
  const addBtn = document.getElementById('monitors-add-btn');
  const addForm = document.getElementById('add-monitor-form');
  const addCancel = document.getElementById('monitors-add-cancel');
  const addSave = document.getElementById('monitors-add-save');
  const urlInput = document.getElementById('monitor-url-input');
  
  if (settingsClose) {
    settingsClose.addEventListener('click', () => {
      if (window.closeMonitorsSettings) {
        window.closeMonitorsSettings();
      }
    });
  }
  
  if (addBtn) {
    addBtn.addEventListener('click', () => {
      if (addForm) {
        addForm.style.display = 'block';
        if (urlInput) {
          urlInput.value = 'https://www.amvara.de/';
          urlInput.focus();
        }
      }
    });
  }
  
  if (addCancel) {
    addCancel.addEventListener('click', () => {
      if (addForm) addForm.style.display = 'none';
      if (urlInput) urlInput.value = 'https://www.amvara.de/';
    });
  }
  
  if (addSave && urlInput) {
    addSave.addEventListener('click', async () => {
      let url = urlInput.value.trim();
      if (!url) {
        alert('Please enter a URL');
        return;
      }
      
      // Add https:// if no protocol specified
      if (!url.match(/^https?:\/\//i)) {
        url = 'https://' + url;
      }
      
      try {
        const urlObj = new URL(url);
        const id = `monitor_${Date.now()}`;
        const name = urlObj.hostname;
        
        await invoke('add_website_monitor', {
          request: {
            id,
            name,
            url,
            timeout_secs: 10,
            check_interval_secs: 30, // 30 seconds like UptimeRobot
            verify_ssl: true
          }
        });
        
        console.log('Monitor added successfully');
        if (addForm) addForm.style.display = 'none';
        if (urlInput) urlInput.value = 'https://www.amvara.de/';
        // After adding, load monitors will update the cache
        await loadMonitors();
        await updateMonitorsSummary();
        await refreshMonitorsSettingsList();
      } catch (err) {
        console.error('Failed to add monitor:', err);
        alert(`Failed to add monitor: ${err}`);
      }
    });
  }
  
  // Close popover on backdrop click
  if (settingsPopover) {
    settingsPopover.addEventListener('click', (e) => {
      if (e.target === settingsPopover) {
        closeMonitorsSettings();
      }
    });
  }
  
  // Close monitors settings function
  window.closeMonitorsSettings = function() {
    const popover = document.getElementById('monitors-settings-popover');
    const addForm = document.getElementById('add-monitor-form');
    if (popover) popover.style.display = 'none';
    if (addForm) addForm.style.display = 'none';
  };
}

// Global ESC key handler for all popovers
document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape' || e.key === 'Esc') {
    // Close monitors settings popover if visible
    const monitorsPopover = document.getElementById('monitors-settings-popover');
    if (monitorsPopover && monitorsPopover.style.display !== 'none') {
      if (window.closeMonitorsSettings) {
        window.closeMonitorsSettings();
      }
      return;
    }
    
    // Close Ollama settings popover if visible
    const ollamaPopover = document.getElementById('ollama-settings-popover');
    if (ollamaPopover && ollamaPopover.style.display !== 'none') {
      ollamaPopover.style.display = 'none';
    }
  }
});

async function showMonitorsSettings() {
  const popover = document.getElementById('monitors-settings-popover');
  if (popover) {
    popover.style.display = 'flex';
    await refreshMonitorsSettingsList();
  }
}

async function refreshMonitorsSettingsList() {
  const settingsList = document.getElementById('monitors-settings-list');
  if (!settingsList) return;
  
  settingsList.innerHTML = '';
  
  try {
    const monitorIds = await invoke('list_monitors');
    
    if (monitorIds.length === 0) {
      settingsList.innerHTML = '<div style="padding: 20px; text-align: center; color: var(--muted);">No monitors configured</div>';
      return;
    }
    
    for (const monitorId of monitorIds) {
      try {
        // Get monitor details including URL
        let monitorUrl = monitorId; // Fallback to ID if details not available
        try {
          const details = await invoke('get_monitor_details', { monitorId });
          if (details.url) {
            monitorUrl = details.url;
          }
        } catch (e) {
          console.warn(`Failed to get details for monitor ${monitorId}:`, e);
        }
        
        // Use cached status data instead of polling backend
        let statusInfo = '';
        const cachedStatus = monitorStatusCache.get(monitorId);
        if (cachedStatus) {
          const statusText = cachedStatus.is_up ? '✓ Up' : '✗ Down';
          const timeText = cachedStatus.response_time_ms ? ` · ${cachedStatus.response_time_ms}ms` : '';
          statusInfo = ` · ${statusText}${timeText}`;
        }
        
        const item = document.createElement('div');
        item.className = 'monitor-settings-item';
        
        const info = document.createElement('div');
        info.className = 'monitor-settings-item-info';
        info.innerHTML = `
          <div class="monitor-settings-item-name">${monitorUrl}${statusInfo}</div>
          <div class="monitor-settings-item-url">${monitorUrl}</div>
        `;
        
        const actions = document.createElement('div');
        actions.className = 'monitor-settings-item-actions';
        
        const removeBtn = document.createElement('button');
        removeBtn.className = 'monitor-remove-btn';
        removeBtn.textContent = 'Remove';
        removeBtn.addEventListener('click', async () => {
          if (confirm(`Remove monitor "${monitorUrl}"?`)) {
            try {
              await invoke('remove_monitor', { monitorId });
              // Remove from cache
              monitorStatusCache.delete(monitorId);
              await refreshMonitorsSettingsList();
              await loadMonitors();
              await updateMonitorsSummary();
            } catch (err) {
              console.error('Failed to remove monitor:', err);
              alert(`Failed to remove monitor: ${err}`);
            }
          }
        });
        
        actions.appendChild(removeBtn);
        item.appendChild(info);
        item.appendChild(actions);
        settingsList.appendChild(item);
      } catch (err) {
        console.error(`Failed to load monitor ${monitorId}:`, err);
      }
    }
  } catch (err) {
    console.error('Failed to refresh monitors list:', err);
    settingsList.innerHTML = '<div style="padding: 20px; text-align: center; color: #ff3b30;">Error loading monitors</div>';
  }
}

async function updateMonitorsSummary() {
  const summaryText = document.getElementById('monitors-summary-text');
  if (!summaryText) return;

  try {
    const monitorIds = await invoke('list_monitors');
    
    if (monitorIds.length === 0) {
      summaryText.textContent = 'No monitors configured';
      updateMonitorsIconStatus(false, 0, 0); // No monitors = not all up
      return;
    }

    let upCount = 0;
    let totalResponseTime = 0;
    let responseTimeCount = 0;

    for (const monitorId of monitorIds) {
      try {
        const status = await invoke('check_monitor', { monitorId });
        // Cache the status for use in settings view
        monitorStatusCache.set(monitorId, status);
        if (status.is_up) upCount++;
        if (status.response_time_ms) {
          totalResponseTime += status.response_time_ms;
          responseTimeCount++;
        }
      } catch (err) {
        console.error(`Failed to check monitor ${monitorId}:`, err);
      }
    }

    const avgResponseTime = responseTimeCount > 0 
      ? Math.round(totalResponseTime / responseTimeCount)
      : 0;

    summaryText.textContent = 
      `${upCount} / ${monitorIds.length} sites up · Avg ${avgResponseTime} ms`;
    
    // Update icon status: green if all monitors are up
    const allUp = upCount === monitorIds.length && monitorIds.length > 0;
    updateMonitorsIconStatus(allUp, upCount, monitorIds.length);
  } catch (err) {
    console.error('Failed to update monitors summary:', err);
    updateMonitorsIconStatus(false, 0, 0);
  }
}

async function loadMonitors() {
  const monitorsList = document.getElementById('monitors-list');
  if (!monitorsList) return;

  try {
    const monitorIds = await invoke('list_monitors');
    
    // Create a map of existing monitor items by their data-monitor-id attribute
    const existingItems = new Map();
    monitorsList.querySelectorAll('.monitor-item').forEach(item => {
      const monitorId = item.getAttribute('data-monitor-id');
      if (monitorId) {
        existingItems.set(monitorId, item);
      }
    });
    
    // Track which monitor IDs we've processed
    const processedIds = new Set();
    
    for (const monitorId of monitorIds) {
      processedIds.add(monitorId);
      
      try {
        // Get monitor details to fetch URL
        let monitorUrl = monitorId; // Fallback to ID if details not available
        try {
          const details = await invoke('get_monitor_details', { monitorId });
          if (details.url) {
            monitorUrl = details.url;
          }
        } catch (e) {
          console.warn(`Failed to get details for monitor ${monitorId}:`, e);
        }
        
        const status = await invoke('check_monitor', { monitorId });
        // Cache the status for use in settings view
        monitorStatusCache.set(monitorId, status);
        
        // Add to history
        addMonitorHistoryEntry(monitorId, status.is_up);
        
        // Check if we already have an item for this monitor
        const existingItem = existingItems.get(monitorId);
        if (existingItem) {
          // Update existing item in place instead of recreating
          updateMonitorItem(existingItem, monitorId, monitorUrl, status);
        } else {
          // Create new item if it doesn't exist
          const monitorItem = createMonitorItem(monitorId, monitorUrl, status);
          monitorsList.appendChild(monitorItem);
        }
      } catch (err) {
        console.error(`Failed to load monitor ${monitorId}:`, err);
      }
    }
    
    // Remove any monitor items that no longer exist
    existingItems.forEach((item, monitorId) => {
      if (!processedIds.has(monitorId)) {
        item.remove();
        monitorStatusCache.delete(monitorId);
      }
    });
    
    // Update icon status based on all monitors
    // Count how many are up
    let upCount = 0;
    for (const monitorId of monitorIds) {
      const status = monitorStatusCache.get(monitorId);
      if (status && status.is_up) upCount++;
    }
    const allUp = upCount === monitorIds.length && monitorIds.length > 0;
    updateMonitorsIconStatus(allUp, upCount, monitorIds.length);
    
    // Update height after loading monitors
    updateMonitorsHeight();
  } catch (err) {
    console.error('Failed to load monitors:', err);
    updateMonitorsIconStatus(false, 0, 0);
  }
}

function updateMonitorsHeight() {
  const monitorsList = document.getElementById('monitors-list');
  const monitorsContent = document.getElementById('monitors-content');
  if (!monitorsList || !monitorsContent) return;
  
  // Calculate height needed: summary (~40px) + each monitor item (~40px including margin)
  const monitorItems = monitorsList.querySelectorAll('.monitor-item');
  const itemHeight = 40; // Approximate height per monitor item (padding + margin + content)
  const summaryHeight = 40; // Approximate height of summary
  const listMargin = 12; // margin-top of monitors-list
  
  // When collapsed, hide everything - don't set any heights
  if (monitorsCollapsed) {
    monitorsContent.style.minHeight = '';
    monitorsList.style.display = '';
    monitorsList.style.visibility = '';
    monitorsList.style.height = '';
    monitorsList.style.overflow = '';
    monitorsList.style.margin = '';
    monitorsList.style.padding = '';
    return;
  }
  
  const totalHeight = summaryHeight + (monitorItems.length > 0 ? listMargin + (monitorItems.length * itemHeight) : 0);
  
  // Set min-height to reserve space and prevent layout shifts
  // This ensures the section always takes up the same space regardless of collapse state
  monitorsContent.style.minHeight = `${totalHeight}px`;
  
  // Show the list when expanded
  monitorsList.style.display = 'block';
  monitorsList.style.visibility = 'visible';
  monitorsList.style.height = 'auto';
  monitorsList.style.overflow = 'visible';
  monitorsList.style.margin = `${listMargin}px 0 0 0`;
  monitorsList.style.padding = '';
}

function createMonitorItem(monitorId, monitorUrl, status) {
  const item = document.createElement('div');
  item.className = 'monitor-item';
  item.setAttribute('data-monitor-id', monitorId);
  
  // Create header container for status indicator and info
  const header = document.createElement('div');
  header.className = 'monitor-item-header';
  
  const statusIndicator = document.createElement('div');
  statusIndicator.className = 'status-indicator';
  if (!status.is_up) {
    statusIndicator.classList.add('down');
  }

  const info = document.createElement('div');
  info.className = 'monitor-info';
  
  // Format: "URL · 240ms" (matching summary format)
  const responseTimeText = status.response_time_ms ? `${status.response_time_ms}ms` : '--';
  const errorText = status.error ? ` · ${status.error}` : '';
  info.innerHTML = `
    <div>${monitorUrl} · ${responseTimeText}${errorText}</div>
  `;

  header.appendChild(statusIndicator);
  header.appendChild(info);
  
  // Add history visualization
  const historyContainer = document.createElement('div');
  historyContainer.className = 'monitor-history';
  historyContainer.setAttribute('data-monitor-id', monitorId);
  updateMonitorHistory(historyContainer, monitorId);

  item.appendChild(header);
  item.appendChild(historyContainer);
  
  return item;
}

function updateMonitorItem(item, monitorId, monitorUrl, status) {
  // Update status indicator
  const statusIndicator = item.querySelector('.status-indicator');
  if (statusIndicator) {
    if (status.is_up) {
      statusIndicator.classList.remove('down');
    } else {
      statusIndicator.classList.add('down');
    }
  }
  
  // Update info text
  const info = item.querySelector('.monitor-info');
  if (info) {
    const responseTimeText = status.response_time_ms ? `${status.response_time_ms}ms` : '--';
    const errorText = status.error ? ` · ${status.error}` : '';
    const infoDiv = info.querySelector('div');
    if (infoDiv) {
      infoDiv.textContent = `${monitorUrl} · ${responseTimeText}${errorText}`;
    } else {
      info.innerHTML = `<div>${monitorUrl} · ${responseTimeText}${errorText}</div>`;
    }
  }
  
  // Update history visualization
  let historyContainer = item.querySelector('.monitor-history');
  if (!historyContainer) {
    historyContainer = document.createElement('div');
    historyContainer.className = 'monitor-history';
    historyContainer.setAttribute('data-monitor-id', monitorId);
    // Insert after the header
    const header = item.querySelector('.monitor-item-header');
    if (header) {
      header.after(historyContainer);
    } else {
      item.appendChild(historyContainer);
    }
  }
  updateMonitorHistory(historyContainer, monitorId);
}

// Update the history visualization for a monitor
function updateMonitorHistory(container, monitorId) {
  const history = getMonitorHistory(monitorId);
  
  if (history.length === 0) {
    container.innerHTML = '';
    return;
  }
  
  // Create a visualization with colored lines
  // We'll show up to 288 lines (24 hours * 60 minutes / 5 minutes per check)
  // But we'll scale based on actual data points
  const maxLines = 288; // 24 hours * 12 checks per hour (5 min intervals)
  const lines = [];
  
  // Sort history by timestamp (oldest first)
  const sortedHistory = [...history].sort((a, b) => a.timestamp - b.timestamp);
  
  // If we have more data points than maxLines, sample them
  let dataPoints = sortedHistory;
  if (sortedHistory.length > maxLines) {
    // Sample evenly
    const step = Math.floor(sortedHistory.length / maxLines);
    dataPoints = [];
    for (let i = 0; i < sortedHistory.length; i += step) {
      dataPoints.push(sortedHistory[i]);
    }
    // Always include the last point
    if (dataPoints[dataPoints.length - 1] !== sortedHistory[sortedHistory.length - 1]) {
      dataPoints.push(sortedHistory[sortedHistory.length - 1]);
    }
  }
  
  // Create lines for each data point
  dataPoints.forEach(entry => {
    const line = document.createElement('span');
    line.className = 'monitor-history-line';
    line.classList.add(entry.is_up ? 'up' : 'down');
    line.title = new Date(entry.timestamp).toLocaleString();
    lines.push(line);
  });
  
  container.innerHTML = '';
  lines.forEach(line => container.appendChild(line));
}

function updateMonitorsStatusDot() {
  const statusDotContainer = document.querySelector('.monitors-status-dot-container');
  if (statusDotContainer) {
    if (monitorsCollapsed) {
      statusDotContainer.classList.add('visible');
    } else {
      statusDotContainer.classList.remove('visible');
    }
  }
}

function updateMonitorsIconStatus(allUp, upCount, totalCount) {
  const monitorsIcon = document.getElementById('icon-monitors');
  if (!monitorsIcon) return;
  
  if (allUp && totalCount > 0) {
    // All monitors are up - make icon green
    monitorsIcon.classList.add('status-good');
    monitorsIcon.classList.remove('status-bad');
  } else if (totalCount > 0) {
    // Some monitors are down - keep default/grey color
    monitorsIcon.classList.remove('status-good');
    monitorsIcon.classList.remove('status-bad');
  } else {
    // No monitors configured - keep default/grey color
    monitorsIcon.classList.remove('status-good');
    monitorsIcon.classList.remove('status-bad');
  }
}

function updateOllamaIconStatus(status) {
  const ollamaIcon = document.getElementById('icon-ollama');
  if (!ollamaIcon) {
    console.warn('[CPU] Ollama icon not found when updating status');
    return;
  }
  
  // Remove all status classes first
  ollamaIcon.classList.remove('status-good', 'status-warning');
  
  if (status === true || status === 'connected') {
    // Connection is good - make icon green
    ollamaIcon.classList.add('status-good');
    console.log('[CPU] Ollama icon set to green (connected)');
  } else if (status === 'error' || status === 'unavailable') {
    // Ollama not installed/not running - make icon yellow
    ollamaIcon.classList.add('status-warning');
    console.log('[CPU] Ollama icon set to yellow (not available/not running)');
  } else {
    // Unknown/checking - keep default/grey color
    console.log('[CPU] Ollama icon set to default (unknown/checking)');
  }
}

function showAddMonitorDialog() {
  console.log('showAddMonitorDialog called');
  let url = prompt('Enter website URL to monitor:');
  if (!url) {
    console.log('User cancelled or empty URL');
    return;
  }
  
  // Add https:// if no protocol specified
  if (!url.match(/^https?:\/\//i)) {
    url = 'https://' + url;
  }
  
  try {
    const urlObj = new URL(url);
    const id = `monitor_${Date.now()}`;
    const name = urlObj.hostname;
    
    console.log('Adding monitor:', { id, name, url });
    
    invoke('add_website_monitor', {
      request: {
        id,
        name,
        url,
        timeout_secs: 10,
        check_interval_secs: 60,
        verify_ssl: true
      }
    })
    .then(() => {
      console.log('Monitor added successfully');
      loadMonitors();
      updateMonitorsSummary();
    })
    .catch(err => {
      console.error('Failed to add monitor:', err);
      alert(`Failed to add monitor: ${err}`);
    });
  } catch (e) {
    console.error('Invalid URL format:', e);
    alert(`Invalid URL format: ${e.message}`);
  }
}

// ============================================================================
// Ollama Chat Section - UI Management Only
// ============================================================================
// NOTE: All Ollama functionality has been moved to src/ollama.js
// This section only handles CPU window-specific UI (collapsing, model dropdown, etc.)
// Actual chat communication is handled by window.Ollama.* functions
// ============================================================================

let ollamaCollapsed = true;

// ============================================================================
// System Prompt Management (UI-specific, stays in cpu.js)
// ============================================================================
// These functions manage system prompt UI in the CPU window
// The actual prompt is used by ollama.js via window.Ollama.getSystemPrompt()
// ============================================================================

const DEFAULT_SYSTEM_PROMPT = 'You are a helpful assistant that answers questions about system metrics and monitoring.';

function getSystemPrompt() {
  const saved = localStorage.getItem('ollama_system_prompt');
  return saved || DEFAULT_SYSTEM_PROMPT;
}

function saveSystemPrompt(prompt) {
  if (prompt && prompt.trim() && prompt !== DEFAULT_SYSTEM_PROMPT) {
    localStorage.setItem('ollama_system_prompt', prompt.trim());
  } else {
    localStorage.removeItem('ollama_system_prompt');
  }
}

function initOllamaSection() {
  const header = document.getElementById('ollama-header');
  const content = document.getElementById('ollama-content');
  const chatInput = document.getElementById('chat-input');
  const chatSendBtn = document.getElementById('chat-send-btn');
  const modelSelect = document.getElementById('ollama-model-select');
  const connectionIndicator = document.getElementById('ollama-connection-indicator');
  const modelText = document.getElementById('ollama-model-text');

  if (!header || !content) return;

  // Restore saved state
  const savedState = localStorage.getItem('ollama_collapsed');
  ollamaCollapsed = savedState !== null ? savedState === 'true' : true;
  const section = document.querySelector('.ollama-section');
  const divider = document.getElementById('monitors-ollama-divider');
  
  if (ollamaCollapsed) {
    content.classList.add('collapsed');
    if (section) {
      section.classList.add('collapsed');
    }
    if (divider) {
      divider.style.display = 'none';
    }
  } else {
    content.classList.remove('collapsed');
    if (section) {
      section.classList.remove('collapsed');
    }
    if (divider) {
      divider.style.display = '';
    }
  }

  // Connection indicator click handler (only when not connected)
  if (connectionIndicator) {
    connectionIndicator.addEventListener('click', (e) => {
      e.stopPropagation();
      if (!connectionIndicator.classList.contains('connected')) {
        if (window.Ollama) {
          window.Ollama.showUrlDialog();
        } else {
          showOllamaUrlDialog(); // Fallback
        }
      }
    });
  }

  // Model text click handler - show dropdown
  if (modelText) {
    modelText.addEventListener('click', (e) => {
      e.stopPropagation();
      toggleModelDropdown();
    });
  }

  // Model selector change handler
  if (modelSelect) {
    modelSelect.addEventListener('change', async (e) => {
      const selectedModel = e.target.value;
      if (selectedModel) {
        await updateOllamaModel(selectedModel);
        updateModelText(selectedModel);
        hideModelDropdown();
      }
    });

    // Close dropdown when clicking outside
    document.addEventListener('click', (e) => {
      if (modelSelect && modelSelect.style.display !== 'none') {
        if (!modelSelect.contains(e.target) && e.target !== modelText) {
          hideModelDropdown();
        }
      }
    });
  }

  header.addEventListener('click', (e) => {
    // Don't toggle if clicking on controls
    const menuBtn = document.getElementById('ollama-menu-btn');
    const menu = document.getElementById('ollama-menu');
    if (e.target === connectionIndicator || 
        e.target === modelText || 
        e.target === menuBtn ||
        connectionIndicator?.contains(e.target) ||
        modelText?.contains(e.target) ||
        modelSelect?.contains(e.target) ||
        menuBtn?.contains(e.target) ||
        menu?.contains(e.target)) {
      return;
    }
    
    ollamaCollapsed = !ollamaCollapsed;
    const section = document.querySelector('.ollama-section');
    const divider = document.getElementById('monitors-ollama-divider');
    
    if (ollamaCollapsed) {
      content.classList.add('collapsed');
      if (section) {
        section.classList.add('collapsed');
      }
      if (divider) {
        divider.style.display = 'none';
      }
      const chat = document.getElementById('ollama-chat');
      if (chat) chat.style.display = 'none';
      hideModelDropdown();
    } else {
      content.classList.remove('collapsed');
      if (section) {
        section.classList.remove('collapsed');
      }
      if (divider) {
        divider.style.display = '';
      }
      const chat = document.getElementById('ollama-chat');
      if (chat) chat.style.display = 'block';
      checkOllamaConnection().then((connected) => {
        // Update icon based on connection result
        if (connected) {
          updateOllamaIconStatus('connected');
        } else {
          // Double-check the connection indicator after section is expanded
          setTimeout(() => {
            const indicator = document.getElementById('ollama-connection-indicator');
            if (indicator) {
              const isConnected = indicator.classList.contains('connected');
              updateOllamaIconStatus(isConnected ? 'connected' : 'unknown');
            } else {
              updateOllamaIconStatus('unknown');
            }
          }, 200);
        }
      }).catch((err) => {
        // Connection check failed - Ollama not available
        console.error('[CPU] Ollama connection check failed:', err);
        updateOllamaIconStatus('error');
      });
    }
    // Update menu text
    const menuCollapse = document.getElementById('ollama-menu-collapse');
    if (menuCollapse) {
      menuCollapse.textContent = ollamaCollapsed ? 'Expand' : 'Collapse';
    }
    localStorage.setItem('ollama_collapsed', ollamaCollapsed.toString());
  });

  // Chat event listeners - handled by Ollama module
  // Initialize Ollama module listeners if available
  if (window.Ollama) {
    window.Ollama.initListeners();
  }

  // Check connection on load
  if (window.Ollama) {
    checkOllamaConnection().then(() => {
      // Double-check the connection indicator after initial load
      setTimeout(() => {
        const indicator = document.getElementById('ollama-connection-indicator');
        if (indicator && indicator.classList.contains('connected')) {
          updateOllamaIconStatus(true);
        }
      }, 300);
    });
  } else {
    // If Ollama module not available, ensure icon is not green
    updateOllamaIconStatus(false);
  }
  
  // Initialize menu button
  const menuBtn = document.getElementById('ollama-menu-btn');
  const menu = document.getElementById('ollama-menu');
  const menuCollapse = document.getElementById('ollama-menu-collapse');
  const menuSettings = document.getElementById('ollama-menu-settings');
  
  if (menuBtn && menu) {
    // Update menu text based on current state
    const updateOllamaMenuText = () => {
      const menuCollapse = document.getElementById('ollama-menu-collapse');
      if (menuCollapse) {
        menuCollapse.textContent = ollamaCollapsed ? 'Expand' : 'Collapse';
      }
    };
    updateOllamaMenuText();
    
    // Toggle menu on button click
    menuBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      e.preventDefault();
      const isVisible = menu.style.display !== 'none';
      menu.style.display = isVisible ? 'none' : 'block';
      
      // Position menu right next to button
      if (!isVisible) {
        // Use requestAnimationFrame to ensure layout is stable before positioning
        requestAnimationFrame(() => {
          const rect = menuBtn.getBoundingClientRect();
          menu.style.position = 'fixed';
          // Align menu top with button top, position immediately adjacent to button
          menu.style.top = `${rect.top}px`;
          menu.style.left = `${rect.right + 2}px`;
          menu.style.transform = 'none'; // Ensure no transforms interfere
          updateOllamaMenuText(); // Update text when opening menu
        });
      }
    });
    
    // Close menu when clicking outside
    document.addEventListener('click', (e) => {
      if (menu && !menu.contains(e.target) && !menuBtn.contains(e.target)) {
        menu.style.display = 'none';
      }
    });
  }
  
  if (menuCollapse) {
    // Update menu text based on current state
    const updateMenuText = () => {
      menuCollapse.textContent = ollamaCollapsed ? 'Expand' : 'Collapse';
    };
    updateMenuText();
    
      menuCollapse.addEventListener('click', (e) => {
      e.stopPropagation();
      e.preventDefault();
      menu.style.display = 'none';
      // Toggle collapse
      ollamaCollapsed = !ollamaCollapsed;
      const content = document.getElementById('ollama-content');
      const chat = document.getElementById('ollama-chat');
      const section = document.querySelector('.ollama-section');
      const divider = document.getElementById('monitors-ollama-divider');
      
      if (content) {
        if (ollamaCollapsed) {
          content.classList.add('collapsed');
          if (section) {
            section.classList.add('collapsed');
          }
          if (divider) {
            divider.style.display = 'none';
          }
          if (chat) chat.style.display = 'none';
        } else {
          content.classList.remove('collapsed');
          if (section) {
            section.classList.remove('collapsed');
          }
          if (divider) {
            divider.style.display = '';
          }
          if (chat) chat.style.display = 'block';
          checkOllamaConnection();
        }
        localStorage.setItem('ollama_collapsed', ollamaCollapsed.toString());
        updateMenuText();
      }
    });
  }
  
  if (menuSettings) {
    menuSettings.addEventListener('click', (e) => {
      e.stopPropagation();
      e.preventDefault();
      menu.style.display = 'none';
      showSystemPromptSettings();
    });
  }
  
  // Initialize settings popover
  const settingsPopover = document.getElementById('ollama-settings-popover');
  const settingsClose = document.getElementById('ollama-settings-close');
  const settingsSave = document.getElementById('ollama-settings-save');
  const settingsReset = document.getElementById('ollama-settings-reset');
  const systemPromptTextarea = document.getElementById('ollama-system-prompt');
  
  if (settingsClose) {
    settingsClose.addEventListener('click', () => {
      if (settingsPopover) settingsPopover.style.display = 'none';
    });
  }
  
  if (settingsSave) {
    settingsSave.addEventListener('click', () => {
      if (systemPromptTextarea) {
        const prompt = systemPromptTextarea.value.trim();
        saveSystemPrompt(prompt || DEFAULT_SYSTEM_PROMPT);
        if (settingsPopover) settingsPopover.style.display = 'none';
        console.log('[Ollama] System prompt saved');
      }
    });
  }
  
  if (settingsReset) {
    settingsReset.addEventListener('click', () => {
      if (systemPromptTextarea) {
        systemPromptTextarea.value = DEFAULT_SYSTEM_PROMPT;
      }
    });
  }
  
  // Close popover on backdrop click
  if (settingsPopover) {
    settingsPopover.addEventListener('click', (e) => {
      if (e.target === settingsPopover) {
        settingsPopover.style.display = 'none';
      }
    });
  }
  
  // Load saved system prompt into textarea if it exists
  if (systemPromptTextarea) {
    systemPromptTextarea.value = getSystemPrompt();
  }
}

function showSystemPromptSettings() {
  const popover = document.getElementById('ollama-settings-popover');
  const textarea = document.getElementById('ollama-system-prompt');
  if (popover && textarea) {
    textarea.value = getSystemPrompt();
    popover.style.display = 'flex';
    // Focus textarea after a short delay
    setTimeout(() => textarea.focus(), 100);
  }
}

function getDefaultModel() {
  // Get saved model from localStorage, or use default
  const saved = localStorage.getItem('ollama_model');
  if (saved) {
    return saved;
  }
  // Default to qwen2.5-coder (or qwen2.5:7b-coder, depending on what's available)
  return 'qwen2.5-coder';
}

function saveSelectedModel(model) {
  localStorage.setItem('ollama_model', model);
}

function toggleModelDropdown() {
  const modelSelect = document.getElementById('ollama-model-select');
  const modelText = document.getElementById('ollama-model-text');
  
  if (!modelSelect || !modelText) return;

  if (modelSelect.style.display === 'none') {
    showModelDropdown();
  } else {
    hideModelDropdown();
  }
}

function showModelDropdown() {
  const modelSelect = document.getElementById('ollama-model-select');
  const modelText = document.getElementById('ollama-model-text');
  
  if (!modelSelect || !modelText) return;

  // Position dropdown below the model text
  const rect = modelText.getBoundingClientRect();
  modelSelect.style.position = 'fixed';
  modelSelect.style.top = `${rect.bottom + 4}px`;
  modelSelect.style.left = `${rect.left}px`;
  modelSelect.style.display = 'block';
  modelSelect.focus();
}

function hideModelDropdown() {
  const modelSelect = document.getElementById('ollama-model-select');
  if (modelSelect) {
    modelSelect.style.display = 'none';
  }
}

function updateModelText(modelName) {
  const modelText = document.getElementById('ollama-model-text');
  if (modelText && modelName) {
    // Shorten long model names for display
    const displayName = modelName.length > 20 
      ? modelName.substring(0, 17) + '...' 
      : modelName;
    modelText.textContent = displayName;
    modelText.title = modelName; // Full name in tooltip
  }
}

async function loadAvailableModels() {
  const modelSelect = document.getElementById('ollama-model-select');
  const modelText = document.getElementById('ollama-model-text');
  if (!modelSelect) return;

  console.log('[Ollama] Loading available models...');
  
  try {
    // Use Ollama module if available, otherwise fallback to direct invoke
    let models;
    if (window.Ollama) {
      models = await window.Ollama.loadModels();
    } else {
      models = await invoke('list_ollama_models');
    }
    
    console.log(`[Ollama] Loaded ${models.length} models:`, models);
    
    modelSelect.innerHTML = ''; // Clear loading message
    
    if (models.length === 0) {
      modelSelect.innerHTML = '<option value="">No models available</option>';
      if (modelText) modelText.style.display = 'none';
      return;
    }

    const defaultModel = getDefaultModel();
    let defaultFound = false;
    let selectedModel = null;

    models.forEach(model => {
      const option = document.createElement('option');
      option.value = model;
      option.textContent = model;
      // Try to match default model (exact match or contains)
      if (model === defaultModel || model.includes('qwen2.5') || (!defaultFound && model.includes('qwen'))) {
        option.selected = true;
        defaultFound = true;
        selectedModel = model;
        saveSelectedModel(model);
      }
      modelSelect.appendChild(option);
    });

    // If default not found, select first model
    if (!defaultFound && models.length > 0) {
      modelSelect.selectedIndex = 0;
      selectedModel = models[0];
      saveSelectedModel(models[0]);
    }

    // Update model text display
    if (selectedModel && modelText) {
      updateModelText(selectedModel);
      modelText.style.display = 'inline';
      console.log('[Ollama] Selected model:', selectedModel);
    }

    // Update Ollama config with selected model
    if (selectedModel) {
      await updateOllamaModel(selectedModel);
    }
  } catch (err) {
    console.error('[Ollama] Failed to load models:', err);
    modelSelect.innerHTML = '<option value="">Error loading models</option>';
    if (modelText) modelText.style.display = 'none';
  }
}

// ============================================================================
// Ollama Connection & Model Management - Delegated to ollama.js Module  
// ============================================================================

async function updateOllamaModel(model) {
  if (window.Ollama) {
    const success = await window.Ollama.updateModel(model);
    if (success) {
      saveSelectedModel(model);
      updateModelText(model);
    }
    return success;
  }
  console.warn('[CPU] Ollama module not available');
  return false;
}

async function autoConfigureOllama() {
  if (window.Ollama) {
    const endpoint = window.Ollama.getEndpoint();
    const defaultModel = getDefaultModel();
    try {
      await window.Ollama.configure(endpoint, defaultModel);
      setTimeout(async () => {
        // checkOllamaConnection will update the icon status
        try {
          const connected = await checkOllamaConnection();
          if (connected) {
            await loadAvailableModels();
            updateOllamaIconStatus('connected');
          } else {
            // Connection check returned false - might be not configured or not running
            // Try to determine which by checking if we got an error
            updateOllamaIconStatus('unknown');
          }
        } catch (checkErr) {
          // Connection check threw an error - Ollama not available
          console.error('[Ollama] Connection check failed:', checkErr);
          updateOllamaIconStatus('error');
        }
      }, 500);
    } catch (err) {
      console.error('[Ollama] Failed to auto-configure:', err);
      // Auto-configuration failed - likely Ollama not running/not installed
      updateOllamaIconStatus('error');
    }
  }
}

async function showOllamaUrlDialog() {
  if (window.Ollama) {
    return await window.Ollama.showUrlDialog();
  }
  console.warn('[CPU] Ollama module not available');
}

async function checkOllamaConnection() {
  if (window.Ollama) {
    try {
      const connected = await window.Ollama.checkConnection();
      // Update UI elements specific to CPU window
      const connectionIndicator = document.getElementById('ollama-connection-indicator');
      const modelText = document.getElementById('ollama-model-text');
      
      // The connection indicator's class is updated by ollama.js synchronously
      // Check it directly as the source of truth, with the return value as fallback
      let isActuallyConnected = connected;
      if (connectionIndicator) {
        // Check the connection indicator's class - this is updated by ollama.js
        isActuallyConnected = connectionIndicator.classList.contains('connected');
      }
      
      // Update icon status based on actual connection state
      // If connection check succeeded but returned false, it's "unknown" (not configured yet)
      // If connection check threw an error, it's "error" (not running/not installed)
      updateOllamaIconStatus(isActuallyConnected ? 'connected' : 'unknown');
      
      // Double-check after a brief delay to catch any late updates
      setTimeout(() => {
        const indicator = document.getElementById('ollama-connection-indicator');
        if (indicator) {
          const isConnected = indicator.classList.contains('connected');
          updateOllamaIconStatus(isConnected ? 'connected' : 'unknown');
        }
      }, 150);
      
      if (connected && connectionIndicator) {
        // Load models when connected
        await loadAvailableModels();
      } else if (modelText) {
        modelText.style.display = 'none';
        hideModelDropdown();
      }
      return connected;
    } catch (error) {
      // Error means Ollama is not installed/not running - set yellow
      console.error('[CPU] Error checking Ollama connection:', error);
      updateOllamaIconStatus('error');
      return false;
    }
  }
  console.warn('[CPU] Ollama module not available');
  updateOllamaIconStatus('error');
  return false;
}

// Expose updateOllamaIconStatus globally so ollama.js can call it
window.updateOllamaIconStatus = updateOllamaIconStatus;

// ============================================================================
// Ollama Chat Functions - Delegated to ollama.js Module
// ============================================================================
// All Ollama implementation code has been moved to src/ollama.js
// These functions delegate to window.Ollama.* for backward compatibility
// ============================================================================

async function sendChatMessage() {
  if (window.Ollama) {
    return await window.Ollama.sendMessage();
  }
  console.warn('[CPU] Ollama module not available - make sure ollama.js is loaded');
}

function addChatMessage(role, content) {
  const messagesContainer = document.getElementById('chat-messages');
  if (!messagesContainer) return;

  const messageDiv = document.createElement('div');
  messageDiv.className = `chat-message ${role}`;
  
  // For assistant messages, render Markdown; for user messages, escape HTML and render as plain text
  if (role === 'assistant' && typeof marked !== 'undefined') {
    // Configure marked for GitHub-flavored Markdown
    marked.setOptions({
      breaks: true,
      gfm: true,
      highlight: function(code, lang) {
        if (typeof hljs !== 'undefined' && lang && hljs.getLanguage(lang)) {
          try {
            return hljs.highlight(code, { language: lang }).value;
          } catch (err) {
            console.warn('[Markdown] Highlight error:', err);
          }
        }
        // Fallback: auto-detect language
        if (typeof hljs !== 'undefined') {
          try {
            return hljs.highlightAuto(code).value;
          } catch (err) {
            console.warn('[Markdown] Auto-highlight error:', err);
          }
        }
        return code;
      }
    });
    
    // Render Markdown to HTML
    const markdownWrapper = document.createElement('div');
    markdownWrapper.className = 'markdown';
    markdownWrapper.innerHTML = marked.parse(content);
    messageDiv.appendChild(markdownWrapper);
    
    // Apply syntax highlighting to code blocks (hljs might not have run during marked.parse)
    if (typeof hljs !== 'undefined') {
      markdownWrapper.querySelectorAll('pre code').forEach((block) => {
        hljs.highlightElement(block);
      });
    }
  } else {
    // User messages: escape HTML and render as plain text
    const textNode = document.createTextNode(content);
    messageDiv.appendChild(textNode);
  }
  
  messagesContainer.appendChild(messageDiv);
  messagesContainer.scrollTop = messagesContainer.scrollHeight;
  
  return messageDiv;
}

function addThinkingAnimation() {
  const messagesContainer = document.getElementById('chat-messages');
  if (!messagesContainer) return null;

  const messageDiv = document.createElement('div');
  const thinkingId = `thinking-${Date.now()}`;
  messageDiv.id = thinkingId;
  messageDiv.className = 'chat-message assistant thinking';
  
  const thinkingContent = document.createElement('div');
  thinkingContent.className = 'thinking-animation';
  thinkingContent.innerHTML = `
    <div class="thinking-dots">
      <span></span>
      <span></span>
      <span></span>
    </div>
  `;
  
  messageDiv.appendChild(thinkingContent);
  messagesContainer.appendChild(messageDiv);
  messagesContainer.scrollTop = messagesContainer.scrollHeight;
  
  return thinkingId;
}

function replaceThinkingWithResponse(thinkingId, content, durationMs) {
  if (!thinkingId) {
    // Fallback: just add the message normally
    addChatMessage('assistant', content);
    return;
  }
  
  const thinkingElement = document.getElementById(thinkingId);
  if (!thinkingElement) {
    // Fallback: just add the message normally
    addChatMessage('assistant', content);
    return;
  }
  
  // Remove thinking class and animation
  thinkingElement.classList.remove('thinking');
  thinkingElement.innerHTML = '';
  
  // Add response time if available (format as seconds)
  if (durationMs !== null && durationMs !== undefined) {
    const timeLabel = document.createElement('div');
    timeLabel.className = 'response-time';
    
    // Format: convert ms to seconds with one decimal place
    // e.g., 3947ms -> 3.9s, 1234ms -> 1.2s, 500ms -> 0.5s
    const seconds = durationMs / 1000;
    const formattedTime = seconds.toFixed(1); // Always show one decimal
    
    timeLabel.textContent = `thinking time: ${formattedTime}s`;
    thinkingElement.appendChild(timeLabel);
  }
  
  // Render Markdown content
  if (typeof marked !== 'undefined') {
    marked.setOptions({
      breaks: true,
      gfm: true,
      highlight: function(code, lang) {
        if (typeof hljs !== 'undefined' && lang && hljs.getLanguage(lang)) {
          try {
            return hljs.highlight(code, { language: lang }).value;
          } catch (err) {
            console.warn('[Markdown] Highlight error:', err);
          }
        }
        if (typeof hljs !== 'undefined') {
          try {
            return hljs.highlightAuto(code).value;
          } catch (err) {
            console.warn('[Markdown] Auto-highlight error:', err);
          }
        }
        return code;
      }
    });
    
    const markdownWrapper = document.createElement('div');
    markdownWrapper.className = 'markdown';
    markdownWrapper.innerHTML = marked.parse(content);
    thinkingElement.appendChild(markdownWrapper);
    
    // Apply syntax highlighting
    if (typeof hljs !== 'undefined') {
      markdownWrapper.querySelectorAll('pre code').forEach((block) => {
        hljs.highlightElement(block);
      });
    }
  } else {
    // Fallback to plain text
    const textNode = document.createTextNode(content);
    thinkingElement.appendChild(textNode);
  }
  
  // Scroll to bottom
  const messagesContainer = document.getElementById('chat-messages');
  if (messagesContainer) {
    messagesContainer.scrollTop = messagesContainer.scrollHeight;
  }
}

// Initialize collapsible sections (Details and Top Processes)
function initCollapsibleSections() {
  // Get collapsed state from localStorage (default to true - hidden)
  const sectionsCollapsed = localStorage.getItem('details_processes_collapsed') !== 'false';
  
  const detailsSection = document.querySelector('.apple-details');
  const processesSection = document.querySelector('.apple-processes');
  const detailsDivider = detailsSection?.previousElementSibling;
  const processesDivider = processesSection?.previousElementSibling;
  const detailsHeader = document.getElementById('details-header');
  const processesHeader = document.getElementById('processes-header');
  const usageCard = document.getElementById('cpu-usage-card');
  
  // Hide Details section
  function hideDetails() {
    if (detailsSection) detailsSection.style.display = 'none';
    if (detailsDivider && detailsDivider.classList.contains('apple-divider')) {
      detailsDivider.style.display = 'none';
    }
  }
  
  // Show Details section
  function showDetails() {
    if (detailsSection) detailsSection.style.display = '';
    if (detailsDivider && detailsDivider.classList.contains('apple-divider')) {
      detailsDivider.style.display = '';
    }
  }
  
  // Hide Processes section
  function hideProcesses() {
    if (processesSection) processesSection.style.display = 'none';
    if (processesDivider && processesDivider.classList.contains('apple-divider')) {
      processesDivider.style.display = 'none';
    }
  }
  
  // Show Processes section
  function showProcesses() {
    if (processesSection) processesSection.style.display = '';
    if (processesDivider && processesDivider.classList.contains('apple-divider')) {
      processesDivider.style.display = '';
    }
  }
  
  // Hide both sections
  function hideSections() {
    hideDetails();
    hideProcesses();
    localStorage.setItem('details_processes_collapsed', 'true');
  }
  
  // Show both sections
  function showSections() {
    showDetails();
    showProcesses();
    localStorage.setItem('details_processes_collapsed', 'false');
  }
  
  // Apply initial state (hidden by default)
  if (sectionsCollapsed) {
    hideSections();
  } else {
    showSections();
  }
  
  // Details header click - hide Details section
  if (detailsHeader) {
    detailsHeader.addEventListener('click', (e) => {
      e.stopPropagation();
      hideDetails();
    });
  }
  
  // Processes header click - hide Processes section
  if (processesHeader) {
    processesHeader.addEventListener('click', (e) => {
      e.stopPropagation();
      hideProcesses();
    });
  }
  
  // Usage card click - toggle both sections (open/close)
  if (usageCard) {
    usageCard.addEventListener('click', (e) => {
      e.stopPropagation();
      const currentlyHidden = detailsSection?.style.display === 'none' || processesSection?.style.display === 'none';
      if (currentlyHidden) {
        showSections();
      } else {
        hideSections();
      }
    });
  }
}

// Initialize icon line click handlers
function initIconLine() {
  const monitorsIcon = document.getElementById('icon-monitors');
  const ollamaIcon = document.getElementById('icon-ollama');
  
  // Monitors icon click - toggle the External / Monitors section
  if (monitorsIcon) {
    monitorsIcon.addEventListener('click', (e) => {
      e.stopPropagation();
      const monitorsHeader = document.getElementById('monitors-header');
      if (monitorsHeader) {
        // Always toggle by clicking the header (it handles expand/collapse)
        monitorsHeader.click();
      }
    });
  }
  
  // Ollama icon click - toggle the AI Chat section
  if (ollamaIcon) {
    ollamaIcon.addEventListener('click', (e) => {
      e.stopPropagation();
      const ollamaHeader = document.getElementById('ollama-header');
      if (ollamaHeader) {
        // Always toggle by clicking the header (it handles expand/collapse)
        ollamaHeader.click();
      }
    });
  }
}

// Initialize monitoring features when DOM is ready
function initMonitoringFeatures() {
  // Use setTimeout to ensure DOM is fully ready
  setTimeout(() => {
    initIconLine();
    initCollapsibleSections();
    initMonitorsSection();
    initOllamaSection();
    // Auto-configure Ollama with default endpoint (if module is available)
    if (window.Ollama) {
      autoConfigureOllama();
    }
  }, 100);
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', initMonitoringFeatures);
} else {
  initMonitoringFeatures();
}

// Battery/power is now updated directly in the refresh() function
// No need for wrapper since refresh() already calls get_cpu_details

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
  if (monitorsUpdateInterval) {
    clearInterval(monitorsUpdateInterval);
    monitorsUpdateInterval = null;
  }
  console.log('Cleaned up animation state on window close');
});
