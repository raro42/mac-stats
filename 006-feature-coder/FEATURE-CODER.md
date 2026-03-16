# Feature Coder — FEAT tasks

This folder and doc define how to pick and implement **FEAT** (feature) tasks in mac-stats.

## What is a FEAT task?

- **FEAT** = a concrete, implementation-ready feature or fix from the backlog.
- Sources: **Open tasks** in `docs/*.md`, **agents-tasks/task-NNN.md** (status open/wip), or **docs/006_roadmap_ai_tasks.md**.
- Prefer small, scoped work: one feature or one doc/code fix per FEAT.

## Before you start

1. Read **AGENTS.md** and **CLAUDE.md** (project rules, structure, FFI safety, logging).
2. Run `cargo check` in `src-tauri/` and fix any build errors before changing behavior.
3. After runtime changes: restart the app and verify (see AGENTS.md restart-and-test rule).

## How to pick a FEAT task

1. Scan **Open tasks** in `docs/` (e.g. `docs/007_discord_agent.md`, `docs/033_docs_vs_code_review.md`, `docs/README.md`).
2. Check **agents-tasks/** for `task-NNN.md` with status **open** or **wip** (see `agents-tasks/README.md`).
3. Pick one item that is clearly defined and not blocked. If an “Open task” was already fixed, mark it done in the doc (e.g. remove or note “Done”) instead of re-implementing.

## After completing a FEAT

- Update the doc or task file: mark the task **done** or remove it from the Open tasks list.
- Optionally add a short CHANGELOG bullet if user-facing.

## Current FEAT backlog (candidates)

| Source | Task | Notes |
|--------|------|--------|
| docs/007_discord_agent.md | ~~user-info.json auto-update on display name change~~ | **Done** |
| docs/007_discord_agent.md | ~~Customize SCHEDULE/REMOVE_SCHEDULE behavior~~ | **Done:** maxSchedules in config.json |
| docs/007_discord_agent.md | ~~Improve docs for schedules.json and user-info.json~~ | **Done:** docs/data_files_reference.md |
| docs/007_discord_agent.md | ~~Allow users to view logs for the Discord bot~~ | **Done:** Settings → View logs button; `get_debug_log_path` / `open_debug_log` |
| docs/009_scheduler_agent.md | ~~Clarify cron/at = local time or UTC~~ | **Done:** documented in data_files_reference.md |
| docs/006_roadmap_ai_tasks.md | ~~Review and refine AI tasks roadmap~~ | **Done:** tools list, Phase 1 done, open tasks trimmed |
| docs/README.md | ~~Trim stale Open tasks in historical docs~~ | **Done:** 007 trimmed, 006 points to this backlog |
| docs/007_discord_agent.md | ~~Improve docs for test_discord_connect binary~~ | **Done:** §12 expanded (token resolution, env file format, behavior, failure) |
| docs/029_browser_automation.md | ~~Improve docs for BROWSER_* tools (connection process)~~ | **Done:** added § "Connection process (step-by-step)" in 029 |
| docs/007_discord_agent.md | ~~Improve error handling when Discord API is unavailable~~ | **Done:** user-facing message + one retry (2s) on connection/timeout/5xx in api.rs; send_message paths use same in mod.rs |
| docs/009_scheduler_agent.md | ~~Improve error handling for scheduler tool invocations~~ | **Done:** on task failure (FETCH_URL, BRAVE_SEARCH, Ollama, TASK run), scheduler sends failure message to schedule’s Discord channel when reply_to_channel_id is set (scheduler/mod.rs). |
| docs/009_scheduler_agent.md | ~~Review deduplication behavior for identical cron+task pairs~~ | **Done:** cron and one-shot (at+task) both deduplicate in add_schedule/add_schedule_at; data_files_reference updated. |
| docs/agent_workflow.md | ~~Improve docs for tool agents and their invocations~~ | **Done:** "How invocations work" section, full tool table (invocation + purpose + implementation), See also (README, 007, 100_all_agents). |

Start with the first FEAT you can complete end-to-end (code or doc), then move to the next.
