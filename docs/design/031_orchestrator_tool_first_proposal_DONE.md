## Install

### DMG (recommended)
[Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.

### Build from source
```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```
Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

### If macOS blocks the app:
Gatekeeper may show "damaged" or block the unsigned app—the file is fine. Right-click the DMG → **Open**, then confirm. Or after install: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

## At a glance

- **Menu bar** — CPU, GPU, RAM, disk at a glance; click to open the details window.
- **AI chat** — Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.
- **Discord**

## Tool agents (what Ollama can invoke)

Whenever Ollama is asked to decide which agent to use (planning step in Discord and scheduler flow), the app sends the **complete list of active agents**: the invocable tools below plus the **SCHEDULER** (informational; Ollama can recommend it for recurring or delayed tasks but cannot invoke it with a tool line). Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

## Proposal: Tool-first routing (avoid wrong AGENT delegation)

### General problem

When the user asks for something that **exactly matches a base tool** (e.g. “go to this URL and take a screenshot”), the **orchestrator** (planning step) can recommend **AGENT: some-agent** instead of the tool (e.g. **BROWSER_SCREENSHOT: &lt;url&gt;**). The sub-agent then often replies with text and never invokes the tool. Result: the user’s request is not fulfilled.

### Root cause

The planner sees (1) the current question and (2) **conversation history**. If a previous turn in the same thread was a similar request that got a **text-only** reply (no tool), the planner may infer “handle this via an agent” instead of “use the tool this time.” Long-term **global memory** can also push toward “delegate” when it contains many “Chrome offline” / “unverified” / “no API” lessons.

### How OpenClaw solves this

OpenClaw does **not** have a separate “planning” step that asks the model to “RECOMMEND: your plan.”

- **Single-phase model:** The active agent receives (1) **structured tool schema** (function definitions sent to the model API) and (2) **system prompt text** (TOOLS.md / human-readable list). The model **directly** chooses tool calls from that schema. There is no intermediate “orchestrator says use AGENT vs BROWSER_SCREENSHOT.”
- **Routing is binding-based:** Which agent handles a request is determined by **bindings** (e.g. channel + account → agent), not by the model “recommending” an agent. So “wrong delegation” in our sense does not exist: the routed agent simply has a set of tools and calls them.
- **Tool presentation:** Tools are first-class: allow/deny, profiles (full, messaging, coding, minimal), and **loop-detection** to block repetitive no-progress tool-call patterns. The agent sees both schema and prose; it doesn’t first “plan” then “execute.”
- **Session pruning:** Context is managed by pruning old tool results (and compaction), not by a separate planner that might be biased by history.

## Proposal (mac-stats): general fix

Keep the two-phase flow but make **base tools win** when the request unambiguously matches one.

### 1. Tool-first rule in the planning prompt

Add an explicit instruction so the orchestrator prefers the matching **base tool** over **AGENT** when the user request clearly fits a single tool:

- **Screenshot + URL** → RECOMMEND: **BROWSER_SCREENSHOT: &lt;url&gt;** (not AGENT).
- **Fetch page text** → RECOMMEND: **FETCH_URL: &lt;url&gt;** (not AGENT).
- **Web search** → RECOMMEND: **BRAVE_SEARCH** or **PERPLEXITY_SEARCH** (not AGENT) when it’s a search query.

Wording for the planning prompt:

- *“If the user request can be fulfilled by exactly one base tool (e.g. BROWSER_SCREENSHOT for ‘screenshot this URL’ or ‘go to URL and take a screenshot’, FETCH_URL for ‘fetch/get this page’), recommend that tool directly. Prefer the tool over AGENT unless the task clearly requires multi-step or specialist capability.”*

This is a **general** rule: it applies to any “clear single-tool” case, not only screenshots.

### 2. Pre-route for unambiguous “screenshot + URL”

Like the existing **RUN_CMD** and **REDMINE_API** pre-routes: when the question clearly asks for a screenshot of a URL and we can extract the URL, **skip planning** and set recommendation to **BROWSER_SCREENSHOT: &lt;url&gt;**:

- Detected intent: e.g. “screenshot”, “take a screenshot”, “create a screenshot”, “capture the page” plus “browser”/“chrome”/“goto”/“visit” and a URL.
- URL extraction: `https?://...` or `www....` from the question (strip trailing punctuation).

This guarantees that “Use the Chrome browser, goto https://www.amvara.de and create a screenshot for me” never goes to the planner and always runs BROWSER_SCREENSHOT.

## Summary

| Approach | OpenClaw | mac-stats proposal |
|----------|----------|--------------------|
| Planning step | None; model calls tools directly | Keep; add tool-first rule + pre-route |
| “Wrong RECOMMEND” | N/A (no planner) | Mitigate with rule + deterministic pre-route for screenshot+URL |
| Routing | Config bindings (channel/account → agent) | Model recommends plan; we bias plan toward tools when obvious |