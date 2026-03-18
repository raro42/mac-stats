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

* **Menu bar**: CPU, GPU, RAM, disk at a glance; click to open the details window.
* **AI Chat**: Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.

## Tool Agents

Whenever Ollama is asked to decide which agent to use, the app sends the complete list of active agents. Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

## Router and Agents

### Short Answers

1. **Can agent-000 (or any other agent) explicitly call the router to execute things on the API?**
   Only the entry-point model can. When a request starts (e.g. from Discord with `agent: 000`), that agent is the one in the router loop: it doesn’t “call” the router as a separate service; it is the model the router is talking to. So agent-000 can “call the router” in the sense that it outputs tool lines (`FETCH_URL: ...`, `TASK_CREATE: ...`, `AGENT: 002 ...`) and the **router** (the Rust code in `answer_with_ollama_and_fetch`) executes them.
   Sub-agents (e.g. 001, 002) cannot. When the orchestrator outputs `AGENT: 002 <task>`, the app runs that agent in a **single** Ollama request (no tool list, no router). So 002 cannot output `FETCH_URL:` or `TASK_APPEND:` and have it executed.

2. **Is the router involved in the discussion between agents?**
   Yes. The **router is the only thing that talks to the entry-point model** and runs tools. When the entry-point outputs `AGENT: 002 <task>`, the router runs 002 (one turn), gets the reply, and injects it back into the router’s conversation with the entry-point. So the “discussion” is: **router ↔ entry-point (orchestrator)**. Sub-agents do not talk to the router and do not talk to each other; they only receive a task and return a single reply.

3. **How do agents and the API router work together?**
   - **One** “main” conversation: router (Rust) + **entry-point model** (e.g. agent-000 when overridden, or default Ollama model).
   - The entry-point gets the **full tool list** (FETCH_URL, BRAVE_SEARCH, AGENT, TASK_*, SCHEDULE, RUN_CMD, OLLAMA_API, etc.) and runs in a **tool loop**: reply → parse tool line → execute → inject result → call model again.
   - **AGENT** is one of those tools: when the entry-point replies `AGENT: <id or slug> <task>`, the router calls `run_agent_ollama_session(agent, task)` — a **single** Ollama request with that agent’s prompt and model, **no** tool list. The sub-agent’s reply is then injected back to the entry-point.
   - So: **router** = executor of all tools; **entry-point agent** = the only “mind” that can ask the router to run tools (including running other agents). **Sub-agents** = tools that return text; they have no API.

## Flow Diagram (Conceptual)

```
User (e.g. Discord)
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│  ROUTER (answer_with_ollama_and_fetch)                          │
│  - Builds tool list (FETCH_URL, AGENT, TASK_*, SCHEDULE, …)     │
│  - One “main” conversation with the ENTRY-POINT model           │
│  - Tool loop: get reply → if tool line → execute → inject result │
└─────────────────────────────────────────────────────────────────┘
    │
    │  entry-point = agent_override (e.g. agent-000) or default
    ▼
┌─────────────────────────────────────────────────────────────────┐
│  ENTRY-POINT MODEL (e.g. orchestrator / agent-000)              │
│  - Sees full API; can output: FETCH_URL:, TASK_CREATE:,         │
│    AGENT: 002 <task>, etc.                                       │
│  - When it outputs AGENT: 002 …, router runs sub-agent 002      │
└─────────────────────────────────────────────────────────────────┘
    │
    │  when entry-point outputs "AGENT: 002 <task>"
    ▼
┌─────────────────────────────────────────────────────────────────┐
│  run_agent_ollama_session(agent_002, task)                       │
│  - Single Ollama request: system=002’s prompt, user=task        │
│  - No tool list; 002 cannot call FETCH_URL, TASK_*, etc.         │
│  - Returns one reply → router injects it back to entry-point     │
└─────────────────────────────────────────────────────────────────┘
```

## Implications

- **Orchestrator (e.g. agent-000)** should have the Router API Commands in its `skill.md` so it knows how to use FETCH_URL, TASK_*, AGENT, etc. It is the only agent that gets to use those commands in that request.
- **Specialist agents (001, 002, 003)** only need to do their job in one turn; they cannot schedule, create tasks, or call other agents via the router. If a specialist “needs” to do that, the orchestrator must do it (e.g. orchestrator calls 002, gets result, then orchestrator does TASK_APPEND or SCHEDULE).
- There is no “agent calling the router as an API” in a separate request: the router is the loop that drives the entry-point. So “explicitly call the router” = the entry-point outputting tool lines in that same conversation.

## Specialist agents

**Definition**: Any agent that is **not** the entry-point is a specialist from the router’s perspective. The entry-point (e.g. agent-000) is the only one in the tool loop; all others are invoked via `AGENT: <id or slug or name> <task>` and run in a **single** Ollama request with **no** tool list.

**Invocation**: The orchestrator (or any entry-point) outputs one line: `AGENT: 002 write a small Python script` or `AGENT: senior-coder refactor this function`. The router resolves the selector (id `002`, slug `senior-coder`, or name) to an agent, then calls `run_agent_ollama_session(agent, task)`.

**What the specialist receives**:
- **User message**: Only the `<task>` string (e.g. “write a small Python script”). No main-conversation history, no tool list.
- **System prompt**: That agent’s assembled prompt (soul + mood + skill from `~/.mac-stats/agents/agent-NNN-<name>/`).

**Where they live**: Under `~/.mac-stats/agents/` (and defaults in the app under `src-tauri/defaults/agents/`). Each agent has `agent.json`, `skill.md`, `mood.md`, `soul.md`, and optionally `testing.md`. See **docs/017_llm_agents.md** for the default agent table and roles.

**Default specialists** (orchestrator 000 delegates to these):

| ID   | Name / slug           | Role / purpose        |
|------|------------------------|------------------------|
| 001  | General Assistant      | General Q&A             |
| 002  | Coder (senior-coder)   | Code generation        |
| 003  | Generalist             | Fast replies           |
| 004  | Discord Expert         | Discord API specialist |
| 005  | Task Runner            | Task file execution    |
| 006  | Redmine                | Redmine ticket review/search/create/update |

**Limitation**: Specialists cannot call FETCH_URL, TASK_*, SCHEDULE, or another AGENT. If the task requires tools or multi-step work, the orchestrator must run the tool (or another agent) and use the result in the main conversation.

## Agent initialization and model resolution

Agents are **loaded from disk on each use** (`agents::load_agents()`): no in-memory cache of the list. Each call reads `~/.mac-stats/agents/`, discovers `agent-<id>` directories, and for each loads `agent.json` and `skill.md` (required), plus optional `soul.md`, `mood.md`, and memory files. Default agents are written by `Config::ensure_defaults()` at startup (agent.json only if missing; skill/testing overwritten from bundle).

**Model resolution** runs when a **ModelCatalog** is available. The catalog is built at startup by `ensure_ollama_agent_ready_at_startup()` (async): it fetches Ollama `/api/tags`, classifies models (general, code, small, vision, etc.), and caches the catalog. When `load_agents()` runs, it calls `resolve_agent_models(agents, catalog)` if the catalog is set; that resolves each agent’s `model_role` (e.g. `"general"`, `"code"`) to an actual model name. If the catalog is **not** set yet (e.g. Ollama still starting or first request before startup task completed), model resolution is skipped and agents with only `model_role` (no explicit `model`) keep `model = None` until the next load after the catalog is ready. The app logs: `Agents: model catalog not yet available, model_role resolution skipped (Ollama may still be starting)` when this happens.

**Startup order**: `ensure_defaults()` runs synchronously; then `ensure_ollama_agent_ready_at_startup()` is spawned (async). So the first few requests (e.g. Discord or CPU window) might call `load_agents()` before the catalog exists. In practice the startup task usually completes within seconds. If Ollama is down or slow, agents may temporarily have no resolved model; once the catalog is set, every subsequent `load_agents()` will resolve correctly.

**Failure modes** that prevent an agent from loading (and are logged): missing or unreadable `agent.json`, invalid JSON in `agent.json`, missing or empty `skill.md`. Disabled agents (`enabled: false`) are skipped with a debug log.

## References

- **Tool loop**: `commands/ollama.rs` → `answer_with_ollama_and_fetch`, `parse_tool_from_response`, `parse_all_tools_from_response`, and the `while tool_count < max_tool_iterations` loop. Plans like `RUN_CMD: date then REDMINE_API GET /time_entries.json?...` are normalized and split into separate steps so each tool runs in sequence (not one RUN_CMD with the whole chain).
- **Sub-agent run (no tools)**: `run_agent_ollama_session` in `commands/ollama.rs` (single request, no tool list).
- **Router API snippet for orchestrator**: `docs/agent_000_router_commands_snippet.md`, `docs/017_llm_agents.md`.

## Open tasks:
- ~~Investigate why some agents are not being properly initialized.~~ **Done:** § "Agent initialization and model resolution" above (load from disk each time; model resolution depends on ModelCatalog from `ensure_ollama_agent_ready_at_startup`; race when catalog not yet set; failure modes logged). Added log when catalog missing in `agents/mod.rs`.
- ~~Improve the documentation for specialist agents.~~ **Done:** § "Specialist agents" above (definition, invocation, what they receive, where they live, default table, limitation).
- Consider adding support for more advanced tool commands.