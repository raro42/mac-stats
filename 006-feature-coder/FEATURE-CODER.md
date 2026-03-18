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

## Design: More robust patching system (Coder agent)

The Coder currently edits this repo **in place** (yolo mode) via the mac-stats-agent-workspace; it does not produce patches or ask for permission. A “more robust patching system” could mean one or more of the following (for future consideration; no implementation required for this FEAT):

- **Dry-run / diff preview**: Coder emits a proposed diff (or list of file edits) before applying; a human or another agent could approve or reject.
- **Atomic apply**: Group related edits into a single logical change; on failure, roll back or leave a clear “revert commit” so the repo stays consistent.
- **Patch files instead of direct edits**: Coder writes `.patch` files under a designated directory; a separate step (human or script) applies them. Allows review and selective apply.
- **Audit trail**: Log what the Coder changed (files, hunks) to a file or channel so changes are traceable without scanning full git history.

**Current choice**: Keep in-place edits for simplicity and speed; the workspace includes both mac-stats and mac-stats-reviewer so the Coder has full context. If misuse or mistakes become a problem, introduce dry-run or patch-file workflow later.

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
| docs/007_discord_agent.md | ~~Customize test_discord_connect behavior (duration)~~ | **Done:** env `TEST_DISCORD_CONNECT_SECS` (1–300), CLI second arg or single numeric arg; §12 updated. |
| docs/007_discord_agent.md | ~~More robust caching for user-info.json~~ | **Done:** in-memory cache + mtime invalidation in `user_info/mod.rs`; external edits trigger reload. |
| docs/033_docs_vs_code_review.md | ~~Verify prefer_headless behavior in edge cases~~ | **Done:** § "prefer_headless — Edge cases and verification" (scenarios table, session reuse, retries, ensure_chrome_on_port, verification checklist). |
| docs/022_feature_review_plan.md | ~~Review 023 and merge missing details into this plan~~ | **Done:** §8 "Externalized prompts (F11) — summary from 023" added (files, {{AGENTS}}, defaults, assembly order, Tauri commands, editing notes); 023 remains full reference. |
| docs/008_brave_agent.md + agent_workflow.md | ~~Brave Search API: compliance review, docs, error handling~~ | **Done:** API compliance § in 008; error-handling/edge-cases § (empty query, 429, timeout, no results); empty-query guard in `brave_web_search`; agent_workflow open task marked done. |
| docs/002_task_optimize_frontend.md | ~~Verify fetch_page_content blocks main thread in frontend path~~ | **Done:** documented in 002 § "fetch_page_content and main-thread blocking"; frontend uses `fetch_page` (spawn_blocking), so main thread not blocked. |
| docs/002_task_optimize_frontend.md | ~~Improve theme switching animation (no extra CPU)~~ | **Done:** 200ms fade-out before theme navigation in `cpu-ui.js` (ensureThemeSwitchStyle + transitionend/250ms fallback). |
| docs/002_task_optimize_frontend.md | ~~Further optimize process list DOM updates~~ | **Done:** replaceChildren(), event delegation for row clicks, skip update when list unchanged (lastProcessListKey) in `dist/cpu.js`. |
| docs/007_discord_agent.md | ~~More efficient method for testing Discord connection~~ | **Done:** `--quick` / `-q` flag in test_discord_connect (2s run); §12 updated. |
| docs/004_notes.md | ~~Plugin script timeout handling~~ | **Done:** thread + `mpsc::recv_timeout(timeout_secs)` + kill on timeout (Unix) in `plugins/mod.rs`; 004 open task and Known Issues updated. |
| docs/004_notes.md | ~~Improve plugin script error messages~~ | **Done:** plugin_id and script path in all errors; JSON parse shows stdout snippet; non-zero exit shows exit code and trimmed stderr; tracing::warn on failures. |
| docs/029_browser_automation.md | ~~Investigate why some users are unable to launch Chrome on port 9222~~ | **Done:** § "Troubleshooting: Chrome won't start or connect on 9222" (default path, port in use, spawn failures, connection timing, firewall, headless fallback, debug log). |
| docs/007_discord_agent.md | ~~More efficient data structure for schedules.json~~ | **Done:** investigation documented in data_files_reference.md (§ "Data structure and performance"); array kept; O(n) acceptable for typical N. |
| docs/007_discord_agent.md | ~~More secure method for storing Discord bot token~~ | **Done:** Keychain already used; §11 "Secure token storage (recommended)" added (Keychain via Settings for production; env/.config.env for dev/CI). Open task marked done. |
| docs/004_notes.md | ~~Add commands for registering Telegram/Slack/Mastodon channels~~ | **Done:** `register_telegram_channel`, `register_slack_channel`, `register_mastodon_channel`, `remove_alert_channel` in `commands/alerts.rs`; AlertManager `remove_channel` in `alerts/mod.rs`. |
| docs/004_notes.md | ~~Sync Known Issues with Open tasks (alert channel registration)~~ | **Done:** Known Issues §2 Alert System updated: channel registration items marked done to match Open tasks. |
| docs/README.md | ~~Trim other docs so active backlog lives here~~ | **Done:** 007, 022, 029, 002, agent_workflow, 008, 012, 035 now point to this file; README Cross-Cutting updated. |
| docs/004_notes.md | ~~Implement stream support for Ollama chat (better UX)~~ | **Done:** backend `send_ollama_chat_messages_streaming` (NDJSON), emit `ollama-chat-chunk`; frontend listens and appends to assistant message; `stream` flag on request (default true). |

### Remaining open (pick next FEAT here)

| Source | Task | Notes |
|--------|------|--------|
| docs/004_notes.md | ~~Improve settings UI for adding monitors and configuring alerts~~ | **Done:** Settings modal with Monitors (list + add form + remove) and Alert channels (list + add Telegram/Slack/Mastodon + remove); + button opens Settings → Monitors; list_monitors_with_details, list_alert_channels, get_monitor_details name from config. |
| docs/009_scheduler_agent.md | ~~Add scheduler UI for creating and editing schedules~~ | **Done:** Settings → Schedules tab: list, add (cron or one-shot), remove; Tauri commands `list_schedules`, `add_schedule`, `add_schedule_at`, `remove_schedule`; scheduler `list_schedules_for_ui`, `ScheduleForUi`. |
| docs/009_scheduler_agent.md | ~~Consider support for multiple API keys in scheduler-driven flows~~ | **Done:** § "Multiple API keys / endpoints (design)" in 009 (current behaviour, what it could mean, options: env-only, per-schedule overrides, named profiles); no code change. |
| docs/035_memory_and_topic_handling.md | ~~Multi-language reset phrases~~ | **Done:** default `session_reset_phrases.md` has EN, DE, ES, FR, IT, PT, NL; fallback in `session_memory.rs`; documented in `data_files_reference.md` § session_reset_phrases.md. |
| docs/012_ollama_context_skills.md | ~~Improve Ollama error handling in skill/context pipeline~~ | **Done:** when user requests a missing skill (e.g. `skill: 99`), Discord replies with "Skill \"X\" not found. Available: 1-summarize, 2-code." and returns early; parser returns `requested_skill_selector` so handler can detect not-found. |
| docs/012_ollama_context_skills.md | ~~Improve FETCH_URL content reduction performance~~ | **Done:** fast path (byte heuristic), truncate-only when slightly over (no summarization), truncate_at_boundary for readability. |
| docs/agent_workflow.md | ~~More robust patching system for Coder agent~~ | **Done:** design documented in this file (§ "Design: More robust patching system"). |
| docs/035_memory_and_topic_handling.md | ~~Memory pruning docs~~ | **Done:** § "Memory pruning and compaction" in 035 (caps, on-request/periodic compaction, having_fun, performance, refs). |
| docs/035_memory_and_topic_handling.md | ~~Per-channel memory in non-Discord contexts~~ | **Done:** `memory-main.md` for in-app (main) session; loaded and searched when no Discord channel (config, ollama.rs, data_files_reference, 035). |
| docs/035_memory_and_topic_handling.md | ~~New-topic detection; compaction performance~~ | **Done:** new-topic (NEW_TOPIC/SAME_TOPIC) and compaction (on-request, periodic, having_fun) already implemented and documented in 035. |
| docs/022_feature_review_plan.md | ~~D2: TASK_CREATE duplicate — suggest new id in error message~~ | **Done:** error in task/mod.rs now says "or use a different id to create a new task"; D2 resolved (option c). |
| docs/004_notes.md | ~~Alert evaluation needs to be called periodically (background task)~~ | **Done:** background thread in lib.rs every 60s; `run_periodic_alert_evaluation()` in commands/alerts.rs; `get_monitor_statuses_snapshot()` in commands/monitors.rs; 004 Known Issues updated. |
| docs/021_router_and_agents.md | ~~Improve the documentation for specialist agents~~ | **Done:** new § "Specialist agents" (definition, invocation, what they receive, where they live, default table, limitation); 021 open task marked done. |
| docs/021_router_and_agents.md | ~~Investigate why some agents are not being properly initialized~~ | **Done:** § "Agent initialization and model resolution" in 021 (load from disk each time; ModelCatalog from startup; race when catalog not set; failure modes). Log when catalog missing in `agents/mod.rs`. |
| docs/021_router_and_agents.md | ~~Consider adding support for more advanced tool commands~~ | **Done:** § "More advanced tool commands (future)" in 021 (options: structured args, result streaming, compound/batch hints, tool schema; scope for future implementation; no code change). |
| agents-tasks/task-008 | Phase 1: Request-local execution state | **Done:** `RequestRunContext` in ollama.rs; `request_id_override` + `retry_count` on `answer_with_ollama_and_fetch`; same request_id on verification retry; logs show retry_count and "request-local criteria only"; NEW_TOPIC log clarifies retries use request-local criteria. |
| agents-tasks/task-008 | Phase 2: Separate conversation memory from execution artifacts | **Done:** `session_memory.rs` — `is_internal_artifact()` filters verifier/criteria/tool-dump/escalation patterns; `add_message` skips internal; `get_messages`, `parse_session_file`, `replace_session` exclude internal when loading; test `internal_artifacts_not_persisted`. |
| agents-tasks/task-008 | Phase 3: Search result shaping for Perplexity and Brave | **Done:** `search_result_shaping.rs` — shared `ShapableSearchResult`, `shape_search_results()` (snippet truncation, domain dedup, cap), `format_search_results_blob()` with head+tail truncation; Brave uses it in `brave_web_search()` (title, url, snippet, date from `age`; 280 chars/snippet, 10 results, 2/domain, 12k blob cap). Perplexity keeps existing news-specific shaping. |
| docs/034_tool_prompt_scaling.md | Redmine create-context only when create/update | **Done:** `build_agent_descriptions` — `wants_create_or_update` aligned with pre-route (add "with the next steps", "put "); when `question` is None, no create-context (`unwrap_or(false)`). |
| agents-tasks/task-008 | Phase 4: News-aware answering and verification | **Done:** `is_news_query` expanded (today, this week); news success criteria override (2+ sources, dates, concise OK); execution `news_format_reminder`; verification `verification_news_format_note` (accept concise/bullet, require sources/dates); retry path already had narrow news hint. |
| agents-tasks/task-008 | Phase 5: Compaction hardening | **Done:** Skip compaction when `count_conversational_messages` < 2; compactor prompt preserves first system/task and most recent assistant/tool outcome; clear logs for skipped vs failed; periodic job does not retry on skip. |

Start with the first FEAT you can complete end-to-end (code or doc), then move to the next. When all rows above are done, check **docs/006_roadmap_ai_tasks.md** (Mail, WhatsApp, Google Docs) or other docs’ "Open tasks" for new candidates.
