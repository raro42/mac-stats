# Task Agent (task files and loop until finished)

The task agent lets you work on tasks stored as Markdown files under `~/.mac-stats/task/`. All task logic lives in the **task** module (`task/mod.rs`, `task/runner.rs`, `task/review.rs`, `task/cli.rs`). Ollama and the scheduler call into the task module; they do not contain task file I/O or business rules.

## Overview

- **Task directory**: `~/.mac-stats/task/`
- **File naming**: `task-<topic>-<id>-<date-time>-<status>.md`
  - `topic`: short slug (e.g. `refactor-api`)
  - `id`: short id (e.g. `1`, `2`, or a label)
  - `date-time`: `%Y%m%d-%H%M%S` (e.g. `20250208-143000`)
  - **Status** in filename: `open`, `wip`, `finished`, `unsuccessful`, or `paused`
- **In-file metadata** (optional): `## Assigned: agent_id`, `## Paused until: ISO datetime`, `## Depends: id1, id2`, `## Sub-tasks: id1, id2`
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

Example: `mac_stats add foo 1 "Hello"` then `mac_stats list`, `mac_stats show 1`, `mac_stats status 1 wip`, `mac_stats assign 1 scheduler`, `mac_stats remove 1`.

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

- **Path or task id**: Full path under `~/.mac-stats/task/` or a short id; the app resolves the id (prefers `open`, then `wip`).
- **Assignee**: Every task has an assignee (default `default`). The **review loop** only works on tasks assigned to `scheduler` or `default`. Use TASK_ASSIGN so Discord-assigned tasks are not picked by the background loop.
- **Discord / natural language**: When a user says "close the task", "finish", "mark done", or "cancel" a task (e.g. in Discord), the agent instructions tell the LLM to reply with **TASK_STATUS: &lt;id&gt; finished** (success) or **TASK_STATUS: &lt;id&gt; unsuccessful** (failed), not `wip`. This is defined in `commands/ollama.rs` in `build_agent_descriptions` so the model reliably closes the task instead of leaving it in progress.

## Task file format

- **Content**: Free-form Markdown. Optional in-file lines:
  - `## Assigned: scheduler` — who owns the task (scheduler, discord, cpu, default).
  - `## Paused until: 2025-02-10T09:00:00` — resume after this time (review loop clears and reopens).
  - `## Depends: id1, id2` — task is only **ready** when all listed tasks are finished or unsuccessful.
  - `## Sub-tasks: id1, id2` — parent cannot be set to `finished` until all sub-tasks are finished or unsuccessful.
- **Sections**: `## Feedback` (appended by TASK_APPEND with timestamp).

## Task review loop (every 1 minute)

- **Entry point**: `task/review.rs` — `spawn_review_thread()` at app startup.
- **Interval**: Review runs every **1 minute** so tasks are looked at at least once per minute (configurable via `REVIEW_INTERVAL_SECS` in `review.rs`).
- **Behaviour** each cycle:
  1. **Close stale WIP**: Any task in `wip` last modified more than **30 minutes** ago is set to **unsuccessful** and a note appended.
  2. **Resume paused**: Any task in `paused` with `## Paused until: <datetime>` where now >= datetime is renamed to `open` and the paused-until line is removed.
  3. **Work on open tasks**: Pick open tasks that are **assigned to scheduler or default**, **ready** (all `## Depends:` satisfied), up to **3 tasks per cycle**, each with up to **20 iterations** via `task::runner::run_task_until_finished`. If max iterations is reached without `finished`, the task is auto-closed as **unsuccessful**.

**Why aren’t my tasks being worked on?** The review loop only picks tasks that are (1) status **open**, (2) **assigned to `scheduler` or `default`** (tasks assigned to `discord` or `cpu` are skipped), and (3) **ready** (no unmet `## Depends:`). If you have many open tasks but none are picked, check assignees: `mac_stats list --all` shows `(assigned: …)`; reassign with `mac_stats assign <id> scheduler` or via **TASK_ASSIGN: &lt;id&gt; scheduler**. The app logs each cycle: "Task scan: open=…" and, when no task is picked, either "none assigned to scheduler/default" or "none are ready (check ## Depends:)".

## Task loop (run until finished)

- **Entry point**: Scheduler (`TASK:` or `TASK_RUN: <path or id>` in `schedules.json`) or the task review loop.
- **Implementation**: `task/runner.rs` — `run_task_until_finished(path, max_iterations)`:
  1. If status is already `finished` or `unsuccessful`, return immediately.
  2. Loop: read task, send to Ollama with instructions to use TASK_APPEND/TASK_STATUS, run tool loop, re-read path. Stop when status is `finished`.
  3. If max iterations reached, set status to **unsuccessful** and append "Max iterations reached; closed as unsuccessful."
- **Scheduler**: Resolves path and calls `task::runner::run_task_until_finished(path, 10)`. If `reply_to_channel_id` is set, sends "Working on task '...' now." and the final result to that Discord channel.

## Module layout

- **task/mod.rs**: Core — create, read, append, set_status, resolve_path, list, format_list, assignee, paused_until, depends_on, is_ready, sub_tasks, all_sub_tasks_closed, delete_task, show_task_content.
- **task/runner.rs**: `run_task_until_finished(path, max_iterations)` — loop that calls Ollama and re-reads task until finished or max iterations (then auto-close as unsuccessful).
- **task/review.rs**: Review loop — close_stale_wips, resume_paused_tasks, pick_one_open_task (filter by assignee and is_ready), run up to 3 tasks per cycle via runner.
- **task/cli.rs**: CLI subcommands (add, list, show, status, remove, assign, append) invoked from `main` when the user runs `mac_stats add ...` etc.
- **commands/ollama.rs**: Thin — parses TASK_* tool lines and calls one task function per tool; no task file I/O.
- **scheduler/mod.rs**: For TASK:/TASK_RUN:, resolves path and calls `task::runner::run_task_until_finished(path, 10)`.

## References

- **All agents:** `docs/100_all_agents.md`
- **RUN_CMD:** `docs/011_local_cmd_agent.md` (read task files with cat/grep)
- **Scheduler:** `docs/009_scheduler_agent.md`
