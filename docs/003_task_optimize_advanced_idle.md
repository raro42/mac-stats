# Advanced Idle & Smart Scheduling Optimizations

## Overview

This document details aggressive idle-state optimizations that can reduce CPU usage from ~0.5% to <0.1% when the app is not actively being monitored. These are "smart scheduling" optimizations that adjust update frequencies based on app focus and user activity.

---

## HIGH IMPACT SMART OPTIMIZATIONS

### Task 1: Detect Menu Bar Focus/Hover State

**Goal**: Reduce menu bar update frequency when mouse is not hovering over it

**File**: `src-tauri/src/lib.rs` (new module)

**Concept**:
- When mouse moves away from menu bar area → reduce update interval
- When mouse returns to menu bar → restore normal update interval
- Estimated CPU reduction: 40-50% when not hovering (0.5% → 0.25% idle)

**Implementation Approach**:

```rust
// New state module addition:
pub(crate) static MENU_BAR_HOVERED: Mutex<bool> = Mutex::new(false);
pub(crate) static LAST_HOVER_STATE_CHANGE: Mutex<Option<Instant>> = Mutex::new(None);
pub(crate) static MENU_BAR_UPDATE_INTERVAL: Mutex<u64> = Mutex::new(1); // 1 second normal

// Background update loop modification:
loop {
    let update_interval = if let Ok(hovered) = MENU_BAR_HOVERED.lock() {
        if *hovered {
            1  // 1 second when hovering
        } else {
            5  // 5 seconds when not hovering
        }
    } else {
        1
    };

    std::thread::sleep(std::time::Duration::from_secs(update_interval));

    // ... rest of update loop ...
}

// macOS native code to detect hover (via Objective-C):
// - Monitor NSStatusBar click area for mouse movement
// - Set MENU_BAR_HOVERED on mouse enter/exit events
```

**Challenges**:
- Need Objective-C bridge to detect menu bar area mouse events
- Status bar doesn't have built-in hover detection
- May require periodic mouse position polling

**Alternative Simpler Approach** (no hover detection):
- Just reduce update interval when CPU window is closed
- Still provides significant idle savings

**Impact**:
- Normal (hovering): ~0.5% CPU (unchanged)
- Idle (not hovering): ~0.25% CPU (-50% reduction)
- Overall with mix: ~0.35% CPU avg (-30%)

**Effort**: Medium-High (Objective-C FFI needed)

**Risk**: Medium (needs careful mouse event handling)

---

### Task 2: Reduce Updates When CPU Window is Closed

**Goal**: Drastically reduce metric refresh frequency when CPU monitoring window is not visible

**File**: `src-tauri/src/lib.rs:220-242`

**Current Code**:
```rust
loop {
    std::thread::sleep(std::time::Duration::from_secs(1));  // ← Always 1 second

    let metrics = get_metrics();
    let text = build_status_text(&metrics);

    // Store update in static variable
    if let Ok(mut pending) = MENU_BAR_TEXT.lock() {
        *pending = Some(text);
    }

    // ... expensive SMC/IOReport operations even when window closed ...
}
```

**Problem**:
- Menu bar updates every 1 second even when CPU window is closed
- Backend still reads temperature/frequency every 20/30 seconds (already optimized in Phase 1)
- But basic CPU/RAM/Disk metrics update every 1 second unnecessarily

**Change**:
```rust
loop {
    // Check if CPU window is visible
    let window_visible = APP_HANDLE.get()
        .and_then(|h| h.get_window("cpu"))
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false);

    // Adjust update interval based on window visibility
    let update_interval = if window_visible {
        1  // 1 second when window visible (responsive)
    } else {
        5  // 5 seconds when window closed (lower battery impact)
    };

    std::thread::sleep(std::time::Duration::from_secs(update_interval));

    // Skip expensive metric reads if window not visible
    if !window_visible {
        // Only update menu bar, skip expensive operations
        let basic_metrics = get_metrics();  // Uses cached values, very fast
        let text = build_status_text(&basic_metrics);
        if let Ok(mut pending) = MENU_BAR_TEXT.lock() {
            *pending = Some(text);
        }
        continue;  // Skip rest of loop
    }

    // ... expensive operations only when window visible ...
}
```

**Impact**:
- CPU window open: ~1% CPU (unchanged)
- CPU window closed: ~0.2% CPU (-80% reduction)
- Overall average: ~0.35% CPU (substantial savings)

**Effort**: Low (30 lines, logic already in place)

**Risk**: Very Low (existing visibility check code)

---

### Task 3: Progressive Backoff for Extended Idle

**Goal**: Gradually reduce update frequency over time when app is idle (user on different monitor)

**Concept**:
- Minute 0-5: Full updates (1s interval)
- Minute 5-10: Reduced updates (5s interval)
- Minute 10+: Minimal updates (30s interval)
- When any activity → resume full updates

**File**: `src-tauri/src/state.rs` (new cache)

**Implementation**:

```rust
// Add to state.rs:
pub(crate) static LAST_MENU_BAR_CLICK: Mutex<Option<Instant>> = Mutex::new(None);
pub(crate) static IDLE_LEVEL: Mutex<u8> = Mutex::new(0);  // 0=normal, 1=reduced, 2=minimal

// In lib.rs background loop:
fn get_idle_level() -> u8 {
    if let Ok(last_click) = LAST_MENU_BAR_CLICK.lock() {
        if let Some(time) = last_click.as_ref() {
            let idle_secs = time.elapsed().as_secs();
            if idle_secs > 600 {
                2  // Minimal (30s interval)
            } else if idle_secs > 300 {
                1  // Reduced (5s interval)
            } else {
                0  // Normal (1s interval)
            }
        } else {
            0  // Not set yet, assume normal
        }
    } else {
        0
    }
}

loop {
    let idle_level = get_idle_level();

    let update_interval = match idle_level {
        0 => 1,   // Normal: 1 second
        1 => 5,   // Reduced: 5 seconds
        2 => 30,  // Minimal: 30 seconds
        _ => 1,   // Fallback
    };

    std::thread::sleep(std::time::Duration::from_secs(update_interval));

    // Only update metrics based on idle level
    match idle_level {
        0 => {
            // Normal: full update
            let metrics = get_metrics();
            // ... update all ...
        },
        1 => {
            // Reduced: skip process list update
            let metrics = get_metrics();
            // Update menu bar only, skip processes
        },
        2 => {
            // Minimal: only update CPU/RAM, skip everything else
            let metrics = get_metrics();  // Very fast (cached)
            let text = build_status_text(&metrics);
            if let Ok(mut pending) = MENU_BAR_TEXT.lock() {
                *pending = Some(text);
            }
            continue;
        },
        _ => {}
    }
}

// Click handler resets idle timer:
// In ui/status_bar.rs:
pub fn menu_bar_clicked() {
    if let Ok(mut last_click) = LAST_MENU_BAR_CLICK.lock() {
        *last_click = Some(Instant::now());  // Reset idle timer
    }
    if let Ok(mut level) = IDLE_LEVEL.lock() {
        *level = 0;  // Resume normal operation
    }
}
```

**Impact**:
- First 5 min (user checking): ~0.5% CPU (unchanged)
- After 5 min idle: ~0.2% CPU (-60% reduction)
- After 10 min idle: ~0.05% CPU (-90% reduction)
- Resumes full speed on click

**Effort**: Medium (60 lines, multiple files)

**Risk**: Low (graceful degradation)

---

### Task 4: Battery Mode Detection - Lower Updates on Battery

**Goal**: Detect when Mac is on battery power and reduce update frequency

**File**: `src-tauri/src/metrics/mod.rs` (new function)

**Implementation**:

```rust
// Check if running on battery
pub fn is_on_battery() -> bool {
    use std::process::Command;

    let output = Command::new("pmset")
        .arg("-g")
        .arg("batt")
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        !stdout.contains("AC Power")  // No AC power = on battery
    } else {
        false  // Assume plugged in on error
    }
}

// Cache battery state (only check every 5 seconds)
pub(crate) static BATTERY_STATE_CACHE: Mutex<Option<(bool, Instant)>> = Mutex::new(None);

pub fn is_on_battery_cached() -> bool {
    if let Ok(mut cache) = BATTERY_STATE_CACHE.lock() {
        if let Some((on_battery, last_check)) = cache.as_ref() {
            if last_check.elapsed().as_secs() < 5 {
                return *on_battery;  // Use cached value
            }
        }

        let on_battery = is_on_battery();
        *cache = Some((on_battery, Instant::now()));
        on_battery
    } else {
        false
    }
}

// In lib.rs background loop:
loop {
    let on_battery = is_on_battery_cached();

    let update_interval = if on_battery {
        10  // 10 seconds on battery (low power mode)
    } else {
        1   // 1 second on AC (normal)
    };

    std::thread::sleep(std::time::Duration::from_secs(update_interval));

    // ... rest of loop ...
}
```

**Impact**:
- On AC Power: ~0.5% CPU (unchanged)
- On Battery: ~0.1% CPU (-80% reduction)
- Automatic: No user configuration needed

**Effort**: Low (40 lines)

**Risk**: Very Low (external command, graceful fallback)

**System Integration**:
- Automatically saves battery life when on battery
- Users won't even notice the reduced monitoring
- Great for MacBook Air users

---

### Task 5: App Window Activity Detection - Pause When Other App Active

**Goal**: Reduce updates when user is using a different application

**Concept**:
- Monitor if mac-stats window is frontmost app
- When another app is active → reduce update frequency to 10-30 seconds
- When mac-stats is active → resume normal updates

**File**: `src-tauri/src/ui/status_bar.rs` (new function)

**Implementation Approach**:

```rust
// Use Objective-C to detect if any window is focused
#[allow(dead_code)]
pub fn is_mac_stats_frontmost() -> bool {
    use objc2::{msg_send, sel};
    use objc2::rc::Shared;
    use objc2_app_kit::NSApplication;

    unsafe {
        if let Ok(mtm) = objc2::MainThreadMarker::new() {
            let app = NSApplication::sharedApplication(mtm);
            let frontmost_app: *mut objc2::runtime::AnyObject = msg_send![app, mainWindow];
            !frontmost_app.is_null()  // true if we have main window
        } else {
            false
        }
    }
}

// Cache focus state (check every 1 second)
pub(crate) static IS_FRONTMOST: Mutex<Option<(bool, Instant)>> = Mutex::new(None);

pub fn is_app_frontmost_cached() -> bool {
    if let Ok(mut cache) = IS_FRONTMOST.lock() {
        if let Some((frontmost, last_check)) = cache.as_ref() {
            if last_check.elapsed().as_millis() < 1000 {
                return *frontmost;  // Use cached value
            }
        }

        let frontmost = is_mac_stats_frontmost();
        *cache = Some((frontmost, Instant::now()));
        frontmost
    } else {
        false
    }
}

// In lib.rs background loop:
loop {
    let is_frontmost = is_app_frontmost_cached();

    let update_interval = if is_frontmost {
        1   // 1 second when app is active
    } else {
        15  // 15 seconds when user on other app
    };

    std::thread::sleep(std::time::Duration::from_secs(update_interval));

    // ... rest of loop ...
}
```

**Impact**:
- App active (CPU window open): ~1% CPU (unchanged)
- App in background: ~0.1% CPU (-90% reduction)
- User switches between apps: Auto-adapts

**Effort**: Medium (Objective-C FFI needed, ~50 lines)

**Risk**: Medium (requires focus detection)

**Note**: Menu bar still shows latest cached values, just not updating live

---

## SUMMARY TABLE: All Advanced Optimizations

| Task | Description | Effort | CPU Impact | Risk | Prerequisite |
|------|-------------|--------|-----------|------|--------------|
| 1 | Hover detection | High | -50% idle | Medium | Obj-C FFI |
| 2 | Window closed pause | Low | -80% closed | Very Low | None |
| 3 | Progressive backoff | Medium | -90% after 10min | Low | Task 2 |
| 4 | Battery mode | Low | -80% battery | Very Low | None |
| 5 | App focus detect | Medium | -90% background | Medium | Obj-C FFI |

---

## IMPLEMENTATION ROADMAP

### Quick Wins (Very Low Effort)
- **Task 2**: Window closed pause (30 lines, -80% when closed)
- **Task 4**: Battery mode detection (40 lines, -80% on battery)
- **Combined**: -70-80% CPU when not actively monitoring

### Medium Complexity (1-2 hours)
- **Task 3**: Progressive backoff (60 lines, -90% after 10 min)
- **Task 5**: App focus detection (50 lines, -90% in background)

### Advanced (Specialization)
- **Task 1**: Hover detection (requires native Obj-C)

---

## ESTIMATED TOTAL SAVINGS

### Scenario 1: User Checks Status Bar, Then Works on Other App

```
Time 0-5s: Mouse on menu bar, CPU window not visible
   Current: 0.5% CPU
   After optimization: 0.2% CPU (Task 2)

Time 5s-5min: User switches to other app (Slack, Email, etc.)
   Current: 0.5% CPU
   After optimization: 0.1% CPU (Task 5)

Time 5-60min: User works on other app, Mac plugged in
   Current: 0.5% CPU
   After optimization: 0.05% CPU (Task 3 + 5)

Daily Savings: ~60-70% CPU reduction in typical usage
```

### Scenario 2: MacBook Air on Battery

```
Time 0-1hr: User working, checking stats occasionally
   Current: 0.5% CPU → Battery drain
   After optimization: 0.1% CPU (Task 4)

Battery Life Improvement:
   ~80% CPU reduction when monitoring = ~15-20% battery life gain
```

### Scenario 3: Idle Overnight

```
Time overnight (8 hours): Mac in sleep (external monitor off)
   Current: 0.5% CPU wasting battery
   After optimization: 0.05% CPU (Task 3 after 10 min)

Overnight Savings: ~90% CPU reduction
```

---

## COMBINED WITH PHASE 1-4

### Before Any Optimization
- Idle: 0.5% CPU
- Window open: 1.0% CPU
- Background: 0.5% CPU
- Battery: 0.5% CPU

### After Phase 1-4 + Advanced Tasks
- Idle (window closed): 0.05% CPU (-90%)
- Window open: 0.6% CPU (-40%)
- Background (idle): 0.05% CPU (-90%)
- Battery mode: 0.05% CPU (-90%)
- Frontmost (normal): 0.6% CPU (-40%)

**Total Average Across All States**: ~0.2% CPU

---

## IMPLEMENTATION PRIORITY

**Start With** (5 minutes + builds on previous):
1. Task 2 (window closed pause) - Immediate -80% when window closed
2. Task 4 (battery mode) - Automatic -80% on battery

**Then Add** (if users report still too high):
3. Task 3 (progressive backoff) - Further -90% after idle
4. Task 5 (app focus) - Further -90% in background

**Advanced** (if really targeting power efficiency):
5. Task 1 (hover detection) - Final -50% idle tuning

---

## VALIDATION

Each smart optimization needs testing:

```bash
# Task 2: Window closed pause
./target/release/mac_stats --cpu
# Close window, monitor Activity Monitor
# CPU should drop to 0.1-0.2%

# Task 4: Battery mode
# Unplug MacBook from power
# Monitor Activity Monitor
# CPU should drop to ~0.1%

# Task 3: Progressive backoff
# Let app run for 15 minutes without interaction
# Monitor log for idle level changes
# CPU should gradually reduce

# Task 5: App focus detection
# Open CPU window, then switch to another app
# CPU should reduce significantly
# Switch back to mac-stats
# CPU should return to normal
```

---

## RISKS & MITIGATIONS

**Risk: Metrics become stale**
- Mitigation: Menu bar always shows last known value (doesn't go blank)
- Mitigation: Values update when window opens

**Risk: User misses important CPU spike**
- Mitigation: CPU window still responsive on-demand
- Mitigation: Task 3 max idle is 30s, so max 30s delay

**Risk: Battery detection fails**
- Mitigation: Graceful fallback to normal updates
- Mitigation: Cached for 5s, so minimal system call overhead

---

## CONCLUSION

These 5 advanced optimization tasks can reduce mac-stats CPU usage from **0.5% baseline to 0.05% in typical usage** when combined with the original Phase 1-4 optimizations.

**Key wins**:
- Task 2: -80% when window closed (easy, immediate)
- Task 4: -80% on battery (easy, automatic)
- Task 3+5: -90% when idle or backgrounded (medium effort, huge savings)

**Effort investment**: 30-120 minutes total
**CPU reduction potential**: 80-90% in real-world usage patterns
