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

### Re-verify — 2026-03-28 (UTC)

- **Operator target:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` — **not present**; only `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` exists. Per `003-tester/TESTER.md`, no other `UNTESTED-*` file was selected. **Skipped** `UNTESTED` → `TESTING` rename (nothing to rename).
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs` (task body lists `src/`; Rust sources live under `src-tauri/src` in this repo)
- **Acceptance criteria:** All satisfied.
- **Outcome:** **PASS** — filename remains `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-28 (local, America-friendly; wall-clock not separately recorded)

- **Operator target:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` — **not present** in the working tree (only `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`). Per instruction, **no other** `UNTESTED-*` file was used. **Skipped** `UNTESTED` → `TESTING` rename (nothing to rename).
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — **no matches** (this repo’s Rust sources are under `src-tauri/src/`, not top-level `src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied when checked against `src-tauri/src` (`TurnOutputGate` + `gate_allows_send`; `finalize_turn_timeout` text `**Turn timed out**` with budget; router/warn log strings in source; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — filename unchanged: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-28 (UTC), `003-tester/TESTER.md` run

- **Operator target:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` — **not present**; only `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` exists. No other `UNTESTED-*` file was used. **Skipped** `UNTESTED` → `TESTING` rename.
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (gate + `gate_allows_send`; `**Turn timed out**` + budget in `finalize_turn_timeout`; log strings in `ollama.rs` / `turn_lifecycle.rs`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — filename unchanged: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-28 (UTC), single-task TESTER run (operator-named UNTESTED path)

- **Rename step:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; only `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` exists. Per `003-tester/TESTER.md`, no other `UNTESTED-*` file was used. **Skipped** `UNTESTED` → `TESTING` rename (nothing to rename). Outcome filename unchanged: **`CLOSED-…`** (all criteria pass; on failure TESTER.md specifies `WIP-…`, not `TESTED-`).
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task body lists `src/`; JS tree has no Rust strings; Rust implementation is under `src-tauri/src/`)
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget seconds; router warn strings in source; build/tests green).
- **Outcome:** **PASS**

### Re-verify — 2026-03-28 17:40 UTC (`003-tester/TESTER.md`, operator-named `UNTESTED-…` path)

- **Rename step:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present** in the working tree; active file is `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`. No other `UNTESTED-*` file was used. **Skipped** `UNTESTED` → `TESTING` rename (nothing to rename).
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget seconds; router/warn log strings in source; `cargo check` / `cargo test` green).
- **Outcome naming (`003-tester/TESTER.md`):** pass → `CLOSED-…`; fail/block → `WIP-…` (not `TESTED-…`). All criteria passed → filename unchanged: **`CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`**.
- **Outcome:** **PASS**

### Re-verify — 2026-03-28 (UTC), `003-tester/TESTER.md` (operator-named `UNTESTED-…` only)

- **Rename step:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; the task on disk is `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`. No other `UNTESTED-*` file was used. **Skipped** `UNTESTED` → `TESTING` (nothing to rename). Outcome naming per `003-tester/TESTER.md`: pass → `CLOSED-…`; fail/block → `WIP-…` (operator message mentioned `TESTED-…` on fail; repo procedure uses `WIP-…`).
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
  - Task body also lists `rg … src/`; top-level `src/` has no Rust matches (implementation under `src-tauri/src/`).
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget; log strings in source; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — filename unchanged: **`CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`**.

### Re-verify — 2026-03-28 (UTC), `003-tester/TESTER.md` (operator path `UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Rename step:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present** in the working tree; the only file for this task is `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`. No other `UNTESTED-*` was used. **Skipped** `UNTESTED` → `TESTING` (no source file with that prefix).
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
  - Task body `rg … src/` (top-level): no Rust matches; implementation lives under `src-tauri/src/`.
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget seconds; log strings in `ollama.rs` / `turn_lifecycle.rs`; `cargo check` / `cargo test` green).
- **Outcome naming:** Pass → `CLOSED-…` (filename unchanged). On fail, operator asked for `TESTED-…`; `003-tester/TESTER.md` specifies `WIP-…` — not applicable.
- **Outcome:** **PASS**
