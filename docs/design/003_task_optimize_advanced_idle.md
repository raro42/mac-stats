# mac-stats

## Install

**DMG (recommended):** [Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.

**Build from source:**
```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```
Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

**If macOS blocks the app:** Gatekeeper may show "damaged" or block the unsigned app—the file is fine. Right-click the DMG → **Open**, then confirm. Or after install: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

## At a Glance

- **Menu bar** — CPU, GPU, RAM, disk at a glance; click to open the details window.
- **AI chat** — Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.
- **Discord**

## 1. Tool agents (what Ollama can invoke)

Whenever Ollama is asked to decide which agent to use (planning step in Discord and scheduler flow), the app sends the **complete list of active agents**: the invocable tools below plus the **SCHEDULER** (informational; Ollama can recommend it for recurring or delayed tasks but cannot invoke it with a tool line). Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

## 2. Task 1: Detect Menu Bar Focus/Hover State

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

## 3. Task 2: Reduce Updates When CPU Window is Closed

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

## 4. Task 3: Progressive Backoff for Extended Idle

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
            let metrics = get_metrics();  // Uses cached values, very fast
            // ... update all ...
        },
        1 => {
            // Reduced: skip process list update
            let metrics = get_metrics();  // Uses cached values, very fast
            // Update menu bar only, skip processes
        },
        2 => {
            // Minimal: only update CPU/RAM, skip everything else
            let metrics = get_metrics();  // Uses cached values, very fast
            let text = build_status_text(&metrics);
            if let Ok(mut pending) = MENU_BAR_TEXT.lock() {
                *pending = Some(text);
            }
            continue;  // Skip rest of loop
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

## 5. Task 4: Battery Mode Detection - Lower Updates on Battery

**Goal**: Detect when Mac is on battery power and reduce update frequency

**Concept**:
- Monitor if mac-stats window is frontmost app
- When another app is active → reduce update frequency to 10-30 seconds
- When mac-stats is active → resume normal updates

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
            if last_check.elapsed().as_millis() < 1000 {
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

## Open tasks:

- Task 1: Hover detection (requires native Obj-C)
- Task 5: App focus detection (requires native Obj-C)
- Potential issues with battery detection and focus detection
- Further optimization of progressive backoff