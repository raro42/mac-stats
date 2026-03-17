# mac-stats Documentation

## Global Context

A local AI agent for macOS: Ollama chat, Discord bot, task runner, scheduler, and MCP—all on your Mac. No cloud, no telemetry. Lives in your menu bar—CPU, GPU, RAM, disk at a glance. Real-time, minimal, there when you look. Built with Rust and Tauri.

## Install

### DMG (recommended)
[Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.

### Build from source:
```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```
Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

### If macOS blocks the app:
Gatekeeper may show "damaged" or block the unsigned app—the file is fine. Right-click the DMG → **Open**, then confirm. Or after install: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

## At a Glance

- **Menu bar** — CPU, GPU, RAM, disk at a glance; click to open the details window.
- **AI chat** — Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.
- **Discord**

## Tool Agents (what Ollama can invoke)

Whenever Ollama is asked to decide which agent to use (planning step in Discord and scheduler flow), the app sends the **complete list of active agents**: the invocable tools below plus the **SCHEDULER** (informational; Ollama can recommend it for recurring or delayed tasks but cannot invoke it with a tool line). Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

## CPU Optimization Task Suite

This folder contains comprehensive CPU optimization analysis and implementation tasks for reducing mac-stats CPU usage from ~1% (window open) to ~0.6-0.8%.

## Quick Start

**New to these optimizations?** Start here:

1. Read: [`000_task_optimize_summary.md`](000_task_optimize_summary.md) (5-10 min read)
2. Choose a phase (Phase 1 recommended for quick wins)
3. Track progress using the task docs (000, 001, 002) and phase table below
4. Implement: Reference the detailed task documents

## Documents Overview

### 📋 Summary & Planning
**[`000_task_optimize_summary.md`](000_task_optimize_summary.md)** (403 lines)
- Executive summary with effort/benefit matrix
- 4 implementation phases with timeline options
- Expected results and risk assessment
- Quick reference table for all 15 tasks

### 🔧 Backend Optimization Tasks
**[`001_task_optimize_backend.md`](001_task_optimize_backend.md)** (549 lines)
- 8 Rust/FFI backend tasks
- High, medium, low impact optimizations
- Detailed code changes with line numbers
- Testing and validation for each task
- **Est. savings**: 16-18% CPU reduction

**Key Tasks**:
- Task 1: Temperature interval 15s → 20s (1 line)
- Task 2: Frequency interval 20s → 30s (1 line)
- Task 3: Process cache 5s → 10s (1 line)
- Task 4: Split ACCESS_CACHE locks (30 lines)
- Task 5: Window visibility early exit (20 lines)
- Task 6: IOReport state caching (25 lines)
- Task 7: CFRelease batching (15 lines)
- Task 8: Frequency logging cache (10 lines)

### 💻 Frontend Optimization Tasks
**[`002_task_optimize_frontend.md`](002_task_optimize_frontend.md)** (695 lines)
- 7 JavaScript/HTML frontend tasks
- DOM, animation, and IPC optimizations
- Before/after code examples
- Browser DevTools testing guides
- **Est. savings**: 6-7% CPU reduction

**Key Tasks**:
- Task F1: Gauge threshold 2% → 5% (1 number)
- Task F2: Animation threshold 15% → 20% (1 number)
- Task F3: Defer slow metrics to 5s (40 lines)
- Task F4: Replace innerHTML with textContent (10 lines)
- Task F5: Cache version in localStorage (15 lines)
- Task F6: Window cleanup listeners (5 lines)
- Task F7: Optimize process list DOM (25 lines)

## Phase Overview

| Phase | Duration | Tasks | CPU Reduction | Risk | Status |
|-------|----------|-------|---------------|------|--------|
| **1: Quick Wins** | 5 min | 5 tasks (1-line each) | -12-18% | ⭐ Very Low | Ready |
| **2: Easy Wins** | 30 min | 4 tasks (10-25 lines each) | -1.5-2% | ⭐ Low | Ready |
| **3: Refactoring** | 1-2 hrs | 3 tasks (20-40 lines each) | -2.5-3% | ⭐⭐ Medium | Ready |
| **4: Advanced** | 2-4 hrs | 3 tasks (specialized) | -1-1.5% | ⭐⭐ Medium | Ready |
| **Total** | ~8 hrs | **15 tasks** | **-18-24%** | ⭐⭐ Medium | **Go** |

## File Structure

```
docs/
├── README.md                            ← You are here
├── 000_task_optimize_summary.md         ← Start here
├── 001_task_optimize_backend.md         ← Backend tasks
├── 002_task_optimize_frontend.md        ← Frontend tasks
└── data-poster-charts-backend.md        ← (Unrelated)
```

## Quick Decision Tree

**Q: How much time do I have?**

- **5 minutes**: Do Phase 1 only (5 one-line changes, -12-18% CPU) 
- **30 minutes**: Phase 1 + 2 (-15% CPU) 
- **2 hours**: Phase 1 + 2 + 3 (-20% CPU) 
- **Full day**: All phases 1-4 (-20-24% CPU) 

**Q: How risky are these changes?**

- **Very Low Risk**: All Phase 1 + Phase 2 tasks (10 total)
- **Low Risk**: Phase 3 tasks
- **Medium Risk**: Phase 4 tasks (optional, advanced)
- **None identified as High Risk**

**Q: What's the biggest improvement?**

Tasks 1-3 (interval adjustments) save -12-18% CPU with just 5 one-line changes.

## Getting Started: Step-by-Step

### Step 1: Understand the Current State
```bash
# Current performance baseline
Activity Monitor → Search "mac_stats"
# Note CPU usage idle and with window open
```

### Step 2: Choose Your Phase
- **Aggressive**: Do all 4 phases (8 hours, -20-24% CPU)
- **Balanced**: Phases 1-3 (2-3 hours, -20% CPU)
- **Conservative**: Phases 1-2 (35 minutes, -15% CPU)
- **Minimal**: Phase 1 only (5 minutes, -15% CPU)

### Step 3: Read the Summary
Open and read [`000_task_optimize_summary.md`](000_task_optimize_summary.md)
- Understand phases and effort/benefit
- Pick your implementation approach
- Estimate timeline

### Step 4: Implement Tasks
For each task in your chosen phase:

1. Open [`001_task_optimize_backend.md`](001_task_optimize_backend.md) or [`002_task_optimize_frontend.md`](002_task_optimize_frontend.md)
2. Find the task section
3. Make code changes as specified
4. Build: `cargo build --release`
5. Test: Open app and verify behavior
6. Check: Track completion via the phase table and task docs

### Step 5: Measure Results
```bash
# Before optimization (baseline already noted)
# After Phase 1:
Activity Monitor → Compare CPU usage
# Should see 12-18% reduction

# After later phases:
# Continue measuring cumulative improvement
```

### Step 6: Commit & Ship
```bash
git add .
git commit -m "Optimize CPU: phases 1-3 (-20%)"
git push
```

## Code Examples

### Phase 1 Example (5-second fix)
```rust
# File: src-tauri/src/lib.rs:409
# Before:
.map(|t| t.elapsed().as_secs() >= 15)

# After:
.map(|t| t.elapsed().as_secs() >= 20)
```

### Phase 3 Example (40-line refactor)
```javascript
# File: src-tauri/dist/cpu.js:414-448
# Before: Update all metrics every 1 second
function scheduleDOMUpdate() {
    // Updates load, uptime, power...
}

# After: Split into fast (1s) and slow (5s)
function scheduleDOMUpdate() {
    // Fast metrics only
}
function updateSlowMetrics() {
    // Slow metrics every 5s
}
```

## Testing & Validation

### Quick Test
```bash
cd /Users/raro42/projects/mac-stats/src-tauri

# Build
cargo build --release

# Run with CPU window
./target/release/mac_stats --cpu

# Monitor Activity Monitor for CPU usage
# Compare against baseline
```

### Detailed Test
```bash
# Enable verbose logging
./target/release/mac-stats --cpu -vvv

# Watch debug output
tail -f ~/.mac-stats/debug.log

# Look for expected patterns:
# - Temperature updates every 20s (after Task 1)
# - Frequency updates every 30s (after Task 2)
# - Process cache hits every 10s (after Task 3)
```

### DevTools Test (Frontend)
```bash
# Open app
./target/release/mac-stats --cpu

# Open DevTools: F12
# Performance tab: Record 30 seconds
# Check:
# - Main thread work < 16ms (60 FPS capable)
# - No long tasks
# - DOM updates efficient
```

### Agent test (regression)
Run an agent with prompts from its `testing.md` (e.g. Redmine). Each prompt has a **45s default timeout** so a stuck model fails fast instead of hanging. Override via `MAC_STATS_AGENT_TEST_TIMEOUT_SECS` or `config.json` `agentTestTimeoutSecs` (5–300).

```bash
cd src-tauri
./target/release/mac_stats agent test redmine
# Or with custom test file:
./target/release/mac_stats agent test redmine /path/to/testing.md
```

Requires Ollama running. Logs: `~/.mac-stats/debug.log`. See also [007_discord_agent.md](007_discord_agent.md) (§14) for the Discord-style pipeline without Discord.

## Common Questions

**Q: Can I mix tasks from different phases?**
A: Yes, but not recommended. Tasks in Phase 1 are prerequisites for Phase 2, etc.

**Q: What if a task doesn't compile?**
A: Check the line numbers match (file might have changed). Reference the detailed task description for context.

**Q: How do I rollback a task?**
A: Edit the file to revert the change, rebuild. Each task is independent.

**Q: Can I measure CPU reduction myself?**
A: Yes! Use Activity Monitor or `top -p $(pgrep mac_stats)`. Before and after same workload.

**Q: Which task has the most impact?**
A: Tasks 1-3 combined save -12-18% CPU with just 5 one-line changes.

**Q: Do I need to do all tasks?**
A: No. Phase 1 alone gives good improvement with minimal effort. Phases 2-4 are optional.

**Q: Will these changes affect app features?**
A: No. These are performance-only optimizations. All features unchanged.

## Document Statistics

- **Total lines**: 2,015
- **Backend tasks**: 8 detailed tasks with code
- **Frontend tasks**: 7 detailed tasks with code
- **Checklist items**: 80+ verification points
- **Code examples**: 20+ before/after comparisons
- **Testing procedures**: 15+ test scenarios

## Related Documentation

- **Architecture Overview**: [`../CLAUDE.md`](../CLAUDE.md)
- **Project Principles**: [`../agents.md`](../agents.md)
- **README**: [`../README.md`](../README.md)
- **Docs vs code review**: [`033_docs_vs_code_review.md`](033_docs_vs_code_review.md)
- **What others do & plan**: [`034_what_others_do_and_plan.md`](034_what_others_do_and_plan.md) — research and proposed priorities
- **Memory and topic handling**: [`035_memory_and_topic_handling.md`](035_memory_and_topic_handling.md) — log review, OpenClaw/Hermes, topic-aware compaction and user reset

## Status

| Component | Status |
|-----------|--------|
| Backend analysis | 
| Frontend analysis | 
| Task documentation | 
| Code examples | 
| Testing procedures | 
| Checklist | 

## Cross-Cutting Follow-Ups

- Investigate the macOS Gatekeeper / quarantine friction for DMG installs and first launch.
- Keep reviewing browser automation and screenshot routing so single-tool requests do not drift into unrelated tool chains.
- Trimmed (2025-03): removed completed items from `007_discord_agent.md`, added FEATURE-CODER pointer in `006_roadmap_ai_tasks.md`. Other docs now point to **006-feature-coder/FEATURE-CODER.md** for the active backlog (007, 022, 029, 002, agent_workflow, 008, 012, 035).