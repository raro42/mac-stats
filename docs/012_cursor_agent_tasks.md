## Installation

### DMG (recommended)
[Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.

### Build from source
```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```
Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

### If macOS blocks the app:
Gatekeeper may show "damaged" or block the unsigned app—the file is fine. Right-click the DMG → **Open**, then confirm. Or after install: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

## At a Glance

- **Menu Bar**: CPU, GPU, RAM, disk at a glance; click to open the details window.
- **AI Chat**: Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.

## Tool Agents (What Ollama Can Invoke)

Whenever Ollama is asked to decide which agent to use, the app sends the complete list of active agents. Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

## Using Cursor-Agent CLI for Tasks

### Current State

#### Already Wired

1. **CURSOR_AGENT tool**: When `cursor-agent` is on PATH, the orchestrator (and any agent using the tool loop) sees:
   - **CURSOR_AGENT:** `<prompt>` — runs cursor-agent in the configured workspace with `--print --trust --output-format text`, returns stdout to the conversation.

2. **RUN_CMD allowlist**: `cursor-agent` is in the default allowlist (and in `~/.mac-stats/agents/agent-000/skill.md`). The agent can also run e.g. `RUN_CMD: cursor-agent --help` or `RUN_CMD: cursor-agent <args>` (path rules don’t apply to cursor-agent). **Security:** RUN_CMD’s cursor-agent runs arbitrary prompts in the user environment; it is a privileged capability. To disable it, remove `cursor-agent` from the RUN_CMD allowlist in the orchestrator’s `skill.md`.

3. **Task Runner Uses Full Tool Loop**: When the scheduler runs `TASK: <path_or_id>` (or `TASK_RUN: <path_or_id>`), or when you run a task until finished, the code uses `answer_with_ollama_and_fetch`. That includes **all** tools (TASK_*, RUN_CMD, CURSOR_AGENT, etc.). So the model **can** already:
   - Read the task file content (it’s in the prompt).
   - Reply with `CURSOR_AGENT: <implement this: …>`.
   - Then `TASK_APPEND` with the result and `TASK_STATUS: <id> finished`.

4. **Config**:
   - **CURSOR_AGENT_WORKSPACE**: env or `.config.env` (cwd, `src-tauri/`, `~/.mac-stats/.config.env`). Default: `$HOME/projects/mac-stats` if that dir exists, else `.`.
   - **CURSOR_AGENT_MODEL**: optional; passed as `--model` to cursor-agent.

### What You Need to Do

#### 1. Install cursor-agent and put it on PATH

- The app only offers **CURSOR_AGENT** when `which cursor-agent` succeeds.
- Install the Cursor Agent CLI so that `cursor-agent` is available in the environment used to run mac-stats (e.g. the same terminal or launch context).
- Verify: `cursor-agent --help` (and ideally `cursor-agent --print --trust --output-format text "echo hello"` in a test dir).

#### 2. (Optional) Set workspace and model

- In `~/.mac-stats/.config.env` (or env):
  - `CURSOR_AGENT_WORKSPACE=/path/to/your/code` — directory cursor-agent will run in.
  - `CURSOR_AGENT_MODEL=...` — if you want a specific model.
- If unset, workspace defaults to `$HOME/projects/mac-stats` when that directory exists.

#### 3. (Optional) Guide the model to use CURSOR_AGENT for coding tasks

The task runner prompt says: *"Decide the next step. Use TASK_APPEND to add feedback and TASK_STATUS to set wip or finished when done."* It does **not** mention CURSOR_AGENT. So the model might not choose it for “implement this” style tasks.

To make that more likely:

- **Option A – Task runner prompt**: In `src-tauri/src/task/runner.rs`, extend the `question` to mention that for coding tasks (implement, refactor, fix, add feature) the agent may use **CURSOR_AGENT:** `<prompt>` to delegate to the Cursor Agent CLI, then TASK_APPEND the result and TASK_STATUS finished.
- **Option B – Orchestrator skill**: The default orchestrator skill (`src-tauri/defaults/agents/agent-000/skill.md`, synced to `~/.mac-stats/agents/agent-000/skill.md`) now includes a section **Implementation tasks: prefer CURSOR_AGENT** recommending CURSOR_AGENT for code changes or implementation work. If you use a custom orchestrator skill, add the same short rule: for task files that describe code changes or implementation work, use CURSOR_AGENT with the task description, then update the task with the result and set status to finished.
- **Option C – Task file convention**: In task content, include a line like “Implement using Cursor Agent” or “Use CURSOR_AGENT for this.” The model will then see the hint in the task text.

#### 4. (Optional) Workspace per task

Right now cursor-agent always runs in **one** workspace (from config). There is no syntax like `CURSOR_AGENT: workspace:/other/proj <prompt>`. If you need different workspaces per task:

- Either put the desired project path in the task text and add a convention in the orchestrator skill (e.g. “If the task says ‘project: /path/to/X’, run cursor-agent in that directory”) — then you’d need a **code change** to pass `--workspace` from the task (e.g. parse a prefix or a special line and call `run_cursor_agent` with a different workspace).
- Or use different `CURSOR_AGENT_WORKSPACE` values per environment (e.g. per user or per schedule) and keep one workspace per run.

## Summary

| Requirement | Status |
|------------|--------|
| cursor-agent on PATH | **You must ensure this.** |
| CURSOR_AGENT tool in tool loop | Done. |
| Task runner uses tool loop (so CURSOR_AGENT is available) | Done. |
| RUN_CMD allows cursor-agent | Done (default + skill.md). |
| Prompt/skill hints to use CURSOR_AGENT for coding tasks | Optional; improves likelihood the model delegates. |
| Per-task or per-invocation workspace | Not implemented; optional enhancement. |

## Open tasks:

See **006-feature-coder/FEATURE-CODER.md** for the current FEAT backlog.

- ~~Ensure that the model uses CURSOR_AGENT for coding tasks by default.~~ Deferred: future/backlog (model delegation depends on prompt quality; orchestrator skill.md can be tuned to prefer CURSOR_AGENT for code).
- ~~Implement workspace per task.~~ Deferred: future/backlog (would require task-specific directory management; not needed for current single-workspace usage).
- ~~Add a task file convention to hint the model to use CURSOR_AGENT.~~ Deferred: future/backlog (task files can include a "tool hint" field when this is scoped).