## Installation
### DMG (recommended)
[Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.

### Build from source
```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```
Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

### Gatekeeper workaround
If macOS blocks the app, right-click the DMG → **Open**, then confirm. Or after install: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

## At a Glance
### Menu Bar
- CPU, GPU, RAM, disk at a glance; click to open the details window.

### AI Chat
- Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.

## Tool Agents
Whenever Ollama is asked to decide which agent to use, the app sends the complete list of active agents. Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

### Available Tools
| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

## Log Analysis
### Use the Chrome browser, goto https://www.amvara.de and create a screenshot for me
#### Date: 2026-03-02, 16:40:04 UTC
#### Source: `~/.mac-stats/debug.log`
#### User request: Use the Chrome browser, goto https://www.amvara.de and create a screenshot for me

#### Flow:
1. **Discord → Router**
   Request reached the agent router with **4 prior messages** as conversation context (session memory).
2. **Planning step (orchestrator, qwen2.5:1.5b)**
   - Prompt included: soul, tool list (including BROWSER_SCREENSHOT), agent list, **and the 4 prior messages**:
     - User: "new session: Switch to model qwen3"
     - Assistant: "I'm sorry, but I can't perform that task at this time."
     - User: "use the browser and goto url=www.amvara.de, then wait a little bit and take a screenshot."
     - Assistant: "Alright, let's go! Here's what you can expect: 1. I'll wait for about a minute and then take a screenshot. 2. If there's anything specific..."
   - **Orchestrator reply:** `RECOMMEND: AGENT: general-purpose-mommy` (39 chars).
3. **Execution**
   Router treated this as a direct tool call and ran **AGENT: general-purpose-mommy** (with the same user question). General-purpose-mommy (qwen3-vl:latest) received the full execution prompt (**74,540 chars**, including global memory). It returned a **89‑char text response** (no tool call). That result was fed back to the main loop. **BROWSER_SCREENSHOT was never invoked.**
4. **Outcome**
   User did not get a screenshot; they got the delegated agent’s short text reply instead.

#### Why the model didn’t get it right
### Primary: wrong plan (delegation instead of tool)
- The correct plan for “Chrome, goto URL, create a screenshot” is: **RECOMMEND: BROWSER_SCREENSHOT: https://www.amvara.de**
- The orchestrator chose: **RECOMMEND: AGENT: general-purpose-mommy**
- So the failure is at **planning**: the orchestrator recommended delegation instead of the screenshot tool.

### Secondary: conversation history (prior screenshot attempt)
- The **4 prior messages** in the same Discord session showed:
  - A previous screenshot request: “use the browser and goto url=www.amvara.de, then wait a little bit and take a screenshot.”
  - An assistant reply that **did not use any tool** (no BROWSER_SCREENSHOT, no BROWSER_NAVIGATE), only text (“I'll wait for about a minute and then take a screenshot…”).
- That context can bias the orchestrator to:
  - Treat “screenshot” in this thread as something “handled by conversation” rather than by tool, and/or
  - Prefer “try another agent” instead of “use BROWSER_SCREENSHOT” when the last attempt didn’t use a tool.

### Tertiary: context learned over days (global memory)
- **Execution** prompts (and the general-purpose-mommy agent) receive a very large **global memory** block (“## Memory (lessons learned — follow these)”): hundreds of bullets about Discord, Redmine, sessions, image 404s, “Chrome offline”, “User must start the browser manually”, “BROWSER_SCREENSHOT”, “Tools that worked: … Auto-attaching screenshots…”, etc.
- The **planning** request to the orchestrator in the logs does not clearly show that same full memory block in the system message; planning may use a shorter system prompt. If the orchestrator *does* get a shortened or different memory, “context learned over days” may matter less at plan time. If it *does* get the full memory, then:
  - The volume of “UNVERIFIED”, “No API tools”, “Chrome offline”, “screenshot” lessons could encourage “delegate to an agent” instead of “call BROWSER_SCREENSHOT”.
- So **context learned over days could have contributed** if the orchestrator’s planning prompt includes that global memory; it’s a plausible but less clearly visible factor than conversation history.

## Recommendations
1. **Planning prompt / tool priority**
   In the orchestrator’s instructions, make it explicit that for **“screenshot”, “take a screenshot”, “capture the page”** plus a URL, the plan must be **BROWSER_SCREENSHOT: <URL>** (or BROWSER_NAVIGATE then BROWSER_SCREENSHOT if multi-step). Prefer the tool over AGENT when the task exactly matches a base tool.

2. **Session memory for planning**
   Consider either:
   - Not sending the last N turns of conversation history into the **planning** step when the user request is a clear, single-tool request (e.g. “go to URL X and take a screenshot”), or
   - Sending a short summary instead of full prior assistant text, so a previous “no tool used” reply doesn’t dominate the next plan.

3. **Global memory for planning**
   If the orchestrator receives the full global memory at plan time, consider a **shorter “planning memory”**: only lessons that are about **which tool to choose** (e.g. “for screenshot + URL use BROWSER_SCREENSHOT”) and drop or compress long “UNVERIFIED / no API” segments that might push toward delegation.

4. **Fallback**
   When the plan is **AGENT: general-purpose-mommy** (or similar) and the user message clearly matches a single tool (e.g. “screenshot” + URL), the router could **additionally try** the matching tool (e.g. BROWSER_SCREENSHOT) and append that result, or heuristically suggest re-planning with “prefer BROWSER_SCREENSHOT for screenshot + URL requests”.

## Open tasks:
- Investigate session memory for planning to reduce bias towards delegation.
- Examine global memory for planning to minimize its impact on tool choice.