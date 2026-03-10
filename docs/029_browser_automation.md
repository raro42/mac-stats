## Global Context
### README.md snippets
#### mac-stats

**The AI agent that just gets it done. All local.**

[![GitHub release](https://img.shields.io/github/v/release/raro42/mac-stats?include_prereleases&style=flat-square)](https://github.com/raro42/mac-stats/releases/latest)

A local AI agent for macOS: Ollama chat, Discord bot, task runner, scheduler, and MCP—all on your Mac. No cloud, no telemetry. Lives in your menu bar—CPU, GPU, RAM, disk at a glance. Real-time, minimal, there when you look. Built with Rust and Tauri.

<img src="screens/data-poster.png" alt="mac-stats Data Poster theme" width="500">

📋 [Changelog](CHANGELOG.md) · 📸 [Screenshots & themes](screens/README.md)

---

## Install
### DMG (recommended)
[Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.

### Build from source
```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```
Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

### If macOS blocks the app
Gatekeeper may show "damaged" or block the unsigned app—the file is fine. Right-click the DMG → **Open**, then confirm. Or after install: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

---

## At a Glance
### Menu Bar
- **CPU, GPU, RAM, disk at a glance; click to open the details window.**

### AI Chat
- **Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.**

### Discord Bot
- **Discord bot functionality, including FETCH_URL and BRAVE_SEARCH.**

---

## Tool Agents (what Ollama can invoke)
### Invocation
Whenever Ollama is asked to decide which agent to use (planning step in Discord and scheduler flow), the app sends the **complete list of active agents**: the invocable tools below plus the **SCHEDULER** (informational; Ollama can recommend it for recurring or delayed tasks but cannot invoke it with a tool line). Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

### Agents
| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

---

## Browser Automation (BROWSER_* tools)
### Requirements
1. **Chrome installed**
   - **macOS:** `/Applications/Google Chrome.app` (standard install).
   - **Linux:** `google-chrome` on PATH.

2. **Chrome on port 9222**
   - **You start Chrome:** run Chrome with remote debugging so mac-stats can attach:
     ```bash
     /Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome --remote-debugging-port=9222
     ```
     Leave that window running. mac-stats will connect to it when you use BROWSER_* tools.

   - **mac-stats starts Chrome:** if nothing is listening on 9222, mac-stats will try to launch Chrome with `--remote-debugging-port=9222`, wait ~3 seconds, then connect. No manual step needed if Chrome is installed in the default location.

3. **If you see “Chrome isn’t running on port 9222” or connection errors**
   - Start Chrome manually with the command above, then retry.
   - If Chrome is not installed at the path above, install it or create a symlink; mac-stats does not install Chrome.
   - After a timeout or crash, mac-stats clears the cached session; the next BROWSER_* use will reconnect or relaunch.

### Cookie consent and screenshots
When the user asks to remove or dismiss a cookie consent banner and take a screenshot, the planning prompt instructs the model to include **BROWSER_CLICK** on the consent button (using the Elements list from BROWSER_NAVIGATE) **before** BROWSER_SCREENSHOT. Pre-routing to NAVIGATE + SCREENSHOT is skipped when the question mentions cookie/consent/banner so the planner can add the click step.

### Navigation timeout, new tab, and go back
- **Navigation timeout:** Maximum wait for BROWSER_NAVIGATE (and BROWSER_GO_BACK) is configurable: `config.json` key `browserNavigationTimeoutSecs` (default 30, range 5–120) or env `MAC_STATS_BROWSER_NAVIGATION_TIMEOUT_SECS`. Slow or stuck navigations fail with a clear message (e.g. "Navigation failed: timeout after 30s") instead of hanging.
- **New tab:** Add `new_tab` after the URL (e.g. `BROWSER_NAVIGATE: https://example.com new_tab`) to open the URL in a new tab and switch focus to it; subsequent BROWSER_CLICK / BROWSER_SCREENSHOT apply to that tab.
- **BROWSER_GO_BACK:** Use `BROWSER_GO_BACK` (no argument) to go back one step in the current tab's history and get the new page state. Use when returning to the previous page without re-entering the URL.

### Grounded browser retries
- `BROWSER_NAVIGATE` must receive a concrete URL. Natural-language filler such as `BROWSER_NAVIGATE to the video URL` is rejected and treated as an agent-side planning/parsing failure, not as evidence about the website.
- After `BROWSER_NAVIGATE`, `BROWSER_CLICK`, or `BROWSER_INPUT`, mac-stats caches the latest `Current page` and `Elements` output. Retries should reuse that latest state instead of stale indices from an earlier page.
- If the browser is already on the relevant page and the content is inline, the next retry should inspect that page or click a real listed element. It should not invent a new target URL unless the browser output already exposed one.

### Summary
| Requirement | What mac-stats does |
|------------|----------------------|
| Chrome on 9222 | If port is free, **launches** Chrome with `--remote-debugging-port=9222` (macOS/Linux). If port is in use, **connects** to existing Chrome. |
| Chrome not installed | Cannot launch; you must install Chrome and/or start it manually on 9222. |
| Connection dies (timeout, crash) | Session is cleared on error; next use will reconnect to 9222 or relaunch. |

## Open tasks:
- Investigate why some users are unable to launch Chrome on port 9222.
- Improve the documentation for BROWSER_* tools to better explain the connection process.