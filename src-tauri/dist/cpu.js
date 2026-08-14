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
  const internals = window.__TAURI_INTERNALS__;
  if (internals && typeof internals.invoke === 'function') {
    return internals.invoke.bind(internals);
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
  gpuUsage: 0,
  frequency: 0,
  cpuPower: 0,
  gpuPower: 0,
  load1: 0,
  load5: 0,
  load15: 0,
  totalPower: 0  // CRITICAL: Cache total power to prevent flickering in updateBatteryPower()
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

// Chart-specific refresh: temperature updates every 3s (changes slowly); usage/frequency every 1s
const TEMPERATURE_UPDATE_INTERVAL_MS = 3000;
let lastTemperatureUpdateMs = 0;

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
let lastProcessListKey = "";
let isWaitingForData = false; // Track if we're waiting for real data (non-zero usage)

const PINNED_PROCESS_NAMES_KEY = "pinned_process_names";
const MAX_PINNED_PROCESSES = 6;
const PROCESS_LIST_DISPLAY_CAP = 10;

function getPinnedProcessNames() {
  try {
    const raw = localStorage.getItem(PINNED_PROCESS_NAMES_KEY);
    const parsed = raw ? JSON.parse(raw) : [];
    if (!Array.isArray(parsed)) return [];
    return parsed
      .filter((n) => typeof n === "string" && n.trim())
      .map((n) => n.trim())
      .slice(0, MAX_PINNED_PROCESSES);
  } catch {
    return [];
  }
}

function setPinnedProcessNames(names) {
  const cleaned = (names || [])
    .filter((n) => typeof n === "string" && n.trim())
    .map((n) => n.trim())
    .slice(0, MAX_PINNED_PROCESSES);
  localStorage.setItem(PINNED_PROCESS_NAMES_KEY, JSON.stringify(cleaned));
  return cleaned;
}

function togglePinnedProcessName(name) {
  if (!name) return getPinnedProcessNames();
  const pins = getPinnedProcessNames();
  const idx = pins.indexOf(name);
  if (idx >= 0) {
    pins.splice(idx, 1);
  } else if (pins.length < MAX_PINNED_PROCESSES) {
    pins.push(name);
  }
  return setPinnedProcessNames(pins);
}

function mergePinnedProcesses(pinOrder, pinnedLookup, top) {
  const pinned = [];
  const usedNames = new Set();
  for (const name of pinOrder) {
    const fromLookup = (pinnedLookup || []).find((p) => p.name === name);
    const fromTop = (top || []).find((p) => p.name === name);
    const proc = fromLookup || fromTop;
    if (proc) {
      pinned.push(proc);
      usedNames.add(name);
    }
  }
  const rest = (top || []).filter((p) => !usedNames.has(p.name));
  return [...pinned, ...rest].slice(0, PROCESS_LIST_DISPLAY_CAP);
}

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
    
    // Update temperature (chart-specific refresh: only every 3s; usage/frequency stay at 1s)
    const nowMs = Date.now();
    const shouldUpdateTemperature = lastTemperatureUpdateMs === 0 || (nowMs - lastTemperatureUpdateMs >= TEMPERATURE_UPDATE_INTERVAL_MS);
    if (shouldUpdateTemperature) {
      lastTemperatureUpdateMs = nowMs;
    }
    const tempEl = document.getElementById("temperature-value");
    const tempHint = document.getElementById("temperature-hint");
    const tempSubtext = document.getElementById("temperature-subtext");
    const newTemp = Math.round(data.temperature);
    
    if (shouldUpdateTemperature) {
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
      // Ring gauge and theme charts only when we refresh temperature
      updateRingGauge("temperature-ring-progress", Math.min(100, data.temperature), 'temperature');
      
      if (window.posterCharts && data.can_read_temperature && data.temperature > 0) {
        window.posterCharts.updateTemperature(data.temperature);
      }
      if (window.darkHistory && data.can_read_temperature && data.temperature > 0) {
        window.darkHistory.updateTemperature(data.temperature);
      }
      if (window.lightHistory && data.can_read_temperature && data.temperature > 0) {
        window.lightHistory.updateTemperature(data.temperature);
      }
      if (window.futuristicHistory && data.can_read_temperature && data.temperature > 0) {
        window.futuristicHistory.updateTemperature(data.temperature);
      }
      if (window.materialHistory && data.can_read_temperature && data.temperature > 0) {
        window.materialHistory.updateTemperature(data.temperature);
      }
      if (window.neonHistory && data.can_read_temperature && data.temperature > 0) {
        window.neonHistory.updateTemperature(data.temperature);
      }
      if (window.swissHistory && data.can_read_temperature && data.temperature > 0) {
        window.swissHistory.updateTemperature(data.temperature);
      }
      if (window.architectHistory && data.can_read_temperature && data.temperature > 0) {
        window.architectHistory.updateTemperature(data.temperature);
      }
      if (window.appleHistory && data.can_read_temperature && data.temperature > 0) {
        window.appleHistory.updateTemperature(data.temperature);
      }
    }

    // Update GPU usage (top gauge)
    const gpuUsageEl = document.getElementById("gpu-usage-value");
    const gpuUsageSubtext = document.getElementById("gpu-usage-subtext");
    if (gpuUsageEl) {
      const newGpuUsage = Math.max(0, Math.round(data.gpu_usage || 0));
      const gpuNumberText = `${newGpuUsage}`;
      const gpuCurrentText = gpuUsageEl.textContent.match(/^\d+/)
        ? gpuUsageEl.textContent.match(/^\d+/)[0]
        : "";
      if (gpuCurrentText !== gpuNumberText) {
        scheduleDOMUpdate(() => {
          if (gpuUsageEl.firstChild && gpuUsageEl.firstChild.nodeType === 3) {
            gpuUsageEl.firstChild.textContent = gpuNumberText;
          } else {
            gpuUsageEl.innerHTML = `${gpuNumberText}<span class="metric-unit">%</span>`;
          }
        });
        previousValues.gpuUsage = newGpuUsage;
      }
      if (gpuUsageSubtext) {
        const gpuSubtext = "Live";
        if (gpuUsageSubtext.textContent !== gpuSubtext) {
          scheduleDOMUpdate(() => {
            gpuUsageSubtext.textContent = gpuSubtext;
          });
        }
      }
      updateRingGauge("gpu-usage-ring-progress", data.gpu_usage || 0, "gpu");
      if (window.posterCharts && typeof window.posterCharts.updateGpuUsage === "function") {
        window.posterCharts.updateGpuUsage(data.gpu_usage || 0);
      }
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
    
    // Update apple theme history charts if available
    if (window.appleHistory) {
      window.appleHistory.updateUsage(data.usage);
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
      
      // Update apple theme history charts if available
      if (window.appleHistory && data.frequency > 0) {
        window.appleHistory.updateFrequency(data.frequency);
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
      console.log("[CPU Power] Cannot read CPU power");
      console.log("[CPU Power] Failed attempts: ", failedAttempts.cpuPower);
      console.log("[CPU Power] Should show hint: ", cpuPowerEl);
      failedAttempts.cpuPower++;
      // Only show hint after multiple failed attempts
      const shouldShowHint = failedAttempts.cpuPower >= FAILED_ATTEMPTS_THRESHOLD;
      const displayText = shouldShowHint ? "--:--" : previousValues.cpuPower;
      if (cpuPowerEl.textContent !== displayText) {
        scheduleDOMUpdate(() => {
          cpuPowerEl.textContent = displayText;
        });
      }
    } else {
      failedAttempts.cpuPower = 0;
      // CRITICAL: Preserve last known good value to prevent flickering
      // Strategy: Only update if backend value > 0, otherwise preserve what we have
      
      // Initialize from DOM on first call if previousValues is 0 (handles page reload)
      if (previousValues.cpuPower === 0 && cpuPowerEl && cpuPowerEl.textContent) {
        const currentText = cpuPowerEl.textContent.trim();
        const match = currentText.match(/(\d+(?:\.\d+)?)\s*W/);
        if (match) {
          const domValue = parseFloat(match[1]);
          if (domValue > 0) {
            previousValues.cpuPower = domValue;
          }
        }
      }
      
      // Use cached value as default
      let cpuPowerValue = previousValues.cpuPower || 0;
      
      // Only update if backend has a valid value > 0
      if (data.cpu_power && data.cpu_power > 0) {
        cpuPowerValue = data.cpu_power;
        previousValues.cpuPower = data.cpu_power;
        console.log("[CPU Power] Updated to: ", data.cpu_power, "from: ", previousValues.cpuPower);
      }
      // If backend value is 0 or undefined, keep using previousValues (don't reset to 0)
      
      // CRITICAL: Show 1 decimal place to prevent flickering when values are < 1W
      // Math.round() would show 0.3W as "0 W", causing flicker between 0 and actual values
      const formatted = `${cpuPowerValue.toFixed(1)} W`;
      // Only update DOM if value actually changed
      if (cpuPowerEl.textContent !== formatted) {
        console.log("[CPU Power] Updating DOM to: ", formatted);
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
      const displayText = shouldShowHint ? "--:--" : "0 W";
      if (gpuPowerEl.textContent !== displayText) {
        scheduleDOMUpdate(() => {
          gpuPowerEl.textContent = displayText;
        });
      }
    } else {
      failedAttempts.gpuPower = 0;
      // CRITICAL: Preserve last known good value to prevent flickering
      // Strategy: Only update if backend value > 0, otherwise preserve what we have
      
      // Initialize from DOM on first call if previousValues is 0 (handles page reload)
      if (previousValues.gpuPower === 0 && gpuPowerEl && gpuPowerEl.textContent) {
        const currentText = gpuPowerEl.textContent.trim();
        const match = currentText.match(/(\d+(?:\.\d+)?)\s*W/);
        if (match) {
          const domValue = parseFloat(match[1]);
          if (domValue > 0) {
            previousValues.gpuPower = domValue;
          }
        }
      }
      
      // Use cached value as default
      let gpuPowerValue = previousValues.gpuPower || 0;
      
      // Only update if backend has a valid value > 0
      if (data.gpu_power && data.gpu_power > 0) {
        gpuPowerValue = data.gpu_power;
        previousValues.gpuPower = data.gpu_power;
      }
      // If backend value is 0 or undefined, keep using previousValues (don't reset to 0)
      
      // CRITICAL: Show 1 decimal place to prevent flickering when values are < 1W
      // Math.round() would show 0.3W as "0 W", causing flicker between 0 and actual values
      const formatted = `${gpuPowerValue.toFixed(1)} W`;
      // Only update DOM if value actually changed
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
      if (!list) return;
      
      // Skip DOM update when process list unchanged (avoids reflows and listener churn)
      const pinnedNames = getPinnedProcessNames();
      let processes =
        data.top_processes && data.top_processes.length > 0
          ? data.top_processes.slice(0, 8)
          : [];
      if (pinnedNames.length > 0) {
        let pinnedLookup = [];
        try {
          pinnedLookup = await invoke("get_processes_by_names", { names: pinnedNames });
        } catch (e) {
          console.warn("get_processes_by_names failed", e);
        }
        processes = mergePinnedProcesses(pinnedNames, pinnedLookup, processes);
      }
      const processKey = processes.length > 0
        ? `${pinnedNames.join(",")}|` +
          processes.map((p) => `${p.pid}:${p.cpu.toFixed(1)}:${p.name}`).join("|")
        : `empty|${pinnedNames.join(",")}`;
      if (!forceUpdate && !isInitialLoad && processKey === lastProcessListKey) {
        return;
      }
      lastProcessListKey = processKey;

      const activeRow = document.activeElement?.closest?.(".process-row");
      const listHadFocus = !!(activeRow && list.contains(activeRow));
      const focusPid =
        (activeRow && list.contains(activeRow) && activeRow.getAttribute("data-pid")) ||
        (currentProcessPid !== null ? String(currentProcessPid) : null);
      
      // Use document fragment to batch all DOM updates and reduce reflows
      const fragment = document.createDocumentFragment();

      const colHeader = document.createElement("div");
      colHeader.className = "process-list-header";
      colHeader.setAttribute("aria-hidden", "true");
      const colPin = document.createElement("span");
      colPin.className = "process-list-header-pin";
      colPin.textContent = "★";
      colPin.title = "Pin favorites";
      const colName = document.createElement("span");
      colName.textContent = "Process";
      const colCpu = document.createElement("span");
      colCpu.textContent = "CPU";
      colHeader.appendChild(colPin);
      colHeader.appendChild(colName);
      colHeader.appendChild(colCpu);
      fragment.appendChild(colHeader);
      
      if (processes.length > 0) {
        let tabIdx = processes.findIndex(
          (p) => focusPid !== null && String(p.pid) === String(focusPid)
        );
        if (tabIdx < 0) tabIdx = 0;
        processes.forEach((proc, i) => {
          const isPinned = pinnedNames.includes(proc.name);
          const row = document.createElement("div");
          row.className = "process-row" + (isPinned ? " is-pinned" : "");
          row.setAttribute("data-pid", String(proc.pid));
          row.setAttribute("data-name", proc.name);
          row.setAttribute("role", "option");
          row.setAttribute("tabindex", i === tabIdx ? "0" : "-1");
          row.title =
            "Click / Enter / d for details · ↑↓ / j k · PgUp/PgDn · P pin · Esc clears";
          const selected =
            currentProcessPid !== null && Number(proc.pid) === Number(currentProcessPid);
          row.setAttribute("aria-selected", selected ? "true" : "false");
          if (selected) {
            row.classList.add("is-selected");
            row.setAttribute("aria-current", "true");
          }

          const pinBtn = document.createElement("button");
          pinBtn.type = "button";
          pinBtn.className = "process-pin" + (isPinned ? " is-pinned" : "");
          pinBtn.setAttribute("aria-label", isPinned ? `Unpin ${proc.name}` : `Pin ${proc.name}`);
          pinBtn.setAttribute("aria-pressed", isPinned ? "true" : "false");
          pinBtn.setAttribute("data-name", proc.name);
          pinBtn.title = isPinned ? "Unpin" : "Pin favorite";
          pinBtn.textContent = isPinned ? "★" : "☆";
          
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
          
          row.appendChild(pinBtn);
          row.appendChild(name);
          row.appendChild(usage);
          fragment.appendChild(row);
        });
      } else {
        const emptyMsg = document.createElement("div");
        emptyMsg.className = "process-empty";
        emptyMsg.textContent = "Waiting for process samples — opens with the CPU window";
        fragment.appendChild(emptyMsg);
      }
      
      // Single click listener on list (event delegation) instead of per-row listeners
      scheduleDOMUpdate(() => {
        if (!list.dataset.processClickDelegation) {
          list.dataset.processClickDelegation = "true";
          list.setAttribute("role", "listbox");
          list.setAttribute("aria-label", "Top processes");
          list.addEventListener("click", (e) => {
            const pinBtn = e.target.closest(".process-pin");
            if (pinBtn && list.contains(pinBtn)) {
              e.preventDefault();
              e.stopPropagation();
              togglePinnedProcessName(pinBtn.getAttribute("data-name"));
              window._forceProcessUpdate = true;
              if (window.refreshData) window.refreshData();
              return;
            }
            const row = e.target.closest(".process-row");
            if (row) {
              const pid = row.getAttribute("data-pid");
              if (pid) showProcessDetails(parseInt(pid, 10));
            }
          });
          list.addEventListener("keydown", (e) => {
            const row = e.target.closest(".process-row");
            if (!row || !list.contains(row)) return;
            // Pin button handles its own keys; do not steal from text fields.
            if (e.target.closest && e.target.closest(".process-pin")) return;
            const rows = Array.from(list.querySelectorAll(".process-row"));
            const idx = rows.indexOf(row);
            if (idx < 0) return;

            if (e.key === "Enter" || e.key === " ") {
              e.preventDefault();
              const pid = row.getAttribute("data-pid");
              if (pid) showProcessDetails(parseInt(pid, 10));
              return;
            }

            // d opens details (Monitors muscle memory; Enter/Space still work).
            if (e.key === "d" || e.key === "D") {
              e.preventDefault();
              const pid = row.getAttribute("data-pid");
              if (pid) showProcessDetails(parseInt(pid, 10));
              return;
            }

            // P toggles pin on the focused row (mouse still uses ★).
            if (e.key === "p" || e.key === "P") {
              e.preventDefault();
              const name = row.getAttribute("data-name");
              if (name) {
                togglePinnedProcessName(name);
                window._forceProcessUpdate = true;
                if (window.refreshData) window.refreshData();
              }
              return;
            }

            // Esc: close process details first, then clear selection (Monitors parity).
            if (e.key === "Escape" || e.key === "Esc") {
              if (!row.classList.contains("is-selected") && document.activeElement !== row) {
                return;
              }
              e.preventDefault();
              e.stopPropagation();
              const modal = document.getElementById("process-details-modal");
              if (modal && modal.style.display !== "none") {
                closeProcessDetailsModal();
                return;
              }
              if (currentProcessPid !== null) {
                currentProcessPid = null;
                syncProcessRowSelection();
              }
              rows.forEach((r) => r.setAttribute("tabindex", "-1"));
              if (rows[0]) rows[0].setAttribute("tabindex", "0");
              row.blur();
              return;
            }

            let next = -1;
            const page = 5;
            if (e.key === "ArrowDown" || e.key === "j") next = Math.min(idx + 1, rows.length - 1);
            else if (e.key === "ArrowUp" || e.key === "k") next = Math.max(idx - 1, 0);
            else if (e.key === "PageDown") next = Math.min(idx + page, rows.length - 1);
            else if (e.key === "PageUp") next = Math.max(idx - page, 0);
            else if (e.key === "Home") next = 0;
            else if (e.key === "End") next = rows.length - 1;
            else return;
            e.preventDefault();
            if (next < 0 || next === idx) return;
            rows.forEach((r, i) => r.setAttribute("tabindex", i === next ? "0" : "-1"));
            rows[next].focus();
            if (typeof rows[next].scrollIntoView === "function") {
              rows[next].scrollIntoView({ block: "nearest" });
            }
          });
        }
        ensureProcessesListKbHint(list, processes.length > 0);
        list.replaceChildren();
        list.appendChild(fragment);
        if (listHadFocus) {
          const target =
            (focusPid && list.querySelector(`.process-row[data-pid="${focusPid}"]`)) ||
            list.querySelector('.process-row[tabindex="0"]') ||
            list.querySelector(".process-row");
          target?.focus();
        }
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
  const rings = ['gpu-usage-ring-progress', 'temperature-ring-progress', 'cpu-usage-ring-progress', 'frequency-ring-progress'];
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
      try {
        const prev = localStorage.getItem("macStatsAssetVersion");
        if (prev !== appVersion) {
          localStorage.setItem("macStatsAssetVersion", appVersion);
          // Hard reload theme shell once per version so gauge/layout HTML updates stick.
          if (prev) {
            window.location.replace(`../../cpu.html?v=${encodeURIComponent(appVersion)}`);
            return appVersion;
          }
        }
      } catch (_) {}
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
    fetchAppVersion().then((v) => {
      showFirstLaunchTip();
      checkForAppUpdate(v);
    });
    initRingGauges();
    init();
  });
} else {
  showFirstLaunchTip();
  (async () => {
    try {
      const inv = typeof getInvoke === "function" ? getInvoke() : null;
      if (inv) {
        const v = await inv("get_app_version");
        checkForAppUpdate(v);
      }
    } catch (_) {}
  })();
  initRingGauges();
  init();
}

/** Shared glass styles for first-launch tip + update banner. */
function ensureAppBannerStyles() {
  if (document.getElementById("mac-stats-banner-styles")) return;
  const style = document.createElement("style");
  style.id = "mac-stats-banner-styles";
  style.textContent = `
    #mac-stats-banners {
      position: fixed;
      top: 14px;
      left: 16px;
      right: 16px;
      z-index: 9999;
      display: flex;
      flex-direction: column;
      gap: 8px;
      pointer-events: none;
    }
    #mac-stats-banners > * { pointer-events: auto; }
    #mac-stats-first-launch-tip,
    #mac-stats-update-banner {
      display: flex;
      align-items: flex-start;
      gap: 12px;
      margin: 0;
      padding: 12px 12px 12px 14px;
      border-radius: 14px;
      font: 12px/1.45 -apple-system, BlinkMacSystemFont, "SF Pro Text", "Segoe UI", sans-serif;
      color: var(--text, rgba(12, 12, 16, 0.88));
      background: var(--panel, rgba(255, 255, 255, 0.72));
      border: 1px solid var(--panel-border, rgba(255, 255, 255, 0.7));
      box-shadow: var(--panel-shadow, 0 16px 48px rgba(0, 0, 0, 0.12));
      backdrop-filter: blur(28px) saturate(160%);
      -webkit-backdrop-filter: blur(28px) saturate(160%);
      animation: mac-stats-banner-in 0.32s ease-out;
    }
    @keyframes mac-stats-banner-in {
      from { opacity: 0; transform: translateY(-6px); }
      to { opacity: 1; transform: translateY(0); }
    }
    #mac-stats-update-banner {
      background: color-mix(in srgb, var(--accent, #8bb4e8) 22%, var(--panel, rgba(255,255,255,0.72)));
      border-color: color-mix(in srgb, var(--accent, #8bb4e8) 35%, transparent);
    }
    #mac-stats-first-launch-tip .tip-glyph,
    #mac-stats-update-banner .tip-glyph {
      flex-shrink: 0;
      width: 28px;
      height: 28px;
      border-radius: 9px;
      display: flex;
      align-items: center;
      justify-content: center;
      background: rgba(255, 255, 255, 0.45);
      border: 1px solid rgba(255, 255, 255, 0.55);
      box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.5);
      color: var(--muted, rgba(12, 12, 16, 0.5));
    }
    #mac-stats-update-banner .tip-glyph {
      color: color-mix(in srgb, var(--accent, #007aff) 70%, var(--text, #1d1d1f));
    }
    #mac-stats-first-launch-tip .tip-glyph svg,
    #mac-stats-update-banner .tip-glyph svg {
      width: 14px;
      height: 14px;
      display: block;
    }
    #mac-stats-first-launch-tip .tip-body,
    #mac-stats-update-banner .tip-body {
      flex: 1;
      min-width: 0;
      display: grid;
      gap: 3px;
      padding-top: 1px;
    }
    #mac-stats-first-launch-tip .tip-title,
    #mac-stats-update-banner .tip-title {
      font-size: 13px;
      font-weight: 650;
      letter-spacing: -0.01em;
      color: var(--text, rgba(12, 12, 16, 0.88));
    }
    #mac-stats-first-launch-tip .tip-copy,
    #mac-stats-update-banner .tip-copy {
      font-size: 12px;
      line-height: 1.45;
      color: var(--muted, rgba(12, 12, 16, 0.55));
    }
    #mac-stats-first-launch-tip .tip-copy strong,
    #mac-stats-update-banner .tip-copy strong {
      font-weight: 600;
      color: var(--text, rgba(12, 12, 16, 0.78));
    }
    #mac-stats-first-launch-tip code,
    #mac-stats-update-banner code {
      font-family: ui-monospace, "SF Mono", Menlo, monospace;
      font-size: 11px;
      padding: 1px 5px;
      border-radius: 5px;
      background: rgba(12, 12, 16, 0.06);
      border: 1px solid rgba(12, 12, 16, 0.04);
      color: var(--text, rgba(12, 12, 16, 0.72));
    }
    #mac-stats-first-launch-tip .tip-dismiss,
    #mac-stats-update-banner .tip-dismiss {
      flex-shrink: 0;
      width: 24px;
      height: 24px;
      margin-top: 2px;
      border: 1px solid rgba(12, 12, 16, 0.08);
      border-radius: 8px;
      background: rgba(255, 255, 255, 0.35);
      cursor: pointer;
      font-size: 14px;
      line-height: 1;
      color: var(--muted, rgba(12, 12, 16, 0.45));
      display: flex;
      align-items: center;
      justify-content: center;
      transition: background 0.15s ease, color 0.15s ease, border-color 0.15s ease;
    }
    #mac-stats-first-launch-tip .tip-dismiss:hover,
    #mac-stats-update-banner .tip-dismiss:hover {
      color: var(--text, rgba(12, 12, 16, 0.8));
      background: rgba(255, 255, 255, 0.55);
      border-color: rgba(12, 12, 16, 0.12);
    }
    #mac-stats-update-banner a {
      color: inherit;
      font-weight: 600;
      text-decoration: underline;
      text-decoration-color: color-mix(in srgb, currentColor 35%, transparent);
      text-underline-offset: 2px;
    }
    #mac-stats-update-banner a:hover {
      text-decoration-color: currentColor;
    }
  `;
  document.head.appendChild(style);
}

function getAppBannerHost() {
  ensureAppBannerStyles();
  let host = document.getElementById("mac-stats-banners");
  if (!host) {
    host = document.createElement("div");
    host.id = "mac-stats-banners";
    document.body.prepend(host);
  }
  return host;
}

/** Non-intrusive first-open tip: where config lives + Ollama requirement. */
function showFirstLaunchTip() {
  try {
    if (localStorage.getItem("mac_stats_first_launch_tip_v1") === "1") return;
  } catch (_) {
    return;
  }
  if (document.getElementById("mac-stats-first-launch-tip")) return;

  const tip = document.createElement("div");
  tip.id = "mac-stats-first-launch-tip";
  tip.setAttribute("role", "status");
  tip.innerHTML = `
    <div class="tip-glyph" aria-hidden="true">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="9"></circle>
        <path d="M12 8v.01"></path>
        <path d="M11 12h1v4h1"></path>
      </svg>
    </div>
    <div class="tip-body">
      <div class="tip-title">Welcome</div>
      <div class="tip-copy">
        Settings &amp; logs live in <code>~/.mac-stats/</code>.
        AI chat needs <strong>Ollama</strong> running locally with a model
        (e.g. <code>ollama pull llama3.2</code>).
      </div>
    </div>
    <button type="button" class="tip-dismiss" title="Dismiss" aria-label="Dismiss">×</button>
  `;
  tip.querySelector("button").addEventListener("click", () => {
    try {
      localStorage.setItem("mac_stats_first_launch_tip_v1", "1");
    } catch (_) {}
    tip.remove();
    const host = document.getElementById("mac-stats-banners");
    if (host && !host.children.length) host.remove();
  });
  getAppBannerHost().prepend(tip);
}

function parseSemverParts(v) {
  const s = String(v || "").replace(/^v/i, "").split(/[+-]/)[0];
  return s.split(".").map((n) => parseInt(n, 10) || 0);
}

function isNewerVersion(latest, current) {
  const a = parseSemverParts(latest);
  const b = parseSemverParts(current);
  const len = Math.max(a.length, b.length);
  for (let i = 0; i < len; i++) {
    const x = a[i] || 0;
    const y = b[i] || 0;
    if (x > y) return true;
    if (x < y) return false;
  }
  return false;
}

/** Lightweight GitHub Releases check (once per day). */
async function checkForAppUpdate(currentVersion) {
  if (!currentVersion || currentVersion === "unknown") return;
  try {
    const dayKey = new Date().toISOString().slice(0, 10);
    if (localStorage.getItem("mac_stats_update_checked_day") === dayKey) return;
    localStorage.setItem("mac_stats_update_checked_day", dayKey);
  } catch (_) {
    return;
  }

  let latestTag = null;
  let htmlUrl = "https://github.com/raro42/mac-stats/releases/latest";
  try {
    const res = await fetch(
      "https://api.github.com/repos/raro42/mac-stats/releases/latest",
      { headers: { Accept: "application/vnd.github+json" } }
    );
    if (!res.ok) return;
    const data = await res.json();
    latestTag = data.tag_name || data.name;
    if (data.html_url) htmlUrl = data.html_url;
  } catch (err) {
    console.debug("Update check skipped:", err);
    return;
  }
  if (!latestTag || !isNewerVersion(latestTag, currentVersion)) return;

  const dismissKey = `mac_stats_update_dismissed_${latestTag}`;
  try {
    if (localStorage.getItem(dismissKey) === "1") return;
  } catch (_) {}

  if (document.getElementById("mac-stats-update-banner")) return;
  const banner = document.createElement("div");
  banner.id = "mac-stats-update-banner";
  banner.setAttribute("role", "status");
  banner.innerHTML = `
    <div class="tip-glyph" aria-hidden="true">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M12 3v12"></path>
        <path d="M8 11l4 4 4-4"></path>
        <path d="M5 19h14"></path>
      </svg>
    </div>
    <div class="tip-body">
      <div class="tip-title">Update available</div>
      <div class="tip-copy">
        <strong>${latestTag}</strong> is ready (you have v${currentVersion}).
        <a href="${htmlUrl}" target="_blank" rel="noopener">Release notes</a>
        · <code>brew upgrade --cask mac-stats</code>
      </div>
    </div>
    <button type="button" class="tip-dismiss" title="Dismiss" aria-label="Dismiss">×</button>
  `;
  banner.querySelector("button").addEventListener("click", () => {
    try {
      localStorage.setItem(dismissKey, "1");
    } catch (_) {}
    banner.remove();
    const host = document.getElementById("mac-stats-banners");
    if (host && !host.children.length) host.remove();
  });
  getAppBannerHost().appendChild(banner);
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
let processDetailsFocusReturn = null;

function closeProcessDetailsModal() {
  if (processDetailsRefreshInterval) {
    clearInterval(processDetailsRefreshInterval);
    processDetailsRefreshInterval = null;
  }
  currentProcessPid = null;
  syncProcessRowSelection();
  if (processDetailsModal) {
    processDetailsModal.style.display = "none";
    processDetailsModal.setAttribute("aria-hidden", "true");
  }
  const returnEl = processDetailsFocusReturn;
  processDetailsFocusReturn = null;
  if (returnEl && typeof returnEl.focus === "function") {
    try {
      returnEl.focus();
    } catch (_) {
      /* ignore */
    }
  }
}

function openProcessDetailsModal() {
  if (!processDetailsModal) return;
  processDetailsFocusReturn = document.activeElement;
  processDetailsModal.style.display = "flex";
  processDetailsModal.setAttribute("aria-hidden", "false");
  processDetailsModal.setAttribute("role", "dialog");
  processDetailsModal.setAttribute("aria-modal", "true");
  if (!processDetailsModal.getAttribute("aria-labelledby")) {
    processDetailsModal.setAttribute("aria-labelledby", "process-details-title");
  }
  requestAnimationFrame(() => {
    processDetailsModal.querySelector("#close-process-details")?.focus();
  });
}

function syncProcessRowSelection() {
  const list = document.getElementById("process-list");
  if (!list) return;
  list.querySelectorAll(".process-row").forEach((row) => {
    const selected =
      currentProcessPid !== null &&
      row.getAttribute("data-pid") === String(currentProcessPid);
    row.classList.toggle("is-selected", selected);
    row.setAttribute("aria-selected", selected ? "true" : "false");
    if (selected) row.setAttribute("aria-current", "true");
    else row.removeAttribute("aria-current");
  });
}
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

function escapeProcessHtml(value) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function populateProcessDetailsBody(body, details, pid) {
    const startDate = formatDate(details.start_time);
    const cpuTimeFormatted = formatTime(Math.floor(details.total_cpu_time / 1000));
    const memoryFormatted = formatBytes(details.memory);
    const virtualMemoryFormatted = formatBytes(details.virtual_memory);
    const diskReadFormatted = formatBytes(details.disk_read);
    const diskWrittenFormatted = formatBytes(details.disk_written);
    const name = escapeProcessHtml(details.name);
    const parentName = details.parent_name ? escapeProcessHtml(details.parent_name) : "";
    const userName = details.user_name ? escapeProcessHtml(details.user_name) : "";
    const effectiveUserName = details.effective_user_name
      ? escapeProcessHtml(details.effective_user_name)
      : "";
    
    body.innerHTML = `
      <div class="process-detail-hero">
        <div class="process-detail-name">${name}</div>
        <div class="process-detail-pid">PID ${details.pid}</div>
      </div>
      <div class="process-detail-section">
        <div class="process-detail-row">
          <span class="process-detail-label">Current CPU</span>
          <span class="process-detail-value">${details.cpu.toFixed(1)}%</span>
        </div>
        <div class="process-detail-row">
          <span class="process-detail-label">Total CPU Time</span>
          <span class="process-detail-value">${cpuTimeFormatted}</span>
        </div>
        <div class="process-detail-row">
          <span class="process-detail-label">Started</span>
          <span class="process-detail-value">${startDate}</span>
        </div>
      </div>
      <div class="process-detail-section">
        <div class="process-detail-row">
          <span class="process-detail-label">Parent Process</span>
          <span class="process-detail-value">${parentName ? `${parentName} (PID: ${details.parent_pid})` : "—"}</span>
        </div>
        <div class="process-detail-row">
          <span class="process-detail-label">User</span>
          <span class="process-detail-value">${userName ? `${userName} (${details.user_id})` : (details.user_id || "—")}</span>
        </div>
        <div class="process-detail-row">
          <span class="process-detail-label">Effective User</span>
          <span class="process-detail-value">${effectiveUserName ? `${effectiveUserName} (${details.effective_user_id})` : (details.effective_user_id || "—")}</span>
        </div>
      </div>
      <div class="process-detail-section">
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
      </div>
      <div class="force-quit-section">
        <details class="force-quit-advanced">
          <summary>Advanced</summary>
          <p class="force-quit-hint">Force Quit ends the process immediately.</p>
          <button id="force-quit-process-btn" class="force-quit-btn" type="button">Force Quit Process</button>
        </details>
      </div>
    `;
    
    // Set up force quit button handler (remove old listeners first by cloning)
    const forceQuitBtn = document.getElementById("force-quit-process-btn");
    if (forceQuitBtn) {
      // Clone and replace to remove old event listeners when refreshing
      const newBtn = forceQuitBtn.cloneNode(true);
      forceQuitBtn.parentNode.replaceChild(newBtn, forceQuitBtn);
      
      newBtn.addEventListener("click", async () => {
        // WKWebView: window.confirm()/alert() are unreliable — two-click confirm instead.
        if (newBtn.dataset.confirmArmed !== "1") {
          newBtn.dataset.confirmArmed = "1";
          newBtn.classList.add("is-confirming");
          newBtn.textContent = "Click again to confirm Force Quit";
          setTimeout(() => {
            if (newBtn.dataset.confirmArmed === "1") {
              newBtn.dataset.confirmArmed = "0";
              newBtn.classList.remove("is-confirming");
              newBtn.textContent = "Force Quit Process";
            }
          }, 4000);
          return;
        }
        
        try {
          if (!invoke) {
            invoke = getInvoke();
            if (!invoke) {
              console.error("Cannot force quit: Tauri invoke not available");
              return;
            }
          }
          
          await invoke("force_quit_process", { pid });
          
          // Clear refresh interval and close modal
          closeProcessDetailsModal();
          
          // Force immediate refresh of process list (bypass 15-second throttle)
          window._forceProcessUpdate = true;
          if (window.refreshData) {
            await window.refreshData();
          }
        } catch (error) {
          console.error("Failed to force quit process:", error);
          newBtn.dataset.confirmArmed = "0";
          newBtn.classList.remove("is-confirming");
          newBtn.textContent = "Force Quit Process";
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
      processDetailsModal.setAttribute("aria-hidden", "true");
      processDetailsModal.setAttribute("role", "dialog");
      processDetailsModal.setAttribute("aria-modal", "true");
      processDetailsModal.setAttribute("aria-labelledby", "process-details-title");
      processDetailsModal.innerHTML = `
        <div class="settings-card">
          <div class="settings-header">
            <h2 id="process-details-title">Process Details</h2>
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
          closeProcessDetailsModal();
        });
      }
      
      // Click outside to close
      processDetailsModal.addEventListener("click", (e) => {
        if (e.target === processDetailsModal) {
          // Clear refresh interval when closing
          closeProcessDetailsModal();
        }
      });
      
      // ESC key to close
      const escHandler = (e) => {
        if (e.key === "Escape" && processDetailsModal.style.display !== "none") {
          // Clear refresh interval when closing
          closeProcessDetailsModal();
        }
      };
      document.addEventListener("keydown", escHandler);
      processDetailsModal.dataset.handlersSetup = "true";
    }
    
    // Store current PID for refresh functionality
    currentProcessPid = pid;
    syncProcessRowSelection();
    
    // Clear any existing refresh interval
    if (processDetailsRefreshInterval) {
      clearInterval(processDetailsRefreshInterval);
      processDetailsRefreshInterval = null;
    }
    
    // Populate details
    const body = document.getElementById("process-details-body");
    populateProcessDetailsBody(body, details, pid);
    
    // Show modal (using same display style as settings modal)
    openProcessDetailsModal();
    
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

  // Battery/power logging removed to reduce console noise

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
    
    // CRITICAL: Use cached total power to prevent flickering
    // Only update if backend has valid values > 0, otherwise preserve cached value
    let totalPower = previousValues.totalPower || 0;
    const backendTotal = (cpuDetails.cpu_power || 0) + (cpuDetails.gpu_power || 0);
    if (backendTotal > 0) {
      totalPower = backendTotal;
      previousValues.totalPower = backendTotal;
    }
    // If backend values are 0 or undefined, keep using previousValues (don't reset to 0)

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
    // CRITICAL: Use cached total power to prevent flickering
    // Only update if backend has valid values > 0, otherwise preserve cached value
    let totalPower = previousValues.totalPower || 0;
    const backendTotal = (cpuDetails.cpu_power || 0) + (cpuDetails.gpu_power || 0);
    if (backendTotal > 0) {
      totalPower = backendTotal;
      previousValues.totalPower = backendTotal;
    }
    // If backend values are 0 or undefined, keep using previousValues (don't reset to 0)

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

function wireCollapsibleHeaderA11y(header, options = {}) {
  if (!header || header.dataset.collapseA11y === '1') return;
  const {
    contentId = null,
    getExpanded = () => true,
    onToggle = null,
    ignoreSelector = null,
  } = options;
  header.dataset.collapseA11y = '1';
  header.setAttribute('role', 'button');
  header.setAttribute('tabindex', '0');
  if (contentId) header.setAttribute('aria-controls', contentId);
  const syncExpanded = () => {
    header.setAttribute('aria-expanded', String(!!getExpanded()));
  };
  syncExpanded();
  header._syncCollapseA11y = syncExpanded;
  if (typeof onToggle === 'function') {
    header.addEventListener('keydown', (e) => {
      if (e.key !== 'Enter' && e.key !== ' ') return;
      if (ignoreSelector && e.target.closest && e.target.closest(ignoreSelector)) return;
      e.preventDefault();
      onToggle();
      syncExpanded();
    });
  }
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
  wireMonitorRemoveDelegation();
  wireMonitorsListKeyboard();

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

  // Make header clickable/keyboardable to toggle collapse/expand
  const applyMonitorsCollapsed = () => {
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
    header.setAttribute('aria-expanded', String(!monitorsCollapsed));
  };

  wireCollapsibleHeaderA11y(header, {
    contentId: 'monitors-content',
    getExpanded: () => !monitorsCollapsed,
    ignoreSelector: '#monitors-menu-btn',
    onToggle: () => {
      monitorsCollapsed = !monitorsCollapsed;
      saveMonitorsCollapsedState(monitorsCollapsed);
      applyMonitorsCollapsed();
    },
  });

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
    applyMonitorsCollapsed();
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
  window.closeMonitorsSettings = closeMonitorsSettingsPopover;
}

// Global ESC key handler for all popovers
document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape' || e.key === 'Esc') {
    // Prefer top-level modals already handled elsewhere; skip if process details open
    const processModal = document.getElementById('process-details-modal');
    if (processModal && processModal.style.display !== 'none') return;

    const ollamaMenu = document.getElementById('ollama-menu');
    if (ollamaMenu && ollamaMenu.style.display !== 'none') {
      if (window.closeOllamaOptionsMenu) window.closeOllamaOptionsMenu();
      else ollamaMenu.style.display = 'none';
      e.preventDefault();
      return;
    }

    // Close monitors settings popover if visible
    const monitorsPopover = document.getElementById('monitors-settings-popover');
    if (monitorsPopover && monitorsPopover.style.display !== 'none') {
      if (window.closeMonitorsSettings) {
        window.closeMonitorsSettings();
      }
      e.preventDefault();
      return;
    }
    
    // Close Ollama settings popover if visible
    const ollamaPopover = document.getElementById('ollama-settings-popover');
    if (ollamaPopover && ollamaPopover.style.display !== 'none') {
      closeOllamaSettingsPopover();
      e.preventDefault();
    }
  }
});

let monitorsSettingsFocusReturn = null;
let ollamaSettingsFocusReturn = null;

async function showMonitorsSettings() {
  const popover = document.getElementById('monitors-settings-popover');
  if (popover) {
    monitorsSettingsFocusReturn = document.activeElement;
    popover.style.display = 'flex';
    popover.setAttribute('role', 'dialog');
    popover.setAttribute('aria-modal', 'true');
    popover.setAttribute('aria-hidden', 'false');
    const title = popover.querySelector('.popover-header h3');
    if (title) {
      if (!title.id) title.id = 'monitors-settings-title';
      popover.setAttribute('aria-labelledby', title.id);
    }
    wireMonitorRemoveDelegation();
    await refreshMonitorsSettingsList();
    requestAnimationFrame(() => {
      document.getElementById('monitors-settings-close')?.focus();
    });
  }
}

function closeMonitorsSettingsPopover() {
  const popover = document.getElementById('monitors-settings-popover');
  const addForm = document.getElementById('add-monitor-form');
  if (popover) {
    popover.style.display = 'none';
    popover.setAttribute('aria-hidden', 'true');
  }
  if (addForm) addForm.style.display = 'none';
  const returnEl = monitorsSettingsFocusReturn;
  monitorsSettingsFocusReturn = null;
  if (returnEl && typeof returnEl.focus === 'function') {
    try {
      returnEl.focus();
    } catch (_) {
      /* ignore */
    }
  }
}


async function removeMonitorById(monitorId) {
  const invokeFn = getInvoke() || invoke;
  if (!invokeFn) {
    console.error('[Monitors] remove failed: Tauri invoke unavailable');
    return;
  }
  console.log('[Monitors] remove_monitor invoke', monitorId);
  try {
    await invokeFn('remove_monitor', { monitorId });
    monitorStatusCache.delete(monitorId);
    await refreshMonitorsSettingsList();
    await loadMonitors();
    await updateMonitorsSummary();
    console.log('[Monitors] removed', monitorId);
  } catch (err) {
    console.error('[Monitors] remove_monitor failed:', err);
  }
}

function wireMonitorRemoveDelegation() {
  const settingsList = document.getElementById('monitors-settings-list');
  if (!settingsList || settingsList.dataset.removeDelegation === '1') return;
  settingsList.dataset.removeDelegation = '1';
  settingsList.addEventListener('click', (e) => {
    const btn = e.target && e.target.closest && e.target.closest('.monitor-remove-btn');
    if (!btn) return;
    e.preventDefault();
    e.stopPropagation();
    const monitorId = btn.dataset.monitorId;
    if (!monitorId) {
      console.error('[Monitors] Remove clicked but data-monitor-id missing');
      return;
    }
    btn.disabled = true;
    removeMonitorById(monitorId).finally(() => {
      // List may have been rebuilt; ignore if node detached
      if (btn.isConnected) btn.disabled = false;
    });
  });
}

async function refreshMonitorsSettingsList() {
  const settingsList = document.getElementById('monitors-settings-list');
  if (!settingsList) return;
  
  settingsList.innerHTML = '';
  
  try {
    const monitorIds = await invoke('list_monitors');
    
    if (monitorIds.length === 0) {
      settingsList.innerHTML = '<div class="monitors-empty">No monitors configured</div>';
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
        removeBtn.type = 'button';
        removeBtn.className = 'monitor-remove-btn';
        removeBtn.textContent = 'Remove';
        removeBtn.dataset.monitorId = monitorId;
        removeBtn.setAttribute('aria-label', `Remove ${monitorUrl}`);
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
    settingsList.innerHTML = '<div class="monitors-empty monitors-error" role="alert">Error loading monitors</div>';
  }
}

function applyMonitorsSummaryState({ anyDown, allUp, empty }) {
  const summary = document.getElementById('monitors-summary');
  if (!summary) return;
  summary.classList.toggle('has-down', !!anyDown);
  summary.classList.toggle('is-all-up', !!allUp && !empty);
  summary.classList.toggle('is-empty', !!empty);
}

/** Short host label for Monitors summary (name preferred, else hostname). */
function shortMonitorHostLabel(name, url) {
  const raw = (name && String(name).trim()) || '';
  if (raw && !raw.startsWith('monitor_')) {
    return raw.replace(/^www\./i, '');
  }
  try {
    const host = new URL(url || '').hostname;
    if (host) return host.replace(/^www\./i, '');
  } catch (_) {
    /* ignore */
  }
  const fallback = (url || name || 'site').replace(/^https?:\/\//i, '');
  return fallback.slice(0, 40);
}

/** Prefer classified short reasons (DNS / timeout / TLS / …) for summary chips. */
function shortMonitorFailReason(error) {
  if (!error) return null;
  const e = String(error).trim();
  if (!e) return null;
  const head = e.match(
    /^(DNS|Timeout|TLS|Refused|Unreachable|Connect)(\s+[^\n|;,]{0,36})?/i
  );
  if (head) return head[0].replace(/\s+/g, ' ').trim();
  const lower = e.toLowerCase();
  for (const k of ['dns', 'timeout', 'tls', 'refused', 'unreachable', 'connect']) {
    if (lower.includes(k)) {
      return k === 'dns' ? 'DNS' : k.charAt(0).toUpperCase() + k.slice(1);
    }
  }
  return e.length > 32 ? `${e.slice(0, 29)}…` : e;
}

/** Parse checked_at / down_since into epoch ms. */
function parseMonitorTimeMs(raw) {
  if (raw == null) return null;
  if (typeof raw === 'number' && Number.isFinite(raw)) {
    return raw < 1e12 ? raw * 1000 : raw;
  }
  const parsed = Date.parse(String(raw));
  return Number.isFinite(parsed) ? parsed : null;
}

/** Format a duration like 2h 15m for downtime. */
function formatMonitorDuration(msAgo) {
  if (msAgo == null || !Number.isFinite(msAgo) || msAgo < 0) return null;
  const sec = Math.floor(msAgo / 1000);
  if (sec < 60) return `${sec}s`;
  const mins = Math.floor(sec / 60);
  if (mins < 60) return `${mins}m`;
  const hours = Math.floor(mins / 60);
  const remM = mins % 60;
  if (hours < 48) return remM ? `${hours}h ${remM}m` : `${hours}h`;
  const days = Math.floor(hours / 24);
  const remH = hours % 24;
  return remH ? `${days}d ${remH}h` : `${days}d`;
}

/**
 * Best-effort downtime start: status.extra.down_since, else contiguous DOWN streak
 * in local history, else last check (approx).
 */
function resolveMonitorDownSince(monitorId, status) {
  if (!status || status.is_up) return null;
  const pending =
    !status.response_time_ms || String(status.error || '').includes('Waiting');
  if (pending) return null;

  const extra = status.extra || {};
  const fromExtra = parseMonitorTimeMs(extra.down_since);
  if (fromExtra != null) {
    return {
      ms: fromExtra,
      approx: extra.down_since_approx === true,
      source: 'status',
    };
  }

  const hist = getMonitorHistory(monitorId);
  if (hist.length) {
    const sorted = [...hist].sort((a, b) => a.timestamp - b.timestamp);
    let start = null;
    for (let i = sorted.length - 1; i >= 0; i--) {
      if (sorted[i].is_up) break;
      start = sorted[i].timestamp;
    }
    if (start != null) {
      return { ms: start, approx: true, source: 'history' };
    }
  }

  const checked = parseMonitorTimeMs(status.checked_at);
  if (checked != null) {
    return { ms: checked, approx: true, source: 'checked_at' };
  }
  return null;
}

function formatMonitorDownSinceLabel(downInfo) {
  if (!downInfo) return null;
  const when = new Date(downInfo.ms);
  const dur = formatMonitorDuration(Date.now() - downInfo.ms);
  const clock = when.toLocaleString(undefined, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
  const prefix = downInfo.approx ? 'Down at least since' : 'Down since';
  return dur ? `${prefix} ${clock} (${dur})` : `${prefix} ${clock}`;
}

/** Hover title: failure + downtime + last check (keyboard stays in kb hint). */
function buildMonitorRowTooltip(monitorUrl, status, monitorId) {
  const lines = [monitorUrl || 'Monitor'];
  const pending =
    !status.is_up &&
    (!status.response_time_ms || String(status.error || '').includes('Waiting'));
  if (pending) {
    lines.push('Waiting for first check…');
    lines.push('Click or d for details · Enter checks now · PgUp/PgDn');
    return lines.join('\n');
  }
  if (status.is_up) {
    const ago = formatMonitorCheckedAgo(status);
    const ms = status.response_time_ms != null ? `${status.response_time_ms} ms` : null;
    lines.push(['UP', ms, ago].filter(Boolean).join(' · '));
    if (status.checked_at) {
      lines.push(`Last check: ${new Date(status.checked_at).toLocaleString()}`);
    }
  } else {
    const reason = shortMonitorFailReason(status.error) || 'error';
    lines.push(`DOWN · ${reason}`);
    const downInfo = resolveMonitorDownSince(monitorId, status);
    const downLabel = formatMonitorDownSinceLabel(downInfo);
    if (downLabel) lines.push(downLabel);
    if (status.error) lines.push(String(status.error));
    if (status.checked_at) {
      lines.push(`Last check: ${new Date(status.checked_at).toLocaleString()}`);
    }
    const backoff = formatMonitorBackoffHint(status);
    if (backoff) lines.push(backoff);
  }
  lines.push('Click or d for details · Enter / Space checks now · PgUp/PgDn');
  return lines.join('\n');
}

function applyMonitorRowTooltip(item, monitorUrl, status) {
  if (!item) return;
  const id = item.getAttribute('data-monitor-id') || '';
  item.dataset.monitorUrl = monitorUrl || '';
  // Put the summary tooltip on the header/info only — history ticks own their hover.
  item.removeAttribute('title');
  const tip = buildMonitorRowTooltip(monitorUrl, status, id);
  const header = item.querySelector('.monitor-item-header');
  const info = item.querySelector('.monitor-info');
  if (info) info.title = tip;
  if (header) header.title = tip;
}

/** Contiguous DOWN streak start at or before this history entry (epoch ms). */
function downStreakStartMs(sortedAsc, index) {
  if (!sortedAsc[index] || sortedAsc[index].is_up) return null;
  let start = sortedAsc[index].timestamp;
  for (let i = index - 1; i >= 0; i--) {
    if (sortedAsc[i].is_up) break;
    start = sortedAsc[i].timestamp;
  }
  return start;
}

function buildMonitorHistoryTickTitle(entry, sortedAsc, index) {
  const when = new Date(entry.timestamp).toLocaleString();
  if (entry.is_up) {
    return `UP · ${when}`;
  }
  const streakStart = downStreakStartMs(sortedAsc, index);
  const lines = [`DOWN · ${when}`];
  if (streakStart != null) {
    const startLabel = new Date(streakStart).toLocaleString();
    const dur = formatMonitorDuration(entry.timestamp - streakStart);
    if (streakStart === entry.timestamp) {
      lines.push(`Outage started here`);
    } else {
      lines.push(
        dur
          ? `Outage started ${startLabel} (${dur} by this check)`
          : `Outage started ${startLabel}`
      );
    }
  }
  return lines.join('\n');
}

function recentMonitorLogLines(monitorId, limit = 12) {
  const hist = getMonitorHistory(monitorId);
  if (!hist.length) return [];
  return [...hist]
    .sort((a, b) => b.timestamp - a.timestamp)
    .slice(0, limit)
    .map((e) => {
      const t = new Date(e.timestamp).toLocaleString(undefined, {
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
      });
      return `${t}  ${e.is_up ? 'UP' : 'DOWN'}`;
    });
}

function fillMonitorDetail(detail, monitorId, monitorUrl, status) {
  detail.replaceChildren();
  const pending =
    !status.is_up &&
    (!status.response_time_ms || String(status.error || '').includes('Waiting'));

  const addRow = (label, value) => {
    if (!value) return;
    const row = document.createElement('div');
    row.className = 'monitor-detail-row';
    const k = document.createElement('span');
    k.className = 'monitor-detail-k';
    k.textContent = label;
    const v = document.createElement('span');
    v.className = 'monitor-detail-v';
    v.textContent = value;
    row.appendChild(k);
    row.appendChild(v);
    detail.appendChild(row);
  };

  addRow('URL', monitorUrl);
  if (pending) {
    addRow('Status', 'Pending first check');
  } else if (status.is_up) {
    addRow('Status', 'UP');
    if (status.response_time_ms != null) addRow('Latency', `${status.response_time_ms} ms`);
  } else {
    addRow('Status', 'DOWN');
    addRow('Failure', shortMonitorFailReason(status.error) || status.error || 'error');
    const downInfo = resolveMonitorDownSince(monitorId, status);
    if (downInfo) {
      addRow(
        downInfo.approx ? 'Down at least since' : 'Down since',
        `${new Date(downInfo.ms).toLocaleString()}${
          formatMonitorDuration(Date.now() - downInfo.ms)
            ? ` (${formatMonitorDuration(Date.now() - downInfo.ms)})`
            : ''
        }`
      );
    }
    if (status.error) addRow('Detail', String(status.error));
  }
  if (status.checked_at && !pending) {
    addRow('Last check', new Date(status.checked_at).toLocaleString());
  }
  const backoff = formatMonitorBackoffHint(status);
  if (backoff) addRow('Auto-check', backoff);

  const log = recentMonitorLogLines(monitorId, 14);
  if (log.length) {
    const logTitle = document.createElement('div');
    logTitle.className = 'monitor-detail-log-title';
    logTitle.textContent = 'Recent checks (this Mac)';
    detail.appendChild(logTitle);
    const pre = document.createElement('pre');
    pre.className = 'monitor-detail-log';
    pre.textContent = log.join('\n');
    detail.appendChild(pre);
  } else {
    const note = document.createElement('div');
    note.className = 'monitor-detail-note';
    note.textContent = 'No local check history yet for this monitor.';
    detail.appendChild(note);
  }

  const actions = document.createElement('div');
  actions.className = 'monitor-detail-actions';
  const checkBtn = document.createElement('button');
  checkBtn.type = 'button';
  checkBtn.className = 'btn-secondary monitor-detail-check';
  checkBtn.textContent = 'Check now';
  checkBtn.addEventListener('click', (e) => {
    e.preventDefault();
    e.stopPropagation();
    void forceCheckMonitorNow(monitorId, detail.closest('.monitor-item'));
  });
  actions.appendChild(checkBtn);
  detail.appendChild(actions);
}

function setMonitorDetailOpen(item, open) {
  if (!item) return;
  const detail = item.querySelector('.monitor-detail');
  if (!detail) return;
  item.classList.toggle('is-detail-open', !!open);
  detail.hidden = !open;
  if (open) {
    const id = item.getAttribute('data-monitor-id');
    const url = item.dataset.monitorUrl || id;
    const status = monitorStatusCache.get(id) || {
      is_up: !item.classList.contains('is-down'),
      error: null,
      checked_at: null,
      response_time_ms: null,
      extra: {},
    };
    fillMonitorDetail(detail, id, url, status);
  }
  updateMonitorsHeight();
}

function toggleMonitorDetail(item) {
  if (!item) return;
  setMonitorDetailOpen(item, !item.classList.contains('is-detail-open'));
}

async function updateMonitorsSummary() {
  const summaryText = document.getElementById('monitors-summary-text');
  if (!summaryText) return;

  try {
    const monitorIds = await invoke('list_monitors');
    
    if (monitorIds.length === 0) {
      summaryText.textContent = 'No monitors configured';
      summaryText.removeAttribute('title');
      applyMonitorsSummaryState({ anyDown: false, allUp: false, empty: true });
      updateMonitorsIconStatus({ anyDown: false, allUp: false, upCount: 0, totalCount: 0 });
      return;
    }

    let upCount = 0;
    let downCount = 0;
    let checkedCount = 0;
    let totalResponseTime = 0;
    let responseTimeCount = 0;
    const downHints = [];
    const upLatencyHints = [];

    // Use cached status from the background monitor thread — never live-probe here.
    // Live check_monitor waits on HTTP (up to timeout_secs) and freezes window open.
    for (const monitorId of monitorIds) {
      try {
        const status = await invoke('get_monitor_status', { monitorId });
        if (!status) continue;
        monitorStatusCache.set(monitorId, status);
        checkedCount++;
        if (status.is_up) {
          upCount++;
          if (status.response_time_ms) {
            let name = monitorId;
            let url = '';
            try {
              const details = await invoke('get_monitor_details', { monitorId });
              if (details?.name) name = details.name;
              if (details?.url) url = details.url;
            } catch (_) {
              /* keep id */
            }
            const host = shortMonitorHostLabel(name, url);
            const ago = formatMonitorCheckedAgo(status);
            upLatencyHints.push({
              host,
              ms: status.response_time_ms,
              label: ago
                ? `${host} ${status.response_time_ms}ms (${ago})`
                : `${host} ${status.response_time_ms}ms`,
            });
          }
        } else {
          downCount++;
          const pending =
            !status.response_time_ms || String(status.error || '').includes('Waiting');
          if (!pending) {
            let name = monitorId;
            let url = '';
            try {
              const details = await invoke('get_monitor_details', { monitorId });
              if (details?.name) name = details.name;
              if (details?.url) url = details.url;
            } catch (_) {
              /* keep id */
            }
            const host = shortMonitorHostLabel(name, url);
            const reason = shortMonitorFailReason(status.error);
            const ago = formatMonitorCheckedAgo(status);
            const downInfo = resolveMonitorDownSince(monitorId, status);
            const downLabel = formatMonitorDownSinceLabel(downInfo);
            const base = reason ? `${host} (${reason})` : host;
            const parts = [base];
            if (downLabel) parts.push(downLabel);
            else if (ago) parts.push(ago);
            downHints.push(parts.join(' · '));
          }
        }
        if (status.response_time_ms) {
          totalResponseTime += status.response_time_ms;
          responseTimeCount++;
        }
      } catch (err) {
        console.error(`Failed to read monitor status ${monitorId}:`, err);
      }
    }

    const avgResponseTime = responseTimeCount > 0 
      ? Math.round(totalResponseTime / responseTimeCount)
      : 0;

    if (downHints.length > 0) {
      const shown = downHints.slice(0, 2);
      const more = downHints.length > 2 ? ` +${downHints.length - 2}` : '';
      summaryText.textContent =
        `${upCount} / ${monitorIds.length} up · DOWN: ${shown.join(', ')}${more}`;
      summaryText.title = downHints.join('; ');
    } else {
      upLatencyHints.sort((a, b) => b.ms - a.ms);
      const slowest = upLatencyHints[0];
      if (slowest && upLatencyHints.length >= 2) {
        summaryText.textContent =
          `${upCount} / ${monitorIds.length} sites up · Avg ${avgResponseTime} ms · slowest ${slowest.host} ${slowest.ms}ms`;
      } else {
        summaryText.textContent =
          `${upCount} / ${monitorIds.length} sites up · Avg ${avgResponseTime} ms`;
      }
      if (upLatencyHints.length > 0) {
        summaryText.title = upLatencyHints.map((h) => h.label).join('; ');
      } else {
        summaryText.removeAttribute('title');
      }
    }
    
    // Green only when every configured monitor has checked in and is up.
    // Red as soon as any checked monitor is down (pending checks stay neutral).
    const anyDown = downCount > 0;
    const allUp = checkedCount > 0 && downCount === 0 && checkedCount === monitorIds.length;
    applyMonitorsSummaryState({ anyDown, allUp, empty: false });
    updateMonitorsIconStatus({ anyDown, allUp, upCount, totalCount: monitorIds.length });
  } catch (err) {
    console.error('Failed to update monitors summary:', err);
    applyMonitorsSummaryState({ anyDown: false, allUp: false, empty: false });
    updateMonitorsIconStatus({ anyDown: false, allUp: false, upCount: 0, totalCount: 0 });
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
        
        // Cached status only — background thread owns live HTTP checks
        const status = await invoke('get_monitor_status', { monitorId });
        if (!status) {
          if (existingItems.has(monitorId)) {
            // Keep existing row until first background check lands
            continue;
          }
          const pending = { is_up: false, response_time_ms: null, error: 'Waiting for first check…' };
          monitorStatusCache.set(monitorId, pending);
          const monitorItem = createMonitorItem(monitorId, monitorUrl, pending);
          monitorsList.appendChild(monitorItem);
          continue;
        }
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
    let upCount = 0;
    let downCount = 0;
    let checkedCount = 0;
    for (const monitorId of monitorIds) {
      const status = monitorStatusCache.get(monitorId);
      if (!status) continue;
      checkedCount++;
      if (status.is_up) upCount++;
      else downCount++;
    }
    const anyDown = downCount > 0;
    const allUp = checkedCount > 0 && downCount === 0 && checkedCount === monitorIds.length;
    updateMonitorsIconStatus({ anyDown, allUp, upCount, totalCount: monitorIds.length });

    sortMonitorsListByHealth(monitorsList);
    syncMonitorsListTabOrder(monitorsList);

    // Update height after loading monitors
    updateMonitorsHeight();
  } catch (err) {
    console.error('Failed to load monitors:', err);
    updateMonitorsIconStatus({ anyDown: false, allUp: false, upCount: 0, totalCount: 0 });
  }
}

function updateMonitorsHeight() {
  const monitorsList = document.getElementById('monitors-list');
  const monitorsContent = document.getElementById('monitors-content');
  if (!monitorsList || !monitorsContent) return;
  
  // Calculate height needed: summary + each monitor item (+ open detail panels)
  const monitorItems = monitorsList.querySelectorAll('.monitor-item');
  const itemHeight = 52; // row + down-meta / error lines
  const summaryHeight = 40;
  const listMargin = 12;
  let openDetailExtra = 0;
  monitorItems.forEach((el) => {
    if (el.classList.contains('is-detail-open')) openDetailExtra += 200;
  });
  
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
  
  const totalHeight =
    summaryHeight +
    (monitorItems.length > 0
      ? listMargin + monitorItems.length * itemHeight + openDetailExtra
      : 0);
  
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

function formatMonitorBackoffHint(status) {
  if (!status || status.is_up) return null;
  const extra = status.extra || {};
  const nextRaw = extra.next_background_check_secs;
  const intervalRaw = extra.background_interval_secs;
  const next = typeof nextRaw === 'number' ? nextRaw : Number(nextRaw);
  const interval = typeof intervalRaw === 'number' ? intervalRaw : Number(intervalRaw);
  if (Number.isFinite(next) && next > 0) {
    const mins = Math.max(1, Math.ceil(next / 60));
    return mins === 1
      ? 'Next auto-check in <1 min'
      : `Next auto-check in ~${mins} min`;
  }
  if (Number.isFinite(interval) && interval >= 180) {
    return `Auto-checks every ${Math.round(interval / 60)}+ min while down`;
  }
  return null;
}

/** Relative age from checked_at (ISO / epoch ms) for Monitors rows + tooltips. */
function formatMonitorCheckedAgo(status) {
  if (!status || status.checked_at == null) return null;
  const raw = status.checked_at;
  let ms = null;
  if (typeof raw === 'number' && Number.isFinite(raw)) {
    ms = raw < 1e12 ? raw * 1000 : raw;
  } else {
    const parsed = Date.parse(String(raw));
    if (Number.isFinite(parsed)) ms = parsed;
  }
  if (ms == null) return null;
  const sec = Math.max(0, Math.floor((Date.now() - ms) / 1000));
  if (sec < 5) return 'just now';
  if (sec < 60) return `${sec}s ago`;
  const mins = Math.floor(sec / 60);
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 48) return `${hours}h ago`;
  return `${Math.floor(hours / 24)}d ago`;
}

/** Keep DOWN (then pending) above UP so failures stay visible. */
function sortMonitorsListByHealth(monitorsList) {
  if (!monitorsList) return;
  const items = Array.from(monitorsList.querySelectorAll('.monitor-item'));
  if (items.length < 2) return;
  const rank = (el) => {
    if (el.classList.contains('is-down')) return 0;
    if (el.classList.contains('is-pending')) return 1;
    return 2;
  };
  items
    .sort((a, b) => {
      const d = rank(a) - rank(b);
      if (d !== 0) return d;
      return (a.getAttribute('data-monitor-id') || '').localeCompare(
        b.getAttribute('data-monitor-id') || ''
      );
    })
    .forEach((el) => monitorsList.appendChild(el));
}

/** Roving tabindex for External / Monitors rows (process-list parity). */
function syncMonitorsListTabOrder(monitorsList, preferId) {
  if (!monitorsList) return;
  const items = Array.from(monitorsList.querySelectorAll('.monitor-item'));
  if (items.length === 0) return;
  let activeIdx = 0;
  if (preferId) {
    const hit = items.findIndex((el) => el.getAttribute('data-monitor-id') === preferId);
    if (hit >= 0) activeIdx = hit;
  } else {
    const focused = items.findIndex((el) => el === document.activeElement);
    if (focused >= 0) activeIdx = focused;
    else {
      const selected = items.findIndex((el) => el.classList.contains('is-selected'));
      if (selected >= 0) activeIdx = selected;
    }
  }
  items.forEach((el, i) => {
    el.setAttribute('tabindex', i === activeIdx ? '0' : '-1');
    el.classList.toggle('is-selected', i === activeIdx);
    const url = el.dataset.monitorUrl || el.getAttribute('data-monitor-id') || '';
    const id = el.getAttribute('data-monitor-id');
    const status = (id && monitorStatusCache.get(id)) || {
      is_up: !el.classList.contains('is-down'),
      error: null,
      checked_at: null,
      response_time_ms: el.classList.contains('is-pending') ? null : 1,
      extra: {},
    };
    applyMonitorRowTooltip(el, url, status);
  });
  ensureMonitorsListKbHint(monitorsList, items.length > 0);
}

/** Hint above Top Processes list (Monitors / Disk Cleanup kb-hint parity). */
function ensureProcessesListKbHint(processList, show) {
  if (!processList) return;
  let hint = document.getElementById('processes-kb-hint');
  if (!show) {
    hint?.remove();
    return;
  }
  if (!hint) {
    hint = document.createElement('div');
    hint.className = 'processes-kb-hint';
    hint.id = 'processes-kb-hint';
    processList.parentNode?.insertBefore(hint, processList);
  }
  hint.textContent =
    'Click row for details · ↑↓ / j k · PgUp/PgDn · Enter / d opens · P pin/unpin · Esc closes/clears';
}

/** Hint above External / Monitors list (Disk Cleanup kb-hint parity). */
function ensureMonitorsListKbHint(monitorsList, show) {
  if (!monitorsList) return;
  let hint = document.getElementById('monitors-kb-hint');
  if (!show) {
    hint?.remove();
    return;
  }
  if (!hint) {
    hint = document.createElement('div');
    hint.className = 'monitors-kb-hint';
    hint.id = 'monitors-kb-hint';
    monitorsList.parentNode?.insertBefore(hint, monitorsList);
  }
  hint.textContent =
    'Click row for details · ↑↓ / j k · PgUp/PgDn · Enter check now · d details · Esc closes/clears';
}

function wireMonitorsListKeyboard() {
  const monitorsList = document.getElementById('monitors-list');
  if (!monitorsList || monitorsList.dataset.keyboardNav === '1') return;
  monitorsList.dataset.keyboardNav = '1';
  monitorsList.setAttribute('role', 'listbox');
  monitorsList.setAttribute('aria-label', 'External monitors');

  monitorsList.addEventListener('click', (e) => {
    const item = e.target && e.target.closest && e.target.closest('.monitor-item');
    if (!item || !monitorsList.contains(item)) return;
    if (e.target.closest && e.target.closest('.monitor-detail-check')) return;
    const id = item.getAttribute('data-monitor-id');
    syncMonitorsListTabOrder(monitorsList, id);
    item.focus();
    toggleMonitorDetail(item);
  });

  monitorsList.addEventListener('keydown', (e) => {
    const item = e.target && e.target.closest && e.target.closest('.monitor-item');
    if (!item || !monitorsList.contains(item)) return;
    const items = Array.from(monitorsList.querySelectorAll('.monitor-item'));
    const idx = items.indexOf(item);
    if (idx < 0) return;

    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      const monitorId = item.getAttribute('data-monitor-id');
      if (monitorId) void forceCheckMonitorNow(monitorId, item);
      return;
    }

    // d toggles the detail panel (mouse still uses click).
    if (e.key === 'd' || e.key === 'D') {
      e.preventDefault();
      toggleMonitorDetail(item);
      return;
    }

    // Esc: close open detail first, then clear selection (Hermes escape-skips parity).
    if (e.key === 'Escape' || e.key === 'Esc') {
      if (!item.classList.contains('is-selected') && document.activeElement !== item) {
        return;
      }
      e.preventDefault();
      e.stopPropagation();
      if (item.classList.contains('is-detail-open')) {
        setMonitorDetailOpen(item, false);
        return;
      }
      items.forEach((el) => {
        el.classList.remove('is-selected');
        el.setAttribute('tabindex', '-1');
      });
      if (items[0]) items[0].setAttribute('tabindex', '0');
      item.blur();
      return;
    }

    let next = -1;
    const page = 5;
    if (e.key === 'ArrowDown' || e.key === 'j') next = Math.min(idx + 1, items.length - 1);
    else if (e.key === 'ArrowUp' || e.key === 'k') next = Math.max(idx - 1, 0);
    else if (e.key === 'PageDown') next = Math.min(idx + page, items.length - 1);
    else if (e.key === 'PageUp') next = Math.max(idx - page, 0);
    else if (e.key === 'Home') next = 0;
    else if (e.key === 'End') next = items.length - 1;
    else return;
    e.preventDefault();
    if (next < 0 || next === idx) return;
    const preferId = items[next].getAttribute('data-monitor-id');
    syncMonitorsListTabOrder(monitorsList, preferId);
    items[next].focus();
    if (typeof items[next].scrollIntoView === 'function') {
      items[next].scrollIntoView({ block: 'nearest' });
    }
  });
}

/** Manual check: bypasses background DOWN backoff (UI was never wired to check_monitor). */
async function forceCheckMonitorNow(monitorId, itemEl) {
  if (!monitorId) return;
  const findRow = () =>
    Array.from(document.querySelectorAll('.monitor-item')).find(
      (el) => el.getAttribute('data-monitor-id') === monitorId
    );
  const item = itemEl || findRow();
  if (item?.dataset.checking === '1') return;
  if (item) {
    item.dataset.checking = '1';
    item.classList.add('is-checking');
    const latencyEl = item.querySelector('.monitor-latency');
    if (latencyEl) latencyEl.textContent = '…';
  }
  try {
    const status = await invoke('check_monitor', { monitorId });
    if (status) {
      monitorStatusCache.set(monitorId, status);
      addMonitorHistoryEntry(monitorId, status.is_up);
      let monitorUrl = monitorId;
      try {
        const details = await invoke('get_monitor_details', { monitorId });
        if (details?.url) monitorUrl = details.url;
      } catch (_) {
        /* keep id */
      }
      const row = item || findRow();
      if (row) updateMonitorItem(row, monitorId, monitorUrl, status);
    }
    await updateMonitorsSummary();
    const list = document.getElementById('monitors-list');
    sortMonitorsListByHealth(list);
    syncMonitorsListTabOrder(list, monitorId);
    updateMonitorsHeight();
  } catch (err) {
    console.error(`[Monitors] check_monitor failed for ${monitorId}:`, err);
  } finally {
    const row = item || findRow();
    if (row) {
      row.dataset.checking = '0';
      row.classList.remove('is-checking');
    }
  }
}

function fillMonitorInfo(info, monitorUrl, status, monitorId) {
  const responseTimeText = status.response_time_ms ? `${status.response_time_ms}ms` : '--';
  const pending =
    !status.is_up &&
    (!status.response_time_ms || String(status.error || '').includes('Waiting'));
  info.replaceChildren();

  const primary = document.createElement('div');
  primary.className = 'monitor-info-primary';

  const urlEl = document.createElement('span');
  urlEl.className = 'monitor-url';
  urlEl.textContent = monitorUrl;

  const latencyEl = document.createElement('span');
  latencyEl.className = 'monitor-latency';
  latencyEl.textContent = responseTimeText;

  primary.appendChild(urlEl);
  primary.appendChild(latencyEl);

  const ago = formatMonitorCheckedAgo(status);
  if (ago && !pending) {
    const agoEl = document.createElement('span');
    agoEl.className = 'monitor-checked-ago';
    agoEl.textContent = ago;
    agoEl.title = status.checked_at
      ? `Last check: ${new Date(status.checked_at).toLocaleString()}`
      : 'Last check';
    primary.appendChild(agoEl);
  }

  info.appendChild(primary);

  if (!status.is_up && !pending) {
    const reason = shortMonitorFailReason(status.error);
    const downInfo = resolveMonitorDownSince(monitorId, status);
    const downLabel = formatMonitorDownSinceLabel(downInfo);
    if (reason || downLabel) {
      const meta = document.createElement('div');
      meta.className = 'monitor-down-meta';
      meta.textContent = [reason, downLabel].filter(Boolean).join(' · ');
      info.appendChild(meta);
    }
  }

  if (status.error && !pending) {
    const errEl = document.createElement('div');
    errEl.className = 'monitor-error';
    errEl.textContent = status.error;
    info.appendChild(errEl);
  } else if (pending && status.error) {
    const errEl = document.createElement('div');
    errEl.className = 'monitor-error';
    errEl.textContent = status.error;
    info.appendChild(errEl);
  }

  const backoffHint = !pending ? formatMonitorBackoffHint(status) : null;
  if (backoffHint) {
    const hintEl = document.createElement('div');
    hintEl.className = 'monitor-backoff-hint';
    hintEl.textContent = backoffHint;
    info.appendChild(hintEl);
  }
}

function applyMonitorItemState(item, status) {
  const pending =
    !status.is_up &&
    (!status.response_time_ms || String(status.error || '').includes('Waiting'));
  item.classList.toggle('is-down', !status.is_up && !pending);
  item.classList.toggle('is-pending', pending);
}

function createMonitorItem(monitorId, monitorUrl, status) {
  const item = document.createElement('div');
  item.className = 'monitor-item';
  item.setAttribute('data-monitor-id', monitorId);
  item.tabIndex = -1;
  item.setAttribute('role', 'option');
  item.setAttribute('aria-label', `Monitor ${monitorUrl}`);
  applyMonitorItemState(item, status);
  applyMonitorRowTooltip(item, monitorUrl, status);
  
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
  fillMonitorInfo(info, monitorUrl, status, monitorId);

  header.appendChild(statusIndicator);
  header.appendChild(info);
  
  // Add history visualization
  const historyContainer = document.createElement('div');
  historyContainer.className = 'monitor-history';
  historyContainer.setAttribute('data-monitor-id', monitorId);
  updateMonitorHistory(historyContainer, monitorId);

  const detail = document.createElement('div');
  detail.className = 'monitor-detail';
  detail.hidden = true;

  item.appendChild(header);
  item.appendChild(historyContainer);
  item.appendChild(detail);
  
  return item;
}

function updateMonitorItem(item, monitorId, monitorUrl, status) {
  applyMonitorItemState(item, status);
  if (!item.hasAttribute('tabindex')) item.tabIndex = -1;
  if (!item.getAttribute('role')) item.setAttribute('role', 'option');
  item.setAttribute('aria-label', `Monitor ${monitorUrl}`);
  applyMonitorRowTooltip(item, monitorUrl, status);

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
    fillMonitorInfo(info, monitorUrl, status, monitorId);
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

  let detail = item.querySelector('.monitor-detail');
  if (!detail) {
    detail = document.createElement('div');
    detail.className = 'monitor-detail';
    detail.hidden = true;
    item.appendChild(detail);
  }
  if (item.classList.contains('is-detail-open')) {
    fillMonitorDetail(detail, monitorId, monitorUrl, status);
    detail.hidden = false;
  }
}

// Update the history visualization for a monitor
function hideMonitorTickTip() {
  const tip = document.getElementById('monitor-tick-tip');
  if (tip) tip.hidden = true;
}

function showMonitorTickTip(anchor, text) {
  let tip = document.getElementById('monitor-tick-tip');
  if (!tip) {
    tip = document.createElement('div');
    tip.id = 'monitor-tick-tip';
    tip.className = 'monitor-tick-tip';
    tip.setAttribute('role', 'tooltip');
    document.body.appendChild(tip);
  }
  tip.textContent = text;
  tip.hidden = false;
  const r = anchor.getBoundingClientRect();
  const x = r.left + r.width / 2;
  const y = r.top;
  tip.style.left = `${Math.round(x)}px`;
  tip.style.top = `${Math.round(y)}px`;
  // Keep on screen horizontally
  const tw = tip.offsetWidth || 160;
  const minX = tw / 2 + 8;
  const maxX = window.innerWidth - tw / 2 - 8;
  tip.style.left = `${Math.round(Math.min(maxX, Math.max(minX, x)))}px`;
}

function updateMonitorHistory(container, monitorId) {
  hideMonitorTickTip();
  const history = getMonitorHistory(monitorId);

  if (history.length === 0) {
    container.innerHTML = '';
    return;
  }

  // Fit ticks inside the card width (3px bar + 1px gap).
  const widthPx = container.clientWidth || container.parentElement?.clientWidth || 320;
  const maxByWidth = Math.max(48, Math.floor(widthPx / 4));
  const maxLines = Math.min(160, maxByWidth);

  const sortedHistory = [...history].sort((a, b) => a.timestamp - b.timestamp);
  let dataPoints = sortedHistory;
  let indexMap = null;
  if (sortedHistory.length > maxLines) {
    const step = Math.floor(sortedHistory.length / maxLines);
    dataPoints = [];
    indexMap = [];
    for (let i = 0; i < sortedHistory.length; i += step) {
      dataPoints.push(sortedHistory[i]);
      indexMap.push(i);
    }
    if (dataPoints[dataPoints.length - 1] !== sortedHistory[sortedHistory.length - 1]) {
      dataPoints.push(sortedHistory[sortedHistory.length - 1]);
      indexMap.push(sortedHistory.length - 1);
    }
  }

  container.replaceChildren();
  dataPoints.forEach((entry, i) => {
    const line = document.createElement('span');
    line.className = 'monitor-history-line';
    line.classList.add(entry.is_up ? 'up' : 'down');
    const histIdx = indexMap ? indexMap[i] : i;
    const tipText = buildMonitorHistoryTickTitle(entry, sortedHistory, histIdx);
    line.setAttribute(
      'aria-label',
      entry.is_up
        ? `Up at ${new Date(entry.timestamp).toLocaleString()}`
        : `Down at ${new Date(entry.timestamp).toLocaleString()}`
    );
    // No native title — OS balloons sit below the cursor and feel “off” the tick.
    line.addEventListener('mouseenter', () => showMonitorTickTip(line, tipText));
    line.addEventListener('mouseleave', hideMonitorTickTip);
    container.appendChild(line);
  });

  // First paint may have width 0; relayout once so we don't overflow the card.
  if (!(container.clientWidth > 40) && container.dataset.histRelayout !== '1') {
    container.dataset.histRelayout = '1';
    requestAnimationFrame(() => {
      container.dataset.histRelayout = '0';
      updateMonitorHistory(container, monitorId);
    });
  }
}

if (!window.__monitorTickTipScrollBound) {
  window.__monitorTickTipScrollBound = true;
  window.addEventListener('scroll', hideMonitorTickTip, true);
  window.addEventListener('blur', hideMonitorTickTip);
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

function updateMonitorsIconStatus({ anyDown = false, allUp = false, upCount = 0, totalCount = 0 } = {}) {
  const monitorsIcon = document.getElementById('icon-monitors');
  if (monitorsIcon) {
    monitorsIcon.classList.remove('status-good', 'status-bad');
    if (anyDown) {
      // At least one monitor is down — red icon
      monitorsIcon.classList.add('status-bad');
      monitorsIcon.title = `Monitors: ${upCount}/${totalCount} up — one or more down`;
    } else if (allUp && totalCount > 0) {
      monitorsIcon.classList.add('status-good');
      monitorsIcon.title = `Monitors: all ${totalCount} up`;
    } else if (totalCount > 0) {
      monitorsIcon.title = `Monitors: ${upCount}/${totalCount} up (checking…)`;
    } else {
      monitorsIcon.title = 'Monitors';
    }
  }

  // Collapsed-section status dot matches icon color
  const statusDot = document.querySelector('.monitors-status-dot');
  if (statusDot) {
    statusDot.classList.toggle('down', !!anyDown);
    statusDot.classList.toggle('ok', !!(allUp && totalCount > 0 && !anyDown));
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

function updateDiscordIconStatus(connected) {
  const discordIcon = document.getElementById('icon-discord');
  if (!discordIcon) return;
  if (connected) {
    discordIcon.classList.add('status-good');
    discordIcon.title = 'Discord connected — click to disconnect';
  } else {
    discordIcon.classList.remove('status-good');
    discordIcon.title = 'Discord disconnected — click to connect';
  }
}

async function refreshDiscordIconStatus() {
  const inv = getInvoke() || invoke;
  if (!inv) return;
  try {
    const ready = await inv('is_discord_gateway_ready');
    updateDiscordIconStatus(!!ready);
  } catch (err) {
    updateDiscordIconStatus(false);
  }
}

let discordToggleInFlight = false;

async function toggleDiscordGatewayFromIcon() {
  if (discordToggleInFlight) return;
  const inv = getInvoke() || invoke;
  if (!inv) return;
  discordToggleInFlight = true;
  const discordIcon = document.getElementById('icon-discord');
  try {
    let ready = false;
    try {
      ready = !!(await inv('is_discord_gateway_ready'));
    } catch (_) {
      ready = false;
    }
    if (ready) {
      updateDiscordIconStatus(false);
      if (discordIcon) discordIcon.title = 'Discord disconnecting…';
      await inv('set_discord_gateway_enabled', { enabled: false });
      updateDiscordIconStatus(false);
      console.log('[CPU] Discord gateway disabled via icon');
    } else {
      if (discordIcon) discordIcon.title = 'Discord connecting…';
      await inv('set_discord_gateway_enabled', { enabled: true });
      // Poll briefly until Ready (or timeout) so the icon turns green
      for (let i = 0; i < 20; i++) {
        await new Promise((r) => setTimeout(r, 500));
        const nowReady = !!(await inv('is_discord_gateway_ready'));
        if (nowReady) {
          updateDiscordIconStatus(true);
          console.log('[CPU] Discord gateway ready via icon');
          return;
        }
      }
      await refreshDiscordIconStatus();
      console.log('[CPU] Discord gateway enable requested (still connecting or failed)');
    }
  } catch (err) {
    console.error('[CPU] Discord toggle failed:', err);
    await refreshDiscordIconStatus();
  } finally {
    discordToggleInFlight = false;
  }
}

function initDiscordIconStatus() {
  const discordIcon = document.getElementById('icon-discord');
  if (discordIcon) {
    discordIcon.addEventListener('click', (e) => {
      e.stopPropagation();
      toggleDiscordGatewayFromIcon();
    });
  }
  refreshDiscordIconStatus();
  setInterval(refreshDiscordIconStatus, 5000);
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

const DEFAULT_SYSTEM_PROMPT = 'You are a helpful assistant that answers questions, a general purpose AI, you are extremly well about system metrics and monitoring.\n\
### Super Powers\n\
You have super powers: you can execute javascript code to get real-time information.\n\
In case your ansswer would container a [variable-name], you must answer using ROLE=code-assistant.\n\
In case your need realtime information, you must answer using ROLE=code-assistant.\n\
In case you find it usefull to execute code, e.g. someone asks you for todays date or what ever that can be answered with javascript, you can answer using ROLE=code-assistant\n --oneliner of pure javascript code without anyformatting or comments.\n\
Your code-request, will be execute somewhere else and the result will be returned to you, to use it in your answer.\n\
Important: only oneliner, no comments, no formatting, no nothing.\n\
Do not add any other words around it. \n\
Do not insert formatting. Only return the code to be executed.\n\
This is needed for the next AI to understand and execute the same.\n\
When answering, use the ROLE=code-assistant to signle the execution of the code in the response.\n\
\n\
Keep it simple and concise.\n\
';

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
    const openUrlDialog = () => {
      if (!connectionIndicator.classList.contains('connected')) {
        if (window.Ollama) {
          window.Ollama.showUrlDialog();
        } else {
          showOllamaUrlDialog(); // Fallback
        }
      }
    };
    connectionIndicator.addEventListener('click', (e) => {
      e.stopPropagation();
      openUrlDialog();
    });
    connectionIndicator.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        e.stopPropagation();
        openUrlDialog();
      }
    });
  }

  // Model text click handler - show dropdown
  if (modelText) {
    modelText.addEventListener('click', (e) => {
      e.stopPropagation();
      toggleModelDropdown();
    });
    modelText.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        e.stopPropagation();
        toggleModelDropdown();
      }
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

  const applyOllamaCollapsed = () => {
    const section = document.querySelector('.ollama-section');
    const divider = document.getElementById('monitors-ollama-divider');
    const chat = document.getElementById('ollama-chat');

    if (ollamaCollapsed) {
      content.classList.add('collapsed');
      if (section) {
        section.classList.add('collapsed');
      }
      if (divider) {
        divider.style.display = 'none';
      }
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
    if (header._syncCollapseA11y) header._syncCollapseA11y();
  };

  wireCollapsibleHeaderA11y(header, {
    contentId: 'ollama-content',
    getExpanded: () => !ollamaCollapsed,
    ignoreSelector: '#ollama-menu-btn, #ollama-menu, #ollama-connection-indicator, #ollama-model-text, #ollama-model-select',
    onToggle: () => {
      ollamaCollapsed = !ollamaCollapsed;
      applyOllamaCollapsed();
    },
  });

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
    applyOllamaCollapsed();
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
    menuBtn.setAttribute('aria-haspopup', 'menu');
    menuBtn.setAttribute('aria-expanded', 'false');
    menuBtn.setAttribute('aria-controls', 'ollama-menu');
    menu.setAttribute('role', 'menu');
    menu.setAttribute('aria-hidden', 'true');
    menu.querySelectorAll('.ollama-menu-item').forEach((item) => {
      item.setAttribute('role', 'menuitem');
    });

    // Update menu text based on current state
    const updateOllamaMenuText = () => {
      const menuCollapseEl = document.getElementById('ollama-menu-collapse');
      if (menuCollapseEl) {
        menuCollapseEl.textContent = ollamaCollapsed ? 'Expand' : 'Collapse';
      }
    };
    updateOllamaMenuText();

    const setOllamaMenuOpen = (open) => {
      menu.style.display = open ? 'block' : 'none';
      menu.setAttribute('aria-hidden', String(!open));
      menuBtn.setAttribute('aria-expanded', String(open));
      if (open) {
        requestAnimationFrame(() => {
          const rect = menuBtn.getBoundingClientRect();
          menu.style.position = 'fixed';
          menu.style.top = `${rect.top}px`;
          menu.style.left = `${rect.right + 2}px`;
          menu.style.transform = 'none';
          updateOllamaMenuText();
          menu.querySelector('.ollama-menu-item')?.focus();
        });
      } else if (document.activeElement && menu.contains(document.activeElement)) {
        menuBtn.focus();
      }
    };
    
    // Toggle menu on button click
    menuBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      e.preventDefault();
      const isVisible = menu.style.display !== 'none';
      setOllamaMenuOpen(!isVisible);
    });
    
    // Close menu when clicking outside
    document.addEventListener('click', (e) => {
      if (menu && !menu.contains(e.target) && !menuBtn.contains(e.target)) {
        setOllamaMenuOpen(false);
      }
    });

    menu.addEventListener('keydown', (e) => {
      if (e.key !== 'Escape') return;
      e.preventDefault();
      e.stopPropagation();
      setOllamaMenuOpen(false);
    });

    window.closeOllamaOptionsMenu = () => setOllamaMenuOpen(false);
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
      if (window.closeOllamaOptionsMenu) window.closeOllamaOptionsMenu();
      else menu.style.display = 'none';
      // Toggle collapse
      ollamaCollapsed = !ollamaCollapsed;
      applyOllamaCollapsed();
      updateMenuText();
    });
  }
  
  if (menuSettings) {
    menuSettings.addEventListener('click', (e) => {
      e.stopPropagation();
      e.preventDefault();
      if (window.closeOllamaOptionsMenu) window.closeOllamaOptionsMenu();
      else menu.style.display = 'none';
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
      closeOllamaSettingsPopover();
    });
  }
  
  if (settingsSave) {
    settingsSave.addEventListener('click', () => {
      if (systemPromptTextarea) {
        const prompt = systemPromptTextarea.value.trim();
        saveSystemPrompt(prompt || DEFAULT_SYSTEM_PROMPT);
        closeOllamaSettingsPopover();
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
        closeOllamaSettingsPopover();
      }
    });
  }
  
  // Load saved system prompt into textarea if it exists
  if (systemPromptTextarea) {
    systemPromptTextarea.value = getSystemPrompt();
  }
}

function closeOllamaSettingsPopover() {
  const popover = document.getElementById('ollama-settings-popover');
  if (popover) {
    popover.style.display = 'none';
    popover.setAttribute('aria-hidden', 'true');
  }
  const returnEl = ollamaSettingsFocusReturn;
  ollamaSettingsFocusReturn = null;
  if (returnEl && typeof returnEl.focus === 'function') {
    try {
      returnEl.focus();
    } catch (_) {
      /* ignore */
    }
  }
}

function showSystemPromptSettings() {
  const popover = document.getElementById('ollama-settings-popover');
  const textarea = document.getElementById('ollama-system-prompt');
  if (popover && textarea) {
    ollamaSettingsFocusReturn = document.activeElement;
    textarea.value = getSystemPrompt();
    popover.style.display = 'flex';
    popover.setAttribute('role', 'dialog');
    popover.setAttribute('aria-modal', 'true');
    popover.setAttribute('aria-hidden', 'false');
    const title = popover.querySelector('.popover-header h3');
    if (title) {
      if (!title.id) title.id = 'ollama-settings-title';
      popover.setAttribute('aria-labelledby', title.id);
    }
    // Focus close for consistent dialog pattern; textarea remains one Tab away
    requestAnimationFrame(() => {
      document.getElementById('ollama-settings-close')?.focus();
    });
  }
}

function getDefaultModel() {
  // Prefer saved selection; validity against installed models is enforced in ollama.js.
  return localStorage.getItem('ollama_model') || '';
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

    const savedModel = localStorage.getItem('ollama_model');
    let selectedModel =
      savedModel && models.includes(savedModel) ? savedModel : models[0];

    models.forEach(model => {
      const option = document.createElement('option');
      option.value = model;
      option.textContent = model;
      if (model === selectedModel) {
        option.selected = true;
      }
      modelSelect.appendChild(option);
    });

    saveSelectedModel(selectedModel);

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
    try {
      // Delegate entirely to ollama.js — it validates the model against /api/tags.
      // Do not pass cpu.js's sync getDefaultModel() (stale localStorage like qwen2.5:1.5b).
      await window.Ollama.autoConfigure();
      setTimeout(async () => {
        try {
          const connected = await checkOllamaConnection();
          if (connected) {
            await loadAvailableModels();
            updateOllamaIconStatus('connected');
          } else {
            updateOllamaIconStatus('unknown');
          }
        } catch (checkErr) {
          console.error('[Ollama] Connection check failed:', checkErr);
          updateOllamaIconStatus('error');
        }
      }, 500);
    } catch (err) {
      console.error('[Ollama] Failed to auto-configure:', err);
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
// Universal implementation that works across all themes using IDs
function initCollapsibleSections() {
  // Get collapsed state from localStorage (default to true - hidden)
  const sectionsCollapsed = localStorage.getItem('details_processes_collapsed') !== 'false';
  
  // Use IDs for universal theme support (fallback to class selectors for backward compatibility)
  const detailsSection = document.getElementById('details-section') || 
                         document.querySelector('.apple-details, .arch-details, .swiss-details, .mat-details, .cpu-details, .details-section');
  const processesSection = document.getElementById('processes-section') || 
                           document.querySelector('.apple-processes, .arch-processes, .swiss-processes, .mat-processes, .cpu-processes, .processes-section');
  const detailsDivider = detailsSection?.previousElementSibling;
  const processesDivider = processesSection?.previousElementSibling;
  const detailsHeader = document.getElementById('details-header');
  const processesHeader = document.getElementById('processes-header');
  const usageCard = document.getElementById('cpu-usage-card');
  
  // Helper to check if element is a divider (works across themes)
  function isDivider(el) {
    if (!el) return false;
    return el.classList.contains('apple-divider') || 
           el.classList.contains('arch-rule') || 
           el.classList.contains('panel-divider') ||
           el.classList.contains('swiss-rule') ||
           el.classList.contains('mat-divider') ||
           el.classList.contains('theme-divider') ||
           el.getAttribute('aria-hidden') === 'true';
  }
  
  // Hide Details section
  function hideDetails() {
    if (detailsSection) detailsSection.style.display = 'none';
    if (detailsDivider && isDivider(detailsDivider)) {
      detailsDivider.style.display = 'none';
    }
  }
  
  // Show Details section
  function showDetails() {
    if (detailsSection) detailsSection.style.display = '';
    if (detailsDivider && isDivider(detailsDivider)) {
      detailsDivider.style.display = '';
    }
  }
  
  // Hide Processes section
  function hideProcesses() {
    if (processesSection) processesSection.style.display = 'none';
    if (processesDivider && isDivider(processesDivider)) {
      processesDivider.style.display = 'none';
    }
  }
  
  // Show Processes section
  function showProcesses() {
    if (processesSection) processesSection.style.display = '';
    if (processesDivider && isDivider(processesDivider)) {
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

  window.hideDetailsProcessesSections = hideSections;
  window.showDetailsProcessesSections = showSections;
  
  // Apply initial state (hidden by default)
  if (sectionsCollapsed) {
    hideSections();
  } else {
    showSections();
  }
  
  // Details header click - hide Details section
  if (detailsHeader) {
    detailsHeader.setAttribute('role', 'button');
    detailsHeader.setAttribute('aria-label', 'Hide Details section');
    if (!detailsHeader.hasAttribute('tabindex')) detailsHeader.setAttribute('tabindex', '0');
    const hideDetailsAction = (e) => {
      e.stopPropagation();
      hideDetails();
    };
    detailsHeader.addEventListener('click', hideDetailsAction);
    detailsHeader.addEventListener('keydown', (e) => {
      if (e.key !== 'Enter' && e.key !== ' ') return;
      e.preventDefault();
      hideDetailsAction(e);
    });
  }
  
  // Processes header click - hide Processes section
  if (processesHeader) {
    processesHeader.setAttribute('role', 'button');
    processesHeader.setAttribute('aria-label', 'Hide Processes section');
    if (!processesHeader.hasAttribute('tabindex')) processesHeader.setAttribute('tabindex', '0');
    const hideProcessesAction = (e) => {
      e.stopPropagation();
      hideProcesses();
    };
    processesHeader.addEventListener('click', hideProcessesAction);
    processesHeader.addEventListener('keydown', (e) => {
      if (e.key !== 'Enter' && e.key !== ' ') return;
      e.preventDefault();
      hideProcessesAction(e);
    });
  }
  
  // Usage card click - toggle both sections (open/close)
  if (usageCard) {
    usageCard.setAttribute('role', 'button');
    usageCard.setAttribute('tabindex', '0');
    const syncUsageExpanded = () => {
      const hidden =
        detailsSection?.style.display === 'none' ||
        processesSection?.style.display === 'none';
      usageCard.setAttribute('aria-expanded', String(!hidden));
      usageCard.setAttribute(
        'aria-label',
        hidden ? 'Show Details and Processes' : 'Hide Details and Processes'
      );
    };
    syncUsageExpanded();
    const toggleSections = (e) => {
      e.stopPropagation();
      const currentlyHidden =
        detailsSection?.style.display === 'none' ||
        processesSection?.style.display === 'none';
      if (currentlyHidden) {
        showSections();
      } else {
        hideSections();
      }
      syncUsageExpanded();
    };
    usageCard.addEventListener('click', toggleSections);
    usageCard.addEventListener('keydown', (e) => {
      if (e.key !== 'Enter' && e.key !== ' ') return;
      e.preventDefault();
      toggleSections(e);
    });
  }
}

// ============================================================================
// Perplexity Search Section
// ============================================================================
const PERPLEXITY_KEYCHAIN_ACCOUNT = 'perplexity_api_key';

/** Turn AEMET-style `|cell|cell|` Markdown tables into readable bullets for the results card. */
function formatPerplexitySnippet(raw) {
  const s = String(raw || '');
  const pipeCount = (s.match(/\|/g) || []).length;
  if (pipeCount < 4) return s;
  const rows = s.includes('\n') ? s.split(/\n/) : [s];
  const cells = [];
  for (const row of rows) {
    const t = row.trim();
    if (!t) continue;
    if (/^[\|\-\:\s]+$/.test(t)) continue;
    for (const part of t.split('|')) {
      let c = part.trim();
      if (!c || /^[\-:]+$/.test(c)) continue;
      // "06–12 h 22°C" → "06–12 h · 22°C"
      const m = c.match(/^(.+?)\s+(\d+\s*°C)$/i);
      if (m && (m[1].includes('h') || /[–-]/.test(m[1]))) {
        c = `${m[1]} · ${m[2]}`;
      }
      cells.push(c);
    }
  }
  if (cells.length < 2) return s;
  const max = 12;
  const shown = cells.length > max
    ? cells.slice(0, max).concat([`… +${cells.length - max} more`])
    : cells;
  return shown.map((c) => `• ${c}`).join('\n');
}

function updatePerplexityConfigStatus(statusText, elId) {
  const el = document.getElementById(elId);
  if (el) el.textContent = statusText;
}

/** @type {boolean} */
let perplexityConfigured = false;
/** @type {boolean} */
let perplexityCollapsed = true;

const PERPLEXITY_API_KEY_HELP_URL = 'https://www.perplexity.ai/settings/api';

function ensurePerplexitySetupPanel() {
  const content = document.getElementById('perplexity-content');
  if (!content || document.getElementById('perplexity-setup')) return;

  const setup = document.createElement('div');
  setup.id = 'perplexity-setup';
  setup.className = 'perplexity-setup';
  setup.hidden = true;
  setup.innerHTML =
    '<p class="perplexity-setup-lead">Web search needs a Perplexity API key. Create a free key on their site, then paste it here. It is stored in the macOS Keychain.</p>' +
    '<p class="perplexity-setup-link"><a href="' + PERPLEXITY_API_KEY_HELP_URL + '" target="_blank" rel="noopener noreferrer">Get an API key at perplexity.ai/settings/api</a></p>' +
    '<div class="perplexity-setup-row">' +
    '<input type="password" id="perplexity-inline-key" class="perplexity-inline-key" placeholder="Paste API key (pplx-…)" autocomplete="off" />' +
    '<button type="button" id="perplexity-inline-save" class="perplexity-inline-save">Save key</button>' +
    '</div>' +
    '<p class="perplexity-setup-note" id="perplexity-setup-note" hidden></p>';
  content.insertBefore(setup, content.firstChild);

  const inlineSave = document.getElementById('perplexity-inline-save');
  const inlineKey = document.getElementById('perplexity-inline-key');
  if (inlineSave && inlineKey) {
    const saveInline = async () => {
      const invoke = getInvoke();
      if (!invoke) return;
      const key = inlineKey.value.trim();
      const note = document.getElementById('perplexity-setup-note');
      if (!key) {
        if (note) {
          note.hidden = false;
          note.textContent = 'Paste a key first (it usually starts with pplx-).';
        }
        return;
      }
      try {
        await invoke('store_credential', {
          request: { account: PERPLEXITY_KEYCHAIN_ACCOUNT, password: key },
        });
        inlineKey.value = '';
        if (note) note.hidden = true;
        if (typeof flashSaveButton === 'function') flashSaveButton(inlineSave);
        await refreshPerplexityStatus();
        if (document.getElementById('perplexity-query')) {
          document.getElementById('perplexity-query').focus();
        }
      } catch (e) {
        console.error('Perplexity save key:', e);
        if (note) {
          note.hidden = false;
          note.textContent = 'Could not save key: ' + String(e);
        }
      }
    };
    inlineSave.addEventListener('click', saveInline);
    inlineKey.addEventListener('keydown', (ev) => {
      if (ev.key === 'Enter') {
        ev.preventDefault();
        saveInline();
      }
    });
  }
}

function updatePerplexitySetupVisibility() {
  ensurePerplexitySetupPanel();
  const setup = document.getElementById('perplexity-setup');
  const content = document.getElementById('perplexity-content');
  const searchBox = content ? content.querySelector('.perplexity-search-box') : null;
  const resultsEl = document.getElementById('perplexity-results');
  const showSetup = !perplexityCollapsed && !perplexityConfigured;
  if (setup) setup.hidden = !showSetup;
  if (searchBox) searchBox.hidden = showSetup;
  if (resultsEl) resultsEl.hidden = showSetup;
  if (showSetup) {
    const inlineKey = document.getElementById('perplexity-inline-key');
    if (inlineKey) {
      setTimeout(() => inlineKey.focus(), 50);
    }
  }
}

async function refreshPerplexityStatus() {
  const invoke = getInvoke();
  if (!invoke) return;
  try {
    const configured = await invoke('is_perplexity_configured');
    perplexityConfigured = !!configured;
    // Header: never shout "No API key" on a fresh install — hide until section is open + key exists.
    const headerStatus = document.getElementById('perplexity-config-status');
    if (headerStatus) {
      if (perplexityCollapsed || !perplexityConfigured) {
        headerStatus.textContent = '';
        headerStatus.hidden = true;
      } else {
        headerStatus.hidden = false;
        headerStatus.textContent = 'API key set';
      }
    }
    updatePerplexityConfigStatus(
      perplexityConfigured ? 'Key set' : 'No key',
      'perplexity-settings-status'
    );
  } catch (_) {
    perplexityConfigured = false;
    const headerStatus = document.getElementById('perplexity-config-status');
    if (headerStatus) {
      headerStatus.textContent = '';
      headerStatus.hidden = true;
    }
    updatePerplexityConfigStatus('—', 'perplexity-settings-status');
  }
  updatePerplexitySetupVisibility();
}

function initPerplexitySection() {
  const header = document.getElementById('perplexity-header');
  const content = document.getElementById('perplexity-content');
  const section = document.querySelector('.perplexity-section');
  const divider = document.getElementById('perplexity-details-divider');
  const searchBtn = document.getElementById('perplexity-search-btn');
  const queryInput = document.getElementById('perplexity-query');
  const resultsEl = document.getElementById('perplexity-results');

  if (!header || !content) return;

  perplexityCollapsed = localStorage.getItem('perplexity_collapsed') !== 'false';
  const applyPerplexityCollapsed = () => {
    if (perplexityCollapsed) {
      content.classList.add('collapsed');
      if (section) section.classList.add('collapsed');
      if (divider) divider.style.display = 'none';
    } else {
      content.classList.remove('collapsed');
      if (section) section.classList.remove('collapsed');
      if (divider) divider.style.display = '';
    }
    if (header._syncCollapseA11y) header._syncCollapseA11y();
    refreshPerplexityStatus();
  };
  applyPerplexityCollapsed();

  wireCollapsibleHeaderA11y(header, {
    contentId: 'perplexity-content',
    getExpanded: () => !perplexityCollapsed,
    onToggle: () => {
      perplexityCollapsed = !perplexityCollapsed;
      localStorage.setItem('perplexity_collapsed', perplexityCollapsed.toString());
      applyPerplexityCollapsed();
    },
  });

  header.addEventListener('click', () => {
    perplexityCollapsed = !perplexityCollapsed;
    localStorage.setItem('perplexity_collapsed', perplexityCollapsed.toString());
    applyPerplexityCollapsed();
  });

  if (searchBtn && queryInput && resultsEl) {
    searchBtn.addEventListener('click', async () => {
      const query = queryInput.value.trim();
      if (!query) return;
      const invoke = getInvoke();
      if (!invoke) {
        resultsEl.innerHTML = '<div class="perplexity-empty" role="status">App not ready.</div>';
        return;
      }
      if (!perplexityConfigured) {
        await refreshPerplexityStatus();
        updatePerplexitySetupVisibility();
        return;
      }
      resultsEl.innerHTML = '<div class="perplexity-empty" role="status">Searching…</div>';
      try {
        const resp = await invoke('perplexity_search', { request: { query: query, max_results: 10 } });
        const esc = (window.Ollama && window.Ollama.escapeHtml)
          ? window.Ollama.escapeHtml
          : function (t) {
              const d = document.createElement('div');
              d.textContent = t == null ? '' : String(t);
              return d.innerHTML;
            };
        let weatherHtml = '';
        if (resp.weather_markdown) {
          let body = '';
          if (typeof marked !== 'undefined') {
            try {
              marked.setOptions({ breaks: true, gfm: true });
              body = marked.parse(String(resp.weather_markdown));
            } catch (_) {
              body = '<pre>' + esc(resp.weather_markdown) + '</pre>';
            }
          } else {
            body = esc(String(resp.weather_markdown)).replace(/\n/g, '<br>');
          }
          weatherHtml =
            '<article class="perplexity-weather-card">' +
            '<div class="perplexity-result-meta">Live conditions · Open-Meteo</div>' +
            '<div class="perplexity-weather-body chat-message markdown-body">' + body + '</div>' +
            '</article>';
        }
        if (!resp.results || resp.results.length === 0) {
          resultsEl.innerHTML = weatherHtml ||
            '<div class="perplexity-empty" role="status">No results.</div>';
          return;
        }
        resultsEl.innerHTML = weatherHtml + resp.results.map(function (r) {
          const title = esc(r.title || 'Untitled');
          const url = esc(r.url || '#');
          const snippetRaw = formatPerplexitySnippet(
            String(r.snippet || '')
              .replace(/\\n/g, '\n')
              .replace(/\r/g, '')
          );
          let snippetHtml = '';
          if (snippetRaw) {
            const lines = snippetRaw.split('\n').map((l) => l.trim()).filter(Boolean);
            const allBullets = lines.length > 0 && lines.every((l) => l.startsWith('• '));
            if (allBullets) {
              snippetHtml = '<ul class="perplexity-result-list">' +
                lines.map((l) => '<li>' + esc(l.replace(/^•\s*/, '')) + '</li>').join('') +
                '</ul>';
            } else {
              snippetHtml = '<div class="perplexity-result-snippet">' +
                esc(snippetRaw).replace(/\n/g, '<br>') +
                '</div>';
            }
          }
          let domain = '';
          try {
            domain = r.url ? new URL(r.url).hostname.replace(/^www\./, '') : '';
          } catch (_) {}
          const date = r.date || r.last_updated || '';
          const meta = [domain, date].filter(Boolean).join(' · ');
          return '<article class="perplexity-result-item">' +
            '<a href="' + url + '" target="_blank" rel="noopener noreferrer">' + title + '</a>' +
            (meta ? '<div class="perplexity-result-meta">' + esc(meta) + '</div>' : '') +
            snippetHtml +
            '</article>';
        }).join('');
      } catch (err) {
        resultsEl.innerHTML = '<div class="perplexity-empty perplexity-empty-error" role="alert">Error: ' + String(err) + '</div>';
      }
    });
  }

  // Settings: Save / Clear API key (only if elements exist, e.g. Apple theme)
  const saveBtn = document.getElementById('perplexity-save-key');
  const clearBtn = document.getElementById('perplexity-clear-key');
  const keyInput = document.getElementById('perplexity-api-key-input');
  if (saveBtn && keyInput) {
    saveBtn.addEventListener('click', async () => {
      const invoke = getInvoke();
      if (!invoke) return;
      const key = keyInput.value.trim();
      try {
        await invoke('store_credential', { request: { account: PERPLEXITY_KEYCHAIN_ACCOUNT, password: key } });
        keyInput.value = '';
        if (typeof flashSaveButton === 'function') flashSaveButton(saveBtn);
        await refreshPerplexityStatus();
      } catch (e) {
        console.error('Perplexity save key:', e);
      }
    });
  }
  if (clearBtn) {
    clearBtn.addEventListener('click', async () => {
      const invoke = getInvoke();
      if (!invoke) return;
      try {
        await invoke('delete_credential', { account: PERPLEXITY_KEYCHAIN_ACCOUNT });
        if (keyInput) keyInput.value = '';
        await refreshPerplexityStatus();
      } catch (e) {
        console.error('Perplexity clear key:', e);
      }
    });
  }

  refreshPerplexityStatus();
  window.Perplexity = { refreshStatus: refreshPerplexityStatus };
}

let logsAutoRefreshTimer = null;

async function refreshLogsViewer(scrollToEnd = true) {
  const viewer = document.getElementById('logs-viewer');
  const pathHint = document.getElementById('logs-path-hint');
  if (!viewer) return;
  if (!viewer.hasAttribute('tabindex')) viewer.setAttribute('tabindex', '0');
  const inv = getInvoke() || invoke;
  if (!inv) {
    viewer.textContent = 'App not ready.';
    viewer.classList.add('is-empty');
    return;
  }
  try {
    const tail = await inv('read_debug_log', { maxBytes: 262144 });
    if (pathHint && tail.path) {
      pathHint.textContent = tail.path.replace(/^\/Users\/[^/]+/, '~');
      pathHint.title = tail.path;
    }
    const prefix = tail.truncated
      ? `… truncated (showing last ~${Math.round((tail.content || '').length / 1024)} KiB of ${Math.round((tail.total_bytes || 0) / 1024)} KiB)\n\n`
      : '';
    const body = tail.content || '(empty log)';
    viewer.textContent = prefix + body;
    viewer.classList.toggle('is-empty', !tail.content);
    if (scrollToEnd) {
      viewer.scrollTop = viewer.scrollHeight;
    }
  } catch (err) {
    viewer.textContent = 'Failed to read log: ' + String(err);
    viewer.classList.add('is-empty');
  }
}

function stopLogsAutoRefresh() {
  if (logsAutoRefreshTimer) {
    clearInterval(logsAutoRefreshTimer);
    logsAutoRefreshTimer = null;
  }
}

function startLogsAutoRefresh() {
  stopLogsAutoRefresh();
  logsAutoRefreshTimer = setInterval(() => refreshLogsViewer(true), 2000);
}

function formatDiskBytes(n) {
  const x = Number(n) || 0;
  if (x >= 1024 * 1024 * 1024) return `${(x / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  if (x >= 1024 * 1024) return `${(x / (1024 * 1024)).toFixed(1)} MB`;
  if (x >= 1024) return `${Math.round(x / 1024)} KB`;
  return `${x} B`;
}

function formatDiskWhen(iso) {
  if (!iso) return '—';
  try {
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return iso;
    return d.toLocaleString(undefined, {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  } catch (_) {
    return iso;
  }
}

async function refreshDiskCleanupPanel() {
  const list = document.getElementById('disk-cleanup-list');
  const summary = document.getElementById('disk-cleanup-summary');
  const lastEl = document.getElementById('disk-cleanup-last');
  const nextEl = document.getElementById('disk-cleanup-next');
  const triggersEl = document.getElementById('disk-cleanup-triggers');
  const reclaimEl = document.getElementById('disk-cleanup-reclaim');
  const scopeSummaryEl = document.getElementById('disk-cleanup-scope-summary');
  const scopesEl = document.getElementById('disk-cleanup-scopes');
  const icon = document.getElementById('icon-disk-cleanup');
  const runBtn = document.getElementById('disk-cleanup-run-btn');
  const inv = getInvoke();
  if (!inv || !list) return null;

  try {
    const status = await inv('get_disk_cleanup_status');
    window.__diskCleanupScopes = Array.isArray(status.scopes)
      ? status.scopes.map((s) => ({ ...s }))
      : [];
    const reclaimBytes = status.reclaimableBytes || 0;
    const reclaimFiles = status.reclaimableFiles || 0;

    if (summary) {
      summary.textContent =
        reclaimBytes > 0
          ? `${formatDiskBytes(reclaimBytes)} reclaimable`
          : 'Clean';
    }
    if (reclaimEl) {
      reclaimEl.textContent =
        reclaimBytes > 0
          ? `${formatDiskBytes(reclaimBytes)} · ${reclaimFiles} item(s)`
          : 'Nothing pending';
      reclaimEl.closest('.disk-cleanup-meta-card')?.classList.toggle(
        'has-reclaim',
        reclaimBytes > 0
      );
    }
    if (nextEl) {
      nextEl.textContent = status.nextRunLabel || '—';
      nextEl.title = status.nextRunUtc || '';
    }
    if (triggersEl) {
      triggersEl.textContent = (status.triggers || []).join(' · ') || '—';
    }
    if (scopeSummaryEl) {
      scopeSummaryEl.textContent = status.enabledScopeSummary || status.rootHint || '—';
    }
    const softEl = document.getElementById('disk-cleanup-soft-delete');
    if (softEl) {
      softEl.checked = status.softDelete !== false;
      const softLabel = softEl.closest?.('label.disk-cleanup-soft-delete');
      if (softLabel) {
        softLabel.title =
          'T toggles Trash soft-delete (unchecked = permanent delete)';
      }
    }
    if (icon) {
      icon.classList.toggle('has-reclaim', reclaimBytes >= 1024 * 1024);
      const base = icon.getAttribute('data-title-base') || 'Disk cleanup';
      icon.title =
        reclaimBytes > 0
          ? `${base} — ${formatDiskBytes(reclaimBytes)} reclaimable`
          : base;
    }

    if (scopesEl) {
      const scopes = window.__diskCleanupScopes || [];
      const preferIdx =
        typeof window.__diskCleanupScopeFocusIdx === 'number'
          ? window.__diskCleanupScopeFocusIdx
          : null;
      scopesEl.innerHTML = scopes
        .map((s, idx) => {
          const pathHint = s.path || (s.kind === 'temp' ? 'system temp + /tmp' : s.kind);
          const ageDisabled = s.kind === 'mac-stats' ? 'disabled' : '';
          const ageVal = s.maxAgeDays != null ? s.maxAgeDays : '';
          const removeBtn = s.builtin
            ? ''
            : `<button type="button" class="disk-cleanup-scope-remove" data-scope-remove="${idx}">Remove</button>`;
          const rowTitle = s.builtin
            ? '↑↓ / j k · PgUp/PgDn select · Space toggle enable · R toggle recurse · Esc clears'
            : '↑↓ / j k · PgUp/PgDn select · Space toggle enable · R toggle recurse · Delete removes custom · Esc clears';
          return `<div class="disk-cleanup-scope-row${s.enabled ? '' : ' is-disabled'}" data-scope-idx="${idx}" role="option" title="${rowTitle}">
            <input type="checkbox" data-scope-enabled="${idx}" ${s.enabled ? 'checked' : ''} aria-label="Enable ${s.label}" />
            <div class="disk-cleanup-scope-main">
              <div class="disk-cleanup-scope-title">${s.label} <span class="disk-cleanup-scope-kind">(${s.kind})</span></div>
              <div class="disk-cleanup-scope-path" title="${pathHint}">${pathHint}</div>
            </div>
            <input type="number" min="1" max="3650" data-scope-days="${idx}" value="${ageVal}" ${ageDisabled} title="Max age (days)" placeholder="days" />
            <label class="disk-cleanup-scope-rec"><input type="checkbox" data-scope-rec="${idx}" ${s.recursive ? 'checked' : ''} ${s.kind === 'mac-stats' ? 'disabled' : ''} /> Recurse</label>
            ${removeBtn}
          </div>`;
        })
        .join('');
      if (scopes.length > 0) {
        let hint = document.getElementById('disk-cleanup-kb-hint');
        if (!hint) {
          hint = document.createElement('div');
          hint.className = 'disk-cleanup-kb-hint';
          hint.id = 'disk-cleanup-kb-hint';
          scopesEl.parentNode?.insertBefore(hint, scopesEl);
        }
        hint.textContent =
          '↑↓ / j k · PgUp/PgDn select scope · Esc clears · Space toggle enable · R toggle recurse · T toggle Trash soft-delete · Delete removes custom · Enter in Add form adds · ⌘S saves';
      } else {
        document.getElementById('disk-cleanup-kb-hint')?.remove();
      }
      syncDiskCleanupScopeTabOrder(scopesEl, preferIdx);
    }

    const cats = (status.categories || []).filter((c) => c.enabled !== false);
    if (!cats.length) {
      list.innerHTML =
        '<li class="disk-cleanup-empty">No enabled scopes — turn some on and Save scopes.</li>';
      document.getElementById('disk-cleanup-list-kb-hint')?.remove();
    } else {
      const preferItemIdx =
        typeof window.__diskCleanupItemFocusIdx === 'number'
          ? window.__diskCleanupItemFocusIdx
          : 0;
      list.innerHTML = cats
        .map((c, idx) => {
          const has = (c.bytes || 0) > 0 || (c.fileCount || 0) > 0;
          const samples = (c.sampleNames || []).slice(0, 3).join(', ');
          const title = has
            ? '↑↓ / j k · PgUp/PgDn select · Enter Clean now · Esc clears'
            : '↑↓ / j k · PgUp/PgDn select · Enter focuses Clean now · Esc clears';
          return `<li class="disk-cleanup-item${has ? ' has-reclaim' : ''}" role="option" data-item-idx="${idx}" title="${title}">
            <div class="disk-cleanup-item-head">
              <span class="disk-cleanup-item-title">${c.label || c.id}</span>
              <span class="disk-cleanup-item-stat">${
                has
                  ? `${formatDiskBytes(c.bytes || 0)} · ${c.fileCount || 0}`
                  : 'OK'
              }</span>
            </div>
            <div class="disk-cleanup-item-policy">${c.policy || ''}</div>
            <div class="disk-cleanup-item-path">${c.pathHint || ''}</div>
            ${
              samples
                ? `<div class="disk-cleanup-item-samples">${samples}</div>`
                : ''
            }
          </li>`;
        })
        .join('');
      let listHint = document.getElementById('disk-cleanup-list-kb-hint');
      if (!listHint && list.parentNode) {
        listHint = document.createElement('div');
        listHint.className = 'disk-cleanup-list-kb-hint';
        listHint.id = 'disk-cleanup-list-kb-hint';
        listHint.textContent =
          'Categories: ↑↓ / j k · PgUp/PgDn · Home / End select · Esc clears · Enter runs Clean now when reclaimable';
        list.parentNode.insertBefore(listHint, list);
      }
      syncDiskCleanupItemTabOrder(list, preferItemIdx);
    }

    if (lastEl) {
      const last = status.lastRun;
      if (!last) {
        lastEl.innerHTML =
          '<strong>Last run</strong><br>Not yet this install — will run on launch.';
      } else {
        const catBits = (last.categories || [])
          .filter((c) => (c.filesRemoved || 0) > 0 || (c.bytesFreed || 0) > 0)
          .map(
            (c) =>
              `${c.label}: ${c.filesRemoved || 0} / ${formatDiskBytes(c.bytesFreed || 0)}`
          )
          .join(' · ');
        lastEl.innerHTML = `<strong>Last run</strong> · ${formatDiskWhen(last.atUtc)} · ${
          last.trigger || '?'
        }<br>${
          last.note ||
          (last.filesRemoved
            ? `Removed ${last.filesRemoved} · freed ${formatDiskBytes(last.bytesFreed || 0)}`
            : 'Nothing removed')
        }${catBits ? `<br>${catBits}` : ''}`;
      }
    }

    if (runBtn) {
      runBtn.disabled = false;
      const soft = status.softDelete !== false;
      runBtn.textContent =
        reclaimBytes > 0
          ? soft
            ? 'Clean now (→ Trash)'
            : 'Clean now (permanent)'
          : soft
            ? 'Run cleanup (→ Trash)'
            : 'Run cleanup (permanent)';
    }
    return status;
  } catch (e) {
    console.warn('disk cleanup status', e);
    if (list) {
      list.innerHTML = `<li class="disk-cleanup-empty">Could not load status: ${String(
        e?.message || e
      )}</li>`;
    }
    return null;
  }
}

window.refreshDiskCleanupPanel = refreshDiskCleanupPanel;

function readDiskCleanupScopesFromDom() {
  const scopes = (window.__diskCleanupScopes || []).map((s) => ({ ...s }));
  scopes.forEach((s, idx) => {
    const en = document.querySelector(`[data-scope-enabled="${idx}"]`);
    const days = document.querySelector(`[data-scope-days="${idx}"]`);
    const rec = document.querySelector(`[data-scope-rec="${idx}"]`);
    if (en) s.enabled = !!en.checked;
    if (days && !days.disabled && days.value !== '') {
      const n = parseInt(days.value, 10);
      if (!Number.isNaN(n) && n > 0) s.maxAgeDays = n;
    }
    if (rec && !rec.disabled) s.recursive = !!rec.checked;
  });
  return scopes;
}

async function saveDiskCleanupScopes(scopes) {
  const inv = getInvoke();
  if (!inv) return;
  await inv('set_disk_cleanup_scopes', { scopes });
  await refreshDiskCleanupPanel();
}

/** Add a custom scope from the add-scope form fields; focuses the new row. */
async function addDiskCleanupScopeFromForm() {
  const label = (document.getElementById('disk-cleanup-add-label')?.value || '').trim();
  const path = (document.getElementById('disk-cleanup-add-path')?.value || '').trim();
  const days = parseInt(document.getElementById('disk-cleanup-add-days')?.value || '30', 10);
  const recursive = !!document.getElementById('disk-cleanup-add-recursive')?.checked;
  if (!label || !path) {
    alert('Label and path are required for a custom scope.');
    return false;
  }
  const scopes = readDiskCleanupScopesFromDom();
  const id = `custom-${Date.now().toString(36)}`;
  scopes.push({
    id,
    kind: 'path',
    label,
    enabled: true,
    path,
    maxAgeDays: Number.isNaN(days) || days < 1 ? 30 : days,
    recursive,
    builtin: false,
  });
  window.__diskCleanupScopeFocusIdx = scopes.length - 1;
  await saveDiskCleanupScopes(scopes);
  const labelEl = document.getElementById('disk-cleanup-add-label');
  const pathEl = document.getElementById('disk-cleanup-add-path');
  if (labelEl) labelEl.value = '';
  if (pathEl) pathEl.value = '';
  // Focus the new scope row after DOM refresh.
  requestAnimationFrame(() => {
    const scopesEl = document.getElementById('disk-cleanup-scopes');
    if (!scopesEl) return;
    const rows = Array.from(scopesEl.querySelectorAll('.disk-cleanup-scope-row'));
    const idx = Math.min(window.__diskCleanupScopeFocusIdx || 0, Math.max(0, rows.length - 1));
    syncDiskCleanupScopeTabOrder(scopesEl, idx);
    rows[idx]?.focus();
  });
  return true;
}

/** Remove a custom (non-builtin) scope by index; no-op for builtins. */
async function removeDiskCleanupScopeAt(idx) {
  const scopes = readDiskCleanupScopesFromDom();
  if (idx < 0 || idx >= scopes.length) return false;
  if (scopes[idx]?.builtin) return false;
  const next = scopes.filter((_, i) => i !== idx);
  const focusAfter = Math.min(idx, Math.max(0, next.length - 1));
  window.__diskCleanupScopeFocusIdx = next.length ? focusAfter : 0;
  await saveDiskCleanupScopes(next);
  return true;
}

/** Brief success flash on a Save control; restores label after ~1.8s. */
function flashSaveButton(btn, opts = {}) {
  if (!btn) return;
  const savedLabel = opts.savedLabel || 'Saved';
  const durationMs = opts.durationMs || 1800;
  if (btn._saveFlashTimer) {
    clearTimeout(btn._saveFlashTimer);
    btn._saveFlashTimer = null;
  }
  if (btn._saveFlashOriginalLabel == null) {
    btn._saveFlashOriginalLabel = btn.textContent;
  }
  btn.classList.add('is-just-saved');
  btn.textContent = savedLabel;
  btn._saveFlashTimer = setTimeout(() => {
    btn.classList.remove('is-just-saved');
    btn.textContent = btn._saveFlashOriginalLabel || 'Save';
    btn._saveFlashOriginalLabel = null;
    btn._saveFlashTimer = null;
  }, durationMs);
}

/** Roving tabindex for Disk Cleanup scope rows (Monitors / process-list parity). */
function syncDiskCleanupScopeTabOrder(scopesEl, preferIdx) {
  if (!scopesEl) return;
  const rows = Array.from(scopesEl.querySelectorAll('.disk-cleanup-scope-row'));
  if (rows.length === 0) return;
  let activeIdx = 0;
  if (typeof preferIdx === 'number' && preferIdx >= 0 && preferIdx < rows.length) {
    activeIdx = preferIdx;
  } else {
    const focused = rows.findIndex(
      (el) => el === document.activeElement || el.contains(document.activeElement)
    );
    if (focused >= 0) activeIdx = focused;
    else {
      const selected = rows.findIndex((el) => el.classList.contains('is-selected'));
      if (selected >= 0) activeIdx = selected;
    }
  }
  window.__diskCleanupScopeFocusIdx = activeIdx;
  rows.forEach((el, i) => {
    el.setAttribute('tabindex', i === activeIdx ? '0' : '-1');
    el.classList.toggle('is-selected', i === activeIdx);
    el.setAttribute('aria-selected', i === activeIdx ? 'true' : 'false');
  });
}

/** Roving tabindex for Disk Cleanup category / reclaim rows. */
function syncDiskCleanupItemTabOrder(listEl, preferIdx) {
  if (!listEl) return;
  const rows = Array.from(listEl.querySelectorAll('.disk-cleanup-item'));
  if (rows.length === 0) return;
  let activeIdx = 0;
  if (typeof preferIdx === 'number' && preferIdx >= 0 && preferIdx < rows.length) {
    activeIdx = preferIdx;
  } else {
    const focused = rows.findIndex((el) => el === document.activeElement);
    if (focused >= 0) activeIdx = focused;
    else {
      const selected = rows.findIndex((el) => el.classList.contains('is-selected'));
      if (selected >= 0) activeIdx = selected;
    }
  }
  window.__diskCleanupItemFocusIdx = activeIdx;
  rows.forEach((el, i) => {
    el.setAttribute('tabindex', i === activeIdx ? '0' : '-1');
    el.classList.toggle('is-selected', i === activeIdx);
    el.setAttribute('aria-selected', i === activeIdx ? 'true' : 'false');
  });
}

function wireDiskCleanupScopesKeyboard() {
  const scopesEl = document.getElementById('disk-cleanup-scopes');
  if (!scopesEl || scopesEl.dataset.keyboardNav === '1') return;
  scopesEl.dataset.keyboardNav = '1';
  scopesEl.setAttribute('role', 'listbox');
  scopesEl.setAttribute('aria-label', 'Cleanup scopes');

  scopesEl.addEventListener('click', (e) => {
    const row = e.target && e.target.closest && e.target.closest('.disk-cleanup-scope-row');
    if (!row || !scopesEl.contains(row)) return;
    const idx = parseInt(row.getAttribute('data-scope-idx') || '0', 10);
    syncDiskCleanupScopeTabOrder(scopesEl, idx);
    if (e.target === row) row.focus();
  });

  scopesEl.addEventListener('change', (e) => {
    const row = e.target && e.target.closest && e.target.closest('.disk-cleanup-scope-row');
    if (!row || !scopesEl.contains(row)) return;
    if (e.target.matches && e.target.matches('input[data-scope-enabled]')) {
      row.classList.toggle('is-disabled', !e.target.checked);
    }
  });

  scopesEl.addEventListener('keydown', (e) => {
    const row = e.target && e.target.closest && e.target.closest('.disk-cleanup-scope-row');
    if (!row || !scopesEl.contains(row)) return;
    const rows = Array.from(scopesEl.querySelectorAll('.disk-cleanup-scope-row'));
    const idx = rows.indexOf(row);
    if (idx < 0) return;

    const onNumber = e.target.matches && e.target.matches('input[type="number"]');
    const onTextLike =
      e.target.matches &&
      (e.target.matches('input[type="text"]') || e.target.matches('textarea'));
    const onEnable =
      e.target.matches && e.target.matches('input[data-scope-enabled]');
    const onRecurse =
      e.target.matches && e.target.matches('input[data-scope-rec]');
    const onButton = e.target.closest && e.target.closest('button');

    // Space / Enter on the row (not nested controls): toggle enable.
    if (
      (e.key === 'Enter' || e.key === ' ') &&
      !onNumber &&
      !onTextLike &&
      !onEnable &&
      !onRecurse &&
      !onButton
    ) {
      e.preventDefault();
      const cb = row.querySelector('input[data-scope-enabled]');
      if (cb && !cb.disabled) {
        cb.checked = !cb.checked;
        cb.dispatchEvent(new Event('change', { bubbles: true }));
        row.classList.toggle('is-disabled', !cb.checked);
      }
      return;
    }

    // Delete / Backspace removes custom scopes (same as Remove button); builtins stay.
    if (
      (e.key === 'Delete' || e.key === 'Backspace') &&
      !onNumber &&
      !onTextLike &&
      !onButton
    ) {
      const scopeIdx = parseInt(row.getAttribute('data-scope-idx') || `${idx}`, 10);
      const scopes = window.__diskCleanupScopes || [];
      if (scopes[scopeIdx]?.builtin) return;
      e.preventDefault();
      void removeDiskCleanupScopeAt(scopeIdx).catch((err) => {
        alert(`Remove failed: ${err?.message || err}`);
      });
      return;
    }

    // R toggles Recurse on the selected row (mouse still uses the checkbox).
    if (
      (e.key === 'r' || e.key === 'R') &&
      !onNumber &&
      !onTextLike &&
      !onButton &&
      !e.metaKey &&
      !e.ctrlKey &&
      !e.altKey
    ) {
      if (onRecurse) return; // native checkbox already handles Space/click
      const rec = row.querySelector('input[data-scope-rec]');
      if (rec && !rec.disabled) {
        e.preventDefault();
        rec.checked = !rec.checked;
        rec.dispatchEvent(new Event('change', { bubbles: true }));
      }
      return;
    }

    // Esc clears row selection (Monitors / Agent Ops parity).
    if (
      (e.key === 'Escape' || e.key === 'Esc') &&
      !onNumber &&
      !onTextLike
    ) {
      if (!row.classList.contains('is-selected') && document.activeElement !== row) {
        return;
      }
      e.preventDefault();
      e.stopPropagation();
      rows.forEach((el) => {
        el.classList.remove('is-selected');
        el.setAttribute('tabindex', '-1');
      });
      if (rows[0]) rows[0].setAttribute('tabindex', '0');
      window.__diskCleanupScopeFocusIdx = null;
      if (document.activeElement === row || row.contains(document.activeElement)) {
        row.blur();
        if (document.activeElement && row.contains(document.activeElement)) {
          document.activeElement.blur();
        }
      }
      return;
    }

    // Leave ArrowUp/Down to number steppers; leave Space to native checkboxes.
    // Skip j/k while typing in age/path fields.
    if (onNumber && (e.key === 'ArrowUp' || e.key === 'ArrowDown')) return;
    if ((onEnable || onRecurse) && (e.key === ' ' || e.key === 'Spacebar')) return;
    if (onNumber || onTextLike) return;

    let next = -1;
    const page = 5;
    if (e.key === 'ArrowDown' || e.key === 'j') next = Math.min(idx + 1, rows.length - 1);
    else if (e.key === 'ArrowUp' || e.key === 'k') next = Math.max(idx - 1, 0);
    else if (e.key === 'PageDown') next = Math.min(idx + page, rows.length - 1);
    else if (e.key === 'PageUp') next = Math.max(idx - page, 0);
    else if (e.key === 'Home') next = 0;
    else if (e.key === 'End') next = rows.length - 1;
    else return;

    e.preventDefault();
    if (next < 0 || next === idx) return;
    syncDiskCleanupScopeTabOrder(scopesEl, next);
    rows[next].focus();
    if (typeof rows[next].scrollIntoView === 'function') {
      rows[next].scrollIntoView({ block: 'nearest' });
    }
  });
}

function wireDiskCleanupListKeyboard() {
  const listEl = document.getElementById('disk-cleanup-list');
  if (!listEl || listEl.dataset.keyboardNav === '1') return;
  listEl.dataset.keyboardNav = '1';
  listEl.setAttribute('role', 'listbox');
  listEl.setAttribute('aria-label', 'Cleanup categories');

  listEl.addEventListener('click', (e) => {
    const row = e.target && e.target.closest && e.target.closest('.disk-cleanup-item');
    if (!row || !listEl.contains(row)) return;
    const idx = parseInt(row.getAttribute('data-item-idx') || '0', 10);
    syncDiskCleanupItemTabOrder(listEl, idx);
    row.focus();
  });

  listEl.addEventListener('keydown', (e) => {
    const row = e.target && e.target.closest && e.target.closest('.disk-cleanup-item');
    if (!row || !listEl.contains(row)) return;
    const rows = Array.from(listEl.querySelectorAll('.disk-cleanup-item'));
    const idx = rows.indexOf(row);
    if (idx < 0) return;

    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      const runBtn = document.getElementById('disk-cleanup-run-btn');
      if (runBtn && !runBtn.disabled) {
        if (row.classList.contains('has-reclaim')) {
          runBtn.click();
        } else {
          runBtn.focus();
        }
      }
      return;
    }

    // Esc clears category selection (Monitors / Agent Ops parity).
    if (e.key === 'Escape' || e.key === 'Esc') {
      if (!row.classList.contains('is-selected') && document.activeElement !== row) {
        return;
      }
      e.preventDefault();
      e.stopPropagation();
      rows.forEach((el) => {
        el.classList.remove('is-selected');
        el.setAttribute('tabindex', '-1');
      });
      if (rows[0]) rows[0].setAttribute('tabindex', '0');
      window.__diskCleanupItemFocusIdx = null;
      row.blur();
      return;
    }

    let next = -1;
    const page = 5;
    if (e.key === 'ArrowDown' || e.key === 'j') next = Math.min(idx + 1, rows.length - 1);
    else if (e.key === 'ArrowUp' || e.key === 'k') next = Math.max(idx - 1, 0);
    else if (e.key === 'PageDown') next = Math.min(idx + page, rows.length - 1);
    else if (e.key === 'PageUp') next = Math.max(idx - page, 0);
    else if (e.key === 'Home') next = 0;
    else if (e.key === 'End') next = rows.length - 1;
    else return;

    e.preventDefault();
    if (next < 0 || next === idx) return;
    syncDiskCleanupItemTabOrder(listEl, next);
    rows[next].focus();
    if (typeof rows[next].scrollIntoView === 'function') {
      rows[next].scrollIntoView({ block: 'nearest' });
    }
  });
}

function initDiskCleanupSection() {
  const header = document.getElementById('disk-cleanup-header');
  const content = document.getElementById('disk-cleanup-content');
  const section = document.querySelector('.disk-cleanup-section');
  const refreshBtn = document.getElementById('disk-cleanup-refresh-btn');
  const runBtn = document.getElementById('disk-cleanup-run-btn');
  const saveBtn = document.getElementById('disk-cleanup-save-scopes-btn');
  const addBtn = document.getElementById('disk-cleanup-add-btn');
  const scopesEl = document.getElementById('disk-cleanup-scopes');
  const icon = document.getElementById('icon-disk-cleanup');
  if (!header || !content) return;

  wireDiskCleanupScopesKeyboard();
  wireDiskCleanupListKeyboard();

  if (icon && !icon.getAttribute('data-title-base')) {
    icon.setAttribute('data-title-base', icon.title || 'Disk cleanup');
  }

  let collapsed = localStorage.getItem('disk_cleanup_collapsed') !== 'false';
  const applyCollapsed = () => {
    if (collapsed) {
      content.classList.add('collapsed');
      if (section) section.classList.add('collapsed');
    } else {
      content.classList.remove('collapsed');
      if (section) section.classList.remove('collapsed');
      refreshDiskCleanupPanel();
    }
    if (header._syncCollapseA11y) header._syncCollapseA11y();
  };
  applyCollapsed();
  refreshDiskCleanupPanel();

  if (typeof wireCollapsibleHeaderA11y === 'function') {
    wireCollapsibleHeaderA11y(header, {
      contentId: 'disk-cleanup-content',
      getExpanded: () => !collapsed,
      ignoreSelector:
        '#disk-cleanup-refresh-btn, #disk-cleanup-run-btn, #disk-cleanup-save-scopes-btn, #disk-cleanup-add-btn, #disk-cleanup-soft-delete, #disk-cleanup-scopes, #disk-cleanup-add-label, #disk-cleanup-add-path, #disk-cleanup-add-days, #disk-cleanup-add-recursive, .disk-cleanup-add-scope, .disk-cleanup-scopes, .disk-cleanup-soft-delete, input, button, label',
      onToggle: () => {
        collapsed = !collapsed;
        localStorage.setItem('disk_cleanup_collapsed', collapsed.toString());
        applyCollapsed();
      },
    });
  }

  header.addEventListener('click', (e) => {
    if (
      e.target.closest(
        '#disk-cleanup-refresh-btn, #disk-cleanup-run-btn, #disk-cleanup-save-scopes-btn, #disk-cleanup-add-btn, .disk-cleanup-scopes, .disk-cleanup-add-scope, .disk-cleanup-soft-delete, input, button, label'
      )
    ) {
      return;
    }
    e.stopPropagation();
    collapsed = !collapsed;
    localStorage.setItem('disk_cleanup_collapsed', collapsed.toString());
    applyCollapsed();
  });

  if (refreshBtn) {
    refreshBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      refreshDiskCleanupPanel();
    });
  }

  if (saveBtn) {
    saveBtn.addEventListener('click', async (e) => {
      e.stopPropagation();
      try {
        await saveDiskCleanupScopes(readDiskCleanupScopesFromDom());
        const softEl = document.getElementById('disk-cleanup-soft-delete');
        const invoke = getInvoke();
        if (softEl && invoke) {
          await invoke('set_disk_cleanup_soft_delete', { softDelete: !!softEl.checked });
          await refreshDiskCleanupPanel();
        }
        flashSaveButton(saveBtn);
      } catch (err) {
        alert(`Save scopes failed: ${err?.message || err}`);
      }
    });
  }

  const softToggle = document.getElementById('disk-cleanup-soft-delete');
  if (softToggle) {
    softToggle.addEventListener('change', async (e) => {
      e.stopPropagation();
      const inv = getInvoke();
      if (!inv) return;
      try {
        await inv('set_disk_cleanup_soft_delete', { softDelete: !!softToggle.checked });
        await refreshDiskCleanupPanel();
      } catch (err) {
        alert(`Could not save delete mode: ${err?.message || err}`);
        softToggle.checked = !softToggle.checked;
      }
    });
  }

  if (addBtn) {
    addBtn.addEventListener('click', async (e) => {
      e.stopPropagation();
      try {
        await addDiskCleanupScopeFromForm();
      } catch (err) {
        alert(`Add scope failed: ${err?.message || err}`);
      }
    });
  }

  // Enter in Add form fields submits (same as Add scope button).
  const addForm = document.querySelector('.disk-cleanup-add-scope');
  if (addForm && addForm.dataset.enterAdd !== '1') {
    addForm.dataset.enterAdd = '1';
    addForm.addEventListener('keydown', (e) => {
      if (e.key !== 'Enter') return;
      if (e.target && e.target.matches && e.target.matches('textarea')) return;
      e.preventDefault();
      e.stopPropagation();
      void addDiskCleanupScopeFromForm().catch((err) => {
        alert(`Add scope failed: ${err?.message || err}`);
      });
    });
  }

  // Disk Cleanup section shortcuts: ⌘/Ctrl+S save; T toggles Trash soft-delete.
  const diskSection = document.querySelector('.disk-cleanup-section');
  if (diskSection && diskSection.dataset.saveShortcut !== '1') {
    diskSection.dataset.saveShortcut = '1';
    diskSection.addEventListener('keydown', (e) => {
      if ((e.metaKey || e.ctrlKey) && (e.key === 's' || e.key === 'S')) {
        if (!saveBtn) return;
        e.preventDefault();
        e.stopPropagation();
        saveBtn.click();
        return;
      }

      // T toggles "Move cleaned items to Trash" (same as the soft-delete checkbox).
      if (
        (e.key === 't' || e.key === 'T') &&
        !e.metaKey &&
        !e.ctrlKey &&
        !e.altKey
      ) {
        const t = e.target;
        const typing =
          t &&
          t.matches &&
          (t.matches('input[type="text"]') ||
            t.matches('textarea') ||
            t.matches('input[type="number"]'));
        if (typing) return;
        const softEl = document.getElementById('disk-cleanup-soft-delete');
        if (!softEl) return;
        e.preventDefault();
        e.stopPropagation();
        softEl.checked = !softEl.checked;
        softEl.dispatchEvent(new Event('change', { bubbles: true }));
      }
    });
  }

  if (scopesEl) {
    scopesEl.addEventListener('click', async (e) => {
      const btn = e.target.closest('[data-scope-remove]');
      if (!btn) return;
      e.stopPropagation();
      const idx = parseInt(btn.getAttribute('data-scope-remove'), 10);
      try {
        await removeDiskCleanupScopeAt(idx);
      } catch (err) {
        alert(`Remove failed: ${err?.message || err}`);
      }
    });
  }

  if (runBtn) {
    runBtn.addEventListener('click', async (e) => {
      e.stopPropagation();
      const inv = getInvoke();
      if (!inv) return;
      try {
        await saveDiskCleanupScopes(readDiskCleanupScopesFromDom());
        const softEl = document.getElementById('disk-cleanup-soft-delete');
        if (softEl) {
          await inv('set_disk_cleanup_soft_delete', { softDelete: !!softEl.checked });
        }
      } catch (_) {}
      runBtn.disabled = true;
      runBtn.textContent = 'Cleaning…';
      try {
        await inv('run_disk_cleanup_now');
        await refreshDiskCleanupPanel();
      } catch (err) {
        console.warn('disk cleanup run', err);
        runBtn.disabled = false;
        runBtn.textContent = 'Clean now';
        alert(`Cleanup failed: ${err?.message || err}`);
      }
    });
  }
}

function initLogsSection() {
  const header = document.getElementById('logs-header');
  const content = document.getElementById('logs-content');
  const section = document.querySelector('.logs-section');
  const divider = document.getElementById('logs-details-divider');
  const refreshBtn = document.getElementById('logs-refresh-btn');
  const openBtn = document.getElementById('logs-open-btn');
  const autoCb = document.getElementById('logs-autorefresh');
  if (!header || !content) return;

  let logsCollapsed = localStorage.getItem('logs_collapsed') !== 'false';
  const applyCollapsed = () => {
    if (logsCollapsed) {
      content.classList.add('collapsed');
      if (section) section.classList.add('collapsed');
      if (divider) divider.style.display = 'none';
      stopLogsAutoRefresh();
    } else {
      content.classList.remove('collapsed');
      if (section) section.classList.remove('collapsed');
      if (divider) divider.style.display = '';
      refreshLogsViewer(true);
      if (autoCb && autoCb.checked) startLogsAutoRefresh();
    }
    if (header._syncCollapseA11y) header._syncCollapseA11y();
  };
  applyCollapsed();

  wireCollapsibleHeaderA11y(header, {
    contentId: 'logs-content',
    getExpanded: () => !logsCollapsed,
    ignoreSelector: '#logs-refresh-btn, #logs-open-btn, #logs-autorefresh, label',
    onToggle: () => {
      logsCollapsed = !logsCollapsed;
      localStorage.setItem('logs_collapsed', logsCollapsed.toString());
      applyCollapsed();
    },
  });

  header.addEventListener('click', (e) => {
    e.stopPropagation();
    logsCollapsed = !logsCollapsed;
    localStorage.setItem('logs_collapsed', logsCollapsed.toString());
    applyCollapsed();
  });

  if (refreshBtn) {
    refreshBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      refreshLogsViewer(true);
    });
  }
  if (openBtn) {
    openBtn.addEventListener('click', async (e) => {
      e.stopPropagation();
      const inv = getInvoke() || invoke;
      if (!inv) return;
      try {
        await inv('open_debug_log');
      } catch (err) {
        console.error('[Logs] open_debug_log failed:', err);
      }
    });
  }
  if (autoCb) {
    autoCb.addEventListener('change', () => {
      if (autoCb.checked && !logsCollapsed) startLogsAutoRefresh();
      else stopLogsAutoRefresh();
    });
  }
}

function collapseSectionByIds(sectionSel, contentId, collapsedKey) {
  const content = contentId ? document.getElementById(contentId) : null;
  const section = sectionSel ? document.querySelector(sectionSel) : null;
  if (content) {
    content.classList.add('collapsed');
    if (content.classList.contains('section-content-collapsible')) {
      content.style.display = 'none';
    }
  }
  if (section) section.classList.add('collapsed');
  if (collapsedKey) localStorage.setItem(collapsedKey, 'true');
}

/** Force-collapse heavy sections when Compact CPU window is enabled (does not run on disable). */
window.applyCpuWindowCompactLayout = function applyCpuWindowCompactLayout(compact) {
  if (!compact) return;
  collapseSectionByIds('.monitors-section', 'monitors-content', 'monitors_collapsed');
  collapseSectionByIds('.ollama-section', 'ollama-content', 'ollama_collapsed');
  collapseSectionByIds('.perplexity-section', 'perplexity-content', 'perplexity_collapsed');
  collapseSectionByIds('.logs-section', 'logs-content', 'logs_collapsed');
  collapseSectionByIds('.disk-cleanup-section', 'disk-cleanup-content', 'disk_cleanup_collapsed');
  if (typeof window.applyOpsCollapsed === 'function') {
    window.applyOpsCollapsed(true);
  } else {
    collapseSectionByIds('.agent-ops-section', 'agent-ops-content', null);
  }
  if (typeof window.hideDetailsProcessesSections === 'function') {
    window.hideDetailsProcessesSections();
  }
};

async function initCpuWindowCompactPreference() {
  try {
    const inv = getInvoke();
    if (!inv) return;
    const compact = !!(await inv('get_cpu_window_compact'));
    document.body.classList.toggle('cpu-window-compact', compact);
    if (compact) window.applyCpuWindowCompactLayout(true);
  } catch (e) {
    console.warn('cpu window compact pref', e);
  }
}

function initIconLine() {
  const monitorsIcon = document.getElementById('icon-monitors');
  const ollamaIcon = document.getElementById('icon-ollama');
  const perplexityIcon = document.getElementById('icon-perplexity');
  
  // Monitors icon click - toggle the External / Monitors section
  if (monitorsIcon) {
    monitorsIcon.addEventListener('click', (e) => {
      e.stopPropagation();
      const monitorsHeader = document.getElementById('monitors-header');
      if (monitorsHeader) {
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
        ollamaHeader.click();
      }
    });
  }
  
  // Perplexity icon click - toggle the Perplexity Search section
  if (perplexityIcon) {
    perplexityIcon.addEventListener('click', (e) => {
      e.stopPropagation();
      const perplexityHeader = document.getElementById('perplexity-header');
      if (perplexityHeader) {
        perplexityHeader.click();
      }
    });
  }

  const logsIcon = document.getElementById('icon-logs');
  if (logsIcon) {
    logsIcon.addEventListener('click', (e) => {
      e.stopPropagation();
      const logsHeader = document.getElementById('logs-header');
      if (logsHeader) {
        logsHeader.click();
      }
    });
  }

  const diskIcon = document.getElementById('icon-disk-cleanup');
  if (diskIcon) {
    diskIcon.addEventListener('click', (e) => {
      e.stopPropagation();
      const diskHeader = document.getElementById('disk-cleanup-header');
      if (diskHeader) {
        diskHeader.click();
      }
    });
  }

  // Agent Ops icon is wired exclusively in agent-ops.js (avoid double-toggle).

  initDiscordIconStatus();
}

// Check if history data is available and show/hide dropdown accordingly
async function checkHistoryAvailability() {
  try {
    // Check if we have >24h of data available to show the dropdown
    const inv = getInvoke();
    if (!inv) return;
    const result = await inv('get_metrics_history', {
      time_range_seconds: 86400, // 24 hours
      max_display_points: null
    });

    if (result && result.oldest_available_timestamp) {
      const now = Math.floor(Date.now() / 1000);
      const availableSeconds = now - result.oldest_available_timestamp;
      const hasMore24h = availableSeconds > 86400;

      const historyControls = document.getElementById('history-controls');
      if (historyControls) {
        // Show dropdown if we have >24h of data
        historyControls.style.display = hasMore24h ? 'flex' : 'none';
      }
    }
  } catch (error) {
    // History not yet available (normal on startup) - silent
  }
}

// Initialize history controls
function initHistoryControls() {
  const timeRangeSelect = document.getElementById('time-range-select');
  if (timeRangeSelect) {
    timeRangeSelect.addEventListener('change', (e) => {
      const timeRange = e.target.value;
      // Time range selection removed - charts now use real-time updates only
    });
  }

  // Check history availability periodically
  checkHistoryAvailability();
  setInterval(checkHistoryAvailability, 60000); // Check every minute
}

// Initialize monitoring features when DOM is ready
function initMonitoringFeatures() {
  // Use setTimeout to ensure DOM is fully ready
  setTimeout(() => {
    initIconLine();
    initCollapsibleSections();
    initMonitorsSection();
    initPerplexitySection();
    initLogsSection();
    initDiskCleanupSection();
    initOllamaSection();
    initHistoryControls();
    // Auto-configure Ollama with default endpoint (if module is available)
    if (window.Ollama) {
      autoConfigureOllama();
    }
    initCpuWindowCompactPreference();
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


