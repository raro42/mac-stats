# OpenClaw: hung turn wall-clock timeout + output event gate

Full-turn wall-clock timeout stops a hung agent run: output gate closes (no Discord status/draft/ATTACH spam), user-visible **Turn timed out** reply, optional `about:blank` cleanup only if the timed-out `request_id` still owns the coordination slot.

## Revision note (instructions fix)

Earlier drafts of **Verification commands** used `rg` against top-level `src/`. In **mac-stats**, that directory is the web frontend (HTML/JS/CSS), not the Rust backend. The strings in the acceptance criteria appear under **`src-tauri/src/`** only. The steps below use the correct tree so “no matches under top-level `src/`” is not treated as an implementation failure.

## Acceptance criteria

1. `TurnOutputGate` (`Arc<AtomicBool>`) exists; tool loop respects `gate_allows_send` after timeout closes the gate.
2. `finalize_turn_timeout` in `commands/turn_lifecycle.rs` returns `OllamaReply` with text starting `**Turn timed out**` and including the budget in seconds.
3. Router logs include `closing output gate after turn wall-clock timeout` and turn-lifecycle warns include `turn wall-clock timeout` / `closing output gate and running cleanup`.
4. `cargo check` and `cargo test` in `src-tauri/` succeed.

## Testing instructions

- **Repository:** **mac-stats** (this workspace). All shell snippets assume the **repository root** (directory that contains `src-tauri/Cargo.toml` and both `src/` and `src-tauri/`).
- **Backend code path:** Search and read Rust implementation under **`src-tauri/src/`** only for this task. Do **not** expect the acceptance-criteria strings in top-level **`src/`** (frontend).
- **Optional sanity check:** `test -f src-tauri/Cargo.toml && echo OK` from repo root must print `OK` before running `cargo` or `rg` paths below.

## Verification commands

From the **mac-stats** repository root:

```bash
test -f src-tauri/Cargo.toml
( cd src-tauri && cargo check && cargo test )
rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src
```

The subshell around `cargo` keeps your current directory at **repo root** after the block, so the `rg` path `src-tauri/src` stays valid. If you already `cd src-tauri` manually, run `rg` against **`src`** instead (same pattern, relative to that directory).

## Test report

_(Tester: append results here after `UNTESTED` → `TESTING` per `003-tester/TESTER.md`.)_
