## Installation

### DMG (recommended)
[Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.

### Build from source:
```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```
Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

### Gatekeeper workaround:
Right-click the DMG → **Open**, then confirm. Or after install: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

## At a Glance

- **Menu bar**: CPU, GPU, RAM, disk at a glance; click to open the details window.
- **AI chat**: Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.

## Tool Agents

Whenever Ollama is asked to decide which agent to use, the app sends the **complete list of active agents**: the invocable tools below plus the **SCHEDULER** (informational; Ollama can recommend it for recurring or delayed tasks but cannot invoke it with a tool line). Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

## Tool-First Fix

The fix for “orchestrator recommends AGENT instead of tool” is **implemented** for the minimal scope (proposal [031](031_orchestrator_tool_first_proposal_DONE.md)).

### Already Implemented

| Step | What | Where |
|------|------|--------|
| 1 | **Screenshot + URL pre-route** | `src-tauri/src/commands/ollama.rs`: `extract_url_from_question()`, `extract_screenshot_recommendation()`, and pre-routing runs screenshot first, then RUN_CMD, then REDMINE. Requests like “Use Chrome, goto https://example.com and create a screenshot” skip the planner and go straight to BROWSER_SCREENSHOT. |
| 2 | **Tool-first rule in default planning prompt** | `src-tauri/defaults/prompts/planning_prompt.md`: paragraph “**Tool-first rule:** If the user request can be fulfilled by exactly one base tool … recommend that tool directly … Prefer the tool over AGENT …” |
| 3 | **Proposal + OpenClaw references** | `docs/031_orchestrator_tool_first_proposal_DONE.md`: problem, OpenClaw comparison, codebase references, and mac-stats proposal. |

## Remaining Baby Steps

### Step 4 — Changelog (recommended)
- [x] In `CHANGELOG.md` under `[Unreleased]` add:
  - **Tool-first routing:** Pre-route “screenshot + URL” requests to BROWSER_SCREENSHOT (skip planner). Default planning prompt now includes a tool-first rule: when one base tool fits the request, recommend that tool instead of AGENT. See `docs/031_orchestrator_tool_first_proposal_DONE.md`.

### Step 5 — Document custom planning prompt (optional)
- Optional future extension: in the proposal doc or in a short “merge defaults” note, explain that custom `~/.mac-stats/prompts/planning_prompt.md` files should include the tool-first paragraph from `defaults/prompts/planning_prompt.md` so the planner prefers tools over AGENT when the request clearly matches one tool.

### Step 6 — Append tool-first rule when loading (optional)
- Optional future extension: in `Config::load_planning_prompt()`, if the loaded content does not contain `"Tool-first"` (or a chosen marker), append the default tool-first paragraph so custom prompts still get the rule without manual edit. Decide whether to append always or only when file is user-provided (e.g. path exists and content differs from default).

### Step 7 — Reduce planning context (optional, proposal §3)
- Optional future extension: for the planning step only, either send no conversation history or only the last user message (or a short summary of recent turns) so prior “no tool used” replies do not bias the planner toward AGENT. This would require changing what `answer_with_ollama_and_fetch` passes into the planning messages.

### Step 8 — Pre-route FETCH_URL + URL (done)
- [x] **Done:** `try_pre_route_fetch_url()` in `commands/pre_routing.rs` detects explicit `FETCH_URL:` prefix and keyword-based patterns ("fetch", "get the page/content/html", "read the page/url/site", "what's on", "summarize the/this page/url/site") combined with a URL. Browser/navigate/screenshot patterns are excluded. Wired into the pre-route chain after RUN_CMD, before Redmine. 17 tests.