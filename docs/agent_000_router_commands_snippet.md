# Snippet: add this to agent-000 (orchestrator) skill.md

Copy the section below into `~/.mac-stats/agents/agent-000/skill.md` so the orchestrator can ask and use Router API commands.

---

## Router API Commands

You can invoke these by replying with **exactly one line** in the form `COMMAND: <args>`.

| Command | Example | Use when |
|--------|---------|----------|
| **AGENT:** | `AGENT: senior-coder write a small Python script` | Delegate to a specialist agent (id, slug, or name). |
| **FETCH_URL:** | `FETCH_URL: https://example.com/page` | Get the text content of a web page. |
| **BRAVE_SEARCH:** | `BRAVE_SEARCH: current weather Berlin` | Web search (Brave API). |
| **RUN_CMD:** | `RUN_CMD: cat ~/.mac-stats/schedules.json` | Read app data under ~/.mac-stats or run date/whoami. |
| **TASK_LIST** | `TASK_LIST` or `TASK_LIST: all` | List tasks (default: open/WIP; use `all` for every status). |
| **TASK_SHOW:** | `TASK_SHOW: <id>` | Show one task's content and status. |
| **TASK_CREATE:** | `TASK_CREATE: <topic> <id> <content>` | Create a new task file. |
| **TASK_APPEND:** | `TASK_APPEND: <path or id> <content>` | Append feedback to a task. |
| **TASK_STATUS:** | `TASK_STATUS: <path or id> wip\|finished\|unsuccessful` | Set task status. |
| **TASK_ASSIGN:** | `TASK_ASSIGN: <path or id> discord` | Reassign task to scheduler, discord, cpu, or default. |
| **SCHEDULE:** | `SCHEDULE: every 5 minutes <task>` or `SCHEDULE: 0 0 8 * * * <task>` or `SCHEDULE: at 2025-02-10T09:00:00 <task>` | Add recurring (cron) or one-shot (at) schedule. |
| **REMOVE_SCHEDULE:** | `REMOVE_SCHEDULE: discord-123` | Remove a schedule by ID. |
| **OLLAMA_API:** | `OLLAMA_API: list_models` or `OLLAMA_API: version` | List models, version, running, pull/delete/load/unload, embed. |
| **SKILL:** | `SKILL: 2 <task>` or `SKILL: summarize <task>` | Run a skill by number or topic. |
| **MCP:** | `MCP: <tool_name> <args>` | Call an MCP server tool (if configured). |
| **DISCORD_API:** | `DISCORD_API: GET /users/@me/guilds` | Discord API (only when the request is from Discord). |

Paths for tasks must be under `~/.mac-stats/task/`. After we run a command, we give you the result; you can then reply to the user or invoke another command.
