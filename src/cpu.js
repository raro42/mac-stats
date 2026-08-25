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

/** Section open/closed state — config.json (survives WebView destroy) + localStorage mirror. */
const CPU_UI_SECTION_DEFAULTS = {
  monitors_collapsed: true,
  ollama_collapsed: true,
  perplexity_collapsed: true,
  agent_ops_collapsed: true,
  logs_collapsed: true,
  disk_cleanup_collapsed: true,
  details_processes_collapsed: true,
};

let cpuUiSectionsCache = null;
let cpuUiSectionsPersistTimer = null;
let cpuUiSectionsLoadPromise = null;

function readCpuUiSectionFromLocalStorage(key, defaultCollapsed) {
  try {
    const raw = localStorage.getItem(key);
    if (raw === null) return defaultCollapsed;
    return raw === 'true';
  } catch (_) {
    return defaultCollapsed;
  }
}

function seedCpuUiSectionsFromLocalStorage() {
  const seeded = { ...CPU_UI_SECTION_DEFAULTS };
  for (const key of Object.keys(CPU_UI_SECTION_DEFAULTS)) {
    seeded[key] = readCpuUiSectionFromLocalStorage(key, CPU_UI_SECTION_DEFAULTS[key]);
  }
  try {
    const tab = localStorage.getItem('agent_ops_tab');
    if (tab) seeded.agent_ops_tab = tab;
  } catch (_) {}
  return seeded;
}

async function loadCpuUiSections() {
  if (cpuUiSectionsLoadPromise) return cpuUiSectionsLoadPromise;
  cpuUiSectionsLoadPromise = (async () => {
    const seeded = seedCpuUiSectionsFromLocalStorage();
    cpuUiSectionsCache = { ...seeded };
    const inv = getInvoke();
    if (!inv) return cpuUiSectionsCache;
    // Tauri invoke may not be ready on first tick.
    for (let i = 0; i < 30; i++) {
      try {
        const remote = await inv('get_cpu_window_ui_state');
        if (remote && typeof remote === 'object' && !Array.isArray(remote)) {
          cpuUiSectionsCache = { ...seeded, ...remote };
          for (const [k, v] of Object.entries(cpuUiSectionsCache)) {
            if (typeof v === 'boolean') {
              try {
                localStorage.setItem(k, v ? 'true' : 'false');
              } catch (_) {}
            } else if (k === 'agent_ops_tab' && typeof v === 'string') {
              try {
                localStorage.setItem(k, v);
              } catch (_) {}
            }
          }
        }
        return cpuUiSectionsCache;
      } catch (_) {
        await new Promise((r) => setTimeout(r, 50));
      }
    }
    console.warn('cpuWindowUi load failed; using localStorage defaults');
    return cpuUiSectionsCache;
  })();
  return cpuUiSectionsLoadPromise;
}

function getSectionCollapsed(key) {
  const def = Object.prototype.hasOwnProperty.call(CPU_UI_SECTION_DEFAULTS, key)
    ? CPU_UI_SECTION_DEFAULTS[key]
    : true;
  if (cpuUiSectionsCache && Object.prototype.hasOwnProperty.call(cpuUiSectionsCache, key)) {
    return !!cpuUiSectionsCache[key];
  }
  return readCpuUiSectionFromLocalStorage(key, def);
}

function setSectionCollapsed(key, collapsed) {
  const value = !!collapsed;
  if (!cpuUiSectionsCache) cpuUiSectionsCache = seedCpuUiSectionsFromLocalStorage();
  cpuUiSectionsCache[key] = value;
  try {
    localStorage.setItem(key, value ? 'true' : 'false');
  } catch (_) {}
  schedulePersistCpuUiSections();
}

function setCpuUiSectionValue(key, value) {
  if (!cpuUiSectionsCache) cpuUiSectionsCache = seedCpuUiSectionsFromLocalStorage();
  cpuUiSectionsCache[key] = value;
  try {
    if (typeof value === 'boolean') {
      localStorage.setItem(key, value ? 'true' : 'false');
    } else if (typeof value === 'string') {
      localStorage.setItem(key, value);
    }
  } catch (_) {}
  schedulePersistCpuUiSections();
}

function getCpuUiSectionValue(key, fallback = null) {
  if (cpuUiSectionsCache && Object.prototype.hasOwnProperty.call(cpuUiSectionsCache, key)) {
    const v = cpuUiSectionsCache[key];
    return v == null ? fallback : v;
  }
  try {
    const raw = localStorage.getItem(key);
    return raw == null ? fallback : raw;
  } catch (_) {
    return fallback;
  }
}

function schedulePersistCpuUiSections() {
  if (cpuUiSectionsPersistTimer) clearTimeout(cpuUiSectionsPersistTimer);
  cpuUiSectionsPersistTimer = setTimeout(() => {
    cpuUiSectionsPersistTimer = null;
    void persistCpuUiSectionsNow();
  }, 120);
}

async function persistCpuUiSectionsNow() {
  const inv = getInvoke();
  if (!inv || !cpuUiSectionsCache) return;
  try {
    await inv('set_cpu_window_ui_state', { state: cpuUiSectionsCache });
  } catch (e) {
    console.warn('cpuWindowUi save failed', e);
  }
}

window.getSectionCollapsed = getSectionCollapsed;
window.setSectionCollapsed = setSectionCollapsed;
window.setCpuUiSectionValue = setCpuUiSectionValue;
window.getCpuUiSectionValue = getCpuUiSectionValue;
window.loadCpuUiSections = loadCpuUiSections;
/** Highlight icon when its section is open; fade when closed. */
window.syncSectionIcon = function syncSectionIcon(iconId, isOpen) {
  const icon = document.getElementById(iconId);
  if (!icon) return;
  icon.classList.toggle('section-open', !!isOpen);
  icon.setAttribute('aria-pressed', isOpen ? 'true' : 'false');
};

/** Fully hide or show an icon-line pane (no keep-header / collapsed glance). */
function setIconPaneVisibility(section, content, hidden, divider) {
  if (section) {
    section.style.display = hidden ? 'none' : '';
    section.classList.toggle('collapsed', hidden);
    if (hidden) section.setAttribute('aria-hidden', 'true');
    else section.removeAttribute('aria-hidden');
  }
  if (divider) divider.style.display = hidden ? 'none' : '';
  if (content) {
    content.classList.toggle('collapsed', hidden);
    if (!hidden) content.style.display = '';
  }
}
window.setIconPaneVisibility = setIconPaneVisibility;
// Kick off early so Agent Ops can await the same promise.
window.cpuUiSectionsReady = loadCpuUiSections();

// Flush section state before WebView destroy (menu-bar toggle / title-bar close).
window.addEventListener('pagehide', () => {
  if (cpuUiSectionsPersistTimer) {
    clearTimeout(cpuUiSectionsPersistTimer);
    cpuUiSectionsPersistTimer = null;
  }
  void persistCpuUiSectionsNow();
});

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
/** Top Processes list filter: all | pinned (Monitors All/Up/Down parity). */
let processesFilterMode = "all";

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

/** Brief ★ flash after pin/unpin (survives list rebuild). */
let processPinFlash = null; // { name, pinned }
let processPinFlashTimer = null;

function clearProcessPinFlashTimers() {
  if (processPinFlashTimer) {
    clearTimeout(processPinFlashTimer);
    processPinFlashTimer = null;
  }
}

function requestProcessPinFlash(name, pinned) {
  if (!name) return;
  processPinFlash = { name, pinned: !!pinned };
  clearProcessPinFlashTimers();
  processPinFlashTimer = setTimeout(() => {
    processPinFlash = null;
    processPinFlashTimer = null;
    document.querySelectorAll(".process-pin.is-just-saved").forEach((el) => {
      el.classList.remove("is-just-saved");
      const isPinned = el.classList.contains("is-pinned");
      el.title = isPinned ? "Unpin" : "Pin favorite";
    });
  }, 1200);
}

function applyProcessPinFlash(list) {
  if (!processPinFlash || !list) return;
  const want = processPinFlash.name;
  const btn = Array.from(list.querySelectorAll(".process-pin")).find(
    (el) => el.getAttribute("data-name") === want
  );
  if (!btn) return;
  btn.classList.add("is-just-saved");
  btn.title = processPinFlash.pinned ? "Pinned" : "Unpinned";
}

/** Brief Copied flash on process name + row wash (survives list rebuild). */
let processNameCopyFlash = null; // { name }
let processNameCopyFlashTimer = null;

function clearProcessNameCopyFlashTimers() {
  if (processNameCopyFlashTimer) {
    clearTimeout(processNameCopyFlashTimer);
    processNameCopyFlashTimer = null;
  }
}

function clearProcessRowCopiedWash(list) {
  const root = list || document.getElementById("process-list");
  if (!root) return;
  root.querySelectorAll(".process-row.is-just-copied").forEach((row) => {
    if (row._processCopiedTimer) {
      clearTimeout(row._processCopiedTimer);
      row._processCopiedTimer = null;
    }
    row.classList.remove("is-just-copied");
    row.removeAttribute("aria-label");
    const prev = row._processCopiedPrevTitle;
    if (prev) row.title = prev;
    else row.removeAttribute("title");
    row._processCopiedPrevTitle = undefined;
  });
}

/** Brief green Copied wash on process row (Disk Cleanup / Debug Log parity). */
function flashProcessRowCopied(row) {
  if (!row) return;
  if (row._processCopiedTimer) {
    clearTimeout(row._processCopiedTimer);
    row._processCopiedTimer = null;
  }
  if (!row._processCopiedPrevTitle && row.hasAttribute("title")) {
    row._processCopiedPrevTitle = row.getAttribute("title") || "";
  }
  row.classList.add("is-just-copied");
  row.title = "Copied";
  row.setAttribute("aria-label", "Copied");
  row._processCopiedTimer = setTimeout(() => {
    row.classList.remove("is-just-copied");
    row._processCopiedTimer = null;
    row.removeAttribute("aria-label");
    const prev = row._processCopiedPrevTitle;
    if (prev) row.title = prev;
    else row.removeAttribute("title");
    row._processCopiedPrevTitle = undefined;
  }, 1600);
}

function requestProcessNameCopyFlash(name) {
  if (!name) return;
  processNameCopyFlash = { name };
  clearProcessNameCopyFlashTimers();
  processNameCopyFlashTimer = setTimeout(() => {
    processNameCopyFlash = null;
    processNameCopyFlashTimer = null;
    document.querySelectorAll("button.process-name.is-just-saved").forEach((el) => {
      el.classList.remove("is-just-saved");
      const n = el.getAttribute("data-name") || "";
      el.textContent = n;
      el.title = "Click to copy name";
    });
    clearProcessRowCopiedWash();
  }, 1600);
}

function applyProcessNameCopyFlash(list) {
  if (!processNameCopyFlash || !list) return;
  const want = processNameCopyFlash.name;
  const btn = Array.from(list.querySelectorAll("button.process-name")).find(
    (el) => el.getAttribute("data-name") === want
  );
  if (!btn) return;
  btn.classList.add("is-just-saved");
  btn.textContent = "Copied";
  btn.title = "Copied";
  const row = btn.closest(".process-row");
  if (row) flashProcessRowCopied(row);
}

async function copyProcessNameFromUi(name) {
  const value = String(name || "").trim();
  if (!value) return false;
  const list = document.getElementById("process-list");
  const existingRow = list
    ? Array.from(list.querySelectorAll(".process-row")).find(
        (el) => el.getAttribute("data-name") === value
      )
    : null;
  if (
    existingRow &&
    (existingRow.classList.contains("is-just-copied") ||
      existingRow.querySelector("button.process-name.is-just-saved"))
  ) {
    return true;
  }
  const ok = await copyTextToClipboard(value);
  if (!ok) {
    alert("Could not copy process name.");
    return false;
  }
  requestProcessNameCopyFlash(value);
  applyProcessNameCopyFlash(list);
  return true;
}

function toggleProcessPinFromUi(name) {
  if (!name) return;
  if (processPinFlash && processPinFlash.name === name) return;
  const before = getPinnedProcessNames().includes(name);
  togglePinnedProcessName(name);
  const after = getPinnedProcessNames().includes(name);
  if (before === after) return;
  requestProcessPinFlash(name, after);
  window._forceProcessUpdate = true;
  if (window.refreshData) window.refreshData();
}

/** Visible process rows after All / Pinned filter. */
function visibleProcessRows(processList) {
  if (!processList) return [];
  return Array.from(processList.querySelectorAll(".process-row")).filter(
    (el) => el.style.display !== "none"
  );
}

/** Open a process row + details from a Top Processes glance strip. */
function openProcessFromGlance(pid) {
  if (!pid) return;
  if (typeof window.showDetailsProcessesSections === "function") {
    window.showDetailsProcessesSections();
  }
  if (processesFilterMode !== "all") setProcessesFilterMode("all");
  const list = document.getElementById("process-list");
  if (!list) return;
  const row = list.querySelector(
    `.process-row[data-pid="${CSS.escape(String(pid))}"]`
  );
  if (!row) return;
  const visible = visibleProcessRows(list);
  visible.forEach((r) =>
    r.setAttribute("tabindex", r === row ? "0" : "-1")
  );
  row.focus();
  if (typeof row.scrollIntoView === "function") {
    row.scrollIntoView({ block: "nearest" });
  }
  showProcessDetails(parseInt(pid, 10));
}

/** Highest-GPU process in the current list (Top CPU may differ). */
function pickTopGpuProcess(processes) {
  if (!processes || processes.length === 0) return null;
  let best = null;
  let bestGpu = -1;
  for (const p of processes) {
    const g = Number(p.gpu) || 0;
    if (g > bestGpu) {
      best = p;
      bestGpu = g;
    }
  }
  return best;
}

/** Top CPU glance under Top Processes header (Monitors slowest-summary parity). */
function ensureProcessesTopGlance() {
  const header = document.getElementById("processes-header");
  if (!header) return null;
  let glance = document.getElementById("processes-top-glance");
  if (!glance) {
    glance = document.createElement("div");
    glance.id = "processes-top-glance";
    glance.className = "processes-top-glance";
    glance.hidden = true;
    glance.innerHTML = '<span id="processes-top-glance-text"></span>';
    header.insertAdjacentElement("afterend", glance);
    wireProcessesTopGlanceClick(glance);
  }
  return glance;
}

function isProcessesSectionCollapsed() {
  const section =
    document.getElementById("processes-section") ||
    document.querySelector(
      ".apple-processes, .arch-processes, .swiss-processes, .mat-processes, .cpu-processes, .processes-section"
    );
  return !!(
    section &&
    (section.classList.contains("collapsed") || section.style.display === "none")
  );
}

function applyProcessesTopGlanceState({ topPid, topName, topCpu, waiting }) {
  const glance = ensureProcessesTopGlance();
  if (!glance) return;
  const text = document.getElementById("processes-top-glance-text");
  if (waiting || topPid == null || !topName) {
    window.__processesTopPid = null;
    glance.classList.remove("is-hot");
    // Collapsed: always show a glance so keep-header is useful (Debug Log / Perplexity parity).
    if (isProcessesSectionCollapsed()) {
      glance.hidden = false;
      if (text) text.textContent = "Waiting · processes";
      glance.setAttribute("role", "button");
      glance.tabIndex = 0;
      glance.title = "Show Top Processes";
      glance.setAttribute(
        "aria-label",
        "Top Processes waiting — click to expand"
      );
      return;
    }
    glance.hidden = true;
    return;
  }
  window.__processesTopPid = String(topPid);
  glance.hidden = false;
  const cpuStr = typeof topCpu === "number" ? `${topCpu.toFixed(1)}%` : "—";
  if (text) text.textContent = `Top CPU · ${topName} ${cpuStr}`;
  glance.classList.toggle("is-hot", typeof topCpu === "number" && topCpu >= 15);
  glance.setAttribute("role", "button");
  glance.tabIndex = 0;
  glance.title = `Click to open ${topName} details`;
  glance.setAttribute(
    "aria-label",
    `Top CPU process ${topName} at ${cpuStr} — click to open details`
  );
}

function wireProcessesTopGlanceClick(glance) {
  if (!glance || glance.dataset.topGlanceWired === "1") return;
  glance.dataset.topGlanceWired = "1";
  const activate = () => {
    if (window.__processesTopPid) {
      openProcessFromGlance(window.__processesTopPid);
      return;
    }
    if (typeof window.showDetailsProcessesSections === "function") {
      window.showDetailsProcessesSections();
    }
  };
  glance.addEventListener("click", (e) => {
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
  glance.addEventListener("keydown", (e) => {
    if (e.key !== "Enter" && e.key !== " ") return;
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
}

/** Top GPU glance under Top Processes (Top CPU / GPU-strip parity). */
function ensureProcessesTopGpuGlance() {
  ensureProcessesTopGlance();
  const anchor =
    document.getElementById("processes-top-glance") ||
    document.getElementById("processes-header");
  if (!anchor) return null;
  let glance = document.getElementById("processes-top-gpu-glance");
  if (!glance) {
    glance = document.createElement("div");
    glance.id = "processes-top-gpu-glance";
    glance.className = "processes-top-gpu-glance";
    glance.hidden = true;
    glance.innerHTML = '<span id="processes-top-gpu-glance-text"></span>';
    anchor.insertAdjacentElement("afterend", glance);
    wireProcessesTopGpuGlanceClick(glance);
  }
  return glance;
}

function applyProcessesTopGpuGlanceState({ topPid, topName, topGpu, waiting }) {
  const glance = ensureProcessesTopGpuGlance();
  if (!glance) return;
  const text = document.getElementById("processes-top-gpu-glance-text");
  if (waiting || topPid == null || !topName) {
    glance.hidden = true;
    window.__processesTopGpuPid = null;
    glance.classList.remove("is-hot");
    return;
  }
  window.__processesTopGpuPid = String(topPid);
  glance.hidden = false;
  const gpuNum = Number(topGpu) || 0;
  const gpuStr = gpuNum >= 0.1 ? `${gpuNum.toFixed(1)}%` : "—";
  if (text) text.textContent = `Top GPU · ${topName} ${gpuStr}`;
  glance.classList.toggle("is-hot", gpuNum >= 15);
  glance.setAttribute("role", "button");
  glance.tabIndex = 0;
  glance.title = `Click to open ${topName} details`;
  glance.setAttribute(
    "aria-label",
    `Top GPU process ${topName} at ${gpuStr} — click to open details`
  );
}

function wireProcessesTopGpuGlanceClick(glance) {
  if (!glance || glance.dataset.topGpuGlanceWired === "1") return;
  glance.dataset.topGpuGlanceWired = "1";
  const activate = () => openProcessFromGlance(window.__processesTopGpuPid);
  glance.addEventListener("click", (e) => {
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
  glance.addEventListener("keydown", (e) => {
    if (e.key !== "Enter" && e.key !== " ") return;
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
}

/** Highest-RAM process in the current list (Top CPU / GPU may differ). */
function pickTopRamProcess(processes) {
  if (!processes || processes.length === 0) return null;
  let best = null;
  let bestMem = -1;
  for (const p of processes) {
    const m = Number(p.memory) || 0;
    if (m > bestMem) {
      best = p;
      bestMem = m;
    }
  }
  return best;
}

/** Top RAM glance under Top Processes (Top CPU / Top GPU parity). */
function ensureProcessesTopRamGlance() {
  ensureProcessesTopGpuGlance();
  const anchor =
    document.getElementById("processes-top-gpu-glance") ||
    document.getElementById("processes-top-glance") ||
    document.getElementById("processes-header");
  if (!anchor) return null;
  let glance = document.getElementById("processes-top-ram-glance");
  if (!glance) {
    glance = document.createElement("div");
    glance.id = "processes-top-ram-glance";
    glance.className = "processes-top-ram-glance";
    glance.hidden = true;
    glance.innerHTML = '<span id="processes-top-ram-glance-text"></span>';
    anchor.insertAdjacentElement("afterend", glance);
    wireProcessesTopRamGlanceClick(glance);
  }
  return glance;
}

function applyProcessesTopRamGlanceState({ topPid, topName, topMem, waiting }) {
  const glance = ensureProcessesTopRamGlance();
  if (!glance) return;
  const text = document.getElementById("processes-top-ram-glance-text");
  if (waiting || topPid == null || !topName) {
    glance.hidden = true;
    window.__processesTopRamPid = null;
    glance.classList.remove("is-hot");
    return;
  }
  window.__processesTopRamPid = String(topPid);
  glance.hidden = false;
  const memNum = Number(topMem) || 0;
  const memStr = memNum > 0 ? formatBytes(memNum) : "—";
  if (text) text.textContent = `Top RAM · ${topName} ${memStr}`;
  // Amber wash when resident ≥ 1 GiB (heavy process).
  glance.classList.toggle("is-hot", memNum >= 1024 * 1024 * 1024);
  glance.setAttribute("role", "button");
  glance.tabIndex = 0;
  glance.title = `Click to open ${topName} details`;
  glance.setAttribute(
    "aria-label",
    `Top RAM process ${topName} at ${memStr} — click to open details`
  );
}

function wireProcessesTopRamGlanceClick(glance) {
  if (!glance || glance.dataset.topRamGlanceWired === "1") return;
  glance.dataset.topRamGlanceWired = "1";
  const activate = () => openProcessFromGlance(window.__processesTopRamPid);
  glance.addEventListener("click", (e) => {
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
  glance.addEventListener("keydown", (e) => {
    if (e.key !== "Enter" && e.key !== " ") return;
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
}

/** All / Pinned chips (Monitors / Debug Log filter parity). */
function ensureProcessesFilterChips() {
  const list = document.getElementById("process-list");
  if (!list || !list.parentNode) return;
  let wrap = document.getElementById("processes-filter-chips");
  if (!wrap) {
    wrap = document.createElement("div");
    wrap.id = "processes-filter-chips";
    wrap.className = "processes-filter-chips";
    wrap.setAttribute("role", "group");
    wrap.setAttribute("aria-label", "Process list filter");
    wrap.hidden = true;
    wrap.innerHTML =
      '<button type="button" class="processes-filter-chip is-active" data-processes-filter="all" aria-pressed="true" title="Show every process in the list">All</button>' +
      '<button type="button" class="processes-filter-chip" data-processes-filter="pinned" aria-pressed="false" title="Show pinned favorites only">Pinned <span class="processes-filter-count" data-processes-filter-count="pinned">0</span></button>';
    list.parentNode.insertBefore(wrap, list);
    wrap.addEventListener("click", (e) => {
      const btn = e.target && e.target.closest && e.target.closest("[data-processes-filter]");
      if (!btn || !wrap.contains(btn)) return;
      e.preventDefault();
      e.stopPropagation();
      setProcessesFilterMode(btn.getAttribute("data-processes-filter") || "all");
    });
  }
  wireFilterChipToolbarKeyboard(wrap);
}

function setProcessesFilterMode(mode) {
  const next = mode === "pinned" ? "pinned" : "all";
  processesFilterMode = next;
  document.querySelectorAll("#processes-filter-chips [data-processes-filter]").forEach((btn) => {
    const on = btn.getAttribute("data-processes-filter") === next;
    btn.classList.toggle("is-active", on);
    btn.setAttribute("aria-pressed", on ? "true" : "false");
  });
  applyProcessesListFilter();
}

function ensureProcessesFilterMissState(processList, show) {
  if (!processList) return;
  const existing = processList.querySelector(".processes-filter-miss");
  if (!show) {
    existing?.remove();
    return;
  }
  let wrap = existing;
  if (!wrap) {
    wrap = document.createElement("div");
    wrap.className = "process-empty processes-filter-miss";
    wrap.setAttribute("role", "status");
    wrap.innerHTML =
      `<div class="processes-filter-miss-msg">Nothing matches this filter</div>` +
      `<div class="processes-filter-miss-hint">Pin a favorite with ★ or P, or clear the filter.</div>` +
      `<button type="button" class="processes-filter-miss-cta processes-clear-filter">Clear filter</button>`;
    processList.appendChild(wrap);
    wrap.querySelector(".processes-clear-filter")?.addEventListener("click", (e) => {
      e.preventDefault();
      e.stopPropagation();
      setProcessesFilterMode("all");
    });
  }
}

function applyProcessesListFilter() {
  ensureProcessesFilterChips();
  const chips = document.getElementById("processes-filter-chips");
  const processList = document.getElementById("process-list");
  if (!processList) return;

  const rows = Array.from(processList.querySelectorAll(".process-row"));
  const waiting = !!processList.querySelector(".process-empty:not(.processes-filter-miss)");
  if (chips) chips.hidden = waiting || rows.length === 0;

  let pinnedCount = 0;
  rows.forEach((el) => {
    if (el.classList.contains("is-pinned")) pinnedCount++;
  });

  const pinnedEl = document.querySelector('[data-processes-filter-count="pinned"]');
  if (pinnedEl) pinnedEl.textContent = String(pinnedCount);
  document.querySelectorAll("#processes-filter-chips [data-processes-filter]").forEach((btn) => {
    const key = btn.getAttribute("data-processes-filter");
    btn.classList.toggle("has-hits", key === "pinned" ? pinnedCount > 0 : false);
  });

  if (waiting || rows.length === 0) {
    ensureProcessesFilterMissState(processList, false);
    return;
  }

  let visible = 0;
  rows.forEach((el) => {
    const isPinned = el.classList.contains("is-pinned");
    const show = processesFilterMode !== "pinned" || isPinned;
    el.style.display = show ? "" : "none";
    if (show) visible++;
  });

  const header = processList.querySelector(".process-list-header");
  if (header) header.style.display = visible === 0 ? "none" : "";

  ensureProcessesFilterMissState(processList, visible === 0);
  ensureProcessesListKbHint(processList, visible > 0);

  const visibleRows = visibleProcessRows(processList);
  visibleRows.forEach((r, i) => r.setAttribute("tabindex", i === 0 ? "0" : "-1"));
  rows
    .filter((r) => r.style.display === "none")
    .forEach((r) => r.setAttribute("tabindex", "-1"));
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
      // 2s matches backend get_cpu_details rate limit (1s polls were mostly cache hits
      // but still woke WebKit + IPC every second while the window was open).
      refreshInterval = setInterval(refresh, 2000);
      console.log("Got real data, switched to 2-second interval");
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
          // Thermal state subtext — prefer OS thermalState; else °C bands
          const thermalLevel = thermalLevelFromCpuDetails(data);
          const thermalText = thermalLevel
            ? `Thermal: ${thermalLevel}`
            : "Thermal: Nominal";
          if (tempSubtext.textContent !== thermalText) {
            scheduleDOMUpdate(() => {
              tempSubtext.textContent = thermalText;
            });
          }
        }
      }
      // Ring gauge and theme charts only when we refresh temperature
      updateRingGauge("temperature-ring-progress", Math.min(100, data.temperature), 'temperature');
      
      ensureTempStrip();
      const tempStripEl = document.getElementById("temp-strip-value");
      const tempStripCell = document.getElementById("temp-strip");
      let tempStripText = "—";
      if (data.can_read_temperature) {
        tempStripText = `${newTemp}°C`;
      }
      if (tempStripEl && tempStripEl.textContent !== tempStripText) {
        scheduleDOMUpdate(() => {
          tempStripEl.textContent = tempStripText;
        });
      }
      if (tempStripCell) {
        const hot =
          data.can_read_temperature &&
          typeof data.temperature === "number" &&
          data.temperature >= 70;
        tempStripCell.classList.toggle("is-hot", hot);
        const title = "Show temperature ring";
        tempStripCell.title = title;
        tempStripCell.setAttribute(
          "aria-label",
          `Temperature ${tempStripText}. ${title}`
        );
      }
      // Thermal / heat band on the power strip (OS thermalState, else °C bands)
      ensureThermalStrip();
      const thermalStripEl = document.getElementById("thermal-strip-value");
      const thermalStripCell = document.getElementById("thermal-strip");
      const thermalLevel = thermalLevelFromCpuDetails(data);
      const thermalStripText = thermalLevel || "—";
      if (thermalStripEl && thermalStripEl.textContent !== thermalStripText) {
        scheduleDOMUpdate(() => {
          thermalStripEl.textContent = thermalStripText;
        });
      }
      if (thermalStripCell) {
        thermalStripCell.classList.toggle("is-fair", thermalLevel === "Fair");
        thermalStripCell.classList.toggle(
          "is-hot",
          thermalLevel === "Serious" || thermalLevel === "Critical"
        );
        thermalStripCell.classList.toggle(
          "is-critical",
          thermalLevel === "Critical"
        );
        const fromOs =
          typeof data.thermal_state === "string" &&
          ["Nominal", "Fair", "Serious", "Critical"].includes(
            data.thermal_state.trim()
          );
        const title = fromOs
          ? "Apple thermal pressure — show temperature ring"
          : "Show temperature ring";
        thermalStripCell.title = title;
        thermalStripCell.setAttribute(
          "aria-label",
          `Thermal ${thermalStripText === "—" ? "unavailable" : thermalStripText}. ${title}`
        );
      }
    }

    updateLpmStripFromData(data);

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
      ensureGpuStrip();
      const gpuStripEl = document.getElementById("gpu-strip-value");
      const gpuStripCell = document.getElementById("gpu-strip");
      const gpuPctText = `${newGpuUsage}%`;
      if (gpuStripEl && gpuStripEl.textContent !== gpuPctText) {
        scheduleDOMUpdate(() => {
          gpuStripEl.textContent = gpuPctText;
        });
      }
      if (gpuStripCell) {
        gpuStripCell.classList.toggle("is-hot", newGpuUsage >= 15);
        const title = "Show GPU ring";
        gpuStripCell.title = title;
        gpuStripCell.setAttribute("aria-label", `GPU ${gpuPctText}. ${title}`);
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

    ensureCpuStrip();
    const cpuStripEl = document.getElementById("cpu-strip-value");
    const cpuStripCell = document.getElementById("cpu-strip");
    const cpuPctText = `${newUsage}%`;
    if (cpuStripEl && cpuStripEl.textContent !== cpuPctText) {
      scheduleDOMUpdate(() => {
        cpuStripEl.textContent = cpuPctText;
      });
    }
    if (cpuStripCell) {
      cpuStripCell.classList.toggle("is-hot", newUsage >= 50);
      const title = "Show CPU ring";
      cpuStripCell.title = title;
      cpuStripCell.setAttribute("aria-label", `CPU ${cpuPctText}. ${title}`);
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
    
      ensureFreqStrip();
      const freqStripEl = document.getElementById("freq-strip-value");
      const freqStripCell = document.getElementById("freq-strip");
      let freqStripText = "—";
      if (data.can_read_frequency && typeof data.frequency === "number" && data.frequency > 0) {
        freqStripText = data.frequency.toFixed(1);
      }
      if (freqStripEl && freqStripEl.textContent !== freqStripText) {
        scheduleDOMUpdate(() => {
          freqStripEl.textContent = freqStripText;
        });
      }
      if (freqStripCell) {
        const hot =
          data.can_read_frequency &&
          typeof data.frequency === "number" &&
          data.frequency >= 3.5;
        freqStripCell.classList.toggle("is-hot", hot);
        const title = "Show frequency ring";
        freqStripCell.title = title;
        freqStripCell.setAttribute(
          "aria-label",
          `CPU frequency ${freqStripText === "—" ? "unavailable" : freqStripText + " GHz"}. ${title}`
        );
      }

    // Update uptime — Details row + battery/power strip
    ensureUptimeStrip();
    const uptimeEl = document.getElementById("uptime-value");
    const uptimeStripEl = document.getElementById("uptime-strip-value");
    const uptimeStripCell = document.getElementById("uptime-strip");
    const uptimeSecs =
      typeof data.uptime_secs === "number" && Number.isFinite(data.uptime_secs)
        ? data.uptime_secs
        : 0;
    const uptimeFormatted = uptimeSecs > 0 ? formatUptime(uptimeSecs) : "—";
    if (uptimeEl && uptimeEl.textContent !== uptimeFormatted) {
      scheduleDOMUpdate(() => {
        uptimeEl.textContent = uptimeFormatted;
      });
    }
    if (uptimeStripEl && uptimeStripEl.textContent !== uptimeFormatted) {
      scheduleDOMUpdate(() => {
        uptimeStripEl.textContent = uptimeFormatted;
      });
    }
    if (uptimeStripCell) {
      // Soft wash when up ≥ 7 days (long-run awareness; SSD/CPU hot-threshold parity).
      uptimeStripCell.classList.toggle("is-long", uptimeSecs >= 7 * 24 * 3600);
      const title = "Show uptime in Details";
      uptimeStripCell.title = title;
      uptimeStripCell.setAttribute(
        "aria-label",
        `Uptime ${uptimeFormatted === "—" ? "unavailable" : uptimeFormatted}. ${title}`
      );
    }

    // System RAM (menu-bar parity) — Details section + battery/power strip
    ensureRamStrip();
    const ramPctEl = document.getElementById("ram-percent-value");
    const ramUsedEl = document.getElementById("ram-used-value");
    const ramTotalEl = document.getElementById("ram-total-value");
    const ramStripEl = document.getElementById("ram-strip-value");
    const ramStripCell = document.getElementById("ram-strip");
    if (ramPctEl || ramUsedEl || ramTotalEl || ramStripEl) {
      const pct =
        typeof data.ram_percent === "number" && Number.isFinite(data.ram_percent)
          ? data.ram_percent
          : null;
      const used = Number(data.ram_used_bytes) || 0;
      const total = Number(data.ram_total_bytes) || 0;
      const pctText = pct != null ? `${pct.toFixed(0)}%` : "—";
      const usedText = used > 0 ? formatBytes(used) : "—";
      const totalText = total > 0 ? formatBytes(total) : "—";
      if (ramPctEl && ramPctEl.textContent !== pctText) {
        scheduleDOMUpdate(() => {
          ramPctEl.textContent = pctText;
        });
      }
      if (ramUsedEl && ramUsedEl.textContent !== usedText) {
        scheduleDOMUpdate(() => {
          ramUsedEl.textContent = usedText;
        });
      }
      if (ramTotalEl && ramTotalEl.textContent !== totalText) {
        scheduleDOMUpdate(() => {
          ramTotalEl.textContent = totalText;
        });
      }
      if (ramStripEl && ramStripEl.textContent !== pctText) {
        scheduleDOMUpdate(() => {
          ramStripEl.textContent = pctText;
        });
      }
      if (ramStripCell) {
        const hot = pct != null && pct >= 85;
        ramStripCell.classList.toggle("is-hot", hot);
        const extra =
          usedText !== "—" && totalText !== "—"
            ? ` (${usedText} of ${totalText})`
            : "";
        const title = `Show RAM in Details${extra}`;
        ramStripCell.title = title;
        ramStripCell.setAttribute("aria-label", `RAM ${pctText}. ${title}`);
      }
    }

    // System SSD (menu-bar parity) — Disk Cleanup section + battery/power strip
    ensureDiskStrip();
    const diskStripEl = document.getElementById("disk-strip-value");
    const diskStripCell = document.getElementById("disk-strip");
    if (diskStripEl || diskStripCell) {
      const diskPct =
        typeof data.disk_percent === "number" && Number.isFinite(data.disk_percent)
          ? data.disk_percent
          : null;
      const diskPctText = diskPct != null ? `${diskPct.toFixed(0)}%` : "—";
      if (diskStripEl && diskStripEl.textContent !== diskPctText) {
        scheduleDOMUpdate(() => {
          diskStripEl.textContent = diskPctText;
        });
      }
      if (diskStripCell) {
        const hot = diskPct != null && diskPct >= 85;
        diskStripCell.classList.toggle("is-hot", hot);
        const title = "Show Disk Cleanup";
        diskStripCell.title = title;
        diskStripCell.setAttribute(
          "aria-label",
          `SSD ${diskPctText}. ${title}`
        );
      }
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

    // Details collapsed keep-header glance (Load · RAM · Up)
    {
      const glanceLoad1 =
        typeof data.load_1 === "number" && Number.isFinite(data.load_1)
          ? data.load_1
          : null;
      const glanceRam =
        typeof data.ram_percent === "number" && Number.isFinite(data.ram_percent)
          ? data.ram_percent
          : null;
      const glanceUpSecs =
        typeof data.uptime_secs === "number" && Number.isFinite(data.uptime_secs)
          ? data.uptime_secs
          : 0;
      applyDetailsCollapsedGlanceState({
        load1: glanceLoad1,
        ramPct: glanceRam,
        uptime: glanceUpSecs > 0 ? formatUptime(glanceUpSecs) : "—",
        waiting: glanceLoad1 == null && glanceRam == null && glanceUpSecs <= 0,
      });
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

    feedThemeHistoryCharts(data, shouldUpdateTemperature);

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
          ? data.top_processes.slice(0, 10)
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
          processes
            .map(
              (p) =>
                `${p.pid}:${p.cpu.toFixed(1)}:${(p.gpu || 0).toFixed(1)}:${p.memory || 0}:${p.name}`
            )
            .join("|")
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
      colCpu.className = "process-list-header-cpu";
      colCpu.textContent = "CPU";
      const colGpu = document.createElement("span");
      colGpu.className = "process-list-header-gpu";
      colGpu.textContent = "GPU";
      colHeader.appendChild(colPin);
      colHeader.appendChild(colName);
      colHeader.appendChild(colCpu);
      colHeader.appendChild(colGpu);
      fragment.appendChild(colHeader);
      
      const topProc = processes.length > 0 ? processes[0] : null;
      applyProcessesTopGlanceState({
        topPid: topProc?.pid ?? null,
        topName: topProc?.name ?? null,
        topCpu: topProc?.cpu ?? null,
        waiting: processes.length === 0,
      });
      const topGpuProc = pickTopGpuProcess(processes);
      applyProcessesTopGpuGlanceState({
        topPid: topGpuProc?.pid ?? null,
        topName: topGpuProc?.name ?? null,
        topGpu: topGpuProc?.gpu ?? null,
        waiting: processes.length === 0,
      });
      const topRamProc = pickTopRamProcess(processes);
      applyProcessesTopRamGlanceState({
        topPid: topRamProc?.pid ?? null,
        topName: topRamProc?.name ?? null,
        topMem: topRamProc?.memory ?? null,
        waiting: processes.length === 0,
      });

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
          // No row.title: keyboard tips live in #processes-kb-hint under the section title.
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
          
          const name = document.createElement("button");
          name.type = "button";
          name.className = "process-name";
          name.setAttribute("data-name", proc.name);
          name.setAttribute("tabindex", "-1");
          name.title = "Click to copy name";
          name.setAttribute("aria-label", `Copy name ${proc.name}`);
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

          const gpuUsage = document.createElement("div");
          gpuUsage.className = "process-usage process-usage-gpu";
          const gpuPct = Number(proc.gpu) || 0;
          const gpuBar = document.createElement("div");
          gpuBar.className = "process-bar process-bar-gpu";
          const gpuFill = document.createElement("div");
          gpuFill.className = "process-bar-fill process-bar-fill-gpu";
          gpuFill.style.width = `${Math.min(100, gpuPct)}%`;
          const gpuPercent = document.createElement("div");
          gpuPercent.className = "process-percent";
          gpuPercent.textContent = gpuPct >= 0.1 ? `${gpuPct.toFixed(1)}%` : "—";
          gpuBar.appendChild(gpuFill);
          gpuUsage.appendChild(gpuBar);
          gpuUsage.appendChild(gpuPercent);
          
          row.appendChild(pinBtn);
          row.appendChild(name);
          row.appendChild(usage);
          row.appendChild(gpuUsage);
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
          if (!list.hasAttribute("tabindex")) {
            list.setAttribute("tabindex", "0");
          }
          list.addEventListener("click", (e) => {
            const pinBtn = e.target.closest(".process-pin");
            if (pinBtn && list.contains(pinBtn)) {
              e.preventDefault();
              e.stopPropagation();
              toggleProcessPinFromUi(pinBtn.getAttribute("data-name"));
              return;
            }
            const nameBtn = e.target.closest("button.process-name");
            if (nameBtn && list.contains(nameBtn)) {
              e.preventDefault();
              e.stopPropagation();
              void copyProcessNameFromUi(nameBtn.getAttribute("data-name"));
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
            if (!row || !list.contains(row)) {
              // First arrow/j from listbox chrome focuses first/last row (Monitors parity).
              if (e.target !== list) return;
              const rows = visibleProcessRows(list);
              if (!rows.length) return;
              let next = -1;
              if (e.key === "ArrowDown" || e.key === "j" || e.key === "Home") next = 0;
              else if (e.key === "ArrowUp" || e.key === "k" || e.key === "End")
                next = rows.length - 1;
              else return;
              e.preventDefault();
              rows.forEach((r, i) =>
                r.setAttribute("tabindex", i === next ? "0" : "-1")
              );
              rows[next].focus();
              if (typeof rows[next].scrollIntoView === "function") {
                rows[next].scrollIntoView({ block: "nearest" });
              }
              return;
            }
            if (row.style.display === "none") return;
            // Pin button handles its own keys; do not steal from text fields.
            if (e.target.closest && e.target.closest(".process-pin")) return;
            const rows = visibleProcessRows(list);
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
              toggleProcessPinFromUi(row.getAttribute("data-name"));
              return;
            }

            // c copies the process name (PID copy lives in the details hero).
            if (
              (e.key === "c" || e.key === "C") &&
              !e.metaKey &&
              !e.ctrlKey &&
              !e.altKey
            ) {
              e.preventDefault();
              void copyProcessNameFromUi(row.getAttribute("data-name"));
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
        list.replaceChildren();
        list.appendChild(fragment);
        applyProcessPinFlash(list);
        applyProcessNameCopyFlash(list);
        applyProcessesListFilter();
        if (listHadFocus) {
          const visible = visibleProcessRows(list);
          const byPid =
            focusPid &&
            visible.find((r) => r.getAttribute("data-pid") === String(focusPid));
          const target =
            byPid ||
            list.querySelector('.process-row[tabindex="0"]') ||
            visible[0] ||
            null;
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
  refreshInterval = setInterval(refresh, 2000); // 2s: matches backend rate limit / lower WebKit wakeups
}

// Initialize when DOM and Tauri are ready
function init() {
  // Force immediate process update on initial load
  window._forceProcessUpdate = true;
  wireMetricValueCopy();
  ensureCpuHeaderToolbarKeyboard();
  ensureRingGaugeKeyboard();
  ensureHistorySparklineKeyboard();
  ensureGpuHistoryChart();
  ensureRamStripStyles();
  pruneMetricStripChips();
  ensurePowerStripKeyboard();
  
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

/** Inject GPU sparkline (CPU · GPU · Freq · Temp) when themes only ship three charts. */
function ensureGpuHistoryChart() {
  if (document.getElementById('gpu-history-chart')) return;
  const cpuCanvas = document.getElementById('usage-history-chart');
  const freqContainer = document.getElementById('frequency-history-chart')?.closest(
    '.history-chart-container'
  );
  if (!cpuCanvas || !freqContainer) return;
  const gpuContainer = document.createElement('div');
  gpuContainer.className = 'history-chart-container';
  gpuContainer.setAttribute('aria-label', 'GPU usage history');
  const caption = document.createElement('span');
  caption.className = 'history-chart-caption';
  caption.textContent = 'GPU';
  const canvas = document.createElement('canvas');
  canvas.className = 'history-chart';
  canvas.id = 'gpu-history-chart';
  canvas.width = 200;
  canvas.height = 40;
  gpuContainer.appendChild(caption);
  gpuContainer.appendChild(canvas);
  freqContainer.parentNode.insertBefore(gpuContainer, freqContainer);
  if (!document.getElementById('mac-stats-history-four-col')) {
    const style = document.createElement('style');
    style.id = 'mac-stats-history-four-col';
    style.textContent = `
      .history-section:has(#gpu-history-chart) {
        grid-template-columns: repeat(4, minmax(0, 1fr)) !important;
      }
    `;
    document.head.appendChild(style);
  }
  if (window.themeHistory?.init) window.themeHistory.init();
}

function feedThemeHistoryCharts(data, includeTemperature) {
  ensureGpuHistoryChart();
  const usage = typeof data?.usage === 'number' ? data.usage : null;
  const gpu =
    typeof data?.gpu_usage === 'number' ? Math.max(0, data.gpu_usage) : null;
  const freq =
    typeof data?.frequency === 'number' && data.frequency > 0
      ? data.frequency
      : null;
  const temp =
    includeTemperature &&
    typeof data?.temperature === 'number' &&
    data.temperature > 0
      ? data.temperature
      : null;

  const handlers = [
    window.posterCharts,
    window.themeHistory,
    window.appleHistory,
    window.darkHistory,
    window.lightHistory,
    window.futuristicHistory,
    window.materialHistory,
    window.neonHistory,
    window.swissHistory,
    window.architectHistory,
  ];
  for (const h of handlers) {
    if (!h) continue;
    if (usage !== null && typeof h.updateUsage === 'function') h.updateUsage(usage);
    if (gpu !== null) {
      if (typeof h.updateGpu === 'function') h.updateGpu(gpu);
      else if (typeof h.updateGpuUsage === 'function') h.updateGpuUsage(gpu);
    }
    if (freq !== null && typeof h.updateFrequency === 'function') {
      h.updateFrequency(freq);
    }
    if (temp !== null && typeof h.updateTemperature === 'function') {
      h.updateTemperature(temp);
    }
  }
}

/** Gauge metrics belong on the rings — not duplicated on the battery row. LPM stays on the strip. */
const METRIC_STRIP_CHIP_IDS = [
  'cpu-strip',
  'ram-strip',
  'gpu-strip',
  'temp-strip',
  'thermal-strip',
  'freq-strip',
  'disk-strip',
  'uptime-strip',
];

function pruneMetricStripChips() {
  for (const id of METRIC_STRIP_CHIP_IDS) {
    document.getElementById(id)?.remove();
  }
}

/** Inject click-to-copy styles for ring metric values + battery/power strip (all themes; flash via ::after so refresh can keep updating the number). */
function ensureMetricValueCopyStyles() {
  if (document.getElementById('mac-stats-metric-copy-styles')) return;
  const style = document.createElement('style');
  style.id = 'mac-stats-metric-copy-styles';
  style.textContent = `
    .metric-value[data-metric-copy="1"],
    .battery-level[data-metric-copy="1"],
    .power-value[data-metric-copy="1"] {
      cursor: pointer;
      position: relative;
      border-radius: 8px;
      outline: none;
      transition: background-color 0.2s ease, box-shadow 0.2s ease;
    }
    .metric-value[data-metric-copy="1"]:hover,
    .battery-level[data-metric-copy="1"]:hover,
    .power-value[data-metric-copy="1"]:hover {
      background-color: color-mix(in srgb, var(--accent, #0a84ff) 12%, transparent);
    }
    .metric-value[data-metric-copy="1"]:focus-visible,
    .battery-level[data-metric-copy="1"]:focus-visible,
    .power-value[data-metric-copy="1"]:focus-visible {
      box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent, #0a84ff) 55%, transparent);
    }
    .metric-value[data-metric-copy="1"].is-just-copied,
    .battery-level[data-metric-copy="1"].is-just-copied,
    .power-value[data-metric-copy="1"].is-just-copied {
      background-color: color-mix(in srgb, var(--accent, #0a84ff) 18%, transparent);
    }
    .metric-value[data-metric-copy="1"].is-just-copied::after,
    .battery-level[data-metric-copy="1"].is-just-copied::after,
    .power-value[data-metric-copy="1"].is-just-copied::after {
      content: "Copied";
      position: absolute;
      left: 50%;
      bottom: calc(100% + 4px);
      transform: translateX(-50%);
      font-size: 11px;
      font-weight: 600;
      letter-spacing: 0.02em;
      line-height: 1;
      padding: 3px 7px;
      border-radius: 999px;
      white-space: nowrap;
      color: var(--text, #fff);
      background: color-mix(in srgb, var(--accent, #0a84ff) 55%, rgba(0, 0, 0, 0.45));
      pointer-events: none;
      z-index: 4;
    }
  `;
  document.head.appendChild(style);
}

/**
 * Normalize metric / battery / power text for clipboard (e.g. "42%", "3.2 GHz", "58°C", "12.3 W").
 * Skips empty / em-dash / N/A / -- placeholders.
 */
function metricValueCopyText(el) {
  if (!el) return '';
  let raw = (el.textContent || '').replace(/\s+/g, ' ').trim();
  if (!raw || raw === '—' || raw.startsWith('—')) return '';
  if (/^(N\/A|--|–)$/i.test(raw)) return '';
  if (/^--\s*W$/i.test(raw) || /^–\s*W$/i.test(raw)) return '';
  // Prefer a space before unit when the unit is a sibling span (GHz / % / °C).
  const unit = el.querySelector?.('.metric-unit');
  if (unit) {
    const unitText = (unit.textContent || '').trim();
    const num = raw.replace(unitText, '').trim();
    if (num && unitText) return `${num} ${unitText}`.replace(/\s+/g, ' ').trim();
  }
  return raw;
}

async function copyMetricValueFromUi(el) {
  if (!el || el.classList.contains('is-just-copied')) return false;
  const value = metricValueCopyText(el);
  if (!value) return false;
  const ok = await copyTextToClipboard(value);
  if (!ok) {
    alert('Could not copy metric value.');
    return false;
  }
  el.classList.add('is-just-copied');
  el.setAttribute('aria-label', 'Copied');
  const prevTitle = el.getAttribute('data-copy-title') || el.title || 'Click to copy';
  el.title = 'Copied';
  window.setTimeout(() => {
    el.classList.remove('is-just-copied');
    el.title = prevTitle;
    el.removeAttribute('aria-label');
  }, 1600);
  return true;
}

/**
 * Wire ring metric values + battery/power strip for click / Enter / Space copy.
 * Skip CPU % — `#cpu-usage-card` already toggles Details / Top Processes on click;
 * copy+stopPropagation there stole that behavior (v0.1.513).
 */
function wireMetricValueCopy() {
  ensureMetricValueCopyStyles();
  const ids = [
    'gpu-usage-value',
    'frequency-value',
    'temperature-value',
    'battery-level',
    'power-value',
  ];
  for (const id of ids) {
    const el = document.getElementById(id);
    if (!el || el.dataset.metricCopyWired === '1') continue;
    el.dataset.metricCopyWired = '1';
    el.dataset.metricCopy = '1';
    el.setAttribute('role', 'button');
    el.tabIndex = 0;
    const title = 'Click to copy';
    el.title = title;
    el.setAttribute('data-copy-title', title);
    el.addEventListener('click', (e) => {
      e.preventDefault();
      e.stopPropagation();
      copyMetricValueFromUi(el);
    });
    el.addEventListener('keydown', (e) => {
      if (e.key !== 'Enter' && e.key !== ' ') return;
      e.preventDefault();
      e.stopPropagation();
      copyMetricValueFromUi(el);
    });
  }
}

/** Minimal battery/power strip layout (no duplicate CPU/RAM/GPU chips). */
function ensureRamStripStyles() {
  if (document.getElementById('mac-stats-ram-strip-styles')) return;
  const style = document.createElement('style');
  style.id = 'mac-stats-ram-strip-styles';
  style.textContent = `
    #battery-power-strip {
      flex-wrap: wrap;
      gap: 8px 12px;
    }
    .battery-info.is-low {
      border-radius: 8px;
      padding: 2px 6px;
      margin: -2px -6px;
      background-color: color-mix(in srgb, #ff9f0a 16%, transparent);
      box-shadow: 0 0 0 1px color-mix(in srgb, #ff9f0a 35%, transparent);
      transition: background-color 0.2s ease, box-shadow 0.2s ease;
    }
    #battery-power-strip.is-lpm-highlight {
      background-color: color-mix(in srgb, #30d158 16%, transparent);
      border-radius: 10px;
      box-shadow: 0 0 0 4px color-mix(in srgb, #30d158 16%, transparent);
      transition: background-color 0.2s ease, box-shadow 0.2s ease;
    }
    .lpm-info {
      display: flex;
      align-items: center;
      gap: 6px;
      cursor: pointer;
      border-radius: 8px;
      padding: 2px 6px;
      margin: -2px -6px;
      outline: none;
      transition: background-color 0.2s ease, box-shadow 0.2s ease;
    }
    .lpm-info:hover {
      background-color: color-mix(in srgb, var(--accent, #0a84ff) 12%, transparent);
    }
    .lpm-info:focus-visible {
      box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent, #0a84ff) 55%, transparent);
    }
    .lpm-info.is-on {
      background-color: color-mix(in srgb, #30d158 16%, transparent);
      box-shadow: 0 0 0 1px color-mix(in srgb, #30d158 35%, transparent);
    }
    .lpm-info.is-on .lpm-value {
      color: #248a3d;
    }
    .lpm-toggle {
      position: relative;
      flex-shrink: 0;
      width: 34px;
      height: 18px;
      border-radius: 999px;
      background: color-mix(in srgb, var(--muted, #888) 35%, transparent);
      box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--muted, #888) 25%, transparent);
      transition: background-color 0.2s ease, box-shadow 0.2s ease;
    }
    .lpm-toggle::after {
      content: "";
      position: absolute;
      top: 2px;
      left: 2px;
      width: 14px;
      height: 14px;
      border-radius: 50%;
      background: #fff;
      box-shadow: 0 1px 3px rgba(0, 0, 0, 0.18);
      transition: transform 0.2s ease;
    }
    .lpm-info.is-on .lpm-toggle {
      background: color-mix(in srgb, #30d158 85%, #fff);
      box-shadow: inset 0 0 0 1px color-mix(in srgb, #248a3d 40%, transparent);
    }
    .lpm-info.is-on .lpm-toggle::after {
      transform: translateX(16px);
    }
    .lpm-info.is-busy {
      opacity: 0.72;
      pointer-events: none;
    }
    .lpm-info.is-lpm-flash {
      box-shadow: 0 0 0 2px color-mix(in srgb, #30d158 55%, transparent);
    }
    .lpm-info.is-lpm-error {
      background-color: color-mix(in srgb, #ff9f0a 14%, transparent);
      box-shadow: 0 0 0 1px color-mix(in srgb, #ff9f0a 45%, transparent);
    }
    .lpm-label {
      color: var(--muted);
      font-size: 11px;
      letter-spacing: -0.01em;
      white-space: nowrap;
    }
    .lpm-value {
      font-weight: 650;
      letter-spacing: -0.01em;
      color: var(--text);
    }
    .power-strip-kb-hint {
      margin: 2px 0 0;
      font-size: 11px;
      opacity: 0.72;
      width: 100%;
      flex-basis: 100%;
    }
    .detail-label.is-ram-highlight,
    .detail-value.is-ram-highlight {
      background-color: color-mix(in srgb, var(--accent, #0a84ff) 16%, transparent);
      border-radius: 6px;
      box-shadow: 0 0 0 4px color-mix(in srgb, var(--accent, #0a84ff) 16%, transparent);
    }
    #cpu-usage-card.is-cpu-highlight {
      background-color: color-mix(in srgb, var(--accent, #0a84ff) 16%, transparent);
      border-radius: 10px;
      box-shadow: 0 0 0 4px color-mix(in srgb, var(--accent, #0a84ff) 16%, transparent);
    }
    .metric-card.is-gpu-highlight,
    #gpu-power.is-gpu-highlight {
      background-color: color-mix(in srgb, var(--accent, #0a84ff) 16%, transparent);
      border-radius: 10px;
      box-shadow: 0 0 0 4px color-mix(in srgb, var(--accent, #0a84ff) 16%, transparent);
    }
    .metric-card.is-temp-highlight,
    .metric-card.is-freq-highlight {
      background-color: color-mix(in srgb, var(--accent, #0a84ff) 16%, transparent);
      border-radius: 10px;
      box-shadow: 0 0 0 4px color-mix(in srgb, var(--accent, #0a84ff) 16%, transparent);
    }
    .disk-cleanup-section.is-disk-highlight {
      background-color: color-mix(in srgb, var(--accent, #0a84ff) 10%, transparent);
      border-radius: 12px;
      box-shadow: 0 0 0 4px color-mix(in srgb, var(--accent, #0a84ff) 14%, transparent);
    }
    .detail-label.is-uptime-highlight,
    .detail-value.is-uptime-highlight {
      background-color: color-mix(in srgb, var(--accent, #0a84ff) 16%, transparent);
      border-radius: 6px;
      box-shadow: 0 0 0 4px color-mix(in srgb, var(--accent, #0a84ff) 16%, transparent);
    }
  `;
  document.head.appendChild(style);
}

function _metricStripStub() {
  pruneMetricStripChips();
  return null;
}

function flashRamDetails() {
  const ids = ['ram-percent-value', 'ram-used-value', 'ram-total-value'];
  for (const id of ids) {
    const el = document.getElementById(id);
    if (!el) continue;
    el.classList.add('is-ram-highlight');
    const label = el.previousElementSibling;
    if (label && label.classList.contains('detail-label')) {
      label.classList.add('is-ram-highlight');
    }
    window.setTimeout(() => {
      el.classList.remove('is-ram-highlight');
      if (label && label.classList.contains('detail-label')) {
        label.classList.remove('is-ram-highlight');
      }
    }, 1600);
  }
}

function openRamDetailsFromStrip() {
  if (typeof window.showCpuDetailsSection === 'function') {
    window.showCpuDetailsSection();
  } else if (typeof window.showDetailsProcessesSections === 'function') {
    window.showDetailsProcessesSections();
  }
  const ramEl = document.getElementById('ram-percent-value');
  if (ramEl && typeof ramEl.scrollIntoView === 'function') {
    ramEl.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
  }
  flashRamDetails();
}

function flashCpuRing() {
  const card = document.getElementById("cpu-usage-card");
  if (!card) return;
  card.classList.add("is-cpu-highlight");
  window.setTimeout(() => {
    card.classList.remove("is-cpu-highlight");
  }, 1600);
}

function openCpuRingFromStrip() {
  const cpuVal = document.getElementById("cpu-usage-value");
  if (cpuVal && typeof cpuVal.scrollIntoView === "function") {
    cpuVal.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }
  flashCpuRing();
}

function flashMetricRingHighlight(valueId, highlightClass) {
  const el = document.getElementById(valueId);
  const card = el?.closest?.(".metric-card") || el;
  if (!card) return;
  card.classList.add(highlightClass);
  window.setTimeout(() => card.classList.remove(highlightClass), 1600);
}

function openGpuRingFromStrip() {
  const el = document.getElementById("gpu-usage-value");
  if (el && typeof el.scrollIntoView === "function") {
    el.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }
  flashMetricRingHighlight("gpu-usage-value", "is-gpu-highlight");
}

function openFreqRingFromStrip() {
  const el = document.getElementById("frequency-value");
  if (el && typeof el.scrollIntoView === "function") {
    el.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }
  flashMetricRingHighlight("frequency-value", "is-freq-highlight");
}

function openTempRingFromStrip() {
  const el = document.getElementById("temperature-value");
  if (el && typeof el.scrollIntoView === "function") {
    el.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }
  flashMetricRingHighlight("temperature-value", "is-temp-highlight");
}

function ensureCpuStrip() { return _metricStripStub(); }
function ensureRamStrip() { return _metricStripStub(); }
function ensureGpuStrip() { return _metricStripStub(); }
function ensureTempStrip() { return _metricStripStub(); }
function ensureThermalStrip() { return _metricStripStub(); }
const LPM_GUI_LABEL = 'Low Power Mode (LPM)';

function isLowPowerModeOn(data) {
  if (!data) return false;
  if (typeof data.low_power_mode === 'boolean') return data.low_power_mode;
  if (typeof data.lowPowerMode === 'boolean') return data.lowPowerMode;
  return false;
}

function updateLpmStripFromData(data, { optimisticOn } = {}) {
  ensureLowPowerStrip();
  const lpmStripEl = document.getElementById('lpm-strip-value');
  const lpmStripCell = document.getElementById('lpm-strip');
  if (!lpmStripEl || !lpmStripCell) return;
  const lpmOn =
    typeof optimisticOn === 'boolean' ? optimisticOn : isLowPowerModeOn(data);
  const lpmStripText = lpmOn ? 'On' : 'Off';
  if (lpmStripEl.textContent !== lpmStripText) {
    lpmStripEl.textContent = lpmStripText;
  }
  lpmStripCell.classList.toggle('is-on', lpmOn);
  lpmStripCell.dataset.lpmState = lpmOn ? 'on' : 'off';
  const title = 'Toggle Low Power Mode (macOS may ask for your password)';
  lpmStripCell.title = title;
  lpmStripCell.setAttribute(
    'aria-label',
    `${LPM_GUI_LABEL} ${lpmStripText}. ${title}`
  );
  lpmStripCell.setAttribute('aria-pressed', lpmOn ? 'true' : 'false');
}

function ensureLowPowerStrip() {
  ensureRamStripStyles();
  const strip = document.getElementById('battery-power-strip');
  if (!strip) return null;
  let cell = document.getElementById('lpm-strip');
  if (!cell) {
    cell = document.createElement('div');
    cell.id = 'lpm-strip';
    cell.className = 'lpm-info';
    cell.setAttribute('role', 'button');
    cell.tabIndex = 0;
    cell.title = 'Toggle Low Power Mode (macOS may ask for your password)';
    cell.setAttribute('aria-label', `${LPM_GUI_LABEL}. ${cell.title}`);
    cell.innerHTML =
      `<span class="lpm-label">${LPM_GUI_LABEL}</span>` +
      '<span class="lpm-value" id="lpm-strip-value">…</span>' +
      '<span class="lpm-toggle" aria-hidden="true"></span>';
    const powerEl = document.getElementById('power-value');
    const timeEl = document.getElementById('time-remaining');
    if (powerEl && powerEl.parentElement) {
      powerEl.parentElement.insertAdjacentElement('afterend', cell);
    } else if (timeEl) {
      strip.insertBefore(cell, timeEl);
    } else {
      strip.appendChild(cell);
    }
  }
  const labelEl = cell.querySelector('.lpm-label');
  if (labelEl && labelEl.textContent !== LPM_GUI_LABEL) {
    labelEl.textContent = LPM_GUI_LABEL;
  }
  if (cell.dataset.lpmStripWired === '1') return cell;
  cell.dataset.lpmStripWired = '1';
  const onLpmActivate = (e) => {
    e.preventDefault();
    e.stopPropagation();
    toggleLowPowerModeFromStrip();
  };
  cell.addEventListener('click', onLpmActivate);
  cell.addEventListener('keydown', (e) => {
    if (e.key !== 'Enter' && e.key !== ' ') return;
    onLpmActivate(e);
  });
  return cell;
}

async function openBatterySettingsFromStrip() {
  const urls = [
    'x-apple.systempreferences:com.apple.Battery-Settings.extension',
    'x-apple.systempreferences:com.apple.preference.battery',
  ];
  const invokeFn = typeof getInvoke === 'function' ? getInvoke() : null;
  for (const url of urls) {
    try {
      if (invokeFn) {
        await invokeFn('plugin:shell|open', { path: url });
        return;
      }
    } catch (_) {
      /* try next */
    }
    try {
      if (window.__TAURI__?.shell?.open) {
        await window.__TAURI__.shell.open(url);
        return;
      }
    } catch (_) {
      /* try next */
    }
  }
  const strip = document.getElementById('battery-power-strip');
  if (strip) {
    strip.classList.add('is-lpm-highlight');
    window.setTimeout(() => strip.classList.remove('is-lpm-highlight'), 1600);
  }
}

async function toggleLowPowerModeFromStrip() {
  const cell = document.getElementById('lpm-strip');
  if (!cell || cell.classList.contains('is-busy')) return;
  const wasOn = cell.classList.contains('is-on');
  cell.classList.remove('is-lpm-error');
  cell.classList.add('is-busy');
  updateLpmStripFromData(null, { optimisticOn: !wasOn });
  const invokeFn = typeof getInvoke === 'function' ? getInvoke() : null;
  if (invokeFn) {
    try {
      const result = await invokeFn('toggle_low_power_mode');
      const enabled =
        result && typeof result.enabled === 'boolean'
          ? result.enabled
          : !wasOn;
      updateLpmStripFromData({ low_power_mode: enabled });
      cell.classList.add('is-lpm-flash');
      window.setTimeout(() => cell.classList.remove('is-lpm-flash'), 900);
      if (typeof refresh === 'function') refresh();
      return;
    } catch (err) {
      console.warn('[LPM] toggle failed', err);
      updateLpmStripFromData({ low_power_mode: wasOn });
      cell.classList.add('is-lpm-error');
      const msg =
        (err && (err.message || err.toString && err.toString())) ||
        'Could not toggle Low Power Mode';
      cell.title = msg;
      window.setTimeout(() => {
        cell.classList.remove('is-lpm-error');
        cell.title = 'Toggle Low Power Mode (macOS may ask for your password)';
      }, 4000);
      alert(msg);
      return;
    } finally {
      cell.classList.remove('is-busy');
    }
  }
  cell.classList.remove('is-busy');
  await openBatterySettingsFromStrip();
}
function ensureFreqStrip() { return _metricStripStub(); }
function ensureDiskStrip() { return _metricStripStub(); }
function ensureUptimeStrip() { return _metricStripStub(); }

/** Focusable chips on #battery-power-strip (DOM order). */
function getPowerStripChips() {
  const strip = document.getElementById('battery-power-strip');
  if (!strip) return [];
  const sel = ['#battery-level', '#lpm-strip', '#power-value'].join(',');
  return Array.from(strip.querySelectorAll(sel)).filter((el) => {
    if (!el || el.hidden) return false;
    // Prefer laid-out chips; keep just-created nodes before first paint.
    return el.getClientRects().length > 0 || el.offsetParent !== null || strip.contains(el);
  });
}

/** One Tab stop on the strip; arrows move focus (Details / listbox chrome parity). */
function refreshPowerStripRovingTabindex(preferred) {
  const chips = getPowerStripChips();
  if (!chips.length) return;
  const focused = chips.find((el) => el === document.activeElement);
  const current =
    (preferred && chips.includes(preferred) && preferred) ||
    focused ||
    chips.find((el) => el.tabIndex === 0) ||
    chips[0];
  for (const el of chips) {
    el.tabIndex = el === current ? 0 : -1;
  }
}

/** Soft tip under the battery/power strip (Details kb-hint parity). */
function ensurePowerStripKbHint() {
  ensureRamStripStyles();
  const strip = document.getElementById('battery-power-strip');
  if (!strip) return;
  let hint = document.getElementById('power-strip-kb-hint');
  if (!hint) {
    hint = document.createElement('div');
    hint.id = 'power-strip-kb-hint';
    hint.className = 'power-strip-kb-hint';
    hint.setAttribute('aria-hidden', 'true');
    strip.appendChild(hint);
  }
  hint.textContent =
    'Tab or click Bat · LPM · Power · ← → / h l · Home/End · at start crosses to last history chart · at end crosses to section icons · LPM toggles (password may be required)';
}

/**
 * Power strip toolbar keyboard — focus a chip, then ←→ / h l / Home/End
 * moves across Bat · LPM · Power (Details / sparkline wrap parity).
 * Enter/Space keep existing activate/copy.
 */
function ensurePowerStripKeyboard() {
  ensureRamStripStyles();
  const strip = document.getElementById('battery-power-strip');
  if (!strip) return;
  ensurePowerStripKbHint();
  if (strip.dataset.powerStripChainKbWired !== '1') {
    strip.dataset.powerStripChainKbWired = '1';
    strip.addEventListener(
      'keydown',
      (e) => {
        if (e.metaKey || e.ctrlKey || e.altKey) return;
        const chips = getPowerStripChips();
        if (!chips.length) return;
        const idx = chips.indexOf(document.activeElement);
        if (idx < 0) return;
        const back =
          e.key === 'ArrowLeft' ||
          e.key === 'h' ||
          e.key === 'ArrowUp' ||
          e.key === 'k';
        const forward =
          e.key === 'ArrowRight' ||
          e.key === 'l' ||
          e.key === 'ArrowDown' ||
          e.key === 'j';
        if (back && idx === 0) {
          if (tryChainPowerStripToSparklineLast()) {
            e.preventDefault();
            e.stopPropagation();
          }
        } else if (forward && idx === chips.length - 1) {
          if (tryChainPowerStripToIconLineFirst()) {
            e.preventDefault();
            e.stopPropagation();
          }
        }
      },
      true
    );
  }
  wireToolbarKeyboard(
    strip,
    () => getPowerStripChips(),
    (preferred) => refreshPowerStripRovingTabindex(preferred),
    null
  );
  if (!strip.getAttribute('aria-label')) {
    strip.setAttribute('aria-label', 'Battery and power metrics');
  }
  strip.dataset.powerStripKbWired = '1';
}

/** Metrics section that holds the four ring cards (theme class varies). */
function getRingGaugeSection() {
  const cpuCard = document.getElementById('cpu-usage-card');
  if (!cpuCard) return null;
  return (
    cpuCard.closest(
      '.apple-metrics, .cpu-metrics, .metrics-grid, .arch-metrics, .swiss-metrics, .mat-metrics, .poster-metrics, section'
    ) || cpuCard.parentElement
  );
}

/**
 * Focusable ring-gauge targets in DOM order: CPU card (toggles Details /
 * Processes) then GPU · Frequency · Temperature values (click-to-copy).
 */
function getRingGaugeChips() {
  const section = getRingGaugeSection();
  if (!section) return [];
  const ids = [
    'cpu-usage-card',
    'gpu-usage-value',
    'frequency-value',
    'temperature-value',
  ];
  return ids
    .map((id) => document.getElementById(id))
    .filter((el) => {
      if (!el || !section.contains(el)) return false;
      if (el.hidden) return false;
      return (
        el.getClientRects().length > 0 ||
        el.offsetParent !== null ||
        section.contains(el)
      );
    });
}

function refreshRingGaugeRovingTabindex(preferred) {
  const chips = getRingGaugeChips();
  if (!chips.length) return;
  const focused = chips.find((el) => el === document.activeElement);
  const current =
    (preferred && chips.includes(preferred) && preferred) ||
    focused ||
    chips.find((el) => el.tabIndex === 0) ||
    chips[0];
  for (const el of chips) {
    el.tabIndex = el === current ? 0 : -1;
  }
}

function ensureRingGaugeKbStyles() {
  if (document.getElementById('mac-stats-ring-gauge-kb-styles')) return;
  const style = document.createElement('style');
  style.id = 'mac-stats-ring-gauge-kb-styles';
  style.textContent = `
    .ring-gauge-kb-hint {
      margin: 6px 0 0;
      font-size: 11px;
      opacity: 0.72;
      width: 100%;
      flex-basis: 100%;
      grid-column: 1 / -1;
    }
  `;
  document.head.appendChild(style);
}

/** Soft tip under the ring gauges (power-strip kb-hint parity). */
function ensureRingGaugeKbHint() {
  ensureRingGaugeKbStyles();
  const section = getRingGaugeSection();
  if (!section) return;
  let hint = document.getElementById('ring-gauge-kb-hint');
  if (!hint) {
    hint = document.createElement('div');
    hint.id = 'ring-gauge-kb-hint';
    hint.className = 'ring-gauge-kb-hint';
    hint.setAttribute('aria-hidden', 'true');
    section.appendChild(hint);
  }
  hint.textContent =
    'Tab or click a ring · ← → / h l · Home/End move · at start crosses to Settings · at end crosses to history charts · Enter / Space activates';
}

/**
 * Ring-gauge toolbar keyboard — click or Tab to CPU · GPU · Frequency · Temperature,
 * then ←→ / h l / Home/End. Enter/Space keeps CPU toggle and metric copy.
 */
function ensureRingGaugeKeyboard() {
  const section = getRingGaugeSection();
  if (!section) return;
  ensureRingGaugeKbHint();
  if (section.dataset.ringGaugeChainKbWired !== '1') {
    section.dataset.ringGaugeChainKbWired = '1';
    section.addEventListener(
      'keydown',
      (e) => {
        if (e.metaKey || e.ctrlKey || e.altKey) return;
        const chips = getRingGaugeChips();
        if (!chips.length) return;
        const idx = chips.indexOf(document.activeElement);
        if (idx < 0) return;
        const back =
          e.key === 'ArrowLeft' ||
          e.key === 'h' ||
          e.key === 'ArrowUp' ||
          e.key === 'k';
        const forward =
          e.key === 'ArrowRight' ||
          e.key === 'l' ||
          e.key === 'ArrowDown' ||
          e.key === 'j';
        if (back && idx === 0) {
          if (tryChainRingGaugeToHeaderSettings()) {
            e.preventDefault();
            e.stopPropagation();
          }
        } else if (forward && idx === chips.length - 1) {
          if (tryChainRingGaugeToSparklineFirst()) {
            e.preventDefault();
            e.stopPropagation();
          }
        }
      },
      true
    );
  }
  wireToolbarKeyboard(
    section,
    () => getRingGaugeChips(),
    (preferred) => refreshRingGaugeRovingTabindex(preferred),
    null
  );
  if (!section.getAttribute('aria-label')) {
    section.setAttribute('aria-label', 'CPU metric rings');
  }
  section.dataset.ringGaugeKbWired = '1';
}

/** History sparklines section (CPU · Freq · Temp canvases). */
function getHistorySparklineSection() {
  const canvas =
    document.getElementById('usage-history-chart') ||
    document.getElementById('gpu-history-chart') ||
    document.getElementById('frequency-history-chart') ||
    document.getElementById('temperature-history-chart');
  if (!canvas) return null;
  return (
    canvas.closest('.history-section') ||
    canvas.closest('section') ||
    canvas.parentElement?.parentElement ||
    null
  );
}

/**
 * Focusable history sparkline containers in DOM order (CPU · Freq · Temp).
 */
function getHistorySparklineChips() {
  const section = getHistorySparklineSection();
  if (!section) return [];
  return Array.from(section.querySelectorAll('.history-chart-container')).filter(
    (el) => {
      if (!el || el.hidden) return false;
      return (
        el.getClientRects().length > 0 ||
        el.offsetParent !== null ||
        section.contains(el)
      );
    }
  );
}

function refreshHistorySparklineRovingTabindex(preferred) {
  const chips = getHistorySparklineChips();
  if (!chips.length) return;
  const focused = chips.find((el) => el === document.activeElement);
  const current =
    (preferred && chips.includes(preferred) && preferred) ||
    focused ||
    chips.find((el) => el.tabIndex === 0) ||
    chips[0];
  for (const el of chips) {
    el.tabIndex = el === current ? 0 : -1;
  }
}

function ensureHistorySparklineKbStyles() {
  if (document.getElementById('mac-stats-history-sparkline-kb-styles')) return;
  const style = document.createElement('style');
  style.id = 'mac-stats-history-sparkline-kb-styles';
  style.textContent = `
    .history-sparkline-kb-hint {
      margin: 6px 0 0;
      font-size: 11px;
      opacity: 0.72;
      width: 100%;
      flex-basis: 100%;
      grid-column: 1 / -1;
    }
    .history-chart-container[role="button"] {
      cursor: pointer;
      outline: none;
    }
    .history-chart-container[role="button"]:focus-visible {
      box-shadow:
        0 0 0 2px color-mix(in srgb, var(--accent, #0a84ff) 55%, transparent),
        inset 0 1px 0 rgba(255, 255, 255, 0.58);
    }
  `;
  document.head.appendChild(style);
}

/** Soft tip under history sparklines (ring-gauge / power-strip kb-hint parity). */
function ensureHistorySparklineKbHint() {
  ensureHistorySparklineKbStyles();
  const section = getHistorySparklineSection();
  if (!section) return;
  let hint = document.getElementById('history-sparkline-kb-hint');
  if (!hint) {
    hint = document.createElement('div');
    hint.id = 'history-sparkline-kb-hint';
    hint.className = 'history-sparkline-kb-hint';
    hint.setAttribute('aria-hidden', 'true');
    section.appendChild(hint);
  }
  hint.textContent =
    'Tab or click a chart · ← → / h l · Home/End move · at start crosses to temperature ring · at end crosses to battery strip · Enter / Space jumps to ring';
}

/**
 * Jump from a history sparkline to its ring (power-strip strip parity —
 * scroll + flash; no toggle / copy steal).
 */
function activateHistorySparkline(container) {
  if (!container) return;
  const canvas = container.querySelector('canvas.history-chart');
  const id = canvas?.id || '';
  if (id === 'usage-history-chart') {
    openCpuRingFromStrip();
    return;
  }
  if (id === 'gpu-history-chart') {
    openGpuRingFromStrip();
    return;
  }
  if (id === 'frequency-history-chart') {
    openFreqRingFromStrip();
    return;
  }
  if (id === 'temperature-history-chart') {
    openTempRingFromStrip();
  }
}

/**
 * History sparkline toolbar keyboard — focus CPU · Freq · Temp charts,
 * then ←→ / h l / Home/End (ring-gauge / power-strip parity). Enter/Space
 * jumps to the matching ring.
 */
function ensureHistorySparklineKeyboard() {
  const section = getHistorySparklineSection();
  if (!section) return;
  ensureHistorySparklineKbHint();
  const chips = getHistorySparklineChips();
  for (const el of chips) {
    if (!el.getAttribute('role')) el.setAttribute('role', 'button');
    const canvas = el.querySelector('canvas.history-chart');
    const id = canvas?.id || '';
    let title = 'Show related ring';
    if (id === 'usage-history-chart') title = 'Show CPU ring';
    else if (id === 'gpu-history-chart') title = 'Show GPU ring';
    else if (id === 'frequency-history-chart') title = 'Show frequency ring';
    else if (id === 'temperature-history-chart') title = 'Show temperature ring';
    if (!el.title) el.title = title;
    if (!el.getAttribute('aria-label')) {
      el.setAttribute(
        'aria-label',
        el.getAttribute('aria-label') || title
      );
    }
    if (el.dataset.historySparkActivateWired !== '1') {
      el.dataset.historySparkActivateWired = '1';
      el.addEventListener('click', (e) => {
        if (e.target.closest('select, input, button, a, label')) return;
        e.preventDefault();
        activateHistorySparkline(el);
      });
      el.addEventListener('keydown', (e) => {
        if (e.key !== 'Enter' && e.key !== ' ') return;
        if (e.metaKey || e.ctrlKey || e.altKey) return;
        e.preventDefault();
        e.stopPropagation();
        activateHistorySparkline(el);
      });
    }
  }
  refreshHistorySparklineRovingTabindex();
  if (section.dataset.historySparkChainKbWired !== '1') {
    section.dataset.historySparkChainKbWired = '1';
    section.addEventListener(
      'keydown',
      (e) => {
        if (e.metaKey || e.ctrlKey || e.altKey) return;
        const sparkChips = getHistorySparklineChips();
        if (!sparkChips.length) return;
        const idx = sparkChips.indexOf(document.activeElement);
        if (idx < 0) return;
        const back =
          e.key === 'ArrowLeft' ||
          e.key === 'h' ||
          e.key === 'ArrowUp' ||
          e.key === 'k';
        const forward =
          e.key === 'ArrowRight' ||
          e.key === 'l' ||
          e.key === 'ArrowDown' ||
          e.key === 'j';
        if (back && idx === 0) {
          if (tryChainSparklineToRingLast()) {
            e.preventDefault();
            e.stopPropagation();
          }
        } else if (forward && idx === sparkChips.length - 1) {
          if (tryChainSparklineToPowerStripFirst()) {
            e.preventDefault();
            e.stopPropagation();
          }
        }
      },
      true
    );
  }
  wireToolbarKeyboard(
    section,
    () => getHistorySparklineChips(),
    (preferred) => refreshHistorySparklineRovingTabindex(preferred),
    null
  );
  if (!section.getAttribute('aria-label')) {
    section.setAttribute('aria-label', 'CPU history sparklines');
  }
  section.dataset.historySparkKbWired = '1';
}

const TOOLBAR_NAV_KEYS = new Set([
  'ArrowRight',
  'ArrowLeft',
  'ArrowUp',
  'ArrowDown',
  'Home',
  'End',
  'h',
  'l',
  'j',
  'k',
]);

/** Pick a sensible default item when the toolbar has focus but no item is focused yet. */
function seedToolbarFocusItem(items) {
  if (!items.length) return null;
  return (
    items.find(
      (el) =>
        el.classList?.contains('is-active') ||
        el.classList?.contains('section-open') ||
        el.getAttribute('aria-pressed') === 'true'
    ) || items[0]
  );
}

/**
 * Shared ←→ / h l / Home/End handler for chip toolbars (filter rows, ring gauges,
 * power strip, icon line). Uses capture so WKWebView delivers keys on <button>.
 * Returns true when the key was handled.
 */
function handleToolbarArrowKeydown(e, container, getItems, refreshRoving) {
  if (e.metaKey || e.ctrlKey || e.altKey) return false;
  if (!TOOLBAR_NAV_KEYS.has(e.key)) return false;
  const items = getItems();
  if (!items.length) return false;

  const active = document.activeElement;
  if (active !== container && !container.contains(active)) return false;

  let idx = items.indexOf(active);
  if (idx < 0) {
    const seed = seedToolbarFocusItem(items);
    if (!seed) return false;
    refreshRoving(seed);
    seed.focus();
    idx = items.indexOf(document.activeElement);
    if (idx < 0) idx = items.indexOf(seed);
    if (idx < 0) return false;
  }

  let next = -1;
  if (
    e.key === 'ArrowRight' ||
    e.key === 'l' ||
    e.key === 'ArrowDown' ||
    e.key === 'j'
  ) {
    next = Math.min(idx + 1, items.length - 1);
  } else if (
    e.key === 'ArrowLeft' ||
    e.key === 'h' ||
    e.key === 'ArrowUp' ||
    e.key === 'k'
  ) {
    next = Math.max(idx - 1, 0);
  } else if (e.key === 'Home') {
    next = 0;
  } else if (e.key === 'End') {
    next = items.length - 1;
  } else {
    return false;
  }

  if (next === idx) {
    e.preventDefault();
    e.stopPropagation();
    return true;
  }
  refreshRoving(items[next]);
  items[next].focus();
  e.preventDefault();
  e.stopPropagation();
  return true;
}

function wireToolbarKeyboard(container, getItems, refreshRoving, hintText) {
  if (!container) return;
  refreshRoving();
  if (hintText) {
    let hint = container.querySelector(
      ':scope > .toolbar-kb-hint, :scope > .filter-chip-kb-hint'
    );
    if (!hint) {
      hint = document.createElement('div');
      hint.className = 'toolbar-kb-hint filter-chip-kb-hint';
      hint.setAttribute('aria-hidden', 'true');
      container.appendChild(hint);
    }
    hint.textContent = hintText;
  }
  if (container.dataset.toolbarKbWired === '1') return;
  container.dataset.toolbarKbWired = '1';
  if (!container.getAttribute('role')) {
    container.setAttribute('role', 'toolbar');
  }
  container.addEventListener(
    'click',
    (e) => {
      const items = getItems();
      if (!items.length) return;
      let node = e.target;
      while (node && node !== container) {
        if (items.includes(node)) {
          refreshRoving(node);
          node.focus();
          return;
        }
        node = node.parentElement;
      }
    },
    true
  );
  container.addEventListener(
    'keydown',
    (e) => {
      handleToolbarArrowKeydown(e, container, getItems, refreshRoving);
    },
    true
  );
  container.addEventListener('focusin', (e) => {
    const items = getItems();
    if (items.includes(e.target)) refreshRoving(e.target);
  });
}

/** Visible filter / kind chip buttons inside a chip row. */
function getFilterChipButtons(wrap) {
  if (!wrap) return [];
  return Array.from(wrap.querySelectorAll(':scope > button')).filter((el) => {
    if (!el || el.hidden || el.disabled) return false;
    return el.getClientRects().length > 0 || el.offsetParent !== null || wrap.contains(el);
  });
}

function refreshFilterChipRovingTabindex(wrap, preferred) {
  const chips = getFilterChipButtons(wrap);
  if (!chips.length) return;
  const focused = chips.find((el) => el === document.activeElement);
  const current =
    (preferred && chips.includes(preferred) && preferred) ||
    focused ||
    chips.find((el) => el.tabIndex === 0) ||
    chips[0];
  for (const el of chips) {
    el.tabIndex = el === current ? 0 : -1;
  }
}

function ensureFilterChipKbStyles() {
  if (document.getElementById('mac-stats-filter-chip-kb-styles')) return;
  const style = document.createElement('style');
  style.id = 'mac-stats-filter-chip-kb-styles';
  style.textContent = `
    .filter-chip-kb-hint {
      margin: 2px 0 0;
      font-size: 11px;
      opacity: 0.72;
      width: 100%;
      flex-basis: 100%;
    }
    .processes-filter-chips,
    .monitors-filter-chips,
    .logs-filter-chips,
    .disk-cleanup-filter-chips,
    .chat-filter-chips,
    .ops-session-kind-chips,
    .ops-memory-kind-chips,
    .ops-runs-lane-chips,
    .ops-agents-enabled-chips,
    .ops-schedules-kind-chips {
      flex-wrap: wrap;
    }
  `;
  document.head.appendChild(style);
}

/**
 * Filter-chip toolbar keyboard — click or Tab to a chip, then ←→ / h l / Home/End.
 * Exposed on window for ollama.js / agent-ops.js.
 */
function wireFilterChipToolbarKeyboard(wrap) {
  if (!wrap) return;
  ensureFilterChipKbStyles();
  const hint =
    'Tab or click a chip · ← → / h l · Home/End move · Enter / Space selects';
  wireToolbarKeyboard(
    wrap,
    () => getFilterChipButtons(wrap),
    (preferred) => refreshFilterChipRovingTabindex(wrap, preferred),
    hint
  );
  // Legacy flag — ensure*FilterChips checks this before re-wiring listeners.
  wrap.dataset.filterChipKbWired = '1';
}
window.wireFilterChipToolbarKeyboard = wireFilterChipToolbarKeyboard;

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
        document.title = "mac-stats · glad you're here";
      } catch (_) {}
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

function tryChainProcessDetailsHeaderToHero() {
  const body = document.getElementById("process-details-body");
  const hero = body?.querySelector(".process-detail-hero");
  if (!hero) return false;
  const items = getProcessDetailHeroToolbarItems(hero);
  if (!items.length) return false;
  refreshProcessDetailHeroToolbarRovingTabindex(hero, items[0]);
  items[0].focus();
  return true;
}

function tryChainProcessDetailsHeroToHeader() {
  const header = processDetailsModal?.querySelector(".settings-header");
  if (!header || typeof window.getModalHeaderToolbarItems !== "function") return false;
  const items = window.getModalHeaderToolbarItems(
    header,
    "process-details-title",
    "close-process-details"
  );
  if (!items.length) return false;
  const target = items[items.length - 1];
  if (typeof window.refreshModalHeaderRovingTabindex === "function") {
    window.refreshModalHeaderRovingTabindex(
      header,
      "process-details-title",
      "close-process-details",
      target
    );
  }
  target.focus();
  return true;
}

function tryChainProcessDetailsHeroToForceQuit() {
  const body = document.getElementById("process-details-body");
  const section = body?.querySelector(".force-quit-section");
  if (!section) return false;
  const items = getForceQuitToolbarItems(section);
  if (!items.length) return false;
  refreshForceQuitToolbarRovingTabindex(section, items[0]);
  items[0].focus();
  return true;
}

function tryChainForceQuitToHero() {
  const body = document.getElementById("process-details-body");
  const hero = body?.querySelector(".process-detail-hero");
  if (!hero) return false;
  const items = getProcessDetailHeroToolbarItems(hero);
  if (!items.length) return false;
  const target = items[items.length - 1];
  refreshProcessDetailHeroToolbarRovingTabindex(hero, target);
  target.focus();
  return true;
}

function wireProcessDetailsHeaderToolbarKeyboard(header) {
  if (!header || typeof window.wireModalHeaderToolbarKeyboard !== "function") return;
  window.wireModalHeaderToolbarKeyboard(header, {
    titleId: "process-details-title",
    closeId: "close-process-details",
    ariaLabel: "Process details header",
    wireKey: "processDetailsHeaderToolbarKbWired",
    hintText:
      "← → / h l · Home/End move · Enter / Space on Close closes · at end crosses to name",
    chainForwardFromEnd: () => tryChainProcessDetailsHeaderToHero(),
  });
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
  const header = processDetailsModal.querySelector(".settings-header");
  if (header) wireProcessDetailsHeaderToolbarKeyboard(header);
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

function snapshotCopyFlash(el) {
  if (!el) return null;
  return {
    classJustSaved: el.classList.contains("is-just-saved"),
    text: el.textContent || "",
    saveFlashOriginal: el._saveFlashOriginalLabel ?? null,
    saveFlashTimer: el._saveFlashTimer ?? null,
  };
}

function wireProcessDetailCopyButton(el, copyValue, idleLabel, prevFlash, failMsg, onOk) {
  if (!el) return;
  const idle = String(idleLabel || "");
  el._saveFlashOriginalLabel =
    (prevFlash && prevFlash.saveFlashOriginal) || idle;
  if (prevFlash && prevFlash.classJustSaved) {
    el.classList.add("is-just-saved");
    el.textContent = prevFlash.text || "Copied";
    if (prevFlash.saveFlashTimer) {
      clearTimeout(prevFlash.saveFlashTimer);
    }
    el._saveFlashTimer = setTimeout(() => {
      el.classList.remove("is-just-saved");
      el.textContent = idle;
      el._saveFlashOriginalLabel = idle;
      el._saveFlashTimer = null;
    }, 1600);
  }
  const copy = async (e) => {
    if (e) {
      e.preventDefault();
      e.stopPropagation();
    }
    if (el.classList.contains("is-just-saved")) return;
    const ok = await copyTextToClipboard(String(copyValue));
    if (!ok) {
      alert(failMsg || "Could not copy.");
      return;
    }
    if (typeof onOk === "function") onOk();
    if (typeof flashSaveButton === "function") {
      flashSaveButton(el, { savedLabel: "Copied", durationMs: 1600 });
    } else {
      el._saveFlashOriginalLabel = idle;
      el.classList.add("is-just-saved");
      el.textContent = "Copied";
      clearTimeout(el._saveFlashTimer);
      el._saveFlashTimer = setTimeout(() => {
        el.classList.remove("is-just-saved");
        el.textContent = idle;
        el._saveFlashOriginalLabel = null;
        el._saveFlashTimer = null;
      }, 1600);
    }
  };
  el.addEventListener("click", copy);
  el.addEventListener("keydown", (e) => {
    if (e.key === "Enter" || e.key === " ") {
      void copy(e);
    }
  });
}

/** Focusable Top Processes detail hero toolbar items (name · PID). */
function getProcessDetailHeroToolbarItems(hero) {
  const wrap = hero || document.querySelector(".process-detail-hero");
  if (!wrap) return [];
  const items = [];
  const name = wrap.querySelector(".process-detail-name");
  const pid = wrap.querySelector(".process-detail-pid");
  if (name && wrap.contains(name) && !name.hidden) items.push(name);
  if (pid && wrap.contains(pid) && !pid.hidden) items.push(pid);
  return items.filter((el) => {
    if (!el || el.hidden) return false;
    return el.getClientRects().length > 0 || wrap.contains(el);
  });
}

function refreshProcessDetailHeroToolbarRovingTabindex(hero, preferred) {
  const wrap = hero || document.querySelector(".process-detail-hero");
  const items = getProcessDetailHeroToolbarItems(wrap);
  if (!items.length) return;
  const focused = items.find((el) => el === document.activeElement);
  const current =
    (preferred && items.includes(preferred) && preferred) ||
    focused ||
    items.find((el) => el.tabIndex === 0) ||
    items[0];
  for (const el of items) {
    el.tabIndex = el === current ? 0 : -1;
  }
}

function ensureProcessDetailHeroToolbarKbHint(hero) {
  const wrap = hero || document.querySelector(".process-detail-hero");
  if (!wrap) return;
  let hint = wrap.querySelector(".process-detail-hero-toolbar-kb-hint");
  if (!hint) {
    hint = document.createElement("div");
    hint.className = "process-detail-hero-toolbar-kb-hint";
    hint.setAttribute("aria-hidden", "true");
    wrap.appendChild(hint);
  }
  const items = getProcessDetailHeroToolbarItems(wrap);
  hint.hidden = items.length < 2;
  hint.textContent =
    "← → / h l · Home/End move · Enter / Space copies · at start crosses to header · at end crosses to Force Quit";
}

/**
 * Top Processes detail hero toolbar keyboard — focus name · PID, then ←→ / h l /
 * Home/End (Disk Cleanup add-scope / Monitors detail parity). Enter/Space keeps copy.
 */
function ensureProcessDetailHeroToolbarKeyboard(hero) {
  const wrap = hero || document.querySelector(".process-detail-hero");
  if (!wrap) return;
  ensureProcessDetailHeroToolbarKbHint(wrap);
  refreshProcessDetailHeroToolbarRovingTabindex(wrap);
  if (wrap.dataset.processDetailHeroToolbarKbWired === "1") return;
  wrap.dataset.processDetailHeroToolbarKbWired = "1";
  if (!wrap.getAttribute("role")) wrap.setAttribute("role", "toolbar");
  if (!wrap.getAttribute("aria-label")) {
    wrap.setAttribute("aria-label", "Process name and PID");
  }
  wrap.addEventListener("focusin", (e) => {
    const items = getProcessDetailHeroToolbarItems(wrap);
    if (items.includes(e.target)) {
      refreshProcessDetailHeroToolbarRovingTabindex(wrap, e.target);
      ensureProcessDetailHeroToolbarKbHint(wrap);
    }
  });
  wrap.addEventListener("keydown", (e) => {
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    const items = getProcessDetailHeroToolbarItems(wrap);
    if (!items.length) return;
    const idx = items.indexOf(document.activeElement);
    if (idx < 0) return;
    if (e.key === "Enter" || e.key === " ") return;
    let next = -1;
    if (
      e.key === "ArrowRight" ||
      e.key === "l" ||
      e.key === "ArrowDown" ||
      e.key === "j"
    ) {
      if (idx === items.length - 1) {
        if (tryChainProcessDetailsHeroToForceQuit()) {
          e.preventDefault();
          e.stopPropagation();
        }
        return;
      }
      next = idx + 1;
    } else if (
      e.key === "ArrowLeft" ||
      e.key === "h" ||
      e.key === "ArrowUp" ||
      e.key === "k"
    ) {
      if (idx === 0) {
        if (tryChainProcessDetailsHeroToHeader()) {
          e.preventDefault();
          e.stopPropagation();
        }
        return;
      }
      next = idx - 1;
    } else if (e.key === "Home") {
      next = 0;
    } else if (e.key === "End") {
      next = items.length - 1;
    } else {
      return;
    }
    e.preventDefault();
    e.stopPropagation();
    if (next === idx) return;
    refreshProcessDetailHeroToolbarRovingTabindex(wrap, items[next]);
    items[next].focus();
  });
}

/** Focusable Top Processes force-quit toolbar items (Advanced summary · Force Quit). */
function getForceQuitToolbarItems(section) {
  const wrap = section || document.querySelector(".force-quit-section");
  if (!wrap) return [];
  const details = wrap.querySelector(".force-quit-advanced");
  const items = [];
  const summary = details?.querySelector("summary");
  const quit = wrap.querySelector("#force-quit-process-btn");
  if (summary && wrap.contains(summary) && !summary.hidden) items.push(summary);
  if (quit && wrap.contains(quit) && !quit.hidden) items.push(quit);
  return items.filter((el) => {
    if (!el || el.hidden) return false;
    return el.getClientRects().length > 0 || wrap.contains(el);
  });
}

function refreshForceQuitToolbarRovingTabindex(section, preferred) {
  const wrap = section || document.querySelector(".force-quit-section");
  const items = getForceQuitToolbarItems(wrap);
  if (!items.length) return;
  const focused = items.find((el) => el === document.activeElement);
  const current =
    (preferred && items.includes(preferred) && preferred) ||
    focused ||
    items.find((el) => el.tabIndex === 0) ||
    items[0];
  for (const el of items) {
    el.tabIndex = el === current ? 0 : -1;
  }
}

function ensureForceQuitToolbarKbHint(section) {
  const wrap = section || document.querySelector(".force-quit-section");
  if (!wrap) return;
  let hint = wrap.querySelector(".force-quit-toolbar-kb-hint");
  if (!hint) {
    hint = document.createElement("div");
    hint.className = "force-quit-toolbar-kb-hint";
    hint.setAttribute("aria-hidden", "true");
    wrap.appendChild(hint);
  }
  const items = getForceQuitToolbarItems(wrap);
  hint.hidden = items.length < 2;
  hint.textContent =
    "← → / h l · Home/End move · Enter / Space on buttons · at start crosses to PID";
}

/**
 * Top Processes force-quit toolbar keyboard — focus Advanced summary · Force Quit,
 * then ←→ / h l / Home/End (detail hero toolbar parity).
 */
function ensureForceQuitToolbarKeyboard(section) {
  const wrap = section || document.querySelector(".force-quit-section");
  if (!wrap) return;
  ensureForceQuitToolbarKbHint(wrap);
  refreshForceQuitToolbarRovingTabindex(wrap);
  if (wrap.dataset.forceQuitToolbarKbWired === "1") return;
  wrap.dataset.forceQuitToolbarKbWired = "1";
  if (!wrap.getAttribute("role")) wrap.setAttribute("role", "toolbar");
  if (!wrap.getAttribute("aria-label")) {
    wrap.setAttribute("aria-label", "Force quit process");
  }
  const details = wrap.querySelector(".force-quit-advanced");
  if (details) {
    details.addEventListener("toggle", () => {
      refreshForceQuitToolbarRovingTabindex(wrap);
      ensureForceQuitToolbarKbHint(wrap);
    });
  }
  wrap.addEventListener("focusin", (e) => {
    const items = getForceQuitToolbarItems(wrap);
    if (items.includes(e.target)) {
      refreshForceQuitToolbarRovingTabindex(wrap, e.target);
      ensureForceQuitToolbarKbHint(wrap);
    }
  });
  wrap.addEventListener("keydown", (e) => {
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    const items = getForceQuitToolbarItems(wrap);
    if (!items.length) return;
    const idx = items.indexOf(document.activeElement);
    if (idx < 0) return;
    if (e.key === "Enter" || e.key === " ") return;
    let next = -1;
    if (
      e.key === "ArrowRight" ||
      e.key === "l" ||
      e.key === "ArrowDown" ||
      e.key === "j"
    ) {
      next = Math.min(idx + 1, items.length - 1);
    } else if (
      e.key === "ArrowLeft" ||
      e.key === "h" ||
      e.key === "ArrowUp" ||
      e.key === "k"
    ) {
      if (idx === 0) {
        if (tryChainForceQuitToHero()) {
          e.preventDefault();
          e.stopPropagation();
        }
        return;
      }
      next = idx - 1;
    } else if (e.key === "Home") {
      next = 0;
    } else if (e.key === "End") {
      next = items.length - 1;
    } else {
      return;
    }
    e.preventDefault();
    e.stopPropagation();
    if (next === idx) return;
    refreshForceQuitToolbarRovingTabindex(wrap, items[next]);
    items[next].focus();
  });
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

    // Live refresh replaces innerHTML — keep Advanced open + Force Quit confirm UI.
    const prevAdvanced = body.querySelector(".force-quit-advanced");
    const advancedWasOpen = !!(prevAdvanced && prevAdvanced.open);
    const prevQuitBtn = body.querySelector("#force-quit-process-btn");
    const quitUi = prevQuitBtn
      ? {
          confirming: prevQuitBtn.dataset.confirmArmed === "1",
          quitting: prevQuitBtn.dataset.quitting === "1",
          disabled: !!prevQuitBtn.disabled,
          text: prevQuitBtn.textContent || "Force Quit Process",
          classConfirming: prevQuitBtn.classList.contains("is-confirming"),
          classJustSaved: prevQuitBtn.classList.contains("is-just-saved"),
          saveFlashOriginal: prevQuitBtn._saveFlashOriginalLabel ?? null,
        }
      : null;
    const prevPidEl = body.querySelector(".process-detail-pid");
    const pidUi = snapshotCopyFlash(prevPidEl);
    const prevNameEl = body.querySelector(".process-detail-name");
    const nameUi = snapshotCopyFlash(prevNameEl);
    
    body.innerHTML = `
      <div class="process-detail-hero">
        <button type="button" class="process-detail-name" title="Click to copy name" aria-label="Copy name ${name}">${name}</button>
        <button type="button" class="process-detail-pid" title="Click to copy PID" aria-label="Copy PID ${details.pid}">PID ${details.pid}</button>
      </div>
      <div class="process-detail-section">
        <div class="process-detail-row">
          <span class="process-detail-label">Current CPU</span>
          <span class="process-detail-value">${details.cpu.toFixed(1)}%</span>
          <span class="process-detail-label">Current GPU</span>
          <span class="process-detail-value">${(Number(details.gpu) || 0) >= 0.1 ? `${Number(details.gpu).toFixed(1)}%` : "—"}</span>
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
        <details class="force-quit-advanced"${advancedWasOpen ? " open" : ""}>
          <summary>Advanced</summary>
          <p class="force-quit-hint">Force Quit ends the process immediately.</p>
          <button id="force-quit-process-btn" class="force-quit-btn" type="button">Force Quit Process</button>
        </details>
      </div>
    `;

    const nameEl = body.querySelector(".process-detail-name");
    if (nameEl) {
      wireProcessDetailCopyButton(
        nameEl,
        details.name,
        details.name,
        nameUi,
        "Could not copy process name.",
        () => {
          requestProcessNameCopyFlash(details.name);
          applyProcessNameCopyFlash(document.getElementById("process-list"));
        }
      );
    }
    const pidEl = body.querySelector(".process-detail-pid");
    if (pidEl) {
      wireProcessDetailCopyButton(
        pidEl,
        String(details.pid),
        `PID ${details.pid}`,
        pidUi,
        "Could not copy PID."
      );
    }
    const heroEl = body.querySelector(".process-detail-hero");
    if (heroEl) ensureProcessDetailHeroToolbarKeyboard(heroEl);

    // Set up force quit button handler (remove old listeners first by cloning)
    const forceQuitBtn = document.getElementById("force-quit-process-btn");
    if (forceQuitBtn) {
      if (quitUi) {
        forceQuitBtn.dataset.confirmArmed = quitUi.confirming ? "1" : "0";
        forceQuitBtn.dataset.quitting = quitUi.quitting ? "1" : "0";
        forceQuitBtn.disabled = quitUi.disabled || quitUi.quitting;
        forceQuitBtn.textContent = quitUi.text;
        forceQuitBtn.classList.toggle("is-confirming", quitUi.classConfirming);
        forceQuitBtn.classList.toggle("is-just-saved", quitUi.classJustSaved);
        if (quitUi.saveFlashOriginal != null) {
          forceQuitBtn._saveFlashOriginalLabel = quitUi.saveFlashOriginal;
        }
      }

      // Clone and replace to remove old event listeners when refreshing
      const newBtn = forceQuitBtn.cloneNode(true);
      if (forceQuitBtn._saveFlashOriginalLabel != null) {
        newBtn._saveFlashOriginalLabel = forceQuitBtn._saveFlashOriginalLabel;
      }
      forceQuitBtn.parentNode.replaceChild(newBtn, forceQuitBtn);
      
      newBtn.addEventListener("click", async () => {
        // Ignore while quit is in flight (blocks double confirm / double invoke).
        if (newBtn.dataset.quitting === "1" || newBtn.disabled) return;

        // WKWebView: window.confirm()/alert() are unreliable — two-click confirm instead.
        if (newBtn.dataset.confirmArmed !== "1") {
          newBtn.dataset.confirmArmed = "1";
          newBtn.classList.add("is-confirming");
          newBtn.textContent = "Click again to confirm Force Quit";
          setTimeout(() => {
            if (newBtn.dataset.confirmArmed === "1" && newBtn.dataset.quitting !== "1") {
              newBtn.dataset.confirmArmed = "0";
              newBtn.classList.remove("is-confirming");
              newBtn.textContent = "Force Quit Process";
            }
          }, 4000);
          return;
        }

        newBtn.dataset.quitting = "1";
        newBtn.dataset.confirmArmed = "0";
        newBtn.disabled = true;
        newBtn.classList.remove("is-confirming");
        if (newBtn._saveFlashOriginalLabel == null) {
          newBtn._saveFlashOriginalLabel = "Force Quit Process";
        }
        newBtn.textContent = "Quitting…";

        try {
          if (!invoke) {
            invoke = getInvoke();
            if (!invoke) {
              console.error("Cannot force quit: Tauri invoke not available");
              newBtn.dataset.quitting = "0";
              newBtn.disabled = false;
              newBtn.textContent = newBtn._saveFlashOriginalLabel || "Force Quit Process";
              return;
            }
          }

          await invoke("force_quit_process", { pid });

          if (typeof flashSaveButton === "function") {
            flashSaveButton(newBtn, { savedLabel: "Quit", durationMs: 900 });
            await new Promise((r) => setTimeout(r, 450));
          }

          // Clear refresh interval and close modal
          closeProcessDetailsModal();

          // Force immediate refresh of process list (bypass 15-second throttle)
          window._forceProcessUpdate = true;
          if (window.refreshData) {
            await window.refreshData();
          }
        } catch (error) {
          console.error("Failed to force quit process:", error);
          newBtn.dataset.quitting = "0";
          newBtn.disabled = false;
          newBtn.classList.remove("is-just-saved");
          newBtn.textContent = newBtn._saveFlashOriginalLabel || "Force Quit Process";
          newBtn._saveFlashOriginalLabel = null;
        }
      });
    }
    const forceQuitSection = body.querySelector(".force-quit-section");
    if (forceQuitSection) ensureForceQuitToolbarKeyboard(forceQuitSection);
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
      const header = processDetailsModal.querySelector(".settings-header");
      if (header) wireProcessDetailsHeaderToolbarKeyboard(header);
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
    
    // Live metrics every 5s (was 2s). Faster polls forced full work + DOM rebuild.
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
    }, 5000);
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
    const batteryInfo = batteryLevel && batteryLevel.closest
      ? batteryLevel.closest('.battery-info')
      : document.querySelector('#battery-power-strip .battery-info');
    if (batteryInfo) {
      // Soft amber wash when ≤20% and not charging (menu-bar Bat cue parity).
      batteryInfo.classList.toggle('is-low', level <= 20 && !isCharging);
    }

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
    const batteryInfo = batteryLevel && batteryLevel.closest
      ? batteryLevel.closest('.battery-info')
      : document.querySelector('#battery-power-strip .battery-info');
    if (batteryInfo) batteryInfo.classList.remove('is-low');
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
/** List filter: all | up | down (Debug Log chip parity). */
let monitorsFilterMode = 'all';

/** Disk Cleanup section collapsed (module-level for empty-CTA expand). */
let diskCleanupCollapsed = true;
/** Category list filter: all | reclaim | clean (Monitors All/Up/Down parity). */
let diskCleanupFilterMode = 'all';
/** Shallow status poll while Disk Cleanup is collapsed (collapsed glance). */
let diskCleanupGlanceInterval = null;

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
  return getSectionCollapsed('monitors_collapsed');
}

function saveMonitorsCollapsedState(collapsed) {
  setSectionCollapsed('monitors_collapsed', collapsed);
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
  wireMonitorsSummaryClick();
  ensureMonitorsFilterChips();
  ensureMonitorsCollapsedGlance();

  // Always load monitors to calculate height, even when collapsed
  loadMonitors().then(() => {
    updateMonitorsHeight();
  });
  updateMonitorsSummary();
  
  // Restore saved state
  monitorsCollapsed = getMonitorsCollapsedState();
  updateMonitorsStatusDot();

  // Make header clickable/keyboardable to toggle collapse/expand
  const applyMonitorsCollapsed = () => {
    const section = document.querySelector('.monitors-section');
    const divider = document.getElementById('monitors-ollama-divider');

    setIconPaneVisibility(section, content, monitorsCollapsed, divider);

    if (monitorsCollapsed) {
      // Keep a light summary poll so the collapsed glance stays fresh (no list rebuild).
      if (monitorsUpdateInterval) {
        clearInterval(monitorsUpdateInterval);
        monitorsUpdateInterval = null;
      }
      monitorsUpdateInterval = setInterval(() => {
        updateMonitorsSummary();
      }, 30000);
    } else {
      if (monitorsUpdateInterval) {
        clearInterval(monitorsUpdateInterval);
        monitorsUpdateInterval = null;
      }
      // Start interval if not already running (but don't call immediately)
      monitorsUpdateInterval = setInterval(() => {
        updateMonitorsSummary();
        loadMonitors().then(() => {
          updateMonitorsHeight();
        });
      }, 30000);
    }
    updateMonitorsStatusDot();
    header.setAttribute('aria-expanded', String(!monitorsCollapsed));
    syncSectionIcon('icon-monitors', !monitorsCollapsed);
    syncMonitorsCollapsedGlance();
  };
  applyMonitorsCollapsed();

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
        ensureMonitorAddFormToolbarKeyboard(addForm);
      }
    });
  }
  
  if (addCancel) {
    addCancel.addEventListener('click', () => {
      if (addForm) addForm.style.display = 'none';
      if (urlInput) urlInput.value = 'https://www.amvara.de/';
    });
  }
  
  let monitorsAddBusy = false;
  if (addSave && urlInput) {
    addSave.addEventListener('click', async () => {
      if (monitorsAddBusy) return;
      if (addSave.classList.contains('is-just-saved')) return;

      let url = urlInput.value.trim();
      if (!url) {
        alert('Please enter a URL');
        return;
      }

      // Add https:// if no protocol specified
      if (!url.match(/^https?:\/\//i)) {
        url = 'https://' + url;
      }

      monitorsAddBusy = true;
      addSave.disabled = true;
      addSave.classList.remove('is-just-saved');
      if (addSave._saveFlashOriginalLabel == null) {
        addSave._saveFlashOriginalLabel = addSave.textContent || 'Add Monitor';
      }
      addSave.textContent = 'Adding…';

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
        // After adding, load monitors will update the cache
        await loadMonitors();
        await updateMonitorsSummary();
        await refreshMonitorsSettingsList();

        monitorsAddBusy = false;
        addSave.disabled = false;
        if (typeof flashSaveButton === 'function') {
          flashSaveButton(addSave, { savedLabel: 'Added', durationMs: 1600 });
        } else {
          addSave.classList.add('is-just-saved');
          addSave.textContent = 'Added';
          setTimeout(() => {
            addSave.classList.remove('is-just-saved');
            addSave.textContent =
              addSave._saveFlashOriginalLabel || 'Add Monitor';
            addSave._saveFlashOriginalLabel = null;
          }, 1600);
        }
        // Keep form open so the Added flash is visible, then reset.
        setTimeout(() => {
          if (addForm) addForm.style.display = 'none';
          if (urlInput) urlInput.value = 'https://www.amvara.de/';
          if (!addSave.classList.contains('is-just-saved')) {
            addSave.textContent =
              addSave._saveFlashOriginalLabel || 'Add Monitor';
            addSave._saveFlashOriginalLabel = null;
          }
        }, 1600);
      } catch (err) {
        console.error('Failed to add monitor:', err);
        monitorsAddBusy = false;
        addSave.disabled = false;
        addSave.textContent =
          addSave._saveFlashOriginalLabel || 'Add Monitor';
        addSave._saveFlashOriginalLabel = null;
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
    return false;
  }
  console.log('[Monitors] remove_monitor invoke', monitorId);
  try {
    await invokeFn('remove_monitor', { monitorId });
    monitorStatusCache.delete(monitorId);
    await refreshMonitorsSettingsList();
    await loadMonitors();
    await updateMonitorsSummary();
    console.log('[Monitors] removed', monitorId);
    return true;
  } catch (err) {
    console.error('[Monitors] remove_monitor failed:', err);
    return false;
  }
}

/** Remove selected monitor; focus a neighbor after the list rebuilds. */
async function removeMonitorFromListRow(item) {
  if (!item || item.dataset.removing === '1') return false;
  const monitorId = item.getAttribute('data-monitor-id');
  if (!monitorId) return false;
  const list = document.getElementById('monitors-list');
  const items = list
    ? Array.from(list.querySelectorAll('.monitor-item'))
    : [];
  const idx = items.indexOf(item);
  const preferId =
    (idx >= 0 && items[idx + 1]?.getAttribute('data-monitor-id')) ||
    (idx > 0 && items[idx - 1]?.getAttribute('data-monitor-id')) ||
    null;
  item.dataset.removing = '1';
  item.classList.add('is-removing');
  const removeBtn = item.querySelector('.monitor-detail-remove');
  if (removeBtn) {
    removeBtn.disabled = true;
    removeBtn.dataset.idleLabel = removeBtn.dataset.idleLabel || 'Remove';
    removeBtn.textContent = 'Removing…';
  }
  const settingsBtn = document.querySelector(
    `.monitor-remove-btn[data-monitor-id="${CSS.escape(monitorId)}"]`
  );
  if (settingsBtn) {
    settingsBtn.disabled = true;
    settingsBtn.dataset.idleLabel = settingsBtn.dataset.idleLabel || 'Remove';
    settingsBtn.textContent = 'Removing…';
  }
  let ok = false;
  try {
    ok = await removeMonitorById(monitorId);
  } finally {
    if (item.isConnected) {
      item.dataset.removing = '0';
      item.classList.remove('is-removing');
      if (removeBtn?.isConnected) {
        removeBtn.disabled = false;
        removeBtn.textContent = removeBtn.dataset.idleLabel || 'Remove';
      }
    }
    if (settingsBtn?.isConnected) {
      settingsBtn.disabled = false;
      settingsBtn.textContent = settingsBtn.dataset.idleLabel || 'Remove';
    }
  }
  if (ok && preferId) {
    const listAfter = document.getElementById('monitors-list');
    syncMonitorsListTabOrder(listAfter, preferId);
    const next = listAfter?.querySelector?.(
      `.monitor-item[data-monitor-id="${CSS.escape(preferId)}"]`
    );
    if (next && typeof next.focus === 'function') {
      next.focus();
      if (typeof next.scrollIntoView === 'function') {
        next.scrollIntoView({ block: 'nearest' });
      }
    }
  }
  return ok;
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
    if (btn.disabled || btn.classList.contains('is-just-saved')) return;
    const monitorId = btn.dataset.monitorId;
    if (!monitorId) {
      console.error('[Monitors] Remove clicked but data-monitor-id missing');
      return;
    }
    const row = document.querySelector(
      `.monitor-item[data-monitor-id="${CSS.escape(monitorId)}"]`
    );
    if (row) {
      void removeMonitorFromListRow(row);
      return;
    }
    btn.disabled = true;
    btn.dataset.idleLabel = btn.dataset.idleLabel || 'Remove';
    btn.textContent = 'Removing…';
    removeMonitorById(monitorId).then((ok) => {
      if (!btn.isConnected) return;
      btn.disabled = false;
      btn.textContent = btn.dataset.idleLabel || 'Remove';
      if (ok && typeof flashSaveButton === 'function') {
        flashSaveButton(btn, { savedLabel: 'Removed', durationMs: 1200 });
      }
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
      settingsList.innerHTML =
        `<div class="monitors-empty monitors-settings-empty" role="status">` +
        `<div class="monitors-empty-msg">No monitors configured</div>` +
        `<div class="monitors-empty-hint">Add a site to start uptime checks.</div>` +
        `<button type="button" class="monitors-empty-cta">Add a monitor</button>` +
        `</div>`;
      settingsList.querySelector('.monitors-empty-cta')?.addEventListener('click', (e) => {
        e.preventDefault();
        const addForm = document.getElementById('add-monitor-form');
        const urlInput = document.getElementById('monitor-url-input');
        if (addForm) addForm.style.display = 'block';
        requestAnimationFrame(() => {
          if (urlInput) {
            if (!urlInput.value.trim()) urlInput.value = 'https://www.amvara.de/';
            urlInput.focus();
            urlInput.select?.();
          }
        });
      });
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

function applyMonitorsSummaryState({ anyDown, allUp, empty, slowestId }) {
  const summary = document.getElementById('monitors-summary');
  if (!summary) return;
  summary.classList.toggle('has-down', !!anyDown);
  summary.classList.toggle('is-all-up', !!allUp && !empty);
  summary.classList.toggle('is-empty', !!empty);
  summary.classList.toggle(
    'has-slowest-hint',
    !!slowestId && !!allUp && !anyDown && !empty
  );
  // Clickable glance (Agent Ops health/overview parity): empty → Add, DOWN → first down row.
  summary.setAttribute('role', 'button');
  summary.setAttribute('tabindex', '0');
  if (empty) {
    summary.title = 'Click to add a monitor';
    summary.setAttribute('aria-label', 'No monitors configured — click to add');
  } else if (anyDown) {
    summary.title = 'Click to open the first DOWN monitor';
    summary.setAttribute('aria-label', 'Monitors summary — click to open first DOWN site');
  } else if (slowestId) {
    summary.title = 'Click to open the slowest monitor';
    summary.setAttribute('aria-label', 'Monitors summary — click to open slowest site');
  } else {
    summary.title = 'Click to open the first monitor';
    summary.setAttribute('aria-label', 'Monitors summary — click to open first site');
  }
  syncMonitorsCollapsedGlance();
}

/** Summary / collapsed-glance activate (empty → Add; DOWN → first down; else slowest / first). */
function activateMonitorsSummaryGlance() {
  const summary = document.getElementById('monitors-summary');
  ensureMonitorsSectionExpanded();
  if (summary?.classList.contains('is-empty')) {
    void openMonitorsAddFlow();
    return;
  }
  const list = document.getElementById('monitors-list');
  if (!list) return;
  const visible = visibleMonitorItems(list);
  const down = visible.find((el) => el.classList.contains('is-down'));
  let first = down || visible[0];
  if (!down && window.__monitorsSlowestId) {
    const slowRow = list.querySelector(
      `.monitor-item[data-monitor-id="${CSS.escape(window.__monitorsSlowestId)}"]`
    );
    if (slowRow && slowRow.style.display !== 'none') {
      first = slowRow;
    }
  }
  if (!first) {
    void openMonitorsAddFlow();
    return;
  }
  const id = first.getAttribute('data-monitor-id');
  syncMonitorsListTabOrder(list, id);
  first.focus();
  if (typeof first.scrollIntoView === 'function') {
    first.scrollIntoView({ block: 'nearest' });
  }
  setMonitorDetailOpen(first, true);
}

/** Collapsed-section glance under Monitors header (Debug Log / Perplexity parity). */
function ensureMonitorsCollapsedGlance() {
  const header = document.getElementById('monitors-header');
  if (!header) return null;
  let glance = document.getElementById('monitors-collapsed-glance');
  if (!glance) {
    glance = document.createElement('div');
    glance.id = 'monitors-collapsed-glance';
    glance.className = 'monitors-collapsed-glance';
    glance.hidden = true;
    glance.innerHTML = '<span id="monitors-collapsed-glance-text"></span>';
    header.insertAdjacentElement('afterend', glance);
    wireMonitorsCollapsedGlanceClick(glance);
  }
  return glance;
}

function syncMonitorsCollapsedGlance() {
  const glance = ensureMonitorsCollapsedGlance();
  if (!glance) return;
  const summary = document.getElementById('monitors-summary');
  const summaryText = document.getElementById('monitors-summary-text');
  const glanceText = document.getElementById('monitors-collapsed-glance-text');
  if (!monitorsCollapsed) {
    glance.hidden = true;
    return;
  }
  glance.hidden = false;
  if (glanceText && summaryText) {
    glanceText.textContent = summaryText.textContent || 'Monitors';
  }
  glance.classList.toggle('has-down', !!summary?.classList.contains('has-down'));
  glance.classList.toggle('is-all-up', !!summary?.classList.contains('is-all-up'));
  glance.classList.toggle('is-empty', !!summary?.classList.contains('is-empty'));
  glance.classList.toggle(
    'has-slowest-hint',
    !!summary?.classList.contains('has-slowest-hint')
  );
  glance.setAttribute('role', 'button');
  glance.tabIndex = 0;
  glance.title = summary?.title || 'Show External / Monitors';
  const aria = summary?.getAttribute('aria-label');
  glance.setAttribute(
    'aria-label',
    aria || 'Monitors summary — click to expand'
  );
}

function wireMonitorsCollapsedGlanceClick(glance) {
  if (!glance || glance.dataset.monitorsCollapsedGlanceWired === '1') return;
  glance.dataset.monitorsCollapsedGlanceWired = '1';
  const activate = () => {
    activateMonitorsSummaryGlance();
  };
  glance.addEventListener('click', (e) => {
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
  glance.addEventListener('keydown', (e) => {
    if (e.key !== 'Enter' && e.key !== ' ') return;
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
}

/** Expand External / Monitors if collapsed (summary click / empty CTA). */
function ensureMonitorsSectionExpanded() {
  if (!monitorsCollapsed) {
    updateMonitorsHeight();
    return;
  }
  monitorsCollapsed = false;
  saveMonitorsCollapsedState(false);
  const content = document.getElementById('monitors-content');
  const section = document.querySelector('.monitors-section');
  const header = document.getElementById('monitors-header');
  const divider = document.getElementById('monitors-ollama-divider');
  content?.classList.remove('collapsed');
  section?.classList.remove('collapsed');
  if (divider) divider.style.display = '';
  header?.setAttribute('aria-expanded', 'true');
  syncSectionIcon('icon-monitors', true);
  updateMonitorsHeight();
  syncMonitorsCollapsedGlance();
  if (!monitorsUpdateInterval) {
    monitorsUpdateInterval = setInterval(() => {
      updateMonitorsSummary();
      loadMonitors().then(() => {
        updateMonitorsHeight();
      });
    }, 30000);
  }
}

/** Open Monitor Settings with the Add form focused (empty-state CTA). */
async function openMonitorsAddFlow() {
  ensureMonitorsSectionExpanded();
  await showMonitorsSettings();
  const addForm = document.getElementById('add-monitor-form');
  const urlInput = document.getElementById('monitor-url-input');
  if (addForm) addForm.style.display = 'block';
  requestAnimationFrame(() => {
    if (urlInput) {
      if (!urlInput.value.trim()) urlInput.value = 'https://www.amvara.de/';
      urlInput.focus();
      urlInput.select?.();
    }
  });
}

/** Visible monitor rows after All / Up / Down filter. */
function visibleMonitorItems(monitorsList) {
  if (!monitorsList) return [];
  return Array.from(monitorsList.querySelectorAll('.monitor-item')).filter(
    (el) => el.style.display !== 'none'
  );
}

/** All / Up / Down chips (Debug Log filter parity). */
function ensureMonitorsFilterChips() {
  const content = document.getElementById('monitors-content');
  const summary = document.getElementById('monitors-summary');
  if (!content || !summary) return;
  let wrap = document.getElementById('monitors-filter-chips');
  if (!wrap) {
    wrap = document.createElement('div');
    wrap.id = 'monitors-filter-chips';
    wrap.className = 'monitors-filter-chips';
    wrap.setAttribute('role', 'group');
    wrap.setAttribute('aria-label', 'Monitor status filter');
    wrap.hidden = true;
    wrap.innerHTML =
      '<button type="button" class="monitors-filter-chip is-active" data-monitors-filter="all" aria-pressed="true" title="Show every monitor">All</button>' +
      '<button type="button" class="monitors-filter-chip" data-monitors-filter="up" aria-pressed="false" title="Show UP sites only">Up <span class="monitors-filter-count" data-monitors-filter-count="up">0</span></button>' +
      '<button type="button" class="monitors-filter-chip" data-monitors-filter="down" aria-pressed="false" title="Show DOWN sites only">Down <span class="monitors-filter-count" data-monitors-filter-count="down">0</span></button>';
    summary.insertAdjacentElement('afterend', wrap);
    wrap.addEventListener('click', (e) => {
      const btn = e.target && e.target.closest && e.target.closest('[data-monitors-filter]');
      if (!btn || !wrap.contains(btn)) return;
      e.preventDefault();
      e.stopPropagation();
      setMonitorsFilterMode(btn.getAttribute('data-monitors-filter') || 'all');
    });
  }
  wireFilterChipToolbarKeyboard(wrap);
}

function setMonitorsFilterMode(mode) {
  const next = mode === 'up' || mode === 'down' ? mode : 'all';
  monitorsFilterMode = next;
  document.querySelectorAll('#monitors-filter-chips [data-monitors-filter]').forEach((btn) => {
    const on = btn.getAttribute('data-monitors-filter') === next;
    btn.classList.toggle('is-active', on);
    btn.setAttribute('aria-pressed', on ? 'true' : 'false');
  });
  applyMonitorsListFilter();
}

function ensureMonitorsFilterMissState(monitorsList, show) {
  if (!monitorsList) return;
  const existing = monitorsList.querySelector('.monitors-filter-miss');
  if (!show) {
    existing?.remove();
    return;
  }
  let wrap = existing;
  if (!wrap) {
    wrap = document.createElement('div');
    wrap.className = 'monitors-empty monitors-filter-miss';
    wrap.setAttribute('role', 'status');
    wrap.innerHTML =
      `<div class="monitors-empty-msg">Nothing matches this filter</div>` +
      `<div class="monitors-empty-hint">Try All, or clear the status filter.</div>` +
      `<button type="button" class="monitors-empty-cta monitors-clear-filter">Clear filter</button>`;
    monitorsList.appendChild(wrap);
    wrap.querySelector('.monitors-clear-filter')?.addEventListener('click', (e) => {
      e.preventDefault();
      e.stopPropagation();
      setMonitorsFilterMode('all');
    });
  }
}

function applyMonitorsListFilter() {
  ensureMonitorsFilterChips();
  const chips = document.getElementById('monitors-filter-chips');
  const monitorsList = document.getElementById('monitors-list');
  if (!monitorsList) return;

  const items = Array.from(monitorsList.querySelectorAll('.monitor-item'));
  const trueEmpty = !!monitorsList.querySelector('.monitors-list-empty');
  if (chips) chips.hidden = trueEmpty || items.length === 0;

  let upCount = 0;
  let downCount = 0;
  items.forEach((el) => {
    if (el.classList.contains('is-down')) downCount++;
    else if (!el.classList.contains('is-pending')) upCount++;
  });

  const upEl = document.querySelector('[data-monitors-filter-count="up"]');
  const downEl = document.querySelector('[data-monitors-filter-count="down"]');
  if (upEl) upEl.textContent = String(upCount);
  if (downEl) downEl.textContent = String(downCount);
  document.querySelectorAll('#monitors-filter-chips [data-monitors-filter]').forEach((btn) => {
    const key = btn.getAttribute('data-monitors-filter');
    btn.classList.toggle(
      'has-hits',
      key === 'up' ? upCount > 0 : key === 'down' ? downCount > 0 : false
    );
  });

  if (trueEmpty || items.length === 0) {
    ensureMonitorsFilterMissState(monitorsList, false);
    updateMonitorsHeight();
    return;
  }

  let visible = 0;
  items.forEach((el) => {
    const isDown = el.classList.contains('is-down');
    const isPending = el.classList.contains('is-pending');
    let show = true;
    if (monitorsFilterMode === 'down') show = isDown;
    else if (monitorsFilterMode === 'up') show = !isDown && !isPending;
    el.style.display = show ? '' : 'none';
    if (show) visible++;
  });

  ensureMonitorsFilterMissState(monitorsList, visible === 0);
  ensureMonitorsListKbHint(monitorsList, visible > 0);
  syncMonitorsListTabOrder(monitorsList);
  updateMonitorsHeight();
}

/** Empty list: title + Add a monitor CTA (Agent Ops overview empty parity). */
function ensureMonitorsListEmptyState(monitorsList, empty) {
  if (!monitorsList) return;
  const existing = monitorsList.querySelector('.monitors-list-empty');
  if (!empty) {
    existing?.remove();
    return;
  }
  monitorsList.querySelectorAll('.monitor-item').forEach((el) => el.remove());
  ensureMonitorsFilterMissState(monitorsList, false);
  ensureMonitorsListKbHint(monitorsList, false);
  let wrap = existing;
  if (!wrap) {
    wrap = document.createElement('div');
    wrap.className = 'monitors-empty monitors-list-empty';
    wrap.setAttribute('role', 'status');
    wrap.innerHTML =
      `<div class="monitors-empty-msg">Nothing watching yet</div>` +
      `<div class="monitors-empty-hint">Add a site to see uptime here.</div>` +
      `<button type="button" class="monitors-empty-cta">Add a monitor</button>`;
    monitorsList.appendChild(wrap);
    wrap.querySelector('.monitors-empty-cta')?.addEventListener('click', (e) => {
      e.preventDefault();
      e.stopPropagation();
      void openMonitorsAddFlow();
    });
  }
  applyMonitorsListFilter();
}

/** Summary click / Enter / Space → Add (empty) or first DOWN / first row. */
function wireMonitorsSummaryClick() {
  const summary = document.getElementById('monitors-summary');
  if (!summary || summary.dataset.summaryClick === '1') return;
  summary.dataset.summaryClick = '1';

  summary.addEventListener('click', (e) => {
    e.preventDefault();
    e.stopPropagation();
    activateMonitorsSummaryGlance();
  });
  summary.addEventListener('keydown', (e) => {
    if (e.key !== 'Enter' && e.key !== ' ') return;
    e.preventDefault();
    e.stopPropagation();
    activateMonitorsSummaryGlance();
  });
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
  lines.push(
    'Click or d for details · Enter / Space checks now · Delete removes · PgUp/PgDn'
  );
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

/** Brief green Copied wash on monitor row (Top Processes / Disk Cleanup parity). */
function flashMonitorRowCopied(row) {
  if (!row) return;
  if (row._monitorCopiedTimer) {
    clearTimeout(row._monitorCopiedTimer);
    row._monitorCopiedTimer = null;
  }
  row.classList.add('is-just-copied');
  row.title = 'Copied';
  row.setAttribute('aria-label', 'Copied');
  row._monitorCopiedTimer = setTimeout(() => {
    row.classList.remove('is-just-copied');
    row._monitorCopiedTimer = null;
    row.removeAttribute('title');
    row.removeAttribute('aria-label');
    const id = row.getAttribute('data-monitor-id') || '';
    const url = row.dataset.monitorUrl || '';
    // Restore hover tip on header/info (row title stays empty).
    if (id || url) {
      const status = monitorStatusCache.get(id) || {
        is_up: !row.classList.contains('is-down'),
        response_time_ms: null,
        error: '',
        checked_at: null,
      };
      applyMonitorRowTooltip(row, url || id, status);
    }
  }, 1600);
}

/**
 * Wire a monitor URL control for click/Enter/Space copy + Copied flash.
 * Survives live refresh when prevFlash is passed from the previous element.
 */
function wireMonitorUrlCopy(el, url, prevFlash) {
  if (!el || !url) return;
  const idleLabel = String(url);
  el._saveFlashOriginalLabel =
    (prevFlash && prevFlash.saveFlashOriginal) || idleLabel;
  if (prevFlash && prevFlash.classJustSaved) {
    el.classList.add('is-just-saved');
    el.textContent = prevFlash.text || 'Copied';
    if (prevFlash.saveFlashTimer) {
      clearTimeout(prevFlash.saveFlashTimer);
    }
    el._saveFlashTimer = setTimeout(() => {
      el.classList.remove('is-just-saved');
      el.textContent = idleLabel;
      el._saveFlashOriginalLabel = idleLabel;
      el._saveFlashTimer = null;
    }, 1600);
  }
  const copyUrl = async (e) => {
    if (e) {
      e.preventDefault();
      e.stopPropagation();
    }
    if (el.classList.contains('is-just-saved')) return;
    const ok = await copyTextToClipboard(idleLabel);
    if (!ok) {
      alert('Could not copy URL.');
      return;
    }
    if (typeof flashSaveButton === 'function') {
      flashSaveButton(el, { savedLabel: 'Copied', durationMs: 1600 });
    } else {
      el._saveFlashOriginalLabel = idleLabel;
      el.classList.add('is-just-saved');
      el.textContent = 'Copied';
      clearTimeout(el._saveFlashTimer);
      el._saveFlashTimer = setTimeout(() => {
        el.classList.remove('is-just-saved');
        el.textContent = idleLabel;
        el._saveFlashOriginalLabel = null;
        el._saveFlashTimer = null;
      }, 1600);
    }
    const row = el.closest && el.closest('.monitor-item');
    if (row) flashMonitorRowCopied(row);
  };
  el.addEventListener('click', copyUrl);
  el.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' || e.key === ' ') {
      void copyUrl(e);
    }
  });
}

function captureMonitorUrlFlash(el) {
  if (!el) return null;
  return {
    classJustSaved: el.classList.contains('is-just-saved'),
    text: el.textContent,
    saveFlashOriginal: el._saveFlashOriginalLabel,
    saveFlashTimer: el._saveFlashTimer,
  };
}

/** Keyboard `c` / list parity with Top Processes name + Agent Ops id copy. */
async function copyMonitorUrlFromRow(item) {
  if (!item) return false;
  const urlEl =
    item.querySelector('.monitor-detail-url') || item.querySelector('.monitor-url');
  const value = String(
    item.dataset.monitorUrl ||
      urlEl?._saveFlashOriginalLabel ||
      item.getAttribute('data-monitor-id') ||
      ''
  ).trim();
  if (!value) return false;
  if (
    item.classList.contains('is-just-copied') ||
    (urlEl && urlEl.classList.contains('is-just-saved'))
  ) {
    return true;
  }
  const ok = await copyTextToClipboard(value);
  if (!ok) {
    alert('Could not copy URL.');
    return false;
  }
  if (urlEl && typeof flashSaveButton === 'function') {
    flashSaveButton(urlEl, { savedLabel: 'Copied', durationMs: 1600 });
  } else if (urlEl) {
    const idle = urlEl._saveFlashOriginalLabel || value;
    urlEl._saveFlashOriginalLabel = idle;
    urlEl.classList.add('is-just-saved');
    urlEl.textContent = 'Copied';
    clearTimeout(urlEl._saveFlashTimer);
    urlEl._saveFlashTimer = setTimeout(() => {
      urlEl.classList.remove('is-just-saved');
      urlEl.textContent = idle;
      urlEl._saveFlashOriginalLabel = null;
      urlEl._saveFlashTimer = null;
    }, 1600);
  }
  flashMonitorRowCopied(item);
  return true;
}

/** Focusable monitor detail action items (Check now · Remove). */
function monitorUrlInputAtMoveBoundary(input, direction) {
  if (!input || input.tagName !== 'INPUT') return true;
  if (direction > 0) {
    const len = (input.value || '').length;
    return input.selectionStart === len && input.selectionEnd === len;
  }
  return input.selectionStart === 0 && input.selectionEnd === 0;
}

/** Focusable Monitors add-form toolbar items (URL · Cancel · Add Monitor). */
function getMonitorAddFormToolbarItems(wrap) {
  const form = wrap || document.getElementById('add-monitor-form');
  if (!form || form.style.display === 'none') return [];
  const ids = ['monitor-url-input', 'monitors-add-cancel', 'monitors-add-save'];
  return ids
    .map((id) => document.getElementById(id))
    .filter((el) => {
      if (!el || !form.contains(el)) return false;
      if (el.hidden || el.disabled) return false;
      return el.getClientRects().length > 0 || form.contains(el);
    });
}

function refreshMonitorAddFormToolbarRovingTabindex(wrap, preferred) {
  const form = wrap || document.getElementById('add-monitor-form');
  const items = getMonitorAddFormToolbarItems(form);
  if (!items.length) return;
  const focused = items.find((el) => el === document.activeElement);
  const current =
    (preferred && items.includes(preferred) && preferred) ||
    focused ||
    items.find((el) => el.tabIndex === 0) ||
    items[0];
  for (const el of items) {
    el.tabIndex = el === current ? 0 : -1;
  }
}

function ensureMonitorAddFormToolbarKbHint(wrap) {
  const form = wrap || document.getElementById('add-monitor-form');
  if (!form) return;
  const actions = form.querySelector('.popover-actions');
  if (!actions) return;
  let hint = actions.querySelector('.monitor-add-toolbar-kb-hint');
  if (!hint) {
    hint = document.createElement('div');
    hint.className = 'monitor-add-toolbar-kb-hint';
    hint.setAttribute('aria-hidden', 'true');
    actions.appendChild(hint);
  }
  const items = getMonitorAddFormToolbarItems(form);
  hint.hidden = items.length < 2;
  hint.textContent =
    '← → / h l · Home/End move · Enter adds from URL · buttons keep activate';
}

/**
 * Monitors add-form toolbar keyboard — focus URL · Cancel · Add Monitor, then
 * ←→ / h l / Home/End (Discord settings toolbar parity).
 */
function ensureMonitorAddFormToolbarKeyboard(wrap) {
  const form = wrap || document.getElementById('add-monitor-form');
  if (!form || form.style.display === 'none') return;
  ensureMonitorAddFormToolbarKbHint(form);
  refreshMonitorAddFormToolbarRovingTabindex(form);
  if (form.dataset.monitorAddToolbarKbWired === '1') return;
  form.dataset.monitorAddToolbarKbWired = '1';
  if (!form.getAttribute('role')) form.setAttribute('role', 'toolbar');
  if (!form.getAttribute('aria-label')) {
    form.setAttribute('aria-label', 'Add monitor');
  }
  form.addEventListener('focusin', (e) => {
    const items = getMonitorAddFormToolbarItems(form);
    if (items.includes(e.target)) {
      refreshMonitorAddFormToolbarRovingTabindex(form, e.target);
      ensureMonitorAddFormToolbarKbHint(form);
    }
  });
  form.addEventListener('keydown', (e) => {
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    const items = getMonitorAddFormToolbarItems(form);
    if (!items.length) return;
    const idx = items.indexOf(document.activeElement);
    if (idx < 0) return;
    const active = items[idx];
    if (e.key === 'Enter' || e.key === ' ') {
      if (
        active?.id === 'monitor-url-input' ||
        active?.id === 'monitors-add-cancel' ||
        active?.id === 'monitors-add-save'
      ) {
        return;
      }
    }
    let next = -1;
    const forward =
      e.key === 'ArrowRight' ||
      e.key === 'l' ||
      e.key === 'ArrowDown' ||
      e.key === 'j';
    const back =
      e.key === 'ArrowLeft' ||
      e.key === 'h' ||
      e.key === 'ArrowUp' ||
      e.key === 'k';
    if (forward) {
      if (
        active?.id === 'monitor-url-input' &&
        !monitorUrlInputAtMoveBoundary(active, 1)
      ) {
        return;
      }
      next = Math.min(idx + 1, items.length - 1);
    } else if (back) {
      if (
        active?.id === 'monitor-url-input' &&
        !monitorUrlInputAtMoveBoundary(active, -1)
      ) {
        return;
      }
      next = Math.max(idx - 1, 0);
    } else if (e.key === 'Home') {
      next = 0;
    } else if (e.key === 'End') {
      next = items.length - 1;
    } else {
      return;
    }
    e.preventDefault();
    e.stopPropagation();
    if (next === idx) return;
    refreshMonitorAddFormToolbarRovingTabindex(form, items[next]);
    items[next].focus();
    if (
      items[next]?.id === 'monitor-url-input' &&
      typeof items[next].setSelectionRange === 'function'
    ) {
      const len = (items[next].value || '').length;
      items[next].setSelectionRange(len, len);
    }
  });
}

function getMonitorDetailActionItems(row) {
  const wrap =
    row || document.querySelector('.monitor-detail-actions');
  if (!wrap) return [];
  const items = [];
  const check = wrap.querySelector('.monitor-detail-check');
  const remove = wrap.querySelector('.monitor-detail-remove');
  if (check && wrap.contains(check) && !check.hidden) items.push(check);
  if (remove && wrap.contains(remove) && !remove.hidden) items.push(remove);
  return items.filter((el) => {
    if (!el || el.hidden) return false;
    return el.getClientRects().length > 0 || wrap.contains(el);
  });
}

function refreshMonitorDetailActionsRovingTabindex(row, preferred) {
  const items = getMonitorDetailActionItems(row);
  if (!items.length) return;
  const focused = items.find((el) => el === document.activeElement);
  const current =
    (preferred && items.includes(preferred) && preferred) ||
    focused ||
    items.find((el) => el.tabIndex === 0) ||
    items[0];
  for (const el of items) {
    el.tabIndex = el === current ? 0 : -1;
  }
}

function ensureMonitorDetailActionsKbHint(row) {
  const wrap = row || document.querySelector('.monitor-detail-actions');
  if (!wrap) return;
  let hint = wrap.querySelector('.monitor-detail-toolbar-kb-hint');
  if (!hint) {
    hint = document.createElement('div');
    hint.className = 'monitor-detail-toolbar-kb-hint';
    hint.setAttribute('aria-hidden', 'true');
    wrap.appendChild(hint);
  }
  const items = getMonitorDetailActionItems(wrap);
  hint.hidden = items.length < 2;
  hint.textContent =
    '← → / h l · Home/End move · Enter / Space on buttons';
}

/**
 * Monitors detail action toolbar keyboard — focus Check now · Remove, then
 * ←→ / h l / Home/End (Disk Cleanup action toolbar parity).
 */
function ensureMonitorDetailActionsKeyboard(row) {
  const wrap = row || document.querySelector('.monitor-detail-actions');
  if (!wrap) return;
  ensureMonitorDetailActionsKbHint(wrap);
  refreshMonitorDetailActionsRovingTabindex(wrap);
  if (wrap.dataset.monitorDetailKbWired === '1') return;
  wrap.dataset.monitorDetailKbWired = '1';
  if (!wrap.getAttribute('role')) wrap.setAttribute('role', 'toolbar');
  if (!wrap.getAttribute('aria-label')) {
    wrap.setAttribute('aria-label', 'Monitor actions');
  }
  wrap.addEventListener('focusin', (e) => {
    const items = getMonitorDetailActionItems(wrap);
    if (items.includes(e.target)) {
      refreshMonitorDetailActionsRovingTabindex(wrap, e.target);
      ensureMonitorDetailActionsKbHint(wrap);
    }
  });
  wrap.addEventListener('keydown', (e) => {
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    const items = getMonitorDetailActionItems(wrap);
    if (!items.length) return;
    const idx = items.indexOf(document.activeElement);
    if (idx < 0) return;
    if (e.key === 'Enter' || e.key === ' ') return;
    let next = -1;
    if (
      e.key === 'ArrowRight' ||
      e.key === 'l' ||
      e.key === 'ArrowDown' ||
      e.key === 'j'
    ) {
      next = Math.min(idx + 1, items.length - 1);
    } else if (
      e.key === 'ArrowLeft' ||
      e.key === 'h' ||
      e.key === 'ArrowUp' ||
      e.key === 'k'
    ) {
      next = Math.max(idx - 1, 0);
    } else if (e.key === 'Home') {
      next = 0;
    } else if (e.key === 'End') {
      next = items.length - 1;
    } else {
      return;
    }
    e.preventDefault();
    e.stopPropagation();
    if (next === idx) return;
    refreshMonitorDetailActionsRovingTabindex(wrap, items[next]);
    items[next].focus();
  });
}

function fillMonitorDetail(detail, monitorId, monitorUrl, status) {
  const prevUrlFlash = captureMonitorUrlFlash(
    detail.querySelector('.monitor-detail-url')
  );
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

  {
    const row = document.createElement('div');
    row.className = 'monitor-detail-row';
    const k = document.createElement('span');
    k.className = 'monitor-detail-k';
    k.textContent = 'URL';
    const v = document.createElement('button');
    v.type = 'button';
    v.className = 'monitor-detail-v monitor-detail-url';
    v.textContent = monitorUrl;
    v.title = 'Click to copy URL';
    v.setAttribute('aria-label', `Copy URL ${monitorUrl}`);
    wireMonitorUrlCopy(v, monitorUrl, prevUrlFlash);
    row.appendChild(k);
    row.appendChild(v);
    detail.appendChild(row);
  }
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
  checkBtn.dataset.idleLabel = 'Check now';
  const row = detail.closest('.monitor-item');
  const checking = row?.dataset?.checking === '1';
  const removing = row?.dataset?.removing === '1';
  if (checking) {
    checkBtn.disabled = true;
    checkBtn.textContent = 'Checking…';
  } else {
    checkBtn.textContent = 'Check now';
    checkBtn.disabled = !!removing;
  }
  checkBtn.addEventListener('click', (e) => {
    e.preventDefault();
    e.stopPropagation();
    void forceCheckMonitorNow(monitorId, detail.closest('.monitor-item'));
  });
  actions.appendChild(checkBtn);

  const removeBtn = document.createElement('button');
  removeBtn.type = 'button';
  removeBtn.className = 'btn-secondary monitor-detail-remove';
  removeBtn.dataset.idleLabel = 'Remove';
  removeBtn.setAttribute('aria-label', `Remove monitor ${monitorUrl || monitorId}`);
  removeBtn.title = 'Remove this monitor (Delete)';
  if (removing) {
    removeBtn.disabled = true;
    removeBtn.textContent = 'Removing…';
  } else {
    removeBtn.textContent = 'Remove';
    removeBtn.disabled = !!checking;
  }
  removeBtn.addEventListener('click', (e) => {
    e.preventDefault();
    e.stopPropagation();
    void removeMonitorFromListRow(detail.closest('.monitor-item'));
  });
  actions.appendChild(removeBtn);
  ensureMonitorDetailActionsKeyboard(actions);
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
              id: monitorId,
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

    upLatencyHints.sort((a, b) => b.ms - a.ms);
    const anyDown = downCount > 0;
    const allUp = checkedCount > 0 && downCount === 0 && checkedCount === monitorIds.length;
    const slowest = upLatencyHints[0];
    // Amber slowest hint: relative (≥2 UP) or absolute (any UP ≥ 2000 ms — menu-bar Mon parity).
    const anySlowAbs = upLatencyHints.some((h) => h.ms >= 2000);
    const slowestHint =
      !anyDown && slowest && (upLatencyHints.length >= 2 || anySlowAbs)
        ? slowest.id || null
        : null;
    window.__monitorsSlowestId = slowestHint;

    if (downHints.length > 0) {
      const shown = downHints.slice(0, 2);
      const more = downHints.length > 2 ? ` +${downHints.length - 2}` : '';
      summaryText.textContent =
        `${upCount} / ${monitorIds.length} up · DOWN: ${shown.join(', ')}${more}`;
      summaryText.title = downHints.join('; ');
    } else {
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
    applyMonitorsSummaryState({ anyDown, allUp, empty: false, slowestId: slowestHint });
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

    if (monitorIds.length === 0) {
      ensureMonitorsListEmptyState(monitorsList, true);
      updateMonitorsIconStatus({ anyDown: false, allUp: false, upCount: 0, totalCount: 0 });
      updateMonitorsHeight();
      return;
    }
    ensureMonitorsListEmptyState(monitorsList, false);
    
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
    applyMonitorsListFilter();
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
  
  // Calculate height needed: summary + filter chips + each visible monitor item
  const monitorItems = visibleMonitorItems(monitorsList);
  const emptyEl =
    monitorsList.querySelector('.monitors-list-empty') ||
    monitorsList.querySelector('.monitors-filter-miss');
  const chips = document.getElementById('monitors-filter-chips');
  const itemHeight = 52; // row + down-meta / error lines
  const summaryHeight = 40;
  const chipsHeight = chips && !chips.hidden ? 36 : 0;
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
  
  const emptyHeight = emptyEl ? 110 : 0;
  const totalHeight =
    summaryHeight +
    chipsHeight +
    (monitorItems.length > 0
      ? listMargin + monitorItems.length * itemHeight + openDetailExtra
      : emptyHeight
        ? listMargin + emptyHeight
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
  const items = visibleMonitorItems(monitorsList);
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
  // Clear selection on hidden rows so filter does not leave a ghost highlight.
  monitorsList.querySelectorAll('.monitor-item').forEach((el) => {
    if (el.style.display === 'none') {
      el.classList.remove('is-selected');
      el.setAttribute('tabindex', '-1');
    }
  });
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
    'All · Pinned filters · click row for details · click name / c copies · focus list then ↑↓ / j k / Home / End · PgUp/PgDn · Enter / d opens · P pin/unpin · Esc closes/clears';
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
    'All · Up · Down filters · click row for details · ↑↓ / j k · PgUp/PgDn · Enter check now · c copy URL · d details · Delete removes · Esc closes/clears';
}

function wireMonitorsListKeyboard() {
  const monitorsList = document.getElementById('monitors-list');
  if (!monitorsList || monitorsList.dataset.keyboardNav === '1') return;
  monitorsList.dataset.keyboardNav = '1';
  monitorsList.setAttribute('role', 'listbox');
  monitorsList.setAttribute('aria-label', 'External monitors');
  if (!monitorsList.hasAttribute('tabindex')) {
    monitorsList.setAttribute('tabindex', '0');
  }

  monitorsList.addEventListener('click', (e) => {
    const item = e.target && e.target.closest && e.target.closest('.monitor-item');
    if (!item || !monitorsList.contains(item)) return;
    if (
      e.target.closest &&
      (e.target.closest('.monitor-detail-check') ||
        e.target.closest('.monitor-detail-remove') ||
        e.target.closest('.monitor-url') ||
        e.target.closest('.monitor-detail-url'))
    ) {
      return;
    }
    const id = item.getAttribute('data-monitor-id');
    syncMonitorsListTabOrder(monitorsList, id);
    item.focus();
    toggleMonitorDetail(item);
  });

  monitorsList.addEventListener('keydown', (e) => {
    const item = e.target && e.target.closest && e.target.closest('.monitor-item');
    if (!item || !monitorsList.contains(item)) {
      // First arrow/j from listbox chrome focuses first/last row (Disk Cleanup parity).
      if (e.target !== monitorsList) return;
      const items = visibleMonitorItems(monitorsList);
      if (!items.length) return;
      let next = -1;
      if (e.key === 'ArrowDown' || e.key === 'j' || e.key === 'Home') next = 0;
      else if (e.key === 'ArrowUp' || e.key === 'k' || e.key === 'End') next = items.length - 1;
      else return;
      e.preventDefault();
      const preferId = items[next].getAttribute('data-monitor-id');
      syncMonitorsListTabOrder(monitorsList, preferId);
      items[next].focus();
      if (typeof items[next].scrollIntoView === 'function') {
        items[next].scrollIntoView({ block: 'nearest' });
      }
      return;
    }
    if (item.style.display === 'none') return;
    const items = visibleMonitorItems(monitorsList);
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

    // c copies the monitor URL (click-to-copy parity; Top Processes / Agent Ops).
    if (
      (e.key === 'c' || e.key === 'C') &&
      !e.metaKey &&
      !e.ctrlKey &&
      !e.altKey
    ) {
      e.preventDefault();
      void copyMonitorUrlFromRow(item);
      return;
    }

    // Delete / Backspace removes the selected monitor (Settings Remove parity).
    if (
      (e.key === 'Delete' || e.key === 'Backspace') &&
      !e.metaKey &&
      !e.ctrlKey &&
      !e.altKey
    ) {
      e.preventDefault();
      void removeMonitorFromListRow(item);
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
  const setCheckBtnBusy = (row) => {
    const btn = row?.querySelector?.('.monitor-detail-check');
    if (!btn) return;
    if (btn._saveFlashTimer) {
      clearTimeout(btn._saveFlashTimer);
      btn._saveFlashTimer = null;
    }
    btn.classList.remove('is-just-saved');
    btn.dataset.idleLabel = btn.dataset.idleLabel || 'Check now';
    btn.disabled = true;
    btn.textContent = 'Checking…';
  };
  const item = itemEl || findRow();
  if (item?.dataset.checking === '1') return;
  if (item) {
    item.dataset.checking = '1';
    item.classList.add('is-checking');
    const latencyEl = item.querySelector('.monitor-latency');
    if (latencyEl) latencyEl.textContent = '…';
    setCheckBtnBusy(item);
  }
  let checkOk = false;
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
      checkOk = true;
    }
    await updateMonitorsSummary();
    const list = document.getElementById('monitors-list');
    sortMonitorsListByHealth(list);
    applyMonitorsListFilter();
    syncMonitorsListTabOrder(list, monitorId);
    updateMonitorsHeight();
  } catch (err) {
    console.error(`[Monitors] check_monitor failed for ${monitorId}:`, err);
  } finally {
    const row = item || findRow();
    if (row) {
      row.dataset.checking = '0';
      row.classList.remove('is-checking');
      const btn = row.querySelector('.monitor-detail-check');
      if (btn) {
        btn.disabled = false;
        const idle = btn.dataset.idleLabel || 'Check now';
        if (checkOk && typeof flashSaveButton === 'function') {
          btn.textContent = idle;
          flashSaveButton(btn, { savedLabel: 'Checked', durationMs: 1600 });
        } else {
          btn.classList.remove('is-just-saved');
          btn.textContent = idle;
        }
      }
    }
  }
}

function fillMonitorInfo(info, monitorUrl, status, monitorId) {
  const responseTimeText = status.response_time_ms ? `${status.response_time_ms}ms` : '--';
  const pending =
    !status.is_up &&
    (!status.response_time_ms || String(status.error || '').includes('Waiting'));
  const prevUrlFlash = captureMonitorUrlFlash(info.querySelector('.monitor-url'));
  info.replaceChildren();

  const primary = document.createElement('div');
  primary.className = 'monitor-info-primary';

  const urlEl = document.createElement('button');
  urlEl.type = 'button';
  urlEl.className = 'monitor-url';
  urlEl.textContent = monitorUrl;
  urlEl.title = 'Click to copy URL';
  urlEl.setAttribute('aria-label', `Copy URL ${monitorUrl}`);
  wireMonitorUrlCopy(urlEl, monitorUrl, prevUrlFlash);

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
    // Open pane → section-open highlight; closed → fade (unless alert).
    syncSectionIcon('icon-monitors', !monitorsCollapsed);
    if (anyDown) {
      // At least one monitor is down — red icon (visible even when section closed)
      monitorsIcon.classList.add('status-bad');
      monitorsIcon.title = `Monitors: ${upCount}/${totalCount} up — one or more down`;
    } else if (totalCount > 0) {
      monitorsIcon.title = monitorsCollapsed
        ? `Monitors: ${upCount}/${totalCount} up`
        : `Monitors: ${upCount}/${totalCount} up`;
    } else {
      monitorsIcon.title = monitorsCollapsed ? 'Monitors' : 'Hide Monitors';
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
  
  // Remove health classes; open/closed uses section-open (fade vs highlight).
  ollamaIcon.classList.remove('status-good', 'status-warning');
  syncSectionIcon('icon-ollama', !ollamaCollapsed);
  
  if (status === 'error' || status === 'unavailable') {
    // Ollama not installed/not running — warn even when section is closed
    ollamaIcon.classList.add('status-warning');
    console.log('[CPU] Ollama icon set to yellow (not available/not running)');
  } else if (status === true || status === 'connected') {
    console.log('[CPU] Ollama connected; section-open=', !ollamaCollapsed);
  } else {
    console.log('[CPU] Ollama icon default (unknown/checking)');
  }
  ollamaIcon.title = ollamaCollapsed ? 'AI Chat (Ollama)' : 'Hide AI Chat';
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
window.__setOllamaCollapsed = (v) => {
  ollamaCollapsed = !!v;
};

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
  ollamaCollapsed = getSectionCollapsed('ollama_collapsed');
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
  syncSectionIcon('icon-ollama', !ollamaCollapsed);
  if (window.Ollama && typeof window.Ollama.syncCollapsedGlance === 'function') {
    window.Ollama.syncCollapsedGlance();
  } else {
    // Ollama module may load after this init — retry once for collapsed glance.
    setTimeout(() => {
      if (window.Ollama && typeof window.Ollama.syncCollapsedGlance === 'function') {
        window.Ollama.syncCollapsedGlance();
      }
    }, 250);
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

    setIconPaneVisibility(section, content, ollamaCollapsed, divider);

    if (ollamaCollapsed) {
      if (chat) chat.style.display = 'none';
      hideModelDropdown();
    } else {
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
    setSectionCollapsed('ollama_collapsed', ollamaCollapsed);
    if (header._syncCollapseA11y) header._syncCollapseA11y();
    syncSectionIcon('icon-ollama', !ollamaCollapsed);
    if (window.Ollama && typeof window.Ollama.syncCollapsedGlance === 'function') {
      window.Ollama.syncCollapsedGlance();
    }
  };

  wireCollapsibleHeaderA11y(header, {
    contentId: 'ollama-content',
    getExpanded: () => !ollamaCollapsed,
    ignoreSelector: '#ollama-menu-btn, #ollama-menu, #ollama-connection-indicator, #ollama-model-text, #ollama-model-select, #ollama-collapsed-glance, #chat-model-glance, #chat-turn-glance, #chat-answer-glance',
    onToggle: () => {
      ollamaCollapsed = !ollamaCollapsed;
      applyOllamaCollapsed();
    },
  });

  header.addEventListener('click', (e) => {
    // Don't toggle if clicking on controls
    const menuBtn = document.getElementById('ollama-menu-btn');
    const menu = document.getElementById('ollama-menu');
    const collapsedGlance = document.getElementById('ollama-collapsed-glance');
    if (e.target === connectionIndicator || 
        e.target === modelText || 
        e.target === menuBtn ||
        e.target === collapsedGlance ||
        connectionIndicator?.contains(e.target) ||
        modelText?.contains(e.target) ||
        modelSelect?.contains(e.target) ||
        menuBtn?.contains(e.target) ||
        menu?.contains(e.target) ||
        collapsedGlance?.contains(e.target) ||
        e.target?.closest?.('#chat-model-glance, #chat-turn-glance, #chat-answer-glance')) {
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
  
  let ollamaSettingsSaveBusy = false;
  if (settingsSave) {
    settingsSave.addEventListener('click', () => {
      if (ollamaSettingsSaveBusy) return;
      if (settingsSave.classList.contains('is-just-saved')) return;
      if (!systemPromptTextarea) return;

      ollamaSettingsSaveBusy = true;
      settingsSave.disabled = true;
      settingsSave.classList.remove('is-just-saved');
      if (settingsSave._saveFlashOriginalLabel == null) {
        settingsSave._saveFlashOriginalLabel = settingsSave.textContent || 'Save';
      }
      settingsSave.textContent = 'Saving…';

      try {
        const prompt = systemPromptTextarea.value.trim();
        saveSystemPrompt(prompt || DEFAULT_SYSTEM_PROMPT);
        ollamaSettingsSaveBusy = false;
        settingsSave.disabled = false;
        if (typeof flashSaveButton === 'function') {
          flashSaveButton(settingsSave, { savedLabel: 'Saved', durationMs: 1600 });
        } else {
          settingsSave.classList.add('is-just-saved');
          settingsSave.textContent = 'Saved';
          setTimeout(() => {
            settingsSave.classList.remove('is-just-saved');
            settingsSave.textContent =
              settingsSave._saveFlashOriginalLabel || 'Save';
            settingsSave._saveFlashOriginalLabel = null;
          }, 1600);
        }
        console.log('[Ollama] System prompt saved');
        // Keep popover open so the Saved flash is visible (was silent close).
      } catch (err) {
        ollamaSettingsSaveBusy = false;
        settingsSave.disabled = false;
        settingsSave.textContent =
          settingsSave._saveFlashOriginalLabel || 'Save';
        settingsSave._saveFlashOriginalLabel = null;
        console.error('[Ollama] System prompt save failed:', err);
        alert(`Save failed: ${err?.message || err}`);
      }
    });
  }
  
  if (settingsReset) {
    settingsReset.addEventListener('click', () => {
      if (settingsReset.disabled || settingsReset.classList.contains('is-just-saved')) return;
      if (!systemPromptTextarea) return;
      const originalLabel =
        settingsReset._saveFlashOriginalLabel || settingsReset.textContent || 'Reset to Default';
      settingsReset._saveFlashOriginalLabel = originalLabel;
      settingsReset.disabled = true;
      settingsReset.classList.remove('is-just-saved');
      settingsReset.textContent = 'Resetting…';
      systemPromptTextarea.value = DEFAULT_SYSTEM_PROMPT;
      // Keep popover open so the Reset flash is visible (Save still required to persist).
      settingsReset.disabled = false;
      if (typeof flashSaveButton === 'function') {
        flashSaveButton(settingsReset, { savedLabel: 'Reset', durationMs: 1600 });
      } else {
        settingsReset.classList.add('is-just-saved');
        settingsReset.textContent = 'Reset';
        setTimeout(() => {
          settingsReset.classList.remove('is-just-saved');
          settingsReset.textContent = originalLabel;
          settingsReset._saveFlashOriginalLabel = null;
        }, 1600);
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
    const settingsHeader = settingsPopover.querySelector('.popover-header');
    if (settingsHeader) wireOllamaSettingsHeaderToolbarKeyboard(settingsHeader);
    const settingsContent = settingsPopover.querySelector('.popover-content');
    if (settingsContent) ensureOllamaSettingsToolbarKeyboard(settingsContent);
  }
  
  // Load saved system prompt into textarea if it exists
  if (systemPromptTextarea) {
    systemPromptTextarea.value = getSystemPrompt();
  }
}

function ollamaSettingsTextareaAtMoveBoundary(textarea, direction) {
  if (!textarea || textarea.tagName !== 'TEXTAREA') return true;
  if (direction > 0) {
    const len = (textarea.value || '').length;
    return textarea.selectionStart === len && textarea.selectionEnd === len;
  }
  return textarea.selectionStart === 0 && textarea.selectionEnd === 0;
}

/** Focusable Ollama settings body toolbar items (prompt · Reset · Save; close lives in header). */
function getOllamaSettingsToolbarItems(wrap) {
  const content =
    wrap || document.querySelector('#ollama-settings-popover .popover-content');
  if (!content) return [];
  const ids = [
    'ollama-system-prompt',
    'ollama-settings-reset',
    'ollama-settings-save',
  ];
  return ids
    .map((id) => document.getElementById(id))
    .filter((el) => {
      if (!el || !content.contains(el)) return false;
      if (el.hidden || el.disabled) return false;
      return el.getClientRects().length > 0 || content.contains(el);
    });
}

function refreshOllamaSettingsToolbarRovingTabindex(wrap, preferred) {
  const content =
    wrap || document.querySelector('#ollama-settings-popover .popover-content');
  const items = getOllamaSettingsToolbarItems(content);
  if (!items.length) return;
  const focused = items.find((el) => el === document.activeElement);
  const current =
    (preferred && items.includes(preferred) && preferred) ||
    focused ||
    items.find((el) => el.tabIndex === 0) ||
    items[0];
  for (const el of items) {
    el.tabIndex = el === current ? 0 : -1;
  }
}

function ensureOllamaSettingsToolbarKbHint(wrap) {
  const content =
    wrap || document.querySelector('#ollama-settings-popover .popover-content');
  if (!content) return;
  const actions = content.querySelector('.popover-actions');
  if (!actions) return;
  let hint = actions.querySelector('.ollama-settings-toolbar-kb-hint');
  if (!hint) {
    hint = document.createElement('div');
    hint.className = 'ollama-settings-toolbar-kb-hint';
    hint.setAttribute('aria-hidden', 'true');
    actions.appendChild(hint);
  }
  const items = getOllamaSettingsToolbarItems(content);
  hint.hidden = items.length < 2;
  hint.textContent =
    '← → / h l · Home/End move · arrows at prompt start/end · at start crosses to header Close · at end crosses to header Close';
}

/** Ollama settings header Close → system prompt. */
function tryChainOllamaSettingsHeaderToBody() {
  const content = document.querySelector('#ollama-settings-popover .popover-content');
  if (!content) return false;
  const items = getOllamaSettingsToolbarItems(content);
  if (!items.length) return false;
  refreshOllamaSettingsToolbarRovingTabindex(content, items[0]);
  items[0].focus();
  if (
    items[0]?.id === 'ollama-system-prompt' &&
    typeof items[0].setSelectionRange === 'function'
  ) {
    const len = (items[0].value || '').length;
    items[0].setSelectionRange(len, len);
  }
  return true;
}

/** Ollama settings header title ← Save. */
function tryChainOllamaSettingsHeaderToBodyLast() {
  const content = document.querySelector('#ollama-settings-popover .popover-content');
  if (!content) return false;
  const items = getOllamaSettingsToolbarItems(content);
  if (!items.length) return false;
  const target = items[items.length - 1];
  refreshOllamaSettingsToolbarRovingTabindex(content, target);
  target.focus();
  return true;
}

/** Ollama settings body item ←/→ header Close. */
function tryChainOllamaSettingsBodyToHeader() {
  const header = document
    .querySelector('#ollama-settings-popover .popover-header');
  if (!header || typeof window.getModalHeaderToolbarItems !== 'function') return false;
  const items = window.getModalHeaderToolbarItems(
    header,
    'ollama-settings-title',
    'ollama-settings-close'
  );
  if (!items.length) return false;
  const target = items[items.length - 1];
  if (typeof window.refreshModalHeaderRovingTabindex === 'function') {
    window.refreshModalHeaderRovingTabindex(
      header,
      'ollama-settings-title',
      'ollama-settings-close',
      target
    );
  }
  target.focus();
  return true;
}

function wireOllamaSettingsHeaderToolbarKeyboard(header) {
  if (!header || typeof window.wireModalHeaderToolbarKeyboard !== 'function') return;
  window.wireModalHeaderToolbarKeyboard(header, {
    titleId: 'ollama-settings-title',
    closeId: 'ollama-settings-close',
    ariaLabel: 'Ollama settings header',
    wireKey: 'ollamaSettingsHeaderToolbarKbWired',
    hintText:
      '← → / h l · Home/End move · Enter / Space on Close closes · at end crosses to system prompt · at start crosses to Save',
    chainForwardFromEnd: () => tryChainOllamaSettingsHeaderToBody(),
    chainBackFromStart: () => tryChainOllamaSettingsHeaderToBodyLast(),
  });
}

/**
 * Ollama settings toolbar keyboard — focus system prompt · Reset · Save,
 * then ←→ / h l / Home/End (changelog header↔body parity).
 */
function ensureOllamaSettingsToolbarKeyboard(wrap) {
  const content =
    wrap || document.querySelector('#ollama-settings-popover .popover-content');
  if (!content) return;
  ensureOllamaSettingsToolbarKbHint(content);
  refreshOllamaSettingsToolbarRovingTabindex(content);
  if (content.dataset.ollamaSettingsToolbarKbWired === '1') return;
  content.dataset.ollamaSettingsToolbarKbWired = '1';
  if (!content.getAttribute('role')) content.setAttribute('role', 'toolbar');
  if (!content.getAttribute('aria-label')) {
    content.setAttribute('aria-label', 'Ollama system prompt');
  }
  content.addEventListener('focusin', (e) => {
    const items = getOllamaSettingsToolbarItems(content);
    if (items.includes(e.target)) {
      refreshOllamaSettingsToolbarRovingTabindex(content, e.target);
      ensureOllamaSettingsToolbarKbHint(content);
    }
  });
  content.addEventListener('keydown', (e) => {
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    const items = getOllamaSettingsToolbarItems(content);
    if (!items.length) return;
    const idx = items.indexOf(document.activeElement);
    if (idx < 0) return;
    const active = items[idx];
    if (e.key === 'Enter' || e.key === ' ') {
      if (
        active?.id === 'ollama-system-prompt' ||
        active?.id === 'ollama-settings-reset' ||
        active?.id === 'ollama-settings-save'
      ) {
        return;
      }
    }
    let next = -1;
    const forward =
      e.key === 'ArrowRight' ||
      e.key === 'l' ||
      e.key === 'ArrowDown' ||
      e.key === 'j';
    const back =
      e.key === 'ArrowLeft' ||
      e.key === 'h' ||
      e.key === 'ArrowUp' ||
      e.key === 'k';
    if (forward) {
      if (
        active?.id === 'ollama-system-prompt' &&
        !ollamaSettingsTextareaAtMoveBoundary(active, 1)
      ) {
        return;
      }
      if (idx === items.length - 1) {
        if (tryChainOllamaSettingsBodyToHeader()) {
          e.preventDefault();
          e.stopPropagation();
        }
        return;
      }
      next = idx + 1;
    } else if (back) {
      if (
        active?.id === 'ollama-system-prompt' &&
        !ollamaSettingsTextareaAtMoveBoundary(active, -1)
      ) {
        return;
      }
      if (idx === 0) {
        if (tryChainOllamaSettingsBodyToHeader()) {
          e.preventDefault();
          e.stopPropagation();
        }
        return;
      }
      next = idx - 1;
    } else if (e.key === 'Home') {
      next = 0;
    } else if (e.key === 'End') {
      next = items.length - 1;
    } else {
      return;
    }
    e.preventDefault();
    e.stopPropagation();
    if (next === idx) return;
    refreshOllamaSettingsToolbarRovingTabindex(content, items[next]);
    items[next].focus();
    if (
      items[next]?.id === 'ollama-system-prompt' &&
      typeof items[next].setSelectionRange === 'function'
    ) {
      const len = (items[next].value || '').length;
      items[next].setSelectionRange(len, len);
    }
  });
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
    const settingsHeader = popover.querySelector('.popover-header');
    if (settingsHeader) wireOllamaSettingsHeaderToolbarKeyboard(settingsHeader);
    const settingsContent = popover.querySelector('.popover-content');
    if (settingsContent) {
      ensureOllamaSettingsToolbarKeyboard(settingsContent);
      refreshOllamaSettingsToolbarRovingTabindex(settingsContent);
      ensureOllamaSettingsToolbarKbHint(settingsContent);
    }
    // Focus close for consistent dialog pattern; textarea remains one Tab away
    requestAnimationFrame(() => {
      const closeBtn = document.getElementById('ollama-settings-close');
      if (closeBtn) {
        refreshOllamaSettingsToolbarRovingTabindex(settingsContent, closeBtn);
        closeBtn.focus();
      }
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

/** Soft tip above Details grid (Debug Log / Top Processes kb-hint parity). */
function ensureDetailsKbHint(grid, show) {
  if (!grid || !grid.parentNode) return;
  let hint = document.getElementById('details-kb-hint');
  if (!show) {
    hint?.remove();
    return;
  }
  if (!hint) {
    hint = document.createElement('div');
    hint.className = 'details-kb-hint';
    hint.id = 'details-kb-hint';
    grid.parentNode.insertBefore(hint, grid);
  }
  hint.textContent =
    '↑↓ / j k · Home/End select · Enter / c copies value · Esc clears · focus grid for first/last';
}

function detailsGridValues(grid) {
  if (!grid) return [];
  return Array.from(grid.querySelectorAll(':scope > .detail-value'));
}

function detailsValueLabel(el) {
  if (!el) return 'Detail';
  const prev = el.previousElementSibling;
  if (prev && prev.classList.contains('detail-label')) {
    const t = (prev.textContent || '').trim();
    if (t) return t;
  }
  return 'Detail';
}

/** Clipboard text for a Details cell (strip metric hints; prefer power spans). */
function detailsValueCopyText(el) {
  if (!el) return '';
  const power = el.querySelector('#cpu-power, #gpu-power');
  if (power) return metricValueCopyText(power);
  const clone = el.cloneNode(true);
  clone.querySelectorAll('.metric-hint').forEach((n) => n.remove());
  return metricValueCopyText(clone);
}

function flashDetailsValueCopied(el) {
  if (!el) return;
  if (el._detailsCopiedTimer) {
    clearTimeout(el._detailsCopiedTimer);
    el._detailsCopiedTimer = null;
  }
  el.classList.add('is-just-copied');
  const prevTitle = el.getAttribute('data-copy-title') || el.getAttribute('title') || '';
  el.title = 'Copied';
  el.setAttribute('aria-label', 'Copied');
  el._detailsCopiedTimer = setTimeout(() => {
    el.classList.remove('is-just-copied');
    el._detailsCopiedTimer = null;
    const label = detailsValueLabel(el);
    const title = prevTitle || `Click to copy ${label}`;
    el.title = title;
    el.setAttribute('data-copy-title', title);
    el.setAttribute('aria-label', `${label} — Enter or c copies`);
  }, 1600);
}

async function copyDetailsValue(el) {
  if (!el) return false;
  const value = detailsValueCopyText(el);
  if (!value) return false;
  const ok = await copyTextToClipboard(value);
  if (ok) flashDetailsValueCopied(el);
  return ok;
}

function syncDetailsValuesTabOrder(grid, preferEl) {
  const items = detailsGridValues(grid);
  ensureDetailsKbHint(grid, items.length > 0);
  if (!items.length) return;
  let activeIdx = preferEl ? items.indexOf(preferEl) : -1;
  if (activeIdx < 0) {
    activeIdx = items.findIndex((el) => el.classList.contains('is-selected'));
  }
  if (activeIdx < 0) {
    const focused = document.activeElement;
    activeIdx = focused && items.includes(focused) ? items.indexOf(focused) : -1;
  }
  if (activeIdx < 0) {
    items.forEach((el, i) => {
      el.classList.remove('is-selected');
      el.setAttribute('aria-selected', 'false');
      el.tabIndex = i === 0 ? 0 : -1;
    });
    return;
  }
  items.forEach((el, i) => {
    const on = i === activeIdx;
    el.classList.toggle('is-selected', on);
    el.setAttribute('aria-selected', on ? 'true' : 'false');
    el.tabIndex = on ? 0 : -1;
  });
}

function clearDetailsValueSelection(grid) {
  if (!grid) return;
  grid.querySelectorAll('.detail-value.is-selected').forEach((el) => {
    el.classList.remove('is-selected');
    el.setAttribute('aria-selected', 'false');
  });
  const items = detailsGridValues(grid);
  items.forEach((el, i) => {
    el.tabIndex = i === 0 ? 0 : -1;
  });
  if (document.activeElement && grid.contains(document.activeElement)) {
    document.activeElement.blur();
  }
}

/** Wire click-to-copy + j/k list nav on Details values (Debug Log / Monitors parity). */
function wireDetailsGridKeyboard(grid) {
  if (!grid || grid.dataset.keyboardNav === '1') return;
  grid.dataset.keyboardNav = '1';
  grid.setAttribute('role', 'listbox');
  grid.setAttribute('aria-label', 'System details');
  if (!grid.hasAttribute('tabindex')) grid.setAttribute('tabindex', '0');

  const items = detailsGridValues(grid);
  items.forEach((el) => {
    if (el.dataset.detailsCopyWired === '1') return;
    el.dataset.detailsCopyWired = '1';
    el.setAttribute('role', 'option');
    el.setAttribute('aria-selected', 'false');
    const label = detailsValueLabel(el);
    const title = `Click to copy ${label}`;
    el.title = title;
    el.setAttribute('data-copy-title', title);
    el.setAttribute('aria-label', `${label} — Enter or c copies`);
  });
  syncDetailsValuesTabOrder(grid, null);

  grid.addEventListener('click', (e) => {
    const cell = e.target && e.target.closest && e.target.closest('.detail-value');
    if (!cell || !grid.contains(cell) || cell.parentElement !== grid) return;
    e.preventDefault();
    e.stopPropagation();
    syncDetailsValuesTabOrder(grid, cell);
    cell.focus();
    void copyDetailsValue(cell);
  });

  grid.addEventListener('keydown', (e) => {
    const cell = e.target && e.target.closest && e.target.closest('.detail-value');
    if (!cell || !grid.contains(cell) || cell.parentElement !== grid) {
      // First arrow/j from listbox chrome focuses first/last value (Debug Log parity).
      if (e.target !== grid) return;
      const vals = detailsGridValues(grid);
      if (!vals.length) return;
      let next = -1;
      if (e.key === 'ArrowDown' || e.key === 'j' || e.key === 'Home') next = 0;
      else if (e.key === 'ArrowUp' || e.key === 'k' || e.key === 'End') next = vals.length - 1;
      else return;
      e.preventDefault();
      syncDetailsValuesTabOrder(grid, vals[next]);
      vals[next].focus();
      if (typeof vals[next].scrollIntoView === 'function') {
        vals[next].scrollIntoView({ block: 'nearest' });
      }
      return;
    }
    const vals = detailsGridValues(grid);
    const idx = vals.indexOf(cell);
    if (idx < 0) return;

    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      void copyDetailsValue(cell);
      return;
    }

    if (
      (e.key === 'c' || e.key === 'C') &&
      !e.metaKey &&
      !e.ctrlKey &&
      !e.altKey
    ) {
      e.preventDefault();
      void copyDetailsValue(cell);
      return;
    }

    if (e.key === 'Escape' || e.key === 'Esc') {
      if (!cell.classList.contains('is-selected') && document.activeElement !== cell) {
        return;
      }
      e.preventDefault();
      e.stopPropagation();
      clearDetailsValueSelection(grid);
      return;
    }

    let next = -1;
    const page = 5;
    if (e.key === 'ArrowDown' || e.key === 'j') next = Math.min(idx + 1, vals.length - 1);
    else if (e.key === 'ArrowUp' || e.key === 'k') next = Math.max(idx - 1, 0);
    else if (e.key === 'PageDown') next = Math.min(idx + page, vals.length - 1);
    else if (e.key === 'PageUp') next = Math.max(idx - page, 0);
    else if (e.key === 'Home') next = 0;
    else if (e.key === 'End') next = vals.length - 1;
    else return;
    e.preventDefault();
    if (next < 0 || next === idx) return;
    syncDetailsValuesTabOrder(grid, vals[next]);
    vals[next].focus();
    if (typeof vals[next].scrollIntoView === 'function') {
      vals[next].scrollIntoView({ block: 'nearest' });
    }
  });
}

function initDetailsGridKeyboard() {
  const grid =
    document.getElementById('details-content') ||
    document.querySelector('.details-grid');
  if (!grid) return;
  wireDetailsGridKeyboard(grid);
}

/** Details section collapsed (keep-header + glance; Top Processes / Debug Log parity). */
function isDetailsSectionCollapsed() {
  const section =
    document.getElementById("details-section") ||
    document.querySelector(
      ".apple-details, .arch-details, .swiss-details, .mat-details, .cpu-details, .details-section"
    );
  return !!(
    section &&
    (section.classList.contains("collapsed") || section.style.display === "none")
  );
}

function ensureDetailsCollapsedGlance() {
  const header = document.getElementById("details-header");
  if (!header) return null;
  let glance = document.getElementById("details-collapsed-glance");
  if (!glance) {
    glance = document.createElement("div");
    glance.id = "details-collapsed-glance";
    glance.className = "details-collapsed-glance";
    glance.hidden = true;
    glance.innerHTML = '<span id="details-collapsed-glance-text"></span>';
    header.insertAdjacentElement("afterend", glance);
    wireDetailsCollapsedGlanceClick(glance);
  }
  return glance;
}

function applyDetailsCollapsedGlanceState({ load1, ramPct, uptime, waiting }) {
  const glance = ensureDetailsCollapsedGlance();
  if (!glance) return;
  const text = document.getElementById("details-collapsed-glance-text");
  const collapsed = isDetailsSectionCollapsed();
  if (waiting) {
    glance.classList.remove("is-hot");
    if (collapsed) {
      glance.hidden = false;
      if (text) text.textContent = "Waiting · details";
      glance.setAttribute("role", "button");
      glance.tabIndex = 0;
      glance.title = "Show Details";
      glance.setAttribute("aria-label", "Details waiting — click to expand");
      return;
    }
    glance.hidden = true;
    return;
  }
  if (!collapsed) {
    glance.hidden = true;
    return;
  }
  glance.hidden = false;
  const loadStr =
    typeof load1 === "number" && Number.isFinite(load1) ? load1.toFixed(2) : "—";
  const ramStr =
    typeof ramPct === "number" && Number.isFinite(ramPct)
      ? `${Math.round(ramPct)}%`
      : "—";
  const upStr = uptime && String(uptime).trim() ? String(uptime).trim() : "—";
  if (text) text.textContent = `Load · ${loadStr} · RAM · ${ramStr} · Up · ${upStr}`;
  const hot =
    (typeof load1 === "number" && load1 >= 4) ||
    (typeof ramPct === "number" && ramPct >= 85);
  glance.classList.toggle("is-hot", hot);
  glance.setAttribute("role", "button");
  glance.tabIndex = 0;
  glance.title = "Show Details";
  glance.setAttribute(
    "aria-label",
    `Details Load ${loadStr}, RAM ${ramStr}, uptime ${upStr} — click to expand`
  );
}

function refreshDetailsCollapsedGlanceFromDom() {
  if (!isDetailsSectionCollapsed()) {
    const glance = document.getElementById("details-collapsed-glance");
    if (glance) glance.hidden = true;
    return;
  }
  const loadEl = document.getElementById("load-1");
  const ramEl = document.getElementById("ram-percent-value");
  const upEl = document.getElementById("uptime-value");
  const loadRaw = loadEl?.textContent?.trim();
  const ramRaw = ramEl?.textContent?.trim();
  const upRaw = upEl?.textContent?.trim();
  const load1 = loadRaw != null && loadRaw !== "" ? Number(loadRaw) : NaN;
  const ramPct =
    ramRaw != null && ramRaw.endsWith("%")
      ? Number(ramRaw.replace("%", ""))
      : NaN;
  const waiting =
    (!Number.isFinite(load1) && (!upRaw || upRaw === "—" || upRaw === "0h")) ||
    (loadRaw === "0.0" && (!ramRaw || ramRaw === "—") && (!upRaw || upRaw === "0h"));
  // Prefer live metrics when any row has real data.
  if (
    waiting &&
    !Number.isFinite(load1) &&
    !Number.isFinite(ramPct) &&
    (!upRaw || upRaw === "—" || upRaw === "0h")
  ) {
    applyDetailsCollapsedGlanceState({
      load1: null,
      ramPct: null,
      uptime: null,
      waiting: true,
    });
    return;
  }
  applyDetailsCollapsedGlanceState({
    load1: Number.isFinite(load1) ? load1 : null,
    ramPct: Number.isFinite(ramPct) ? ramPct : null,
    uptime: upRaw && upRaw !== "—" ? upRaw : null,
    waiting: false,
  });
}

function wireDetailsCollapsedGlanceClick(glance) {
  if (!glance || glance.dataset.detailsGlanceWired === "1") return;
  glance.dataset.detailsGlanceWired = "1";
  const activate = () => {
    if (typeof window.showCpuDetailsSection === "function") {
      window.showCpuDetailsSection();
    } else if (typeof window.showDetailsProcessesSections === "function") {
      window.showDetailsProcessesSections();
    }
  };
  glance.addEventListener("click", (e) => {
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
  glance.addEventListener("keydown", (e) => {
    if (e.key !== "Enter" && e.key !== " ") return;
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
}

function initCollapsibleSections() {
  // Get collapsed state from localStorage (default to true - hidden)
  const sectionsCollapsed = getSectionCollapsed('details_processes_collapsed');
  
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
  
  function syncDetailsCollapseA11y() {
    if (!detailsHeader) return;
    const collapsed = isDetailsSectionCollapsed();
    detailsHeader.setAttribute("aria-expanded", String(!collapsed));
    detailsHeader.setAttribute(
      "aria-label",
      collapsed ? "Show Details" : "Hide Details"
    );
  }

  // Collapse Details: fully hidden (icon-line / CPU card toggles).
  function hideDetails() {
    if (detailsSection) {
      detailsSection.classList.add("collapsed");
      detailsSection.style.display = "none";
      detailsSection.setAttribute("aria-hidden", "true");
    }
    if (detailsDivider && isDivider(detailsDivider)) {
      detailsDivider.style.display = "none";
    }
    syncDetailsCollapseA11y();
    refreshDetailsCollapsedGlanceFromDom();
  }
  
  // Show Details section (full grid)
  function showDetails() {
    if (detailsSection) {
      detailsSection.classList.remove("collapsed");
      detailsSection.style.display = "";
      detailsSection.removeAttribute("aria-hidden");
    }
    if (detailsDivider && isDivider(detailsDivider)) {
      detailsDivider.style.display = "";
    }
    syncDetailsCollapseA11y();
    const glance = document.getElementById("details-collapsed-glance");
    if (glance) glance.hidden = true;
  }
  
  function syncProcessesCollapseA11y() {
    if (!processesHeader) return;
    const collapsed = isProcessesSectionCollapsed();
    processesHeader.setAttribute('aria-expanded', String(!collapsed));
    processesHeader.setAttribute(
      'aria-label',
      collapsed ? 'Show Top Processes' : 'Hide Top Processes list'
    );
  }

  // Collapse Processes: keep header + Top CPU/GPU/RAM glances (Debug Log / Perplexity parity).
  function hideProcesses() {
    if (processesSection) {
      processesSection.classList.add('collapsed');
      processesSection.style.display = 'none';
      processesSection.setAttribute('aria-hidden', 'true');
    }
    if (processesDivider && isDivider(processesDivider)) {
      processesDivider.style.display = 'none';
    }
    syncProcessesCollapseA11y();
    // Refresh quiet Waiting glance when list is empty while collapsed.
    const topGlance = document.getElementById('processes-top-glance');
    if (topGlance && !window.__processesTopPid) {
      applyProcessesTopGlanceState({
        topPid: null,
        topName: null,
        topCpu: null,
        waiting: true,
      });
    }
  }
  
  // Show Processes section (full list)
  function showProcesses() {
    if (processesSection) {
      processesSection.classList.remove('collapsed');
      processesSection.style.display = '';
      processesSection.removeAttribute('aria-hidden');
    }
    if (processesDivider && isDivider(processesDivider)) {
      processesDivider.style.display = '';
    }
    syncProcessesCollapseA11y();
    const topGlance = document.getElementById('processes-top-glance');
    if (topGlance && !window.__processesTopPid) {
      applyProcessesTopGlanceState({
        topPid: null,
        topName: null,
        topCpu: null,
        waiting: true,
      });
    }
  }
  
  // Hide both sections (Details fully; Processes keep-header)
  function hideSections() {
    hideDetails();
    hideProcesses();
    setSectionCollapsed('details_processes_collapsed', true);
  }
  
  // Show both sections
  function showSections() {
    showDetails();
    showProcesses();
    setSectionCollapsed('details_processes_collapsed', false);
  }

  function syncUsageExpanded() {
    if (!usageCard) return;
    const hidden =
      isDetailsSectionCollapsed() || isProcessesSectionCollapsed();
    usageCard.setAttribute('aria-expanded', String(!hidden));
    usageCard.setAttribute(
      'aria-label',
      hidden ? 'Show Details and Processes' : 'Hide Details and Processes'
    );
  }

  window.hideDetailsProcessesSections = hideSections;
  window.showDetailsProcessesSections = showSections;
  window.showCpuDetailsSection = () => {
    showDetails();
    syncUsageExpanded();
  };
  
  // Apply initial state (hidden by default)
  if (sectionsCollapsed) {
    hideSections();
  } else {
    showSections();
  }
  
  // Details header click - toggle keep-header collapse (grid hides; glance stays)
  if (detailsHeader) {
    detailsHeader.setAttribute('role', 'button');
    if (!detailsHeader.hasAttribute('tabindex')) detailsHeader.setAttribute('tabindex', '0');
    syncDetailsCollapseA11y();
    const toggleDetailsAction = (e) => {
      if (
        e.target &&
        e.target.closest &&
        e.target.closest('#details-collapsed-glance')
      ) {
        return;
      }
      e.stopPropagation();
      if (isDetailsSectionCollapsed()) {
        showDetails();
      } else {
        hideDetails();
      }
      syncUsageExpanded();
    };
    detailsHeader.addEventListener('click', toggleDetailsAction);
    detailsHeader.addEventListener('keydown', (e) => {
      if (e.key !== 'Enter' && e.key !== ' ') return;
      e.preventDefault();
      toggleDetailsAction(e);
    });
  }
  
  // Processes header click - toggle keep-header collapse (list hides; glances stay)
  if (processesHeader) {
    processesHeader.setAttribute('role', 'button');
    if (!processesHeader.hasAttribute('tabindex')) processesHeader.setAttribute('tabindex', '0');
    syncProcessesCollapseA11y();
    const toggleProcessesAction = (e) => {
      if (
        e.target &&
        e.target.closest &&
        e.target.closest(
          '#processes-top-glance, #processes-top-gpu-glance, #processes-top-ram-glance'
        )
      ) {
        return;
      }
      e.stopPropagation();
      if (isProcessesSectionCollapsed()) {
        showProcesses();
      } else {
        hideProcesses();
      }
      syncUsageExpanded();
    };
    processesHeader.addEventListener('click', toggleProcessesAction);
    processesHeader.addEventListener('keydown', (e) => {
      if (e.key !== 'Enter' && e.key !== ' ') return;
      e.preventDefault();
      toggleProcessesAction(e);
    });
  }
  
  // Usage card click - toggle both sections (open/close)
  if (usageCard) {
    usageCard.setAttribute('role', 'button');
    usageCard.setAttribute('tabindex', '0');
    syncUsageExpanded();
    const toggleSections = (e) => {
      e.stopPropagation();
      const currentlyHidden =
        isDetailsSectionCollapsed() || isProcessesSectionCollapsed();
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

  ensureCpuStrip();
  ensureRamStrip();
  ensureGpuStrip();
  ensureTempStrip();
  ensureThermalStrip();
  ensureLowPowerStrip();
  ensureFreqStrip();
  ensureDiskStrip();
  ensureUptimeStrip();
  ensurePowerStripKeyboard();
  ensureRingGaugeKeyboard();
  ensureHistorySparklineKeyboard();
  ensureProcessesTopGlance();
  ensureDetailsCollapsedGlance();
  initDetailsGridKeyboard();
  if (isDetailsSectionCollapsed()) {
    refreshDetailsCollapsedGlanceFromDom();
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

/** Visible Perplexity result cards for keyboard nav (weather card skipped). */
function visiblePerplexityResultItems(resultsEl) {
  if (!resultsEl) return [];
  return Array.from(resultsEl.querySelectorAll('.perplexity-result-item')).filter((el) => {
    if (el.style.display === 'none') return false;
    return true;
  });
}

/** Soft tip above results (Monitors / AI Chat kb-hint parity). */
function ensurePerplexityResultsKbHint(resultsEl, show) {
  if (!resultsEl || !resultsEl.parentNode) return;
  let hint = document.getElementById('perplexity-kb-hint');
  if (!show) {
    hint?.remove();
    return;
  }
  if (!hint) {
    hint = document.createElement('div');
    hint.className = 'perplexity-kb-hint';
    hint.id = 'perplexity-kb-hint';
    resultsEl.parentNode.insertBefore(hint, resultsEl);
  }
  hint.textContent =
    'Focus results then ↑↓ / j k / Home / End · PgUp/PgDn · Enter opens · c copies URL · Esc clears';
}

function syncPerplexityResultsTabOrder(resultsEl, preferEl) {
  const items = visiblePerplexityResultItems(resultsEl);
  ensurePerplexityResultsKbHint(resultsEl, items.length > 0);
  if (!items.length) {
    resultsEl?.querySelectorAll('.perplexity-result-item.is-selected').forEach((el) => {
      el.classList.remove('is-selected');
      el.setAttribute('aria-selected', 'false');
    });
    return;
  }
  let activeIdx = preferEl ? items.indexOf(preferEl) : -1;
  if (activeIdx < 0) {
    activeIdx = items.findIndex((el) => el.classList.contains('is-selected'));
  }
  if (activeIdx < 0) {
    const focused = document.activeElement;
    activeIdx = focused && items.includes(focused) ? items.indexOf(focused) : 0;
  }
  items.forEach((el, i) => {
    const on = i === activeIdx;
    el.classList.toggle('is-selected', on);
    el.setAttribute('aria-selected', on ? 'true' : 'false');
    el.tabIndex = on ? 0 : -1;
  });
}

function clearPerplexityResultSelection(resultsEl) {
  if (!resultsEl) return;
  resultsEl.querySelectorAll('.perplexity-result-item.is-selected').forEach((el) => {
    el.classList.remove('is-selected');
    el.setAttribute('aria-selected', 'false');
  });
  const items = visiblePerplexityResultItems(resultsEl);
  items.forEach((el, i) => {
    el.tabIndex = i === 0 ? 0 : -1;
  });
  if (document.activeElement && resultsEl.contains(document.activeElement)) {
    document.activeElement.blur();
  }
}

function flashPerplexityResultCopied(el) {
  if (!el) return;
  if (el._perpCopiedTimer) {
    clearTimeout(el._perpCopiedTimer);
    el._perpCopiedTimer = null;
  }
  el.classList.add('is-just-copied');
  const prevTitle = el.getAttribute('title') || '';
  el.title = 'Copied';
  el.setAttribute('aria-label', 'Copied');
  el._perpCopiedTimer = setTimeout(() => {
    el.classList.remove('is-just-copied');
    el._perpCopiedTimer = null;
    if (prevTitle) el.title = prevTitle;
    else el.removeAttribute('title');
    const url = el.dataset.url || '';
    el.setAttribute(
      'aria-label',
      url ? `Search result — Enter opens · c copies URL` : 'Search result'
    );
  }, 1600);
}

async function copyPerplexityResultUrl(item) {
  if (!item) return false;
  const url = String(item.dataset.url || item.querySelector?.('a')?.href || '').trim();
  if (!url || url === '#') return false;
  const ok = await copyTextToClipboard(url);
  if (ok) flashPerplexityResultCopied(item);
  return ok;
}

function openPerplexityResultUrl(item) {
  if (!item) return;
  const url = String(item.dataset.url || item.querySelector?.('a')?.href || '').trim();
  if (!url || url === '#') return;
  try {
    window.open(url, '_blank', 'noopener,noreferrer');
  } catch (_) {
    const a = item.querySelector('a[href]');
    if (a) a.click();
  }
}

function decoratePerplexityResultItems(resultsEl) {
  if (!resultsEl) return;
  visiblePerplexityResultItems(resultsEl).forEach((el) => {
    el.setAttribute('role', 'option');
    if (!el.hasAttribute('tabindex')) el.tabIndex = -1;
    el.setAttribute('aria-selected', 'false');
    el.title = 'Enter opens · c copies URL · ↑↓ / j k to move · Esc clears';
    const url = el.dataset.url || el.querySelector?.('a')?.href || '';
    if (url && !el.dataset.url) el.dataset.url = url;
    el.setAttribute('aria-label', 'Search result — Enter opens · c copies URL');
  });
  syncPerplexityResultsTabOrder(resultsEl);
}

/** Wire j/k list nav + Enter open + c copy on Perplexity results (Monitors parity). */
function wirePerplexityResultsKeyboard(resultsEl) {
  if (!resultsEl || resultsEl.dataset.keyboardNav === '1') return;
  resultsEl.dataset.keyboardNav = '1';
  resultsEl.setAttribute('role', 'listbox');
  resultsEl.setAttribute('aria-label', 'Perplexity search results');
  if (!resultsEl.hasAttribute('tabindex')) {
    resultsEl.setAttribute('tabindex', '0');
  }

  resultsEl.addEventListener('click', (e) => {
    const item = e.target && e.target.closest && e.target.closest('.perplexity-result-item');
    if (!item || !resultsEl.contains(item)) return;
    syncPerplexityResultsTabOrder(resultsEl, item);
    // Keep native <a> navigation; still focus the card for keyboard follow-up.
    if (!e.target.closest || !e.target.closest('a')) {
      item.focus();
    }
  });

  resultsEl.addEventListener('keydown', (e) => {
    const item = e.target && e.target.closest && e.target.closest('.perplexity-result-item');
    if (!item || !resultsEl.contains(item)) {
      // First arrow/j from listbox chrome focuses first/last result (Monitors / Debug Log parity).
      if (e.target !== resultsEl) return;
      const items = visiblePerplexityResultItems(resultsEl);
      if (!items.length) return;
      let next = -1;
      if (e.key === 'ArrowDown' || e.key === 'j' || e.key === 'Home') next = 0;
      else if (e.key === 'ArrowUp' || e.key === 'k' || e.key === 'End') next = items.length - 1;
      else return;
      e.preventDefault();
      syncPerplexityResultsTabOrder(resultsEl, items[next]);
      items[next].focus();
      if (typeof items[next].scrollIntoView === 'function') {
        items[next].scrollIntoView({ block: 'nearest' });
      }
      return;
    }
    if (item.style.display === 'none') return;
    const items = visiblePerplexityResultItems(resultsEl);
    const idx = items.indexOf(item);
    if (idx < 0) return;

    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      openPerplexityResultUrl(item);
      return;
    }

    if (
      (e.key === 'c' || e.key === 'C') &&
      !e.metaKey &&
      !e.ctrlKey &&
      !e.altKey
    ) {
      e.preventDefault();
      void copyPerplexityResultUrl(item);
      return;
    }

    if (e.key === 'Escape' || e.key === 'Esc') {
      if (!item.classList.contains('is-selected') && document.activeElement !== item) {
        return;
      }
      e.preventDefault();
      e.stopPropagation();
      clearPerplexityResultSelection(resultsEl);
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
    syncPerplexityResultsTabOrder(resultsEl, items[next]);
    items[next].focus();
    if (typeof items[next].scrollIntoView === 'function') {
      items[next].scrollIntoView({ block: 'nearest' });
    }
  });
}

/** Focusable Perplexity search toolbar items (query input · Search). */
function getPerplexitySearchToolbarItems(container) {
  const row = container || document.querySelector('.perplexity-search-box');
  if (!row || row.hidden) return [];
  const items = [];
  const query = document.getElementById('perplexity-query');
  const searchBtn = document.getElementById('perplexity-search-btn');
  if (query && row.contains(query) && !query.hidden) items.push(query);
  if (searchBtn && row.contains(searchBtn) && !searchBtn.hidden) items.push(searchBtn);
  return items.filter((el) => {
    if (!el || el.hidden) return false;
    return el.getClientRects().length > 0 || row.contains(el);
  });
}

function perplexitySearchInputAtMoveBoundary(input, direction) {
  if (!input || input.tagName !== 'INPUT') return true;
  if (direction > 0) {
    const len = (input.value || '').length;
    return input.selectionStart === len && input.selectionEnd === len;
  }
  return input.selectionStart === 0 && input.selectionEnd === 0;
}

function refreshPerplexitySearchRovingTabindex(container, preferred) {
  const row = container || document.querySelector('.perplexity-search-box');
  const items = getPerplexitySearchToolbarItems(row);
  if (!items.length) return;
  const focused = items.find((el) => el === document.activeElement);
  const current =
    (preferred && items.includes(preferred) && preferred) ||
    focused ||
    items.find((el) => el.tabIndex === 0) ||
    items[0];
  for (const el of items) {
    el.tabIndex = el === current ? 0 : -1;
  }
}

function ensurePerplexitySearchKbHint(container) {
  const row = container || document.querySelector('.perplexity-search-box');
  if (!row) return;
  let hint = row.querySelector('.perplexity-search-kb-hint');
  if (!hint) {
    hint = document.createElement('div');
    hint.className = 'perplexity-search-kb-hint';
    hint.setAttribute('aria-hidden', 'true');
    row.appendChild(hint);
  }
  const items = getPerplexitySearchToolbarItems(row);
  hint.hidden = items.length < 2;
  hint.textContent =
    '← → / h l · Home/End move · Enter searches from input · Search on button';
}

/**
 * Perplexity search toolbar keyboard — focus query · Search, then ←→ / h l /
 * Home/End (composer parity). Input keeps normal typing; arrows move only at
 * text start/end. One Tab stop via roving tabindex.
 */
function wirePerplexitySearchToolbarKeyboard(row) {
  if (!row) return;
  ensurePerplexitySearchKbHint(row);
  refreshPerplexitySearchRovingTabindex(row);
  if (row.dataset.perplexitySearchKbWired === '1') return;
  row.dataset.perplexitySearchKbWired = '1';
  if (!row.getAttribute('role')) row.setAttribute('role', 'toolbar');
  if (!row.getAttribute('aria-label')) row.setAttribute('aria-label', 'Perplexity search');
  row.addEventListener('focusin', (e) => {
    const items = getPerplexitySearchToolbarItems(row);
    if (items.includes(e.target)) {
      refreshPerplexitySearchRovingTabindex(row, e.target);
      ensurePerplexitySearchKbHint(row);
    }
  });
  row.addEventListener('keydown', (e) => {
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    const items = getPerplexitySearchToolbarItems(row);
    if (!items.length) return;
    const idx = items.indexOf(document.activeElement);
    if (idx < 0) return;
    const active = items[idx];
    if (e.key === 'Enter' || e.key === ' ') {
      if (active?.id === 'perplexity-query' || active?.id === 'perplexity-search-btn') return;
    }
    let next = -1;
    const forward =
      e.key === 'ArrowRight' ||
      e.key === 'l' ||
      e.key === 'ArrowDown' ||
      e.key === 'j';
    const back =
      e.key === 'ArrowLeft' ||
      e.key === 'h' ||
      e.key === 'ArrowUp' ||
      e.key === 'k';
    if (forward) {
      if (active?.id === 'perplexity-query' && !perplexitySearchInputAtMoveBoundary(active, 1)) {
        return;
      }
      next = Math.min(idx + 1, items.length - 1);
    } else if (back) {
      if (active?.id === 'perplexity-query' && !perplexitySearchInputAtMoveBoundary(active, -1)) {
        return;
      }
      next = Math.max(idx - 1, 0);
    } else if (e.key === 'Home') {
      next = 0;
    } else if (e.key === 'End') {
      next = items.length - 1;
    } else {
      return;
    }
    e.preventDefault();
    e.stopPropagation();
    if (next === idx) return;
    refreshPerplexitySearchRovingTabindex(row, items[next]);
    items[next].focus();
    if (items[next]?.id === 'perplexity-query' && typeof items[next].select === 'function') {
      const len = (items[next].value || '').length;
      items[next].setSelectionRange(len, len);
    }
  });
}

/** Focusable Perplexity setup toolbar items (inline key · Save key). */
function getPerplexitySetupToolbarItems(container) {
  const row = container || document.querySelector('.perplexity-setup-row');
  if (!row) return [];
  const setup = document.getElementById('perplexity-setup');
  if (setup?.hidden) return [];
  const items = [];
  const keyInput = document.getElementById('perplexity-inline-key');
  const saveBtn = document.getElementById('perplexity-inline-save');
  if (keyInput && row.contains(keyInput) && !keyInput.hidden) items.push(keyInput);
  if (saveBtn && row.contains(saveBtn) && !saveBtn.hidden) items.push(saveBtn);
  return items.filter((el) => {
    if (!el || el.hidden) return false;
    return el.getClientRects().length > 0 || row.contains(el);
  });
}

function refreshPerplexitySetupRovingTabindex(container, preferred) {
  const row = container || document.querySelector('.perplexity-setup-row');
  const items = getPerplexitySetupToolbarItems(row);
  if (!items.length) return;
  const focused = items.find((el) => el === document.activeElement);
  const current =
    (preferred && items.includes(preferred) && preferred) ||
    focused ||
    items.find((el) => el.tabIndex === 0) ||
    items[0];
  for (const el of items) {
    el.tabIndex = el === current ? 0 : -1;
  }
}

function ensurePerplexitySetupKbHint(container) {
  const row = container || document.querySelector('.perplexity-setup-row');
  if (!row) return;
  let hint = row.querySelector('.perplexity-setup-kb-hint');
  if (!hint) {
    hint = document.createElement('div');
    hint.className = 'perplexity-setup-kb-hint';
    hint.setAttribute('aria-hidden', 'true');
    row.appendChild(hint);
  }
  const items = getPerplexitySetupToolbarItems(row);
  hint.hidden = items.length < 2;
  hint.textContent =
    '← → / h l · Home/End move · Enter saves from key field · Save key on button';
}

function wirePerplexitySetupToolbarKeyboard(row) {
  if (!row) return;
  ensurePerplexitySetupKbHint(row);
  refreshPerplexitySetupRovingTabindex(row);
  if (row.dataset.perplexitySetupKbWired === '1') return;
  row.dataset.perplexitySetupKbWired = '1';
  if (!row.getAttribute('role')) row.setAttribute('role', 'toolbar');
  if (!row.getAttribute('aria-label')) row.setAttribute('aria-label', 'Perplexity API key');
  row.addEventListener('focusin', (e) => {
    const items = getPerplexitySetupToolbarItems(row);
    if (items.includes(e.target)) {
      refreshPerplexitySetupRovingTabindex(row, e.target);
      ensurePerplexitySetupKbHint(row);
    }
  });
  row.addEventListener('keydown', (e) => {
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    const items = getPerplexitySetupToolbarItems(row);
    if (!items.length) return;
    const idx = items.indexOf(document.activeElement);
    if (idx < 0) return;
    const active = items[idx];
    if (e.key === 'Enter' || e.key === ' ') {
      if (active?.id === 'perplexity-inline-key' || active?.id === 'perplexity-inline-save') {
        return;
      }
    }
    let next = -1;
    const forward =
      e.key === 'ArrowRight' ||
      e.key === 'l' ||
      e.key === 'ArrowDown' ||
      e.key === 'j';
    const back =
      e.key === 'ArrowLeft' ||
      e.key === 'h' ||
      e.key === 'ArrowUp' ||
      e.key === 'k';
    if (forward) {
      if (
        active?.id === 'perplexity-inline-key' &&
        !perplexitySearchInputAtMoveBoundary(active, 1)
      ) {
        return;
      }
      next = Math.min(idx + 1, items.length - 1);
    } else if (back) {
      if (
        active?.id === 'perplexity-inline-key' &&
        !perplexitySearchInputAtMoveBoundary(active, -1)
      ) {
        return;
      }
      next = Math.max(idx - 1, 0);
    } else if (e.key === 'Home') {
      next = 0;
    } else if (e.key === 'End') {
      next = items.length - 1;
    } else {
      return;
    }
    e.preventDefault();
    e.stopPropagation();
    if (next === idx) return;
    refreshPerplexitySetupRovingTabindex(row, items[next]);
    items[next].focus();
    if (
      items[next]?.id === 'perplexity-inline-key' &&
      typeof items[next].setSelectionRange === 'function'
    ) {
      const len = (items[next].value || '').length;
      items[next].setSelectionRange(len, len);
    }
  });
}

function perplexitySettingsInputAtMoveBoundary(input, direction) {
  if (!input || input.tagName !== 'INPUT') return true;
  if (direction > 0) {
    const len = (input.value || '').length;
    return input.selectionStart === len && input.selectionEnd === len;
  }
  return input.selectionStart === 0 && input.selectionEnd === 0;
}

/** Focusable Settings Perplexity key toolbar items (key input · Save · Clear). */
function getPerplexitySettingsToolbarItems(wrap) {
  const container = wrap || document.getElementById('perplexity-setting');
  if (!container) return [];
  const ids = [
    'perplexity-api-key-input',
    'perplexity-save-key',
    'perplexity-clear-key',
  ];
  return ids
    .map((id) => document.getElementById(id))
    .filter((el) => {
      if (!el || !container.contains(el)) return false;
      if (el.hidden || el.disabled) return false;
      return el.getClientRects().length > 0 || container.contains(el);
    });
}

function refreshPerplexitySettingsToolbarRovingTabindex(wrap, preferred) {
  const container = wrap || document.getElementById('perplexity-setting');
  const items = getPerplexitySettingsToolbarItems(container);
  if (!items.length) return;
  const focused = items.find((el) => el === document.activeElement);
  const current =
    (preferred && items.includes(preferred) && preferred) ||
    focused ||
    items.find((el) => el.tabIndex === 0) ||
    items[0];
  for (const el of items) {
    el.tabIndex = el === current ? 0 : -1;
  }
}

function ensurePerplexitySettingsToolbarKbHint(wrap) {
  const container = wrap || document.getElementById('perplexity-setting');
  if (!container) return;
  const actions = container.querySelector('.perplexity-actions');
  if (!actions) return;
  let hint = actions.querySelector('.perplexity-settings-toolbar-kb-hint');
  if (!hint) {
    hint = document.createElement('div');
    hint.className = 'perplexity-settings-toolbar-kb-hint';
    hint.setAttribute('aria-hidden', 'true');
    actions.appendChild(hint);
  }
  const items = getPerplexitySettingsToolbarItems(container);
  hint.hidden = items.length < 2;
  hint.textContent =
    '← → / h l · Home/End move · Enter saves from key field · buttons keep activate';
}

/**
 * Settings Perplexity key toolbar keyboard — focus key input · Save · Clear,
 * then ←→ / h l / Home/End (Discord settings toolbar parity).
 */
function wirePerplexitySettingsToolbarKeyboard(wrap) {
  const container = wrap || document.getElementById('perplexity-setting');
  if (!container) return;
  ensurePerplexitySettingsToolbarKbHint(container);
  refreshPerplexitySettingsToolbarRovingTabindex(container);
  if (container.dataset.perplexitySettingsToolbarKbWired === '1') return;
  container.dataset.perplexitySettingsToolbarKbWired = '1';
  if (!container.getAttribute('role')) container.setAttribute('role', 'toolbar');
  if (!container.getAttribute('aria-label')) {
    container.setAttribute('aria-label', 'Perplexity API key');
  }
  container.addEventListener('focusin', (e) => {
    const items = getPerplexitySettingsToolbarItems(container);
    if (items.includes(e.target)) {
      refreshPerplexitySettingsToolbarRovingTabindex(container, e.target);
      ensurePerplexitySettingsToolbarKbHint(container);
    }
  });
  container.addEventListener('keydown', (e) => {
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    const items = getPerplexitySettingsToolbarItems(container);
    if (!items.length) return;
    const idx = items.indexOf(document.activeElement);
    if (idx < 0) return;
    const active = items[idx];
    if (e.key === 'Enter' || e.key === ' ') {
      if (
        active?.id === 'perplexity-api-key-input' ||
        active?.id === 'perplexity-save-key' ||
        active?.id === 'perplexity-clear-key'
      ) {
        return;
      }
    }
    let next = -1;
    const forward =
      e.key === 'ArrowRight' ||
      e.key === 'l' ||
      e.key === 'ArrowDown' ||
      e.key === 'j';
    const back =
      e.key === 'ArrowLeft' ||
      e.key === 'h' ||
      e.key === 'ArrowUp' ||
      e.key === 'k';
    if (forward) {
      if (
        active?.id === 'perplexity-api-key-input' &&
        !perplexitySettingsInputAtMoveBoundary(active, 1)
      ) {
        return;
      }
      next = Math.min(idx + 1, items.length - 1);
    } else if (back) {
      if (
        active?.id === 'perplexity-api-key-input' &&
        !perplexitySettingsInputAtMoveBoundary(active, -1)
      ) {
        return;
      }
      next = Math.max(idx - 1, 0);
    } else if (e.key === 'Home') {
      next = 0;
    } else if (e.key === 'End') {
      next = items.length - 1;
    } else {
      return;
    }
    e.preventDefault();
    e.stopPropagation();
    if (next === idx) return;
    refreshPerplexitySettingsToolbarRovingTabindex(container, items[next]);
    items[next].focus();
    if (
      items[next]?.id === 'perplexity-api-key-input' &&
      typeof items[next].setSelectionRange === 'function'
    ) {
      const len = (items[next].value || '').length;
      items[next].setSelectionRange(len, len);
    }
  });
}

function ensurePerplexitySearchToolbarKeyboard() {
  const searchBox = document.querySelector('.perplexity-search-box');
  if (searchBox) wirePerplexitySearchToolbarKeyboard(searchBox);
  const setupRow = document.querySelector('.perplexity-setup-row');
  if (setupRow) wirePerplexitySetupToolbarKeyboard(setupRow);
  wirePerplexitySettingsToolbarKeyboard();
}

/** @type {boolean} */
let perplexityConfigured = false;
/** @type {boolean} */
let perplexityCollapsed = true;
/** @type {{ query: string, count: number, weather?: boolean, error?: boolean } | null} */
let perplexityLastSearch = null;
/** @type {boolean} */
let perplexitySearchBusyForGlance = false;

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
    let inlineSaveBusy = false;
    const saveInline = async () => {
      if (inlineSaveBusy) return;
      if (inlineSave.classList.contains('is-just-saved')) return;
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
      inlineSaveBusy = true;
      inlineSave.disabled = true;
      inlineSave.classList.remove('is-just-saved');
      if (inlineSave._saveFlashOriginalLabel == null) {
        inlineSave._saveFlashOriginalLabel = inlineSave.textContent || 'Save key';
      }
      inlineSave.textContent = 'Saving…';
      try {
        await invoke('store_credential', {
          request: { account: PERPLEXITY_KEYCHAIN_ACCOUNT, password: key },
        });
        inlineKey.value = '';
        if (note) note.hidden = true;
        inlineSaveBusy = false;
        inlineSave.disabled = false;
        if (typeof flashSaveButton === 'function') {
          flashSaveButton(inlineSave, { savedLabel: 'Saved', durationMs: 1600 });
        }
        await refreshPerplexityStatus();
        if (document.getElementById('perplexity-query')) {
          document.getElementById('perplexity-query').focus();
        }
      } catch (e) {
        console.error('Perplexity save key:', e);
        inlineSaveBusy = false;
        inlineSave.disabled = false;
        inlineSave.textContent =
          inlineSave._saveFlashOriginalLabel || 'Save key';
        inlineSave._saveFlashOriginalLabel = null;
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
  ensurePerplexitySearchToolbarKeyboard();
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
  ensurePerplexitySearchToolbarKeyboard();
}

async function refreshPerplexityStatus() {
  const invoke = getInvoke();
  if (!invoke) {
    applyPerplexityLastGlanceState();
    return;
  }
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
  applyPerplexityLastGlanceState();
}

function truncatePerplexityGlancePreview(raw, maxLen) {
  const s = String(raw || '').replace(/\s+/g, ' ').trim();
  if (!s) return '';
  const n = typeof maxLen === 'number' && maxLen > 8 ? maxLen : 48;
  if (s.length <= n) return s;
  return s.slice(0, Math.max(1, n - 1)).trimEnd() + '…';
}

/** Expand Perplexity if collapsed (glance click). */
function ensurePerplexitySectionExpanded() {
  const content = document.getElementById('perplexity-content');
  const section = document.querySelector('.perplexity-section');
  const header = document.getElementById('perplexity-header');
  const divider = document.getElementById('perplexity-details-divider');
  if (!content) return;
  if (!perplexityCollapsed && !content.classList.contains('collapsed')) {
    return;
  }
  perplexityCollapsed = false;
  setSectionCollapsed('perplexity_collapsed', false);
  content.classList.remove('collapsed');
  if (section) section.classList.remove('collapsed');
  if (divider) divider.style.display = '';
  if (header && header._syncCollapseA11y) header._syncCollapseA11y();
  syncSectionIcon('icon-perplexity', true);
  updatePerplexitySetupVisibility();
  void refreshPerplexityStatus();
}

/** Last-search / key glance under Perplexity header (AI Chat / Debug Log parity). */
function ensurePerplexityLastGlance() {
  const header = document.getElementById('perplexity-header');
  if (!header) return null;
  let glance = document.getElementById('perplexity-last-glance');
  if (!glance) {
    glance = document.createElement('div');
    glance.id = 'perplexity-last-glance';
    glance.className = 'perplexity-last-glance';
    glance.hidden = true;
    glance.innerHTML = '<span id="perplexity-last-glance-text"></span>';
    header.insertAdjacentElement('afterend', glance);
    wirePerplexityLastGlanceClick(glance);
  }
  return glance;
}

function applyPerplexityLastGlanceState() {
  const glance = ensurePerplexityLastGlance();
  if (!glance) return;
  const text = document.getElementById('perplexity-last-glance-text');
  glance.classList.remove(
    'has-results',
    'has-error',
    'needs-key',
    'is-searching',
    'is-ready'
  );

  if (perplexitySearchBusyForGlance && perplexityLastSearch && perplexityLastSearch.query) {
    const preview = truncatePerplexityGlancePreview(perplexityLastSearch.query, 44);
    glance.hidden = false;
    glance.classList.add('is-searching');
    if (text) text.textContent = `Searching · ${preview}`;
    glance.setAttribute('role', 'button');
    glance.tabIndex = 0;
    glance.title = 'Show Perplexity Search';
    glance.setAttribute('aria-label', `Searching for ${preview}`);
    return;
  }

  if (perplexityLastSearch && perplexityLastSearch.query) {
    const preview = truncatePerplexityGlancePreview(perplexityLastSearch.query, 40);
    const last = perplexityLastSearch;
    glance.hidden = false;
    glance.setAttribute('role', 'button');
    glance.tabIndex = 0;
    if (last.error) {
      glance.classList.add('has-error');
      if (text) text.textContent = `Last · ${preview} · error`;
      glance.title = 'Show last search error';
      glance.setAttribute('aria-label', `Last Perplexity search failed: ${preview}`);
      return;
    }
    glance.classList.add('has-results');
    let outcome = 'no results';
    const n = Number(last.count) || 0;
    if (n > 0) outcome = `${n} result${n === 1 ? '' : 's'}`;
    else if (last.weather) outcome = 'weather';
    if (text) text.textContent = `Last · ${preview} · ${outcome}`;
    glance.title = 'Show last Perplexity results';
    glance.setAttribute(
      'aria-label',
      `Last Perplexity search: ${preview}, ${outcome}. Click to open`
    );
    return;
  }

  if (!perplexityConfigured) {
    glance.hidden = false;
    glance.classList.add('needs-key');
    if (text) text.textContent = 'Key · add API key';
    glance.setAttribute('role', 'button');
    glance.tabIndex = 0;
    glance.title = 'Add a Perplexity API key';
    glance.setAttribute('aria-label', 'Perplexity needs an API key — click to set up');
    return;
  }

  // Collapsed: always show a glance so keep-header is useful (Monitors parity).
  if (perplexityCollapsed) {
    glance.hidden = false;
    glance.classList.add('is-ready');
    if (text) text.textContent = 'Ready · search';
    glance.setAttribute('role', 'button');
    glance.tabIndex = 0;
    glance.title = 'Show Perplexity Search';
    glance.setAttribute('aria-label', 'Perplexity ready — click to expand and search');
    return;
  }

  glance.hidden = true;
}

function wirePerplexityLastGlanceClick(glance) {
  if (!glance || glance.dataset.perplexityGlanceWired === '1') return;
  glance.dataset.perplexityGlanceWired = '1';
  const activate = () => {
    ensurePerplexitySectionExpanded();
    if (!perplexityConfigured) {
      const inlineKey = document.getElementById('perplexity-inline-key');
      if (inlineKey && typeof inlineKey.focus === 'function') {
        try {
          inlineKey.focus();
        } catch (_) {
          /* ignore */
        }
      }
      return;
    }
    const resultsEl = document.getElementById('perplexity-results');
    const queryInput = document.getElementById('perplexity-query');
    if (resultsEl && perplexityLastSearch && !perplexityLastSearch.error) {
      if (typeof resultsEl.scrollIntoView === 'function') {
        resultsEl.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
      }
    }
    if (queryInput && typeof queryInput.focus === 'function') {
      try {
        queryInput.focus();
      } catch (_) {
        /* ignore */
      }
    }
  };
  glance.addEventListener('click', (e) => {
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
  glance.addEventListener('keydown', (e) => {
    if (e.key !== 'Enter' && e.key !== ' ') return;
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
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

  perplexityCollapsed = getSectionCollapsed('perplexity_collapsed');
  const applyPerplexityCollapsed = () => {
    setIconPaneVisibility(section, content, perplexityCollapsed, divider);
    if (header._syncCollapseA11y) header._syncCollapseA11y();
    applyPerplexityLastGlanceState();
    refreshPerplexityStatus();
    syncSectionIcon('icon-perplexity', !perplexityCollapsed);
  };
  applyPerplexityCollapsed();

  wireCollapsibleHeaderA11y(header, {
    contentId: 'perplexity-content',
    getExpanded: () => !perplexityCollapsed,
    ignoreSelector: '#perplexity-last-glance',
    onToggle: () => {
      perplexityCollapsed = !perplexityCollapsed;
      setSectionCollapsed('perplexity_collapsed', perplexityCollapsed);
      applyPerplexityCollapsed();
    },
  });

  header.addEventListener('click', (e) => {
    if (e.target.closest && e.target.closest('#perplexity-last-glance')) return;
    perplexityCollapsed = !perplexityCollapsed;
    setSectionCollapsed('perplexity_collapsed', perplexityCollapsed);
    applyPerplexityCollapsed();
  });

  if (searchBtn && queryInput && resultsEl) {
    let perplexitySearchInFlight = false;
    let perplexitySearchFlashTimer = null;
    if (searchBtn.dataset.idleLabel == null) {
      searchBtn.dataset.idleLabel = searchBtn.textContent || 'Search';
    }

    const setPerplexitySearchBusy = (busy) => {
      perplexitySearchInFlight = !!busy;
      perplexitySearchBusyForGlance = !!busy;
      if (perplexitySearchFlashTimer) {
        clearTimeout(perplexitySearchFlashTimer);
        perplexitySearchFlashTimer = null;
      }
      searchBtn.classList.remove('is-just-saved');
      if (busy) {
        searchBtn.disabled = true;
        searchBtn.textContent = 'Searching…';
        searchBtn.title = 'Search in progress';
      } else {
        searchBtn.disabled = false;
        searchBtn.textContent = searchBtn.dataset.idleLabel || 'Search';
        searchBtn.title = 'Search the web with Perplexity';
      }
      applyPerplexityLastGlanceState();
    };

    const flashPerplexitySearched = () => {
      if (perplexitySearchFlashTimer) {
        clearTimeout(perplexitySearchFlashTimer);
        perplexitySearchFlashTimer = null;
      }
      const idle = searchBtn.dataset.idleLabel || 'Search';
      searchBtn.disabled = false;
      searchBtn.classList.add('is-just-saved');
      searchBtn.textContent = 'Searched';
      searchBtn.title = 'Search complete';
      perplexitySearchFlashTimer = setTimeout(() => {
        searchBtn.classList.remove('is-just-saved');
        searchBtn.textContent = idle;
        searchBtn.title = 'Search the web with Perplexity';
        perplexitySearchFlashTimer = null;
      }, 1600);
    };

    const runPerplexitySearch = async () => {
      const query = queryInput.value.trim();
      if (!query) return;
      if (perplexitySearchInFlight) {
        console.log('[Perplexity] Search ignored — already in flight');
        return;
      }
      const invoke = getInvoke();
      if (!invoke) {
        resultsEl.innerHTML = '<div class="perplexity-empty" role="status">App not ready.</div>';
        ensurePerplexityResultsKbHint(resultsEl, false);
        return;
      }
      if (!perplexityConfigured) {
        await refreshPerplexityStatus();
        updatePerplexitySetupVisibility();
        applyPerplexityLastGlanceState();
        return;
      }
      perplexityLastSearch = { query: query, count: 0 };
      setPerplexitySearchBusy(true);
      resultsEl.innerHTML = '<div class="perplexity-empty" role="status">Searching…</div>';
      ensurePerplexityResultsKbHint(resultsEl, false);
      let searchOk = false;
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
          ensurePerplexityResultsKbHint(resultsEl, false);
          perplexityLastSearch = {
            query: query,
            count: 0,
            weather: !!weatherHtml,
          };
          searchOk = true;
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
          return '<article class="perplexity-result-item" role="option" tabindex="-1" aria-selected="false" data-url="' + url + '" title="Enter opens · c copies URL · ↑↓ / j k to move · Esc clears" aria-label="Search result — Enter opens · c copies URL">' +
            '<a href="' + url + '" target="_blank" rel="noopener noreferrer">' + title + '</a>' +
            (meta ? '<div class="perplexity-result-meta">' + esc(meta) + '</div>' : '') +
            snippetHtml +
            '</article>';
        }).join('');
        decoratePerplexityResultItems(resultsEl);
        perplexityLastSearch = {
          query: query,
          count: resp.results.length,
          weather: !!weatherHtml,
        };
        searchOk = true;
      } catch (err) {
        resultsEl.innerHTML = '<div class="perplexity-empty perplexity-empty-error" role="alert">Error: ' + String(err) + '</div>';
        ensurePerplexityResultsKbHint(resultsEl, false);
        perplexityLastSearch = { query: query, count: 0, error: true };
      } finally {
        setPerplexitySearchBusy(false);
        if (searchOk) flashPerplexitySearched();
      }
    };

    wirePerplexityResultsKeyboard(resultsEl);

    searchBtn.addEventListener('click', () => {
      void runPerplexitySearch();
    });
    queryInput.addEventListener('keydown', (e) => {
      if (e.key !== 'Enter' || e.shiftKey) return;
      e.preventDefault();
      if (perplexitySearchInFlight) return;
      void runPerplexitySearch();
    });
  }

  ensurePerplexitySearchToolbarKeyboard();

  // Settings: Save / Clear API key (busy-guard + flash; Discord token parity)
  const saveBtn = document.getElementById('perplexity-save-key');
  const clearBtn = document.getElementById('perplexity-clear-key');
  const keyInput = document.getElementById('perplexity-api-key-input');
  let perplexityKeyBusy = false;

  function setPerplexityKeyBusy(busy, which) {
    perplexityKeyBusy = !!busy;
    if (saveBtn) {
      saveBtn.disabled = !!busy;
      if (busy && which === 'save') {
        saveBtn.classList.remove('is-just-saved');
        if (saveBtn._saveFlashOriginalLabel == null) {
          saveBtn._saveFlashOriginalLabel = saveBtn.textContent || 'Save key';
        }
        saveBtn.textContent = 'Saving…';
      } else if (!busy && !saveBtn.classList.contains('is-just-saved')) {
        saveBtn.textContent = saveBtn._saveFlashOriginalLabel || 'Save key';
        saveBtn._saveFlashOriginalLabel = null;
      }
    }
    if (clearBtn) {
      clearBtn.disabled = !!busy;
      if (busy && which === 'clear') {
        clearBtn.classList.remove('is-just-saved');
        if (clearBtn._saveFlashOriginalLabel == null) {
          clearBtn._saveFlashOriginalLabel = clearBtn.textContent || 'Clear key';
        }
        clearBtn.textContent = 'Clearing…';
      } else if (!busy && !clearBtn.classList.contains('is-just-saved')) {
        clearBtn.textContent = clearBtn._saveFlashOriginalLabel || 'Clear key';
        clearBtn._saveFlashOriginalLabel = null;
      }
    }
  }

  function flashPerplexityKeyBtn(btn, savedLabel) {
    if (!btn) return;
    if (typeof flashSaveButton === 'function') {
      flashSaveButton(btn, { savedLabel, durationMs: 1600 });
      return;
    }
    const prev = btn._saveFlashOriginalLabel || btn.textContent;
    btn.classList.add('is-just-saved');
    btn.textContent = savedLabel;
    setTimeout(() => {
      btn.classList.remove('is-just-saved');
      btn.textContent = prev;
      btn._saveFlashOriginalLabel = null;
    }, 1600);
  }

  if (saveBtn && keyInput) {
    saveBtn.addEventListener('click', async () => {
      if (perplexityKeyBusy) return;
      if (saveBtn.classList.contains('is-just-saved')) return;
      const invoke = getInvoke();
      if (!invoke) return;
      const key = keyInput.value.trim();
      setPerplexityKeyBusy(true, 'save');
      try {
        await invoke('store_credential', {
          request: { account: PERPLEXITY_KEYCHAIN_ACCOUNT, password: key },
        });
        keyInput.value = '';
        setPerplexityKeyBusy(false);
        flashPerplexityKeyBtn(saveBtn, 'Saved');
        await refreshPerplexityStatus();
      } catch (e) {
        console.error('Perplexity save key:', e);
        setPerplexityKeyBusy(false);
        alert('Could not save Perplexity key: ' + String(e));
      }
    });
  }
  if (clearBtn) {
    clearBtn.addEventListener('click', async () => {
      if (perplexityKeyBusy) return;
      if (clearBtn.classList.contains('is-just-saved')) return;
      const invoke = getInvoke();
      if (!invoke) return;
      setPerplexityKeyBusy(true, 'clear');
      try {
        await invoke('delete_credential', { account: PERPLEXITY_KEYCHAIN_ACCOUNT });
        if (keyInput) keyInput.value = '';
        setPerplexityKeyBusy(false);
        flashPerplexityKeyBtn(clearBtn, 'Cleared');
        await refreshPerplexityStatus();
      } catch (e) {
        console.error('Perplexity clear key:', e);
        setPerplexityKeyBusy(false);
        alert('Could not clear Perplexity key: ' + String(e));
      }
    });
  }

  wirePerplexitySettingsToolbarKeyboard();

  refreshPerplexityStatus();
  window.Perplexity = { refreshStatus: refreshPerplexityStatus };
  window.ensurePerplexitySettingsToolbarKeyboard = wirePerplexitySettingsToolbarKeyboard;
}

let logsAutoRefreshTimer = null;
let logsGlancePollTimer = null;
let logsSectionCollapsed = true;
let logsViewerRaw = { prefix: '', body: '', hasContent: false };
let logsFilterMode = 'all'; // all | error | warn
let logsGlanceCounts = { error: 0, warn: 0 };
/** Fingerprint of last rendered viewer body (skip rebuild on unchanged auto-refresh). */
let logsViewerRenderKey = '';
/** Last selected log line text (restore across refresh when possible). */
let logsSelectedLineText = '';

function logsLineKind(line) {
  if (/\sERROR\s|\bERROR:|\bpanic\b/i.test(line)) return 'error';
  if (/\sWARN\s|\bWARN:/i.test(line)) return 'warn';
  return 'other';
}

function countLogsByKind(body) {
  let error = 0;
  let warn = 0;
  if (!body) return { error, warn };
  for (const line of body.split('\n')) {
    const kind = logsLineKind(line);
    if (kind === 'error') error += 1;
    else if (kind === 'warn') warn += 1;
  }
  return { error, warn };
}

function filterLogsBody(body, mode) {
  if (!body || mode === 'all') return body;
  const out = [];
  let keepCont = false;
  for (const line of body.split('\n')) {
    const isCont = /^\s/.test(line) && line.trim() !== '';
    const kind = logsLineKind(line);
    if (kind === mode) {
      out.push(line);
      keepCont = true;
    } else if (keepCont && isCont) {
      out.push(line);
    } else {
      keepCont = false;
    }
  }
  return out.join('\n');
}

/** Error/warn glance under Debug Log header (filter-chip parity; polls when collapsed). */
function ensureLogsErrorGlance() {
  const header = document.getElementById('logs-header');
  if (!header) return null;
  let glance = document.getElementById('logs-error-glance');
  if (!glance) {
    glance = document.createElement('div');
    glance.id = 'logs-error-glance';
    glance.className = 'logs-error-glance';
    glance.hidden = true;
    glance.innerHTML = '<span id="logs-error-glance-text"></span>';
    header.insertAdjacentElement('afterend', glance);
    wireLogsErrorGlanceClick(glance);
  }
  return glance;
}

function applyLogsGlanceState({ error, warn }) {
  const glance = ensureLogsErrorGlance();
  if (!glance) return;
  const text = document.getElementById('logs-error-glance-text');
  const err = Number(error) || 0;
  const wrn = Number(warn) || 0;
  logsGlanceCounts = { error: err, warn: wrn };
  glance.classList.remove('has-errors', 'has-warns-only', 'is-quiet');
  if (err <= 0 && wrn <= 0) {
    // Collapsed: always show a glance so keep-header is useful (Perplexity parity).
    if (logsSectionCollapsed) {
      glance.hidden = false;
      glance.classList.add('is-quiet');
      if (text) text.textContent = 'Quiet · clean';
      glance.setAttribute('role', 'button');
      glance.tabIndex = 0;
      glance.title = 'Show Debug Log';
      glance.setAttribute('aria-label', 'Debug Log quiet — click to expand');
      return;
    }
    glance.hidden = true;
    return;
  }
  glance.hidden = false;
  const parts = [];
  if (err > 0) parts.push(`${err} error${err === 1 ? '' : 's'}`);
  if (wrn > 0) parts.push(`${wrn} warn${wrn === 1 ? '' : 's'}`);
  if (text) text.textContent = parts.join(' · ');
  glance.classList.toggle('has-errors', err > 0);
  glance.classList.toggle('has-warns-only', err <= 0 && wrn > 0);
  glance.setAttribute('role', 'button');
  glance.tabIndex = 0;
  const filterHint = err > 0 ? 'errors' : 'warnings';
  glance.title = `Click to show ${filterHint} in Debug Log`;
  glance.setAttribute(
    'aria-label',
    `Debug Log tail has ${parts.join(' and ')} — click to filter`
  );
}

function wireLogsErrorGlanceClick(glance) {
  if (!glance || glance.dataset.logsGlanceWired === '1') return;
  glance.dataset.logsGlanceWired = '1';
  const activate = () => {
    ensureLogsSectionExpanded();
    const mode =
      logsGlanceCounts.error > 0
        ? 'error'
        : logsGlanceCounts.warn > 0
          ? 'warn'
          : 'all';
    setLogsFilterMode(mode);
  };
  glance.addEventListener('click', (e) => {
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
  glance.addEventListener('keydown', (e) => {
    if (e.key !== 'Enter' && e.key !== ' ') return;
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
}

/** Expand Debug Log if collapsed (glance click / error CTA). */
function ensureLogsSectionExpanded() {
  const content = document.getElementById('logs-content');
  const section = document.querySelector('.logs-section');
  const header = document.getElementById('logs-header');
  if (!content) return;
  if (!logsSectionCollapsed && !content.classList.contains('collapsed')) {
    return;
  }
  logsSectionCollapsed = false;
  setSectionCollapsed('logs_collapsed', false);
  content.classList.remove('collapsed');
  if (section) section.classList.remove('collapsed');
  const divider = document.getElementById('logs-details-divider');
  if (divider) divider.style.display = '';
  header?.setAttribute('aria-expanded', 'true');
  if (typeof header?._syncCollapseA11y === 'function') header._syncCollapseA11y();
  syncSectionIcon('icon-logs', true);
  refreshLogsViewer(true);
  const autoCb = document.getElementById('logs-autorefresh');
  if (autoCb && autoCb.checked) startLogsAutoRefresh();
}

async function pollLogsGlanceCounts() {
  const inv = getInvoke() || invoke;
  if (!inv || !document.getElementById('logs-header')) return;
  try {
    const tail = await inv('read_debug_log', { maxBytes: 65536 });
    const body = tail.content || '';
    const counts = countLogsByKind(body);
    applyLogsGlanceState(counts);
  } catch (_) {
    /* glance poll is best-effort */
  }
}

function startLogsGlancePoll() {
  stopLogsGlancePoll();
  ensureLogsErrorGlance();
  pollLogsGlanceCounts();
  logsGlancePollTimer = setInterval(pollLogsGlanceCounts, 60000);
}

function stopLogsGlancePoll() {
  if (logsGlancePollTimer) {
    clearInterval(logsGlancePollTimer);
    logsGlancePollTimer = null;
  }
}

/** Group Debug Log action controls for toolbar keyboard (filter chips stay separate). */
function ensureLogsToolbarActionsWrap(toolbar) {
  if (!toolbar) return null;
  let wrap = toolbar.querySelector('.logs-toolbar-actions');
  if (wrap) return wrap;
  wrap = document.createElement('div');
  wrap.className = 'logs-toolbar-actions';
  const refresh = document.getElementById('logs-refresh-btn');
  const open = document.getElementById('logs-open-btn');
  const autoLabel = toolbar.querySelector('label.logs-autorefresh');
  if (refresh && toolbar.contains(refresh)) wrap.appendChild(refresh);
  if (open && toolbar.contains(open)) wrap.appendChild(open);
  if (autoLabel && toolbar.contains(autoLabel)) wrap.appendChild(autoLabel);
  if (!wrap.childElementCount) return null;
  toolbar.insertBefore(wrap, toolbar.firstChild);
  return wrap;
}

/** Focusable Debug Log toolbar items (Refresh · Open · Auto-refresh). */
function getLogsToolbarActionItems(row) {
  const wrap =
    row ||
    document.querySelector('.logs-toolbar-actions') ||
    ensureLogsToolbarActionsWrap(
      document.querySelector('#logs-content .logs-toolbar') ||
        document.querySelector('.logs-toolbar')
    );
  if (!wrap) return [];
  const items = [];
  const refresh = document.getElementById('logs-refresh-btn');
  const open = document.getElementById('logs-open-btn');
  const auto = document.getElementById('logs-autorefresh');
  if (refresh && wrap.contains(refresh) && !refresh.hidden) items.push(refresh);
  if (open && wrap.contains(open) && !open.hidden) items.push(open);
  if (auto && wrap.contains(auto) && !auto.hidden) items.push(auto);
  return items.filter((el) => {
    if (!el || el.hidden) return false;
    return el.getClientRects().length > 0 || wrap.contains(el);
  });
}

function refreshLogsToolbarRovingTabindex(row, preferred) {
  const items = getLogsToolbarActionItems(row);
  if (!items.length) return;
  const focused = items.find((el) => el === document.activeElement);
  const current =
    (preferred && items.includes(preferred) && preferred) ||
    focused ||
    items.find((el) => el.tabIndex === 0) ||
    items[0];
  for (const el of items) {
    el.tabIndex = el === current ? 0 : -1;
  }
}

function ensureLogsToolbarKbHint(row) {
  const wrap =
    row ||
    document.querySelector('.logs-toolbar-actions') ||
    ensureLogsToolbarActionsWrap(
      document.querySelector('#logs-content .logs-toolbar') ||
        document.querySelector('.logs-toolbar')
    );
  if (!wrap) return;
  let hint = wrap.querySelector('.logs-toolbar-kb-hint');
  if (!hint) {
    hint = document.createElement('div');
    hint.className = 'logs-toolbar-kb-hint';
    hint.setAttribute('aria-hidden', 'true');
    wrap.appendChild(hint);
  }
  const items = getLogsToolbarActionItems(wrap);
  hint.hidden = items.length < 2;
  hint.textContent =
    '← → / h l · Home/End move · Space toggles auto-refresh · Enter / Space on buttons';
}

/**
 * Debug Log toolbar keyboard — focus Refresh · Open in editor · Auto-refresh,
 * then ←→ / h l / Home/End (refresh-row / filter-chip parity). Enter/Space keeps
 * native button activate; Space toggles the checkbox.
 */
function ensureLogsToolbarKeyboard() {
  const toolbar =
    document.querySelector('#logs-content .logs-toolbar') ||
    document.querySelector('.logs-toolbar');
  if (!toolbar) return;
  const wrap = ensureLogsToolbarActionsWrap(toolbar);
  if (!wrap) return;
  ensureLogsToolbarKbHint(wrap);
  refreshLogsToolbarRovingTabindex(wrap);
  if (wrap.dataset.logsToolbarKbWired === '1') return;
  wrap.dataset.logsToolbarKbWired = '1';
  if (!wrap.getAttribute('role')) wrap.setAttribute('role', 'toolbar');
  if (!wrap.getAttribute('aria-label')) {
    wrap.setAttribute('aria-label', 'Debug log controls');
  }
  wrap.addEventListener('focusin', (e) => {
    const items = getLogsToolbarActionItems(wrap);
    if (items.includes(e.target)) {
      refreshLogsToolbarRovingTabindex(wrap, e.target);
      ensureLogsToolbarKbHint(wrap);
    }
  });
  wrap.addEventListener('keydown', (e) => {
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    const items = getLogsToolbarActionItems(wrap);
    if (!items.length) return;
    const idx = items.indexOf(document.activeElement);
    if (idx < 0) return;
    const active = items[idx];
    if (e.key === 'Enter' || e.key === ' ') {
      if (
        active?.id === 'logs-refresh-btn' ||
        active?.id === 'logs-open-btn' ||
        active?.id === 'logs-autorefresh'
      ) {
        return;
      }
    }
    let next = -1;
    if (
      e.key === 'ArrowRight' ||
      e.key === 'l' ||
      e.key === 'ArrowDown' ||
      e.key === 'j'
    ) {
      next = Math.min(idx + 1, items.length - 1);
    } else if (
      e.key === 'ArrowLeft' ||
      e.key === 'h' ||
      e.key === 'ArrowUp' ||
      e.key === 'k'
    ) {
      next = Math.max(idx - 1, 0);
    } else if (e.key === 'Home') {
      next = 0;
    } else if (e.key === 'End') {
      next = items.length - 1;
    } else {
      return;
    }
    e.preventDefault();
    e.stopPropagation();
    if (next === idx) return;
    refreshLogsToolbarRovingTabindex(wrap, items[next]);
    items[next].focus();
  });
}

function ensureLogsFilterChips() {
  const toolbar = document.querySelector('#logs-content .logs-toolbar') || document.querySelector('.logs-toolbar');
  if (!toolbar) return;
  let wrap = document.getElementById('logs-filter-chips');
  if (!wrap) {
    wrap = document.createElement('div');
    wrap.id = 'logs-filter-chips';
    wrap.className = 'logs-filter-chips';
    wrap.setAttribute('role', 'group');
    wrap.setAttribute('aria-label', 'Log level filter');
    wrap.innerHTML =
      '<button type="button" class="logs-filter-chip is-active" data-logs-filter="all" aria-pressed="true" title="Show the full log tail">All</button>' +
      '<button type="button" class="logs-filter-chip" data-logs-filter="error" aria-pressed="false" title="Show ERROR and panic lines">Error <span class="logs-filter-count" data-logs-filter-count="error">0</span></button>' +
      '<button type="button" class="logs-filter-chip" data-logs-filter="warn" aria-pressed="false" title="Show WARN lines">Warn <span class="logs-filter-count" data-logs-filter-count="warn">0</span></button>';
    toolbar.appendChild(wrap);
    wrap.addEventListener('click', (e) => {
      const btn = e.target && e.target.closest && e.target.closest('[data-logs-filter]');
      if (!btn || !wrap.contains(btn)) return;
      e.preventDefault();
      e.stopPropagation();
      setLogsFilterMode(btn.getAttribute('data-logs-filter') || 'all');
    });
  }
  wireFilterChipToolbarKeyboard(wrap);
}

function setLogsFilterMode(mode) {
  const next = mode === 'error' || mode === 'warn' ? mode : 'all';
  logsFilterMode = next;
  logsViewerRenderKey = '';
  logsSelectedLineText = '';
  document.querySelectorAll('#logs-filter-chips [data-logs-filter]').forEach((btn) => {
    const on = btn.getAttribute('data-logs-filter') === next;
    btn.classList.toggle('is-active', on);
    btn.setAttribute('aria-pressed', on ? 'true' : 'false');
  });
  applyLogsFilter(true);
}

/** Soft tip above Debug Log viewer (Monitors / Perplexity kb-hint parity). */
function ensureLogsKbHint(viewer, show) {
  if (!viewer || !viewer.parentNode) return;
  let hint = document.getElementById('logs-kb-hint');
  if (!show) {
    hint?.remove();
    return;
  }
  if (!hint) {
    hint = document.createElement('div');
    hint.className = 'logs-kb-hint';
    hint.id = 'logs-kb-hint';
    viewer.parentNode.insertBefore(hint, viewer);
  }
  hint.textContent =
    '↑↓ / j k · PgUp/PgDn · Home/End select · Enter / c copies line · Esc clears';
}

function visibleLogsLines(viewer) {
  if (!viewer) return [];
  return Array.from(viewer.querySelectorAll('.logs-line'));
}

function syncLogsLinesTabOrder(viewer, preferEl) {
  const items = visibleLogsLines(viewer);
  ensureLogsKbHint(viewer, items.length > 0);
  if (!items.length) {
    logsSelectedLineText = '';
    viewer?.querySelectorAll('.logs-line.is-selected').forEach((el) => {
      el.classList.remove('is-selected');
      el.setAttribute('aria-selected', 'false');
    });
    return;
  }
  let activeIdx = preferEl ? items.indexOf(preferEl) : -1;
  if (activeIdx < 0 && logsSelectedLineText) {
    activeIdx = items.findIndex((el) => el.dataset.lineText === logsSelectedLineText);
  }
  if (activeIdx < 0) {
    activeIdx = items.findIndex((el) => el.classList.contains('is-selected'));
  }
  if (activeIdx < 0) {
    const focused = document.activeElement;
    activeIdx = focused && items.includes(focused) ? items.indexOf(focused) : -1;
  }
  if (activeIdx < 0) {
    items.forEach((el, i) => {
      el.classList.remove('is-selected');
      el.setAttribute('aria-selected', 'false');
      el.tabIndex = i === 0 ? 0 : -1;
    });
    return;
  }
  items.forEach((el, i) => {
    const on = i === activeIdx;
    el.classList.toggle('is-selected', on);
    el.setAttribute('aria-selected', on ? 'true' : 'false');
    el.tabIndex = on ? 0 : -1;
    if (on) logsSelectedLineText = el.dataset.lineText || el.textContent || '';
  });
}

function clearLogsLineSelection(viewer) {
  if (!viewer) return;
  logsSelectedLineText = '';
  viewer.querySelectorAll('.logs-line.is-selected').forEach((el) => {
    el.classList.remove('is-selected');
    el.setAttribute('aria-selected', 'false');
  });
  const items = visibleLogsLines(viewer);
  items.forEach((el, i) => {
    el.tabIndex = i === 0 ? 0 : -1;
  });
  if (document.activeElement && viewer.contains(document.activeElement)) {
    document.activeElement.blur();
  }
}

function flashLogsLineCopied(el) {
  if (!el) return;
  if (el._logsCopiedTimer) {
    clearTimeout(el._logsCopiedTimer);
    el._logsCopiedTimer = null;
  }
  el.classList.add('is-just-copied');
  const prevTitle = el.getAttribute('title') || '';
  el.title = 'Copied';
  el.setAttribute('aria-label', 'Copied');
  el._logsCopiedTimer = setTimeout(() => {
    el.classList.remove('is-just-copied');
    el._logsCopiedTimer = null;
    if (prevTitle) el.title = prevTitle;
    else el.removeAttribute('title');
    el.setAttribute('aria-label', 'Log line — Enter or c copies');
  }, 1600);
}

async function copyLogsLine(el) {
  if (!el) return false;
  const value = String(el.dataset.lineText || el.textContent || '').trim();
  if (!value) return false;
  const ok = await copyTextToClipboard(value);
  if (ok) flashLogsLineCopied(el);
  return ok;
}

/** Wire j/k list nav + Enter/c copy on Debug Log lines (Perplexity / AI Chat parity). */
function wireLogsViewerKeyboard(viewer) {
  if (!viewer || viewer.dataset.keyboardNav === '1') return;
  viewer.dataset.keyboardNav = '1';
  viewer.setAttribute('role', 'listbox');
  viewer.setAttribute('aria-label', 'Debug log lines');

  viewer.addEventListener('click', (e) => {
    const line = e.target && e.target.closest && e.target.closest('.logs-line');
    if (!line || !viewer.contains(line)) return;
    syncLogsLinesTabOrder(viewer, line);
    line.focus();
  });

  viewer.addEventListener('keydown', (e) => {
    const line = e.target && e.target.closest && e.target.closest('.logs-line');
    if (!line || !viewer.contains(line)) {
      // First arrow/j from viewer chrome focuses first/last line.
      if (e.target !== viewer) return;
      const items = visibleLogsLines(viewer);
      if (!items.length) return;
      let next = -1;
      if (e.key === 'ArrowDown' || e.key === 'j' || e.key === 'Home') next = 0;
      else if (e.key === 'ArrowUp' || e.key === 'k' || e.key === 'End') next = items.length - 1;
      else return;
      e.preventDefault();
      syncLogsLinesTabOrder(viewer, items[next]);
      items[next].focus();
      items[next].scrollIntoView({ block: 'nearest' });
      return;
    }
    const items = visibleLogsLines(viewer);
    const idx = items.indexOf(line);
    if (idx < 0) return;

    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      void copyLogsLine(line);
      return;
    }

    if (
      (e.key === 'c' || e.key === 'C') &&
      !e.metaKey &&
      !e.ctrlKey &&
      !e.altKey
    ) {
      e.preventDefault();
      void copyLogsLine(line);
      return;
    }

    if (e.key === 'Escape' || e.key === 'Esc') {
      if (!line.classList.contains('is-selected') && document.activeElement !== line) {
        return;
      }
      e.preventDefault();
      e.stopPropagation();
      clearLogsLineSelection(viewer);
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
    syncLogsLinesTabOrder(viewer, items[next]);
    items[next].focus();
    if (typeof items[next].scrollIntoView === 'function') {
      items[next].scrollIntoView({ block: 'nearest' });
    }
  });
}

function renderLogsViewerLines(viewer, prefix, text, preferScrollEnd) {
  const wasAtEnd =
    preferScrollEnd ||
    viewer.scrollHeight - viewer.scrollTop - viewer.clientHeight < 48;
  const keepFocus =
    document.activeElement &&
    viewer.contains(document.activeElement) &&
    document.activeElement.classList.contains('logs-line');
  const prevSelected = logsSelectedLineText;

  viewer.replaceChildren();
  if (prefix) {
    const pre = document.createElement('div');
    pre.className = 'logs-viewer-prefix';
    pre.textContent = prefix.replace(/\n+$/, '');
    viewer.appendChild(pre);
  }

  const lines = String(text || '').split('\n');
  // Drop a single trailing empty split from a final newline.
  if (lines.length && lines[lines.length - 1] === '') lines.pop();
  const frag = document.createDocumentFragment();
  for (const raw of lines) {
    const el = document.createElement('div');
    el.className = 'logs-line';
    const kind = logsLineKind(raw);
    if (kind === 'error') el.classList.add('is-error');
    else if (kind === 'warn') el.classList.add('is-warn');
    el.setAttribute('role', 'option');
    el.tabIndex = -1;
    el.setAttribute('aria-selected', 'false');
    el.dataset.lineText = raw;
    el.textContent = raw || ' ';
    el.title = 'Enter / c copies · ↑↓ / j k to move · Esc clears';
    el.setAttribute('aria-label', 'Log line — Enter or c copies');
    frag.appendChild(el);
  }
  viewer.appendChild(frag);
  wireLogsViewerKeyboard(viewer);

  let preferEl = null;
  if (prevSelected) {
    preferEl =
      visibleLogsLines(viewer).find((el) => el.dataset.lineText === prevSelected) || null;
  }
  syncLogsLinesTabOrder(viewer, preferEl);
  if (keepFocus && preferEl) {
    preferEl.focus();
    preferEl.scrollIntoView({ block: 'nearest' });
  } else if (wasAtEnd) {
    viewer.scrollTop = viewer.scrollHeight;
  }
}

function applyLogsFilter(scrollToEnd) {
  const viewer = document.getElementById('logs-viewer');
  if (!viewer) return;
  ensureLogsFilterChips();
  wireLogsViewerKeyboard(viewer);
  if (!viewer.hasAttribute('tabindex')) viewer.setAttribute('tabindex', '0');
  const { prefix, body, hasContent } = logsViewerRaw;
  const counts = countLogsByKind(hasContent ? body : '');
  const errEl = document.querySelector('[data-logs-filter-count="error"]');
  const warnEl = document.querySelector('[data-logs-filter-count="warn"]');
  if (errEl) errEl.textContent = String(counts.error);
  if (warnEl) warnEl.textContent = String(counts.warn);
  applyLogsGlanceState(counts);
  document.querySelectorAll('#logs-filter-chips [data-logs-filter]').forEach((btn) => {
    const key = btn.getAttribute('data-logs-filter');
    btn.classList.toggle('has-hits', key === 'error' ? counts.error > 0 : key === 'warn' ? counts.warn > 0 : false);
  });

  if (!hasContent) {
    const emptyBody = body || '(empty log)';
    const key = `empty|${logsFilterMode}|${prefix}|${emptyBody}`;
    if (key === logsViewerRenderKey && viewer.querySelector('.logs-line, .logs-viewer-empty')) {
      ensureLogsKbHint(viewer, false);
      return;
    }
    logsViewerRenderKey = key;
    logsSelectedLineText = '';
    viewer.replaceChildren();
    const empty = document.createElement('div');
    empty.className = 'logs-viewer-empty';
    empty.textContent = prefix + emptyBody;
    viewer.appendChild(empty);
    viewer.classList.add('is-empty');
    ensureLogsKbHint(viewer, false);
    return;
  }

  const filtered = filterLogsBody(body, logsFilterMode);
  if (logsFilterMode !== 'all' && !filtered.trim()) {
    const empty =
      logsFilterMode === 'error'
        ? 'Nothing here yet — no ERROR lines in this tail'
        : 'Nothing here yet — no WARN lines in this tail';
    const key = `miss|${logsFilterMode}|${prefix}|${empty}`;
    if (key === logsViewerRenderKey && viewer.querySelector('.logs-viewer-empty')) {
      ensureLogsKbHint(viewer, false);
      return;
    }
    logsViewerRenderKey = key;
    logsSelectedLineText = '';
    viewer.replaceChildren();
    const emptyEl = document.createElement('div');
    emptyEl.className = 'logs-viewer-empty';
    emptyEl.textContent = prefix + empty;
    viewer.appendChild(emptyEl);
    viewer.classList.add('is-empty');
    ensureLogsKbHint(viewer, false);
    return;
  }

  const key = `lines|${logsFilterMode}|${prefix}|${filtered}`;
  if (key === logsViewerRenderKey && viewer.querySelector('.logs-line')) {
    if (scrollToEnd) viewer.scrollTop = viewer.scrollHeight;
    ensureLogsKbHint(viewer, visibleLogsLines(viewer).length > 0);
    return;
  }
  logsViewerRenderKey = key;
  viewer.classList.remove('is-empty');
  renderLogsViewerLines(viewer, prefix, filtered, scrollToEnd);
}

async function refreshLogsViewer(scrollToEnd = true) {
  const viewer = document.getElementById('logs-viewer');
  const pathHint = document.getElementById('logs-path-hint');
  if (!viewer) return;
  if (!viewer.hasAttribute('tabindex')) viewer.setAttribute('tabindex', '0');
  ensureLogsFilterChips();
  const inv = getInvoke() || invoke;
  if (!inv) {
    logsViewerRaw = { prefix: '', body: 'App not ready.', hasContent: false };
    applyLogsFilter(false);
    return;
  }
  try {
    const tail = await inv('read_debug_log', { maxBytes: 262144 });
    if (pathHint && tail.path) {
      pathHint.dataset.fullPath = tail.path;
      const display = tail.path.replace(/^\/Users\/[^/]+/, '~');
      pathHint.dataset.pathDisplay = display;
      if (!pathHint.classList.contains('is-just-saved')) {
        pathHint.textContent = display;
      }
      pathHint.title = `${tail.path} — click to copy`;
    }
    const prefix = tail.truncated
      ? `… truncated (showing last ~${Math.round((tail.content || '').length / 1024)} KiB of ${Math.round((tail.total_bytes || 0) / 1024)} KiB)\n\n`
      : '';
    logsViewerRaw = {
      prefix,
      body: tail.content || '(empty log)',
      hasContent: !!tail.content,
    };
    applyLogsFilter(scrollToEnd);
  } catch (err) {
    logsViewerRaw = { prefix: '', body: 'Failed to read log: ' + String(err), hasContent: false };
    applyLogsFilter(false);
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

/** Expand Disk Cleanup if collapsed (empty CTA / Review scopes). */
function ensureDiskCleanupSectionExpanded() {
  const content = document.getElementById('disk-cleanup-content');
  const section = document.querySelector('.disk-cleanup-section');
  const header = document.getElementById('disk-cleanup-header');
  if (!content) return;
  if (!diskCleanupCollapsed && !content.classList.contains('collapsed')) {
    syncDiskCleanupCollapsedGlance();
    return;
  }
  diskCleanupCollapsed = false;
  setSectionCollapsed('disk_cleanup_collapsed', false);
  content.classList.remove('collapsed');
  section?.classList.remove('collapsed');
  header?.setAttribute('aria-expanded', 'true');
  if (typeof header?._syncCollapseA11y === 'function') header._syncCollapseA11y();
  syncSectionIcon('icon-disk-cleanup', true);
  stopDiskCleanupGlancePoll();
  syncDiskCleanupCollapsedGlance();
}

/** Collapsed-section glance under Disk Cleanup header (Monitors parity). */
function ensureDiskCleanupCollapsedGlance() {
  const header = document.getElementById('disk-cleanup-header');
  if (!header) return null;
  let glance = document.getElementById('disk-cleanup-collapsed-glance');
  if (!glance) {
    glance = document.createElement('div');
    glance.id = 'disk-cleanup-collapsed-glance';
    glance.className = 'disk-cleanup-collapsed-glance';
    glance.hidden = true;
    glance.innerHTML = '<span id="disk-cleanup-collapsed-glance-text"></span>';
    header.insertAdjacentElement('afterend', glance);
    wireDiskCleanupCollapsedGlanceClick(glance);
  }
  return glance;
}

function syncDiskCleanupCollapsedGlance() {
  const glance = ensureDiskCleanupCollapsedGlance();
  if (!glance) return;
  const glanceText = document.getElementById('disk-cleanup-collapsed-glance-text');
  const summary = document.getElementById('disk-cleanup-summary');
  if (!diskCleanupCollapsed) {
    glance.hidden = true;
    return;
  }
  glance.hidden = false;
  const st = window.__diskCleanupGlanceState || {};
  const reclaimBytes = Number(st.reclaimBytes) || 0;
  const due = !!st.due;
  const enabledN = Number(st.enabledCount) || 0;
  const totalN = Number(st.totalCount) || 0;
  const scopesOff = totalN > 0 && enabledN < totalN;
  let line =
    (summary && summary.textContent && summary.textContent.trim()) ||
    st.summaryText ||
    'Disk Cleanup';
  if (due && reclaimBytes > 0) {
    line = `${line} · Due now`;
  } else if (due && reclaimBytes <= 0) {
    line = `Due now · ${st.nextLabel || 'run cleanup'}`;
  } else if (scopesOff && reclaimBytes <= 0) {
    line = `${enabledN}/${totalN} scopes on · Review`;
  } else if (st.nextLabel && reclaimBytes <= 0 && line === 'Clean') {
    line = `Clean · ${st.nextLabel}`;
  }
  if (glanceText) glanceText.textContent = line;
  glance.classList.toggle('has-reclaim', reclaimBytes > 0);
  glance.classList.toggle('is-due', due && reclaimBytes <= 0);
  glance.classList.toggle('has-scopes-off', scopesOff && reclaimBytes <= 0 && !due);
  glance.classList.toggle('is-clean', reclaimBytes <= 0 && !due && !scopesOff);
  glance.setAttribute('role', 'button');
  glance.tabIndex = 0;
  if (reclaimBytes > 0) {
    glance.title = 'Show reclaimable categories';
    glance.setAttribute(
      'aria-label',
      `Disk Cleanup: ${line} — click to open reclaimable`
    );
  } else if (due) {
    glance.title = 'Open Disk Cleanup — due now';
    glance.setAttribute('aria-label', 'Disk Cleanup is due — click to open Clean now');
  } else if (scopesOff) {
    glance.title = 'Review Disk Cleanup scopes';
    glance.setAttribute('aria-label', 'Some scopes are off — click to review');
  } else {
    glance.title = 'Show Disk Cleanup';
    glance.setAttribute('aria-label', `Disk Cleanup: ${line} — click to expand`);
  }
}

function activateDiskCleanupCollapsedGlance() {
  const st = window.__diskCleanupGlanceState || {};
  const reclaimBytes = Number(st.reclaimBytes) || 0;
  const due = !!st.due;
  const enabledN = Number(st.enabledCount) || 0;
  const totalN = Number(st.totalCount) || 0;
  if (reclaimBytes > 0) {
    focusDiskCleanupReclaimGlance();
    return;
  }
  if (due) {
    focusDiskCleanupNextRunGlance();
    return;
  }
  if (totalN > 0 && enabledN < totalN) {
    focusDiskCleanupScopesReview();
    return;
  }
  ensureDiskCleanupSectionExpanded();
  const runBtn = document.getElementById('disk-cleanup-run-btn');
  if (runBtn && typeof runBtn.focus === 'function') {
    runBtn.focus();
  }
}

function wireDiskCleanupCollapsedGlanceClick(glance) {
  if (!glance || glance.dataset.diskCleanupCollapsedGlanceWired === '1') return;
  glance.dataset.diskCleanupCollapsedGlanceWired = '1';
  const activate = () => {
    activateDiskCleanupCollapsedGlance();
  };
  glance.addEventListener('click', (e) => {
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
  glance.addEventListener('keydown', (e) => {
    if (e.key !== 'Enter' && e.key !== ' ') return;
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
}

function stopDiskCleanupGlancePoll() {
  if (diskCleanupGlanceInterval) {
    clearInterval(diskCleanupGlanceInterval);
    diskCleanupGlanceInterval = null;
  }
}

function startDiskCleanupGlancePoll() {
  stopDiskCleanupGlancePoll();
  diskCleanupGlanceInterval = setInterval(() => {
    if (!diskCleanupCollapsed) {
      stopDiskCleanupGlancePoll();
      return;
    }
    void refreshDiskCleanupPanel({ deep: false }).then(() => {
      syncDiskCleanupCollapsedGlance();
    });
  }, 60000);
}

/** Focus first disabled scope (or Add form) after empty-list CTA. */
function focusDiskCleanupScopesReview() {
  ensureDiskCleanupSectionExpanded();
  const scopesEl = document.getElementById('disk-cleanup-scopes');
  const addLabel = document.getElementById('disk-cleanup-add-label');
  requestAnimationFrame(() => {
    const disabledRow = scopesEl?.querySelector('.disk-cleanup-scope-row.is-disabled');
    const anyRow = scopesEl?.querySelector('.disk-cleanup-scope-row');
    const row = disabledRow || anyRow;
    if (row) {
      const idx = parseInt(row.getAttribute('data-scope-idx') || '0', 10);
      syncDiskCleanupScopeTabOrder(scopesEl, Number.isFinite(idx) ? idx : 0);
      const enable = row.querySelector('input[data-scope-enabled]');
      const focusEl = enable || row;
      if (typeof focusEl.focus === 'function') focusEl.focus();
      if (typeof row.scrollIntoView === 'function') {
        row.scrollIntoView({ block: 'nearest' });
      }
      return;
    }
    if (addLabel && typeof addLabel.focus === 'function') {
      addLabel.focus();
      if (typeof addLabel.scrollIntoView === 'function') {
        addLabel.scrollIntoView({ block: 'nearest' });
      }
    }
  });
}

/** Reclaimable-now card: jump to first reclaim row, or scopes when nothing pending. */
function focusDiskCleanupReclaimGlance() {
  ensureDiskCleanupSectionExpanded();
  setDiskCleanupFilterMode('reclaim');
  requestAnimationFrame(() => {
    const list = document.getElementById('disk-cleanup-list');
    const reclaimRow =
      visibleDiskCleanupItems(list).find((el) => el.classList.contains('has-reclaim')) ||
      null;
    if (reclaimRow) {
      const idx = parseInt(reclaimRow.getAttribute('data-item-idx') || '0', 10);
      syncDiskCleanupItemTabOrder(list, Number.isFinite(idx) ? idx : 0);
      if (typeof reclaimRow.focus === 'function') reclaimRow.focus();
      if (typeof reclaimRow.scrollIntoView === 'function') {
        reclaimRow.scrollIntoView({ block: 'nearest' });
      }
      return;
    }
    const emptyCta = list?.querySelector('.disk-cleanup-empty-cta');
    if (emptyCta || !list?.querySelector('.disk-cleanup-item')) {
      focusDiskCleanupScopesReview();
      return;
    }
    const runBtn = document.getElementById('disk-cleanup-run-btn');
    if (runBtn && typeof runBtn.focus === 'function') {
      runBtn.focus();
      if (typeof runBtn.scrollIntoView === 'function') {
        runBtn.scrollIntoView({ block: 'nearest' });
      }
    }
  });
}

/** Clickable Reclaimable now card (Monitors summary parity). */
function applyDiskCleanupReclaimCardState(hasReclaim) {
  const reclaimEl = document.getElementById('disk-cleanup-reclaim');
  const card = reclaimEl?.closest('.disk-cleanup-meta-card');
  if (!card) return;
  card.classList.add('is-action');
  card.setAttribute('role', 'button');
  if (hasReclaim) {
    card.title = 'Click to open the first reclaimable category';
    card.setAttribute(
      'aria-label',
      'Reclaimable now — click to open the first reclaimable category'
    );
  } else {
    card.title = 'Click to review cleanup scopes';
    card.setAttribute(
      'aria-label',
      'Nothing reclaimable — click to review cleanup scopes'
    );
  }
  refreshDiskCleanupMetaRovingTabindex();
}

function wireDiskCleanupReclaimCard() {
  const reclaimEl = document.getElementById('disk-cleanup-reclaim');
  const card = reclaimEl?.closest('.disk-cleanup-meta-card');
  if (!card || card.dataset.reclaimNav === '1') return;
  card.dataset.reclaimNav = '1';
  applyDiskCleanupReclaimCardState(false);
  const activate = () => {
    focusDiskCleanupReclaimGlance();
  };
  card.addEventListener('click', (e) => {
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
  card.addEventListener('keydown', (e) => {
    if (e.key !== 'Enter' && e.key !== ' ') return;
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
}

/** Clickable Enabled scopes card (Reclaimable now / Monitors summary parity). */
function applyDiskCleanupEnabledScopesCardState(enabledCount, totalCount) {
  const summaryEl = document.getElementById('disk-cleanup-scope-summary');
  const card = summaryEl?.closest('.disk-cleanup-meta-card');
  if (!card) return;
  const en = Number.isFinite(enabledCount) ? enabledCount : 0;
  const tot = Number.isFinite(totalCount) ? totalCount : 0;
  const off = Math.max(0, tot - en);
  card.classList.add('is-action');
  card.classList.toggle('has-scopes-off', off > 0);
  card.setAttribute('role', 'button');
  if (off > 0) {
    card.title = 'Click to review scopes — some are off';
    card.setAttribute(
      'aria-label',
      `Enabled scopes ${en} of ${tot} — click to turn more on`
    );
  } else if (tot > 0) {
    card.title = 'Click to review cleanup scopes';
    card.setAttribute(
      'aria-label',
      `Enabled scopes ${en} of ${tot} — click to review`
    );
  } else {
    card.title = 'Click to add a cleanup scope';
    card.setAttribute(
      'aria-label',
      'No scopes yet — click to add a cleanup path'
    );
  }
  refreshDiskCleanupMetaRovingTabindex();
}

function wireDiskCleanupEnabledScopesCard() {
  const summaryEl = document.getElementById('disk-cleanup-scope-summary');
  const card = summaryEl?.closest('.disk-cleanup-meta-card');
  if (!card || card.dataset.scopesNav === '1') return;
  card.dataset.scopesNav = '1';
  applyDiskCleanupEnabledScopesCardState(0, 0);
  const activate = () => {
    focusDiskCleanupScopesReview();
  };
  card.addEventListener('click', (e) => {
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
  card.addEventListener('keydown', (e) => {
    if (e.key !== 'Enter' && e.key !== ' ') return;
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
}

/** Scroll/focus Clean now when due, else last-run summary (Reclaimable / Enabled scopes parity). */
function focusDiskCleanupNextRunGlance() {
  const nextEl = document.getElementById('disk-cleanup-next');
  const label = (nextEl?.textContent || '').trim();
  const due =
    label.includes('Due now') ||
    (() => {
      const utc = nextEl?.title || '';
      if (!utc) return false;
      const t = Date.parse(utc);
      return Number.isFinite(t) && t <= Date.now();
    })();
  if (due) {
    const runBtn = document.getElementById('disk-cleanup-run-btn');
    if (runBtn && typeof runBtn.focus === 'function') {
      runBtn.focus();
      if (typeof runBtn.scrollIntoView === 'function') {
        runBtn.scrollIntoView({ block: 'nearest' });
      }
      return;
    }
  }
  const lastEl = document.getElementById('disk-cleanup-last');
  if (lastEl && typeof lastEl.scrollIntoView === 'function') {
    lastEl.scrollIntoView({ block: 'nearest' });
    lastEl.setAttribute('tabindex', '-1');
    lastEl.focus({ preventScroll: true });
  }
}

/** Clickable Next automatic run card (Reclaimable / Enabled scopes parity). */
function applyDiskCleanupNextRunCardState(isDue) {
  const nextEl = document.getElementById('disk-cleanup-next');
  const card = nextEl?.closest('.disk-cleanup-meta-card');
  if (!card) return;
  card.classList.add('is-action');
  card.classList.toggle('has-due', !!isDue);
  card.setAttribute('role', 'button');
  if (isDue) {
    card.title = 'Cleanup is due — click to focus Clean now';
    card.setAttribute(
      'aria-label',
      'Next automatic run due — click to focus Clean now'
    );
  } else {
    card.title = 'Click to scroll to last run summary';
    card.setAttribute(
      'aria-label',
      'Next automatic run — click to scroll to last run summary'
    );
  }
  refreshDiskCleanupMetaRovingTabindex();
}

function wireDiskCleanupNextRunCard() {
  const nextEl = document.getElementById('disk-cleanup-next');
  const card = nextEl?.closest('.disk-cleanup-meta-card');
  if (!card || card.dataset.nextRunNav === '1') return;
  card.dataset.nextRunNav = '1';
  applyDiskCleanupNextRunCardState(false);
  const activate = () => {
    focusDiskCleanupNextRunGlance();
  };
  card.addEventListener('click', (e) => {
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
  card.addEventListener('keydown', (e) => {
    if (e.key !== 'Enter' && e.key !== ' ') return;
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
}

/** Launch/periodic runs use enabled scopes — focus first on scope (not disabled-first). */
function focusDiskCleanupRunsWhenGlance() {
  ensureDiskCleanupSectionExpanded();
  const scopesEl = document.getElementById('disk-cleanup-scopes');
  const addLabel = document.getElementById('disk-cleanup-add-label');
  requestAnimationFrame(() => {
    const enabledRow = scopesEl?.querySelector(
      '.disk-cleanup-scope-row:not(.is-disabled)'
    );
    const anyRow = scopesEl?.querySelector('.disk-cleanup-scope-row');
    const row = enabledRow || anyRow;
    if (row) {
      const idx = parseInt(row.getAttribute('data-scope-idx') || '0', 10);
      syncDiskCleanupScopeTabOrder(scopesEl, Number.isFinite(idx) ? idx : 0);
      const enable = row.querySelector('input[data-scope-enabled]');
      const focusEl = enable || row;
      if (typeof focusEl.focus === 'function') focusEl.focus();
      if (typeof row.scrollIntoView === 'function') {
        row.scrollIntoView({ block: 'nearest' });
      }
      return;
    }
    if (addLabel && typeof addLabel.focus === 'function') {
      addLabel.focus();
      if (typeof addLabel.scrollIntoView === 'function') {
        addLabel.scrollIntoView({ block: 'nearest' });
      }
    }
  });
}

/** Clickable Runs when card (Next automatic run / Enabled scopes parity). */
function applyDiskCleanupRunsWhenCardState(triggersText) {
  const triggersEl = document.getElementById('disk-cleanup-triggers');
  const card = triggersEl?.closest('.disk-cleanup-meta-card');
  if (!card) return;
  card.classList.add('is-action');
  card.setAttribute('role', 'button');
  const periodicOff = (triggersText || '').includes('Periodic: off');
  card.classList.toggle('has-periodic-off', periodicOff);
  if (periodicOff) {
    card.title =
      'Periodic cleanup is off — click to review scopes that run on launch';
    card.setAttribute(
      'aria-label',
      'Runs when — periodic off; click to review launch scopes'
    );
  } else {
    card.title = 'Click to review scopes that run on launch and on schedule';
    card.setAttribute(
      'aria-label',
      'Runs when — click to review launch and scheduled scopes'
    );
  }
  refreshDiskCleanupMetaRovingTabindex();
}

function wireDiskCleanupRunsWhenCard() {
  const triggersEl = document.getElementById('disk-cleanup-triggers');
  const card = triggersEl?.closest('.disk-cleanup-meta-card');
  if (!card || card.dataset.runsWhenNav === '1') return;
  card.dataset.runsWhenNav = '1';
  applyDiskCleanupRunsWhenCardState('');
  const activate = () => {
    focusDiskCleanupRunsWhenGlance();
  };
  card.addEventListener('click', (e) => {
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
  card.addEventListener('keydown', (e) => {
    if (e.key !== 'Enter' && e.key !== ' ') return;
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
}

/** Action meta cards in DOM order (Reclaim · Next · Runs when · Scopes). */
function getDiskCleanupMetaCards() {
  const wrap = document.querySelector('.disk-cleanup-meta');
  if (!wrap) return [];
  return Array.from(
    wrap.querySelectorAll(':scope > .disk-cleanup-meta-card.is-action')
  ).filter((el) => {
    if (!el || el.hidden) return false;
    return (
      el.getClientRects().length > 0 ||
      el.offsetParent !== null ||
      wrap.contains(el)
    );
  });
}

function refreshDiskCleanupMetaRovingTabindex(preferred) {
  const cards = getDiskCleanupMetaCards();
  if (!cards.length) return;
  const focused = cards.find((el) => el === document.activeElement);
  const current =
    (preferred && cards.includes(preferred) && preferred) ||
    focused ||
    cards.find((el) => el.tabIndex === 0) ||
    cards[0];
  for (const el of cards) {
    el.tabIndex = el === current ? 0 : -1;
  }
}

function ensureDiskCleanupMetaKbStyles() {
  if (document.getElementById('mac-stats-disk-meta-kb-styles')) return;
  const style = document.createElement('style');
  style.id = 'mac-stats-disk-meta-kb-styles';
  style.textContent = `
    .disk-cleanup-meta-kb-hint {
      margin: 4px 0 0;
      font-size: 11px;
      opacity: 0.72;
      width: 100%;
      flex-basis: 100%;
      grid-column: 1 / -1;
    }
  `;
  document.head.appendChild(style);
}

function ensureDiskCleanupMetaKbHint() {
  ensureDiskCleanupMetaKbStyles();
  const wrap = document.querySelector('.disk-cleanup-meta');
  if (!wrap) return;
  let hint = document.getElementById('disk-cleanup-meta-kb-hint');
  if (!hint) {
    hint = document.createElement('div');
    hint.id = 'disk-cleanup-meta-kb-hint';
    hint.className = 'disk-cleanup-meta-kb-hint';
    hint.setAttribute('aria-hidden', 'true');
    wrap.appendChild(hint);
  }
  hint.textContent =
    '← → / h l · Home/End move · Enter / Space opens reclaim / next / runs / scopes';
}

/**
 * Disk Cleanup meta-card toolbar keyboard — focus Reclaim · Next · Runs when ·
 * Scopes, then ←→ / h l / Home/End (power-strip / filter-chip parity).
 * Enter/Space keep existing card activate.
 */
function ensureDiskCleanupMetaKeyboard() {
  const wrap = document.querySelector('.disk-cleanup-meta');
  if (!wrap) return;
  ensureDiskCleanupMetaKbHint();
  refreshDiskCleanupMetaRovingTabindex();
  if (wrap.dataset.diskMetaKbWired === '1') return;
  wrap.dataset.diskMetaKbWired = '1';
  if (!wrap.getAttribute('role')) {
    wrap.setAttribute('role', 'toolbar');
  }
  if (!wrap.getAttribute('aria-label')) {
    wrap.setAttribute('aria-label', 'Disk cleanup summary cards');
  }
  wrap.addEventListener('focusin', (e) => {
    const cards = getDiskCleanupMetaCards();
    if (cards.includes(e.target)) refreshDiskCleanupMetaRovingTabindex(e.target);
  });
  wrap.addEventListener('keydown', (e) => {
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    const cards = getDiskCleanupMetaCards();
    if (!cards.length) return;
    const idx = cards.indexOf(document.activeElement);
    if (idx < 0) return;
    let next = -1;
    if (
      e.key === 'ArrowRight' ||
      e.key === 'l' ||
      e.key === 'ArrowDown' ||
      e.key === 'j'
    ) {
      next = Math.min(idx + 1, cards.length - 1);
    } else if (
      e.key === 'ArrowLeft' ||
      e.key === 'h' ||
      e.key === 'ArrowUp' ||
      e.key === 'k'
    ) {
      next = Math.max(idx - 1, 0);
    } else if (e.key === 'Home') {
      next = 0;
    } else if (e.key === 'End') {
      next = cards.length - 1;
    } else {
      return;
    }
    e.preventDefault();
    e.stopPropagation();
    if (next === idx) return;
    refreshDiskCleanupMetaRovingTabindex(cards[next]);
    cards[next].focus();
  });
}

/** Focusable Disk Cleanup toolbar items (Clean now · Refresh · Save scopes). */
function getDiskCleanupToolbarActionItems(row) {
  const wrap =
    row || document.querySelector('.disk-cleanup-toolbar');
  if (!wrap) return [];
  const items = [];
  const runBtn = document.getElementById('disk-cleanup-run-btn');
  const refreshBtn = document.getElementById('disk-cleanup-refresh-btn');
  const saveBtn = document.getElementById('disk-cleanup-save-scopes-btn');
  if (runBtn && wrap.contains(runBtn) && !runBtn.hidden) items.push(runBtn);
  if (refreshBtn && wrap.contains(refreshBtn) && !refreshBtn.hidden) {
    items.push(refreshBtn);
  }
  if (saveBtn && wrap.contains(saveBtn) && !saveBtn.hidden) items.push(saveBtn);
  return items.filter((el) => {
    if (!el || el.hidden) return false;
    return el.getClientRects().length > 0 || wrap.contains(el);
  });
}

function refreshDiskCleanupToolbarRovingTabindex(row, preferred) {
  const items = getDiskCleanupToolbarActionItems(row);
  if (!items.length) return;
  const focused = items.find((el) => el === document.activeElement);
  const current =
    (preferred && items.includes(preferred) && preferred) ||
    focused ||
    items.find((el) => el.tabIndex === 0) ||
    items[0];
  for (const el of items) {
    el.tabIndex = el === current ? 0 : -1;
  }
}

function ensureDiskCleanupToolbarKbHint(row) {
  const wrap = row || document.querySelector('.disk-cleanup-toolbar');
  if (!wrap) return;
  let hint = wrap.querySelector('.disk-cleanup-toolbar-kb-hint');
  if (!hint) {
    hint = document.createElement('div');
    hint.className = 'disk-cleanup-toolbar-kb-hint';
    hint.setAttribute('aria-hidden', 'true');
    wrap.appendChild(hint);
  }
  const items = getDiskCleanupToolbarActionItems(wrap);
  hint.hidden = items.length < 2;
  hint.textContent =
    '← → / h l · Home/End move · Enter / Space on buttons';
}

/**
 * Disk Cleanup action toolbar keyboard — focus Clean now · Refresh · Save scopes,
 * then ←→ / h l / Home/End (Debug Log / meta-card parity). Enter/Space keeps
 * native button activate.
 */
/** Focusable Disk Cleanup add-scope toolbar items (label · path · days · Recursive · Add scope). */
function getDiskCleanupAddScopeToolbarItems(wrap) {
  const form = wrap || document.querySelector('.disk-cleanup-add-scope');
  if (!form) return [];
  const ids = [
    'disk-cleanup-add-label',
    'disk-cleanup-add-path',
    'disk-cleanup-add-days',
    'disk-cleanup-add-recursive',
    'disk-cleanup-add-btn',
  ];
  return ids
    .map((id) => document.getElementById(id))
    .filter((el) => {
      if (!el || !form.contains(el)) return false;
      if (el.hidden || el.disabled) return false;
      return el.getClientRects().length > 0 || form.contains(el);
    });
}

function refreshDiskCleanupAddScopeToolbarRovingTabindex(wrap, preferred) {
  const form = wrap || document.querySelector('.disk-cleanup-add-scope');
  const items = getDiskCleanupAddScopeToolbarItems(form);
  if (!items.length) return;
  const focused = items.find((el) => el === document.activeElement);
  const current =
    (preferred && items.includes(preferred) && preferred) ||
    focused ||
    items.find((el) => el.tabIndex === 0) ||
    items[0];
  for (const el of items) {
    el.tabIndex = el === current ? 0 : -1;
  }
}

function ensureDiskCleanupAddScopeToolbarKbHint(wrap) {
  const form = wrap || document.querySelector('.disk-cleanup-add-scope');
  if (!form) return;
  let hint = form.querySelector('.disk-cleanup-add-scope-toolbar-kb-hint');
  if (!hint) {
    hint = document.createElement('div');
    hint.className = 'disk-cleanup-add-scope-toolbar-kb-hint';
    hint.setAttribute('aria-hidden', 'true');
    form.appendChild(hint);
  }
  const items = getDiskCleanupAddScopeToolbarItems(form);
  hint.hidden = items.length < 2;
  hint.textContent =
    '← → / h l · Home/End move · Enter adds from fields · buttons keep activate';
}

/**
 * Disk Cleanup add-scope toolbar keyboard — focus label · path · days · Recursive ·
 * Add scope, then ←→ / h l / Home/End (Monitors add-form toolbar parity).
 */
function ensureDiskCleanupAddScopeToolbarKeyboard(wrap) {
  const form = wrap || document.querySelector('.disk-cleanup-add-scope');
  if (!form) return;
  ensureDiskCleanupAddScopeToolbarKbHint(form);
  refreshDiskCleanupAddScopeToolbarRovingTabindex(form);
  if (form.dataset.diskAddScopeToolbarKbWired === '1') return;
  form.dataset.diskAddScopeToolbarKbWired = '1';
  if (!form.getAttribute('role')) form.setAttribute('role', 'toolbar');
  if (!form.getAttribute('aria-label')) {
    form.setAttribute('aria-label', 'Add cleanup scope');
  }
  form.addEventListener('focusin', (e) => {
    const items = getDiskCleanupAddScopeToolbarItems(form);
    if (items.includes(e.target)) {
      refreshDiskCleanupAddScopeToolbarRovingTabindex(form, e.target);
      ensureDiskCleanupAddScopeToolbarKbHint(form);
    }
  });
  form.addEventListener('keydown', (e) => {
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    const items = getDiskCleanupAddScopeToolbarItems(form);
    if (!items.length) return;
    const idx = items.indexOf(document.activeElement);
    if (idx < 0) return;
    const active = items[idx];
    if (e.key === 'Enter' || e.key === ' ') {
      if (
        active?.id === 'disk-cleanup-add-label' ||
        active?.id === 'disk-cleanup-add-path' ||
        active?.id === 'disk-cleanup-add-days' ||
        active?.id === 'disk-cleanup-add-recursive' ||
        active?.id === 'disk-cleanup-add-btn'
      ) {
        return;
      }
    }
    let next = -1;
    const forward =
      e.key === 'ArrowRight' ||
      e.key === 'l' ||
      e.key === 'ArrowDown' ||
      e.key === 'j';
    const back =
      e.key === 'ArrowLeft' ||
      e.key === 'h' ||
      e.key === 'ArrowUp' ||
      e.key === 'k';
    const isTextLike =
      active?.tagName === 'INPUT' &&
      (active.type === 'text' || active.type === 'number');
    if (forward) {
      if (isTextLike && !monitorUrlInputAtMoveBoundary(active, 1)) return;
      next = Math.min(idx + 1, items.length - 1);
    } else if (back) {
      if (isTextLike && !monitorUrlInputAtMoveBoundary(active, -1)) return;
      next = Math.max(idx - 1, 0);
    } else if (e.key === 'Home') {
      next = 0;
    } else if (e.key === 'End') {
      next = items.length - 1;
    } else {
      return;
    }
    e.preventDefault();
    e.stopPropagation();
    if (next === idx) return;
    refreshDiskCleanupAddScopeToolbarRovingTabindex(form, items[next]);
    items[next].focus();
    if (
      isTextLike &&
      items[next]?.tagName === 'INPUT' &&
      typeof items[next].setSelectionRange === 'function'
    ) {
      const len = (items[next].value || '').length;
      items[next].setSelectionRange(len, len);
    }
  });
}

function ensureDiskCleanupToolbarKeyboard() {
  const wrap = document.querySelector('.disk-cleanup-toolbar');
  if (!wrap) return;
  ensureDiskCleanupToolbarKbHint(wrap);
  refreshDiskCleanupToolbarRovingTabindex(wrap);
  if (wrap.dataset.diskToolbarKbWired === '1') return;
  wrap.dataset.diskToolbarKbWired = '1';
  if (!wrap.getAttribute('role')) wrap.setAttribute('role', 'toolbar');
  if (!wrap.getAttribute('aria-label')) {
    wrap.setAttribute('aria-label', 'Disk cleanup actions');
  }
  wrap.addEventListener('focusin', (e) => {
    const items = getDiskCleanupToolbarActionItems(wrap);
    if (items.includes(e.target)) {
      refreshDiskCleanupToolbarRovingTabindex(wrap, e.target);
      ensureDiskCleanupToolbarKbHint(wrap);
    }
  });
  wrap.addEventListener('keydown', (e) => {
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    const items = getDiskCleanupToolbarActionItems(wrap);
    if (!items.length) return;
    const idx = items.indexOf(document.activeElement);
    if (idx < 0) return;
    if (e.key === 'Enter' || e.key === ' ') return;
    let next = -1;
    if (
      e.key === 'ArrowRight' ||
      e.key === 'l' ||
      e.key === 'ArrowDown' ||
      e.key === 'j'
    ) {
      next = Math.min(idx + 1, items.length - 1);
    } else if (
      e.key === 'ArrowLeft' ||
      e.key === 'h' ||
      e.key === 'ArrowUp' ||
      e.key === 'k'
    ) {
      next = Math.max(idx - 1, 0);
    } else if (e.key === 'Home') {
      next = 0;
    } else if (e.key === 'End') {
      next = items.length - 1;
    } else {
      return;
    }
    e.preventDefault();
    e.stopPropagation();
    if (next === idx) return;
    refreshDiskCleanupToolbarRovingTabindex(wrap, items[next]);
    items[next].focus();
  });
}

/** Last run panel: jump to first category cleaned last run (meta-card parity). */
function focusDiskCleanupLastRunGlance() {
  ensureDiskCleanupSectionExpanded();
  const last = window.__diskCleanupLastRun;
  const list = document.getElementById('disk-cleanup-list');
  const catIds = (last?.categories || [])
    .filter((c) => (c.filesRemoved || 0) > 0 || (c.bytesFreed || 0) > 0)
    .map((c) => c.id)
    .filter(Boolean);
  requestAnimationFrame(() => {
    if (catIds.length && list) {
      for (const id of catIds) {
        const row = list.querySelector(`.disk-cleanup-item[data-cat-id="${CSS.escape(id)}"]`);
        if (row) {
          const idx = parseInt(row.getAttribute('data-item-idx') || '0', 10);
          syncDiskCleanupItemTabOrder(list, Number.isFinite(idx) ? idx : 0);
          if (typeof row.focus === 'function') row.focus();
          if (typeof row.scrollIntoView === 'function') {
            row.scrollIntoView({ block: 'nearest' });
          }
          return;
        }
      }
    }
    const reclaimRow = list?.querySelector('.disk-cleanup-item.has-reclaim');
    if (reclaimRow) {
      focusDiskCleanupReclaimGlance();
      return;
    }
    if (!last) {
      focusDiskCleanupScopesReview();
      return;
    }
    const runBtn = document.getElementById('disk-cleanup-run-btn');
    if (runBtn && typeof runBtn.focus === 'function') {
      runBtn.focus();
      if (typeof runBtn.scrollIntoView === 'function') {
        runBtn.scrollIntoView({ block: 'nearest' });
      }
    }
  });
}

function applyDiskCleanupLastRunState(last) {
  const lastEl = document.getElementById('disk-cleanup-last');
  if (!lastEl) return;
  window.__diskCleanupLastRun = last || null;
  const hadRemoval =
    !!last &&
    ((last.filesRemoved || 0) > 0 ||
      (last.bytesFreed || 0) > 0 ||
      (last.categories || []).some(
        (c) => (c.filesRemoved || 0) > 0 || (c.bytesFreed || 0) > 0
      ));
  const hadSkip = !!last && (last.filesSkipped || 0) > 0;
  lastEl.classList.add('is-action');
  lastEl.classList.toggle('has-last-run', !!last);
  lastEl.classList.toggle('has-skip', hadSkip);
  lastEl.setAttribute('role', 'button');
  lastEl.setAttribute('tabindex', '0');
  if (!last) {
    lastEl.title = 'Click to review cleanup scopes';
    lastEl.setAttribute(
      'aria-label',
      'Last run — not yet this install; click to review cleanup scopes'
    );
  } else if (hadRemoval) {
    lastEl.title = 'Click to open categories cleaned in the last run';
    lastEl.setAttribute(
      'aria-label',
      'Last run — click to open categories cleaned in the last run'
    );
  } else if (hadSkip) {
    lastEl.title = 'Click to review categories — some files were skipped last run';
    lastEl.setAttribute(
      'aria-label',
      'Last run had skipped files — click to review categories'
    );
  } else {
    lastEl.title = 'Click to review cleanup categories';
    lastEl.setAttribute(
      'aria-label',
      'Last run — click to review cleanup categories'
    );
  }
}

function wireDiskCleanupLastRunPanel() {
  const lastEl = document.getElementById('disk-cleanup-last');
  if (!lastEl || lastEl.dataset.lastRunNav === '1') return;
  lastEl.dataset.lastRunNav = '1';
  applyDiskCleanupLastRunState(null);
  const activate = () => {
    focusDiskCleanupLastRunGlance();
  };
  lastEl.addEventListener('click', (e) => {
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
  lastEl.addEventListener('keydown', (e) => {
    if (e.key !== 'Enter' && e.key !== ' ') return;
    e.preventDefault();
    e.stopPropagation();
    activate();
  });
}

/** Category list empty: warm title + Review scopes CTA (Monitors empty Add parity). */
function renderDiskCleanupListEmpty(list) {
  if (!list) return;
  list.innerHTML =
    `<li class="disk-cleanup-empty disk-cleanup-list-empty" role="status">` +
    `<div class="disk-cleanup-empty-msg">Nothing to reclaim yet</div>` +
    `<div class="disk-cleanup-empty-hint">Turn a scope on and Save, or add a custom path.</div>` +
    `<button type="button" class="disk-cleanup-empty-cta">Review scopes</button>` +
    `</li>`;
  list.querySelector('.disk-cleanup-empty-cta')?.addEventListener('click', (e) => {
    e.preventDefault();
    e.stopPropagation();
    focusDiskCleanupScopesReview();
  });
  applyDiskCleanupListFilter();
}

function visibleDiskCleanupItems(listEl) {
  if (!listEl) return [];
  return Array.from(listEl.querySelectorAll('.disk-cleanup-item')).filter(
    (el) => el.style.display !== 'none'
  );
}

/** All / Reclaim / Clean chips (Monitors All/Up/Down parity). */
function ensureDiskCleanupFilterChips() {
  const list = document.getElementById('disk-cleanup-list');
  if (!list || !list.parentNode) return;
  let wrap = document.getElementById('disk-cleanup-filter-chips');
  if (!wrap) {
    wrap = document.createElement('div');
    wrap.id = 'disk-cleanup-filter-chips';
    wrap.className = 'disk-cleanup-filter-chips';
    wrap.setAttribute('role', 'group');
    wrap.setAttribute('aria-label', 'Cleanup category filter');
    wrap.hidden = true;
    wrap.innerHTML =
      '<button type="button" class="disk-cleanup-filter-chip is-active" data-disk-cleanup-filter="all" aria-pressed="true" title="Show every category">All</button>' +
      '<button type="button" class="disk-cleanup-filter-chip" data-disk-cleanup-filter="reclaim" aria-pressed="false" title="Show categories with reclaimable space">Reclaim <span class="disk-cleanup-filter-count" data-disk-cleanup-filter-count="reclaim">0</span></button>' +
      '<button type="button" class="disk-cleanup-filter-chip" data-disk-cleanup-filter="clean" aria-pressed="false" title="Show categories that are already clean">Clean <span class="disk-cleanup-filter-count" data-disk-cleanup-filter-count="clean">0</span></button>';
    list.parentNode.insertBefore(wrap, list);
    wrap.addEventListener('click', (e) => {
      const btn =
        e.target && e.target.closest && e.target.closest('[data-disk-cleanup-filter]');
      if (!btn || !wrap.contains(btn)) return;
      e.preventDefault();
      e.stopPropagation();
      setDiskCleanupFilterMode(btn.getAttribute('data-disk-cleanup-filter') || 'all');
    });
  }
  wireFilterChipToolbarKeyboard(wrap);
}

function setDiskCleanupFilterMode(mode) {
  const next = mode === 'reclaim' || mode === 'clean' ? mode : 'all';
  diskCleanupFilterMode = next;
  document
    .querySelectorAll('#disk-cleanup-filter-chips [data-disk-cleanup-filter]')
    .forEach((btn) => {
      const on = btn.getAttribute('data-disk-cleanup-filter') === next;
      btn.classList.toggle('is-active', on);
      btn.setAttribute('aria-pressed', on ? 'true' : 'false');
    });
  applyDiskCleanupListFilter();
}

function ensureDiskCleanupFilterMissState(listEl, show) {
  if (!listEl) return;
  const existing = listEl.querySelector('.disk-cleanup-filter-miss');
  if (!show) {
    existing?.remove();
    return;
  }
  let wrap = existing;
  if (!wrap) {
    wrap = document.createElement('li');
    wrap.className = 'disk-cleanup-empty disk-cleanup-filter-miss';
    wrap.setAttribute('role', 'status');
    wrap.innerHTML =
      `<div class="disk-cleanup-empty-msg">Nothing matches this filter</div>` +
      `<div class="disk-cleanup-empty-hint">Try All, or clear the category filter.</div>` +
      `<button type="button" class="disk-cleanup-empty-cta disk-cleanup-clear-filter">Clear filter</button>`;
    listEl.appendChild(wrap);
    wrap.querySelector('.disk-cleanup-clear-filter')?.addEventListener('click', (e) => {
      e.preventDefault();
      e.stopPropagation();
      setDiskCleanupFilterMode('all');
    });
  }
}

function applyDiskCleanupListFilter() {
  ensureDiskCleanupFilterChips();
  const chips = document.getElementById('disk-cleanup-filter-chips');
  const listEl = document.getElementById('disk-cleanup-list');
  if (!listEl) return;

  const items = Array.from(listEl.querySelectorAll('.disk-cleanup-item'));
  const trueEmpty = !!listEl.querySelector('.disk-cleanup-list-empty');
  if (chips) chips.hidden = trueEmpty || items.length === 0;

  let reclaimCount = 0;
  let cleanCount = 0;
  items.forEach((el) => {
    if (el.classList.contains('has-reclaim')) reclaimCount++;
    else cleanCount++;
  });

  const reclaimEl = document.querySelector(
    '[data-disk-cleanup-filter-count="reclaim"]'
  );
  const cleanEl = document.querySelector('[data-disk-cleanup-filter-count="clean"]');
  if (reclaimEl) reclaimEl.textContent = String(reclaimCount);
  if (cleanEl) cleanEl.textContent = String(cleanCount);
  document
    .querySelectorAll('#disk-cleanup-filter-chips [data-disk-cleanup-filter]')
    .forEach((btn) => {
      const key = btn.getAttribute('data-disk-cleanup-filter');
      btn.classList.toggle(
        'has-hits',
        key === 'reclaim' ? reclaimCount > 0 : key === 'clean' ? cleanCount > 0 : false
      );
    });

  if (trueEmpty || items.length === 0) {
    ensureDiskCleanupFilterMissState(listEl, false);
    return;
  }

  let visible = 0;
  items.forEach((el) => {
    const hasReclaim = el.classList.contains('has-reclaim');
    let show = true;
    if (diskCleanupFilterMode === 'reclaim') show = hasReclaim;
    else if (diskCleanupFilterMode === 'clean') show = !hasReclaim;
    el.style.display = show ? '' : 'none';
    if (show) visible++;
  });

  ensureDiskCleanupFilterMissState(listEl, visible === 0);
  const prefer =
    typeof window.__diskCleanupItemFocusIdx === 'number'
      ? window.__diskCleanupItemFocusIdx
      : null;
  syncDiskCleanupItemTabOrder(listEl, prefer);
}

async function refreshDiskCleanupPanel(opts) {
  const deep = !!(opts && opts.deep);
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
    const status = await inv('get_disk_cleanup_status', { deep });
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
    const scopesForGlance = Array.isArray(status.scopes) ? status.scopes : [];
    const enabledGlanceN = scopesForGlance.filter((s) => s && s.enabled).length;
    const nextLabelGlance = (status.nextRunLabel || '').trim();
    const dueGlance =
      nextLabelGlance.includes('Due now') ||
      (() => {
        const utc = status.nextRunUtc || '';
        if (!utc) return false;
        const t = Date.parse(utc);
        return Number.isFinite(t) && t <= Date.now();
      })();
    window.__diskCleanupGlanceState = {
      reclaimBytes,
      reclaimFiles,
      due: dueGlance,
      nextLabel: nextLabelGlance || '',
      enabledCount: enabledGlanceN,
      totalCount: scopesForGlance.length,
      summaryText:
        reclaimBytes > 0
          ? `${formatDiskBytes(reclaimBytes)} reclaimable`
          : 'Clean',
    };
    syncDiskCleanupCollapsedGlance();
    if (reclaimEl) {
      reclaimEl.textContent =
        reclaimBytes > 0
          ? `${formatDiskBytes(reclaimBytes)} · ${reclaimFiles} item(s)`
          : 'Nothing pending';
      reclaimEl.closest('.disk-cleanup-meta-card')?.classList.toggle(
        'has-reclaim',
        reclaimBytes > 0
      );
      applyDiskCleanupReclaimCardState(reclaimBytes > 0);
    }
    if (nextEl) {
      nextEl.textContent = status.nextRunLabel || '—';
      nextEl.title = status.nextRunUtc || '';
      const label = (status.nextRunLabel || '').trim();
      const due =
        label.includes('Due now') ||
        (() => {
          const utc = status.nextRunUtc || '';
          if (!utc) return false;
          const t = Date.parse(utc);
          return Number.isFinite(t) && t <= Date.now();
        })();
      applyDiskCleanupNextRunCardState(due);
    }
    if (triggersEl) {
      const triggersJoined = (status.triggers || []).join(' · ') || '—';
      triggersEl.textContent = triggersJoined;
      applyDiskCleanupRunsWhenCardState(triggersJoined);
    }
    if (scopeSummaryEl) {
      scopeSummaryEl.textContent = status.enabledScopeSummary || status.rootHint || '—';
      const scopesForCard = window.__diskCleanupScopes || [];
      const enabledN = scopesForCard.filter((s) => s && s.enabled).length;
      applyDiskCleanupEnabledScopesCardState(enabledN, scopesForCard.length);
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
          const pathEsc = escapeDiskHtml(pathHint);
          const ageDisabled = s.kind === 'mac-stats' ? 'disabled' : '';
          const ageVal = s.maxAgeDays != null ? s.maxAgeDays : '';
          const removeBtn = s.builtin
            ? ''
            : `<button type="button" class="disk-cleanup-scope-remove" data-scope-remove="${idx}">Remove</button>`;
          const rowTitle = s.builtin
            ? '↑↓ / j k · PgUp/PgDn select · click path / c copies · Space toggle enable · R toggle recurse · Esc clears'
            : '↑↓ / j k · PgUp/PgDn select · click path / c copies · Space toggle enable · R toggle recurse · Delete removes custom · Esc clears';
          return `<div class="disk-cleanup-scope-row${s.enabled ? '' : ' is-disabled'}" data-scope-idx="${idx}" data-copy-path="${pathEsc}" role="option" title="${rowTitle}">
            <input type="checkbox" data-scope-enabled="${idx}" ${s.enabled ? 'checked' : ''} aria-label="Enable ${s.label}" />
            <div class="disk-cleanup-scope-main">
              <div class="disk-cleanup-scope-title">${s.label} <span class="disk-cleanup-scope-kind">(${s.kind})</span></div>
              <button type="button" class="disk-cleanup-scope-path" data-copy-path="${pathEsc}" title="Click to copy path">${pathEsc}</button>
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
          '↑↓ / j k · PgUp/PgDn select scope · click path / c copies · Esc clears · Space toggle enable · R toggle recurse · T toggle Trash soft-delete · Delete removes custom · Enter in Add form adds · ⌘S saves';
      } else {
        document.getElementById('disk-cleanup-kb-hint')?.remove();
      }
      syncDiskCleanupScopeTabOrder(scopesEl, preferIdx);
      applyDiskCleanupPathCopyFlash(scopesEl);
    }

    const cats = (status.categories || []).filter((c) => c.enabled !== false);
    if (!cats.length) {
      renderDiskCleanupListEmpty(list);
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
          const pathHint = String(c.pathHint || '').trim();
          const pathEsc = escapeDiskHtml(pathHint);
          const title = has
            ? '↑↓ / j k · PgUp/PgDn select · click path / c copies · Enter Clean now · Esc clears'
            : '↑↓ / j k · PgUp/PgDn select · click path / c copies · Enter focuses Clean now · Esc clears';
          const pathBtn = pathEsc
            ? `<button type="button" class="disk-cleanup-item-path" data-copy-path="${pathEsc}" title="Click to copy path">${pathEsc}</button>`
            : '';
          const catIdEsc = escapeDiskHtml(String(c.id || ''));
          return `<li class="disk-cleanup-item${has ? ' has-reclaim' : ''}" role="option" data-item-idx="${idx}" data-cat-id="${catIdEsc}" data-copy-path="${pathEsc}" title="${title}">
            <div class="disk-cleanup-item-head">
              <span class="disk-cleanup-item-title">${c.label || c.id}</span>
              <span class="disk-cleanup-item-stat">${
                has
                  ? `${formatDiskBytes(c.bytes || 0)} · ${c.fileCount || 0}`
                  : 'OK'
              }</span>
            </div>
            <div class="disk-cleanup-item-policy">${c.policy || ''}</div>
            ${pathBtn}
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
        list.parentNode.insertBefore(listHint, list);
      }
      if (listHint) {
        listHint.textContent =
          'Categories: All · Reclaim · Clean filters · ↑↓ / j k · PgUp/PgDn · Home / End select · click path / c copies · Esc clears · Enter runs Clean now when reclaimable';
      }
      window.__diskCleanupItemFocusIdx = preferItemIdx;
      applyDiskCleanupListFilter();
      applyDiskCleanupPathCopyFlash(list);
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
            ? `Removed ${last.filesRemoved} · freed ${formatDiskBytes(last.bytesFreed || 0)}${
                last.filesSkipped
                  ? ` · skipped ${last.filesSkipped} (Trash move failed)`
                  : ''
              }`
            : last.filesSkipped
              ? `Nothing moved; skipped ${last.filesSkipped} (Trash move failed)`
              : 'Nothing removed')
        }${catBits ? `<br>${catBits}` : ''}`;
      }
      applyDiskCleanupLastRunState(last || null);
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

/** Busy-guard + Added flash for Add scope (click and Enter). */
let diskCleanupAddBusy = false;
async function submitDiskCleanupAddScope() {
  const addBtn = document.getElementById('disk-cleanup-add-btn');
  if (diskCleanupAddBusy) return false;
  if (addBtn && addBtn.classList.contains('is-just-saved')) return false;

  const label = (document.getElementById('disk-cleanup-add-label')?.value || '').trim();
  const path = (document.getElementById('disk-cleanup-add-path')?.value || '').trim();
  if (!label || !path) {
    alert('Label and path are required for a custom scope.');
    return false;
  }

  diskCleanupAddBusy = true;
  if (addBtn) {
    addBtn.disabled = true;
    addBtn.classList.remove('is-just-saved');
    if (addBtn._saveFlashOriginalLabel == null) {
      addBtn._saveFlashOriginalLabel = addBtn.textContent || 'Add scope';
    }
    addBtn.textContent = 'Adding…';
  }

  try {
    const ok = await addDiskCleanupScopeFromForm();
    diskCleanupAddBusy = false;
    if (addBtn) {
      addBtn.disabled = false;
      if (ok) {
        flashSaveButton(addBtn, { savedLabel: 'Added', durationMs: 1600 });
      } else {
        addBtn.textContent = addBtn._saveFlashOriginalLabel || 'Add scope';
        addBtn._saveFlashOriginalLabel = null;
      }
    }
    return !!ok;
  } catch (err) {
    diskCleanupAddBusy = false;
    if (addBtn) {
      addBtn.disabled = false;
      addBtn.textContent = addBtn._saveFlashOriginalLabel || 'Add scope';
      addBtn._saveFlashOriginalLabel = null;
    }
    throw err;
  }
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
// Shared with discord.js / other script tags (save-button-feedback rule).
window.flashSaveButton = flashSaveButton;

function escapeDiskHtml(s) {
  return String(s ?? '')
    .replace(/&/g, '&amp;')
    .replace(/"/g, '&quot;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
}

/** Brief Copied flash on Disk Cleanup path (survives panel refresh). */
let diskCleanupPathCopyFlash = null; // { path }
let diskCleanupPathCopyFlashTimer = null;

function clearDiskCleanupPathCopyFlashTimers() {
  if (diskCleanupPathCopyFlashTimer) {
    clearTimeout(diskCleanupPathCopyFlashTimer);
    diskCleanupPathCopyFlashTimer = null;
  }
}

function requestDiskCleanupPathCopyFlash(path) {
  if (!path) return;
  diskCleanupPathCopyFlash = { path };
  clearDiskCleanupPathCopyFlashTimers();
  diskCleanupPathCopyFlashTimer = setTimeout(() => {
    diskCleanupPathCopyFlash = null;
    diskCleanupPathCopyFlashTimer = null;
    document
      .querySelectorAll(
        '.disk-cleanup-scope-path.is-just-saved, .disk-cleanup-item-path.is-just-saved'
      )
      .forEach((el) => {
        el.classList.remove('is-just-saved');
        const idle = el.getAttribute('data-copy-path') || '';
        el.textContent = idle;
        el.title = 'Click to copy path';
        el._saveFlashOriginalLabel = null;
      });
  }, 1600);
}

function applyDiskCleanupPathCopyFlash(root) {
  if (!diskCleanupPathCopyFlash || !root) return;
  const want = diskCleanupPathCopyFlash.path;
  const btn = Array.from(
    root.querySelectorAll('.disk-cleanup-scope-path, .disk-cleanup-item-path')
  ).find((el) => (el.getAttribute('data-copy-path') || '') === want);
  if (!btn) return;
  btn._saveFlashOriginalLabel = want;
  btn.classList.add('is-just-saved');
  btn.textContent = 'Copied';
  btn.title = 'Copied';
}

/** Brief Copied wash on scope/category row (Debug Log / Perplexity / AI Chat parity). */
function flashDiskCleanupRowCopied(row) {
  if (!row) return;
  if (row._diskCleanupCopiedTimer) {
    clearTimeout(row._diskCleanupCopiedTimer);
    row._diskCleanupCopiedTimer = null;
  }
  row.classList.add('is-just-copied');
  const prevTitle = row.getAttribute('title') || '';
  row.title = 'Copied';
  row.setAttribute('aria-label', 'Copied');
  row._diskCleanupCopiedTimer = setTimeout(() => {
    row.classList.remove('is-just-copied');
    row._diskCleanupCopiedTimer = null;
    if (prevTitle) row.title = prevTitle;
    else row.removeAttribute('title');
    row.removeAttribute('aria-label');
  }, 1600);
}

/** Keyboard `c` / click-to-copy path (Top Processes name + Monitors URL parity). */
async function copyDiskCleanupPathFromRow(row) {
  if (!row) return false;
  const btn = row.querySelector('.disk-cleanup-scope-path, .disk-cleanup-item-path');
  const value = String(
    row.getAttribute('data-copy-path') ||
      btn?.getAttribute('data-copy-path') ||
      btn?._saveFlashOriginalLabel ||
      ''
  ).trim();
  if (!value) return false;
  if (
    row.classList.contains('is-just-copied') ||
    (btn && btn.classList.contains('is-just-saved'))
  ) {
    return true;
  }
  const ok = await copyTextToClipboard(value);
  if (!ok) {
    alert('Could not copy path.');
    return false;
  }
  requestDiskCleanupPathCopyFlash(value);
  applyDiskCleanupPathCopyFlash(row);
  flashDiskCleanupRowCopied(row);
  return true;
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
  const all = Array.from(listEl.querySelectorAll('.disk-cleanup-item'));
  const rows = visibleDiskCleanupItems(listEl);
  if (rows.length === 0) {
    all.forEach((el) => {
      el.classList.remove('is-selected');
      el.setAttribute('tabindex', '-1');
      el.setAttribute('aria-selected', 'false');
    });
    return;
  }
  let activeIdx = 0;
  if (typeof preferIdx === 'number' && preferIdx >= 0) {
    const hit = rows.findIndex(
      (el) => parseInt(el.getAttribute('data-item-idx') || '-1', 10) === preferIdx
    );
    if (hit >= 0) activeIdx = hit;
  } else {
    const focused = rows.findIndex((el) => el === document.activeElement);
    if (focused >= 0) activeIdx = focused;
    else {
      const selected = rows.findIndex((el) => el.classList.contains('is-selected'));
      if (selected >= 0) activeIdx = selected;
    }
  }
  all.forEach((el) => {
    if (el.style.display === 'none') {
      el.classList.remove('is-selected');
      el.setAttribute('tabindex', '-1');
      el.setAttribute('aria-selected', 'false');
    }
  });
  window.__diskCleanupItemFocusIdx = parseInt(
    rows[activeIdx].getAttribute('data-item-idx') || '0',
    10
  );
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
  if (!scopesEl.hasAttribute('tabindex')) scopesEl.setAttribute('tabindex', '0');

  scopesEl.addEventListener('click', (e) => {
    const row = e.target && e.target.closest && e.target.closest('.disk-cleanup-scope-row');
    if (!row || !scopesEl.contains(row)) return;
    const idx = parseInt(row.getAttribute('data-scope-idx') || '0', 10);
    syncDiskCleanupScopeTabOrder(scopesEl, idx);
    const pathBtn = e.target.closest && e.target.closest('.disk-cleanup-scope-path');
    if (pathBtn && scopesEl.contains(pathBtn)) {
      e.preventDefault();
      e.stopPropagation();
      void copyDiskCleanupPathFromRow(row);
      return;
    }
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
    if (!row || !scopesEl.contains(row)) {
      // First arrow/j from listbox chrome focuses first/last scope (Debug Log parity).
      if (e.target !== scopesEl) return;
      const rows = Array.from(scopesEl.querySelectorAll('.disk-cleanup-scope-row'));
      if (!rows.length) return;
      let next = -1;
      if (e.key === 'ArrowDown' || e.key === 'j' || e.key === 'Home') next = 0;
      else if (e.key === 'ArrowUp' || e.key === 'k' || e.key === 'End') next = rows.length - 1;
      else return;
      e.preventDefault();
      syncDiskCleanupScopeTabOrder(scopesEl, next);
      rows[next].focus();
      if (typeof rows[next].scrollIntoView === 'function') {
        rows[next].scrollIntoView({ block: 'nearest' });
      }
      return;
    }
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

    // c copies the scope path (click-to-copy parity; Top Processes / Monitors).
    if (
      (e.key === 'c' || e.key === 'C') &&
      !onNumber &&
      !onTextLike &&
      !e.metaKey &&
      !e.ctrlKey &&
      !e.altKey
    ) {
      e.preventDefault();
      void copyDiskCleanupPathFromRow(row);
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
  if (!listEl.hasAttribute('tabindex')) listEl.setAttribute('tabindex', '0');

  listEl.addEventListener('click', (e) => {
    const row = e.target && e.target.closest && e.target.closest('.disk-cleanup-item');
    if (!row || !listEl.contains(row)) return;
    const idx = parseInt(row.getAttribute('data-item-idx') || '0', 10);
    syncDiskCleanupItemTabOrder(listEl, idx);
    const pathBtn = e.target.closest && e.target.closest('.disk-cleanup-item-path');
    if (pathBtn && listEl.contains(pathBtn)) {
      e.preventDefault();
      e.stopPropagation();
      void copyDiskCleanupPathFromRow(row);
      return;
    }
    row.focus();
  });

  listEl.addEventListener('keydown', (e) => {
    const row = e.target && e.target.closest && e.target.closest('.disk-cleanup-item');
    if (!row || !listEl.contains(row)) {
      // First arrow/j from listbox chrome focuses first/last category (Debug Log parity).
      if (e.target !== listEl) return;
      const rows = visibleDiskCleanupItems(listEl);
      if (!rows.length) return;
      let next = -1;
      if (e.key === 'ArrowDown' || e.key === 'j' || e.key === 'Home') next = 0;
      else if (e.key === 'ArrowUp' || e.key === 'k' || e.key === 'End') next = rows.length - 1;
      else return;
      e.preventDefault();
      const prefer = parseInt(rows[next].getAttribute('data-item-idx') || '0', 10);
      syncDiskCleanupItemTabOrder(listEl, prefer);
      rows[next].focus();
      if (typeof rows[next].scrollIntoView === 'function') {
        rows[next].scrollIntoView({ block: 'nearest' });
      }
      return;
    }
    if (row.style.display === 'none') return;
    const rows = visibleDiskCleanupItems(listEl);
    const idx = rows.indexOf(row);
    if (idx < 0) return;

    if (e.key === 'Enter' || e.key === ' ') {
      if (e.target.closest && e.target.closest('.disk-cleanup-item-path')) {
        return;
      }
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

    // c copies the category path (click-to-copy parity; scopes / Monitors).
    if (
      (e.key === 'c' || e.key === 'C') &&
      !e.metaKey &&
      !e.ctrlKey &&
      !e.altKey
    ) {
      e.preventDefault();
      void copyDiskCleanupPathFromRow(row);
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
      window.__diskCleanupItemFocusIdx = rows[0]
        ? parseInt(rows[0].getAttribute('data-item-idx') || '0', 10)
        : null;
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
    const prefer = parseInt(rows[next].getAttribute('data-item-idx') || '0', 10);
    syncDiskCleanupItemTabOrder(listEl, prefer);
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
  wireDiskCleanupReclaimCard();
  wireDiskCleanupEnabledScopesCard();
  wireDiskCleanupNextRunCard();
  wireDiskCleanupRunsWhenCard();
  ensureDiskCleanupMetaKeyboard();
  ensureDiskCleanupToolbarKeyboard();
  ensureDiskCleanupAddScopeToolbarKeyboard();
  wireDiskCleanupLastRunPanel();
  ensureDiskCleanupCollapsedGlance();

  if (icon && !icon.getAttribute('data-title-base')) {
    icon.setAttribute('data-title-base', icon.title || 'Disk cleanup');
  }

  diskCleanupCollapsed = getSectionCollapsed('disk_cleanup_collapsed');
  const applyCollapsed = () => {
    setIconPaneVisibility(section, content, diskCleanupCollapsed, null);
    if (diskCleanupCollapsed) {
      // Shallow status poll so the collapsed glance stays fresh (no deep Downloads scan).
      void refreshDiskCleanupPanel({ deep: false });
      startDiskCleanupGlancePoll();
    } else {
      stopDiskCleanupGlancePoll();
      refreshDiskCleanupPanel();
    }
    if (header._syncCollapseA11y) header._syncCollapseA11y();
    syncSectionIcon('icon-disk-cleanup', !diskCleanupCollapsed);
    syncDiskCleanupCollapsedGlance();
  };
  applyCollapsed();
  // Do not scan Downloads/Trash on every CPU-window open — only when the section is expanded
  // (applyCollapsed) or the user clicks Refresh / Clean now.

  if (typeof wireCollapsibleHeaderA11y === 'function') {
    wireCollapsibleHeaderA11y(header, {
      contentId: 'disk-cleanup-content',
      getExpanded: () => !diskCleanupCollapsed,
      ignoreSelector:
        '#disk-cleanup-collapsed-glance, #disk-cleanup-refresh-btn, #disk-cleanup-run-btn, #disk-cleanup-save-scopes-btn, #disk-cleanup-add-btn, #disk-cleanup-soft-delete, #disk-cleanup-scopes, #disk-cleanup-add-label, #disk-cleanup-add-path, #disk-cleanup-add-days, #disk-cleanup-add-recursive, .disk-cleanup-add-scope, .disk-cleanup-scopes, .disk-cleanup-soft-delete, input, button, label',
      onToggle: () => {
        diskCleanupCollapsed = !diskCleanupCollapsed;
        setSectionCollapsed('disk_cleanup_collapsed', diskCleanupCollapsed);
        applyCollapsed();
      },
    });
  }

  header.addEventListener('click', (e) => {
    if (
      e.target.closest(
        '#disk-cleanup-collapsed-glance, #disk-cleanup-refresh-btn, #disk-cleanup-run-btn, #disk-cleanup-save-scopes-btn, #disk-cleanup-add-btn, .disk-cleanup-scopes, .disk-cleanup-add-scope, .disk-cleanup-soft-delete, input, button, label'
      )
    ) {
      return;
    }
    e.stopPropagation();
    diskCleanupCollapsed = !diskCleanupCollapsed;
    setSectionCollapsed('disk_cleanup_collapsed', diskCleanupCollapsed);
    applyCollapsed();
  });

  if (refreshBtn) {
    if (!refreshBtn.dataset.idleLabel) {
      refreshBtn.dataset.idleLabel = refreshBtn.textContent || 'Refresh';
    }
    refreshBtn.addEventListener('click', async (e) => {
      e.stopPropagation();
      if (refreshBtn.disabled) return;
      const runBusy = runBtn && runBtn.disabled && /Cleaning/i.test(runBtn.textContent || '');
      if (runBusy) return;
      refreshBtn.classList.remove('is-just-saved');
      if (refreshBtn._saveFlashTimer) {
        clearTimeout(refreshBtn._saveFlashTimer);
        refreshBtn._saveFlashTimer = null;
        refreshBtn._saveFlashOriginalLabel = null;
      }
      refreshBtn.disabled = true;
      refreshBtn.textContent = 'Refreshing…';
      try {
        await refreshDiskCleanupPanel({ deep: true });
        refreshBtn.disabled = false;
        refreshBtn.textContent = refreshBtn.dataset.idleLabel || 'Refresh';
        flashSaveButton(refreshBtn, { savedLabel: 'Refreshed', durationMs: 1600 });
      } catch (err) {
        console.warn('disk cleanup refresh', err);
        refreshBtn.disabled = false;
        refreshBtn.textContent = refreshBtn.dataset.idleLabel || 'Refresh';
      }
    });
  }

  let diskCleanupSaveBusy = false;
  if (saveBtn) {
    saveBtn.addEventListener('click', async (e) => {
      e.stopPropagation();
      if (diskCleanupSaveBusy) return;
      if (saveBtn.classList.contains('is-just-saved')) return;

      diskCleanupSaveBusy = true;
      saveBtn.disabled = true;
      saveBtn.classList.remove('is-just-saved');
      if (saveBtn._saveFlashOriginalLabel == null) {
        saveBtn._saveFlashOriginalLabel = saveBtn.textContent || 'Save scopes';
      }
      saveBtn.textContent = 'Saving…';

      try {
        await saveDiskCleanupScopes(readDiskCleanupScopesFromDom());
        const softEl = document.getElementById('disk-cleanup-soft-delete');
        const invoke = getInvoke();
        if (softEl && invoke) {
          await invoke('set_disk_cleanup_soft_delete', { softDelete: !!softEl.checked });
          await refreshDiskCleanupPanel();
        }
        diskCleanupSaveBusy = false;
        saveBtn.disabled = false;
        flashSaveButton(saveBtn, { savedLabel: 'Saved', durationMs: 1600 });
      } catch (err) {
        diskCleanupSaveBusy = false;
        saveBtn.disabled = false;
        saveBtn.textContent =
          saveBtn._saveFlashOriginalLabel || 'Save scopes';
        saveBtn._saveFlashOriginalLabel = null;
        alert(`Save scopes failed: ${err?.message || err}`);
      }
    });
  }

  /** Wrap soft-delete label text so we can flash Saved without wiping the checkbox. */
  function ensureDiskCleanupSoftDeleteLabel(softToggle) {
    const label = softToggle?.closest?.('label.disk-cleanup-soft-delete');
    if (!label) return null;
    let span = label.querySelector('.disk-cleanup-soft-delete-label');
    if (span) return span;
    const texts = [];
    for (const node of [...label.childNodes]) {
      if (node.nodeType !== Node.TEXT_NODE) continue;
      const t = node.textContent.replace(/\s+/g, ' ').trim();
      if (t) texts.push(t);
      node.remove();
    }
    span = document.createElement('span');
    span.className = 'disk-cleanup-soft-delete-label';
    span.textContent =
      texts.join(' ') || 'Move cleaned items to Trash (recoverable)';
    label.appendChild(span);
    return span;
  }

  function flashDiskCleanupSoftDeleteSaved(softToggle) {
    const span = ensureDiskCleanupSoftDeleteLabel(softToggle);
    if (!span || span.classList.contains('is-just-saved')) return;
    const original = span._saveFlashOriginalLabel || span.textContent || '';
    span._saveFlashOriginalLabel = original;
    span.classList.add('is-just-saved');
    span.textContent = 'Saved';
    clearTimeout(span._saveFlashTimer);
    span._saveFlashTimer = setTimeout(() => {
      span.classList.remove('is-just-saved');
      span.textContent = original;
      span._saveFlashOriginalLabel = null;
    }, 1600);
  }

  let diskCleanupSoftDeleteBusy = false;
  const softToggle = document.getElementById('disk-cleanup-soft-delete');
  if (softToggle) {
    ensureDiskCleanupSoftDeleteLabel(softToggle);
    softToggle.addEventListener('change', async (e) => {
      e.stopPropagation();
      const span = ensureDiskCleanupSoftDeleteLabel(softToggle);
      if (
        diskCleanupSoftDeleteBusy ||
        softToggle.disabled ||
        (span && span.classList.contains('is-just-saved'))
      ) {
        softToggle.checked = !softToggle.checked;
        return;
      }
      const inv = getInvoke();
      if (!inv) return;
      diskCleanupSoftDeleteBusy = true;
      softToggle.disabled = true;
      try {
        await inv('set_disk_cleanup_soft_delete', {
          softDelete: !!softToggle.checked,
        });
        await refreshDiskCleanupPanel();
        flashDiskCleanupSoftDeleteSaved(softToggle);
      } catch (err) {
        softToggle.checked = !softToggle.checked;
        alert(`Could not save delete mode: ${err?.message || err}`);
      } finally {
        diskCleanupSoftDeleteBusy = false;
        softToggle.disabled = false;
      }
    });
  }

  if (addBtn) {
    addBtn.addEventListener('click', async (e) => {
      e.stopPropagation();
      try {
        await submitDiskCleanupAddScope();
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
      void submitDiskCleanupAddScope().catch((err) => {
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
      if (runBtn.disabled) return;
      const inv = getInvoke();
      if (!inv) return;
      try {
        await saveDiskCleanupScopes(readDiskCleanupScopesFromDom());
        const softEl = document.getElementById('disk-cleanup-soft-delete');
        if (softEl) {
          await inv('set_disk_cleanup_soft_delete', { softDelete: !!softEl.checked });
        }
      } catch (_) {}
      runBtn.classList.remove('is-just-saved');
      if (runBtn._saveFlashTimer) {
        clearTimeout(runBtn._saveFlashTimer);
        runBtn._saveFlashTimer = null;
        runBtn._saveFlashOriginalLabel = null;
      }
      runBtn.disabled = true;
      runBtn.textContent = 'Cleaning…';
      if (refreshBtn) refreshBtn.disabled = true;
      try {
        await inv('run_disk_cleanup_now');
        await refreshDiskCleanupPanel();
        if (refreshBtn) {
          refreshBtn.disabled = false;
          refreshBtn.textContent = refreshBtn.dataset.idleLabel || 'Refresh';
        }
        // refreshDiskCleanupPanel restores idle Clean-now label; flash confirms success.
        flashSaveButton(runBtn, { savedLabel: 'Cleaned', durationMs: 1600 });
      } catch (err) {
        console.warn('disk cleanup run', err);
        runBtn.disabled = false;
        runBtn.textContent = 'Clean now';
        if (refreshBtn) {
          refreshBtn.disabled = false;
          refreshBtn.textContent = refreshBtn.dataset.idleLabel || 'Refresh';
        }
        alert(`Cleanup failed: ${err?.message || err}`);
      }
    });
  }
}

async function copyTextToClipboard(text) {
  const value = String(text || '').trim();
  if (!value) return false;
  try {
    if (navigator.clipboard && typeof navigator.clipboard.writeText === 'function') {
      await navigator.clipboard.writeText(value);
      return true;
    }
  } catch (_) {
    /* fall through */
  }
  try {
    const ta = document.createElement('textarea');
    ta.value = value;
    ta.setAttribute('readonly', '');
    ta.style.position = 'fixed';
    ta.style.left = '-9999px';
    document.body.appendChild(ta);
    ta.select();
    const ok = document.execCommand('copy');
    document.body.removeChild(ta);
    return !!ok;
  } catch (_) {
    return false;
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
  const pathHint = document.getElementById('logs-path-hint');
  if (!header || !content) return;

  ensureLogsFilterChips();
  ensureLogsToolbarKeyboard();
  ensureLogsErrorGlance();
  startLogsGlancePoll();

  logsSectionCollapsed = getSectionCollapsed('logs_collapsed');
  const applyCollapsed = () => {
    setIconPaneVisibility(section, content, logsSectionCollapsed, divider);
    if (logsSectionCollapsed) {
      stopLogsAutoRefresh();
      applyLogsGlanceState(logsGlanceCounts);
    } else {
      refreshLogsViewer(true);
      if (autoCb && autoCb.checked) startLogsAutoRefresh();
    }
    if (header._syncCollapseA11y) header._syncCollapseA11y();
    syncSectionIcon('icon-logs', !logsSectionCollapsed);
  };
  applyCollapsed();

  wireCollapsibleHeaderA11y(header, {
    contentId: 'logs-content',
    getExpanded: () => !logsSectionCollapsed,
    ignoreSelector:
      '#logs-refresh-btn, #logs-open-btn, #logs-autorefresh, #logs-path-hint, #logs-error-glance, label',
    onToggle: () => {
      logsSectionCollapsed = !logsSectionCollapsed;
      setSectionCollapsed('logs_collapsed', logsSectionCollapsed);
      applyCollapsed();
    },
  });

  header.addEventListener('click', (e) => {
    if (e.target && e.target.closest && e.target.closest('#logs-path-hint')) return;
    if (e.target && e.target.closest && e.target.closest('#logs-error-glance')) return;
    e.stopPropagation();
    logsSectionCollapsed = !logsSectionCollapsed;
    setSectionCollapsed('logs_collapsed', logsSectionCollapsed);
    applyCollapsed();
  });

  if (pathHint) {
    pathHint.setAttribute('role', 'button');
    pathHint.tabIndex = 0;
    if (!pathHint.title || pathHint.title === 'Log file path') {
      pathHint.title = 'Click to copy log path';
    }
    const copyLogsPath = async (e) => {
      e.stopPropagation();
      e.preventDefault();
      if (pathHint.classList.contains('is-just-saved')) return;
      const full =
        pathHint.dataset.fullPath ||
        (pathHint.title && !/click to copy/i.test(pathHint.title)
          ? pathHint.title.replace(/\s*—\s*click to copy.*/i, '').trim()
          : '') ||
        (pathHint.textContent || '').trim() ||
        '~/.mac-stats/debug.log';
      const displayPath = pathHint.dataset.pathDisplay || pathHint.textContent || full;
      pathHint.dataset.pathDisplay = displayPath;
      const ok = await copyTextToClipboard(full);
      if (!ok) {
        alert('Could not copy log path.');
        return;
      }
      if (typeof flashSaveButton === 'function') {
        flashSaveButton(pathHint, { savedLabel: 'Copied', durationMs: 1600 });
      } else {
        pathHint.classList.add('is-just-saved');
        pathHint.textContent = 'Copied';
        setTimeout(() => {
          pathHint.classList.remove('is-just-saved');
          pathHint.textContent = pathHint.dataset.pathDisplay || displayPath;
        }, 1600);
      }
    };
    pathHint.addEventListener('click', copyLogsPath);
    pathHint.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' || e.key === ' ') {
        copyLogsPath(e);
      }
    });
  }

  if (refreshBtn) {
    if (!refreshBtn.dataset.idleLabel) {
      refreshBtn.dataset.idleLabel = refreshBtn.textContent || 'Refresh';
    }
    refreshBtn.addEventListener('click', async (e) => {
      e.stopPropagation();
      if (refreshBtn.disabled) return;
      refreshBtn.classList.remove('is-just-saved');
      if (refreshBtn._saveFlashTimer) {
        clearTimeout(refreshBtn._saveFlashTimer);
        refreshBtn._saveFlashTimer = null;
        refreshBtn._saveFlashOriginalLabel = null;
      }
      refreshBtn.disabled = true;
      refreshBtn.textContent = 'Refreshing…';
      try {
        await refreshLogsViewer(true);
        refreshBtn.disabled = false;
        refreshBtn.textContent = refreshBtn.dataset.idleLabel || 'Refresh';
        if (typeof flashSaveButton === 'function') {
          flashSaveButton(refreshBtn, { savedLabel: 'Refreshed', durationMs: 1600 });
        }
      } catch (err) {
        console.warn('[Logs] refresh failed', err);
        refreshBtn.disabled = false;
        refreshBtn.textContent = refreshBtn.dataset.idleLabel || 'Refresh';
      }
    });
  }
  if (openBtn) {
    if (!openBtn.dataset.idleLabel) {
      openBtn.dataset.idleLabel = openBtn.textContent || 'Open in editor';
    }
    openBtn.addEventListener('click', async (e) => {
      e.stopPropagation();
      if (openBtn.disabled || openBtn.classList.contains('is-just-saved')) return;
      const inv = getInvoke() || invoke;
      if (!inv) return;
      openBtn.classList.remove('is-just-saved');
      if (openBtn._saveFlashTimer) {
        clearTimeout(openBtn._saveFlashTimer);
        openBtn._saveFlashTimer = null;
        openBtn._saveFlashOriginalLabel = null;
      }
      openBtn.disabled = true;
      openBtn.textContent = 'Opening…';
      try {
        await inv('open_debug_log');
        openBtn.disabled = false;
        openBtn.textContent = openBtn.dataset.idleLabel || 'Open in editor';
        if (typeof flashSaveButton === 'function') {
          flashSaveButton(openBtn, { savedLabel: 'Opened', durationMs: 1600 });
        }
      } catch (err) {
        console.error('[Logs] open_debug_log failed:', err);
        openBtn.disabled = false;
        openBtn.textContent = openBtn.dataset.idleLabel || 'Open in editor';
      }
    });
  }
  if (autoCb) {
    autoCb.addEventListener('change', () => {
      if (autoCb.checked && !logsSectionCollapsed) startLogsAutoRefresh();
      else stopLogsAutoRefresh();
    });
  }
}

function collapseSectionByIds(sectionSel, contentId, collapsedKey) {
  const content = contentId ? document.getElementById(contentId) : null;
  const section = sectionSel ? document.querySelector(sectionSel) : null;
  if (typeof setIconPaneVisibility === 'function') {
    setIconPaneVisibility(section, content, true, null);
  } else {
    if (content) {
      content.classList.add('collapsed');
      if (content.classList.contains('section-content-collapsible')) {
        content.style.display = 'none';
      }
    }
    if (section) {
      section.classList.add('collapsed');
      section.style.display = 'none';
    }
  }
  if (collapsedKey) setSectionCollapsed(collapsedKey, true);
}

/** Force-collapse heavy sections when Compact CPU window is enabled (does not run on disable). */
window.applyCpuWindowCompactLayout = function applyCpuWindowCompactLayout(compact) {
  if (!compact) return;
  collapseSectionByIds('.monitors-section', 'monitors-content', 'monitors_collapsed');
  collapseSectionByIds('.ollama-section', 'ollama-content', 'ollama_collapsed');
  collapseSectionByIds('.perplexity-section', 'perplexity-content', 'perplexity_collapsed');
  collapseSectionByIds('.logs-section', 'logs-content', 'logs_collapsed');
  collapseSectionByIds('.disk-cleanup-section', 'disk-cleanup-content', 'disk_cleanup_collapsed');
  diskCleanupCollapsed = true;
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

/** CPU window header actions (Refresh · Settings). */
function getCpuHeaderActionsElement() {
  const refresh = document.getElementById('refresh-btn');
  if (refresh) {
    const actions = refresh.closest(
      '.apple-actions, .theme-actions, [class*="actions"]'
    );
    if (actions) return actions;
    const header = refresh.closest('header');
    if (header) return header;
  }
  return (
    document.querySelector(
      'header .apple-actions, header .theme-actions, header [class*="actions"]'
    ) || null
  );
}

/** Refresh · Settings in the CPU window header toolbar. */
function getCpuHeaderToolbarItems() {
  const actions = getCpuHeaderActionsElement();
  const ids = ['refresh-btn', 'settings-btn'];
  return ids
    .map((id) => document.getElementById(id))
    .filter((el) => {
      if (!el) return false;
      if (actions && !actions.contains(el)) return false;
      if (el.hidden || el.disabled) return false;
      if (el.getAttribute('aria-disabled') === 'true') return false;
      return (
        el.getClientRects().length > 0 ||
        el.offsetParent !== null ||
        (actions && actions.contains(el))
      );
    });
}

function refreshCpuHeaderToolbarRovingTabindex(preferred) {
  const items = getCpuHeaderToolbarItems();
  if (!items.length) return;
  const focused = items.find((el) => el === document.activeElement);
  const current =
    (preferred && items.includes(preferred) && preferred) ||
    focused ||
    items.find((el) => el.tabIndex === 0) ||
    items[0];
  for (const el of items) {
    el.tabIndex = el === current ? 0 : -1;
  }
}

function ensureCpuHeaderToolbarKbStyles() {
  if (document.getElementById('mac-stats-header-toolbar-kb-styles')) return;
  const style = document.createElement('style');
  style.id = 'mac-stats-header-toolbar-kb-styles';
  style.textContent = `
    .header-toolbar-kb-hint {
      margin: 2px 0 0;
      font-size: 11px;
      opacity: 0.72;
      width: 100%;
      text-align: right;
    }
  `;
  document.head.appendChild(style);
}

function ensureCpuHeaderToolbarKbHint() {
  ensureCpuHeaderToolbarKbStyles();
  const actions = getCpuHeaderActionsElement();
  if (!actions) return;
  let hint = actions.querySelector('.header-toolbar-kb-hint');
  if (!hint) {
    hint = document.createElement('div');
    hint.className = 'header-toolbar-kb-hint';
    hint.setAttribute('aria-hidden', 'true');
    actions.appendChild(hint);
  }
  const items = getCpuHeaderToolbarItems();
  hint.hidden = items.length < 2;
  hint.textContent =
    '← → / h l · Home/End move · at end crosses to CPU ring · at start crosses to footer';
}

function tryChainHeaderToRingGaugeFirst() {
  const chips = getRingGaugeChips();
  if (!chips.length) return false;
  refreshRingGaugeRovingTabindex(chips[0]);
  chips[0].focus();
  return true;
}

function tryChainRingGaugeToHeaderSettings() {
  const items = getCpuHeaderToolbarItems();
  if (!items.length) return false;
  const target = items[items.length - 1];
  refreshCpuHeaderToolbarRovingTabindex(target);
  target.focus();
  return true;
}

function tryChainRingGaugeToSparklineFirst() {
  const chips = getHistorySparklineChips();
  if (!chips.length) return false;
  refreshHistorySparklineRovingTabindex(chips[0]);
  chips[0].focus();
  return true;
}

function tryChainSparklineToRingLast() {
  const chips = getRingGaugeChips();
  if (!chips.length) return false;
  const target = chips[chips.length - 1];
  refreshRingGaugeRovingTabindex(target);
  target.focus();
  return true;
}

function tryChainSparklineToPowerStripFirst() {
  const chips = getPowerStripChips();
  if (!chips.length) return false;
  const target = chips[0];
  refreshPowerStripRovingTabindex(target);
  target.focus();
  return true;
}

function tryChainPowerStripToSparklineLast() {
  const chips = getHistorySparklineChips();
  if (!chips.length) return false;
  const target = chips[chips.length - 1];
  refreshHistorySparklineRovingTabindex(target);
  target.focus();
  return true;
}

function tryChainPowerStripToIconLineFirst() {
  const items = getIconLineItems();
  if (!items.length) return false;
  const target = items[0];
  refreshIconLineRovingTabindex(target);
  target.focus();
  return true;
}

function tryChainIconLineToPowerStripLast() {
  const chips = getPowerStripChips();
  if (!chips.length) return false;
  const target = chips[chips.length - 1];
  refreshPowerStripRovingTabindex(target);
  target.focus();
  return true;
}

function tryChainHeaderRefreshToFooterLast() {
  const items = getFooterToolbarItems();
  if (!items.length) return false;
  const target = items[items.length - 1];
  refreshFooterToolbarRovingTabindex(target);
  target.focus();
  return true;
}

function activateCpuHeaderToolbarItem(el) {
  if (!el || el.disabled) return;
  if (el.classList.contains('is-just-saved') || el.classList.contains('is-refreshing')) {
    return;
  }
  el.click();
}

/**
 * CPU window header toolbar keyboard — Refresh · Settings, then ←→ / h l / Home/End
 * (ring-gauge + footer wrap chain at ends).
 */
function ensureCpuHeaderToolbarKeyboard() {
  const actions = getCpuHeaderActionsElement();
  if (!actions) return;
  ensureCpuHeaderToolbarKbHint();
  const items = getCpuHeaderToolbarItems();
  for (const el of items) {
    if (!el.hasAttribute('tabindex')) el.tabIndex = -1;
    if (!el.getAttribute('role')) el.setAttribute('role', 'button');
    if (el.id === 'refresh-btn' && !el.getAttribute('aria-label')) {
      el.setAttribute('aria-label', 'Refresh metrics');
    }
    if (el.id === 'settings-btn' && !el.getAttribute('aria-label')) {
      el.setAttribute('aria-label', 'Open settings');
    }
    if (el.dataset.headerToolbarKbWired !== '1') {
      el.dataset.headerToolbarKbWired = '1';
      el.addEventListener('keydown', (e) => {
        if (e.key !== 'Enter' && e.key !== ' ') return;
        if (e.metaKey || e.ctrlKey || e.altKey) return;
        e.preventDefault();
        e.stopPropagation();
        activateCpuHeaderToolbarItem(el);
      });
    }
  }
  refreshCpuHeaderToolbarRovingTabindex();
  if (actions.dataset.headerToolbarKbWired === '1') return;
  actions.dataset.headerToolbarKbWired = '1';
  if (!actions.getAttribute('role')) actions.setAttribute('role', 'toolbar');
  if (!actions.getAttribute('aria-label')) {
    actions.setAttribute('aria-label', 'Window actions');
  }
  actions.addEventListener(
    'click',
    (e) => {
      const toolbarItems = getCpuHeaderToolbarItems();
      if (!toolbarItems.length) return;
      let node = e.target;
      while (node && node !== actions) {
        if (toolbarItems.includes(node)) {
          refreshCpuHeaderToolbarRovingTabindex(node);
          node.focus();
          return;
        }
        node = node.parentElement;
      }
    },
    true
  );
  actions.addEventListener(
    'keydown',
    (e) => {
      if (e.metaKey || e.ctrlKey || e.altKey) return;
      const toolbarItems = getCpuHeaderToolbarItems();
      if (!toolbarItems.length) return;
      const active = document.activeElement;
      if (active !== actions && !actions.contains(active)) return;
      let idx = toolbarItems.indexOf(active);
      if (idx < 0) {
        const seed = toolbarItems[0];
        refreshCpuHeaderToolbarRovingTabindex(seed);
        seed.focus();
        idx = toolbarItems.indexOf(document.activeElement);
        if (idx < 0) return;
      }
      if (e.key === 'Enter' || e.key === ' ') {
        activateCpuHeaderToolbarItem(toolbarItems[idx]);
        return;
      }
      const forward =
        e.key === 'ArrowRight' ||
        e.key === 'l' ||
        e.key === 'ArrowDown' ||
        e.key === 'j';
      const back =
        e.key === 'ArrowLeft' ||
        e.key === 'h' ||
        e.key === 'ArrowUp' ||
        e.key === 'k';
      let next = -1;
      if (forward) {
        if (idx === toolbarItems.length - 1) {
          if (tryChainHeaderToRingGaugeFirst()) {
            e.preventDefault();
            e.stopPropagation();
          }
          return;
        }
        next = idx + 1;
      } else if (back) {
        if (idx === 0) {
          if (tryChainHeaderRefreshToFooterLast()) {
            e.preventDefault();
            e.stopPropagation();
          }
          return;
        }
        next = idx - 1;
      } else if (e.key === 'Home') {
        next = 0;
      } else if (e.key === 'End') {
        next = toolbarItems.length - 1;
      } else {
        return;
      }
      e.preventDefault();
      e.stopPropagation();
      if (next === idx) return;
      refreshCpuHeaderToolbarRovingTabindex(toolbarItems[next]);
      toolbarItems[next].focus();
    },
    true
  );
  actions.addEventListener('focusin', (e) => {
    const toolbarItems = getCpuHeaderToolbarItems();
    if (toolbarItems.includes(e.target)) {
      refreshCpuHeaderToolbarRovingTabindex(e.target);
      ensureCpuHeaderToolbarKbHint();
    }
  });
}

/** CPU window footer (version + GitHub). */
function getCpuFooterElement() {
  return (
    document.querySelector(
      'main footer.apple-footer, main footer.theme-footer, main footer[class*="footer"]'
    ) || document.getElementById('github-link')?.closest('footer') ||
    null
  );
}

/** Version chip + GitHub link in the footer toolbar. */
function getFooterToolbarItems() {
  const footer = getCpuFooterElement();
  if (!footer) return [];
  const version = footer.querySelector(
    '.app-version, .theme-version, .arch-version'
  );
  const github = footer.querySelector('#github-link') || document.getElementById('github-link');
  return [version, github].filter((el) => {
    if (!el || !footer.contains(el)) return false;
    if (el.hidden || el.disabled) return false;
    if (el.getAttribute('aria-disabled') === 'true') return false;
    return (
      el.getClientRects().length > 0 ||
      el.offsetParent !== null ||
      footer.contains(el)
    );
  });
}

function refreshFooterToolbarRovingTabindex(preferred) {
  const items = getFooterToolbarItems();
  if (!items.length) return;
  const focused = items.find((el) => el === document.activeElement);
  const current =
    (preferred && items.includes(preferred) && preferred) ||
    focused ||
    items.find((el) => el.tabIndex === 0) ||
    items[0];
  for (const el of items) {
    el.tabIndex = el === current ? 0 : -1;
  }
}

function ensureFooterToolbarKbStyles() {
  if (document.getElementById('mac-stats-footer-toolbar-kb-styles')) return;
  const style = document.createElement('style');
  style.id = 'mac-stats-footer-toolbar-kb-styles';
  style.textContent = `
    .footer-toolbar-kb-hint {
      margin: 4px 0 0;
      font-size: 11px;
      opacity: 0.72;
      width: 100%;
      text-align: center;
    }
    footer[role="toolbar"] .app-version[tabindex],
    footer[role="toolbar"] .theme-version[tabindex],
    footer[role="toolbar"] .arch-version[tabindex] {
      outline: none;
    }
  `;
  document.head.appendChild(style);
}

function ensureFooterToolbarKbHint() {
  ensureFooterToolbarKbStyles();
  const footer = getCpuFooterElement();
  if (!footer) return;
  let hint = footer.querySelector('.footer-toolbar-kb-hint');
  if (!hint) {
    hint = document.createElement('div');
    hint.className = 'footer-toolbar-kb-hint';
    hint.setAttribute('aria-hidden', 'true');
    footer.appendChild(hint);
  }
  const items = getFooterToolbarItems();
  hint.hidden = items.length < 2;
  hint.textContent =
    '← → / h l · Home/End move · version opens changelog · at end crosses to section icons · at start crosses to last icon';
}

function tryChainFooterToIconLineFirst() {
  const items = getIconLineItems();
  if (!items.length) return false;
  const target = items[0];
  refreshIconLineRovingTabindex(target);
  target.focus();
  return true;
}

function tryChainFooterToIconLineLast() {
  const items = getIconLineItems();
  if (!items.length) return false;
  const target = items[items.length - 1];
  refreshIconLineRovingTabindex(target);
  target.focus();
  return true;
}

function tryChainIconLineToFooterFirst() {
  const items = getFooterToolbarItems();
  if (!items.length) return false;
  const target = items[0];
  refreshFooterToolbarRovingTabindex(target);
  target.focus();
  return true;
}

function tryChainIconLineToFooterLast() {
  const items = getFooterToolbarItems();
  if (!items.length) return false;
  const target = items[items.length - 1];
  refreshFooterToolbarRovingTabindex(target);
  target.focus();
  return true;
}

function activateFooterToolbarItem(el) {
  if (!el) return;
  if (
    el.classList.contains('app-version') ||
    el.classList.contains('theme-version') ||
    el.classList.contains('arch-version')
  ) {
    if (el.classList.contains('is-just-saved')) return;
    el.click();
    return;
  }
  if (el.id === 'github-link') {
    el.click();
  }
}

/**
 * Footer toolbar keyboard — version · GitHub, then ←→ / h l / Home/End
 * (icon-line wrap chain at ends).
 */
function ensureFooterToolbarKeyboard() {
  const footer = getCpuFooterElement();
  if (!footer) return;
  ensureFooterToolbarKbHint();
  const version = footer.querySelector(
    '.app-version, .theme-version, .arch-version'
  );
  if (version) {
    if (!version.hasAttribute('tabindex')) version.tabIndex = 0;
    if (!version.getAttribute('role')) version.setAttribute('role', 'button');
    if (!version.getAttribute('aria-label')) {
      version.setAttribute('aria-label', 'App version — open changelog');
    }
    if (version.dataset.footerVersionKbWired !== '1') {
      version.dataset.footerVersionKbWired = '1';
      version.addEventListener('keydown', (e) => {
        if (e.key !== 'Enter' && e.key !== ' ') return;
        if (e.metaKey || e.ctrlKey || e.altKey) return;
        e.preventDefault();
        e.stopPropagation();
        activateFooterToolbarItem(version);
      });
    }
  }
  refreshFooterToolbarRovingTabindex();
  if (footer.dataset.footerToolbarKbWired === '1') return;
  footer.dataset.footerToolbarKbWired = '1';
  if (!footer.getAttribute('role')) footer.setAttribute('role', 'toolbar');
  if (!footer.getAttribute('aria-label')) {
    footer.setAttribute('aria-label', 'Footer');
  }
  footer.addEventListener(
    'click',
    (e) => {
      const items = getFooterToolbarItems();
      if (!items.length) return;
      let node = e.target;
      while (node && node !== footer) {
        if (items.includes(node)) {
          refreshFooterToolbarRovingTabindex(node);
          node.focus();
          return;
        }
        node = node.parentElement;
      }
    },
    true
  );
  footer.addEventListener(
    'keydown',
    (e) => {
      if (e.metaKey || e.ctrlKey || e.altKey) return;
      const items = getFooterToolbarItems();
      if (!items.length) return;
      const active = document.activeElement;
      if (active !== footer && !footer.contains(active)) return;
      let idx = items.indexOf(active);
      if (idx < 0) {
        const seed = items[0];
        refreshFooterToolbarRovingTabindex(seed);
        seed.focus();
        idx = items.indexOf(document.activeElement);
        if (idx < 0) return;
      }
      if (e.key === 'Enter' || e.key === ' ') {
        activateFooterToolbarItem(items[idx]);
        return;
      }
      const forward =
        e.key === 'ArrowRight' ||
        e.key === 'l' ||
        e.key === 'ArrowDown' ||
        e.key === 'j';
      const back =
        e.key === 'ArrowLeft' ||
        e.key === 'h' ||
        e.key === 'ArrowUp' ||
        e.key === 'k';
      let next = -1;
      if (forward) {
        if (idx === items.length - 1) {
          if (tryChainFooterToIconLineFirst()) {
            e.preventDefault();
            e.stopPropagation();
          }
          return;
        }
        next = idx + 1;
      } else if (back) {
        if (idx === 0) {
          if (tryChainFooterToIconLineLast()) {
            e.preventDefault();
            e.stopPropagation();
          }
          return;
        }
        next = idx - 1;
      } else if (e.key === 'Home') {
        next = 0;
      } else if (e.key === 'End') {
        next = items.length - 1;
      } else {
        return;
      }
      e.preventDefault();
      e.stopPropagation();
      if (next === idx) return;
      refreshFooterToolbarRovingTabindex(items[next]);
      items[next].focus();
    },
    true
  );
  footer.addEventListener('focusin', (e) => {
    const items = getFooterToolbarItems();
    if (items.includes(e.target)) {
      refreshFooterToolbarRovingTabindex(e.target);
      ensureFooterToolbarKbHint();
    }
  });
}

/** Visible, interactive icon-line buttons (skips AI-off / hidden). */
function getIconLineItems() {
  const line = document.getElementById('icon-line');
  if (!line) return [];
  return Array.from(line.querySelectorAll('button.icon-line-item')).filter((el) => {
    if (!el || el.hidden || el.disabled) return false;
    if (el.style.display === 'none') return false;
    if ((el.style.pointerEvents || '').toLowerCase() === 'none') return false;
    try {
      if (window.getComputedStyle(el).pointerEvents === 'none') return false;
    } catch (_) {
      /* ignore */
    }
    return el.getClientRects().length > 0 || el.offsetParent !== null;
  });
}

/** One Tab stop on the icon line; arrows move focus (power-strip parity). */
function refreshIconLineRovingTabindex(preferred) {
  const items = getIconLineItems();
  if (!items.length) return;
  const focused = items.find((el) => el === document.activeElement);
  let current =
    (preferred && items.includes(preferred) && preferred) ||
    focused ||
    items.find((el) => el.tabIndex === 0) ||
    items[0];
  if (focused && !items.includes(focused)) {
    current = items[0];
  }
  for (const el of items) {
    el.tabIndex = el === current ? 0 : -1;
  }
  // Demote any leftover buttons that are currently non-interactive.
  const line = document.getElementById('icon-line');
  if (line) {
    line.querySelectorAll('button.icon-line-item').forEach((el) => {
      if (!items.includes(el)) el.tabIndex = -1;
    });
  }
}
window.refreshIconLineRovingTabindex = refreshIconLineRovingTabindex;

function ensureIconLineKbStyles() {
  if (document.getElementById('mac-stats-icon-line-kb-styles')) return;
  const style = document.createElement('style');
  style.id = 'mac-stats-icon-line-kb-styles';
  style.textContent = `
    .icon-line-kb-hint {
      margin: 2px 0 0;
      font-size: 11px;
      opacity: 0.72;
      width: 100%;
      flex-basis: 100%;
      text-align: center;
    }
  `;
  document.head.appendChild(style);
}

/** Soft tip under the section icon line (power-strip kb-hint parity). */
function ensureIconLineKbHint() {
  ensureIconLineKbStyles();
  const line = document.getElementById('icon-line');
  if (!line) return;
  let hint = document.getElementById('icon-line-kb-hint');
  if (!hint) {
    hint = document.createElement('div');
    hint.id = 'icon-line-kb-hint';
    hint.className = 'icon-line-kb-hint';
    hint.setAttribute('aria-hidden', 'true');
    line.appendChild(hint);
  }
  hint.textContent =
    'Tab or click an icon · ← → / h l · Home/End move · at start crosses to power strip · at end crosses to footer · Enter / Space opens section';
}

/**
 * Icon-line toolbar keyboard — click or Tab to an icon, then ←→ / h l / Home/End.
 */
function ensureIconLineKeyboard() {
  const line = document.getElementById('icon-line');
  if (!line) return;
  ensureIconLineKbHint();
  if (line.dataset.iconLineChainKbWired !== '1') {
    line.dataset.iconLineChainKbWired = '1';
    line.addEventListener(
      'keydown',
      (e) => {
        if (e.metaKey || e.ctrlKey || e.altKey) return;
        const items = getIconLineItems();
        if (!items.length) return;
        const idx = items.indexOf(document.activeElement);
        if (idx < 0) return;
        const forward =
          e.key === 'ArrowRight' ||
          e.key === 'l' ||
          e.key === 'ArrowDown' ||
          e.key === 'j';
        const back =
          e.key === 'ArrowLeft' ||
          e.key === 'h' ||
          e.key === 'ArrowUp' ||
          e.key === 'k';
        if (forward && idx === items.length - 1) {
          if (tryChainIconLineToFooterFirst()) {
            e.preventDefault();
            e.stopPropagation();
          }
        } else if (back && idx === 0) {
          if (tryChainIconLineToPowerStripLast()) {
            e.preventDefault();
            e.stopPropagation();
          }
        }
      },
      true
    );
  }
  wireToolbarKeyboard(
    line,
    () => getIconLineItems(),
    (preferred) => refreshIconLineRovingTabindex(preferred),
    null
  );
  if (!line.getAttribute('aria-label')) {
    line.setAttribute('aria-label', 'Section shortcuts');
  }
  line.dataset.iconLineKbWired = '1';
}

/** Sync icon-line highlights from persisted section open/closed state. */
function syncIconLineFromSavedSections() {
  const pairs = [
    ['icon-monitors', 'monitors_collapsed'],
    ['icon-ollama', 'ollama_collapsed'],
    ['icon-perplexity', 'perplexity_collapsed'],
    ['icon-logs', 'logs_collapsed'],
    ['icon-disk-cleanup', 'disk_cleanup_collapsed'],
    ['icon-agent-ops', 'agent_ops_collapsed'],
  ];
  for (const [iconId, key] of pairs) {
    syncSectionIcon(iconId, !getSectionCollapsed(key));
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
  ensureCpuHeaderToolbarKeyboard();
  ensureIconLineKeyboard();
  ensureFooterToolbarKeyboard();
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
  // Use setTimeout to ensure DOM is fully ready; load persisted section state first
  // (config.json) because the CPU WebView is destroyed on close.
  setTimeout(() => {
    void (async () => {
      await loadCpuUiSections();
      initIconLine();
      syncIconLineFromSavedSections();
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
      await initCpuWindowCompactPreference();
    })();
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


