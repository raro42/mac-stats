## Installation
### Recommended Method
Download the latest release from [GitHub](https://github.com/raro42/mac-stats/releases/latest) and drag the `.dmg` file to the Applications folder.

### Building from Source
```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```
Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

### Workaround for Gatekeeper
If macOS blocks the app, right-click the `.dmg` file and select **Open**, then confirm. Alternatively, run `xattr -rd com.apple.quarantine /Applications/mac-stats.app` after installation.

## At a Glance
### Menu Bar
- Displays CPU, GPU, RAM, and disk usage at a glance.
- Click to open the details window.

### AI Chat
- Ollama chat available in the app or via Discord.
- Supports FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, and MCP.

## Tool Agents

### How invocations work

Ollama invokes tools by replying with **exactly one line** in the form `TOOL_NAME: <argument>`. The app sends the full list of active agents (and optional **SCHEDULER** as informational) when doing the planning step (Discord and scheduler flow). Tool results are appended to the conversation and the model is called again; this repeats until the model replies with `DONE:` or the iteration limit is reached. See **docs/007_discord_agent.md** (§1) and **docs/100_all_agents.md** for full details.

### Tool list (invocation and purpose)

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch web page body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest, 15s timeout). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search; results injected for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY`. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). In Discord, JS is not executed. | CPU window: frontend runs code and returns result; Discord: "not available" message. |
| **BROWSER_SCREENSHOT** | `BROWSER_SCREENSHOT: current` | Screenshot of current page only. Navigate first with BROWSER_NAVIGATE. | `browser_agent/`; PNG saved to `~/.mac-stats/screenshots/`. |
| **BROWSER_NAVIGATE** | `BROWSER_NAVIGATE: <url>` | Open URL; returns page state with numbered elements. | CDP or HTTP fallback; optional `new_tab` for new tab. |
| **BROWSER_GO_BACK** | `BROWSER_GO_BACK` | Go back one step in tab history. | `browser_agent/`. |
| **BROWSER_CLICK** | `BROWSER_CLICK: <index>` | Click element at 1-based index from Elements list. | `browser_agent/`. |
| **BROWSER_INPUT** | `BROWSER_INPUT: <index> <text>` | Type text into element at index. | `browser_agent/`. |
| **BROWSER_SCROLL** | `BROWSER_SCROLL: <direction>` | Scroll (down, up, bottom, top, or pixels). | `browser_agent/`. |
| **BROWSER_EXTRACT** | `BROWSER_EXTRACT` | Return full visible text of current page. | `browser_agent/`. |
| **BROWSER_SEARCH_PAGE** | `BROWSER_SEARCH_PAGE: <pattern>` | Search page text for pattern (grep-like). | `browser_agent/`. |
| **SCHEDULE** | `SCHEDULE: every N minutes <task>`, `SCHEDULE: <cron> <task>`, `SCHEDULE: at <datetime> <task>` | Add recurring or one-shot task to `schedules.json`. | `scheduler/`; returns schedule ID. |
| **REMOVE_SCHEDULE** | `REMOVE_SCHEDULE: <schedule-id>` | Remove a schedule by ID. | `commands/ollama.rs` (tool loop). |
| **LIST_SCHEDULES** | `LIST_SCHEDULES` or `LIST_SCHEDULES:` | List active schedules (id, type, next run, task). | `commands/ollama.rs`. |
| **RUN_CMD** | `RUN_CMD: <command> [args]` | Run allowlisted local command (read-only). | `commands/run_cmd.rs`. Disabled when `ALLOW_LOCAL_CMD=0`. |
| **TASK_LIST** | `TASK_LIST` or `TASK_LIST: all` | List tasks (default: open/wip; `all` = all statuses). | `task/`, `commands/ollama.rs`. |
| **TASK_SHOW** | `TASK_SHOW: <path or id>` | Show task content and status. | `commands/ollama.rs`. |
| **TASK_APPEND** | `TASK_APPEND: <path or id> <content>` | Append feedback to task. | `commands/ollama.rs`. |
| **TASK_STATUS** | `TASK_STATUS: <path or id> wip/finished/unsuccessful` | Set task status. | `commands/ollama.rs`. |
| **TASK_CREATE** | `TASK_CREATE: <topic> <id> <content>` | Create new task file. | `commands/ollama.rs`. |
| **TASK_ASSIGN** | `TASK_ASSIGN: <path or id> <agent_id>` | Assign task to agent (scheduler, discord, cpu, default). | `commands/ollama.rs`. |
| **OLLAMA_API** | `OLLAMA_API: <action> [args]` | List models, version, running, pull, delete, embed, load, unload. | `commands/ollama.rs`. |
| **PERPLEXITY_SEARCH** | `PERPLEXITY_SEARCH: <query>` | Web search via Perplexity API. | `commands/perplexity.rs`. When configured. |
| **PYTHON_SCRIPT** | `PYTHON_SCRIPT: <id> <topic>` + code block | Run Python script; stdout/error returned. | `commands/python_agent.rs`. When `ALLOW_PYTHON_SCRIPT` not 0. |
| **DISCORD_API** | `DISCORD_API: GET <path>` or `POST <path> [body]` | Call Discord REST API (Discord context only). | `commands/ollama.rs` when from Discord. |
| **CURSOR_AGENT** | `CURSOR_AGENT: <prompt>` | Delegate coding to Cursor Agent CLI. | `commands/cursor_agent.rs`. When CLI available. |
| **REDMINE_API** | `REDMINE_API: GET/POST/PUT <path> [body]` | Redmine issues, time entries, search. | `redmine/`. When configured. |
| **SKILL** | `SKILL: <number or topic> [task]` | Run skill in separate Ollama session; result injected back. | `skills/`, `commands/ollama.rs`. When skills exist. |
| **AGENT** | `AGENT: <slug or id> [task]` | Run specialized LLM agent. | `agents/`, `commands/ollama.rs`. When agents exist. |
| **MCP** | `MCP: <tool_name> <args>` | Run tool from configured MCP server. | `mcp/`. When `MCP_SERVER_URL` set. |
| **MEMORY_APPEND** | `MEMORY_APPEND: <lesson>` or `MEMORY_APPEND: agent:<id> <lesson>` | Save lesson for future sessions (channel or agent). | `commands/ollama.rs`. |
| **MASTODON_POST** | `MASTODON_POST: [visibility:] <text>` | Post status to Mastodon. | When Mastodon configured. |
| **DONE** | `DONE: success` or `DONE: no` | Signal task completed or could not complete; stops tool loop. | Parsed in tool loop. |

### See also

- **docs/README.md** — Tool Agents section and doc index.
- **docs/007_discord_agent.md** — §1 Tool agents, §8 SCHEDULE/REMOVE_SCHEDULE, Discord API and setup.
- **docs/100_all_agents.md** — Full tool list, entry-point agents, and flow.

## Agent Workflow
This repo is edited by the **mac-stats-reviewer** Coder agent. Changes are applied **in place** (yolo mode): the Coder edits files directly and does not produce patches.

## Direct Edits (yolo mode)
For the Coder to edit files in this repo directly:
1. Open Cursor with **`mac-stats-agent-workspace.code-workspace`** (located in the mac-stats-reviewer repo).
2. That workspace includes both **mac-stats** and **mac-stats-reviewer**, so the Coder can edit files here in place.
3. Do **not** produce patches or ask for permission; edit files directly.

## Open tasks:
- ~~Improve the documentation for tool agents and their invocations.~~ **Done:** added "How invocations work", full tool table with invocation syntax, and See also links (007, README, 100_all_agents).
- Consider implementing a more robust patching system for the Coder agent.
- Review the Brave Search API usage and ensure it complies with the Brave API key requirements.