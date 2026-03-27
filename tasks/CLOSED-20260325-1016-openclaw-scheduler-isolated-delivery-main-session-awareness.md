# Scheduler → Discord delivery awareness for CPU (main) chat

## Goal

After a **scheduler-initiated** run posts text to Discord (`reply_to_channel_id`), the **in-app CPU window chat** should see a concise system block listing recent successful deliveries (OpenClaw-style “main session awareness” after isolated cron delivery), so the model can continue without blindly re-sending the same content.

## Acceptance criteria

1. **Persistence:** Successful scheduler Discord deliveries append to `scheduler_delivery_awareness.json` under the same directory as `schedules.json` (`~/.mac-stats/`), capped and de-duplicated by `context_key`.
2. **Recording:** The task/runner path calls `delivery_awareness::record_if_new` only after Discord accepts the message when scheduler delivery context is present.
3. **CPU chat injection:** Frontend Ollama chat path prepends `delivery_awareness::format_for_chat_context()` to the system prompt when non-empty (`augment_cpu_system_with_scheduler_awareness` in `commands/ollama_frontend_chat.rs`).
4. **API:** `list_scheduler_delivery_awareness` remains available for Settings/debug (newest-first).

## Verification

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test delivery_awareness -- --nocapture
```

Optional sanity (documentation / wiring):

```bash
rg -n "format_for_chat_context|record_if_new" src-tauri/src/scheduler src-tauri/src/commands/ollama_frontend_chat.rs src-tauri/src/task/runner.rs
```

## Test report

**Preflight:** `tasks/UNTESTED-20260325-1016-openclaw-scheduler-isolated-delivery-main-session-awareness.md` was **not present** in the working tree at run start. The task body was written to that path, then renamed to `TESTING-20260325-1016-openclaw-scheduler-isolated-delivery-main-session-awareness.md` per `003-tester/TESTER.md`. No other `UNTESTED-*` file was used.

**Date:** 2026-03-27 (local macOS environment).

| Step | Command | Result |
|------|---------|--------|
| Compile | `cd src-tauri && cargo check` | **pass** |
| Unit tests | `cd src-tauri && cargo test delivery_awareness -- --nocapture` | **pass** (3 tests: `new_context_key_has_stable_prefix`, `record_if_new_skips_duplicate_context_key`, `list_entries_newest_first_order`) |
| Wiring | `rg -n "format_for_chat_context|record_if_new" …` | **pass** — hits `ollama_frontend_chat.rs`, `task/runner.rs`, `scheduler/mod.rs`, `delivery_awareness.rs` |

**Notes:** End-to-end Discord delivery was not exercised in this run (no live bot); acceptance is satisfied by code review + unit tests + grep wiring. Manual spot-check: trigger a scheduled task with `reply_to_channel_id`, confirm `~/.mac-stats/scheduler_delivery_awareness.json` grows and CPU chat debug log shows scheduler awareness prepended when block non-empty.

**Outcome:** **CLOSED** — all listed acceptance criteria and automated verification passed.

## Test report (re-run)

**Preflight:** `tasks/UNTESTED-20260325-1016-openclaw-scheduler-isolated-delivery-main-session-awareness.md` was **not** in the tree; the task file was already `CLOSED-*`. Per `003-tester/TESTER.md`, it was renamed `CLOSED-…` → `TESTING-…` for this run, verification executed, then renamed back to `CLOSED-…` on success. No other `UNTESTED-*` file was used.

**Date:** 2026-03-27 (local, America-friendly note: same calendar day as prior report).

| Step | Command | Result |
|------|---------|--------|
| Compile | `cd src-tauri && cargo check` | **pass** |
| Unit tests | `cd src-tauri && cargo test delivery_awareness -- --nocapture` | **pass** (3 tests) |
| Wiring | `rg -n "format_for_chat_context|record_if_new" src-tauri/src` | **pass** — `ollama_frontend_chat.rs`, `delivery_awareness.rs`, `scheduler/mod.rs`, `task/runner.rs` |

**Notes:** Live Discord / E2E not re-run; automated criteria unchanged.

**Outcome:** **CLOSED** — all verification steps passed.
