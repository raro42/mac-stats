## mac-stats Project Optimization

### Global Context

The mac-stats project is a local AI agent for macOS that provides various features, including a chat interface, task runner, scheduler, and MCP. The project aims to minimize CPU usage while maintaining functionality and responsiveness.

### Install

#### DMG (recommended)

1. Download the latest release from [GitHub](https://github.com/raro42/mac-stats/releases/latest)
2. Drag the DMG file to the Applications folder

#### Build from source

1. Clone the repository using `git clone https://github.com/raro42/mac-stats.git`
2. Navigate to the project directory using `cd mac-stats`
3. Run the build command using `./run`

#### One-liner

```bash
curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run
```

### At a Glance

* Menu bar displays CPU, GPU, RAM, and disk usage
* AI chat interface with Ollama and Discord bot
* Task runner and scheduler for managing tasks
* MCP integration for monitoring system resources

### Tool Agents

* **FETCH_URL**: Fetches web page content as text
* **BRAVE_SEARCH**: Performs web search using Brave Search API
* **RUN_JS**: Executes JavaScript code
* **SCHEDULER**: Displays scheduling information

### Optimizations

#### High Impact Optimizations

1. **Increase Ring Gauge Update Threshold**: Increases the threshold for gauge updates from 2% to 5% (2% → 5%)
	* Impact: 2-3% CPU reduction
	* Effort: 1 number change (30 seconds)
2. **Increase Animation Animation Skip Threshold**: Increases the threshold for animation updates from 15% to 20% (15% → 20%)
	* Impact: 1-2% CPU reduction
	* Effort: 1 number change (30 seconds)
3. **Defer Non-Critical Metrics to 5-Second Interval**: Defer updates to non-critical metrics (temperature, load average, uptime) to every 5 seconds
	* Impact: 1-2% CPU reduction
	* Effort: Medium (add gating logic, ~40 lines)
4. **Replace innerHTML with textContent**: Replace innerHTML with textContent for static structures
	* Impact: 0.5-1% CPU reduction
	* Effort: Low (find/replace pattern, ~10 lines)

#### Medium Impact Optimizations

1. **Cache App Version in localStorage**: Cache the app version in localStorage to reduce Tauri invoke calls
	* Impact: Negligible CPU reduction (~0.1%)
	* Effort: Low (add caching logic, ~15 lines)
2. **Optimize Process List DOM Updates**: Optimize process list DOM updates by using a document fragment and textContent
	* Impact: 0.2-0.5% CPU reduction
	* Effort: Medium (refactor process list logic, ~25 lines)

#### Low Impact Optimizations

1. **Reduce Gauge Animation Frame Rate (if needed)**: Reduce the gauge animation frame rate from 20fps to 50fps
	* Impact: Negligible CPU reduction (~0%)
	* Effort: No changes needed
2. **Defer Theme Switching Animation (if added)**: Defer theme switching animation to reduce page reloads
	* Impact: Negligible CPU reduction (~0%)
	* Effort: No changes needed

### Performance Targets

| Task | Impact | Cumulative |
|------|--------|-----------|
| After Task 1+2 | -3% | -3% |
| After Task 4 | -1% | -4% |
| After Task 3 | -2% | -6% |
| After Task 5 | -0.1% | -6.1% |
| After Task 7 | -0.5% | -6.6% |
| After Task 6 | -0% | -6.6% |

**Conservative estimate**: Implementing all 7 main frontend tasks yields ~6-7% CPU reduction.

### Frontend + Backend Combined

Combining optimizations from both documents:

* Backend tasks: -16-18% CPU
* Frontend tasks: -6-7% CPU
* **Total combined**: ~20-24% CPU reduction

This achieves the goal of "minimizing CPU usage to the absolute max" while maintaining functionality and responsiveness.

### fetch_page_content and main-thread blocking (verified)

**Question:** Does `fetch_page_content` block the main thread when triggered from the frontend?

**Answer: No.** The frontend does not call `fetch_page_content` directly. It invokes the Tauri command **`fetch_page`** (`commands/browser.rs`). That command is `async` and runs the synchronous `fetch_page_content` inside `tokio::task::spawn_blocking(...)`, so the blocking HTTP work runs on a thread-pool task, not on the main (UI) thread. Other callers (Ollama FETCH_URL flow, scheduler, browser_agent) run in background threads or async contexts, not on the main thread. No change required for frontend-triggered paths.

### Open tasks

Frontend-optimization open tasks are tracked in **006-feature-coder/FEATURE-CODER.md**.