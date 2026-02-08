# Task Agent (task files and loop until finished)

The task agent lets you work on tasks stored as Markdown files under `~/.mac-stats/task/`. Ollama can create tasks, append feedback, and set status. A **task loop** runs a task with Ollama until the status is `finished`.

## Overview

- **Task directory**: `~/.mac-stats/task/`
- **File naming**: `task-<topic>-<id>-<date-time>-<open|wip|finished|unsuccessful>.md`
  - `topic`: short slug (e.g. `refactor-api`)
  - `id`: short id (e.g. `1`, `2`, or a label)
  - `date-time`: `%Y%m%d-%H%M%S` (e.g. `20250208-143000`)
  - **Status** is part of the filename: `open`, `wip`, `finished`, or `unsuccessful`
- **Reading**: Use RUN_CMD to read task files (e.g. `RUN_CMD: cat ~/.mac-stats/task/task-foo-1-20250208-143000-open.md` or `RUN_CMD: grep pattern ~/.mac-stats/task/*.md`).
- **Writing**: Ollama uses **TASK_LIST**, **TASK_APPEND**, **TASK_STATUS**, and **TASK_CREATE** (no shell or raw sed).

## Tools (Ollama invocations)

| Tool | Invocation | Purpose |
|------|------------|---------|
| **TASK_LIST** | `TASK_LIST` or `TASK_LIST:` | Get the list of open and WIP tasks (filenames under ~/.mac-stats/task/). Use when the user asks for open tasks or what tasks exist. |
| **TASK_APPEND** | `TASK_APPEND: <path or task id> <content>` | Append a feedback block to the task file (adds `## Feedback <timestamp>` and the content). |
| **TASK_STATUS** | `TASK_STATUS: <path or task id> wip\|finished` | Set status by renaming the file (e.g. `...-open.md` → `...-wip.md` or `...-finished.md`). |
| **TASK_CREATE** | `TASK_CREATE: <topic> <id> <initial content>` | Create a new task file with status `open` and return its path. |

- **Path or task id**: You can pass a full path under `~/.mac-stats/task/` or a short id; the app resolves the id by listing the task directory and matching the filename (prefers `open`, then `wip`).
- Paths must be under `~/.mac-stats/task/`.

## Task file format

- **Content**: Free-form Markdown. Suggested structure:
  - First line or header: short description.
  - Sections: `## Steps`, `## Feedback` (appended by TASK_APPEND with timestamp).
- Status is **only in the filename** (`-open`, `-wip`, `-finished`), not required inside the file.

## Logging and notifying the author

- **Log file**: Task operations are logged to the app log (e.g. `~/.mac-stats/debug.log`). You will see lines such as: `Task: read ...`, `Task: appended ...`, `Task: status set to ...`, `Task: created ...`, and `Task loop: working on task '...'`, `Task loop: iteration ...`, `Task loop: task '...' finished` (or `max iterations reached`).
- **Discord**: When a task is run from the **scheduler** and the schedule entry has **`reply_to_channel_id`** set, the app sends a message to that channel when work starts: *"Working on task '&lt;filename&gt;' now."* The same channel receives the final result when the task loop completes (or fails).

## Task review loop (every 10 minutes)

- **Entry point**: Started at app startup (same as the scheduler). Runs in a background thread.
- **Behaviour** every 10 minutes:
  1. **List** all tasks with status `open` or `wip` (and their last modification time).
  2. **Close stale WIP**: Any task in `wip` whose file was last modified more than **30 minutes** ago is closed as **unsuccessful** (file renamed to `...-unsuccessful.md` and a note appended: "Closed as unsuccessful (30 min timeout).").
  3. **Start working on one open task**: If there is at least one task with status `open`, pick one and run **`run_task_until_finished(path, 10)`** (same as the scheduler TASK: flow). So you can rely on the review to make progress on open tasks without adding a schedule entry.
- **Summary**: You can review which tasks are open (to start working) and which are WIP (to see if we're still working). If a WIP is not finished within 30 minutes, it is closed as unsuccessful automatically.

## Task loop (run until finished)

- **Entry point**: Scheduler (e.g. `TASK: 1` in `schedules.json`) or the **task review loop** (picks one open task every 10 min).
- **Behaviour**: Resolves the path/id to a task file, then (if from scheduler and `reply_to_channel_id` is set) notifies that channel, then calls **`run_task_until_finished(path, 10)`**:
  1. If the task status is already `finished` or `unsuccessful`, return immediately.
  2. Loop (up to 10 iterations): read the task file, send its content to Ollama with instructions to use TASK_APPEND and TASK_STATUS, run the normal tool loop (FETCH_URL, RUN_CMD, TASK_APPEND, TASK_STATUS, etc.), then re-read the task path (file may have been renamed). If status is `finished`, return the last reply.
  3. If max iterations is reached, return the last reply with a note.
- **Reply to Discord**: If the schedule entry has `reply_to_channel_id`, the task loop result is sent to that channel (same as other scheduler tasks).

## Example

1. Create a task (by hand or via Ollama in a chat that uses the tool loop):
   - File: `~/.mac-stats/task/task-docs-1-20250208-143000-open.md`
   - Content: `# Update README\n\n- [ ] Add installation section\n`
2. Add to `schedules.json`: `"task": "TASK: 1"` or `"task": "TASK_RUN: ~/.mac-stats/task/task-docs-1-20250208-143000-open.md"` (with cron or `at`).
3. When the schedule runs, the app will read the task, call Ollama, Ollama can reply with e.g. `TASK_APPEND: 1 Added installation section.` and `TASK_STATUS: 1 finished`, and the loop stops.

## Implementation

- **Config**: `Config::task_dir()`, `Config::ensure_task_directory()` in `config/mod.rs`.
- **Task helpers**: `task/mod.rs` — `task_path`, `status_from_path`, `set_task_status`, `create_task`, `append_to_task`, `read_task`, `resolve_task_path`, `find_current_path`, `list_open_and_wip_tasks`, `format_list_open_and_wip_tasks`.
- **Task review**: `task/review.rs` — `spawn_review_thread()`; every 10 min closes WIP > 30 min as unsuccessful and runs `run_task_until_finished` on one open task.
- **Ollama**: `commands/ollama.rs` — TASK_APPEND, TASK_STATUS, TASK_CREATE in agent descriptions and tool loop; `run_task_until_finished(task_path, max_iterations)`.
- **Scheduler**: `scheduler/mod.rs` — in `execute_task`, if task starts with `TASK:` or `TASK_RUN:`, resolve path and call `run_task_until_finished(path, 10)`.

## References

- **All agents:** `docs/100_all_agents.md`
- **RUN_CMD:** `docs/011_local_cmd_agent.md` (read task files with cat/grep)
- **Scheduler:** `docs/009_scheduler_agent.md`
