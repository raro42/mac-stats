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
- [x] Verify `toggle_cpu_window` logic: close visible → recreate is the same behaviour as before (no regression).
- [x] The function always recreates the window after closing — **verified intentional.** In `status_bar.rs`, after closing the window (whether it was visible or hidden), the code checks `if app_handle.get_window("cpu").is_none()` and then calls `create_cpu_window(app_handle)`. So every click ends with the window existing and open; there is no path that leaves the window closed. Effectively this is "show CPU window (create if needed)" rather than a strict toggle. To allow "close and leave closed" we would skip the final create when the window was visible before close.
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

## 4. Integration review

| Area | What to verify |
|------|----------------|
| **Compilation** | `cargo check` passes (confirmed). Run `cargo clippy` for lint warnings. |
| **Frontend sync** | `src/ollama.js` has changes — must be synced to `src-tauri/dist/ollama.js`. Run `scripts/sync-dist.sh` or manually copy. |
| **New Tauri commands** | `toggle_cpu_window` and `set_chat_verbosity` must be in `tauri::generate_handler![]` in `lib.rs`. |
| **Session file compat** | Old session files use `session-memory-{topic}-{id}-{ts}.md`; new code creates `session-memory-{id}-{ts}-{topic}.md`. The `load_messages_from_latest_session_file` searches by `session-memory-{id}-` prefix — old files won't match. Decide: delete old files or add backward compat. |
| **Soul path** | Unified to `~/.mac-stats/agents/soul.md`. Users with a customized `~/.mac-stats/agent/soul.md` need to move it. |
| **`run_due_monitor_checks` caller** | Verify it's wired to a background thread. If not, it's dead code. |

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
- **Assembly order:** (1) Soul, (2) Discord user context (when from Discord), (3) Prompt (planning or execution with `{{AGENTS}}` expanded), (4) Plan (execution step only). Code: `commands/ollama.rs` → `answer_with_ollama_and_fetch()`.
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
- **F1**: `get_messages()` is called in Discord before the current user message is added; ordering correct. `load_messages_from_latest_session_file()` uses prefix `session-memory-{id}-` — old format `session-memory-{topic}-{id}-{ts}` does not match (see D1).
- **F2**: `rev().take(20).rev()` at `ollama.rs` 3502–3508 keeps last 20 messages in chronological order. History cap 20 is consistent (CONVERSATION_HISTORY_CAP and Discord HISTORY_CAP).
- **F3/F4**: Soul path and router injection — not re-verified in this pass; doc says resolved.
- **F5**: Dedup in `task/mod.rs` — slug and `## Topic:`/`## Id:` matching present; D2 resolved (block + suggest new id in error message).
- **F6**: Prompt guidance text present in agent descriptions.
- **F7**: `ellipse()`: all call sites use `max_len` ≥ 20; edge case `max_len` < 3 could panic (first_count 0, last_count negative) — consider `max_len.max(SEP_LEN + 1)` for robustness.
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
