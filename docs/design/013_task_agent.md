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

- **Menu bar** — CPU, GPU, RAM, disk at a glance; click to open the details window.
- **AI chat** — Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.
- **Discord bot** — Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.

## Tool Agents (what Ollama can invoke)

Whenever Ollama is asked to decide which agent to use, the app sends the complete list of active agents. Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

## Task Agent (task files and loop until finished)

The task agent lets you work on tasks stored as Markdown files under `~/.mac-stats/task/`. All task logic lives in the **task** module (`task/mod.rs`, `task/runner.rs`, `task/review.rs`, `task/cli.rs`). Ollama and the scheduler call into the task module; they do not contain task file I/O or business rules.

### Overview

- **Task directory**: `~/.mac-stats/task/`
- **File naming**: `task-<date-time>-<status>.md`
  - `date-time`: `%Y%m%d-%H%M%S` (e.g. `20260222-140215`)
  - **Status** in filename: `open`, `wip`, `finished`, `unsuccessful`, or `paused`
- **In-file metadata**: `## Assigned: agent_id`, `## Topic: <topic>`, `## Id: <id>` (topic and id are stored in the file, not in the filename); optional `## Paused until: ISO datetime`, `## Depends: id1, id2`, `## Sub-tasks: id1, id2`
- **Writing**: Ollama uses **TASK_LIST**, **TASK_LIST: all**, **TASK_SHOW**, **TASK_APPEND**, **TASK_STATUS**, **TASK_CREATE**, **TASK_ASSIGN**, **TASK_SLEEP**. Paths must be under `~/.mac-stats/task/`.

## CLI (test from command line)

Run task operations without starting the app:

```bash
mac_stats add <topic> <id> [content]   # Create task (default assignee: default)
mac_stats list [--all]                 # List open+WIP, or all by status (incl. assignee)
mac_stats show <id>                    # Print status, assignee, path, content
mac_stats status <id> [open|wip|finished|unsuccessful|paused]  # Get or set status
mac_stats remove <id>                  # Delete all status files for that task
mac_stats assign <id> <agent>          # Assign to scheduler|discord|cpu|default
mac_stats append <id> <content>        # Append feedback block
```

## Tools (Ollama invocations)

| Tool | Invocation | Purpose |
|------|------------|---------|
| **TASK_LIST** | `TASK_LIST` or `TASK_LIST:` | List open and WIP tasks (with assignee). Sent to the message channel. |
| **TASK_LIST: all** | `TASK_LIST: all` | List all tasks grouped by status: Open, WIP, Finished, Unsuccessful, Paused (with assignee). Sent to the channel. |
| **TASK_SHOW** | `TASK_SHOW: <path or id>` | Show one task's status, assignee, and full content to the user in the channel. |
| **TASK_APPEND** | `TASK_APPEND: <path or id> <content>` | Append a `## Feedback <timestamp>` block. |
| **TASK_STATUS** | `TASK_STATUS: <path or id> wip\|finished\|unsuccessful\|paused` | Set status by renaming the file. If setting `finished`, all sub-tasks (## Sub-tasks: ...) must be finished or unsuccessful. |
| **TASK_CREATE** | `TASK_CREATE: <topic> <id> <initial content>` | Create a new task (status `open`, `## Assigned: default`). |
| **TASK_ASSIGN** | `TASK_ASSIGN: <path or id> <agent_id>` | Reassign task to `scheduler`, `discord`, `cpu`, or `default`. Appends "Reassigned to X." |
| **TASK_SLEEP** | `TASK_SLEEP: <path or id> until <ISO datetime>` | Set status to `paused` and write `## Paused until: <datetime>`. Review loop will auto-resume after that time. |

## Module Layout

- **task/mod.rs**: Core — create, read, append, set_status, resolve_path, list, format_list, assignee, paused_until, depends_on, is_ready, sub_tasks, all_sub_tasks_closed, delete_task, show_task_content.
- **task/runner.rs**: `run_task_until_finished(path, max_iterations)` — loop that calls Ollama and re-reads task until finished or max iterations (then auto-close as unsuccessful).
- **task/review.rs**: Review loop — close_stale_wips, resume_paused_tasks, pick_one_open_task (filter by assignee and is_ready), run up to 3 tasks per cycle via runner.
- **task/cli.rs**: CLI subcommands (add, list, show, status, remove, assign, append) invoked from `main` when the user runs `mac_stats add ...` etc.
- **commands/ollama.rs**: Thin — parses TASK_* tool lines and calls one task function per tool; no task file I/O.
- **scheduler/mod.rs**: For TASK:/TASK_RUN:, resolves path and calls `task::runner::run_task_until_finished(path, 10)`.