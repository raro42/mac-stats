# mac-stats

## Global Context

A local AI agent for macOS: Ollama chat, Discord bot, task runner, scheduler, and MCP—all on your Mac. No cloud, no telemetry. Lives in your menu bar—CPU, GPU, RAM, disk at a glance. Real-time, minimal, there when you look. Built with Rust and Tauri.

## Install

### DMG (recommended)
[Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.

### Build from source
```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```
Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

### Gatekeeper workaround
If macOS blocks the app: Right-click the DMG → **Open**, then confirm. Or after install: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

## At a Glance

- **Menu bar** — CPU, GPU, RAM, disk at a glance; click to open the details window.
- **AI chat** — Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.
- **Discord bot** — Handles user input, sends responses to Discord.

## Tool Agents (what Ollama can invoke)

Whenever Ollama is asked to decide which agent to use, the app sends the **complete list of active agents**: the invocable tools below plus the **SCHEDULER** (informational; Ollama can recommend it for recurring or delayed tasks but cannot invoke it with a tool line). Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

## Reviewing Logs

### Where Logs Are

- **Path:** `~/.mac-stats/debug.log`
- **View live:** `tail -f ~/.mac-stats/debug.log`
- **Verbosity:** Start mac-stats with `-v`, `-vv`, or `-vvv` for more detail.

### What to Search For

| Search term | Meaning |
|-------------|---------|
| `Discord/Ollama:` | Tool the agent chose (e.g. `FETCH_URL requested`, `BROWSER_SCREENSHOT requested`) |
| `BROWSER_SCREENSHOT: arg from parser` | Raw argument from the model (before URL cleaning) |
| `BROWSER_SCREENSHOT: URL sent to CDP` | Exact URL passed to the browser (after trimming trailing punctuation) |
| `Agent router: understood tool` | Which tool and argument were parsed from the model reply |
| `Agent router: running tool` | Which tool is being executed |
| `FETCH_URL` | Fetching page text (no screenshot) |
| `BROWSER_SCREENSHOT` | Opening URL in browser and saving a screenshot PNG |
| `Discord←Ollama: received` | Final reply text sent back to Discord |
| `Discord: sent N attachment(s)` | Screenshot file(s) were posted to the channel |

## PERPLEXITY_SEARCH (search → visit URLs → screenshot)

When the user asks to search the web (e.g. Perplexity) and to visit URLs or get screenshots "here" or "in Discord", the planner recommends **PERPLEXITY_SEARCH: <query>**. The app then:

1. **Truncates the search query** so the API gets only the query (e.g. "spanish newspaper websites"), not the rest of the plan (e.g. "then BROWSER_NAVIGATE: ..."). Truncation uses separators: ` then `, ` and then `, ` → `, `BROWSER_NAVIGATE:`, `BROWSER_SCREENSHOT:`, etc., and a 150-character cap.
2. **Runs Perplexity search** and gets results with URLs.
3. **If the question asked for screenshots** (e.g. "screenshot", "visit", "send me … in Discord"), the app **auto-visits** the first 5 result URLs and **takes a screenshot** of each, then attaches them in Discord.

## Feedback

### TASK_APPEND: Auto-visit + screenshot workflow for Perplexity search results
implemented in `commands/ollama.rs` (query truncation, `want_screenshots` detection including "send me", "in discord", "send the", " here "; first 5 result URLs visited via browser_agent, screenshots saved and attached in Discord). Log review section added above. `want_screenshots` broadened so "send me the screenshots in Discord" triggers the workflow without requiring "visit" or "url" in the question.

### TASK_STATUS: finished