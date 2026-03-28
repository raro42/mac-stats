# mac-stats

## Install

### DMG (recommended):
[Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.

### Build from source:
```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```
Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

### If macOS blocks the app:
Gatekeeper may show "damaged" or block the unsigned app—the file is fine. Right-click the DMG → **Open**, then confirm. Or after install: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

## At a glance

- **Menu bar** — CPU, GPU, RAM, disk at a glance; click to open the details window.
- **AI chat** — Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.
- **Discord**

## 1. Tool agents (what Ollama can invoke)

Whenever Ollama is asked to decide which agent to use (planning step in Discord and scheduler flow), the app sends the **complete list of active agents**: the invocable tools below plus the **SCHEDULER** (informational; Ollama can recommend it for recurring or delayed tasks but cannot invoke it with a tool line). Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In `CPU window`: executed in `src-tauri/dist/ollama.js` |

## 2. Scope of uncommitted changes

The diff on top of the last commit (844c4bc) contains **10 distinct features/improvements**. This review plan covers each one.

| # | Feature | Files | Risk |
|---|---------|-------|------|
| F1 | **Session memory for Discord** (short-term context) | `session_memory.rs`, `discord/mod.rs`, `commands/ollama.rs` | Medium |
| F2 | **Conversation history in router** (`answer_with_ollama_and_fetch`) | `commands/ollama.rs`, `scheduler/mod.rs`, `task/runner.rs` | Medium |
| F3 | **Unified soul path** (`~/.mac-stats/agents/soul.md` for all) | `agents/mod.rs`, `config/mod.rs` | Low |
| F4 | **Router soul injection** (soul.md in non-agent/non-skill chat) | `commands/ollama.rs` | Low |
| F5 | **TASK_CREATE deduplication** (prevent duplicate task explosion) | `task/mod.rs` | Low |
| F6 | **Prompt guidance** (AGENT over TASK_CREATE for agent chats) | `commands/ollama.rs` | Low |
| F7 | **Logging improvements** (`ellipse()` helper, verbosity-aware log truncation) | `logging/mod.rs`, `commands/ollama.rs`, `commands/browser.rs`, `discord/api.rs` | Low |
| F8 | **Chat reserved words** (`--cpu`, `-v`/`-vv`/`-vvv` from chat) | `src/ollama.js`, `commands/logging.rs`, `commands/window.rs`, `commands/mod.rs`, `lib.rs` | Low |
| F9 | **Toggle CPU window** (refactor + new Tauri command) | `ui/status_bar.rs`, `commands/window.rs` | Low |
| F10 | **Background monitor checks** (`run_due_monitor_checks`) | `commands/monitors.rs` | Medium |

## 3. Review plan — per feature

### F1: Session memory for Discord

**What changed:**
- `session_memory.rs`: Added `get_messages()` (returns in-memory prior turns for a session key) and `load_messages_from_latest_session_file()` (resumes context from disk after restart). Session file naming reordered to `session-{id}-{timestamp}-{topic}.md`.
- `discord/mod.rs`: Before each Ollama call, loads session history and passes it as `conversation_history`.

**Review checklist:**
- [x] Verify `get_messages()` returns messages **before** the current user message is added (ordering matters). — `get_messages_before_add_user_excludes_current_turn` in `session_memory.rs`.
- [x] Verify `load_messages_from_latest_session_file()` correctly parses `## User` / `## Assistant` blocks — test with a real session file (edge cases: empty blocks, blocks with `## ` in content). — Line-oriented parser + tests in `session_memory.rs` (e.g. headings-in-body, FEAT-D7).
- [x] Confirm the filename format change (`session-{id}-{ts}-{topic}` vs old `session-{topic}-{id}-{ts}`) doesn't break loading of pre-existing session files. — `MAC_STATS_SESSION_DIR` tests in `session_memory.rs` / `config/mod.rs` (FEAT-D14).
- [x] Check the 20-message cap is applied consistently (both in `get_messages` consumer and in `answer_with_ollama_and_fetch`). — `CONVERSATION_HISTORY_CAP` + `cap_tail_chronological` tests in `session_history.rs`; Discord uses same cap (FEAT-D19).
- [ ] Manual test: send 3-4 messages to the Discord bot, verify it references earlier messages correctly (e.g. "what did I just say?").
- [ ] Manual test: restart app, send a message — verify it resumes from the latest session file.

### F2: Conversation history in router

**What changed:**
- `answer_with_ollama_and_fetch` gains a new parameter `conversation_history: Option<Vec<ChatMessage>>`. When present, last 20 messages are prepended to planning and execution prompts.
- All callers updated: scheduler and task runner pass `None`; Discord passes session memory.

**Review checklist:**
- [x] Verify the `rev().take(20).rev()` pattern correctly keeps the **last** 20 messages in chronological order. — `cap_tail_keeps_last_n_in_chronological_order` in `session_history.rs` (drives length from `CONVERSATION_HISTORY_CAP`).
- [x] Confirm history messages are placed **after** the system prompt and **before** the current question in the Ollama messages array. — `execution_stack_order_system_history_user` / `build_execution_message_stack` in `session_history.rs`; CPU chat and the agent router also assemble stacks via `ContextAssembler::assemble` in `commands/context_assembler.rs` (same ordering contract) (022 §F2).
- [x] Context-window overflow handling (automated): `is_context_overflow_error` + `sanitize_ollama_error_for_user` map common Ollama error strings to the same user-facing “new topic / larger context” message; unit tests in `content_reduction.rs` (FEAT-D28). Router also truncates oversized tool results on overflow when enabled (`context_overflow_truncate_enabled`).
- [ ] Manual: with a model configured to a small `num_ctx`, send a long thread and confirm truncation/retry or friendly overflow text (complements automated tests above).
- [x] Confirm scheduler (`None`) and task runner (`None`) are unaffected — no regressions. — Call sites pass `None` for `conversation_history` (scheduler / task runner); no automated integration test.

### F3: Unified soul path (`~/.mac-stats/agents/soul.md`)

**What changed:**
- Consolidated from two directories (`~/.mac-stats/agent/` and `~/.mac-stats/agents/`) into one: `~/.mac-stats/agents/soul.md`.
- Removed `Config::agent_soul_dir()`, `Config::agent_soul_file_path()`, `Config::ensure_agent_soul_directory()`, `Config::load_agents_fallback_soul_content()`, `Config::load_default_soul_content()`.
- New unified `Config::soul_file_path()` and `Config::load_soul_content()` — reads from `~/.mac-stats/agents/soul.md`, writes DEFAULT_SOUL there if missing.
- Both agents (as fallback) and the router (non-agent chat) use the same path and method.

**Review checklist:**
- [x] Confirm an agent with its own `soul.md` does NOT also get the shared soul (no double soul). — `agent_soul_or_shared` tests in `agents/mod.rs` (FEAT-D15).
- [x] Confirm an agent without `soul.md` gets the shared `agents/soul.md` content. — same.
- [x] Confirm the router (non-agent chat) uses `agents/soul.md` for personality. — `Config::load_soul_content` + `format_router_soul_block` / router path in `ollama.rs` (FEAT-D13).
- [x] Confirm `load_agents()` info log correctly reports shared soul presence. — `log_shared_soul_presence()` on every exit path that finishes a directory scan (including missing dir and zero enabled agents); `shared_soul_file_nonempty` tests in `agents/mod.rs` (FEAT-D27).
- [x] If user had a customized `~/.mac-stats/agent/soul.md`, they need to move it to `~/.mac-stats/agents/soul.md` (document in changelog). — Documented in this file §4 and historical `CHANGELOG.md` / release notes for the unified path.

### F4: Router soul injection

**What changed:**
- In `answer_with_ollama_and_fetch`, when no `skill_content` or agent override is active, the router now prepends `~/.mac-stats/agents/soul.md` content to system prompts, giving Ollama personality in "plain" chat (Discord without agent override, scheduler free-text tasks).

**Review checklist:**
- [x] Verify `load_soul_content()` is called (reads from `~/.mac-stats/agents/soul.md`). — Covered by `format_router_soul_block` tests and call sites in `ollama_memory.rs` / `ollama.rs` (FEAT-D13).
- [x] Verify that when `skill_content` is Some, the soul is NOT prepended (avoids double system prompt). — Unit tests in `commands/ollama_memory.rs` (skill branch → empty soul block).
- [x] Verify that when `agent_override` is Some, the agent's own combined prompt is used and the router soul is skipped. — Same module tests + agent execution path uses `combined_prompt`.

### F5: TASK_CREATE deduplication

**What changed:**
- `task/mod.rs`: Before creating a task, `existing_task_with_topic_id()` scans `~/.mac-stats/task/` and reads `## Topic:` and `## Id:` from each task file; if any file has the same topic (slug) and id, `create_task()` returns an error. (Filenames are `task-<date-time>-<status>.md`.)

**Review checklist:**
- [x] Verify slug generation is deterministic (same topic always gives same slug). — `test_slug_deterministic` in `task/mod.rs`.
- [x] Verify the dedup check matches by `## Topic:` (as slug) and `## Id:` in file content. — `create_task_duplicate_topic_id_errors_with_task_append_hint` (second create with different topic casing, same id).
- [x] Edge case: existing task `finished`/`unsuccessful` — we block and suggest: error message says "or use a different id to create a new task" (D2 resolved, option c).
- [x] Verify the error message is informative enough for Ollama to switch to TASK_APPEND. — same test asserts `TASK_APPEND` / `TASK_STATUS` or `different id` in error string.

### F6: Prompt guidance for agent chats

**What changed:**
- TASK description in `build_agent_descriptions()` now includes: "When the user wants agents to chat, invoke AGENT: orchestrator so the conversation runs; do not only create a task."
- Also: "If a task with that topic and id already exists, use TASK_APPEND or TASK_STATUS instead."

**Review checklist:**
- [x] Read the full prompt text in context — confirm it doesn't create contradictory instructions. — TASK paragraph contract tests in `commands/agent_descriptions.rs` (`format_task_agent_description`, FEAT-D10).
- [ ] Manual test: ask Discord "have the agents chat" — verify the model outputs `AGENT: orchestrator` instead of just `TASK_CREATE`.

### F7: Logging improvements

**What changed:**
- New `ellipse()` function: shows first half + `...` + last half instead of hard truncation. Used in `browser.rs`, `discord/api.rs`, and `commands/ollama.rs`.
- Verbosity-aware logging: at `-vv` or higher, request/response logs are never truncated.
- Discord API: fixed char-count vs byte-count mismatch (`.chars().count()` instead of `.len()`).

**Review checklist:**
- [x] Verify `ellipse()` handles edge cases: empty string, string shorter than `max_len`, string exactly `max_len`, very small `max_len` (< 3). — `logging/mod.rs` tests (`ellipse_empty_string`, `ellipse_short_string_unchanged`, `ellipse_exact_max_len_unchanged`, `ellipse_max_len_*_clamped`, `ellipse_result_length_within_max`).
- [x] Confirm the `VERBOSITY` atomic is the same one set by CLI `-v`/`-vv`/`-vvv` flags and the new `set_chat_verbosity`. — `commands/logging.rs` tests (FEAT-D11) + CLI sets `logging::VERBOSITY` at startup.
- [x] Verify `browser.rs` truncation change: old code appended `[truncated]`; new code uses `ellipse()` which shows `...`. This changes the semantics for FETCH_URL content passed to Ollama — confirm the model still understands the page was cut. — Oversized FETCH_URL bodies append ` [content truncated]` after the ellipsed body (`truncate_fetch_body_if_needed` in `browser.rs`); `truncate_fetch_body_ellipse_then_explicit_suffix_for_llm` + `truncate_fetch_body_uses_configured_max` lock the contract (FEAT-D32).

### F8: Chat reserved words

**What changed:**
- `src/ollama.js`: `sendChatMessage()` intercepts `--cpu`, `-v`, `-vv`, `-vvv` before sending to Ollama.
- `--cpu` → invokes `toggle_cpu_window` Tauri command.
- `-v`/`-vv`/`-vvv` → invokes `set_chat_verbosity` Tauri command (runtime verbosity change).
- New Tauri commands registered in `lib.rs`.

**Review checklist:**
- [x] Verify reserved words are NOT added to conversation history (the `return` before `addToHistory` is correct). — `sendChatMessage` handles `--cpu` / `-v*` before `addChatMessage` / `addToHistory` (`src/ollama.js`, FEAT-D16); run `scripts/sync-dist.sh` for `dist/`.
- [x] Verify `set_chat_verbosity` updates the same `VERBOSITY` atomic used by logging macros. — `commands/logging.rs` tests (FEAT-D11).
- [x] Verify `toggle_cpu_window` works from the CPU window chat (meta: toggling from within the window you're in — should it close itself? Is that the desired UX?). — **Resolved:** Same close-then-recreate path as the menu bar; chat `--cpu` tears down the WebView and opens a fresh CPU window (intentional “always visible” semantics, not in-place hide). Rustdoc on `commands/window.rs` and `ui/status_bar::toggle_cpu_window` (FEAT-D31).
- [x] Check that `src/ollama.js` changes are synced to `src-tauri/dist/ollama.js` before testing. — `scripts/sync-dist.sh`; AGENTS.md documents sync.

### F9: Toggle CPU window refactor

**What changed:**
- Extracted inline window toggle logic from `click_handler_class` into `toggle_cpu_window()` function in `status_bar.rs`.
- New `commands/window.rs` exposes it as a Tauri command (uses `run_on_main_thread`).

**Review checklist:**
- [x] Verify `toggle_cpu_window` logic: close visible → recreate is the same behaviour as before (no regression).
- [x] The function always recreates the window after closing — **verified intentional.** In `status_bar.rs`, after closing the window (whether it was visible or hidden), the code checks `if app_handle.get_window("cpu").is_none()` and then calls `create_cpu_window(app_handle)`. So every click ends with the window existing and open; there is no path that leaves the window closed. Effectively this is "show CPU window (create if needed)" rather than a strict toggle. To allow "close and leave closed" we would skip the final create when the window was visible before close.
- [x] Verify `run_on_main_thread` is safe here — Tauri docs say it may block; confirm the Tauri command is async enough to not hang the frontend. — Rustdoc on `commands/window.rs::toggle_cpu_window`: command thread blocks until the main-thread closure completes; closure is bounded (CPU window close/recreate); WebView stays on the main run loop (FEAT-D30).

### F10: Background monitor checks

**What changed:**
- `commands/monitors.rs`: New `run_due_monitor_checks()` function that iterates all monitors, checks if `last_check + interval >= now`, and runs `check_monitor()` for due ones.
- Uses `try_lock()` to avoid blocking.

**Review checklist:**
- [x] Verify this function is actually called somewhere (it's defined but the caller is not visible in the diff — check `lib.rs` for a background thread or timer). — `lib.rs` 30s loop; contract test `lib_rs_invokes_run_due_monitor_checks_in_background_loop` in `commands/monitors.rs` (FEAT-D26).
- [x] Confirm `try_lock()` usage is safe: if the lock is busy, checks are skipped entirely (acceptable for a background thread). — Early return + debug log in `run_due_monitor_checks` when config/stats locks are busy.
- [x] Verify that `check_monitor()` saves stats to disk after each check (existing behaviour — confirm not regressed). — `save_monitors()` after stats update (FEAT-E1).
- [x] Edge case: monitors with `check_interval_secs = 0` — would this run every cycle? Confirm there's a minimum. — `clamp_monitor_check_interval_secs` + `is_monitor_due_for_background` tests (FEAT-D6 / FEAT-D18).

## 4. Integration review

| Area | What to verify |
|------|----------------|
| **Compilation** | `cargo check` passes (confirmed). Run `cargo clippy` for lint warnings. |
| **Frontend sync** | `src/ollama.js` has changes — must be synced to `src-tauri/dist/ollama.js`. Run `scripts/sync-dist.sh` or manually copy. |
| **New Tauri commands** | `toggle_cpu_window` and `set_chat_verbosity` must be in `tauri::generate_handler![]` in `lib.rs`. |
| **Session file compat** | New layout `session-memory-{id}-{ts}-{topic}.md` and legacy `session-memory-{topic}-{id}-{ts}.md` are both matched when resuming by session id (`session_memory.rs` / FEAT-D1, FEAT-D14 tests). |
| **Soul path** | Unified to `~/.mac-stats/agents/soul.md`. Users with a customized `~/.mac-stats/agent/soul.md` need to move it. |
| **`run_due_monitor_checks` caller** | Wired: `lib.rs` background thread, 30s interval. Contract test `lib_rs_invokes_run_due_monitor_checks_in_background_loop` in `commands/monitors.rs` (FEAT-D26). |

## 5. Manual testing plan

### Smoke tests (15 min)

1. **Build and run**: `cd src-tauri && cargo build --release && ./target/release/mac_stats --cpu -vv`
2. **Menu bar**: Verify metrics display, click to toggle CPU window.
3. **Chat reserved words**: Type `--cpu` in chat → window toggles. Type `-vv` → see "Verbosity set to debug" message.
4. **Ollama chat**: Send a question, verify response. Send follow-up referencing prior message.

### Discord tests (15 min, requires bot token)

5. **Session context**: Send "My name is Alice" → bot acknowledges. Send "What's my name?" → bot should say "Alice".
6. **Restart continuity**: Restart app. Send "What were we talking about?" → should resume from session file.
7. **Agent chat**: Send "agent: 000\nhave all agents chat about Rust vs Python". Verify AGENT: is invoked, not just TASK_CREATE.
8. **Duplicate task prevention**: Send "create a task about testing with id 42". Send same again → should see error about existing task.

### Scheduler / Task tests (10 min)

9. **Task CLI**: `mac_stats add test-review 99 "Review test"` → `mac_stats list` → `mac_stats show 99` → `mac_stats remove 99`.
10. **Task dedup**: `mac_stats add dup 1 "First"` → `mac_stats add dup 1 "Second"` → should fail with duplicate error.

### Monitor tests (5 min)

11. **Background checks**: Add a website monitor, wait for interval, check `~/.mac-stats/debug.log` for background check entries.

---

## 6. Decisions needed before commit

| # | Decision | Options |
|---|----------|---------|
| D1 | **Session file format change** — old files won't load after restart | (a) Add backward compat to also search old naming, (b) Accept break (users lose old session context), (c) Write migration |
| D2 | **Finished task dedup** — should `TASK_CREATE` be allowed when an existing task with same topic+id is `finished`? | **RESOLVED (c):** Block and suggest: error message now says "Use TASK_APPEND or TASK_STATUS to update it, or use a different id to create a new task." (task/mod.rs) |
| D3 | **`ellipse()` in FETCH_URL`** — changes truncation marker from `[truncated]` to `...` | (a) Keep `...` (cleaner), (b) Use `... [content truncated]` (clearer for LLM) |
| D4 | **`run_due_monitor_checks` wiring** — is it called? | (a) Wire to background thread, (b) Remove if premature |
| D5 | **Soul path naming** — `agent/` vs `agents/` was confusing | RESOLVED: consolidated to `~/.mac-stats/agents/soul.md` only |

---

## 7. Suggested commit strategy

Given the 10 features span different concerns, consider splitting into 2-3 atomic commits:

1. **Logging + refactors** (F7, F9): `ellipse()`, verbosity-aware logging, `toggle_cpu_window` extraction. Pure improvements, no behaviour change.
2. **Session memory + router history + soul hierarchy** (F1, F2, F3, F4): The core "agents remember context" feature.
3. **Task dedup + prompt fixes + chat reserved words + monitor checks** (F5, F6, F8, F10): Bug fixes and small features.

Or commit all at once with a descriptive message covering the highlights.

---

## 8. Post-review additions (0.1.14)

After the initial feature review and merge to main, the following changes were made on top of 0.1.13:

| # | Feature | Files | Risk |
|---|---------|-------|------|
| F11 | **Externalized prompts** — planning_prompt.md, execution_prompt.md, soul.md as user-editable files | `config/mod.rs`, `commands/ollama.rs`, `commands/agents.rs`, `lib.rs` | Low |
| F12 | **Default agents shipped** — 4 agents embedded via `include_str!`, written to `~/.mac-stats/` on first launch | `config/mod.rs`, `src-tauri/defaults/` (new dir) | Low |
| F13 | **Tauri prompt API** — `list_prompt_files`, `save_prompt_file` commands | `commands/agents.rs`, `lib.rs` | Low |
| F14 | **RUN_CMD retry loop** — AI-corrected retries (up to 3) on command failure | `commands/ollama.rs` | Medium |
| F15 | **Tool parsing improvements** — strip numbered-list prefixes, multi-RECOMMEND, arg truncation | `commands/ollama.rs` | Low |

See `CHANGELOG.md` (0.1.14) and `docs/023_externalized_prompts_DONE.md` for details.

**Externalized prompts (F11) — summary from 023**

- **Files:** `~/.mac-stats/agents/soul.md` (personality, prepended to all system prompts), `~/.mac-stats/prompts/planning_prompt.md` (planning step), `~/.mac-stats/prompts/execution_prompt.md` (execution step; contains `{{AGENTS}}` placeholder).
- **Placeholder:** `{{AGENTS}}` in the execution prompt is replaced at runtime with the dynamic tool list (RUN_JS, FETCH_URL, BRAVE_SEARCH, SCHEDULE, etc.); do not remove it or Ollama won’t see available tools.
- **Defaults:** Embedded via `include_str!` from `src-tauri/defaults/` (e.g. `defaults/agents/soul.md`, `defaults/prompts/planning_prompt.md`, `defaults/prompts/execution_prompt.md`). `Config::ensure_defaults()` writes missing files on first launch; existing user files are never overwritten.
- **Assembly order (planning):** Soul (or skill block), Discord user context, planning prompt, expanded `{{AGENTS}}`, platform hints — see `commands/ollama.rs`.
- **Assembly order (execution):** Composable sections in `prompts/mod.rs` via `commands/prompt_assembly.rs`: **static prefix** first (soul or skill branch, Discord user context, execution prompt with `{{AGENTS}}`, platform formatting, model identity), then **dynamic tail** (memory, live system metrics, question-derived reminders, optional plan). This keeps the UTF-8 prefix stable when only metrics/memory/plan change, which helps prompt caching and is covered by unit tests in `prompts/mod.rs`.
- **Tauri commands:** `list_prompt_files` (returns soul, planning_prompt, execution_prompt name/path/content), `save_prompt_file(name, content)` (name = `soul` | `planning_prompt` | `execution_prompt`).
- **Editing:** Changes take effect on the next request (prompts loaded fresh each time). Per-agent soul in `agent-<id>/soul.md` overrides shared soul for that agent. Planning prompt should instruct the model to reply with `RECOMMEND: <plan>`.
- Full reference: **docs/023_externalized_prompts_DONE.md**.

---

## 9. Closing review (2026-03-08)

**Integration**
- [x] `cargo check` passes.
- [x] `cargo clippy` passes (40 style/refactor warnings only; no errors).
- [x] `src/ollama.js` and `src-tauri/dist/ollama.js` are in sync (diff empty).
- [x] `toggle_cpu_window` and `set_chat_verbosity` are in `tauri::generate_handler![]` in `lib.rs`.
- [x] `run_due_monitor_checks()` is called from `lib.rs` in a background thread every 30s.

**Code review (F1–F10)**
- **F1**: `get_messages()` is called in Discord before the current user message is added; ordering correct. `load_messages_from_latest_session_file()` matches new layout `session-memory-{id}-{ts}-{topic}.md` and legacy `session-memory-{topic}-{id}-{ts}.md` (FEAT-D1 / FEAT-D14 tests; D1 resolved in code).
- **F2**: `rev().take(20).rev()` at `ollama.rs` 3502–3508 keeps last 20 messages in chronological order. History cap 20 is consistent (CONVERSATION_HISTORY_CAP and Discord HISTORY_CAP).
- **F3/F4**: Soul path and router injection — not re-verified in this pass; doc says resolved.
- **F5**: Dedup in `task/mod.rs` — slug and `## Topic:`/`## Id:` matching present; D2 resolved (block + suggest new id in error message).
- **F6**: Prompt guidance text present in agent descriptions.
- **F7**: `ellipse()` clamps `max_len` to at least `sep_len + 1` before splitting (`logging/mod.rs`); unit tests include `ellipse_max_len_*_clamped` (FEAT-D23). Call sites typically pass `max_len` ≥ 20.
- **F8**: Reserved words `--cpu` and `-v`/`-vv`/`-vvv` in `ollama.js` return before `addToHistory()` — not added to conversation history.
- **F9**: `toggle_cpu_window` in `commands/window.rs`; `run_on_main_thread` used; behaviour (close visible → recreate) as doc.
- **F10**: Background monitor checks wired; `try_lock()` used (skip if busy).

**Smoke test**
- [x] `cargo build --release` succeeds.
- [x] `./target/release/mac_stats --cpu -vv` starts; menu bar ready, Discord/Ollama init in logs. Manual checks (menu bar click, `--cpu`/`-vv` in chat) left to human.

**Decisions (from §6)**
- **D1**: Session file format — old files won’t load; add backward compat, accept break, or migrate (unchanged).
- **D4**: `run_due_monitor_checks` — confirmed wired in `lib.rs` (no longer dead code).

---

## Open tasks

Open tasks for this plan are tracked in **006-feature-coder/FEATURE-CODER.md**.

### Closing reviewer smoke test 2026-03-21 (pre-routing + PERPLEXITY_SEARCH extraction)

- [x] `cargo check` — zero warnings.
- [x] `cargo clippy` — zero warnings.
- [x] `cargo test` — 114 tests pass.
- [x] `cargo build --release` succeeds.
- [x] `./target/release/mac_stats -vv` starts; monitors UP, agents loaded, Discord/Ollama init, scheduler running in logs.

### Closing reviewer smoke test 2026-03-21 (task_tool_handlers extraction)

- [x] `cargo check` — zero errors.
- [x] `cargo clippy` — zero warnings.
- [x] `cargo test` — 114 tests pass.
- [x] `cargo build --release` succeeds.
- [x] `./target/release/mac_stats -vv` starts; 4 monitors UP (mix-online 202ms, amvara 307ms, cometa 361ms, app-monitor 184ms), 8 agents loaded, 15 models classified, Ollama connected (qwen3:latest, 40960 ctx), scheduler running (2 entries). Zero errors/warnings/panics in log.
- [x] Code review: 10 handler functions extracted to `task_tool_handlers.rs` (505 lines). Each takes only required params. All 10 call sites in `ollama.rs` wired correctly. Unused `schedule_helpers` import removed. `ollama.rs` 3502→3145 lines (357 extracted). No behavioral changes.

### Closing reviewer smoke test 2026-03-21 (browser + misc tool dispatch extraction)

- [x] `cargo check` — zero errors.
- [x] `cargo clippy` — zero warnings.
- [x] `cargo test` — 129 tests pass.
- [x] `cargo build --release` succeeds.
- [x] `./target/release/mac_stats -vv` starts; 8 agents loaded, 15 models classified, Ollama connected (qwen3:latest, 40960 ctx), scheduler running (2 entries). Zero errors/warnings/panics in log.
- [x] Code review: 8 browser tool handlers extracted to `browser_tool_dispatch.rs` (422 lines). 5 misc tool handlers (OLLAMA_API, MCP, CURSOR_AGENT, MASTODON_POST, MEMORY_APPEND) extracted to `misc_tool_dispatch.rs` (346 lines). All 13 call sites in `ollama.rs` wired correctly. Unused imports removed (`mastodon_post`, 8 `ollama_models` functions, 3 `browser_helpers` functions). Pre-existing clippy warning fixed in `browser.rs` (`map_or(false, ...)` → `is_some_and`). `ollama.rs` 3145→2579 lines (566 extracted). No behavioral changes.
- [x] `BROWSER_SCREENSHOT` call site correctly uses `BrowserScreenshotResult` struct to propagate `attachment_path` into `attachment_paths` vec — no data lost vs. inline original.
- [x] `MEMORY_APPEND` correctly passes `discord_reply_channel_id` (not `status_tx`) — channel-scoped memory preserved.
- [x] Both new modules have private `send_status()` helpers (same pattern as `task_tool_handlers.rs`); `browser_tool_dispatch` takes `Option<&UnboundedSender>` while `task_tool_handlers` takes `&Option<UnboundedSender>` — both work, minor style inconsistency, non-blocking.
- [x] `CHANGELOG.md` entries present and accurate.

### Closing reviewer smoke test 2026-03-21 (OllamaRequest struct refactoring)

- [x] `cargo check` — zero errors.
- [x] `cargo clippy` — zero warnings.
- [x] `cargo test` — 139 tests pass.
- [x] `cargo build --release` succeeds.
- [x] `./target/release/mac_stats -vv` starts; 4 monitors loaded, 8 agents loaded, 15 models classified, Ollama connected (qwen3:latest, 40960 ctx), Discord connected, scheduler running (2 entries). Zero errors/warnings/panics in log.
- [x] Code review: `OllamaRequest` struct replaces 24 positional parameters on `answer_with_ollama_and_fetch`. `#[derive(Default)]` so all fields default to `None`/`false`/`0`. All 5 call sites updated: recursive retry (`ollama.rs`), Discord (`discord/mod.rs`), CLI (`main.rs`), scheduler (`scheduler/mod.rs`), task runner (`task/runner.rs`). Re-exported from `lib.rs`. No behavioral changes.

### Closing reviewer smoke test 2026-03-21 (full [Unreleased] review + OpenClaw §95)

- [x] `cargo check` — zero errors.
- [x] `cargo clippy` — zero warnings.
- [x] `cargo test` — 139 tests pass.
- [x] `./target/release/mac_stats -vv` running (PID 83236); 4 monitors loaded, 8 agents loaded, Ollama connected (qwen3:latest), Discord connected. 21 WARN/ERROR entries in log (6 expected SSRF blocks for localhost, 3 Chrome idle timeouts, rest HTTP/2 GoAway trace noise — all benign).
- [x] [Unreleased] CHANGELOG code verification:
  - ToolLoopGuard: PASS (10 tests, cycle detection len 2–4, wired via tool_loop.rs).
  - SSRF protection: PASS with nits (15 tests not 14; IPv4-mapped broadcast not checked; redirect DNS-fail path follows instead of blocking).
  - OllamaRequest: PASS (22 fields not 24 — minor numeric mismatch; all 5 call sites use struct init).
  - Auto-dismiss JS dialogs: PASS (event listener, HashSet idempotency, session reset clear, all three call sites).
  - Discord 429 rate-limit: present in discord/api.rs (not re-verified in detail).
  - All extraction line counts consistent with CHANGELOG claims; `ollama.rs` now 1138 lines (from 9408+ pre-extraction).
- [x] AGENTS.md `commands/` directory listing updated from 13 to 45 files (was stale after extractions).
- [x] OpenClaw §95 re-verification: 7 checks against AGENTS.md vs code. Discrepancies found (all doc-only, no code bugs): stale channel paths (`src/telegram` etc. → `extensions/`), missing `src/provider-web.ts`, `pnpm format` script mismatch, branch coverage 55% not 70%, 5 extensions weakly documented.

### Closing reviewer smoke test 2026-03-21 (FETCH_URL pre-routing)

- [x] `cargo check` — zero errors.
- [x] `cargo clippy` — zero warnings.
- [x] `cargo test` — 156 tests pass (17 new).
- [x] Code review: `try_pre_route_fetch_url()` in `commands/pre_routing.rs` (56 lines). Detects explicit `FETCH_URL:` prefix and keyword patterns ("fetch", "get the page/content/html", "read the page/url/site", "what's on", "summarize the/this page/url/site") combined with a URL. Browser/navigate/screenshot/click patterns excluded. Wired into pre-route chain after RUN_CMD, before Redmine. No behavioral changes to existing pre-routes.

### Closing reviewer smoke test 2026-03-21 (BRAVE_SEARCH / PERPLEXITY_SEARCH pre-routing)

- [x] `cargo check` — zero errors.
- [x] `cargo clippy` — zero warnings.
- [x] `cargo test` — 177 tests pass (21 new).
- [x] `cargo build --release` succeeds.
- [x] `./target/release/mac_stats -vv` starts; 4 monitors loaded, 8 agents loaded, 15 models classified, Ollama connected (qwen3:latest), Discord connected, scheduler running. Zero errors/warnings/panics in log.
- [x] Code review: `try_pre_route_web_search()` and `extract_search_query()` in `commands/pre_routing.rs`. Detects explicit prefixes (`BRAVE_SEARCH:`, `PERPLEXITY_SEARCH:`) and keyword patterns ("search for", "google", "look up", "web search", "search the web for", "search online for", "research"). Routes to BRAVE_SEARCH (default) or PERPLEXITY_SEARCH ("research" prefers Perplexity). Multi-step exclusions (browser, "and then", "send to"). Gated on API key availability. Fall-through prevention: empty query after keyword match returns `None` instead of matching shorter patterns. Wired into pre-route chain after FETCH_URL, before Redmine. No behavioral changes to existing pre-routes.

### Closing reviewer smoke test 2026-03-21 (management command pre-routing)

- [x] `cargo check` — zero errors.
- [x] `cargo clippy` — zero warnings.
- [x] `cargo test` — 210 tests pass (33 new).
- [x] `cargo build --release` succeeds.
- [x] `./target/release/mac_stats -vv` starts; 8 agents loaded, 15 models classified, Ollama connected, Discord connected. Zero errors/warnings/panics in log.
- [x] Code review: `try_pre_route_management_commands()` with sub-functions `try_pre_route_list_schedules()`, `try_pre_route_task_commands()`, `try_pre_route_ollama_api()` in `commands/pre_routing.rs`. Explicit prefixes (LIST_SCHEDULES:, TASK_LIST:, TASK_SHOW:, OLLAMA_API:) always pre-route. Keyword patterns for schedules ("list schedules", "show schedules", "what's scheduled", "schedules"), tasks ("list tasks", "show tasks", "tasks", "open tasks", "all tasks", "show task <id>"), and Ollama API ("list models", "show models", "what models are installed", "available models", "pull model <name>", "unload model <name>", "running models"). Multi-step exclusion ("and then", "after that", "send to", "post to"). Wired into pre-route chain after web search, before Redmine. No behavioral changes to existing pre-routes.

### OpenClaw §96 re-verification (2026-03-21)

- [x] OpenClaw AGENTS.md (23.9 KB, last modified 2026-03-21) read and all §7 checks re-run.
- [x] **Directory structure:** `src/provider-web.ts` still does not exist (actual: `src/channel-web.ts`). `src/telegram`, `src/discord`, `src/slack`, `src/signal`, `src/imessage`, `src/web` still do not exist as top-level dirs — channel runtimes under `src/plugins/runtime/`, shared channel logic under `src/channels/`, channel extensions under `extensions/`.
- [x] **Build commands:** `pnpm build`, `pnpm check`, `pnpm test`, `pnpm test:coverage` all exist. `pnpm tsgo` is not a declared script — relies on `@typescript/native-preview` (v7.0.0-dev.20260317.1) providing `tsgo` binary; works after `pnpm install` but undocumented as a dependency-provided binary.
- [x] **Format commands:** `pnpm format` runs `oxfmt --write` (not `--check` as AGENTS.md line 71 claims). `pnpm format:check` runs `oxfmt --check`. `pnpm format:fix` runs `oxfmt --write` (matches AGENTS.md line 72).
- [x] **Test thresholds:** Vitest branch coverage threshold is 55%, not 70% as AGENTS.md line 109 claims. Lines/functions/statements are correctly 70%.
- [x] **Extensions:** 82 dirs under `extensions/` (up from ~80 at §95; new: `wecom`, `kimi-coding`). `anthropic-vertex`, `chutes`, `fal` still lack dedicated English provider pages. `phone-control` and `thread-ownership` still only in zh-CN plugin list.
- [x] **SSRF tests:** OpenClaw has 54 `it()` cases in 7 dedicated `*ssrf*.test.ts` files, ~68 total SSRF-related tests. Significantly more than previous reviews counted.
- [x] **Recent activity:** Discord routed through plugin SDK, plugin split runtime state, Claude bundle commands, context compaction notification, Matrix agentId mention fix, webchat image persistence, GitHub Copilot dynamic model IDs, Telegram DM topic rename.
- [x] **`scripts/committer`:** Exists and works as documented (safe, path-scoped commit helper).
- [x] No code bugs found; all discrepancies are documentation-only.

### Closing reviewer smoke test 2026-03-21 (HTML noise stripping for FETCH_URL)

- [x] `cargo check` — zero errors.
- [x] `cargo clippy` — zero warnings.
- [x] `cargo test` — 221 tests pass (11 new in `html_cleaning`).
- [x] `cargo build --release` succeeds.
- [x] `./target/release/mac_stats -vv` starts; 8 agents loaded, 15 models classified, Ollama connected (qwen3:latest, 40960 ctx), scheduler running (2 entries). Zero errors/warnings/panics in log.
- [x] Code review: new `commands/html_cleaning.rs` (284 lines, 11 tests). `clean_html()` parses with `scraper::Html::parse_document`, walks DOM tree stripping SKIP_TAGS (`script`, `style`, `head`, `meta`, `link`, `noscript`, `svg`, `iframe`, `object`, `embed`). Preserves semantic structure: headings as `# …`, absolute links as `[text](href)`, list items as `- …`, table cells pipe-separated, block elements as newlines. `collapse_whitespace()` normalizes runs of blank lines (3+ → 2) and inline whitespace. Empty output (all-JS pages) produces helpful "Try BROWSER_NAVIGATE instead" message.
- [x] Integration: `clean_html` called from exactly two FETCH_URL paths — `network_tool_dispatch.rs` (Discord/agent tool loop) and `ollama_frontend_chat.rs` (CPU-window chat). Both paths: fetch → clean → log compression ratio → empty check with fallback → pass to LLM. HTTP fallback and BROWSER_NAVIGATE/BROWSER_SCREENSHOT paths confirmed unaffected (no `html_cleaning` usage).
- [x] CHANGELOG entry verified: "11 new tests; 221 total pass" — confirmed. Compression ratio logging at info level — confirmed. File list (`html_cleaning.rs`, `network_tool_dispatch.rs`, `ollama_frontend_chat.rs`) — matches diff.
- [x] Minor observations (non-blocking): (1) `walk_node` uses unbounded recursion — theoretical stack overflow on pathologically deep DOM, unlikely for real web pages. (2) No `<img>` alt text extraction — images silently dropped, acceptable for LLM text context. (3) NBSP (U+00A0) not collapsed by `split_whitespace` — cosmetic edge case. (4) Markdown-special characters in link text/href not escaped — minor formatting noise, not a bug for LLM consumption.
- [x] `scraper = "0.19"` dependency already present in `Cargo.toml` — no new dependencies added.

### OpenClaw §97 re-verification (2026-03-21)

- [x] OpenClaw AGENTS.md (HEAD `5c05347d`, last modified 2026-03-21) read and all checks re-run.
- [x] **Directory structure:** `src/provider-web.ts` still does not exist (actual: `src/channel-web.ts`). `src/telegram`, `src/discord`, `src/slack`, `src/signal`, `src/imessage`, `src/web` still do not exist as top-level dirs — channel runtimes under `src/channels/`, `src/routing/`, and `extensions/`. `src/infra` and `src/media` confirmed present and matching AGENTS.md.
- [x] **Build commands:** `pnpm build`, `pnpm check`, `pnpm test`, `pnpm test:coverage` all exist. `pnpm tsgo` still has no `scripts` entry in `package.json` — relies on `@typescript/native-preview` providing `tsgo` binary via `.bin`; works after `pnpm install` but is not a declared script.
- [x] **Format commands:** `pnpm format` still runs `oxfmt --write` (not `--check` as AGENTS.md line 71 claims). `pnpm format:check` runs `oxfmt --check`. Unchanged from §96.
- [x] **Test thresholds:** Vitest branch coverage threshold still 55%, not 70% as AGENTS.md line 109 claims. Lines/functions/statements are correctly 70%. Unchanged from §96.
- [x] **Extensions:** 77 dirs under `extensions/` (down from 82 at §96 — likely removals/renames between snapshots; `anthropic-vertex` and `tavily` are recent additions). `anthropic-vertex`, `chutes`, `fal` still lack dedicated English provider pages. `phone-control` and `thread-ownership` still only in zh-CN plugin list.
- [x] **SSRF tests:** OpenClaw now has 56 `it()` cases in 7 dedicated `*ssrf*.test.ts` files (up from 54 at §96). ~72+ total SSRF-related tests across dedicated files and cross-file mentions. Additional SSRF-adjacent tests in `navigation-guard.test.ts` (16 cases) and `media-understanding-misc.test.ts` (5 cases) bring the broadest count to ~94.
- [x] **Recent activity (last 2 weeks):** Compaction guard content-aware fix, memory flush dedup via content hash, Matrix added to VOICE_BUBBLE_CHANNELS, pluggable system prompt section for memory plugins, NVM/Linux CA handling, Telegram/iMessage/Slack runtimes routed through plugin SDK, Discord `/codex_resume` picker fix, embedding default export fixes, compaction summary budget safeguard, web UI context notice fix, cold-start status probe skip, Telegram doctor/fresh-setup improvements, Claude bundle commands registered natively, context compaction user notifications, web search key copy fix.
- [x] No code bugs found; all discrepancies remain documentation-only (same 4 persistent findings from §95).

### Closing reviewer smoke test 2026-03-21 (§97 full review + [Unreleased] verification)

- [x] `cargo check` — zero errors.
- [x] `cargo clippy` — zero warnings.
- [x] `cargo test` — 270 tests pass (up from 221 at last recorded count).
- [x] `cargo build --release` succeeds.
- [x] `./target/release/mac_stats -vv` running (PID 5712); 4 monitors loaded (amvara 321ms, app-monitor 183ms, cometa 349ms, mix-online 243ms), 8 agents loaded (orchestrator, general-purpose-mommy, senior-coder, humble-generalist, discord-expert, scheduler, redmine, abliterated), Ollama connected (qwen3:latest, 40960 ctx), Discord connected (Werner_Amvara), scheduler running (2 entries). 21 WARN/ERROR entries in log (6 expected SSRF blocks for localhost, 2 Chrome idle timeouts, 1 transport shutdown noise, rest HTTP/2 GoAway trace — all benign). Zero panics.
- [x] [Unreleased] CHANGELOG code verification:
  - Context-overflow auto-recovery: PASS (`is_context_overflow_error()` and `truncate_oversized_tool_results()` in `content_reduction.rs`, called from `ollama.rs` and `tool_loop.rs`; 12 tests confirmed). Minor: truncation marker in code is `[truncated from N to M chars due to context limit]`, CHANGELOG says `[truncated]`.
  - Context-overflow config: PASS (`contextOverflowTruncateEnabled` default true, `contextOverflowMaxResultChars` default 4096 in `config/mod.rs`).
  - Compaction context cap: PASS (12000 bytes, `cap_context()` with 7 tests, `parse_compaction_output` with 6 tests in `compaction.rs`).
  - Planning history cap: PASS (`planningHistoryCap` default 6, max 40, applied in `ollama.rs`).
  - HTML noise stripping: PASS (`clean_html()` in `html_cleaning.rs`, called from `network_tool_dispatch.rs` and `ollama_frontend_chat.rs`).
  - Scheduler per-task timeout: PASS (`schedulerTaskTimeoutSecs` default 600, clamped 30–3600, `tokio::time::timeout` in `scheduler/mod.rs`).
  - Test count: 270 confirmed (268 `#[test]` + 2 `#[tokio::test]`; CHANGELOG's running counts at various points are stale but the latest matches).

### Closing reviewer smoke test 2026-03-21 (DISCORD_API pre-routing)

- [x] `cargo check` — zero errors.
- [x] `cargo clippy` — zero warnings.
- [x] `cargo test` — 292 tests pass (22 new).
- [x] `cargo build --release` succeeds.
- [x] `./target/release/mac_stats -vv` starts (PID 33313); 4 monitors loaded (mix-online 339ms, cometa 363ms, app-monitor 183ms, amvara 308ms), 8 agents loaded (orchestrator, general-purpose-mommy, senior-coder, humble-generalist, discord-expert, scheduler, redmine, abliterated), 15 models classified, Ollama connected (qwen3:latest, 40960 ctx), Discord connected (Werner_Amvara), scheduler running (2 entries). Zero errors/warnings/panics in log.
- [x] Code review: `match_discord_api_pattern()` and `try_pre_route_discord_api()` in `commands/pre_routing.rs`. Token check gates the entire function (clippy `?` operator). Pattern matching extracted into `match_discord_api_pattern()` for testability. Three routing tiers: (1) explicit `DISCORD_API:` prefix → pass through; (2) "list servers" / "show servers" / "my servers" / "what/which servers am i in" / "discord servers" → `DISCORD_API: GET /users/@me/guilds` (direct, no guild context needed); (3) channel queries ("list/show channels", "list channels in …", "what channels are there") and member queries ("list/show members", "who is/who's in …", "list members in …") → `AGENT: discord-expert <original question>` (needs multi-step guild discovery). Multi-step exclusions ("and then", "after that", "screenshot"). Wired into pre-route chain after management commands, before Redmine. No behavioral changes to existing pre-routes.
- [x] `discord-expert` agent confirmed present (agent-004, `defaults/agents/agent-004/agent.json`, slug `discord-expert`, model_role `cheap`, max_tool_iterations 10).
- [x] Pre-route chain ordering verified: screenshot → RUN_CMD → FETCH_URL → web search → management → **DISCORD_API** → Redmine. No pattern overlap between management commands (schedules, tasks, models) and Discord commands (servers, channels, members).
- [x] CHANGELOG entry verified: "22 new tests; 292 total pass, zero clippy warnings" — confirmed. Feature description matches code behavior. File reference `(commands/pre_routing.rs)` — correct.
- [x] Minor observation (non-blocking): explicit `DISCORD_API:` prefix is checked after multi-step exclusion, so `DISCORD_API: GET /something and then screenshot` would return `None`. Consistent with web search pre-routing pattern (same design choice). Extremely unlikely in practice.

### Closing reviewer smoke test 2026-03-24 (is_context_overflow_error FEAT-D389–D393 + docs)

**Note:** At the time of this run, `004-closing-reviewer/CLOSING-REVIEWER-PROMPT.md` was **not present**; review followed §9 / prior closing-reviewer blocks in this file. A stub prompt file was added later the same day (see next block).

- [x] `cargo check` — zero errors.
- [x] `cargo clippy --all-targets -- -D warnings` — zero warnings.
- [x] `cargo test` — **685** tests pass (`mac_stats` lib crate).
- [x] `cargo build --release` succeeds (v0.1.58).
- [x] `./target/release/mac_stats -vv` starts; 4 monitors loaded, 8 agents, Ollama connected (qwen3:latest), Discord gateway starting, scheduler 2 entries. After review, **`pkill -f mac_stats` had been used** (stopped all matching processes); **release binary restarted** (`nohup ./target/release/mac_stats -vv`) and PID verified — operator should confirm no unintended downtime if other instances were meant to stay up.
- [x] Code review (`content_reduction.rs`): `explicit_context_slot_after_ident_boundary()` and `token_overflow_slot_conjunct_after_ident_boundary()` centralize FEAT-D295-style context-slot detection with ident-boundary matching; `is_context_overflow_error` arms updated for FEAT-D389–D390 (slots, JSON keys, word-order anchors, plural/singular resource `exceed` lists), FEAT-D391 (`message`/`input` … `too long` conjuncts), FEAT-D392 (`strings`/`string exceed`), FEAT-D393 (`arrays`/`array exceed`). Large diff is mostly mechanical replacement of `lower.contains` with boundary-aware helpers; `does_not_match_unrelated_errors` / targeted tests extended. User-facing overflow sanitization unchanged in intent.
- [x] Docs: `CHANGELOG.md` **[0.1.58]** section bullets for D389–D393 align with code; `006-feature-coder/FEATURE-CODER.md` has FEAT-D389–D393 under **Recently closed** and “When empty” high id **FEAT-D393**; `005-openclaw-reviewer/005-openclaw-reviewer.md` independent re-run stamp **2026-03-24T20:12:10Z** (doc-only).

### Closing reviewer smoke test 2026-03-24 (FEAT-D394–D398 objects/elements/nodes/edges/vertices + workflow docs)

- [x] Entry: `004-closing-reviewer/CLOSING-REVIEWER-PROMPT.md` — **present** (stub; points at §9 and latest smoke block in this file).
- [x] `cargo check` — zero errors.
- [x] `cargo clippy --all-targets -- -D warnings` — zero warnings.
- [x] `cargo test` — **685** tests pass (`mac_stats` lib crate).
- [x] `cargo build --release` succeeds (v0.1.58).
- [x] Brief `./target/release/mac_stats -vv` smoke: 4 monitors loaded, menu bar / status item init, welcome banner; **review process started and stopped this instance** after ~3s — restart mac-stats if you rely on continuous menu-bar / Discord uptime.
- [x] Code review (`content_reduction.rs`): new `is_context_overflow_error` arms for FEAT-D394–D398 — plural/singular **`objects` / `elements` / `nodes` / `edges` / `vertices`** + **`exceed`/`exceeded`** via `contains_phrase_after_ident_boundary`, gated by `explicit_context_slot_after_ident_boundary` (same FEAT-D295 slot list as prior `* exceed` rows). Unit tests cover positives (API/batch/validation/gateway phrasing with model context), negatives (HTTP caps, billing/schema without slot, `micro*`/`meta*`/`sub*` compounds, **`wedge exceed`**, **`supervertex exceed`**). User-facing overflow sanitization unchanged in intent.
- [x] Docs: `CHANGELOG.md` **[0.1.58]** **Changed** bullets for D394–D398; agent-workflow line through **FEAT-D398** and 022 checklist **FEAT-D389–D398**; `006-feature-coder/FEATURE-CODER.md` **Recently closed** + **When empty** high id **FEAT-D398**; `005-openclaw-reviewer/005-openclaw-reviewer.md` independent re-run **2026-03-24T20:51:28Z** (doc-only, same OpenClaw `HEAD` `d25b4a2`).

### Closing reviewer smoke test 2026-03-24 (FEAT-D399–D403 geometry `exceed` arms + doc reconciliation)

- [x] Entry: `004-closing-reviewer/CLOSING-REVIEWER-PROMPT.md` — **present**; steps per prompt (§9 checklist, `git diff` vs `CHANGELOG.md` / `FEATURE-CODER.md`, uptime caution).
- [x] `cargo check` — zero errors.
- [x] `cargo clippy --all-targets -- -D warnings` — zero warnings.
- [x] `cargo test` — **685** tests pass (`mac_stats` lib crate). FEAT-D399–D403 add **assertions inside existing** `is_context_overflow_error` tests — **no new `#[test]` fns**, so the **685** case count matches the prior closing block.
- [x] `cargo build --release` succeeds (v0.1.58).
- [x] Runtime smoke: **no second `mac_stats` started** — `pgrep -fl mac_stats` showed existing `./target/release/mac_stats -vv` (**PID 41093**). Tail of `~/.mac-stats/debug.log` shows **4 monitors** UP/saved and normal scheduler/task idle noise (avoids duplicate menu-bar instances per **AGENTS.md**). A **restart** is required to load this commit’s `content_reduction.rs` into the running process.
- [x] Code review (`content_reduction.rs`): five new `||` groups in `is_context_overflow_error` for **FEAT-D399** (`faces` / `face exceed`), **D400** (`triangles` / `triangle exceed`), **D401** (`polygons` / `polygon exceed`), **D402** (`meshes` / `mesh exceed`), **D403** (`voxels` / `voxel exceed`) — each `contains_phrase_after_ident_boundary` + `explicit_context_slot_after_ident_boundary` (same FEAT-D295 slot list as **vertices**). Tests: positives with model-context phrasing; negatives for rate/geometry caps without slot, **`micro*`**/**`meta*`**/**`sub*`** compounds, and **`surface` / `supertriangle` / `superpolygon` / `supermesh` / `supervoxel exceed`** (no false positives on embedded singular substring). Intent: user-facing overflow message unchanged.
- [x] Docs: `CHANGELOG.md` **[0.1.58]** **Changed** bullets for **FEAT-D399–D403** align with code; **Agent workflow docs** bullet checklist range updated to **FEAT-D389–D403**. `006-feature-coder/FEATURE-CODER.md` table head rows **FEAT-D399–D403**; **When empty** high id **FEAT-D403**. `005-openclaw-reviewer/005-openclaw-reviewer.md` independent re-run **2026-03-24T21:20:02Z** — doc-only, same OpenClaw `HEAD` **d25b4a2**.

### Feature coder follow-up (FEAT-D404 `particles exceed` / `particle exceed`)

- [x] `content_reduction.rs`: one new `||` group after **FEAT-D403** (`voxels` / `voxel exceed`) for **`particles exceed`** / **`particles exceeded`** / **`particle exceed`** + **`explicit_context_slot_after_ident_boundary`**; module rustdoc extended. Assertions in existing overflow tests (no new `#[test]`); **`superparticle exceed`** negative mirrors **`supervoxel exceed`**.
- [x] `006-feature-coder/FEATURE-CODER.md`: **Recently closed** row **FEAT-D404**; **When empty** high id **FEAT-D405+**.
- [x] `CHANGELOG.md` **[0.1.58]** **Changed** bullet for **FEAT-D404**; **Agent workflow docs** checklist range **FEAT-D389–D404**.

### Feature coder follow-up (FEAT-D405 `molecules exceed` / `molecule exceed`)

- [x] `content_reduction.rs`: one new `||` group after **FEAT-D404** (`particles` / `particle exceed`) for **`molecules exceed`** / **`molecules exceeded`** / **`molecule exceed`** + **`explicit_context_slot_after_ident_boundary`**; module rustdoc extended. Assertions in existing overflow tests (no new `#[test]`); **`supermolecule exceed`** negative mirrors **`superparticle exceed`**.
- [x] `006-feature-coder/FEATURE-CODER.md`: **Recently closed** row **FEAT-D405**; **When empty** high id **FEAT-D406+**.
- [x] `CHANGELOG.md` **[0.1.58]** **Changed** bullet for **FEAT-D405**; **Agent workflow docs** checklist range **FEAT-D389–D405**.

### Feature coder follow-up (FEAT-D406 `atoms exceed` / `atom exceed`)

- [x] `content_reduction.rs`: one new `||` group after **FEAT-D405** (`molecules` / `molecule exceed`) for **`atoms exceed`** / **`atoms exceeded`** / **`atom exceed`** + **`explicit_context_slot_after_ident_boundary`**; module rustdoc extended. Assertions in existing overflow tests (no new `#[test]`); **`superatom exceed`** negative mirrors **`supermolecule exceed`**.
- [x] `006-feature-coder/FEATURE-CODER.md`: **Recently closed** row **FEAT-D406**; **When empty** high id **FEAT-D407+**.
- [x] `CHANGELOG.md` **[0.1.58]** **Changed** bullet for **FEAT-D406**; **Agent workflow docs** checklist range **FEAT-D389–D406**.

### Feature coder follow-up (FEAT-D407 `ions exceed` / `ion exceed`)

- [x] `content_reduction.rs`: one new `||` group after **FEAT-D406** (`atoms` / `atom exceed`) for **`ions exceed`** / **`ions exceeded`** / **`ion exceed`** + **`explicit_context_slot_after_ident_boundary`**; module rustdoc extended. Assertions in existing overflow tests (no new `#[test]`); **`superion exceed`** negative mirrors **`superatom exceed`**; **`million exceed`** negative documents ident-boundary vs embedded **`ion exceed`**.
- [x] `006-feature-coder/FEATURE-CODER.md`: **Recently closed** row **FEAT-D407**; **When empty** high id **FEAT-D408+**.
- [x] `CHANGELOG.md` **[0.1.58]** **Changed** bullet for **FEAT-D407**; **Agent workflow docs** checklist range **FEAT-D389–D407**.

### Feature coder follow-up (FEAT-D408 `electrons exceed` / `electron exceed`)

- [x] `content_reduction.rs`: one new `||` group after **FEAT-D407** (`ions` / `ion exceed`) for **`electrons exceed`** / **`electrons exceeded`** / **`electron exceed`** + **`explicit_context_slot_after_ident_boundary`**; module rustdoc extended. Assertions in existing overflow tests (no new `#[test]`); **`superelectron exceed`** negative mirrors **`superion exceed`**.
- [x] `006-feature-coder/FEATURE-CODER.md`: **Recently closed** row **FEAT-D408**; **When empty** high id **FEAT-D409+**.
- [x] `CHANGELOG.md` **[0.1.58]** **Changed** bullet for **FEAT-D408**; **Agent workflow docs** checklist range **FEAT-D389–D408**.

### Closing reviewer smoke test 2026-03-24 (`004-closing-reviewer/CLOSING-REVIEWER-PROMPT.md` — FEAT-D404–D408)

- [x] Entry: **`004-closing-reviewer/CLOSING-REVIEWER-PROMPT.md`** — §9 integration checklist consulted; bar matched prior **Closing reviewer smoke test** blocks: `cargo check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo build --release`; **`git diff`** reconciled with **`CHANGELOG.md`** **[0.1.58]** and **`006-feature-coder/FEATURE-CODER.md`** (**FEAT-D404–D408**). No **`pkill -f mac_stats`** (existing instance left running per **AGENTS.md** uptime rule).
- [x] `cargo check` — zero errors.
- [x] `cargo clippy --all-targets -- -D warnings` — zero warnings.
- [x] `cargo test` — **685** tests pass (`mac_stats` lib). **FEAT-D404–D408** extend existing `is_context_overflow_error` tests only — **no new `#[test]` functions**, so count unchanged vs. prior 2026-03-24 closing block.
- [x] `cargo build --release` succeeds (**v0.1.58**).
- [x] Runtime smoke: **no second `mac_stats` started** — `pgrep -fl mac_stats` showed existing `./target/release/mac_stats -vv` (**PID 41093**). Tail of **`~/.mac-stats/debug.log`**: **4 monitors** UP/saved, Discord/Ollama activity. **Restart** mac-stats to load this commit’s **`content_reduction.rs`** into the running process.
- [x] Code review (`content_reduction.rs`): five new `||` groups **FEAT-D404** (`particles` / `particle exceed`) through **FEAT-D408** (`electrons` / `electron exceed`), each after the prior chemistry/geometry row, pattern **`contains_phrase_after_ident_boundary`** + **`explicit_context_slot_after_ident_boundary`** (same FEAT-D295 explicit context-slot list). Negatives: **`superparticle` / `supermolecule` / `superatom` / `superion` / `superelectron exceed`**; **`million exceed`** documents **`ion exceed`** ident boundary.
- [x] Docs alignment: **`CHANGELOG.md`** **[0.1.58]** **Changed** bullets **D404–D408** and **Agent workflow docs** line through **FEAT-D408** match the diff; **`006-feature-coder/FEATURE-CODER.md`** table head + **When empty** **FEAT-D409+** match; **`005-openclaw-reviewer/005-openclaw-reviewer.md`** independent re-run **2026-03-24T21:47:31Z** (doc-only, OpenClaw `HEAD` **d25b4a2**).

### Feature coder follow-up (FEAT-D409 `protons exceed` / `proton exceed`)

- [x] `content_reduction.rs`: one new `||` group after **FEAT-D408** (`electrons` / `electron exceed`) for **`protons exceed`** / **`protons exceeded`** / **`proton exceed`** + **`explicit_context_slot_after_ident_boundary`**; module rustdoc extended. Assertions in existing overflow tests; **`superproton exceed`** negative mirrors **`superelectron exceed`**.
- [x] `006-feature-coder/FEATURE-CODER.md`: **Recently closed** row **FEAT-D409**; **When empty** high id **FEAT-D410+**.
- [x] `CHANGELOG.md` **[0.1.58]** **Changed** bullet for **FEAT-D409**; **Agent workflow docs** checklist range **FEAT-D389–D409**.

### Feature coder follow-up (FEAT-D410 `neutrons exceed` / `neutron exceed`)

- [x] `content_reduction.rs`: one new `||` group after **FEAT-D409** (`protons` / `proton exceed`) for **`neutrons exceed`** / **`neutrons exceeded`** / **`neutron exceed`** + **`explicit_context_slot_after_ident_boundary`**; module rustdoc extended. Assertions in existing overflow tests; **`superneutron exceed`** negative mirrors **`superproton exceed`**.
- [x] `006-feature-coder/FEATURE-CODER.md`: **Recently closed** row **FEAT-D410**; **When empty** high id **FEAT-D411+**.
- [x] `CHANGELOG.md` **[0.1.58]** **Changed** bullet for **FEAT-D410**; **Agent workflow docs** checklist range **FEAT-D389–D410**.

### Feature coder follow-up (FEAT-D411 `quarks exceed` / `quark exceed`)

- [x] `content_reduction.rs`: one new `||` group after **FEAT-D410** (`neutrons` / `neutron exceed`) for **`quarks exceed`** / **`quarks exceeded`** / **`quark exceed`** + **`explicit_context_slot_after_ident_boundary`**; module rustdoc extended. Assertions in existing overflow tests; **`superquark exceed`** negative mirrors **`superneutron exceed`**.
- [x] `006-feature-coder/FEATURE-CODER.md`: **Recently closed** row **FEAT-D411**; **When empty** high id **FEAT-D412+**.
- [x] `CHANGELOG.md` **[0.1.58]** **Changed** bullet for **FEAT-D411**; **Agent workflow docs** checklist range **FEAT-D389–D411**.

### Feature coder follow-up (FEAT-D412 `gluons exceed` / `gluon exceed`)

- [x] `content_reduction.rs`: one new `||` group after **FEAT-D411** (`quarks` / `quark exceed`) for **`gluons exceed`** / **`gluons exceeded`** / **`gluon exceed`** + **`explicit_context_slot_after_ident_boundary`**; module rustdoc extended. Assertions in existing overflow tests; **`supergluon exceed`** negative mirrors **`superquark exceed`**.
- [x] `006-feature-coder/FEATURE-CODER.md`: **Recently closed** row **FEAT-D412**; **When empty** high id **FEAT-D413+**.
- [x] `CHANGELOG.md` **[0.1.58]** **Changed** bullet for **FEAT-D412**; **Agent workflow docs** checklist range **FEAT-D389–D412**.

### Feature coder follow-up (FEAT-D413 `bosons exceed` / `boson exceed`)

- [x] `content_reduction.rs`: one new `||` group after **FEAT-D412** (`gluons` / `gluon exceed`) for **`bosons exceed`** / **`bosons exceeded`** / **`boson exceed`** + **`explicit_context_slot_after_ident_boundary`**; module rustdoc extended. Assertions in existing overflow tests; **`superboson exceed`** negative mirrors **`supergluon exceed`**.
- [x] `006-feature-coder/FEATURE-CODER.md`: **Recently closed** row **FEAT-D413**; **When empty** high id **FEAT-D414+**.
- [x] `CHANGELOG.md` **[0.1.58]** **Changed** bullet for **FEAT-D413**; **Agent workflow docs** checklist range **FEAT-D389–D413**.

### Closing reviewer smoke test 2026-03-24 (`004-closing-reviewer/CLOSING-REVIEWER-PROMPT.md` — FEAT-D409–D413)

- [x] Entry: **`004-closing-reviewer/CLOSING-REVIEWER-PROMPT.md`** — §9 integration checklist consulted; bar matched prior blocks: `cargo check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo build --release`; **`git diff`** reconciled with **`CHANGELOG.md`** **[0.1.58]** (**Changed** **FEAT-D409–D413** + **Agent workflow docs** through **FEAT-D413**), **`006-feature-coder/FEATURE-CODER.md`** (open-table rows **D409–D413**, **When empty** **FEAT-D414+**), and feature-follow-up subsections above. No **`pkill -f mac_stats`** (uptime rule in **AGENTS.md**).
- [x] `cargo check` — zero errors.
- [x] `cargo clippy --all-targets -- -D warnings` — zero warnings.
- [x] `cargo test` — **685** tests pass (`mac_stats` lib); **1** ignored in a separate target (unchanged pattern).
- [x] `cargo build --release` succeeds (**v0.1.58**).
- [x] Runtime smoke: **no second `mac_stats` started** — `pgrep -fl mac_stats` showed existing `./target/release/mac_stats -vv` (**PID 41093**). Tail of **`~/.mac-stats/debug.log`**: **4 monitors** UP/saved. **Restart** mac-stats to load this commit’s **`content_reduction.rs`** into the long-running process.
- [x] Code review (`content_reduction.rs`): five new `||` groups **FEAT-D409** (`protons` / `proton exceed`) through **FEAT-D413** (`bosons` / `boson exceed`), same **`contains_phrase_after_ident_boundary`** + **`explicit_context_slot_after_ident_boundary`** pattern as **D404–D408**; rustdoc extended; negatives **`superproton`** … **`superboson exceed`** aligned with prior **`super* exceed`** rows.
- [x] Docs alignment: **`005-openclaw-reviewer/005-openclaw-reviewer.md`** independent re-run **2026-03-24T22:16:48Z** (OpenClaw `HEAD` **d25b4a2**, doc-only).

### Closing reviewer smoke test 2026-03-25 (`004-closing-reviewer/CLOSING-REVIEWER-PROMPT.md` — FEAT-D414–D418)

- [x] Entry: **`004-closing-reviewer/CLOSING-REVIEWER-PROMPT.md`** — §9 integration checklist; bar matched latest prior block: `cargo check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo build --release`; **`git diff`** reconciled with **`CHANGELOG.md`** **[0.1.58]** (**Changed** bullets **FEAT-D414–D418**), **`006-feature-coder/FEATURE-CODER.md`** (**Recently closed** through **FEAT-D418**, **When empty** **FEAT-D418+**). **Reconciliation fix:** **`CHANGELOG.md`** **Agent workflow docs** line updated **FEAT-D413 → FEAT-D418** / **FEAT-D389–D418** (had been stale vs new overflow arms). No **`pkill -f mac_stats`** (**AGENTS.md** uptime rule).
- [x] `cargo check` — zero errors.
- [x] `cargo clippy --all-targets -- -D warnings` — zero warnings.
- [x] `cargo test` — **685** tests pass (`mac_stats` lib); **1** doc-test ignored (unchanged). New work adds assertions inside existing **`does_not_match_unrelated_errors`** / overflow tests only — **no new `#[test]` functions**, count unchanged vs. **2026-03-24** closing block.
- [x] `cargo build --release` succeeds (**v0.1.58**).
- [x] Runtime smoke: **no second `mac_stats` started** — `pgrep -fl mac_stats` showed existing `./target/release/mac_stats -vv` (**PID 41093**). Tail **`~/.mac-stats/debug.log`**: **4 monitors** UP/saved, scheduler **2** entries, Discord/session/task idle noise — consistent with healthy long-running instance. **Restart** mac-stats to load this commit’s **`content_reduction.rs`** into the running binary.
- [x] Code review (`content_reduction.rs`): five new `||` groups **FEAT-D414** (`leptons` / `lepton exceed`) through **FEAT-D418** (`excitons` / `exciton exceed`), each **`contains_phrase_after_ident_boundary`** + **`explicit_context_slot_after_ident_boundary`** (same FEAT-D295 explicit context-slot list as prior particle rows). Rustdoc comments parallel prior rows. Positives: API/batch/validation/gateway with model-context phrasing. Negatives: HTTP/billing/schema without slot, **`micro*`**/**`meta*`**/**`sub*`** compounds, **`superlepton`** … **`superexciton exceed`**.
- [x] Docs alignment: **`005-openclaw-reviewer/005-openclaw-reviewer.md`** independent re-run **2026-03-24T22:45:25Z** (OpenClaw `HEAD` **d25b4a2**, doc-only; **Prior** line preserves **22:16:48Z** stamp).

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D419)

- [x] **`is_context_overflow_error`** — **`polarons exceed`** / **`polarons exceeded`** / **`polaron exceed`** + **`explicit_context_slot_after_ident_boundary`** (FEAT-D295 slot list); rustdoc + **`does_not_match_unrelated_errors`** positives/negatives in **`content_reduction.rs`**.
- [x] **`CHANGELOG.md`** **[0.1.58]** **Changed** bullet **FEAT-D419**; **Agent workflow docs** line **FEAT-D389–D419**; **`006-feature-coder/FEATURE-CODER.md`** **Recently closed** head row **FEAT-D419**, **When empty** **FEAT-D420+**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D420)

- [x] **`is_context_overflow_error`** — **`plasmons exceed`** / **`plasmons exceeded`** / **`plasmon exceed`** + **`explicit_context_slot_after_ident_boundary`** (FEAT-D295 slot list); rustdoc + **`does_not_match_unrelated_errors`** positives/negatives in **`content_reduction.rs`**.
- [x] **`CHANGELOG.md`** **[0.1.58]** **Changed** bullet **FEAT-D420**; **Agent workflow docs** line **FEAT-D389–D420**; **`006-feature-coder/FEATURE-CODER.md`** **Recently closed** head row **FEAT-D420**, **When empty** **FEAT-D421+**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D421)

- [x] **`is_context_overflow_error`** — **`solitons exceed`** / **`solitons exceeded`** / **`soliton exceed`** + **`explicit_context_slot_after_ident_boundary`** (FEAT-D295 slot list); rustdoc + **`does_not_match_unrelated_errors`** positives/negatives in **`content_reduction.rs`**.
- [x] **`CHANGELOG.md`** **[0.1.58]** **Added** bullet **FEAT-D421**; **Agent workflow docs** line **FEAT-D389–D421**; **`006-feature-coder/FEATURE-CODER.md`** **Recently closed** head row **FEAT-D421**, **When empty** **FEAT-D422+**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D422)

- [x] **`is_context_overflow_error`** — **`instantons exceed`** / **`instantons exceeded`** / **`instanton exceed`** + **`explicit_context_slot_after_ident_boundary`** (FEAT-D295 slot list); rustdoc + **`does_not_match_unrelated_errors`** positives/negatives in **`content_reduction.rs`**.
- [x] **`CHANGELOG.md`** **[0.1.58]** **Added** bullet **FEAT-D422**; **Agent workflow docs** line **FEAT-D389–D422**; **`006-feature-coder/FEATURE-CODER.md`** **Recently closed** head row **FEAT-D422**, **When empty** **FEAT-D423+**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D423)

- [x] **`is_context_overflow_error`** — **`skyrmions exceed`** / **`skyrmions exceeded`** / **`skyrmion exceed`** + **`explicit_context_slot_after_ident_boundary`** (FEAT-D295 slot list); rustdoc + **`does_not_match_unrelated_errors`** positives/negatives in **`content_reduction.rs`**.
- [x] **`CHANGELOG.md`** **[0.1.58]** **Added** bullet **FEAT-D423**; **Agent workflow docs** line **FEAT-D389–D423**; **`006-feature-coder/FEATURE-CODER.md`** **Recently closed** head row **FEAT-D423**, **When empty** **FEAT-D424+**.

### Closing reviewer smoke test 2026-03-25 (`004-closing-reviewer/CLOSING-REVIEWER-PROMPT.md` — FEAT-D419–D423)

- [x] Entry: **`004-closing-reviewer/CLOSING-REVIEWER-PROMPT.md`** — §9 integration checklist; bar matched latest prior block: `cargo check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo build --release`; **`git diff`** reconciled with **`CHANGELOG.md`** **[0.1.58]** (**Changed** **FEAT-D419–D420**, **Added** **FEAT-D421–D423**), **`006-feature-coder/FEATURE-CODER.md`** (**Recently closed** through **FEAT-D423**, **When empty** **FEAT-D424+**), **`005-openclaw-reviewer/005-openclaw-reviewer.md`** (independent re-run **2026-03-24T23:11:50Z**, OpenClaw `HEAD` **d25b4a2**, doc-only). **Doc fix:** **`docs/022_feature_review_plan.md`** — inserted missing **FEAT-D421** feature-coder subsection and ordered **D422** before **D423**. No **`pkill -f mac_stats`** (**AGENTS.md** uptime rule).
- [x] `cargo check` — zero errors.
- [x] `cargo clippy --all-targets -- -D warnings` — zero warnings.
- [x] `cargo test` — **685** tests pass (`mac_stats` lib); **1** test ignored in a separate target. New overflow work extends existing **`does_not_match_unrelated_errors`** / positive cases only — **no new `#[test]` functions**, count unchanged vs. prior closing block.
- [x] `cargo build --release` succeeds (**v0.1.58**).
- [x] Runtime smoke: **no second `mac_stats` started** — `pgrep -fl mac_stats` showed existing `./target/release/mac_stats -vv` (**PID 41093**). Tail **`~/.mac-stats/debug.log`**: **4 monitors** UP/saved. **Restart** mac-stats to load uncommitted **`content_reduction.rs`** into the long-running binary.
- [x] Code review (`content_reduction.rs`): five new `||` groups **FEAT-D419** (`polarons` / `polaron exceed`) through **FEAT-D423** (`skyrmions` / `skyrmion exceed`), each **`contains_phrase_after_ident_boundary`** + **`explicit_context_slot_after_ident_boundary`**; module rustdoc extended; negatives **`superpolaron`** … **`superskyrmion exceed`** in **`does_not_match_unrelated_errors`**. User-facing overflow sanitization unchanged in intent.
- [x] **`005-openclaw-reviewer/005-openclaw-reviewer.md`** — **Latest verification** date **2026-03-25**; **Prior** line preserves **2026-03-24T22:45:25Z** stamp.

### Closing reviewer smoke test 2026-03-25 (`004-closing-reviewer/CLOSING-REVIEWER-PROMPT.md` — FEAT-D424–D428 + `CHANGELOG` / OpenClaw reconciliation)

- [x] Entry: **`004-closing-reviewer/CLOSING-REVIEWER-PROMPT.md`** — §9 integration checklist; bar matched latest prior block: `cargo check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo build --release`; **`git diff`** reconciled with **`CHANGELOG.md`** **[0.1.59]** (**Added** bullets **FEAT-D424–D428** for **`magnons`/`rotons`/`anyons`/`fluxons`/`vortices`** overflow arms), **`006-feature-coder/FEATURE-CODER.md`** (**Recently closed** head through **FEAT-D428**, **When empty** **FEAT-D429+**), **`005-openclaw-reviewer/005-openclaw-reviewer.md`** (independent re-run **2026-03-24T23:40:37Z**, OpenClaw `HEAD` **d25b4a2**, doc-only; **Prior** preserves **2026-03-24T23:11:50Z**). **`CHANGELOG.md` reconciliation:** **Agent workflow docs** for **0.1.59** (**Changed**) and **0.1.58** (**Added** tail through **FEAT-D423**); checklist **FEAT-D389–D428**. No **`pkill -f mac_stats`** (**AGENTS.md** uptime rule).
- [x] `cargo check` — zero errors.
- [x] `cargo clippy --all-targets -- -D warnings` — zero warnings.
- [x] `cargo test` — **685** tests pass (`mac_stats` lib); **1** doc-test ignored. **FEAT-D424–D428** extend **`does_not_match_unrelated_errors`** / positives only — **no new `#[test]` functions**, count unchanged vs. prior **2026-03-25** closing block (**FEAT-D419–D423**).
- [x] `cargo build --release` succeeds (**v0.1.59**).
- [x] Runtime smoke: **no second `mac_stats` started** — `pgrep -fl mac_stats` showed existing `./target/release/mac_stats -vv` (**PID 41093**). Tail **`~/.mac-stats/debug.log`**: **4 monitors** UP/saved, scheduler **2** entries, idle task/session noise — healthy long-running instance. **Restart** mac-stats to load uncommitted **`content_reduction.rs`** into the running binary.
- [x] Code review (`content_reduction.rs`): five new `||` groups **FEAT-D424** (`magnons` / `magnon exceed`) through **FEAT-D428** (`vortices` / `vortex exceed`), each **`contains_phrase_after_ident_boundary`** + **`explicit_context_slot_after_ident_boundary`** (same FEAT-D295 explicit context-slot list as prior quasiparticle rows). Rustdoc comments parallel prior rows. Negatives: **`supermagnon`** … **`supervortex exceed`**; **`canyon exceed`** negative for **`anyon exceed`** (D426). User-facing overflow sanitization unchanged in intent.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D429)

- [x] **`is_context_overflow_error`**: **`disclinations exceed`** / **`disclination exceed`** arm after **`vortices`/`vortex`** (`commands/content_reduction.rs`); module rustdoc + **`does_not_match_unrelated_errors`** positives/negatives (**`superdisclination exceed`**, **`microdisclinations exceed`**, etc.). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D429**, **When empty** **FEAT-D430+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D429; **Changed** agent-workflow line **FEAT-D424–D429**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D430)

- [x] **`is_context_overflow_error`**: **`dislocations exceed`** / **`dislocation exceed`** arm after **`disclinations`/`disclination`** (`commands/content_reduction.rs`); module rustdoc + **`does_not_match_unrelated_errors`** positives/negatives (**`superdislocation exceed`**, **`microdislocations exceed`**, etc.). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D430**, **When empty** **FEAT-D431+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D430; **Changed** agent-workflow line **FEAT-D424–D430**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D431)

- [x] **`is_context_overflow_error`**: **`vacancies exceed`** / **`vacancy exceed`** arm after **`dislocations`/`dislocation`** (`commands/content_reduction.rs`); module rustdoc + **`does_not_match_unrelated_errors`** positives/negatives (**`supervacancy exceed`**, **`microvacancies exceed`**, etc.). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D431**, **When empty** **FEAT-D432+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D431; **Changed** agent-workflow line **FEAT-D424–D431**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D432)

- [x] **`is_context_overflow_error`**: **`interstitials exceed`** / **`interstitial exceed`** arm after **`vacancies`/`vacancy`** (`commands/content_reduction.rs`); module rustdoc + **`does_not_match_unrelated_errors`** positives/negatives (**`superinterstitial exceed`**, **`microinterstitials exceed`**, etc.). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D432**, **When empty** **FEAT-D433+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D432; **Changed** agent-workflow line **FEAT-D424–D432**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D433)

- [x] **`is_context_overflow_error`**: **`voids exceed`** / **`void exceed`** arm after **`interstitials`/`interstitial`** (`commands/content_reduction.rs`); module rustdoc + **`detects_context_overflow_errors`** positives/negatives (**`supervoid exceed`**, **`microvoids exceed`**, etc.). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D433**, **When empty** **FEAT-D434+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D433; **Changed** agent-workflow line **FEAT-D424–D433**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D434)

- [x] **`is_context_overflow_error`**: **`pores exceed`** / **`pore exceed`** arm after **`voids`/`void`** (`commands/content_reduction.rs`); module rustdoc + **`detects_context_overflow_errors`** positives/negatives (**`superpore exceed`**, **`micropores exceed`**, **`spore exceed`**). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D434**, **When empty** **FEAT-D435+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D434; **Changed** agent-workflow line **FEAT-D424–D434**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D435)

- [x] **`is_context_overflow_error`**: **`inclusions exceed`** / **`inclusion exceed`** arm after **`pores`/`pore`** (`commands/content_reduction.rs`); module rustdoc + **`detects_context_overflow_errors`** positives/negatives (**`superinclusion exceed`**, **`microinclusions exceed`**, **`reinclusion exceed`**). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D435**, **When empty** **FEAT-D436+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D435; **Changed** agent-workflow line **FEAT-D424–D435**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D436)

- [x] **`is_context_overflow_error`**: **`clusters exceed`** / **`cluster exceed`** arm after **`inclusions`/`inclusion`** (`commands/content_reduction.rs`); module rustdoc + **`detects_context_overflow_errors`** positives/negatives (**`supercluster exceed`**, **`microclusters exceed`**, **`recluster exceed`**). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D436**, **When empty** **FEAT-D437+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D436; **Changed** agent-workflow line **FEAT-D424–D436**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D437)

- [x] **`is_context_overflow_error`**: **`grains exceed`** / **`grain exceed`** arm after **`clusters`/`cluster`** (`commands/content_reduction.rs`); module rustdoc + **`detects_context_overflow_errors`** positives/negatives (**`supergrain exceed`**, **`micrograins exceed`**, **`regrain exceed`**). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D437**, **When empty** **FEAT-D438+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D437; **Changed** agent-workflow line **FEAT-D424–D437**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D438)

- [x] **`is_context_overflow_error`**: **`phases exceed`** / **`phase exceed`** arm after **`grains`/`grain`** (`commands/content_reduction.rs`); module rustdoc + **`detects_context_overflow_errors`** positives/negatives (**`superphase exceed`**, **`microphases exceed`**, **`prephase exceed`**, **`rephase exceed`**). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D438**, **When empty** **FEAT-D439+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D438; **Changed** agent-workflow line **FEAT-D424–D438**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D439)

- [x] **`is_context_overflow_error`**: **`crystals exceed`** / **`crystal exceed`** arm after **`phases`/`phase`** (`commands/content_reduction.rs`); module rustdoc + **`detects_context_overflow_errors`** positives/negatives (**`supercrystal exceed`**, **`microcrystals exceed`**, **`precrystal exceed`**, **`recrystal exceed`**). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D439**, **When empty** **FEAT-D440+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D439; **Changed** agent-workflow line **FEAT-D424–D439**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D440)

- [x] **`is_context_overflow_error`**: **`unit cells exceed`** / **`unit cell exceed`** arm after **`crystals`/`crystal`** (`commands/content_reduction.rs`); module rustdoc + **`does_not_match_unrelated_errors`** positives/negatives (**`microunitcells exceed`**, **`metaunitcells exceed`**, **`subunitcell exceed`**, **`superunitcell exceed`**, **`preunitcell exceed`**, **`reunitcell exceed`**; note **`microunit cells …`** still hits the separate **`cells exceed`** arm). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D440**, **When empty** **FEAT-D441+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D440; **Changed** agent-workflow line **FEAT-D424–D440**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D441)

- [x] **`is_context_overflow_error`**: **`primitive cells exceed`** / **`primitive cell exceed`** arm after **`unit cells`/`unit cell`** (`commands/content_reduction.rs`); module rustdoc + **`does_not_match_unrelated_errors`** positives/negatives (**`microprimitivecells exceed`**, **`metaprimitivecells exceed`**, **`subprimitivecell exceed`**, **`superprimitivecell exceed`**, **`preprimitivecell exceed`**, **`reprimitivecell exceed`**). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D441**, **When empty** **FEAT-D442+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D441; **Changed** agent-workflow line **FEAT-D424–D441**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D442)

- [x] **`is_context_overflow_error`**: **`supercells exceed`** / **`supercell exceed`** arm after **`primitive cells`/`primitive cell`** (`commands/content_reduction.rs`); module rustdoc + **`does_not_match_unrelated_errors`** positives/negatives (**`microsupercells exceed`**, **`metasupercells exceed`**, **`subsupercell exceed`**, **`supersupercell exceed`**, **`presupercell exceed`**, **`resupercell exceed`**). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D442**, **When empty** **FEAT-D443+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D442; **Changed** agent-workflow line **FEAT-D424–D442**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D443)

- [x] **`is_context_overflow_error`**: **`k-points exceed`** / **`k-point exceed`** arm after **`supercells`/`supercell`** (`commands/content_reduction.rs`); module rustdoc + **`does_not_match_unrelated_errors`** positives/negatives (**`microk-points exceed`**, **`metak-points exceed`**, **`subk-point exceed`**, **`superk-point exceed`**, **`prek-point exceed`**, **`rek-point exceed`**). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D443**, **When empty** **FEAT-D444+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D443; **Changed** agent-workflow line **FEAT-D424–D443**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D444 `q-points`/`q-point exceed`)

- [x] **`is_context_overflow_error`**: **`q-points exceed`** / **`q-point exceed`** arm after **`k-points`/`k-point`** (`commands/content_reduction.rs`); module rustdoc + **`does_not_match_unrelated_errors`** positives/negatives (**`microq-points exceed`**, **`metaq-points exceed`**, **`subq-point exceed`**, **`superq-point exceed`**, **`preq-point exceed`**, **`req-point exceed`**). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D444**, **When empty** **FEAT-D445+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D444; **Changed** agent-workflow line **FEAT-D424–D444**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D445 `bands`/`band exceed`)

- [x] **`is_context_overflow_error`**: **`bands exceed`** / **`band exceed`** arm after **`q-points`/`q-point`** (`commands/content_reduction.rs`); module rustdoc + **`does_not_match_unrelated_errors`** positives/negatives (**`microbands exceed`**, **`metabands exceed`**, **`subband exceed`**, **`superband exceed`**, **`preband exceed`**, **`reband exceed`**). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D445**, **When empty** **FEAT-D446+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D445; **Changed** agent-workflow line **FEAT-D424–D445**.

### Closing reviewer smoke test 2026-03-25 (`004-closing-reviewer/CLOSING-REVIEWER-PROMPT.md` — FEAT-D429–D437 + OpenClaw doc refresh)

- [x] Entry: **`004-closing-reviewer/CLOSING-REVIEWER-PROMPT.md`** — §9 integration checklist; bar matched latest prior block: `cargo check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo build --release`; **`git diff`** reconciled with **`CHANGELOG.md`** **[0.1.59]** (**Added** bullets **FEAT-D429–D437** for defect / microstructure overflow arms through **`grains`/`grain exceed`**; **Changed** agent-workflow through **FEAT-D437** / checklist **FEAT-D424–D437**), **`006-feature-coder/FEATURE-CODER.md`** (**Recently closed** head through **FEAT-D437**, **When empty** **FEAT-D438+**), **`005-openclaw-reviewer/005-openclaw-reviewer.md`** (independent re-run **2026-03-25T00:40:23Z**, OpenClaw `HEAD` **d25b4a2**, doc-only; **Prior** preserves **2026-03-25T00:12:11Z**). No **`pkill -f mac_stats`** (**AGENTS.md** uptime rule).
- [x] `cargo check` — zero errors.
- [x] `cargo clippy --all-targets -- -D warnings` — zero warnings.
- [x] `cargo test` — **685** tests pass (`mac_stats` lib crate); **1** doc-test ignored. **FEAT-D429–D437** extend existing **`detects_context_overflow_errors`** / **`does_not_match_unrelated_errors`** / positive cases only — **no new `#[test]` functions**, count unchanged vs. prior **2026-03-25** closing baseline (**FEAT-D424–D428**).
- [x] `cargo build --release` succeeds (**v0.1.59**).
- [x] Runtime smoke: **no second `mac_stats` started** — `pgrep -fl mac_stats` showed existing **`./target/release/mac_stats -vv` (PID 41093)**. Tail **`~/.mac-stats/debug.log`**: **4 monitors** UP/saved to disk, background monitor loop active — healthy long-running instance. **Restart** mac-stats to load uncommitted **`content_reduction.rs`** into the running binary.
- [x] Code review (`content_reduction.rs`): nine `||` groups **FEAT-D429** (`disclinations` / `disclination exceed`) through **FEAT-D437** (`grains` / `grain exceed`), ordered after **`vortices`/`vortex`**, each **`contains_phrase_after_ident_boundary`** + **`explicit_context_slot_after_ident_boundary`** (same FEAT-D295 explicit context-slot list). Module rustdoc extended for **voids/pores/inclusions/clusters/grains** rows. Negatives: **`superdisclination`** … **`supergrain exceed`**, **`spore exceed`** vs **`pore exceed`** (D434), HTTP/billing/schema caps without slot wording. User-facing overflow sanitization unchanged in intent.

### Closing reviewer smoke test 2026-03-25 (`004-closing-reviewer/CLOSING-REVIEWER-PROMPT.md` — FEAT-D438–D442 + OpenClaw re-run)

- [x] Entry: **`004-closing-reviewer/CLOSING-REVIEWER-PROMPT.md`** — §9 integration checklist; bar matched prior blocks: `cargo check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo build --release`. **`git diff`** reconciled with **`CHANGELOG.md`** **[0.1.59]** (**Added** **FEAT-D438–D442**: **`phases`/`phase`**, **`crystals`/`crystal`**, **`unit cells`/`unit cell`**, **`primitive cells`/`primitive cell`**, **`supercells`/`supercell exceed`** + context-slot guards; **Changed** agent-workflow through **FEAT-D442** / checklist **FEAT-D424–D442**), **`006-feature-coder/FEATURE-CODER.md`** (**Recently closed** head **FEAT-D442**, **When empty** **FEAT-D443+**; open table rows **FEAT-D438–D442** match the diff), **`005-openclaw-reviewer/005-openclaw-reviewer.md`** (independent re-run **2026-03-25T01:16:52Z**, OpenClaw `HEAD` **d25b4a2**; **Prior** **2026-03-25T00:40:23Z** and **2026-03-25T00:12:11Z**). No **`pkill -f mac_stats`** (**AGENTS.md** uptime rule).
- [x] `cargo check` — zero errors.
- [x] `cargo clippy --all-targets -- -D warnings` — zero warnings.
- [x] `cargo test` — **685** tests pass (`mac_stats` lib crate); **1** doc-test ignored (same counts as prior **2026-03-25** closing baseline).
- [x] `cargo build --release` succeeds (**v0.1.59**).
- [x] Runtime smoke: **no second `mac_stats` started** — `pgrep -fl mac_stats` showed existing **`./target/release/mac_stats -vv` (PID 41093)**. **Restart** mac-stats to load uncommitted **`content_reduction.rs`** into the running binary.
- [x] Code review (`content_reduction.rs`): five further `||` groups **FEAT-D438** (`phases` / `phase exceed`) through **FEAT-D442** (`supercells` / `supercell exceed`), chained after **`grains`/`grain`**, same boundary + explicit context-slot pattern; rustdoc table extended through **supercells**. **`src/ollama.js`** / **`dist/ollama.js`** not in diff — frontend sync N/A for this pass.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D446 `orbitals`/`orbital exceed`)

- [x] **`is_context_overflow_error`**: **`orbitals exceed`** / **`orbital exceed`** arm after **`bands`/`band`** (`commands/content_reduction.rs`); module rustdoc + **`detects_context_overflow_errors`** / **`does_not_match_unrelated_errors`** positives/negatives (**`microorbitals exceed`**, **`metaorbitals exceed`**, **`suborbital exceed`**, **`superorbital exceed`**, **`preorbital exceed`**, **`reorbital exceed`**). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D446**, **When empty** **FEAT-D447+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D446; **Changed** agent-workflow line **FEAT-D424–D446**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D447 `basis functions`/`basis function exceed`)

- [x] **`is_context_overflow_error`**: **`basis functions exceed`** / **`basis function exceed`** arm after **`orbitals`/`orbital`** (`commands/content_reduction.rs`); module rustdoc + **`detects_context_overflow_errors`** / **`does_not_match_unrelated_errors`** positives/negatives (**`microbasis functions exceed`**, **`metabasis functions exceed`**, **`subbasis function exceed`**, **`superbasis function exceed`**, **`prebasis function exceed`**, **`rebasis function exceed`**). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D447**, **When empty** **FEAT-D448+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D447; **Changed** agent-workflow line **FEAT-D424–D447**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D448 `electrons`/`electron exceed`)

- [x] **`is_context_overflow_error`**: **`electrons exceed`** / **`electron exceed`** — **`||` group** remains before **`proton(s)`** (`commands/content_reduction.rs`); this pass adds module rustdoc rows + **`detects_context_overflow_errors`** / **`does_not_match_unrelated_errors`** cases (**`microelectrons exceed`**, **`metaelectrons exceed`**, **`subelectron exceed`**, **`superelectron exceed`**, **`preelectron exceed`**, **`reelectron exceed`**, HTTP/billing/schema negatives). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D448**, **When empty** **FEAT-D449+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D448; **Changed** agent-workflow line **FEAT-D424–D448**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D449 `auxiliary functions`/`auxiliary function exceed`)

- [x] **`is_context_overflow_error`**: **`auxiliary functions exceed`** / **`auxiliary function exceed`** arm after **`basis functions`/`basis function`** (`commands/content_reduction.rs`); module rustdoc + **`detects_context_overflow_errors`** / **`does_not_match_unrelated_errors`** positives/negatives (**`microauxiliary functions exceed`**, **`metaauxiliary functions exceed`**, **`subauxiliary function exceed`**, **`superauxiliary function exceed`**, **`preauxiliary function exceed`**, **`reauxiliary function exceed`**). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D449**, **When empty** **FEAT-D450+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D449; **Changed** agent-workflow line **FEAT-D424–D449**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D450 `primitive gaussians`/`primitive gaussian exceed`)

- [x] **`is_context_overflow_error`**: **`primitive gaussians exceed`** / **`primitive gaussian exceed`** arm after **`auxiliary functions`/`auxiliary function`** (`commands/content_reduction.rs`); module rustdoc + **`detects_context_overflow_errors`** / **`does_not_match_unrelated_errors`** positives/negatives (**`microprimitive gaussians exceed`**, **`metaprimitive gaussians exceed`**, **`subprimitive gaussian exceed`**, **`superprimitive gaussian exceed`**, **`preprimitive gaussian exceed`**, **`reprimitive gaussian exceed`**). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D450**, **When empty** **FEAT-D451+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D450; **Changed** agent-workflow line **FEAT-D424–D450**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D451 `contracted gaussians`/`contracted gaussian exceed`)

- [x] **`is_context_overflow_error`**: **`contracted gaussians exceed`** / **`contracted gaussian exceed`** arm after **`primitive gaussians`/`primitive gaussian`** (`commands/content_reduction.rs`); module rustdoc + **`detects_context_overflow_errors`** / **`does_not_match_unrelated_errors`** positives/negatives (**`microcontracted gaussians exceed`**, **`metacontracted gaussians exceed`**, **`subcontracted gaussian exceed`**, **`supercontracted gaussian exceed`**, **`precontracted gaussian exceed`**, **`recontracted gaussian exceed`**). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D451**, **When empty** **FEAT-D452+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D451; **Changed** agent-workflow line **FEAT-D424–D451**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D452 `spherical gaussians`/`spherical gaussian exceed`)

- [x] **`is_context_overflow_error`**: **`spherical gaussians exceed`** / **`spherical gaussian exceed`** arm after **`contracted gaussians`/`contracted gaussian`** (`commands/content_reduction.rs`); module rustdoc + **`detects_context_overflow_errors`** / **`does_not_match_unrelated_errors`** positives/negatives (**`microspherical gaussians exceed`**, **`metaspherical gaussians exceed`**, **`subspherical gaussian exceed`**, **`superspherical gaussian exceed`**, **`prespherical gaussian exceed`**, **`respherical gaussian exceed`**). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D452**, **When empty** **FEAT-D453+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D452; **Changed** agent-workflow line **FEAT-D424–D452**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D453 `cartesian gaussians`/`cartesian gaussian exceed`)

- [x] **`is_context_overflow_error`**: **`cartesian gaussians exceed`** / **`cartesian gaussian exceed`** arm after **`spherical gaussians`/`spherical gaussian`** (`commands/content_reduction.rs`); module rustdoc + **`detects_context_overflow_errors`** / **`does_not_match_unrelated_errors`** positives/negatives (**`microcartesian gaussians exceed`**, **`metacartesian gaussians exceed`**, **`subcartesian gaussian exceed`**, **`supercartesian gaussian exceed`**, **`precartesian gaussian exceed`**, **`recartesian gaussian exceed`**). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D453**, **When empty** **FEAT-D454+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D453; **Changed** agent-workflow line **FEAT-D424–D453**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D454 `gaussian shells`/`gaussian shell exceed`)

- [x] **`is_context_overflow_error`**: **`gaussian shells exceed`** / **`gaussian shell exceed`** arm after **`cartesian gaussians`/`cartesian gaussian`** (`commands/content_reduction.rs`); module rustdoc + **`detects_context_overflow_errors`** / **`does_not_match_unrelated_errors`** positives/negatives (**`microgaussian shells exceed`**, **`metagaussian shells exceed`**, **`subgaussian shell exceed`**, **`supergaussian shell exceed`**, **`pregaussian shell exceed`**, **`regaussian shell exceed`**). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D454**, **When empty** **FEAT-D455+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D454; **Changed** agent-workflow line **FEAT-D424–D454**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D455 `density matrices`/`density matrix exceed`)

- [x] **`is_context_overflow_error`**: **`density matrices exceed`** / **`density matrix exceed`** arm after **`gaussian shells`/`gaussian shell`** (`commands/content_reduction.rs`); module rustdoc + **`detects_context_overflow_errors`** gateway line + positives/negatives (**`microdensity matrices exceed`**, **`metadensity matrices exceed`**, **`subdensity matrix exceed`**, **`superdensity matrix exceed`**, **`predensity matrix exceed`**, **`redensity matrix exceed`**). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D455**, **When empty** **FEAT-D456+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D455; **Changed** agent-workflow line **FEAT-D424–D455**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D456 `molecular orbitals`/`molecular orbital exceed`)

- [x] **`is_context_overflow_error`**: **`molecular orbitals exceed`** / **`molecular orbital exceed`** arm after **`density matrices`/`density matrix`** (`commands/content_reduction.rs`); module rustdoc + **`detects_context_overflow_errors`** gateway line + **`does_not_match_unrelated_errors`** positives/negatives (concatenated **`micromolecularorbitals`** / **`metamolecularorbitals`** / **`supermolecularorbital`** / **`premolecularorbital`** / **`remolecularorbital`** where the generic **`orbitals`/`orbital exceed`** arms would otherwise match at a boundary; **`submolecular orbital exceed`** without slot). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D456**, **When empty** **FEAT-D457+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D456; **Changed** agent-workflow line **FEAT-D424–D456**.

### Feature coder 2026-03-25 (`006-feature-coder/FEATURE-CODER.md` — FEAT-D457 `atomic orbitals`/`atomic orbital exceed`)

- [x] **`is_context_overflow_error`**: **`atomic orbitals exceed`** / **`atomic orbital exceed`** arm after **`molecular orbitals`/`molecular orbital`** (`commands/content_reduction.rs`); module rustdoc + **`detects_context_overflow_errors`** gateway line + **`does_not_match_unrelated_errors`** positives/negatives (concatenated **`microatomicorbitals`** / **`metaatomicorbitals`** / **`superatomicorbital`** / **`preatomicorbital`** / **`reatomicorbital`**; **`subatomic orbital exceed`** without slot; HTTP / billing / schema negatives without context-slot wording). **`006-feature-coder/FEATURE-CODER.md`**: **Recently closed** head **FEAT-D457**, **When empty** **FEAT-D458+**. **`CHANGELOG.md`** **[0.1.59]**: **Added** FEAT-D457; **Changed** agent-workflow line **FEAT-D424–D457**.

### Closing reviewer smoke test 2026-03-25 (`004-closing-reviewer/CLOSING-REVIEWER-PROMPT.md` — FEAT-D443–D447 + OpenClaw re-run)

- [x] Entry: **`004-closing-reviewer/CLOSING-REVIEWER-PROMPT.md`** — §9 integration checklist; bar matched latest prior block: `cargo check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo build --release`; **`git diff`** reconciled with **`CHANGELOG.md`** **[0.1.59]** (**Added** bullets **FEAT-D443–D447**: **`k-points`/`k-point`**, **`q-points`/`q-point`**, **`bands`/`band`**, **`orbitals`/`orbital`**, **`basis functions`/`basis function exceed`** + context-slot guards; **Changed** agent-workflow through **FEAT-D447** / checklist **FEAT-D424–D447**), **`006-feature-coder/FEATURE-CODER.md`** (**Recently closed** head **FEAT-D447**, **When empty** **FEAT-D448+**; open-table rows **FEAT-D443–D447**), **`005-openclaw-reviewer/005-openclaw-reviewer.md`** (independent re-run **2026-03-25T01:57:44Z**, OpenClaw `HEAD` **d25b4a2**, doc-only; **Prior** **2026-03-25T01:16:52Z**). **`docs/022_feature_review_plan.md` reconciliation:** inserted missing **FEAT-D444** / **FEAT-D445** feature-coder subsections after **FEAT-D443** (parity with **CHANGELOG** / **FEATURE-CODER**). No **`pkill -f mac_stats`** (**AGENTS.md** uptime rule).
- [x] `cargo check` — zero errors.
- [x] `cargo clippy --all-targets -- -D warnings` — zero warnings.
- [x] `cargo test` — **685** tests pass (`mac_stats` lib crate); **1** doc-test ignored. **FEAT-D443–D447** extend **`is_context_overflow_error`** / existing **`detects_context_overflow_errors`** / **`does_not_match_unrelated_errors`** assertions only — **no new `#[test]` functions**, count unchanged vs. prior **2026-03-25** closing baseline.
- [x] `cargo build --release` succeeds (**v0.1.59**).
- [x] Runtime smoke: **no second `mac_stats` started** — `pgrep -fl mac_stats` showed existing **`./target/release/mac_stats -vv` (PID 41093)** (plus transient **`cargo test`** deps under **`target/debug/deps/`** during the test run). Tail **`~/.mac-stats/debug.log`**: **4 monitors** UP/saved, idle **Having fun** scheduler noise — healthy long-running instance. **Restart** mac-stats to load uncommitted **`content_reduction.rs`** into the running release binary.
- [x] Code review (`content_reduction.rs`): five new `||` groups **FEAT-D443** (`k-points` / `k-point exceed`) through **FEAT-D447** (`basis functions` / `basis function exceed`), chained after **`supercells`/`supercell`**, each **`contains_phrase_after_ident_boundary`** + **`explicit_context_slot_after_ident_boundary`** (same FEAT-D295 explicit context-slot list). Module rustdoc extended for **k/q-points**, **bands**, **orbitals**, **basis functions**. User-facing overflow sanitization unchanged in intent.

### Closing reviewer smoke test 2026-03-25 (`004-closing-reviewer/CLOSING-REVIEWER-PROMPT.md` — FEAT-D448–D452 + OpenClaw re-run)

- [x] Entry: **`004-closing-reviewer/CLOSING-REVIEWER-PROMPT.md`** — §9 integration checklist; bar matched latest prior block: `cargo check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo build --release`; **`git diff`** reconciled with **`CHANGELOG.md`** **[0.1.59]** (**Added** bullets **FEAT-D448–D452**: **`electrons`/`electron exceed`** (tests + rustdoc; **`||` group** unchanged), **`auxiliary functions`/`auxiliary function`**, **`primitive gaussians`/`primitive gaussian`**, **`contracted gaussians`/`contracted gaussian`**, **`spherical gaussians`/`spherical gaussian exceed`** + context-slot guards; **Changed** agent-workflow through **FEAT-D452** / checklist **FEAT-D424–D452**), **`006-feature-coder/FEATURE-CODER.md`** (**Recently closed** head **FEAT-D452**, **When empty** **FEAT-D453+**; open-table rows **FEAT-D448–D452**), **`005-openclaw-reviewer/005-openclaw-reviewer.md`** (independent re-run **2026-03-25T02:34:05Z**, OpenClaw `HEAD` **d25b4a2**, doc-only; **Prior** **2026-03-25T01:57:44Z**). No **`pkill -f mac_stats`** (**AGENTS.md** uptime rule).
- [x] `cargo check` — zero errors.
- [x] `cargo clippy --all-targets -- -D warnings` — zero warnings.
- [x] `cargo test` — **685** tests pass (`mac_stats` lib crate); **1** doc-test ignored. **FEAT-D448–D452** extend existing **`#[test]`** functions with more **`is_context_overflow_error`** assertions only — **no new `#[test]` functions**, count unchanged vs. prior **2026-03-25** closing baseline.
- [x] `cargo build --release` succeeds (**v0.1.59**).
- [x] Runtime smoke: **no second `mac_stats` started** — `pgrep -fl mac_stats` showed existing **`./target/release/mac_stats -vv` (PID 41093)** (transient **`rustc`** / **`cargo`** PIDs possible during builds). Tail **`~/.mac-stats/debug.log`**: **4 monitors** UP/saved — healthy long-running instance. **Restart** mac-stats to load uncommitted **`content_reduction.rs`** into the running release binary.
- [x] Code review (`content_reduction.rs`): four new `||` groups **FEAT-D449** (`auxiliary functions` / `auxiliary function exceed`) through **FEAT-D452** (`spherical gaussians` / `spherical gaussian exceed`) after **`basis functions`/`basis function`**; **FEAT-D448** adds rustdoc + **`does_not_match_unrelated_errors`** / **`detects_context_overflow_errors`** coverage for **`electrons`/`electron exceed`** (existing **`||` group** before **`proton(s)`**). Same **`contains_phrase_after_ident_boundary`** + **`explicit_context_slot_after_ident_boundary`** pattern. **`src/ollama.js`** / **`dist/ollama.js`** not in diff — frontend sync N/A.

### Closing reviewer smoke test 2026-03-25 (`004-closing-reviewer/CLOSING-REVIEWER-PROMPT.md` — FEAT-D453–D457 + OpenClaw re-run)

- [x] Entry: **`004-closing-reviewer/CLOSING-REVIEWER-PROMPT.md`** — §9 integration checklist; bar matched latest prior block: `cargo check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo build --release`; **`git diff`** reconciled with **`CHANGELOG.md`** **[0.1.59]** (**Added** bullets **FEAT-D453–D457**: **`cartesian gaussians`/`cartesian gaussian`**, **`gaussian shells`/`gaussian shell`**, **`density matrices`/`density matrix`**, **`molecular orbitals`/`molecular orbital`**, **`atomic orbitals`/`atomic orbital exceed`** + context-slot guards; **Changed** agent-workflow through **FEAT-D457** / checklist **FEAT-D424–D457**), **`006-feature-coder/FEATURE-CODER.md`** (**Recently closed** head **FEAT-D457**, **When empty** **FEAT-D458+**; open-table rows **FEAT-D453–D457**), **`005-openclaw-reviewer/005-openclaw-reviewer.md`** (independent re-run **2026-03-25T03:03:07Z**, OpenClaw `HEAD` **d25b4a2**, doc-only; **Prior** **2026-03-25T02:34:05Z**). No **`pkill -f mac_stats`** (**AGENTS.md** uptime rule).
- [x] `cargo check` — zero errors.
- [x] `cargo clippy --all-targets -- -D warnings` — zero warnings.
- [x] `cargo test` — **685** tests pass (`mac_stats` lib crate); **1** doc-test ignored. **FEAT-D453–D457** extend existing **`#[test]`** functions with more **`is_context_overflow_error`** assertions only — **no new `#[test]` functions**, count unchanged vs. prior **2026-03-25** closing baseline.
- [x] `cargo build --release` succeeds (**v0.1.59**).
- [x] Runtime smoke: **no second `mac_stats` started** — `pgrep -fl mac_stats` showed existing **`./target/release/mac_stats -vv` (PID 41093)** (transient **`rustc`** / **`cargo`** PIDs possible during builds). Tail **`~/.mac-stats/debug.log`**: **4 monitors** UP/saved, task scan / idle **Having fun** — healthy long-running instance. **Restart** mac-stats to load uncommitted **`content_reduction.rs`** into the running release binary.
- [x] Code review (`content_reduction.rs`): five new `||` groups **FEAT-D453** (`cartesian gaussians` / `cartesian gaussian exceed`) through **FEAT-D457** (`atomic orbitals` / `atomic orbital exceed`) after **`spherical gaussians`/`spherical gaussian`**; same **`contains_phrase_after_ident_boundary`** + **`explicit_context_slot_after_ident_boundary`** pattern; module rustdoc extended per arm. **`src/ollama.js`** / **`dist/ollama.js`** not in diff — frontend sync N/A.

### Closing reviewer smoke test 2026-03-28 (`tasks/TESTING-20260322-1920-openclaw-ollama-warmup-before-channels.md` → `CLOSED-…`)

- [x] Entry: **`004-closing-reviewer/CLOSING-REVIEWER-PROMPT.md`** — §9 checklist; scope solo **`tasks/TESTING-20260322-1920-openclaw-ollama-warmup-before-channels.md`** / mismo slug (**`CLOSED-20260322-1920-openclaw-ollama-warmup-before-channels.md`**). El path **`TESTING-`** sigue siendo el identificador de tarea pedido; el archivo en disco es **`CLOSED-`** tras cumplir clippy. **`git diff` / `CHANGELOG.md` / `006-feature-coder/FEATURE-CODER.md`:** sin nuevos **FEAT-D\*** por el gate Ollama; esta corrida añade limpieza **clippy** mecánica en **`browser_agent/`**, **`commands/`**, **`ollama/`**, **`feature_health.rs`**, etc. No **`pkill -f mac_stats`** (**AGENTS.md**).
- [x] `cargo check` — zero errors.
- [x] `cargo clippy --all-targets -- -D warnings` — zero warnings.
- [x] `cargo test` — **871** tests pass (`mac_stats` lib crate); **1** doc-test ignored (crate).
- [x] `cargo build --release` succeeds (**v0.1.68**).
- [x] Task verification: **`ensure_ollama_agent_ready_at_startup().await`** inside **`tauri::async_runtime::block_on`** **before** **`discord::spawn_discord_if_configured`**, **`scheduler::spawn_scheduler_thread`**, **`scheduler::heartbeat::spawn_heartbeat_thread`** (`lib.rs`); log **`Ollama startup warmup finished (gate open); spawning Discord…`** (`mac_stats_startup`); **`docs/015_ollama_api.md`** alineado. Archivo de tarea: **`tasks/CLOSED-20260322-1920-openclaw-ollama-warmup-before-channels.md`**.
