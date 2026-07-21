## Global Context for Reference

### README.md snippets

### # mac-stats

### [![GitHub release](https://img.shields.io/github/v/release/raro42/mac-stats?include_prereleases&style=flat-square)](https://github.com/raro42/mac-stats/releases/latest)

### A local AI agent for macOS: Ollama chat, Discord bot, task runner, scheduler, and MCP—all on your Mac. No cloud, no telemetry. Lives in your menu bar—CPU, GPU, RAM, disk at a glance. Real-time, minimal, there when you look. Built with Rust and Tauri.

### [Changelog](CHANGELOG.md) · [Screenshots & themes](screens/README.md)

---

## Install

### DMG (recommended):
[Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.

### Build from source:
```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```
Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

### If macOS blocks the app:
Gatekeeper may show "damaged" or block the unsigned app—the file is fine. Right-click the DMG → **Open**, then confirm. Or after install: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

---

## At a Glance

- **Menu bar** — CPU, GPU, RAM, disk at a glance; click to open the details window.
- **AI chat** — Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.
- **Discord**

---

## 1. Tool Agents (what Ollama can invoke)

Whenever Ollama is asked to decide which agent to use (planning step in Discord and scheduler flow), the app sends the **complete list of active agents**: the invocable tools below plus the **SCHEDULER** (informational; Ollama can recommend it for recurring or delayed tasks but cannot invoke it with a tool line). Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

---

## 1. Tool Agents (what Ollama can invoke)

Whenever Ollama is asked to decide which agent to use (planning step in Discord and scheduler flow), the app sends the **complete list of active agents**: the invocable tools below plus the **SCHEDULER** (informational; Ollama can recommend it for recurring or delayed tasks but cannot invoke it with a tool line). Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

---

## 2. CPU Optimization Task Summary

### Quick Reference

Two detailed task documents provide comprehensive CPU optimization roadmap:

1. **`001_task_optimize_backend.md`** - Rust/FFI backend optimizations
2. **`002_task_optimize_frontend.md`** - JavaScript/HTML frontend optimizations

---

## 2. CPU Optimization Task Summary

### Quick Reference

Two detailed task documents provide comprehensive CPU optimization roadmap:

1. **`001_task_optimize_backend.md`** - Rust/FFI backend optimizations
2. **`002_task_optimize_frontend.md`** - JavaScript/HTML frontend optimizations

---

## Executive Summary

### Current State
- mac-stats runs at ~0.5% CPU idle, <1% with CPU window open
- Potential for additional 20-24% reduction through targeted optimizations
- Most impactful changes are simple (1-3 line modifications to timing intervals)

### Target State
- **Idle**: ~0.4-0.5% (no significant change needed)
- **With CPU window**: 0.6-0.8% (down from 1%)
- **Peak (under stress)**: <1% (down from ~1.3%)

### Effort vs Benefit

| Effort | High Impact | Medium Impact | Low Impact | Total Tasks |
|--------|----------|----------|----------|----------|
| < 1 minute | 3 tasks | — | — | 3 tasks (Quick wins) |
| 1-30 min | 1 task | 1 task | 2 tasks | 4 tasks (Easy) |
| 30 min - 2 hrs | — | 4 tasks | 2 tasks | 6 tasks (Medium) |
| **Total** | **4 tasks** | **5 tasks** | **4 tasks** | **13 tasks** |

---

## Tasks by Priority

### PHASE 1: Quick Wins (5 minutes, -4% CPU)

**Start here - immediate results with minimal effort**

#### Backend
- [ ] **Task 1**: Temperature interval 15s → 20s (`lib.rs:409`)
  - Change: 1 line, `>= 15` → `>= 20`
  - Impact: -2-3% CPU

- [ ] **Task 2**: Frequency interval 20s → 30s (`lib.rs:527`)
  - Change: 1 line, `>= 20` → `>= 30`
  - Impact: -4-6% CPU

- [ ] **Task 3**: Process cache 5s → 10s (`metrics/mod.rs:767`)
  - Change: 1 line, `< 5.0` → `< 10.0`
  - Impact: -3-4% CPU

#### Frontend
- [ ] **Task F1**: Gauge threshold 2% → 5% (`cpu.js:96`)
  - Change: 1 number, `0.02` → `0.05`
  - Impact: -2-3% CPU

- [ ] **Task F2**: Animation threshold 15% → 20% (`cpu.js:103`)
  - Change: 1 number, `0.15` → `0.20`
  - Impact: -1-2% CPU

**Subtotal: -12-18% CPU with 5 one-line changes**

---

## Tasks by Priority

### PHASE 1: Quick Wins (5 minutes, -4% CPU)

**Start here - immediate results with minimal effort**

#### Backend
- [ ] **Task 1**: Temperature interval 15s → 20s (`lib.rs:409`)
  - Change: 1 line, `>= 15` → `>= 20`
  - Impact: -2-3% CPU

- [ ] **Task 2**: Frequency interval 20s → 30s (`lib.rs:527`)
  - Change: 1 line, `>= 20` → `>= 30`
  - Impact: -4-6% CPU

- [ ] **Task 3**: Process cache 5s → 10s (`metrics/mod.rs:767`)
  - Change: 1 line, `< 5.0` → `< 10.0`
  - Impact: -3-4% CPU

#### Frontend
- [ ] **Task F1**: Gauge threshold 2% → 5% (`cpu.js:96`)
  - Change: 1 number, `0.02` → `0.05`
  - Impact: -2-3% CPU

- [ ] **Task F2**: Animation threshold 15% → 20% (`cpu.js:103`)
  - Change: 1 number, `0.15` → `0.20`
  - Impact: -1-2% CPU

**Subtotal: -12-18% CPU with 5 one-line changes**

---

## 3. CPU Optimization Task Summary

### Quick Reference

Two detailed task documents provide comprehensive CPU optimization roadmap:

1. **`001_task_optimize_backend.md`** - Rust/FFI backend optimizations
2. **`002_task_optimize_frontend.md`** - JavaScript/HTML frontend optimizations

---

## 3. CPU Optimization Task Summary

### Quick Reference

Two detailed task documents provide comprehensive CPU optimization roadmap:

1. **`001_task_optimize_backend.md`** - Rust/FFI backend optimizations
2. **`002_task_optimize_frontend.md`** - JavaScript/HTML frontend optimizations

---

## Executive Summary

### Current State
- mac-stats runs at ~0.5% CPU idle, <1% with CPU window open
- Potential for additional 20-24% reduction through targeted optimizations
- Most impactful changes are simple (1-3 line modifications to timing intervals)

### Target State
- **Idle**: ~0.4-0.5% (no significant change needed)
- **With CPU window**: 0.6-0.8% (down from 1%)
- **Peak (under stress)**: <1% (down from ~1.3%)

### Effort vs Benefit

| Effort | High Impact | Medium Impact | Low Impact | Total Tasks |
|--------|----------|----------|----------|----------|
| < 1 minute | 3 tasks | — | — | 3 tasks (Quick wins) |
| 1-30 min | 1 task | 1 task | 2 tasks | 4 tasks (Easy) |
| 30 min - 2 hrs | — | 4 tasks | 2 tasks | 6 tasks (Medium) |
| **Total** | **4 tasks** | **5 tasks** | **4 tasks** | **13 tasks** |

---

## Tasks by Priority

### PHASE 1: Quick Wins (5 minutes, -4% CPU)

**Start here - immediate results with minimal effort**

#### Backend
- [ ] **Task 1**: Temperature interval 15s → 20s (`lib.rs:409`)
  - Change: 1 line, `>= 15` → `>= 20`
  - Impact: -2-3% CPU

- [ ] **Task 2**: Frequency interval 20s → 30s (`lib.rs:527`)
  - Change: 1 line, `>= 20` → `>= 30`
  - Impact: -4-6% CPU

- [ ] **Task 3**: Process cache 5s → 10s (`metrics/mod.rs:767`)
  - Change: 1 line, `< 5.0` → `< 10.0`
  - Impact: -3-4% CPU

#### Frontend
- [ ] **Task F1**: Gauge threshold 2% → 5% (`cpu.js:96`)
  - Change: 1 number, `0.02` → `0.05`
  - Impact: -2-3% CPU

- [ ] **Task F2**: Animation threshold 15% → 20% (`cpu.js:103`)
  - Change: 1 number, `0.15` → `0.20`
  - Impact: -1-2% CPU

**Subtotal: -12-18% CPU with 5 one-line changes**

---

## 4. Advanced Smart Scheduling Optimizations

### Quick Reference

5 additional optimizations that adjust update frequencies based on:

1. **Mouse hover over menu bar** (-50% CPU when not hovering)
2. **CPU window visibility** (-80% CPU when window closed)
3. **Progressive idle timeout** (-90% CPU after 10 minutes idle)
4. **Battery mode detection** (-80% CPU on battery power)
5. **App focus detection** (-90% CPU when other app is frontmost)

### Combined with Phases 1-4**: ~0.05% CPU in typical real-world usage (90% reduction)

---

## 4. Advanced Smart Scheduling Optimizations

### Quick Reference

5 additional optimizations that adjust update frequencies based on:

1. **Mouse hover over menu bar** (-50% CPU when not hovering)
2. **CPU window visibility** (-80% CPU when window closed)
3. **Progressive idle timeout** (-90% CPU after 10 minutes idle)
4. **Battery mode detection** (-80% CPU on battery power)
5. **App focus detection** (-90% CPU when other app is frontmost)

### Combined with Phases 1-4**: ~0.05% CPU in typical real-world usage (90% reduction)

---

## Implementation Timeline

### Recommended Approach

**Option A: Aggressive (1 day)**
- Phase 1: 5 minutes → -4% CPU
- Phase 2: 30 minutes → -3% CPU (total: -7%)
- Build & test: 30 minutes
- Phase 3: 1-2 hours → -2.5-3% CPU (total: -9.5-10%)
- Phase 4: 2-4 hours → -1-1.5% CPU (total: -10.5-11.5%)
- **Total: ~8-10 hours, -18-22% CPU reduction**

**Option B: Conservative (1 week)**
- Week 1: Phase 1 + Phase 2 → -7% CPU reduction
- Week 2: Phase 3 → -10% CPU reduction
- Week 3: Phase 4 → -11% CPU reduction
- **Total spread over 3 weeks, -20-24% CPU reduction**

**Option C: Minimal (No change)**
- Ship current state (~0.5% idle, <1% window open)
- Re-evaluate after user feedback

---

## Implementation Timeline

### Recommended Approach

**Option A: Aggressive (1 day)**
- Phase 1: 5 minutes → -4% CPU
- Phase 2: 30 minutes → -3% CPU (total: -7%)
- Build & test: 30 minutes
- Phase 3: 1-2 hours → -2.5-3% CPU (total: -9.5-10%)
- Phase 4: 2-4 hours → -1-1.5% CPU (total: -10.5-11.5%)
- **Total: ~8-10 hours, -18-22% CPU reduction**

**Option B: Conservative (1 week)**
- Week 1: Phase 1 + Phase 2 → -7% CPU reduction
- Week 2: Phase 3 → -10% CPU reduction
- Week 3: Phase 4 → -11% CPU reduction
- **Total spread over 3 weeks, -20-24% CPU reduction**

**Option C: Minimal (No change)**
- Ship current state (~0.5% idle, <1% window open)
- Re-evaluate after user feedback

---

## Expected Results

### Benchmarks (Activity Monitor on MacBook Air M2)

**Current State**
- Idle (menu bar only): ~0.5% CPU, 80 MB RAM
- CPU window open: ~0.8-1.0% CPU, 120 MB RAM
- Under sustained load: ~1.2-1.5% CPU, 150 MB RAM

**After Phase 1 (5 min work, -12-18% CPU)**
- Idle: ~0.5% CPU (no change)
- CPU window open: **~0.65-0.85% CPU** 
- Under sustained load: **~1.0-1.2% CPU** 

**After Phase 1 + Phase 2 (-15% CPU total)**
- Idle: ~0.5% CPU
- CPU window open: **~0.65-0.8% CPU** 
- Under sustained load: **~0.95-1.1% CPU** 

**After All Phases (-20-24% CPU total)**
- Idle: **~0.4-0.5% CPU** 
- CPU window open: **~0.6-0.75% CPU** 
- Under sustained load: **~0.9-1.0% CPU** 

---

## Expected Results

### Benchmarks (Activity Monitor on MacBook Air M2)

**Current State**
- Idle (menu bar only): ~0.5% CPU, 80 MB RAM
- CPU window open: ~0.8-1.0% CPU, 120 MB RAM
- Under sustained load: ~1.2-1.5% CPU, 150 MB RAM

**After Phase 1 (5 min work, -12-18% CPU)**
- Idle: ~0.5% CPU (no change)
- CPU window open: **~0.65-0.85% CPU** 
- Under sustained load: **~1.0-1.2% CPU** 

**After Phase 1 + Phase 2 (-15% CPU total)**
- Idle: ~0.5% CPU
- CPU window open: **~0.65-0.8% CPU** 
- Under sustained load: **~0.95-1.1% CPU** 

**After All Phases (-20-24% CPU total)**
- Idle: **~0.4-0.5% CPU** 
- CPU window open: **~0.6-0.75% CPU** 
- Under sustained load: **~0.9-1.0% CPU** 

---

## Risk Assessment

### Very Low Risk (can ship immediately)
- Task 1: Temperature interval change
- Task 2: Frequency interval change
- Task 3: Process cache interval change
- Task F1: Gauge threshold change
- Task F2: Animation threshold change
- Task F4: innerHTML → textContent
- Task F5: localStorage caching
- Task F6: Cleanup listeners

**Recommendation**: Implement all "Very Low Risk" tasks in Phase 1+2 first. These are safe and provide 15-20% improvement.

### Low Risk (good candidate for Phase 3)
- Task 5: Window visibility early exit
- Task F3: Defer slow metrics
- Task F7: Process list DOM batching

**Recommendation**: Implement after Phase 1+2, these require refactoring but are low-complexity.

### Medium Risk (Phase 4, optional)
- Task 4: Split ACCESS_CACHE
- Task 6: IOReport state caching

**Recommendation**: Only implement if measured improvements plateauing. These touch core state management.

### High Risk (not recommended)
- None identified

---

## Risk Assessment

### Very Low Risk (can ship immediately)
- Task 1: Temperature interval change
- Task 2: Frequency interval change
- Task 3: Process cache interval change
- Task F1: Gauge threshold change
- Task F2: Animation threshold change
- Task F4: innerHTML → textContent
- Task F5: localStorage caching
- Task F6: Cleanup listeners

**Recommendation**: Implement all "Very Low Risk" tasks in Phase 1+2 first. These are safe and provide 15-20% improvement.

### Low Risk (good candidate for Phase 3)
- Task 5: Window visibility early exit
- Task F3: Defer slow metrics
- Task F7: Process list DOM batching

**Recommendation**: Implement after Phase 1+2, these require refactoring but are low-complexity.

### Medium Risk (Phase 4, optional)
- Task 4: Split ACCESS_CACHE
- Task 6: IOReport state caching

**Recommendation**: Only implement if measured improvements plateauing. These touch core state management.

### High Risk (not recommended)
- None identified

---

## Testing Checklist

### Per-Task Testing
```bash
# Build
cargo build --release

# Smoke test
./target/release/mac_stats --cpu &
sleep 2 && pkill mac_stats

# Performance test (before optimization)
time ./target/release/mac_stats --cpu &
sleep 30 && pkill mac_stats

# Performance test (after optimization)
time ./target/release/mac_stats --cpu &
sleep 30 && pkill mac_stats

# Compare Activity Monitor CPU % before/after
```

### Integration Testing
1. Open CPU window
2. Let run for 5 minutes
3. Check no new log errors: `grep -i error ~/.mac-stats/debug.log`
4. Verify all metrics display: temperature, frequency, processes
5. Test theme switching (settings modal)
6. Verify responsive at ~60 FPS (DevTools Performance)

### Regression Testing
- [ ] Menu bar updates every 1-2 seconds
- [ ] CPU window refresh shows latest data
- [ ] Temperature updates every 20s (after Task 1)
- [ ] Frequency updates every 30s (after Task 2)
- [ ] Process list updates every 10s (after Task 3)
- [ ] No memory leaks over 1 hour runtime
- [ ] No log errors or warnings

---

## Testing Checklist

### Per-Task Testing
```bash
# Build
cargo build --release

# Smoke test
./target/release/mac_stats --cpu &
sleep 2 && pkill mac_stats

# Performance test (before optimization)
time ./target/release/mac_stats --cpu &
sleep 30 && pkill mac_stats

# Performance test (after optimization)
time ./target/release/mac_stats --cpu &
sleep 30 && pkill mac_stats

# Compare Activity Monitor CPU % before/after
```

### Integration Testing
1. Open CPU window
2. Let run for 5 minutes
3. Check no new log errors: `grep -i error ~/.mac-stats/debug.log`
4. Verify all metrics display: temperature, frequency, processes
5. Test theme switching (settings modal)
6. Verify responsive at ~60 FPS (DevTools Performance)

### Regression Testing
- [ ] Menu bar updates every 1-2 seconds
- [ ] CPU window refresh shows latest data
- [ ] Temperature updates every 20s (after Task 1)
- [ ] Frequency updates every 30s (after Task 2)
- [ ] Process list updates every 10s (after Task 3)
- [ ] No memory leaks over 1 hour runtime
- [ ] No log errors or warnings

---

## Measurement Tools

### CPU Usage
```bash
# Activity Monitor (GUI)
open -a "Activity Monitor" --args "mac_stats"

# Command line (top)
top -p $(pgrep mac_stats) -n 1

# Advanced (Instruments)
open -a Instruments  # Profile app
```

### Memory Usage
```bash
# Activity Monitor or:
ps aux | grep mac_stats | awk '{print $6}' # RSS in KB

# Over time:
watch -n 1 'ps aux | grep mac_stats'
```

### Frame Rate / DOM Performance
```bash
# DevTools (F12)
# Performance tab: Record 30s, check Main thread work
# Goal: < 16ms per frame (60 FPS capable)
```

### Logs
```bash
# Real-time monitoring
tail -f ~/.mac-stats/debug.log

# Error checking
grep ERROR ~/.mac-stats/debug.log | tail -20
```

---

## Measurement Tools

### CPU Usage
```bash
# Activity Monitor (GUI)
open -a "Activity Monitor" --args "mac_stats"

# Command line (top)
top -p $(pgrep mac_stats) -n 1

# Advanced (Instruments)
open -a Instruments  # Profile app
```

### Memory Usage
```bash
# Activity Monitor or:
ps aux | grep mac_stats | awk '{print $6}' # RSS in KB

# Over time:
watch -n 1 'ps aux | grep mac_stats'
```

### Frame Rate / DOM Performance
```bash
# DevTools (F12)
# Performance tab: Record 30s, check Main thread work
# Goal: < 16ms per frame (60 FPS capable)
```

### Logs
```bash
# Real-time monitoring
tail -f ~/.mac-stats/debug.log

# Error checking
grep ERROR ~/.mac-stats/debug.log | tail -20
```

---