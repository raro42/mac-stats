# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.21] - 2026-02-28

### Added
- **Discord having_fun per-channel model/agent**: In `discord_channels.json`, channels can set `model` (Ollama model override) and `agent` (agent id, e.g. `abliterated`) so having_fun uses that agent's soul+skill and model. When `agent` is set, the channel uses the agent's combined prompt and model; otherwise soul + optional channel `prompt` and `model` as before.
- **Discord having_fun configurable loop protection**: `having_fun.max_consecutive_bot_replies` in config (default 0). 0 = do not reply to bot messages; 1–20 = max consecutive bot messages before dropping (loop protection). Replaces hardcoded limit of 5.
- **Ollama chat timeout config**: `config.json` key `ollamaChatTimeoutSecs` (default 300, range 15–900). Env override `MAC_STATS_OLLAMA_CHAT_TIMEOUT_SECS`. Used for all Ollama /api/chat requests (UI, Discord, session compaction).
- **Model identity in prompts**: Agent router and having_fun system prompts now include "You are replying as the Ollama model: **<name>**" so the bot can answer "which model are you?" accurately. `get_default_ollama_model_name()` exposed for Discord/UI.
- **Default agent with soul**: New macro `default_agent_entry_with_soul!("id")` and default agent **abliterated** (`agent-abliterated/`: agent.json, skill.md, soul.md, testing.md) for having_fun channels that want a distinct persona.
- **docs/012_cursor_agent_tasks.md**: Cursor agent tasks documentation.

### Changed
- **having_fun**: Uses agent's soul+skill+model when channel has `agent`; otherwise unchanged (soul + channel prompt/model). Default `max_consecutive_bot_replies` 0 to avoid replying to other bots unless explicitly configured.
- **agents-tasks**: README clarifies log-NNN vs task-NNN, log path `~/.mac-stats/debug.log`; review docs and .gitignore use `agents-tasks` (fixed typo).

### Removed
- **OPTIMIZATION_PROGRESS.md** and **docs/OPTIMIZE_CHECKLIST.md**: Obsolete optimization checklists removed.

## [0.1.20] - 2026-02-27

### Added
- **Loop-protection visibility (log-007)**: Per-channel `loop_protection_drops` counter in having_fun state; incremented when a bot message is dropped; every 60s heartbeat logs `DEBUG Discord: loop protection: channel <id> dropped N message(s) this period` and resets counter. Use `-vv` to see summaries.

### Changed
- **Agent-tasks**: All log-001 through log-009 verified implemented; README and task files updated to status **done**. Log-002 (log rotation), log-003 (temperature N/A), log-004 (image 404), log-005 (Discord scope sanitize), log-006 (Ollama dedupe), log-007 (loop-protection visibility), log-008 (FETCH_URL redmine hint), log-009 (Redmine 422) confirmed in code.
- **Release**: Version 0.1.20; release build and app restart with `-vv` for verification.

## [0.1.19] - 2026-02-23

### Added
- **Redmine search API**: Keyword search for issues via `GET /search.json?q=<keyword>&issues=1&limit=100`. Documented in REDMINE_API tool description and in `docs/025_redmine_api_skill.md`. Use for "search/list tickets about X"; the issues list endpoint has no full-text search param.
- **Redmine create context**: When Redmine is configured, the app fetches projects, trackers, issue statuses, and priorities from the API, caches them for 5 minutes, and injects the list into the agent description so the model can resolve "Create in AMVARA" (or similar) to the correct `project_id`. See `docs/025_redmine_api_skill.md`.
- **Default agent macro**: New `default_agent_entry!("id")` macro in config; default agents are built from `DEFAULT_AGENT_IDS` so adding agent-004/005 (or more) is a single line. `Config::tmp_dir()` and `Config::tmp_js_dir()` for runtime scratch paths.
- **AGENTS.md restart-and-test rule**: After changes that affect runtime behavior (Redmine, tasks, agent prompts, scheduler, Discord, Ollama tools), restart mac-stats and test; do not assume it works without verification.
- **Merge-defaults doc**: `docs/024_mac_stats_merge_defaults.md` and agents.md section on updating `~/.mac-stats` from defaults (merge, do not overwrite).

### Changed
- **RUN_CMD logging**: Logs the exact command string (e.g. `RUN_CMD: executing: cat ~/.mac-stats/schedules.json`) and at entry the full argument line for debugging.
- **cargo default binary**: `default-run = "mac_stats"` in Cargo.toml so `cargo run -- -vv` works without `--bin mac_stats`.
- **Discord having_fun**: Casual-context constant for having_fun channels; channel config log moved to after having_fun state init; log line includes next response and next idle thought timing when having_fun channels exist.
- **Orchestrator skill**: REDMINE_API bullet now includes search endpoint and create-context note; task-create-and-assign flow documented for delegated coding tasks; RUN_CMD allowlist in skill.

## [0.1.18] - 2026-02-22

### Added
- **Task file naming**: New convention `task-<date-time>-<status>.md` (e.g. `task-20260222-140215-open.md`). Topic and id are stored in-file as `## Topic:` and `## Id:` for listing and resolution.
- **Task conversation logging**: When the agent touches a task (TASK_CREATE, TASK_APPEND, TASK_STATUS, etc.), the full user question and assistant reply are appended to the task file as a `## Conversation` block. Runner turns (synthetic "Current task file content..." prompts) are skipped.
- **Having_fun ASAP**: In having_fun channels, messages that are a mention or from a human trigger an immediate response (next tick) instead of the random delay.
- **Having_fun idle timer log**: The periodic "Having fun: idle timer" log now includes time until next response and next idle thought (e.g. `next response in 45s, next idle thought in 120s`). Logged about once a minute when there are having_fun channels.
- **Perplexity Search**: Optional web search via Perplexity API. Tauri commands `perplexity_search` and `is_perplexity_configured`; API key stored in Keychain (Settings). Use from Ollama/agents for real-time web search.

### Changed
- **Task resolution**: Resolve by full task filename (with or without `.md`), by short id (from `## Id:` in file), or by topic (from `## Topic:` in file). Legacy filenames (task-topic-id-datetime-status) still supported.
- **TASK_CREATE**: Rejects when topic looks like an existing task filename; sanitizes id (strips quotes/slashes). Deduplication checks `## Topic:` and `## Id:` in existing files.
- **TASK_APPEND / TASK_CREATE parsing**: Multi-line content is preserved (all lines until the next tool line), so research and long text are stored completely in the task file.
- **Having_fun flow**: Before replying, the app fetches the latest messages from Discord (after the bot's last response) and uses those as context for better flow. Falls back to the in-memory buffer if the API fetch fails.
- **Docs and memory**: All MD files and `~/.mac-stats/agents/memory.md` updated to document the new task naming (`task-<date>-<status>.md`, topic/id in-file). See `docs/013_task_agent.md`, `docs/021_task_system_fix_feb2026.md`, `docs/022_feature_review_plan.md`.

## [0.1.17] - 2026-02-22

### Added
- **Periodic session compaction**: Every 30 minutes a background thread compacts all in-memory sessions (≥ 4 messages) into long-term memory. Lessons are appended to global `memory.md`. Active sessions (activity within 30 min) are replaced with the summary; inactive sessions are cleared after compacting.
- **Session memory `last_activity` and `list_sessions()`**: Sessions now track last activity time. `list_sessions()` returns all in-memory sessions for the periodic compaction loop.
- **Having_fun configurable delays**: `discord_channels.json` supports a top-level `having_fun` block with `response_delay_secs_min/max` and `idle_thought_secs_min/max` (seconds). Each response or idle thought uses a random value in that range (e.g. 5–60 min). Default 300–3600. Config is reloaded when the file changes.
- **Having_fun time-of-day awareness**: The having_fun system prompt now includes current local time and period-aware guidance (night / morning / afternoon / evening) so the bot can behave differently by time of day (e.g. shorter and calmer at night, more energetic in the morning).
- **Discord channels config upgrade**: If `~/.mac-stats/discord_channels.json` exists but has no `having_fun` block, the app adds the default block on load and writes the file back.
- **Chatbot avatar assets**: SVG (and optional PNG) avatar icon for mac-stats chatbot in `src/assets/`.
- **Discord send CLI**: Subcommand `mac_stats discord send <channel_id> <message>` to post a message to a Discord channel (uses bot token from config). Useful for scripts and wrap-up notifications.

### Changed
- **Session compaction**: On-request compaction unchanged (≥ 8 messages); periodic compaction uses a lower threshold (4 messages) so more conversations are flushed to long-term memory.
- **docs/session_compaction_and_memory_plan.md**: Updated to document implemented behavior (30-min loop, last_activity, time-of-day).

## [0.1.16] - 2026-02-21

### Added
- **Discord channel modes** (`~/.mac-stats/discord_channels.json`): Per-channel listen configuration with three modes:
  - `mention_only` (default) — respond only to @mentions and DMs
  - `all_messages` — respond to every human message, no @mention required
  - `having_fun` — respond to everyone including other bots, with 30s buffered responses, idle thoughts after 5min silence, and loop protection (max 5 consecutive bot exchanges)
- **Per-channel prompt injection**: Channels support an optional `prompt` field that shapes response style (e.g. "be casual, no bullet points, never offer help"). Injected into the system context for that channel only.
- **Discord typing indicator**: Werner_Amvara now shows "is typing..." while processing a message. Fires immediately and refreshes every 8s until the reply is ready.
- **Verbose mode for Discord**: Status/thinking messages (e.g. "Asking Ollama for a plan...") are suppressed by default to keep channels clean. Add `verbose` as a header line to see them.
- **Bot mention stripping**: The `<@BOT_ID>` tag is now removed from message content before processing, so Ollama receives a clean question.
- **Session compaction**: When conversation history exceeds 8 messages, it is automatically compacted using a fast model (small role). Extracts verified facts and successful outcomes, drops failed attempts and hallucinations. Lessons learned are appended to global `memory.md`.
- **Session memory `replace_session()`**: Persists old session to disk and replaces in-memory history with compacted summary.
- **Discord Expert agent** (agent-004): Specialized agent for Discord API operations with its own tool loop and memory.
- **Persistent memory system**: Global (`memory.md`) and per-agent memory files loaded into every agent's prompt. `MEMORY_APPEND` tool for agents to write lessons learned.
- **Default `discord_channels.json`**: Shipped with the app via `ensure_defaults()`, with documentation and examples for all three modes.

### Changed
- **Discord bot ignores other bots** in `mention_only` and `all_messages` channels (prevents accidental bot-to-bot loops).
- **`having_fun` uses direct Ollama chat**: Bypasses the full planning/tools pipeline for faster, more conversational responses. Soul + channel prompt + history only.
- **FETCH_URL Discord intercept widened**: All `discord.com` URLs (not just `/api/`) are now intercepted and redirected to `DISCORD_API` or rejected with guidance to use the discord-expert agent.
- **Orchestrator skill.md**: Updated with Discord Expert delegation rules and DISCORD_API critical rules.

### Dependencies
- Added `tokio-util` (CancellationToken for typing indicator lifecycle).

## [0.1.15] - 2026-02-21

### Added
- **Dynamic model resolution for agents**: Agents now declare a `model_role` ("general", "code", "small") instead of hardcoding a model name. At startup, the app queries Ollama `/api/tags`, classifies all installed models by capability (Code vs General) and size tier (Small <4B, Medium 4-15B, Large >15B), and resolves each agent to the best available model. Models above 15B are excluded from auto-selection. Resolution is logged at startup for full visibility. The `model` field remains as an optional explicit override.
  - New module: `ollama/models.rs` with `ModelCatalog`, classification logic, and 7 unit tests
  - New field: `model_role` in `AgentConfig` / `Agent` structs and all CRUD commands
  - Default agent configs updated: orchestrator=general, coder=code, generalist=small
- **Redmine API agent**: Ollama can access Redmine issues, projects, and time entries via `REDMINE_API: GET /issues/1234.json`. Pre-routes ticket/issue questions directly to Redmine when configured. Configure via `REDMINE_URL` and `REDMINE_API_KEY` in env or `~/.mac-stats/.config.env`.
- **Discord "new session" command**: Type `new session: <question>` in Discord to clear conversation history and start fresh. Prior messages are persisted to disk before clearing.
- **Session memory `clear_session()`**: New function to flush and clear in-memory conversation history for a source/channel.
- **RUN_CMD dynamic allowlist**: The command allowlist is now read from the orchestrator agent's `skill.md` (section `## RUN_CMD allowlist`). Falls back to the default list if not configured. Added `cursor-agent` to default allowlist.
- **RUN_CMD pipe support**: Commands now support `cmd1 | cmd2 | cmd3` pipelines; each stage is validated against the allowlist independently.

### Changed
- **Agent default configs**: Shipped agent.json files use `model_role` instead of hardcoded `model` names. Existing user configs with explicit `model` continue to work (explicit model takes priority when available, falls back to `model_role` if the model is removed).

## [0.1.14] - 2026-02-19

### Added
- **Externalized prompts**: System prompts (`planning_prompt.md`, `execution_prompt.md`) and soul (`soul.md`) are now editable files under `~/.mac-stats/prompts/` and `~/.mac-stats/agents/`. Previously hardcoded as Rust constants. The execution prompt supports a `{{AGENTS}}` placeholder that is replaced at runtime with the dynamically generated tool list.
- **Default agents shipped**: Four default agents (orchestrator, general assistant, coder, generalist) are embedded in the binary via `include_str!` from `src-tauri/defaults/`. On first launch, `ensure_defaults()` writes all missing files (`agent.json`, `skill.md`, `testing.md` per agent, plus `soul.md` and prompts). Existing user files are never overwritten.
- **Tauri commands for prompt editing**: `list_prompt_files` returns name, path, and content of all prompt files; `save_prompt_file(name, content)` writes to a named prompt file. Available for frontend integration.
- **RUN_CMD retry loop**: When a local command fails (non-zero exit), the app sends the error to Ollama in a focused prompt asking for a corrected command. Retries up to 3 times. Handles cases where the model appends plan commentary to the command arg (e.g. `cat file.json then do X`).
- **Empty response fallback**: When Ollama returns an empty response after a successful tool execution, the raw tool output is returned directly to the user instead of showing nothing. Covers RUN_CMD, FETCH_URL, DISCORD_API, MCP, and search results.

### Fixed
- **Tool parsing: numbered list plans**: `parse_tool_from_response` now strips leading list numbering (`1. `, `2) `, `- `, `* `) and multiple nested `RECOMMEND:` prefixes. Previously, plans like `1. RUN_CMD: cat file.json 2. Extract...` were not recognized as tool calls.
- **Tool arg truncation**: When Ollama concatenates multiple plan steps on one line, the arg is now truncated at the next numbered step boundary (e.g. ` 2. `) so commands receive clean arguments.
- **RECOMMEND prefix stripping**: The recommendation from the planning step now has all `RECOMMEND:` prefixes stripped before being used in the execution system prompt and before tool parsing. Previously, the raw `RECOMMEND: RUN_CMD: ...` was passed to Ollama's execution step as `Your plan: RECOMMEND: RUN_CMD: ...`, which confused the model into returning empty responses.
- **Stale binary**: Ensured all code changes (fast-path tool execution, RECOMMEND stripping) are compiled into the running binary. Previous session's changes were only in source but not rebuilt.

### Changed
- **Prompts loaded from files**: `EXECUTION_PROMPT` and `PLANNING_PROMPT` are no longer Rust `const` strings. They are read from `~/.mac-stats/prompts/` at each request, so edits take effect immediately without rebuild.
- **`DEFAULT_SOUL` uses `include_str!`**: The default soul content is now a real Markdown file at `src-tauri/defaults/agents/soul.md`, embedded at compile time. Easier to read and diff than an inline Rust string literal.
- **`src-tauri/defaults/` directory**: All default content (soul, prompts, agents) lives as real `.md`/`.json` files in the repo, embedded via `include_str!`. Clean Markdown diffs in PRs.

## [0.1.13] - 2026-02-19

### Added
- **Task module and CLI**: All task logic centralized in `task/` (mod, runner, review, cli). Ollama and scheduler only call into the task module.
  - **CLI**: `mac_stats add|list|show|status|remove|assign|append` for testing and scripting (e.g. `mac_stats add foo 1 "Content"`, `mac_stats list --all`, `mac_stats assign 1 scheduler`).
  - **TASK_SHOW**: Show one task's status, assignee, and content to the user in the message channel (Discord/UI).
  - **Assignee**: Every task has `## Assigned: agent_id` (default `default`). **TASK_ASSIGN** reassigns to scheduler|discord|cpu|default. Review loop only picks tasks assigned to **scheduler** or **default**.
  - **TASK_STATUS** allows **unsuccessful** and **paused**. **TASK_SLEEP: &lt;id&gt; until &lt;ISO datetime&gt;** pauses until that time; review loop auto-resumes when time has passed.
  - **Dependencies**: `## Depends: id1, id2` in task file; review loop only picks tasks whose dependencies are finished or unsuccessful (**is_ready**).
  - **Sub-tasks**: `## Sub-tasks: id1, id2`; parent cannot be set to **finished** until all sub-tasks are finished or unsuccessful.
  - **Review loop**: Up to 3 open tasks per cycle, 20 iterations per task; auto-close as unsuccessful on max iterations; resume due paused tasks each cycle.
  - **task/runner.rs**: `run_task_until_finished` moved from ollama to task module; scheduler and review call `task::runner::run_task_until_finished`.
- **delete_task**: Remove all status files for a task (CLI `remove`, used by CLI only).
- **Discord session memory**: Discord bot now maintains short-term conversation context (last 20 messages per channel). The model can resolve references like "there", "it", etc. from prior turns. After app restart, context is resumed from the latest session file on disk.
- **Conversation history in agent router**: `answer_with_ollama_and_fetch` accepts optional `conversation_history` so Discord (and future entry points) can pass prior turns into planning and execution prompts.
- **Chat reserved words**: Type `--cpu` in chat to toggle the CPU window, or `-v`/`-vv`/`-vvv` to change log verbosity at runtime without restarting. New Tauri commands: `toggle_cpu_window`, `set_chat_verbosity`.
- **Background monitor checks**: Website monitors are now checked in a background thread every 30 seconds (by their configured interval), even when the CPU window is closed.
- **TASK_CREATE deduplication**: Creating a task with the same topic and id as an existing task now returns an error instead of silently creating duplicates.

### Fixed
- **Ollama model auto-detection at startup**: The app no longer hardcodes `llama2` as the default model. At startup, it queries `GET /api/tags` and picks the first available model. Frontend `getDefaultModel()` also queries installed models via `list_ollama_models`. Fallback is `llama3.2`.
- **Native tool-call parsing errors**: Models with built-in tool support (qwen3, command-r, etc.) caused Ollama to fail with "error parsing tool call" because Ollama tried to parse text tool invocations as JSON. Fixed by sending `"tools": []` in all chat requests, which disables Ollama's native tool-call parser.
- **Direct tool execution from plan**: When the planning step returns a recommendation that already contains a parseable tool call (e.g. `DISCORD_API: GET /users/@me/guilds`), the router now executes it directly instead of asking Ollama a second time. Saves one full Ollama round-trip per request.
- **Logging `ellipse()` helper**: Truncated text now shows first half + `...` + last half instead of hard truncation. Applied to Ollama request/response logs, FETCH_URL content, and Discord API responses.
- **Verbosity-aware logging**: At `-vv` or higher, Ollama request/response logs are never truncated.
- **Char-count vs byte-count**: Fixed Discord API response truncation to use `.chars().count()` instead of `.len()` for correct Unicode handling.

### Changed
- **Unified soul path**: Consolidated `~/.mac-stats/agent/soul.md` (router) and `~/.mac-stats/agents/soul.md` (agent fallback) into a single path: `~/.mac-stats/agents/soul.md`. Used by all agents (as fallback) and by the router (non-agent chat). The old `~/.mac-stats/agent/` directory is no longer used. **Migration**: move any customized `~/.mac-stats/agent/soul.md` to `~/.mac-stats/agents/soul.md`.
- **Task prompt guidance**: Agent descriptions now instruct the model to invoke `AGENT: orchestrator` (not just `TASK_CREATE`) when users want agents to chat, and to use `TASK_APPEND`/`TASK_STATUS` when a task with the same topic+id already exists.
- **Toggle CPU window refactored**: Extracted inline window toggle logic from the click handler into `toggle_cpu_window()` function, reusable from both menu bar clicks and the new `--cpu` chat command.
- **Task docs**: `docs/013_task_agent.md` rewritten — CLI, TASK_SHOW, assignee, TASK_ASSIGN, pause/sleep, dependencies, sub-tasks, module layout, review behaviour.

## [0.1.11] - 2026-02-09

### Added
- **SKILL agent documentation**: `docs/016_skill_agent.md` — SKILL tool invocation, logging, and future modify path. Agent descriptions sent to Ollama include enriched SKILL info for better recommendation; skills load is logged (info: available list; warn: read errors). See `docs/100_all_agents.md` (tool table, subsection 2.9).
- **SCHEDULE tool (one-shot and cron)**: The agent can add one-shot reminders and recurring tasks via SCHEDULE in three formats:
  - **Every N minutes**: `SCHEDULE: every N minutes <task>` (unchanged).
  - **Cron**: `SCHEDULE: <cron expression> <task>` — 6-field (sec min hour day month dow) or 5-field (min hour day month dow; app prepends `0` for seconds). Cron examples are injected into the agent context (e.g. every day at 5am, weekdays at 9am). See `docs/007_discord_agent.md`.
  - **One-shot (at datetime)**: `SCHEDULE: at <datetime> <task>` — run once at local time; datetime ISO `YYYY-MM-DDTHH:MM:SS` or `YYYY-MM-DD HH:MM`. Scheduler supports `add_schedule_at()` for one-shot entries in `~/.mac-stats/schedules.json`.

### Changed
- **SCHEDULE status**: Status line now shows a short preview of the schedule text while adding (e.g. "Scheduling: every 5 minutes…").

## [0.1.10] - 2026-02-09

### Added
- **Full Ollama API coverage**: List models with details, get version, list running models, pull/update/delete models, generate embeddings, load/unload models from memory.
  - Tauri commands: `list_ollama_models_full`, `get_ollama_version`, `list_ollama_running_models`, `pull_ollama_model`, `delete_ollama_model`, `ollama_embeddings`, `unload_ollama_model`, `load_ollama_model`. All use the configured Ollama endpoint (same as chat/Discord/scheduler).
  - Backend: `ollama/mod.rs` types and `OllamaClient` methods for GET /api/tags (full), GET /api/version, GET /api/ps, POST /api/pull, DELETE /api/delete, POST /api/embed, and load/unload via keep_alive on generate/chat.
  - Documentation: `docs/015_ollama_api.md`.
- **User info (user-info.json)**: Per-user details from `~/.mac-stats/user-info.json` (keyed by Discord user id) are merged into the agent context (display_name, notes, timezone, extra). See `docs/007_discord_agent.md`.
- **Task review loop**: Background loop every 10 minutes: lists open/wip tasks, closes WIP tasks older than 30 minutes as **unsuccessful** (appends note), then runs `run_task_until_finished` on one open task. Started at app startup. See `docs/013_task_agent.md`.
- **TASK_LIST tool**: Ollama can invoke `TASK_LIST` or `TASK_LIST:` to get the list of open and WIP task filenames under `~/.mac-stats/task/` (naming: `task-<date-time>-<status>.md`; topic and id are stored in-file).
- **Task status "unsuccessful"**: Task filenames can use status `unsuccessful`; review loop uses it for stale WIP timeouts.

### Changed
- **Agent status messages**: When the agent uses a skill or the Ollama API, the status line now shows details: "Using skill &lt;number&gt;-&lt;topic&gt;…" and "Ollama API: &lt;action&gt; [args]…".
- **README**: Features and Current Features sections updated to include all agents (Discord, MCP, Task, PYTHON_SCRIPT, Scheduler, Skills) and grouped by system monitoring, website & monitoring, AI & agents, UI.

## [0.1.9] - 2026-02-09

### Added
- **Discord API agent**: When a request comes from Discord, Ollama can call the Discord HTTP API via the DISCORD_API tool (e.g. list guilds, channels, members, get user). Endpoint list is documented in `docs/007_discord_agent.md` and injected into the agent context. Only GET and POST to `/channels/{id}/messages` are allowed.
- **Discord user names**: The bot records the message author's display name and passes it to Ollama so it can address the user by name; names are cached for reuse in the session.
- **MCP Agent (Model Context Protocol)**: Ollama can use tools from any MCP server
  - Configure via `MCP_SERVER_URL` (HTTP/SSE) or `MCP_SERVER_STDIO` (e.g. `npx|-y|@openbnb/mcp-server-airbnb`) in env or `~/.mac-stats/.config.env`
  - When configured, the app fetches the tool list and adds it to the agent descriptions; Ollama invokes tools by replying `MCP: <tool_name> <arguments>`
  - Supported in Discord bot, scheduler, and CPU window chat (same tool loop)
  - Documentation: `docs/010_mcp_agent.md`
- **Task agent**: Task files under `~/.mac-stats/task/` with TASK_APPEND, TASK_STATUS, TASK_CREATE. Scheduler supports `TASK: <path or id>` / `TASK_RUN: <path or id>` to run a task loop until status is `finished`; optional `reply_to_channel_id` sends start and result to Discord. Documentation: `docs/013_task_agent.md`.
- **PYTHON_SCRIPT agent**: Ollama can create and run Python scripts; scripts are written to `~/.mac-stats/scripts/` and executed with `python3`. Disable with `ALLOW_PYTHON_SCRIPT=0`. Documentation: `docs/014_python_agent.md`.

## [0.1.8] - 2026-02-08

### Added
- **Ollama context window and model/params**: Per-model context size via `POST /api/show`, cached; Discord can override model (`model: llama3.2`), temperature and num_ctx (`temperature: 0.7`, `num_ctx: 8192` or `params: ...`). Config supports optional default temperature/num_ctx. See `docs/012_ollama_context_skills.md`.
- **Context-aware FETCH_URL**: When fetched page content would exceed the model context, the app summarizes it via one Ollama call or truncates with a note. Uses heuristic token estimate (chars/4) and reserved space for the reply.
- **Skills**: Markdown files in `~/.mac-stats/skills/skill-<number>-<topic>.md` can be selected in Discord with `skill: 2` or `skill: code`; content is prepended to the system prompt so different “agents” respond differently.
- **Ollama agent at startup**: The app configures and checks the default Ollama endpoint at startup so the agent is available for Discord, scheduler, and CPU window without opening the CPU window first.

### Changed
- **Discord agent**: Reply uses full Ollama + tools pipeline (planning + execution). Message prefixes for model, temperature, num_ctx, and skill documented in `docs/007_discord_agent.md` and `docs/012_ollama_context_skills.md`.

## [0.1.7] - 2026-02-06

### Added
- **Discord Agent (Gateway)**: Optional Discord bot that connects via the Gateway and responds to DMs and @mentions
  - Bot token stored in macOS Keychain (account `discord_bot_token`); never logged or exposed
  - Listens for direct messages and guild messages that mention the bot; ignores own messages
  - Requires Message Content Intent enabled in Discord Developer Portal
  - Tauri commands: `configure_discord(token?)` to set/clear token, `is_discord_configured()` to check
  - Reply is currently a stub; Ollama/browser pipeline to be wired in a follow-up
  - Documentation: `docs/007_discord_agent.md`

## [0.1.6] - 2026-01-22

### Fixed
- **DMG Asset Bundling**: Fixed missing assets (Ollama icon, JavaScript/Tauri icons) in DMG builds
  - Added explicit `resources` configuration in `tauri.conf.json` to bundle `dist/assets/` files
  - Assets are now properly included in production DMG builds
- **Ollama Icon Path**: Fixed Ollama icon not displaying in DMG builds
  - Changed icon paths from relative (`../../assets/ollama.svg`) to absolute (`/assets/ollama.svg`) in all theme HTML files
  - Icons now resolve correctly in Tauri production builds
- **History Chart Initialization**: Fixed history charts not drawing in DMG builds
  - Moved canvas element lookup and context initialization to `initializeCanvases()` function
  - Added defensive initialization in `updateCharts()` to handle delayed DOM loading
  - Charts now properly initialize in production builds

### Added
- **DMG Testing Script**: Added `scripts/test-dmg.sh` for automated DMG verification before release
  - Mounts DMG and verifies app structure
  - Checks for required assets and theme files
  - Provides installation and testing instructions
  - Validates bundle correctness before distribution

### Changed
- **Test Script Path Detection**: Updated test script to check correct asset path (`dist/assets/` instead of `assets/`)

## [0.1.5] - 2026-01-22

### Changed
- **Release**: Version bump for release build

## [0.1.4] - 2026-01-22

### Added
- **Welcome Message**: Added a friendly welcome message that displays when the application starts and the menu bar is ready
  - Always visible in console (not dependent on verbosity flags)
  - Includes app version, warm greetings, and encouragement to share on GitHub and Mastodon
  - Encourages community contributions and feedback

## [0.1.3] - 2026-01-19

### Added
- **CLI Parameter Support**: Added support for passing CLI arguments through the `run` script
  - `./run --help` or `./run -h` to show help
  - `./run --openwindow` flag to optionally open CPU window at startup
  - All CLI flags (`-v`, `-vv`, `-vvv`, `--cpu`, `--frequency`, `--power-usage`, `--changelog`) now work through the `run` script
  - Development mode (`./run dev`) also passes arguments to `cargo run`

### Fixed
- **Window Opening at Startup**: Fixed issue where CPU window was automatically opening at startup
  - Removed manual `sendAction` test code that was triggering the click handler during setup
  - All windows are now properly hidden at startup (menu bar only mode)
  - Window only opens when explicitly requested via `--cpu` or `--openwindow` flags or when menu bar is clicked
- **Compilation Warnings**: Suppressed dead code warnings for utility methods
  - Added `#[allow(dead_code)]` to `total_points()`, `estimate_memory_bytes()`, `save_to_disk()`, and `load_from_disk()` methods
  - These methods are reserved for future use or used in tests
- **Power Consumption Flickering**: Fixed power consumption values flickering to 0.0W when background thread updates cache
  - Added `LAST_SUCCESSFUL_POWER` fallback cache to prevent flickering when main lock is unavailable
  - Power values now persist across lock contention scenarios
  - Improved power cache update logic to always maintain last successful reading
- **Power Display Precision**: Fixed power values < 1W showing as "0 W" causing visual flicker
  - Changed from `Math.round()` to `.toFixed(1)` to show 1 decimal place (e.g., "0.3 W" instead of "0 W")
  - Applied to both CPU and GPU power displays
  - Total power calculation now uses cached values to prevent flickering
- **Ollama Logging Safety**: Enhanced JavaScript execution logging with comprehensive sanitization
  - Added `sanitizeForLogging()` function to prevent dangerous characters from breaking logs
  - Safe logging wrapper that never throws errors, ensuring logging failures don't break execution flow
  - Truncates long strings, removes control characters, and sanitizes quotes/backticks
  - Prevents log injection and system breakage from malformed execution results

### Changed
- **Startup Behavior**: App now starts in menu bar only mode by default
  - No windows are visible at startup
  - CPU window is created on-demand when menu bar is clicked
  - Improved startup logging to indicate menu bar only mode
- **History Chart Styling**: Improved visual design of history chart container
  - Enhanced glass effect with backdrop blur and subtle shadows
  - Removed border, added inset highlights for depth
  - Better visual consistency with macOS glass aesthetic
- **Power Capability Detection**: Improved `can_read_cpu_power()` and `can_read_gpu_power()` functions
  - Now checks power cache existence as fallback when capability flags aren't set yet
  - Handles edge cases where power reading works but flags haven't been initialized
- **Development Logging**: Added verbose logging (`-vvv`) to release build script for easier debugging

### Technical
- **State Management**: Added `LAST_SUCCESSFUL_POWER` static state for power reading fallback
- **Error Handling**: Enhanced error handling in power consumption reading with graceful fallbacks
- **Logging Infrastructure**: Improved Ollama JavaScript execution logging with sanitization and error isolation

## [0.1.2] - 2026-01-19

### Added
- **Universal Collapsible Sections**: Replicated Apple theme's USAGE card click behavior across all themes
  - Clicking the USAGE card toggles both Details and Processes sections
  - Clicking section headers individually hides respective sections
  - Sections are hidden by default (collapsed state)
  - State persists in localStorage across sessions
  - Added universal IDs (`cpu-usage-card`, `details-section`, `processes-section`, `details-header`, `processes-header`) to all themes
  - Added clickable cursor and hover effects for better UX

### Fixed
- **Ollama Icon Visibility**: Fixed Ollama icon not being visible/green in themes other than Apple
  - Added default gray filter and opacity to all themes for icon visibility
  - Fixed green status filter to properly override default styling using `!important`
  - Icon now correctly displays green when Ollama is connected, yellow/amber when unavailable
  - Applied fixes to all 9 themes (apple, dark, architect, data-poster, futuristic, light, material, neon, swiss-minimalistic)
- **Data-Poster Theme Layout**: Fixed battery/power strip layout alignment with Apple theme
  - Removed unwanted grey background box around "Power:" label
  - Fixed battery icon color for dark theme visibility
  - Added missing `--hairline` CSS variable
  - Aligned spacing, padding, and styling to match Apple theme exactly
  - Fixed charging indicator to display green when charging

## [0.1.1] - 2026-01-19

### Fixed
- **Monitor Stats Persistence**: Fixed issue where external monitor stats (last_check, last_status) were not persisting after host reboot
  - Monitor stats are now saved to disk after each check
  - Stats are restored when monitors are loaded on app startup
  - Added `get_monitor_status()` command to retrieve cached stats without performing a new check
  - Stats persist across reboots in the monitors configuration file

## [0.1.0] - 2026-01-19

### Added
- **Monitoring System**: Comprehensive website and social media monitoring
  - Website uptime monitoring with response time tracking
  - Social media platform monitoring (Twitter/X, Facebook, Instagram, LinkedIn, YouTube)
  - Monitor status indicators (up/down) with response time display
  - Configurable monitor intervals and timeout settings
  - Monitor health summary with up/down counts
- **Alert System**: Multi-channel alerting infrastructure
  - Alert rules engine for monitor status changes
  - Alert channel support (prepared for future integrations)
  - Alert history and management
- **Ollama AI Chat Integration**: AI-powered chat assistant
  - Integration with local Ollama instance
  - Chat interface for system metrics queries
  - Model selection and connection status indicators
  - System prompt customization
  - Code execution support for JavaScript
  - Markdown rendering with syntax highlighting
- **Status Icon Line**: Quick access icon bar with status indicators
  - Monitors icon with green status when all monitors are up
  - Ollama icon with green status when connected, yellow when unavailable
  - 15-icon layout with placeholders for future features
  - Click-to-toggle section visibility
- **Dashboard UI**: New dashboard view for monitoring overview
  - Centralized monitoring status display
  - Quick access to all monitoring features
- **Security Infrastructure**: Keychain integration for secure credential storage
  - API key storage in macOS Keychain
  - Secure credential management for monitors and services
- **Plugin System**: Extensible plugin architecture
  - Plugin loading and management infrastructure
  - Prepared for future plugin integrations

### Changed
- **UI Layout**: Added collapsible sections for Monitors and AI Chat
  - Sections can be toggled via header clicks or icon clicks
  - Smooth expand/collapse animations
  - State persistence across sessions
- **Icon Styling**: Enhanced icon display with status-based color coding
  - Green for healthy/connected status
  - Yellow/amber for warnings/unavailable status
  - CSS filters for external SVG icons
- **Connection Status**: Real-time connection status updates
  - Visual indicators for Ollama connection state
  - Automatic connection checking on section expansion

### Technical
- **Backend Commands**: New Tauri commands for monitoring and Ollama
  - `list_monitors`, `add_monitor`, `remove_monitor`, `check_monitor`
  - `check_ollama_connection`, `ollama_chat`, `configure_ollama`
  - `list_alerts`, `add_alert_rule`, `remove_alert_rule`
- **State Management**: Enhanced application state with monitoring and Ollama state
- **Error Handling**: Comprehensive error handling for network requests and API calls
- **Logging**: Structured logging for monitoring and Ollama operations
- **Cross-Theme Support**: All new features (monitoring, Ollama chat, status icons) are available across all 9 themes
- **CSS Architecture**: Universal CSS with cascading variable fallbacks for cross-theme compatibility

## [0.0.6] - 2026-01-18

### Added
- **Power Consumption Monitoring**: Real-time CPU and GPU power consumption monitoring via IOReport Energy Model API
  - CPU power consumption in watts (W)
  - GPU power consumption in watts (W)
  - Power readings only when CPU window is visible (optimized for low CPU usage)
  - `--power-usage` command-line flag for detailed power debugging logs
- **Battery Monitoring**: Battery level and charging status display
  - Battery percentage display
  - Charging status indicator
  - Battery information only read when CPU window is visible
- **Process Details Modal**: Click any process in the list to view comprehensive details including:
  - Process name, PID, and current CPU usage
  - Total CPU time, parent process information
  - Start time with relative time display
  - User and effective user information
  - Memory usage (physical and virtual)
  - Disk I/O statistics (read/written)
- **Force Quit Functionality**: Force quit processes directly from the process details modal
- **Process List Interactivity**: Process rows are now clickable and show cursor pointer
- **Auto-refresh Process Details**: Process details modal automatically refreshes every 2 seconds while open
- **Scrollable Sections**: Added scrollable containers for Details and Processes sections with custom scrollbar styling
- **Process PID Display**: Process list now includes PID information in data attributes
- **Embedded Changelog**: Changelog is now embedded in the binary for reliable access
- **Changelog CLI Flag**: Added `--changelog` flag to test changelog functionality

### Changed
- **Process List Refresh**: Increased refresh interval from 5 seconds to 15 seconds for better CPU efficiency
- **Process Cache**: Improved process cache handling with immediate refresh on window open
- **UI Layout**: Improved flex layout with proper min-height and overflow handling for scrollable sections
- **Process Data Structure**: Added PID field to ProcessUsage struct for better process identification
- **Changelog Reading**: Improved changelog reading with multiple fallback strategies (current directory, executable directory, embedded)

### Performance
- **Smart Process Refresh**: Process details only refresh when CPU window is visible (saves CPU when window is hidden)
- **Conditional Process Updates**: Process list updates immediately on initial load and when window becomes visible
- **Efficient Modal Updates**: Process details modal only refreshes when actually visible
- **Power Reading Optimization**: Power consumption and battery readings only occur when CPU window is visible, maintaining <0.1% CPU usage when window is closed
- **IOReport Power Subscription**: Power subscription is created on-demand and cleaned up when window closes

### Technical
- **IOReport Power Integration**: Implemented IOReport Energy Model API integration for power monitoring
- **Array Channel Support**: Added support for IOReportChannels as arrays (Energy Model format)
- **Memory Management**: Proper CoreFoundation memory management for power channel dictionaries
- **Error Handling**: Graceful handling when power channels are not available on certain Mac models

## [0.0.5] - 2026-01-18

### Performance Improvements
- **Access Flags Optimization**: Replaced `Mutex<Option<_>>` with `OnceLock<bool>` for capability flags (temperature, frequency, power reading) - eliminates locking overhead on every access
- **Process Cache TTL**: Increased process list cache from 5 seconds to 10 seconds to reduce CPU overhead
- **Temperature Update Interval**: Increased from 15 seconds to 20 seconds for better efficiency
- **Frequency Read Interval**: Increased from 20 seconds to 30 seconds to reduce IOReport overhead
- **DOM Update Optimization**: Changed from `innerHTML` rebuilds to direct text node updates for metric values (reduces WebKit rendering overhead)
- **Ring Gauge Thresholds**: Increased update thresholds from 2% to 5% (visual) and 15% to 20% (animation) to reduce unnecessary animations
- **Window Cleanup**: Added cleanup handlers on window unload to clear animation state and pending updates

### Fixed
- **GitHub Actions Workflow**: Fixed workflow to properly handle missing code signing secrets (builds successfully without secrets)
- **Code Signing**: Made code signing optional - workflow builds unsigned DMG when secrets are not available
- **Legacy Code**: Removed outdated ACCESS_CACHE comment references

### Changed
- **Theme Gallery**: Updated README with comprehensive theme gallery showing all 9 themes
- **Screenshot Organization**: Removed old screenshot folders (screen_actual, screen-what-i-see), consolidated to screens/ folder

## [0.0.4] - 2026-01-18

### Added
- **App Version Display**: Added version number display in footer of all HTML templates
- **Version API**: Added `get_app_version` Tauri command to fetch version at runtime
- **Window Decorations Toggle**: Added window frame toggle in settings (affects new windows)
- **Config File Support**: Added persistent configuration file (`~/.mac-stats/config.json`) for window decorations preference
- **Toggle Switch Component**: Added modern toggle switch styling to all themes
- **GitHub Actions Workflow**: Automated DMG build and release on GitHub
- **Build Script**: Added `scripts/build-dmg.sh` for local DMG creation
- **DMG Download Section**: Added download instructions to README with Gatekeeper bypass steps

### Changed
- **Theme Improvements**: Massively improved all themes with better styling and visual consistency
- **Data Poster Theme**: Improved Details section styling to match Processes section (flex layout, consistent font sizes and weights)
- **Metric Unit Styling**: Improved metric unit display (%, GHz) with better font sizing and positioning
- **CPU Usage Display**: Fixed CPU usage value updates to properly maintain HTML structure with unit spans
- **Frequency Display**: Enhanced frequency display to include unit (GHz) with proper formatting
- **HTTPS Support**: Changed git clone URLs from SSH to HTTPS for better accessibility
- **Window Creation**: CPU window now respects window decorations preference from config file

### Fixed
- **Build Configuration**: Fixed Tauri build configuration (custom-protocol feature, bundle settings)
- **Binary Naming**: Fixed binary name from `mac-stats-backend` to `mac_stats` to match package name
- **DMG Detection**: Fixed build-dmg.sh script to properly detect DMG files using zsh array expansion
- **Release Workflow**: Fixed GitHub Actions workflow to properly upload DMG files to releases
- **Version Fetching**: Fixed duplicate command definition by moving `get_app_version` to metrics module

### Documentation
- **README Updates**: Added comprehensive DMG download instructions with Gatekeeper bypass methods
- **Known Limitations**: Added note about window frame toggle behavior (affects new windows only)
- **Installation Guide**: Improved installation section with multiple options and troubleshooting

## [0.0.3] - 2026-01-18

### Added
- **DMG Build Support**: Added DMG bundle creation for macOS distribution
- **GitHub Actions**: Added automated release workflow for building and publishing DMG files

### Changed
- **Version**: Bumped to 0.0.3

## [0.0.2] - 2026-01-18

### Fixed
- **CPU Frequency Reading**: Fixed frequency reading from IOReport to use delta samples instead of absolute counters, providing accurate recent frequency values instead of long-term averages
- **Memory Leaks**: Fixed CoreFoundation object leaks by properly retaining and releasing CF objects (channels_dict, subscription_dict, samples)
- **Crash Safety**: Added validation for IOReport channel dictionaries before calling IOReport functions to prevent crashes from invalid data
- **Channel Filtering**: Made `is_performance_channel()` more restrictive to only match actual CPU performance channels (ECPU*, PCPU*), reducing unnecessary processing

### Changed
- **Delta Sampling**: Frequency calculation now uses `IOReportCreateSamplesDelta()` to compute recent frequency over the sampling interval (20s) instead of since boot
- **Channel Classification**: Improved channel classification to correctly identify E-core (ECPU*) and P-core (PCPU*) channels
- **Frequency Extraction**: Enhanced frequency extraction to handle VxPy voltage/performance state format (e.g., V0P5, V19P0)
- **Command Execution**: Replaced fragile `sh -c` commands with direct binary calls using full paths (`/usr/sbin/sysctl`, `/usr/sbin/system_profiler`)
- **Code Organization**: Removed large redundant comment blocks from refactoring

### Refactored
- **Frequency Reading Logic**: Extracted complex nested frequency reading code from `lib.rs` into modular functions in `ffi/ioreport.rs`, reducing nesting from 5+ levels to max 2-3 levels
- **Array Processing**: Added support for IOReportChannels as an array (type_id=19) in addition to dictionary format
- **Logging**: Refactored `debug1/2/3` macros to use `write_structured_log` with timestamps for consistent logging format

### Added
- **Frequency Logging**: Added `--frequency` command-line flag for detailed frequency debugging logs
- **Validation**: Added validation checks for IOReport channel dictionaries (channel name, state count) before processing
- **Memory Management**: Added proper CFRetain/CFRelease cycles for all stored CoreFoundation objects
- **Cleanup**: Added cleanup path to release all CF objects when CPU window closes

### Security
- **FFI Safety**: Improved FFI safety by validating CoreFoundation types and null pointers before use
- **Memory Safety**: Fixed use-after-free risks by properly managing CF object lifetimes with guards

## [0.1.0] - Initial Release

### Added
- Basic system monitoring (CPU, RAM, Disk, GPU)
- Temperature monitoring via SMC
- CPU frequency monitoring via IOReport
- Process list with top CPU consumers
- Menu bar integration
- Multiple UI themes
- Low CPU usage optimizations
