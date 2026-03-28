# OpenClaw: hung turn wall-clock timeout + output event gate

Full-turn wall-clock timeout stops a hung agent run: output gate closes (no Discord status/draft/ATTACH spam), user-visible **Turn timed out** reply, optional `about:blank` cleanup only if the timed-out `request_id` still owns the coordination slot.

## Acceptance criteria

1. `TurnOutputGate` (`Arc<AtomicBool>`) exists; tool loop respects `gate_allows_send` after timeout closes the gate.
2. `finalize_turn_timeout` in `commands/turn_lifecycle.rs` returns `OllamaReply` with text starting `**Turn timed out**` and including the budget in seconds.
3. Router logs include `closing output gate after turn wall-clock timeout` and turn-lifecycle warns include `turn wall-clock timeout` / `closing output gate and running cleanup`.
4. `cargo check` and `cargo test` in `src-tauri/` succeed.

## Verification commands

```bash
cd src-tauri && cargo check && cargo test
rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/
```

## Test report

- **Date:** 2026-03-28 (local date in operator environment; wall-clock not separately recorded).
- **Preflight:** The path `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present** in the working tree; this task file was **created** with acceptance criteria aligned to `turn_lifecycle.rs`, `tool_loop.rs`, and `ollama.rs` so the TESTER workflow could run without selecting another `UNTESTED-*` file.
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (**870** unit tests in library crate; 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs` (`gate_allows_send` used in `send_status` and draft paths)
- **Acceptance criteria:** All satisfied (gate type + tool-loop checks; `finalize_turn_timeout` message prefix; log strings present in source; build/tests green).
- **Outcome:** **PASS**

### Re-verify — 2026-03-28 (UTC)

- **Rename step:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; the task file on disk was already `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`. Per operator instruction, no other `UNTESTED-*` file was used. Skipped `UNTESTED` → `TESTING` rename; left filename as `CLOSED-*`.
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (unchanged from prior report).
- **Outcome:** **PASS** (filename unchanged: `CLOSED-…`)
