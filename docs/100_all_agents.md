# All Agents – Overview and Behaviour

This document describes the **agentic behaviour** in mac-stats: who triggers Ollama, which **tools** (agents) Ollama can use, and how each **entry-point agent** (Discord, CPU window, Cursor) works.

---

## 1. Tool agents (what Ollama can invoke)

Whenever Ollama is asked to decide which agent to use (planning step in Discord and scheduler flow), the app sends the **complete list of active agents**: the invocable tools below plus the **SCHEDULER** (informational; Ollama can recommend it for recurring or delayed tasks but cannot invoke it with a tool line). Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in frontend and result sent back to Ollama. In **Discord**: not available; Ollama is told "JavaScript execution is not available in this context." |
| **RUN_CMD** | `RUN_CMD: <command> [args]` | Run a restricted local command (read-only). Use for reading app data under ~/.mac-stats (e.g. schedules.json). | `commands/run_cmd.rs` → `run_local_command()`. Only cat, head, tail, ls; paths must be under ~/.mac-stats. Disabled when `ALLOW_LOCAL_CMD=0`. See `docs/011_local_cmd_agent.md`. |
| **MCP** | `MCP: <tool_name> <arguments>` | Run a tool from the configured MCP server (any server on the internet via HTTP/SSE). | `mcp/` or `commands/mcp.rs` → list tools, `call_tool()`. Requires `MCP_SERVER_URL` (env or `.config.env`). See `docs/010_mcp_agent.md`. |

**Parsing:** The app parses assistant content for lines starting with `FETCH_URL:`, `BRAVE_SEARCH:`, `RUN_JS:`, `RUN_CMD:`, or `MCP:` (see `parse_tool_from_response` in `commands/ollama.rs`). For FETCH_URL and BRAVE_SEARCH, the argument is truncated at the first `;` if Ollama concatenates multiple tools on one line.

---

## 2. Entry-point agents (who calls Ollama and how)

### 2.1 Discord agent (Gateway bot)

- **Docs:** `docs/007_discord_agent.md`
- **Behaviour:** Listens for **DMs** and **@mentions** via Discord Gateway. For each relevant message it calls a **shared "answer with Ollama + tools"** API.
- **Flow:**
  1. **Planning:** Send user question + list of available tools; ask Ollama to reply with `RECOMMEND: <plan>` (which agents to use, in what order). No execution yet.
  2. **Execution:** Send system prompt + agent descriptions + the plan + user question. Then **tool loop**: if the model replies with `FETCH_URL:`, `BRAVE_SEARCH:`, `RUN_JS:`, `RUN_CMD:`, or `MCP:`, the app runs the tool (FETCH_URL/BRAVE_SEARCH/RUN_CMD in Rust; RUN_JS only returns "not available"), appends the result to the conversation, and calls Ollama again. Up to **5 tool iterations**.
- **Rust:** `discord/mod.rs` (EventHandler) → `commands::ollama::answer_with_ollama_and_fetch(question)`. Token from env, `.config.env`, or Keychain (see 007).
- **Result:** Reply is sent back to the same Discord channel. If something fails (e.g. FETCH_URL error), the user sees a short error message and "(Is Ollama configured?)".

### 2.2 CPU window / Chat UI agent

- **Docs:** `agents.md`, `CLAUDE.md` (Ollama + FETCH_URL + code execution).
- **Behaviour:** User types in the CPU window chat. Frontend calls `ollama_chat_with_execution` (with conversation history). No separate "planning" step; the model can directly output `FETCH_URL: <url>` or (in UI) run code via RUN_JS.
- **Flow:** Same tool protocol (FETCH_URL, and RUN_JS in frontend). FETCH_URL is handled in Rust (`browser::fetch_page_content`); content is fed back to Ollama. Up to 3 FETCH_URL iterations in this path. Code execution is handled in JS and followed up with `ollama_chat_continue_with_result`.
- **Rust:** `commands/ollama.rs` → `ollama_chat_with_execution` (and `ollama_chat_continue_with_result` for code follow-up).

### 2.3 Scheduler agent

- **Docs:** `docs/009_scheduler_agent.md`
- **Behaviour:** Runs at app startup; reads `~/.mac-stats/schedules.json` and in a loop sleeps until the next due time, then executes the task. Tasks are either free text (passed to `answer_with_ollama_and_fetch`, so Ollama plans and uses FETCH_URL/BRAVE_SEARCH/RUN_JS) or direct tool lines (`FETCH_URL: <url>` / `BRAVE_SEARCH: <query>` run without Ollama).
- **Rust:** `scheduler/mod.rs` → `spawn_scheduler_thread()`; schedule entries use **cron** (recurring, local time) or **at** (one-shot datetime). File is reloaded periodically so edits take effect without restart.

### 2.4 Brave Search agent (tool only)

- **Docs:** `docs/008_brave_agent.md`
- **Behaviour:** Not an entry point; it's a **tool** Ollama uses when it outputs `BRAVE_SEARCH: <query>`. The app calls Brave Search API, formats results, and injects them into the conversation.
- **Where used:** Discord agent, Scheduler agent, and (when the CPU-window flow uses the same tool loop) the CPU window. Rate limits and headers are documented in 008.

### 2.5 RUN_CMD agent (tool only)

- **Docs:** `docs/011_local_cmd_agent.md`
- **Behaviour:** Not an entry point; it's a **tool** Ollama uses when it outputs `RUN_CMD: <command> [args]`. The app runs a restricted set of local commands (cat, head, tail, ls) with paths under ~/.mac-stats and injects the result into the conversation.
- **Where used:** Discord agent, Scheduler agent, and (when the CPU-window flow uses the same tool loop) the CPU window. Omitted from the agent list when `ALLOW_LOCAL_CMD=0`.

### 2.6 MCP agent (tool only)

- **Docs:** `docs/010_mcp_agent.md`
- **Behaviour:** Not an entry point; it's a **tool** Ollama uses when MCP is configured and the model outputs `MCP: <tool_name> <arguments>`. The app calls the configured MCP server (HTTP/SSE), runs the tool, and injects the result into the conversation.
- **Where used:** Discord agent, Scheduler agent, and (when the CPU-window flow uses the same tool loop) the CPU window. Only active when `MCP_SERVER_URL` is set.

### 2.7 "ask-local-ollama" (Cursor skill)

- **File:** `src-tauri/.claude/agents/ask-local-ollama.md`
- **Behaviour:** Instruction for Cursor to try **local Ollama** (e.g. `http://127.0.0.1:11434/`) for tasks/questions/code before using other models. No direct effect on Discord or CPU-window agents; it only guides the IDE's model usage.

---

## 3. Where each tool is used

| Tool | Discord | CPU window chat | Scheduler |
|------|--------|------------------|-----------|
| FETCH_URL | Yes (via `answer_with_ollama_and_fetch`) | Yes (via `ollama_chat_with_execution`) | Yes (scheduler: direct or via Ollama) |
| BRAVE_SEARCH | Yes | Same pipeline when wired | Yes (scheduler: direct or via Ollama) |
| RUN_JS | No (returns "not available") | Yes (executed in frontend) | No (scheduler runs headless) |
| RUN_CMD | Yes (when allowed) | Same pipeline when wired | Yes (when allowed) |
| MCP | Yes (when configured) | Same pipeline when wired | Yes (when configured) |

---

## 4. FETCH_URL failure when it works in the browser

If Discord (or the app) fails with something like:

`Fetch page failed: Request failed: error sending request for url (https://worldtimeapi.org/api/ip): error trying to connect: connection closed via error`

but the same URL works in a browser, the usual cause is **request shape**, not the URL itself:

- **User-Agent:** The fetch client may send an empty or default reqwest User-Agent; some APIs (including WorldTimeAPI) reject or close the connection. Browsers send a normal UA.
- **TLS/HTTP:** Different ALPN (HTTP/1.1 vs HTTP/2) or connection handling can cause the server to close the connection.

**Fix:** Set a standard `User-Agent` header (e.g. `mac-stats/1.0`) in `commands/browser.rs` on the request. Optionally force HTTP/1.1 if issues persist.

---

## 5. References

- **Roadmap / fetch:** `docs/006_roadmap_ai_tasks.md` (Phase 1: fetch, Discord)
- **Discord setup:** `docs/007_discord_agent.md`
- **Brave Search:** `docs/008_brave_agent.md`
- **Scheduler:** `docs/009_scheduler_agent.md`
- **MCP agent:** `docs/010_mcp_agent.md`
- **RUN_CMD agent:** `docs/011_local_cmd_agent.md`
- **Code:** `src-tauri/src/commands/ollama.rs` (planning + tool loop, parsing), `commands/browser.rs` (FETCH_URL), `commands/brave.rs` (BRAVE_SEARCH), `commands/run_cmd.rs` (RUN_CMD), `src-tauri/src/mcp/` or `commands/mcp.rs` (MCP client), `src-tauri/src/discord/mod.rs` (Gateway + handler), `src-tauri/src/scheduler/mod.rs` (schedule loop + execution)
