## Installation

### DMG (recommended)
[Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.

### Build from source
```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```
Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

### macOS Gatekeeper workaround
Right-click the DMG → **Open**, then confirm. Or after install: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

## At a Glance

* **Menu Bar**: CPU, GPU, RAM, disk at a glance; click to open the details window.
* **AI Chat**: Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.

## Tool Agents

Whenever Ollama is asked to decide which agent to use, the app sends the complete list of active agents. Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

## Task System Fix

### What Happened

1. User asked for a casual chat between all agents (via Discord).
2. Plan produced: `TASK_CREATE: Casual Chat Task 174 "Have a casual chat..." | AGENT: orchestrator`
3. Router executes one tool per response. The first line was `TASK_CREATE`, so the app created the task file and never ran `AGENT: orchestrator`. The chat between agents never happened.
4. Model then replied with natural language (“Great! I've appended the summary…”) instead of outputting `AGENT: orchestrator ...`, so the conversation was never started.
5. In later turns the model repeatedly output `TASK_CREATE` with the same intent (e.g. `TASK_CREATE: Casual Chat between Agents ~/mac-stats/task/casual-chat-agents.md`). Each call created a new task file (new timestamp in filename). With up to 15 tool iterations, this produced many duplicate tasks.

### Root Causes

* Agent chat: Planning said both “create task” and “run AGENT: orchestrator”, but execution only runs one tool per Ollama response. The first tool was TASK_CREATE, so AGENT was never invoked.
* Duplicate tasks: No check for “task with same topic+id already exists”; every TASK_CREATE created a new file.

### Fixes Applied

1. **TASK_CREATE deduplication** (`src-tauri/src/task/mod.rs`):
   - Before creating a task, we check for an existing file with the same topic and id by reading `## Topic:` and `## Id:` from each task file under `~/.mac-stats/task/`.
   - If one exists, `create_task` returns an error: *"A task with this topic and id already exists: ... Use TASK_APPEND or TASK_STATUS to update it."*
   - This stops the loop of creating many identical tasks. (Task filenames are now `task-<date-time>-<status>.md`; topic and id are stored in-file.)

2. **Prompt guidance** (`src-tauri/src/commands/ollama.rs`):
   - **Planning**: Added to the planning prompt: *"If the user wants agents to have a conversation or chat together, your plan must start with AGENT: orchestrator (or the appropriate agent) so the conversation actually runs; do not only create a task file (TASK_CREATE)."*
   - **TASK description**: Added that when the user wants agents to chat, invoke `AGENT: orchestrator` so the conversation runs, and that if a task with that topic+id already exists, use TASK_APPEND or TASK_STATUS instead of TASK_CREATE.

## References

- Task agent: `docs/013_task_agent.md`
- LLM agents / AGENT tool: `docs/017_llm_agents.md`
- Review loop (scheduler/default): `src-tauri/src/task/review.rs`