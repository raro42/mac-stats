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
| docs/006_roadmap_ai_tasks.md | Review and refine AI tasks roadmap | Doc only |
| docs/README.md | ~~Trim stale Open tasks in historical docs~~ | **Done:** 007 trimmed, 006 points to this backlog |
| agents-tasks/task-008-* | Overnight plan items | Phased; pick one sub-item |

Start with the first FEAT you can complete end-to-end (code or doc), then move to the next.
