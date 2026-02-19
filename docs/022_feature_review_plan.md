# Feature Review Plan — `feat/agents-discord-mcp-scheduler` branch

**Date:** 2026-02-19  
**Branch:** `feat/agents-discord-mcp-scheduler` (up-to-date with remote)  
**Build:** Compiles cleanly (`cargo check` passes, no warnings)  
**Uncommitted changes:** 19 files modified, 2 new files (+422 / -115 lines)

---

## 1. Scope of uncommitted changes

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

---

## 2. Review plan — per feature

### F1: Session memory for Discord

**What changed:**
- `session_memory.rs`: Added `get_messages()` (returns in-memory prior turns for a session key) and `load_messages_from_latest_session_file()` (resumes context from disk after restart). Session file naming reordered to `session-{id}-{timestamp}-{topic}.md`.
- `discord/mod.rs`: Before each Ollama call, loads session history and passes it as `conversation_history`.

**Review checklist:**
- [ ] Verify `get_messages()` returns messages **before** the current user message is added (ordering matters).
- [ ] Verify `load_messages_from_latest_session_file()` correctly parses `## User` / `## Assistant` blocks — test with a real session file (edge cases: empty blocks, blocks with `## ` in content).
- [ ] Confirm the filename format change (`session-{id}-{ts}-{topic}` vs old `session-{topic}-{id}-{ts}`) doesn't break loading of pre-existing session files.
- [ ] Check the 20-message cap is applied consistently (both in `get_messages` consumer and in `answer_with_ollama_and_fetch`).
- [ ] Manual test: send 3-4 messages to the Discord bot, verify it references earlier messages correctly (e.g. "what did I just say?").
- [ ] Manual test: restart app, send a message — verify it resumes from the latest session file.

### F2: Conversation history in router

**What changed:**
- `answer_with_ollama_and_fetch` gains a new parameter `conversation_history: Option<Vec<ChatMessage>>`. When present, last 20 messages are prepended to planning and execution prompts.
- All callers updated: scheduler and task runner pass `None`; Discord passes session memory.

**Review checklist:**
- [ ] Verify the `rev().take(20).rev()` pattern correctly keeps the **last** 20 messages in chronological order.
- [ ] Confirm history messages are placed **after** the system prompt and **before** the current question in the Ollama messages array.
- [ ] Verify that adding history doesn't exceed model context window (the existing context-window-aware truncation should handle this, but test with a model with small `num_ctx`).
- [ ] Confirm scheduler (`None`) and task runner (`None`) are unaffected — no regressions.

### F3: Unified soul path (`~/.mac-stats/agents/soul.md`)

**What changed:**
- Consolidated from two directories (`~/.mac-stats/agent/` and `~/.mac-stats/agents/`) into one: `~/.mac-stats/agents/soul.md`.
- Removed `Config::agent_soul_dir()`, `Config::agent_soul_file_path()`, `Config::ensure_agent_soul_directory()`, `Config::load_agents_fallback_soul_content()`, `Config::load_default_soul_content()`.
- New unified `Config::soul_file_path()` and `Config::load_soul_content()` — reads from `~/.mac-stats/agents/soul.md`, writes DEFAULT_SOUL there if missing.
- Both agents (as fallback) and the router (non-agent chat) use the same path and method.

**Review checklist:**
- [ ] Confirm an agent with its own `soul.md` does NOT also get the shared soul (no double soul).
- [ ] Confirm an agent without `soul.md` gets the shared `agents/soul.md` content.
- [ ] Confirm the router (non-agent chat) uses `agents/soul.md` for personality.
- [ ] Confirm `load_agents()` info log correctly reports shared soul presence.
- [ ] If user had a customized `~/.mac-stats/agent/soul.md`, they need to move it to `~/.mac-stats/agents/soul.md` (document in changelog).

### F4: Router soul injection

**What changed:**
- In `answer_with_ollama_and_fetch`, when no `skill_content` or agent override is active, the router now prepends `~/.mac-stats/agents/soul.md` content to system prompts, giving Ollama personality in "plain" chat (Discord without agent override, scheduler free-text tasks).

**Review checklist:**
- [ ] Verify `load_soul_content()` is called (reads from `~/.mac-stats/agents/soul.md`).
- [ ] Verify that when `skill_content` is Some, the soul is NOT prepended (avoids double system prompt).
- [ ] Verify that when `agent_override` is Some, the agent's own combined prompt is used and the router soul is skipped.

### F5: TASK_CREATE deduplication

**What changed:**
- `task/mod.rs`: Before creating a task, `existing_task_with_topic_id()` scans `~/.mac-stats/task/` for any file matching `task-{slug}-{id}-*`. If found, `create_task()` returns an error.

**Review checklist:**
- [ ] Verify slug generation is deterministic (same topic always gives same slug).
- [ ] Verify the dedup check matches by `topic_slug + id` prefix, not the full filename (so status differences don't matter).
- [ ] Edge case: what if the existing task is `finished` or `unsuccessful`? Should creating a new task with the same topic+id be allowed? Currently it is not — decide if this is intentional.
- [ ] Verify the error message is informative enough for Ollama to switch to TASK_APPEND.

### F6: Prompt guidance for agent chats

**What changed:**
- TASK description in `build_agent_descriptions()` now includes: "When the user wants agents to chat, invoke AGENT: orchestrator so the conversation runs; do not only create a task."
- Also: "If a task with that topic and id already exists, use TASK_APPEND or TASK_STATUS instead."

**Review checklist:**
- [ ] Read the full prompt text in context — confirm it doesn't create contradictory instructions.
- [ ] Manual test: ask Discord "have the agents chat" — verify the model outputs `AGENT: orchestrator` instead of just `TASK_CREATE`.

### F7: Logging improvements

**What changed:**
- New `ellipse()` function: shows first half + `...` + last half instead of hard truncation. Used in `browser.rs`, `discord/api.rs`, and `commands/ollama.rs`.
- Verbosity-aware logging: at `-vv` or higher, request/response logs are never truncated.
- Discord API: fixed char-count vs byte-count mismatch (`.chars().count()` instead of `.len()`).

**Review checklist:**
- [ ] Verify `ellipse()` handles edge cases: empty string, string shorter than `max_len`, string exactly `max_len`, very small `max_len` (< 3).
- [ ] Confirm the `VERBOSITY` atomic is the same one set by CLI `-v`/`-vv`/`-vvv` flags and the new `set_chat_verbosity`.
- [ ] Verify `browser.rs` truncation change: old code appended `[truncated]`; new code uses `ellipse()` which shows `...`. This changes the semantics for FETCH_URL content passed to Ollama — confirm the model still understands the page was cut.

### F8: Chat reserved words

**What changed:**
- `src/ollama.js`: `sendChatMessage()` intercepts `--cpu`, `-v`, `-vv`, `-vvv` before sending to Ollama.
- `--cpu` → invokes `toggle_cpu_window` Tauri command.
- `-v`/`-vv`/`-vvv` → invokes `set_chat_verbosity` Tauri command (runtime verbosity change).
- New Tauri commands registered in `lib.rs`.

**Review checklist:**
- [ ] Verify reserved words are NOT added to conversation history (the `return` before `addToHistory` is correct).
- [ ] Verify `set_chat_verbosity` updates the same `VERBOSITY` atomic used by logging macros.
- [ ] Verify `toggle_cpu_window` works from the CPU window chat (meta: toggling from within the window you're in — should it close itself? Is that the desired UX?).
- [ ] Check that `src/ollama.js` changes are synced to `src-tauri/dist/ollama.js` before testing.

### F9: Toggle CPU window refactor

**What changed:**
- Extracted inline window toggle logic from `click_handler_class` into `toggle_cpu_window()` function in `status_bar.rs`.
- New `commands/window.rs` exposes it as a Tauri command (uses `run_on_main_thread`).

**Review checklist:**
- [ ] Verify `toggle_cpu_window` logic: close visible → recreate is the same behaviour as before (no regression).
- [ ] The function always recreates the window after closing — verify this is intentional (it means "toggle" always ends with the window open if it was hidden but existed; the only way to close is if it was visible).
- [ ] Verify `run_on_main_thread` is safe here — Tauri docs say it may block; confirm the Tauri command is async enough to not hang the frontend.

### F10: Background monitor checks

**What changed:**
- `commands/monitors.rs`: New `run_due_monitor_checks()` function that iterates all monitors, checks if `last_check + interval >= now`, and runs `check_monitor()` for due ones.
- Uses `try_lock()` to avoid blocking.

**Review checklist:**
- [ ] Verify this function is actually called somewhere (it's defined but the caller is not visible in the diff — check `lib.rs` for a background thread or timer).
- [ ] Confirm `try_lock()` usage is safe: if the lock is busy, checks are skipped entirely (acceptable for a background thread).
- [ ] Verify that `check_monitor()` saves stats to disk after each check (existing behaviour — confirm not regressed).
- [ ] Edge case: monitors with `check_interval_secs = 0` — would this run every cycle? Confirm there's a minimum.

---

## 3. Integration review

| Area | What to verify |
|------|----------------|
| **Compilation** | `cargo check` passes (confirmed). Run `cargo clippy` for lint warnings. |
| **Frontend sync** | `src/ollama.js` has changes — must be synced to `src-tauri/dist/ollama.js`. Run `scripts/sync-dist.sh` or manually copy. |
| **New Tauri commands** | `toggle_cpu_window` and `set_chat_verbosity` must be in `tauri::generate_handler![]` in `lib.rs`. |
| **Session file compat** | Old session files use `session-memory-{topic}-{id}-{ts}.md`; new code creates `session-memory-{id}-{ts}-{topic}.md`. The `load_messages_from_latest_session_file` searches by `session-memory-{id}-` prefix — old files won't match. Decide: delete old files or add backward compat. |
| **Soul path** | Unified to `~/.mac-stats/agents/soul.md`. Users with a customized `~/.mac-stats/agent/soul.md` need to move it. |
| **`run_due_monitor_checks` caller** | Verify it's wired to a background thread. If not, it's dead code. |

---

## 4. Manual testing plan

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

## 5. Decisions needed before commit

| # | Decision | Options |
|---|----------|---------|
| D1 | **Session file format change** — old files won't load after restart | (a) Add backward compat to also search old naming, (b) Accept break (users lose old session context), (c) Write migration |
| D2 | **Finished task dedup** — should `TASK_CREATE` be allowed when an existing task with same topic+id is `finished`? | (a) Allow (re-create), (b) Block (current behaviour), (c) Block but suggest a new id |
| D3 | **`ellipse()` in FETCH_URL** — changes truncation marker from `[truncated]` to `...` | (a) Keep `...` (cleaner), (b) Use `... [content truncated]` (clearer for LLM) |
| D4 | **`run_due_monitor_checks` wiring** — is it called? | (a) Wire to background thread, (b) Remove if premature |
| D5 | **Soul path naming** — `agent/` vs `agents/` was confusing | RESOLVED: consolidated to `~/.mac-stats/agents/soul.md` only |

---

## 6. Suggested commit strategy

Given the 10 features span different concerns, consider splitting into 2-3 atomic commits:

1. **Logging + refactors** (F7, F9): `ellipse()`, verbosity-aware logging, `toggle_cpu_window` extraction. Pure improvements, no behaviour change.
2. **Session memory + router history + soul hierarchy** (F1, F2, F3, F4): The core "agents remember context" feature.
3. **Task dedup + prompt fixes + chat reserved words + monitor checks** (F5, F6, F8, F10): Bug fixes and small features.

Or commit all at once with a descriptive message covering the highlights.
