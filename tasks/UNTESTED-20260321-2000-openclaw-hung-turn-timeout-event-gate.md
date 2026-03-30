# OpenClaw: hung turn wall-clock timeout + output event gate

Full-turn wall-clock timeout stops a hung agent run: output gate closes (no Discord status/draft/ATTACH spam), user-visible **Turn timed out** reply, optional `about:blank` cleanup only if the timed-out `request_id` still owns the coordination slot.

**Where the code lives:** The title refers to the **agent router** behavior implemented in **this repository (mac-stats)** under `src-tauri/src/`. Do **not** expect to verify this task in the sibling **openclaw** checkout (`../openclaw`); that path is unrelated to these acceptance checks.

## Acceptance criteria

1. `TurnOutputGate` (`Arc<AtomicBool>`) exists; the tool loop respects `gate_allows_send` after timeout closes the gate.
2. `finalize_turn_timeout` in `commands/turn_lifecycle.rs` returns `OllamaReply` whose `text` starts with `**Turn timed out**` and includes the budget in seconds (see the `format!` that builds the user message).
3. **Log strings (static check):** Router-side code logs include the substring `closing output gate after turn wall-clock timeout`. Turn-lifecycle warning text includes `turn wall-clock timeout` and `closing output gate and running cleanup`. For this task, **“logs include” means these literals appear in the Rust sources** that emit them (verified with `rg` below). A live Discord timeout repro is **optional**, not required for pass.
4. `cargo check` and `cargo test` in `src-tauri/` succeed on the host used for verification (typically macOS for this project).

## Testing instructions

- **Repository:** **mac-stats** (workspace root: directory containing `src-tauri/Cargo.toml`, plus top-level `src/` and `src-tauri/`).
- **Backend path:** Implementation and acceptance strings live under **`src-tauri/src/`** only. Top-level **`src/`** is the web frontend; do **not** treat “no matches in `src/`” as a failure.
- **Preflight:** From repo root, `test -f src-tauri/Cargo.toml && echo OK` must print `OK` before `cargo` or `rg` steps.
- **Runtime log (optional):** If you want to confirm emissions at run time, reproduce or simulate a turn timeout and grep **`~/.mac-stats/debug.log`** for the same substrings. This is **not** part of the minimum pass bar if criteria 1–4 are met via source + build.

## Verification commands

From the **mac-stats** repository root:

```bash
test -f src-tauri/Cargo.toml
( cd src-tauri && cargo check && cargo test )
rg -n "TurnOutputGate|gate_allows_send|finalize_turn_timeout" src-tauri/src
rg -n "\*\*Turn timed out\*\*" src-tauri/src/commands/turn_lifecycle.rs
rg -n "closing output gate after turn wall-clock timeout" src-tauri/src
rg -n "turn wall-clock timeout|closing output gate and running cleanup" src-tauri/src/commands/turn_lifecycle.rs
```

Notes:

- The `( cd src-tauri && … )` subshell returns you to **repo root** so later `rg … src-tauri/src` paths stay correct. If you stay in `src-tauri/`, run the same patterns against **`src/`** (relative to that directory) instead.
- If `cargo test` is slow, `cargo test --no-fail-fast` is acceptable; the requirement is **zero failing tests** for this crate on the verification host.

## Test report

_(Tester: append results after `UNTESTED` → `TESTING` per `003-tester/TESTER.md`.)_
