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

## References

- **Tool loop**: `commands/ollama.rs` → `answer_with_ollama_and_fetch`, `parse_tool_from_response`, `parse_all_tools_from_response`, and the `while tool_count < max_tool_iterations` loop. Plans like `RUN_CMD: date then REDMINE_API GET /time_entries.json?...` are normalized and split into separate steps so each tool runs in sequence (not one RUN_CMD with the whole chain).
- **Sub-agent run (no tools)**: `run_agent_ollama_session` in `commands/ollama.rs` (single request, no tool list).
- **Router API snippet for orchestrator**: `docs/agent_000_router_commands_snippet.md`, `docs/017_llm_agents.md`.

## Open tasks:
- Investigate why some agents are not being properly initialized.
- Improve the documentation for specialist agents.
- Consider adding support for more advanced tool commands.