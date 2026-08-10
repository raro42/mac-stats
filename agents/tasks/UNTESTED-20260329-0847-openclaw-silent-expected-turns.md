# OpenClaw parity: silent expected turns for `schedules.json` (HEARTBEAT_OK)

## Goal

Isolated **scheduled** runs with `reply_to_channel_id` should treat a **heartbeat-only** model reply like OpenClaw cron (`skipHeartbeatDelivery` / `isHeartbeatOnlyResponse`): **no Discord post** and **no** `scheduler_delivery_awareness` row when the text is only `HEARTBEAT_OK` (plus optional short tail per `config.json` → `heartbeat.ackMaxChars`) and there are **no** attachments from that Ollama turn.

## Acceptance criteria

1. After a successful **Ollama** schedule run, if the reply matches the same ack rules as `scheduler/heartbeat.rs` (`is_heartbeat_ack`) using `Config::heartbeat_settings().ack_max_chars`, and the run has **no** attachment paths from `OllamaReply`, the scheduler **does not** call Discord send and **does not** record delivery awareness.
2. **TASK:** / **TASK_RUN:** completions use the same ack rule in `task/runner.rs` before sending the “Task finished” summary so Discord is not spammed; the scheduler path remains a safety net for plain Ollama schedule tasks.
3. If the reply would be a heartbeat ack **but** the Ollama reply included attachments, delivery is **not** suppressed (parity with “media present” in OpenClaw).
4. `cd src-tauri && cargo check && cargo test` succeed.

## Implementation (mac-stats)

- `src-tauri/src/scheduler/heartbeat.rs` — `should_skip_discord_for_heartbeat_ack(reply, ack_max_chars, has_attachments)` built on `is_heartbeat_ack`.
- `src-tauri/src/scheduler/mod.rs` — `ScheduleExecuteSuccess` carries `attachment_count` from `OllamaReply`; Discord branch checks skip before send.
- `src-tauri/src/task/runner.rs` — skip finished-summary Discord send when ack-only (no attachments in this path).

## Verification (automated)

```bash
cd src-tauri && cargo check && cargo test
```

```bash
rg -n "should_skip_discord_for_heartbeat_ack|Schedule ack|schedule heartbeat ack" src-tauri/src/scheduler src-tauri/src/task/runner.rs
```

---

## Testing instructions

### What to verify

- A **cron schedule** with `reply_to_channel_id` whose Ollama result is only `HEARTBEAT_OK` (optional tiny tail within `heartbeat.ackMaxChars` in `~/.mac-stats/config.json`) **does not** produce a new Discord message for that run.
- With `-vv`, `~/.mac-stats/debug.log` contains an **info** line from subsystem `scheduler`: `Schedule ack — not delivering to Discord (HEARTBEAT_OK contract, …)` when the skip triggers.
- For a **TASK:** schedule that finishes with a summary matching the heartbeat ack pattern, the task runner logs `Task runner: schedule heartbeat ack — not delivering finished summary to Discord (channel …)` and Discord receives neither the “Task finished” block nor a duplicate scheduler post for that ack text.
- `scheduler_delivery_awareness.json` does **not** gain an entry when delivery was skipped for heartbeat ack.
- A substantive schedule reply (no heartbeat ack) still posts as before.

### How to test

1. Run the **Verification (automated)** commands above.
2. **Log grep (after an ack-only run or from historical logs):**  
   `rg 'Schedule ack — not delivering|Task runner: schedule heartbeat ack' ~/.mac-stats/debug.log`  
   Expect a match when a skip occurred. The scheduler path logs via `mac_stats_info!("scheduler", …)` (subsystem `scheduler`, message begins `Schedule ack — not delivering to Discord`); the task-runner path uses `tracing::info!` with prefix `Task runner:`—both appear in `debug.log` at INFO with default `-vv`.
3. **E2E (optional, needs Discord + Ollama):** Add a schedule with a trivial prompt that instructs the model to answer `HEARTBEAT_OK` only, `reply_to_channel_id` set to a test channel, and a short cron (or wait for the next tick). Confirm no Discord message on that run and the log line above appears.
4. **Regression:** Send a normal schedule task that returns a short real summary; confirm Discord still receives `[Schedule: …]` as before.

### Pass / fail criteria

- **Pass:** `cargo check` / full `cargo test` green; optional E2E shows no spam for ack-only runs and unchanged behaviour for normal replies.
- **Fail:** Build or test failure; Discord still receives bare `HEARTBEAT_OK` for schedule runs; delivery awareness row written for skipped acks; substantive replies no longer delivered.
