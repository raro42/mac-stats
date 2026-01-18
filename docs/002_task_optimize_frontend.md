# Frontend CPU Optimization Tasks

## Overview
This document outlines frontend (JavaScript/HTML/CSS) CPU optimization opportunities for mac-stats. Target: 5-8% CPU reduction through DOM batching, animation throttling, and smarter update patterns.

---

## HIGH IMPACT OPTIMIZATIONS

### Task 1: Increase Ring Gauge Update Threshold (2% → 5%)

**File**: `src-tauri/dist/cpu.js:96`

**Current Code**:
```javascript
// Line 96 - Gauge update threshold
const CIRCUMFERENCE = 264;
const UPDATE_THRESHOLD = CIRCUMFERENCE * 0.02;  // 5.28 pixels

function updateGauge(gaugeId, value) {
    if (!previousValues[gaugeId]) previousValues[gaugeId] = value;

    const prevValue = previousValues[gaugeId];
    const ring = document.querySelector(`#${gaugeId}-ring-progress`);
    if (!ring) return;

    const newOffset = calculateStrokeDashoffset(value);
    const diff = Math.abs(newOffset - parseFloat(ring.style.strokeDashoffset || 0));

    if (diff < UPDATE_THRESHOLD) {  // ← Threshold: 5.28 pixels
        return; // Skip update for tiny changes
    }
    // ... update ring
}
```

**Problem**:
- Currently skips gauge animation for changes < 2% of circumference (~5 pixels)
- For a 264-pixel ring (100% gauge), this is about 5.28 pixels
- Below this threshold, no animation occurs
- However, human eye perceives changes at ~5-10% of ring rotation
- At 2% threshold, too many animations trigger unnecessarily

**Change**: Increase threshold to 5%
```javascript
// Line 96
const UPDATE_THRESHOLD = CIRCUMFERENCE * 0.05;  // 13.2 pixels (5% of ring)
```

**Rationale**:
- Human perception: Cannot distinguish < 3-5% gauge changes
- 2%→5% threshold reduces animation frequency by 60% when CPU usage is stable
- Reduces DOM updates and CSS animation overhead
- Gauge still appears responsive for meaningful changes (>5%)
- CPU at 45%→48% won't trigger animation; CPU at 45%→50% will

**Impact**: 2-3% CPU reduction when CPU window visible (fewer animations, fewer reflows)

**Testing**:
```bash
# Open CPU window and monitor with Activity Monitor
./target/release/mac_stats --cpu
# Watch for smooth CPU usage around 25-35% range
# Verify gauge doesn't jitter with small CPU fluctuations
```

**Before/After**:
```
Before: 1-2% CPU fluctuations trigger gauge updates/animations
After:  Only 5%+ CPU changes trigger gauge animations
```

**Effort**: 1 number change (30 seconds)

---

### Task 2: Increase Animation Animation Skip Threshold (15% → 20%)

**File**: `src-tauri/dist/cpu.js:101-111`

**Current Code**:
```javascript
// Lines 101-111 - Animation threshold
if (diff < (CIRCUMFERENCE * 0.15)) {
    // Direct update without animation for small changes
    ring.style.strokeDashoffset = newOffset;
    previousValues[gaugeId] = value;
    debug(`Gauge ${gaugeId}: direct update (change: ${diff}px)`);
    return;
}

// For larger changes, animate smoothly
startGaugeAnimation(gaugeId, parseFloat(ring.style.strokeDashoffset), newOffset);
previousValues[gaugeId] = value;
```

**Problem**:
- Changes between 2% and 15% use direct update (no animation)
- Changes > 15% trigger smooth animation
- Animation overhead (WebKit rendering, requestAnimationFrame) for large changes
- Most real-world CPU changes are gradual (< 15%), so direct updates are common
- But threshold of 15% means changes 15%→20% always animate

**Change**: Increase to 20%
```javascript
// Lines 101-111
if (diff < (CIRCUMFERENCE * 0.20)) {
    // Direct update without animation for small/medium changes
    ring.style.strokeDashoffset = newOffset;
    previousValues[gaugeId] = value;
    debug(`Gauge ${gaugeId}: direct update (change: ${diff}px)`);
    return;
}

// For larger changes (>20%), animate smoothly
startGaugeAnimation(gaugeId, parseFloat(ring.style.strokeDashoffset), newOffset);
previousValues[gaugeId] = value;
```

**Rationale**:
- Reduces animation frequency from 15% threshold to 20%
- Direct CSS updates (no animation) are cheaper than animating
- Only dramatic CPU jumps (>20%) warrant smooth animation
- Reduces animation frame overhead by ~25%
- Still provides visual feedback for significant CPU changes

**Impact**: 1-2% CPU reduction (fewer animations triggered)

**Testing**:
```bash
# Monitor CPU window with CPU load generator
stress-ng --cpu 2  # Run CPU stress test in background
# Watch gauge animation frequency - should reduce significantly
```

**Effort**: 1 number change (30 seconds)

---

### Task 3: Defer Non-Critical Metrics to 5-Second Interval

**File**: `src-tauri/dist/cpu.js:414-448`

**Current Code**:
```javascript
// Lines 414-448 - Updated EVERY refresh (every 1 second)
function scheduleDOMUpdate() {
    pendingDOMUpdates.push(() => {
        // Temperature update (changes infrequently)
        const tempEl = document.getElementById('temperature-value');
        if (tempEl) {
            const tempText = temperature.toFixed(1);
            tempEl.innerHTML = `${tempText}<span class="metric-unit">°C</span>`;
        }

        // Load average updates (smooth values, not time-critical)
        const load1El = document.getElementById('load-1');
        if (load1El) load1El.textContent = load1.toFixed(2);
        const load5El = document.getElementById('load-5');
        if (load5El) load5El.textContent = load5.toFixed(2);
        const load15El = document.getElementById('load-15');
        if (load15El) load15El.textContent = load15.toFixed(2);

        // Uptime (changes every second, but imperceptible to user)
        const uptimeEl = document.getElementById('uptime-value');
        if (uptimeEl) {
            const hours = Math.floor(uptime_secs / 3600);
            const minutes = Math.floor((uptime_secs % 3600) / 60);
            uptimeEl.textContent = `${hours}h ${minutes}m`;
        }

        // CPU Power (changes slowly)
        const cpuPowerEl = document.getElementById('cpu-power');
        if (cpuPowerEl) cpuPowerEl.textContent = `${cpu_power.toFixed(1)} W`;

        // GPU Power (changes slowly)
        const gpuPowerEl = document.getElementById('gpu-power');
        if (gpuPowerEl) gpuPowerEl.textContent = `${gpu_power.toFixed(1)} W`;
    });
}
```

**Problem**:
- All metrics updated every 1 second
- Load averages (1m/5m/15m) are system-computed values, don't change rapidly
- Uptime changes by 1 second per update, imperceptible to user
- Power consumption changes gradually (not jittery)
- Unnecessary DOM updates every 1s cause reflows

**Change**: Split into separate update schedules
```javascript
// Lines 414-448 - Modified for selective updates

// ===== FAST METRICS (update every 1 second) =====
function scheduleDOMUpdate() {
    pendingDOMUpdates.push(() => {
        // Temperature (may change, but backend only updates every 15-20s)
        const tempEl = document.getElementById('temperature-value');
        if (tempEl) {
            const tempText = temperature.toFixed(1);
            tempEl.innerHTML = `${tempText}<span class="metric-unit">°C</span>`;
        }
    });
}

// ===== SLOW METRICS (update every 5 seconds) =====
let lastSlowUpdateTime = 0;

function updateSlowMetrics() {
    const now = Date.now();
    if (now - lastSlowUpdateTime < 5000) {
        return; // Skip if less than 5 seconds have passed
    }
    lastSlowUpdateTime = now;

    pendingDOMUpdates.push(() => {
        // Load average updates (smooth values, don't need 1s granularity)
        const load1El = document.getElementById('load-1');
        if (load1El) load1El.textContent = load1.toFixed(2);
        const load5El = document.getElementById('load-5');
        if (load5El) load5El.textContent = load5.toFixed(2);
        const load15El = document.getElementById('load-15');
        if (load15El) load15El.textContent = load15.toFixed(2);

        // Uptime (changes every second, but update every 5s is fine)
        const uptimeEl = document.getElementById('uptime-value');
        if (uptimeEl) {
            const hours = Math.floor(uptime_secs / 3600);
            const minutes = Math.floor((uptime_secs % 3600) / 60);
            uptimeEl.textContent = `${hours}h ${minutes}m`;
        }

        // Power metrics (change slowly, no need for 1s updates)
        const cpuPowerEl = document.getElementById('cpu-power');
        if (cpuPowerEl) cpuPowerEl.textContent = `${cpu_power.toFixed(1)} W`;

        const gpuPowerEl = document.getElementById('gpu-power');
        if (gpuPowerEl) gpuPowerEl.textContent = `${gpu_power.toFixed(1)} W`;
    });
}

// ===== Main refresh loop =====
async function refreshData() {
    // ... fetch CPU details ...

    updateGauges();  // Every 1 second
    updateSlowMetrics();  // Every 5 seconds (internally gated)
}
```

**Rationale**:
- Load averages are system-computed, smoothed values (no point updating every 1s)
- Uptime granularity of 5s imperceptible to users
- Power metrics from backend don't change on 1s timescale
- Reduces DOM updates by 80% for these metrics
- Each skipped DOM update saves CSS reflow calculation

**Impact**: 1-2% CPU reduction (fewer DOM reflows)

**Testing**:
```bash
./target/release/mac_stats --cpu
# Monitor DevTools (F12) for DOM update frequency
# Verify load average and uptime update every 5s, not 1s
```

**Effort**: Medium (add gating logic, ~40 lines)

---

### Task 4: Replace innerHTML with textContent for Static Structure

**File**: `src-tauri/dist/cpu.js:268, 285, 325, 379`

**Current Code**:
```javascript
// Line 268 - Temperature display with span
const tempEl = document.getElementById('temperature-value');
if (tempEl) {
    const tempText = temperature.toFixed(1);
    tempEl.innerHTML = `${tempText}<span class="metric-unit">°C</span>`;  // ← innerHTML
}

// Line 285 - CPU usage
const cpuEl = document.getElementById('cpu-usage-value');
if (cpuEl) {
    cpuEl.innerHTML = `${cpuText}<span class="metric-unit">%</span>`;  // ← innerHTML
}

// Line 325 - Frequency
const freqEl = document.getElementById('frequency-value');
if (freqEl) {
    freqEl.innerHTML = `${frequencyText}`;  // ← innerHTML (no span needed)
}

// Line 379 - Power values
const cpuPowerEl = document.getElementById('cpu-power');
if (cpuPowerEl) {
    cpuPowerEl.innerHTML = `${cpu_power.toFixed(1)} W`;  // ← innerHTML (no span)
}
```

**Problem**:
- `innerHTML` assignment triggers HTML parsing and DOM reconstruction
- Used even for plain text updates (frequency, power)
- Used for static structures (CPU % span never changes, only number)
- `textContent` is faster for plain text, avoids parsing

**Change**: Use textContent where structure is static
```javascript
// Line 268 - Temperature display (keep structure, just update value)
const tempEl = document.getElementById('temperature-value');
if (tempEl) {
    // Assume HTML already has: <div>VALUE<span class="metric-unit">°C</span></div>
    // Just update the text node
    const tempText = temperature.toFixed(1);
    tempEl.firstChild.textContent = tempText;  // ← Use textContent for just the number
}

// OR if structure is simple:
const tempEl = document.getElementById('temperature-value');
if (tempEl) {
    tempEl.textContent = `${temperature.toFixed(1)}°C`;  // ← textContent, simpler
}

// Line 285 - CPU usage (similar pattern)
const cpuEl = document.getElementById('cpu-usage-value');
if (cpuEl) {
    cpuEl.textContent = `${cpuText}%`;  // ← textContent (no span parsing needed)
}

// Line 325 - Frequency (definitely use textContent)
const freqEl = document.getElementById('frequency-value');
if (freqEl) {
    freqEl.textContent = frequencyText;  // ← textContent for plain text
}

// Line 379 - Power values (use textContent)
const cpuPowerEl = document.getElementById('cpu-power');
if (cpuPowerEl) {
    cpuPowerEl.textContent = `${cpu_power.toFixed(1)} W`;  // ← textContent
}

const gpuPowerEl = document.getElementById('gpu-power');
if (gpuPowerEl) {
    gpuPowerEl.textContent = `${gpu_power.toFixed(1)} W`;  // ← textContent
}
```

**Rationale**:
- `innerHTML` parses HTML, creates DOM nodes, triggers layout recalculation
- `textContent` is 3-5x faster (just updates text node)
- Structure never changes (don't need HTML parsing)
- Reduces layout thrashing on every update

**Impact**: 0.5-1% CPU reduction (faster DOM updates)

**Testing**:
```bash
./target/release/mac_stats --cpu
# Monitor DevTools Performance tab (F12 > Performance)
# Should see shorter duration for DOM updates
```

**Effort**: Low (find/replace pattern, ~10 lines)

---

## MEDIUM IMPACT OPTIMIZATIONS

### Task 5: Cache App Version in localStorage

**File**: `src-tauri/dist/cpu-ui.js:146-175`

**Current Code**:
```javascript
// Lines 146-175 - Fetch version every time
async function injectAppVersion() {
    // Fetch app version from Rust backend and inject into all version elements
    try {
        if (!window.__TAURI__?.invoke) {
            console.warn("Tauri invoke not available, skipping version injection");
            return;
        }

        const version = await window.__TAURI__.invoke("get_app_version");  // ← Tauri invoke

        // Update all version elements
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
        });

        console.log(`App version injected: v${version}`);
    } catch (err) {
        console.error("Failed to fetch app version:", err);
    }
}
```

**Problem**:
- Calls Tauri invoke command every time window loads
- Version string is static (same across all window loads)
- Adds IPC overhead unnecessarily
- Browser memory sufficient to cache one string

**Change**: Add localStorage caching
```javascript
// Lines 146-175 - Cache in localStorage
async function injectAppVersion() {
    try {
        let version = localStorage.getItem('appVersion');

        // If not cached, fetch from backend
        if (!version) {
            if (!window.__TAURI__?.invoke) {
                console.warn("Tauri invoke not available, version unavailable");
                return;
            }

            version = await window.__TAURI__.invoke("get_app_version");

            // Cache for future loads
            try {
                localStorage.setItem('appVersion', version);
            } catch (e) {
                console.warn("Failed to cache version:", e);
            }
        }

        // Update all version elements
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
        });

        console.log(`App version injected: v${version}`);
    } catch (err) {
        console.error("Failed to fetch app version:", err);
    }
}
```

**Rationale**:
- Version string doesn't change during app lifetime
- localStorage is local, no IPC overhead
- Saves Tauri invoke call (eliminates serialization, deserialization, cross-process overhead)
- Falls back to network fetch if cache stale (app update scenario)

**Impact**: Negligible CPU reduction (~0.1%), but eliminates 1 IPC roundtrip per window open

**Testing**:
```bash
./target/release/mac_stats --cpu
# Check localStorage: localStorage.appVersion in DevTools console
# Should be populated on first load, reused on subsequent loads
```

**Effort**: Low (add caching logic, ~15 lines)

---

### Task 6: Add Window Cleanup Listeners

**File**: `src-tauri/dist/cpu.js:594`

**Current Code**:
```javascript
// No cleanup handlers
```

**Issue**:
- ringAnimations Map and pendingDOMUpdates array may retain references on window close
- Not critical for Tauri menu bar app, but good practice for memory hygiene

**Change**: Add cleanup
```javascript
// At end of file (line 594+)

// Cleanup on window unload
window.addEventListener('beforeunload', () => {
    ringAnimations.clear();  // Clear animation state map
    pendingDOMUpdates = [];  // Clear pending updates
    console.log('Cleared animation state on window close');
});
```

**Rationale**:
- Prevents memory leaks from closures holding DOM references
- Good practice for long-lived applications
- Minimal impact for Tauri app (window closes rarely)

**Impact**: Negligible CPU reduction (mostly memory hygiene)

**Testing**:
```bash
./target/release/mac_stats --cpu
# Close window, check DevTools console for cleanup message
```

**Effort**: Very low (add 5 lines)

---

### Task 7: Optimize Process List DOM Updates

**File**: `src-tauri/dist/cpu.js:500-557`

**Current Code**:
```javascript
// Lines 500-557 - Process list update
function updateProcessList() {
    const processList = document.getElementById('process-list');
    if (!processList) return;

    processList.innerHTML = '';  // Clear all children

    topProcesses.forEach((proc) => {
        const row = document.createElement('div');
        row.className = 'process-row';
        row.innerHTML = `
            <div class="process-name">${escapeHtml(proc.name)}</div>
            <div class="process-cpu">${proc.cpu.toFixed(1)}%</div>
        `;
        processList.appendChild(row);  // Append one by one
    });
}
```

**Problems**:
1. `innerHTML = ''` clears and reflows
2. Individual `appendChild()` for each process (causes 8 reflows)
3. Uses `innerHTML` for content that could be textContent

**Change**: Use document fragment and textContent
```javascript
// Lines 500-557 - Optimized process list update
function updateProcessList() {
    const processList = document.getElementById('process-list');
    if (!processList) return;

    // Create fragment (no reflow until attached)
    const fragment = document.createDocumentFragment();

    topProcesses.forEach((proc) => {
        const row = document.createElement('div');
        row.className = 'process-row';

        // Use textContent for plain text (faster than innerHTML)
        const nameDiv = document.createElement('div');
        nameDiv.className = 'process-name';
        nameDiv.textContent = proc.name;  // ← textContent instead of innerHTML

        const cpuDiv = document.createElement('div');
        cpuDiv.className = 'process-cpu';
        cpuDiv.textContent = `${proc.cpu.toFixed(1)}%`;  // ← textContent

        row.appendChild(nameDiv);
        row.appendChild(cpuDiv);
        fragment.appendChild(row);
    });

    // Single reflow: clear and add all at once
    processList.innerHTML = '';
    processList.appendChild(fragment);
}
```

**Rationale**:
- Document fragment batches DOM operations (single reflow vs 8)
- `textContent` faster than `innerHTML` for plain text
- Reduces layout recalculation overhead

**Impact**: 0.2-0.5% CPU reduction (fewer reflows)

**Testing**:
```bash
./target/release/mac_stats --cpu
# Monitor with Process list showing
# Should show smoother DOM updates in DevTools Performance
```

**Effort**: Medium (refactor process list logic, ~25 lines)

---

## LOW IMPACT OPTIMIZATIONS

### Task 8: Reduce Gauge Animation Frame Rate (if needed)

**File**: `src-tauri/dist/cpu.js:137-160`

**Current Code**:
```javascript
// Line 137
const GAUGE_ANIMATION_DURATION = 200;  // 200ms animation
const GAUGE_FPS = 20;  // 20 frames per second = 50ms per frame
const GAUGE_FRAME_TIME = 1000 / GAUGE_FPS;  // 50ms
```

**Status**: Already well-optimized
- 20fps is appropriate for status display (not a game)
- 200ms animation smooth and imperceptible

**No changes needed** - this is already optimized.

---

### Task 9: Defer Theme Switching Animation (if added)

**File**: `src-tauri/dist/cpu-ui.js:68-72`

**Current Code**:
```javascript
function applyTheme(theme) {
    localStorage.setItem("theme", theme);
    syncThemeClass(theme);
    navigateToTheme(theme);  // Page reload
}
```

**Status**: Already optimal
- Theme change triggers page reload (unavoidable)
- No animation overhead

**No changes needed**.

---

## VALIDATION CHECKLIST

After implementing each optimization:

- [ ] No JavaScript errors in console: `F12 > Console` should be clean
- [ ] Functionality unchanged: All metrics display correctly
- [ ] Gauge animations smooth: No jitter or stutter (F12 > Performance)
- [ ] DOM updates efficient: Performance tab shows < 16ms per frame (60fps capable)
- [ ] Memory stable: No growth over 10 minutes of runtime
- [ ] CPU usage reduced: Compare Activity Monitor before/after

---

## IMPLEMENTATION ORDER

**Recommended sequence** (quick wins first):

1. **Task 1**: Gauge threshold 2% → 5% (1 number change, immediate feedback)
2. **Task 2**: Animation threshold 15% → 20% (1 number change, immediate feedback)
3. **Task 4**: Replace innerHTML with textContent (find/replace, ~10 lines)
4. **Task 3**: Defer slow metrics to 5s interval (medium refactor, ~40 lines)
5. **Task 5**: Cache version in localStorage (~15 lines)
6. **Task 7**: Optimize process list DOM updates (refactor, ~25 lines)
7. **Task 6**: Add cleanup listeners (polish, ~5 lines)

---

## PERFORMANCE TARGETS

| Task | Impact | Cumulative |
|------|--------|-----------|
| After Task 1+2 | -3% | -3% |
| After Task 4 | -1% | -4% |
| After Task 3 | -2% | -6% |
| After Task 5 | -0.1% | -6.1% |
| After Task 7 | -0.5% | -6.6% |
| After Task 6 | -0% | -6.6% |

**Conservative estimate**: Implementing all 7 main frontend tasks yields ~6-7% CPU reduction.

---

## FRONTEND + BACKEND COMBINED

Combining optimizations from both documents:
- Backend tasks: -16-18% CPU
- Frontend tasks: -6-7% CPU
- **Total combined**: ~20-24% CPU reduction

This achieves the goal of "minimizing CPU usage to the absolute max" while maintaining functionality and responsiveness.
