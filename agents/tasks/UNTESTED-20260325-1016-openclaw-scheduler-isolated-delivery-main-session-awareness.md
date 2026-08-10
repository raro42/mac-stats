# Scheduler → Discord delivery awareness for CPU (main) chat

## Goal

After a **scheduler-initiated** run posts text to Discord (`reply_to_channel_id`), the **in-app CPU window chat** should see a concise system block listing recent successful deliveries (OpenClaw-style “main session awareness” after isolated cron delivery), so the model can continue without blindly re-sending the same content.

## Acceptance criteria

1. **Persistence:** Successful scheduler Discord deliveries append to `scheduler_delivery_awareness.json` under the same directory as `schedules.json` (`~/.mac-stats/`), capped and de-duplicated by `context_key`.
2. **Recording:** The task/runner path calls `delivery_awareness::record_if_new` only after Discord accepts the message when scheduler delivery context is present.
3. **CPU chat injection:** The frontend Ollama chat path prepends `delivery_awareness::format_for_chat_context()` to the system prompt when non-empty (`augment_cpu_system_with_scheduler_awareness` in `commands/ollama_frontend_chat.rs`).
4. **API:** `list_scheduler_delivery_awareness` remains available for Settings/debug (newest-first).

## Implementation (mac-stats)

- `src-tauri/src/scheduler/delivery_awareness.rs` — file I/O, cap, dedupe, `format_for_chat_context`, unit tests.
- `src-tauri/src/scheduler/mod.rs` — `new_context_key_for_schedule`, passes `(context_key, schedule_id)` into `task::runner` when `reply_to_channel_id` is set; `record_if_new` after Discord send for Ollama-scheduler replies and task-runner fallback when the scheduler sends the summary.
- `src-tauri/src/task/runner.rs` — `record_if_new` after successful `send_message_to_channel` when `scheduler_delivery_awareness` is `Some`.
- `src-tauri/src/commands/ollama_frontend_chat.rs` — `augment_cpu_system_with_scheduler_awareness`.
- `src-tauri/src/commands/scheduler.rs` + `lib.rs` — Tauri command `list_scheduler_delivery_awareness`.

Operator-oriented behaviour: `docs/009_scheduler_agent.md`, `docs/data_files_reference.md` (`scheduler_delivery_awareness.json`).

**Coder workflow note:** `002-coder-backend/CODER.md` lives under the **mac-stats-reviewer** workspace (`agents/002-coder-backend/CODER.md`), not inside this repo.

## Verification (automated)

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test delivery_awareness -- --nocapture
```

Optional wiring check:

```bash
rg -n "format_for_chat_context|record_if_new" src-tauri/src/scheduler src-tauri/src/commands/ollama_frontend_chat.rs src-tauri/src/task/runner.rs
```

---

## Testing instructions

### What to verify

- After a **scheduled** job with **`reply_to_channel_id`** successfully posts to Discord, a new row appears in `~/.mac-stats/scheduler_delivery_awareness.json` (valid JSON, `entries` array growing; each item has `context_key`, `utc`, `channel_id`, `summary`; duplicates with the same `context_key` are not appended).
- The **CPU window** Ollama chat’s system prompt includes the scheduler→Discord awareness block when that file has at least one entry (model should “know” what was already posted).
- With **verbosity `-vv`**, `~/.mac-stats/debug.log` contains a **debug** line about prepending scheduler delivery awareness when the block is non-empty (`CPU chat: prepending scheduler→Discord delivery awareness` / grep `prepending scheduler` or `delivery awareness` as in `docs/009_scheduler_agent.md`).
- **Settings → Schedules** (or the Tauri command `list_scheduler_delivery_awareness`) shows recent deliveries **newest-first**.

### How to test

1. Run automated verification (commands in **Verification** above).
2. **E2E (optional, needs Discord bot + Ollama):** Configure a schedule with `reply_to_channel_id` pointing to a test channel and a harmless task (e.g. short Ollama prompt or `TASK:` that finishes quickly). Let it fire once; confirm the message appears in Discord, then open `~/.mac-stats/scheduler_delivery_awareness.json` and confirm a new entry whose `summary` matches the posted text (truncation allowed).
3. Open the **CPU window**, send any chat message with `-vv` logging enabled; tail `~/.mac-stats/debug.log` and confirm the prepend debug line when the JSON file is non-empty.
4. Clear or rename `scheduler_delivery_awareness.json`, send another CPU chat turn, and confirm the debug line does **not** appear (empty block).

### Pass / fail criteria

- **Pass:** `cargo check` and `cargo test delivery_awareness` succeed; optional `rg` wiring hits the four expected modules; E2E (if run) shows JSON growth only after successful Discord delivery; CPU chat shows prepend log only when the awareness file has entries.
- **Fail:** Build/test failure; `record_if_new` firing without a successful Discord send; duplicate `context_key` rows in JSON; CPU chat never prepends when the file has entries; list API order not newest-first.
