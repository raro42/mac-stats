## Global Context
### mac-stats

A local AI agent for macOS: Ollama chat, Discord bot, task runner, scheduler, and MCP—all on your Mac. No cloud, no telemetry. Lives in your menu bar—CPU, GPU, RAM, disk at a glance. Real-time, minimal, there when you look. Built with Rust and Tauri.

### Install

#### DMG (recommended)
[Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.

#### Build from source:
```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```
Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

#### If macOS blocks the app:
Gatekeeper may show "damaged" or block the unsigned app—the file is fine. Right-click the DMG → **Open**, then confirm. Or after install: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

## At a Glance

* **Menu bar**: CPU, GPU, RAM, disk at a glance; click to open the details window.
* **AI chat**: Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.

## Tool Agents (what Ollama can invoke)

Whenever Ollama is asked to decide which agent to use, the app sends the **complete list of active agents**: the invocable tools below plus the **SCHEDULER** (informational; Ollama can recommend it for recurring or delayed tasks but cannot invoke it with a tool line). Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

## Data Poster Charts - Backend Requirements

### Overview
The data-poster theme now uses bar graphs and line charts instead of ring gauges. The frontend implementation is complete and works with the current backend data, but there are potential optimizations that could be made.

### Current Status
✅ **Frontend is fully functional** - The charts work with the existing `get_cpu_details()` API response.

### Current Data Usage
The frontend currently uses:
- `data.temperature` - For temperature charts (updated every refresh cycle)
- `data.usage` - For CPU usage charts (updated every refresh cycle)  
- `data.frequency` - For frequency charts (updated every refresh cycle)

### Potential Backend Optimizations (Optional)

#### 1. Historical Data Buffer
* Current: Frontend maintains its own rolling buffer of values from each refresh call.
* Potential Enhancement: Backend could maintain a shared buffer and return the last N values in the API response:
```rust
pub struct CpuDetails {
    // ... existing fields ...
    
    // Optional: Historical data for charts
    pub temperature_history: Option<Vec<f64>>,  // Last 60 values
    pub usage_history: Option<Vec<f64>>,        // Last 60 values
    pub frequency_history: Option<Vec<f64>>,    // Last 60 values
}
```

#### 2. Chart-Specific Refresh Rate
* Current: All metrics refresh at the same rate (1 second when window is visible).
* Potential Enhancement: Different refresh rates for different metrics:
- Temperature: 2-3 seconds (changes slowly)
- Usage: 1 second (needs to be responsive)
- Frequency: 1 second (can change quickly)

#### 3. Data Smoothing
* Current: Frontend displays raw values.
* Potential Enhancement: Backend could apply moving average or exponential smoothing to reduce noise in charts.

## Open tasks:
- ~~Investigate why the frontend is not utilizing the historical data buffer effectively.~~ **Done:** Root cause: the Data Poster theme (and others) had history-section canvases (`temperature-history-chart`, etc.) but did not load `history.js`. The backend exposes `get_metrics_history` (adaptive tiered buffer in `metrics/history.rs`); `history.js` calls it and draws the history charts. Fix: Data Poster theme now loads `../../history.js` in `themes/data-poster/cpu.html`, so the history section uses the backend buffer. Real-time bar/line charts still use `poster-charts.js` + frontend buffer from `get_cpu_details()` (unchanged). See 006-feature-coder/FEATURE-CODER.md.
- ~~Implement chart-specific refresh rates for each metric.~~ **Done:** Temperature updates every 3s (DOM + ring + theme charts in `cpu.js`; history chart redraw in `history.js`); usage and frequency stay at 1s. See 006-feature-coder/FEATURE-CODER.md.
- ~~Consider adding data smoothing to reduce noise in charts.~~ **Done:** Frontend moving average (window 5) in Data Poster theme `poster-charts.js`; bar and line charts use smoothed series for display only (raw values still drive scale). See 006-feature-coder/FEATURE-CODER.md.
- ~~Review and refactor the `get_cpu_details()` API response to improve performance and consistency.~~ **Done:** API contract documented below; struct doc comment added in `metrics/mod.rs`. Future refactors can use this as the single reference.

### get_cpu_details() API contract

**Definition:** `src-tauri/src/metrics/mod.rs` — `get_cpu_details() -> CpuDetails`. Rate-limited; returns cached values when called too frequently (see `state.rs` rate limiter).

| Field | Type | Semantics | Consumers |
|-------|------|-----------|-----------|
| `usage` | f32 | CPU usage 0–100 (%) | CPU window (ring, charts), dashboard |
| `temperature` | f32 | °C; 0 if unreadable | CPU window, Data Poster / history charts |
| `frequency` | f32 | GHz; combined or P-core | CPU window, charts |
| `p_core_frequency` | f32 | GHz; 0 if N/A | CPU window (frequency subtext) |
| `e_core_frequency` | f32 | GHz; 0 if N/A | CPU window (frequency subtext) |
| `cpu_power` | f32 | Watts; 0 if N/A | CPU window, dashboard, alerts, history buffer |
| `gpu_power` | f32 | Watts; 0 if N/A | CPU window, dashboard, history buffer |
| `load_1`, `load_5`, `load_15` | f64 | Load averages | CPU window (load display) |
| `uptime_secs` | u64 | System uptime seconds | CPU window (chip/uptime) |
| `top_processes` | Vec&lt;ProcessUsage&gt; | Top N by CPU; cached ~30s | CPU window (process list), alerts |
| `chip_info` | String | e.g. "Apple M3 · 16 cores" | CPU window |
| `can_read_temperature` | bool | Whether SMC/IOReport temp is available | CPU window (hints, chart visibility) |
| `can_read_frequency` | bool | Whether IOReport freq is available | CPU window (hints) |
| `can_read_cpu_power` | bool | Whether power read succeeded | CPU window (power section visibility) |
| `can_read_gpu_power` | bool | Whether power read succeeded | CPU window (power section visibility) |
| `battery_level` | f32 | 0–100 or **-1.0** if no battery | CPU window, dashboard, alerts (BatteryLow) |
| `is_charging` | bool | True if charging | CPU window, dashboard |
| `has_battery` | bool | True if device has battery | CPU window, dashboard, alerts |

**Consistency notes:** `battery_level` uses -1.0 for “not available”; power and frequency use 0. All `can_read_*` flags reflect capability/access, not just “value &gt; 0”. For historical data (e.g. Data Poster), the frontend uses `get_metrics_history` (separate API) for history; `get_cpu_details()` is the real-time snapshot only.