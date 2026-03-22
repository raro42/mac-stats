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

### Connection process (step-by-step)

When you invoke a BROWSER_* tool (e.g. BROWSER_NAVIGATE, BROWSER_SCREENSHOT), the app does the following:

1. **Session lookup**  
   mac-stats keeps a single cached CDP (Chrome DevTools Protocol) session. If a session already exists and was used recently (within the idle timeout, default 1 hour), that connection is reused and the tool runs immediately.

2. **No session or session expired**  
   The app needs a live Chrome to talk to:
   - **Port check:** It requests `http://127.0.0.1:9222/json/version` to see if Chrome is already running with remote debugging. If that succeeds, it reads the WebSocket URL from the response and connects to that Chrome (your manually started one or an existing mac-stats-launched one).
   - **Connect:** It opens a CDP connection over WebSocket. That connection is then cached as the "session" for subsequent BROWSER_* calls.

3. **Nothing on port 9222**  
   If nothing is listening on 9222:
   - **Visible Chrome (default):** mac-stats starts Chrome with `--remote-debugging-port=9222`, waits about 3 seconds for it to be ready, then connects. No manual step needed if Chrome is installed in the default location.
   - **Headless:** If the run is configured for headless mode, mac-stats uses the headless_chrome crate to launch a separate Chrome process (no port 9222). That session is still cached the same way.

4. **Session cleared on error**  
   If a CDP call fails with a connection error (e.g. Chrome closed, network error), mac-stats clears the cached session. The next BROWSER_* use goes back to step 1 and will either reconnect to 9222 or relaunch Chrome (or headless) as needed.

5. **Idle timeout**  
   If no BROWSER_* tool is used for longer than the configured idle timeout (default 1 hour), the session is dropped. The next use follows steps 2–3 again.

So: **first use** or **after an error/timeout** → check 9222 → connect or launch → cache session. **Later uses** → reuse cached session until idle too long or an error occurs.

### Cookie consent and screenshots
When the user asks to remove or dismiss a cookie consent banner and take a screenshot, the planning prompt instructs the model to include **BROWSER_CLICK** on the consent button (using the Elements list from BROWSER_NAVIGATE) **before** BROWSER_SCREENSHOT. Pre-routing to NAVIGATE + SCREENSHOT is skipped when the question mentions cookie/consent/banner so the planner can add the click step.

### Navigation timeout, new tab, and go back
- **Navigation timeout:** Maximum wait for BROWSER_NAVIGATE (and BROWSER_GO_BACK) is configurable: `config.json` key `browserNavigationTimeoutSecs` (default 30, range 5–120) or env `MAC_STATS_BROWSER_NAVIGATION_TIMEOUT_SECS`. Slow or stuck navigations fail with a clear message (e.g. "Navigation failed: timeout after 30s") instead of hanging.
- **Same-domain shorter timeout (optional):** When the navigation target is on the same domain as the current page (e.g. in-site link or SPA transition), a shorter wait can be used so in-site navigations don’t wait the full timeout. Set `config.json` key `browserSameDomainNavigationTimeoutSecs` (e.g. 5) or env `MAC_STATS_BROWSER_SAME_DOMAIN_NAVIGATION_TIMEOUT_SECS`. When set, same-domain navigations use this value for the post-navigate wait; cross-domain and BROWSER_GO_BACK still use `browserNavigationTimeoutSecs`. When not set, all navigations use the single navigation timeout. Debug log line "same-domain navigation, using Ns timeout" confirms when the shorter timeout is applied.
- **New tab:** Add `new_tab` after the URL (e.g. `BROWSER_NAVIGATE: https://example.com new_tab`) to open the URL in a new tab and switch focus to it; subsequent BROWSER_CLICK / BROWSER_SCREENSHOT apply to that tab.
- **Screenshot with URL:** `BROWSER_SCREENSHOT: https://…` navigates and captures the **focused** tab (the same internal tab index as `BROWSER_NAVIGATE` and `new_tab`), not the first tab in the Chrome window.
- **BROWSER_GO_BACK:** Use `BROWSER_GO_BACK` (no argument) to go back one step in the current tab's history and get the new page state. Use when returning to the previous page without re-entering the URL.

### Hard navigation failures (Chrome `errorText` and error pages)
CDP **`Page.navigate`** can return an **`errorText`** field (e.g. `net::ERR_NAME_NOT_RESOLVED`, TLS errors). The **headless_chrome** crate surfaces that as a failed `navigate_to` call. **BROWSER_NAVIGATE** and **BROWSER_SCREENSHOT** (with a URL) turn that into an explicit tool error: the message states that the page did **not** load, includes a **sanitized** Chrome error string (paths such as `/Users/...` are redacted for the model), and tells the model not to treat the tab as the requested site.

If Chrome loads an internal **`chrome-error://`** document without failing `navigate_to`, mac-stats detects that from the tab URL after the navigation wait and returns a similar failure instead of a normal **Elements** list. **Cookie-banner** auto-dismiss is skipped on **`chrome-error://`** pages.

When an **HTTP(S)** URL was requested but the tab stays **`about:blank`** and Chrome did not report `errorText`, the tool returns a **single** cautious line that the navigation **may** have failed (heuristic; do not assume the page loaded). This is separate from **SPA / hash** behaviour: if **`wait_until_navigated`** times out with a **timeout** error, the existing **timeout** failure still applies; non-timeout wait failures keep the **warn + short sleep** path when there is no hard error.

Debug logging (`browser/cdp`): one line with **host only** (no path/query) plus a short **error class** (first token of `errorText`), and a second **debug** line with the **full** `errorText` for reviewers.

### Grounded browser retries
- `BROWSER_NAVIGATE` must receive a concrete URL. Natural-language filler such as `BROWSER_NAVIGATE to the video URL` is rejected and treated as an agent-side planning/parsing failure, not as evidence about the website.
- After `BROWSER_NAVIGATE`, `BROWSER_CLICK`, or `BROWSER_INPUT`, mac-stats caches the latest `Current page` and `Elements` output. Retries should reuse that latest state instead of stale indices from an earlier page.
- If the browser is already on the relevant page and the content is inline, the next retry should inspect that page or click a real listed element. It should not invent a new target URL unless the browser output already exposed one.

### Sequence-terminating navigation
When the model returns multiple tools in one turn, page-changing browser actions (`BROWSER_NAVIGATE`, `BROWSER_GO_BACK`) **terminate the browser sequence** for that turn. After one of these actions completes successfully, any remaining browser tools (BROWSER_CLICK, BROWSER_INPUT, BROWSER_SCROLL, BROWSER_EXTRACT, BROWSER_SEARCH_PAGE, BROWSER_SCREENSHOT) from the same response are skipped. The model receives the new page state and a note that remaining browser actions were skipped due to stale indices. Non-browser tools (FETCH_URL, BRAVE_SEARCH, RUN_CMD, etc.) in the same response still run.

This prevents wrong clicks or inputs caused by stale element indices from the previous page. The model can emit new browser actions in the next turn using the fresh state.

### SSRF protection
All server-side URL fetches and browser navigations triggered by the model are protected against SSRF (Server-Side Request Forgery). Before any HTTP request (`FETCH_URL`) or CDP navigation (`BROWSER_NAVIGATE`, `BROWSER_SCREENSHOT` with a URL), the target URL is validated:

- **Blocked targets:** loopback (127.0.0.0/8, ::1), RFC 1918 private (10/8, 172.16/12, 192.168/16), link-local (169.254.0.0/16, fe80::/10), cloud metadata (169.254.169.254), IPv6 unique-local (fc00::/7), unspecified (0.0.0.0, ::), broadcast, and IPv4-mapped IPv6 variants.
- **Userinfo rejected:** URLs with embedded credentials (e.g. `http://user:pass@host/`) are blocked.
- **DNS resolution check:** The hostname is resolved to IP addresses and each is checked against the blocklist, catching hostnames that resolve to private IPs.
- **Redirect protection:** For HTTP fetches (reqwest), a custom redirect policy checks each redirect hop against the same blocklist, preventing redirect-to-private chains.
- **Allowlist:** To explicitly allow fetching from a local service (e.g. a local Redmine), add `"ssrfAllowedHosts": ["hostname-or-ip"]` in `~/.mac-stats/config.json`. Default: empty (no exceptions). The same allowlist applies to the initial URL and to redirect targets matching the allowed host.

### Summary
| Requirement | What mac-stats does |
|------------|----------------------|
| Chrome on 9222 | If port is free, **launches** Chrome with `--remote-debugging-port=9222` (macOS/Linux). If port is in use, **connects** to existing Chrome. |
| Chrome not installed | Cannot launch; you must install Chrome and/or start it manually on 9222. |
| Connection dies (timeout, crash) | Session is cleared on error; next use will reconnect to 9222 or relaunch. |

### Troubleshooting: Chrome won't start or connect on 9222

If BROWSER_* tools fail with "Chrome isn't running on port 9222" or launch errors, work through the following.

1. **Chrome not at default path**
   - **macOS:** mac-stats expects Chrome at `/Applications/Google Chrome.app/Contents/MacOS/Google Chrome`. If you installed Chrome elsewhere (e.g. Chromium, a different name), either move/symlink it to that path or start Chrome yourself with `--remote-debugging-port=9222` and leave it running; mac-stats will connect to it.
   - **Linux:** Chrome must be on PATH as `google-chrome`. Install the official package or ensure the binary is named/linked so that `google-chrome` runs.

2. **Port 9222 already in use**
   - Another Chrome (or another app) may be using 9222. mac-stats will *connect* to an existing Chrome on 9222; if that Chrome is stuck or not responding, close it and retry.
   - To see what is using the port:
     - **macOS/Linux:** `lsof -i :9222` or `netstat -an | grep 9222`. If it's an old Chrome, kill that process (e.g. `kill <PID>`) and retry.
   - If you want to use a different port, that is not currently configurable; use 9222 or start Chrome manually on 9222 before using BROWSER_*.

3. **Spawn fails (e.g. "Launch Chrome: ... is Chrome installed at ...?")**
   - **Permissions:** On macOS, ensure the app is allowed to run external apps (e.g. not overly restricted in Privacy & Security). If you built from source, run the binary from Terminal once to rule out Gatekeeper blocking.
   - **Quarantine:** If Chrome was downloaded and is quarantined, the system might block the child process. Try: `xattr -d com.apple.quarantine "/Applications/Google Chrome.app"` (or the path you use).
   - **Architecture:** Use a Chrome build that matches your Mac (Apple Silicon vs Intel). mac-stats does not switch Chrome variants.

4. **Launch succeeds but connection still fails**
   - mac-stats waits ~3 seconds after launch before connecting. On a slow machine or under load, Chrome may need longer. Start Chrome manually with `--remote-debugging-port=9222`, wait until it's fully up, then trigger the BROWSER_* tool again.
   - **Firewall:** Ensure nothing blocks localhost (127.0.0.1) port 9222. macOS firewall "Block all incoming" can still allow outgoing/local connections; if you use a strict tool, allow the app or Chrome.

5. **Fallback: headless Chrome**
   - If visible Chrome on 9222 is not an option, the app can use the headless_chrome crate (no port 9222). When the model or user says "headless", BROWSER_* uses that path. You can also retry after a failed launch; in some cases the code falls back to headless automatically.

If the problem persists, check `~/.mac-stats/debug.log` for lines like `Browser agent [CDP]: ...` or `Launch Chrome: ...` to see the exact error.

## Open tasks

Browser-automation open tasks are tracked in **006-feature-coder/FEATURE-CODER.md**.
