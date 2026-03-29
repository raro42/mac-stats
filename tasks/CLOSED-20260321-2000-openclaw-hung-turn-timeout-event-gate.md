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

### Re-verify — 2026-03-28 (UTC), `003-tester/TESTER.md` (operator path `UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Rename step:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present** in the working tree; the only file for this task is `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`. No other `UNTESTED-*` was used. **Skipped** `UNTESTED` → `TESTING` (no source file with that prefix).
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
  - Task body also lists `rg … src/`; top-level `src/` has no Rust matches (implementation under `src-tauri/src/`).
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget seconds; log strings in source; `cargo check` / `cargo test` green).
- **Outcome naming (`003-tester/TESTER.md`):** pass → `CLOSED-…` (filename unchanged). Fail/block → `WIP-…` (not `TESTED-…`).
- **Outcome:** **PASS** — filename unchanged: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-28 (UTC), `003-tester/TESTER.md` (operator path `UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Rename step:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; the task file is `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`. No other `UNTESTED-*` file was used. **Skipped** `UNTESTED` → `TESTING` (nothing to rename).
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
  - Task body lists `rg … src/`; top-level `src/` has no Rust matches (implementation under `src-tauri/src/`).
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget seconds; router warn `closing output gate after turn wall-clock timeout` and turn-lifecycle `turn wall-clock timeout` / `closing output gate and running cleanup`; `cargo check` / `cargo test` green).
- **Outcome naming (`003-tester/TESTER.md`):** pass → `CLOSED-…`; fail/block → `WIP-…` (operator message sometimes says `TESTED-…` on fail; repo procedure uses `WIP-…`).
- **Outcome:** **PASS** — filename unchanged: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-28 (UTC), `003-tester/TESTER.md` (operator-named `UNTESTED-…` only)

- **Rename step:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; verified against `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only. No other `UNTESTED-*` file was used. **Skipped** `UNTESTED` → `TESTING`.
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
  - Task verification block uses `rg … src/`; this repo’s Rust sources are under `src-tauri/src/` (top-level `src/` is JS).
- **Acceptance criteria:** All satisfied.
- **Outcome naming:** Per `003-tester/TESTER.md`, pass keeps `CLOSED-…`; fail/block would be `WIP-…` (not `TESTED-…`).
- **Outcome:** **PASS** — filename unchanged: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-28 (UTC), `003-tester/TESTER.md` (operator path `UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Rename step:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**. To follow the TESTING-phase rename without touching any other `UNTESTED-*` file, the canonical task file was renamed **`CLOSED-…` → `TESTING-…`** for this run; after verification it is renamed back to **`CLOSED-…`** (pass).
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
  - Task body lists `rg … src/`; top-level `src/` has no Rust matches (implementation under `src-tauri/src/`).
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send` in `tool_loop.rs`; `finalize_turn_timeout` text starts `**Turn timed out**` with budget seconds in `turn_lifecycle.rs`; router string `closing output gate after turn wall-clock timeout` in `ollama.rs`; turn-lifecycle warn strings `turn wall-clock timeout` / `closing output gate and running cleanup`; `cargo check` / `cargo test` green).
- **Outcome naming (`003-tester/TESTER.md`):** pass → `CLOSED-…`; fail/block → `WIP-…` (not `TESTED-…` per repo procedure).
- **Outcome:** **PASS** — file restored to `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-28 UTC (`003-tester/TESTER.md`, operator-named `UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; the task on disk was `CLOSED-…`. To honor the TESTING phase without touching any other `UNTESTED-*`, the file was renamed **`CLOSED-…` → `TESTING-…`**, verification was run, then renamed back to **`CLOSED-…`** on pass.
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
  - Task body also lists `rg … src/`; top-level `src/` (JS) has no matches; Rust lives under `src-tauri/src/`.
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` prefix `**Turn timed out**` with budget seconds; log strings in `ollama.rs` / `turn_lifecycle.rs`; `cargo check` / `cargo test` green).
- **Outcome naming:** Operator asked for `TESTED-` on fail; `003-tester/TESTER.md` specifies `WIP-` — not applicable (pass).
- **Outcome:** **PASS** — final filename: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-28 UTC (`003-tester/TESTER.md`, objetivo operador `UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` únicamente)

- **Renombre UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` **no existía**; no se tocó ningún otro `UNTESTED-*`. Para la fase TESTING se renombró **`CLOSED-…` → `TESTING-…`**, se ejecutó la verificación y, al pasar, se devuelve el nombre a **`CLOSED-…`**.
- **Comandos ejecutados:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (crate biblioteca: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — coincidencias en `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
  - Cuerpo de la tarea lista `rg … src/`; en la raíz `src/` (JS) no hay coincidencias Rust; implementación en `src-tauri/src/`.
- **Criterios de aceptación:** Cumplidos (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` con prefijo `**Turn timed out**` y presupuesto en segundos; cadenas de log en `ollama.rs` / `turn_lifecycle.rs`; `cargo check` / `cargo test` en verde).
- **Nomenclatura de resultado:** El operador pidió `TESTED-` en fallo; `003-tester/TESTER.md` indica `WIP-` — no aplica (pass).
- **Resultado:** **PASS** — nombre final tras este run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-28 UTC (`003-tester/TESTER.md`, operator path `UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. For the TESTING phase the canonical file was renamed **`CLOSED-…` → `TESTING-…`**, then restored to **`CLOSED-…`** after verification (pass).
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
  - Task body lists `rg … src/`; top-level `src/` has no Rust matches (implementation under `src-tauri/src/`).
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` text starts `**Turn timed out**` with budget seconds; router string `closing output gate after turn wall-clock timeout` in `ollama.rs`; turn-lifecycle warns include `turn wall-clock timeout` / `closing output gate and running cleanup`; `cargo check` / `cargo test` green).
- **Outcome naming:** `003-tester/TESTER.md` — pass → `CLOSED-…`; fail/block → `WIP-…` (operator message mentioned `TESTED-…` on fail; repo procedure uses `WIP-…`).
- **Outcome:** **PASS** — final filename: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-28 UTC (`003-tester/TESTER.md`, operator path `UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` **no existía**; no se usó ningún otro `UNTESTED-*`. Fase TESTING: **`CLOSED-…` → `TESTING-…`** antes de verificar; tras **PASS** el archivo vuelve a **`CLOSED-…`**. (`003-tester/TESTER.md`: en fallo/bloqueo sería `WIP-…`, no `TESTED-…`.)
- **Comandos ejecutados:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (crate biblioteca: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — coincidencias en `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
  - El bloque de verificación de la tarea usa `rg … src/`; en `src/` de la raíz (JS) no hay coincidencias Rust; implementación en `src-tauri/src/`.
- **Criterios de aceptación:** Cumplidos (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` con prefijo `**Turn timed out**` y presupuesto en segundos; logs en fuente; `cargo check` / `cargo test` en verde).
- **Resultado:** **PASS** — nombre final: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-28 UTC (`003-tester/TESTER.md`, operator path `UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was touched. **TESTING phase:** renamed **`CLOSED-…` → `TESTING-…`** for this run, appended this report, then restored **`TESTING-…` → `CLOSED-…`** after **PASS**. (`003-tester/TESTER.md`: fail/block → `WIP-…`, not `TESTED-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task body path; Rust lives under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src/` — matches in `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` text starts `**Turn timed out**` with budget seconds; `turn wall-clock timeout` / `closing output gate and running cleanup` in `turn_lifecycle.rs`; router string `closing output gate after turn wall-clock timeout` in `ollama.rs`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-28 UTC (`003-tester/TESTER.md`, operator path `UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** renamed **`CLOSED-…` → `TESTING-…`** for this run, appended this report, then **`TESTING-…` → `CLOSED-…`** after **PASS**. (`003-tester/TESTER.md`: fail/block → `WIP-…`; operator wording `TESTED-…` on fail is superseded by repo `TESTER.md` → `WIP-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task body lists top-level `src/`; Rust is under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` + `gate_allows_send` in tool loop; `finalize_turn_timeout` with `**Turn timed out**` and budget; `turn wall-clock timeout` / `closing output gate and running cleanup` in `turn_lifecycle.rs`; router log string in `ollama.rs`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-28 UTC (`003-tester/TESTER.md`, operator path `UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** this run started with the task file as **`TESTING-…`** (renamed from **`CLOSED-…`** immediately before verification). After **PASS**, filename restored to **`CLOSED-…`**. (`003-tester/TESTER.md`: fail/block → `WIP-…`; operator message mentioned `TESTED-…` on fail — not applicable.)
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task body lists top-level `src/`; Rust under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget seconds; `turn wall-clock timeout` / `closing output gate and running cleanup` in `turn_lifecycle.rs`; router string in `ollama.rs`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-28 20:38 UTC (`003-tester/TESTER.md`, objetivo operador `UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` únicamente)

- **Renombre UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` **no existía** en el árbol de trabajo; no se eligió ningún otro `UNTESTED-*`. Para cumplir la fase TESTING se renombró **`CLOSED-…` → `TESTING-…`** antes de ejecutar la verificación. Tras **PASS**, el archivo vuelve a **`CLOSED-…`**. (En fallo, el operador pidió `TESTED-…`; `003-tester/TESTER.md` indica `WIP-…` — no aplica.)
- **Comandos ejecutados:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (crate biblioteca: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — sin coincidencias (el bloque de la tarea lista `src/`; el Rust está en `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — coincidencias en `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Criterios de aceptación:** Cumplidos (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` con prefijo `**Turn timed out**` y presupuesto en segundos; cadenas `turn wall-clock timeout` / `closing output gate and running cleanup` en `turn_lifecycle.rs`; router `closing output gate after turn wall-clock timeout` en `ollama.rs`; `cargo check` / `cargo test` en verde).
- **Resultado:** **PASS** — nombre final: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-28 20:50 UTC (`003-tester/TESTER.md`, operator path `UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** renamed **`CLOSED-…` → `TESTING-…`** before verification; after **PASS**, restored **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: fail/block → `WIP-…`, not operator-mentioned `TESTED-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task body lists top-level `src/`; Rust lives under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Spot-check (acceptance #2–3):** `finalize_turn_timeout` in `turn_lifecycle.rs` includes `**Turn timed out**` and `**{}s**` budget; `turn_lifecycle.rs` warns include `turn wall-clock timeout` and `closing output gate and running cleanup`; `ollama.rs` logs `closing output gate after turn wall-clock timeout`.
- **Acceptance criteria:** All satisfied.
- **Outcome:** **PASS** — final filename: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-28 UTC (`003-tester/TESTER.md`, operator path `UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** renamed **`CLOSED-…` → `TESTING-…`** before verification; after **PASS**, restored **`TESTING-…` → `CLOSED-…`**. (On fail, operator asked for `TESTED-…`; `003-tester/TESTER.md` specifies `WIP-…` — not applicable.)
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task verification block lists `src/`; Rust is under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget; `turn wall-clock timeout` / `closing output gate and running cleanup` in `turn_lifecycle.rs`; `closing output gate after turn wall-clock timeout` in `ollama.rs`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-28 UTC (`003-tester/TESTER.md`, operator path `UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** renamed **`CLOSED-…` → `TESTING-…`** before verification; after **PASS**, restored **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: fail/block → `WIP-…`; operator message mentioned `TESTED-…` on fail — repo procedure uses `WIP-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/ src-tauri/src/` — matches only under `src-tauri/src/` (`ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`); top-level `src/` (JS) has no matches
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and `**{}s**` budget in `turn_lifecycle.rs`; warns include `turn wall-clock timeout` / `closing output gate and running cleanup`; router `closing output gate after turn wall-clock timeout` in `ollama.rs`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-28 UTC (`003-tester/TESTER.md`, operator path `UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** renamed **`CLOSED-…` → `TESTING-…`** before verification; after **PASS**, restored **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: fail/block → `WIP-…`; el operador mencionó `TESTED-…` en fallo — el procedimiento del repo usa `WIP-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (bloque de verificación de la tarea apunta a `src/`; el Rust está en `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and `**{}s**` budget; `turn wall-clock timeout` / `closing output gate and running cleanup` in `turn_lifecycle.rs`; `closing output gate after turn wall-clock timeout` in `ollama.rs`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-28 UTC (`003-tester/TESTER.md`, operator path `UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** renamed **`CLOSED-…` → `TESTING-…`** before verification; after **PASS**, restored **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: fail/block → `WIP-…`; the operator message mentioned `TESTED-…` on fail — repo procedure uses `WIP-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task verification lists top-level `src/`; Rust is under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget seconds; `turn wall-clock timeout` / `closing output gate and running cleanup` in `turn_lifecycle.rs`; `closing output gate after turn wall-clock timeout` in `ollama.rs`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-28 UTC (`003-tester/TESTER.md`, operator path `UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** renamed **`CLOSED-…` → `TESTING-…`** at the start of this run; after **PASS**, restored **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: fail/block → `WIP-…`; el mensaje del operador mencionaba `TESTED-…` en fallo — el procedimiento del repo usa `WIP-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (el bloque de la tarea cita `src/`; el Rust está en `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — coincidencias en `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** Todas satisfechas (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` con `**Turn timed out**` y presupuesto en segundos; cadenas de log en `turn_lifecycle.rs` y `ollama.rs`; `cargo check` / `cargo test` en verde).
- **Outcome:** **PASS** — nombre final tras este run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-28 22:46 UTC (`003-tester/TESTER.md`, operator path `UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** renamed **`CLOSED-…` → `TESTING-…`** at the start of this run; after **PASS**, restored **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: fail/block → `WIP-…`; the operator message mentioned `TESTED-…` on fail — repo procedure uses `WIP-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task body lists top-level `src/`; Rust is under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget seconds; `turn wall-clock timeout` / `closing output gate and running cleanup` in `turn_lifecycle.rs`; `closing output gate after turn wall-clock timeout` in `ollama.rs`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-28 UTC (`003-tester/TESTER.md`, únicamente `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`)

- **Renombre UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` **no existía**; no se usó ningún otro `UNTESTED-*`. Fase TESTING: el archivo se renombró **`CLOSED-…` → `TESTING-…`** antes de la verificación; tras **PASS** se restauró **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: en fallo/bloqueo → `WIP-…`; el operador mencionó `TESTED-…` en fallo — el repo usa `WIP-…`.)
- **Comandos ejecutados:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (crate biblioteca: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — sin coincidencias (la tarea cita `src/` a nivel raíz; el Rust está en `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — coincidencias en `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Criterios de aceptación:** Todos satisfechos (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` con prefijo `**Turn timed out**` y presupuesto en segundos; cadenas de log en `turn_lifecycle.rs` y `ollama.rs`; `cargo check` / `cargo test` en verde).
- **Resultado:** **PASS** — nombre final tras este run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29 UTC (`003-tester/TESTER.md`, operator path `UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** renamed **`CLOSED-…` → `TESTING-…`** at the start of this run; after **PASS**, restored **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: fail/block → `WIP-…`; the operator message mentioned `TESTED-…` on fail — repo procedure uses `WIP-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task body lists top-level `src/`; Rust is under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Spot-check (acceptance #2–3):** `finalize_turn_timeout` in `turn_lifecycle.rs` includes `**Turn timed out**` and budget `**{}s**`; warns include `turn wall-clock timeout` and `closing output gate and running cleanup`; `ollama.rs` logs `closing output gate after turn wall-clock timeout`.
- **Acceptance criteria:** All satisfied.
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29 UTC, second run (`003-tester/TESTER.md`, operator path `UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** renamed **`CLOSED-…` → `TESTING-…`** at the start of this run; after **PASS**, restored **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: fail/block → `WIP-…`; el operador pidió `TESTED-…` en fallo — el procedimiento del repo usa `WIP-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task body lists top-level `src/`; Rust is under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget seconds; log strings in `turn_lifecycle.rs` and `ollama.rs`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29 (`003-tester/TESTER.md`, objetivo operador `UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` únicamente)

- **Fecha:** 2026-03-29 (local del entorno; hora UTC no registrada por separado).
- **Renombre UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` **no existía**; no se usó ningún otro `UNTESTED-*`. Fase TESTING: el archivo canónico se renombró **`CLOSED-…` → `TESTING-…`**, se añadió este informe y, al **PASS**, se restaura **`TESTING-…` → `CLOSED-…`**. (En fallo, el operador pidió `TESTED-…`; `003-tester/TESTER.md` indica `WIP-…` — no aplica.)
- **Comandos ejecutados:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (crate biblioteca: **871** passed, 0 failed)
  - Búsqueda de patrones (equivalente al `rg` del cuerpo de la tarea): coincidencias en `src-tauri/src/commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs` para `TurnOutputGate`, `finalize_turn_timeout` y `closing output gate after turn wall-clock`; el bloque de la tarea cita `rg … src/` en la raíz — en `src/` (JS) no hay esas cadenas Rust.
- **Criterios de aceptación:** Cumplidos (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` con `**Turn timed out**` y presupuesto en segundos; avisos `turn wall-clock timeout` / `closing output gate and running cleanup` en `turn_lifecycle.rs`; router con `closing output gate after turn wall-clock timeout` en `ollama.rs`; `cargo check` / `cargo test` en verde).
- **Resultado:** **PASS** — nombre final tras este run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29 UTC (`003-tester/TESTER.md`, operator path `UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** renamed **`CLOSED-…` → `TESTING-…`** at the start of this run; after **PASS**, restored **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: fail/block → `WIP-…`; the operator message mentioned `TESTED-…` on fail — repo procedure uses `WIP-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task body lists top-level `src/`; Rust is under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget seconds; log strings in `turn_lifecycle.rs` and `ollama.rs`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T00:47:45Z UTC (`003-tester/TESTER.md`, operator path `UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** canonical task file renamed **`CLOSED-…` → `TESTING-…`** for this run; on **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (On failure, operator asked for `TESTED-…`; `003-tester/TESTER.md` specifies `WIP-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task verification block lists top-level `src/`; Rust lives under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget seconds; router / turn-lifecycle log strings in source; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29 UTC (`003-tester/TESTER.md`, operator path `UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** renamed **`CLOSED-…` → `TESTING-…`** at the start of this run; after **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (On failure, operator asked for `TESTED-…`; this run **passed** → **`CLOSED-…`**.)
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task body lists top-level `src/`; Rust is under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget `**{}s**`; `turn wall-clock timeout` / `closing output gate and running cleanup` in `turn_lifecycle.rs`; `closing output gate after turn wall-clock timeout` in `ollama.rs`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29 (Cursor agent run, UTC) (`003-tester/TESTER.md`, operator path `UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date:** 2026-03-29 (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** renamed **`CLOSED-…` → `TESTING-…`** at the start of this run; after **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: fail/block → `WIP-…`; operator wording `TESTED-…` on fail is superseded by repo `TESTER.md`.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task body lists top-level `src/`; Rust is under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget seconds; router / turn-lifecycle log strings in source; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29 UTC (`003-tester/TESTER.md`, objetivo operador `UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` únicamente)

- **Fecha:** 2026-03-29 (UTC).
- **Renombre UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` **no existía**; no se usó ningún otro `UNTESTED-*`. Fase TESTING: el archivo canónico estaba como **`CLOSED-…`** y se renombró a **`TESTING-…`** para este run; tras **PASS** se restaura **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: en fallo/bloqueo → `WIP-…`; el operador mencionó `TESTED-…` en fallo — el procedimiento del repo usa `WIP-…`.)
- **Comandos ejecutados:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (crate biblioteca: **871** passed, 0 failed)
  - Búsqueda en workspace (`TurnOutputGate`, `finalize_turn_timeout`, `closing output gate after turn wall-clock`): coincidencias en `src-tauri/src/commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`; el bloque de la tarea lista `rg … src/` en la raíz — en `src/` (JS) no hay esas cadenas Rust.
- **Criterios de aceptación:** Cumplidos (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` con `**Turn timed out**` y presupuesto en segundos; logs en `turn_lifecycle.rs` y `ollama.rs`; `cargo check` / `cargo test` en verde).
- **Resultado:** **PASS** — nombre final tras este run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29 UTC (`003-tester/TESTER.md`, operator target `UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date:** 2026-03-29 (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. The task file on disk was **`CLOSED-…`**; it was renamed **`CLOSED-…` → `TESTING-…`** for this run, then **`TESTING-…` → `CLOSED-…`** after **PASS**. (On failure, operator asked for **`TESTED-…`**; `003-tester/TESTER.md` specifies **`WIP-…`** — not applicable.)
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches, exit 1 (task verification lists top-level `src/`; Rust is under `src-tauri/src/`)
  - Workspace grep for the same patterns: matches in `src-tauri/src/commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` text starts `**Turn timed out**` with budget `**{}s**`; `turn wall-clock timeout` / `closing output gate and running cleanup` in `turn_lifecycle.rs`; router `closing output gate after turn wall-clock timeout` in `ollama.rs`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29 UTC (`003-tester/TESTER.md`, únicamente `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`; run del agente Cursor)

- **Fecha:** 2026-03-29 (UTC).
- **Renombre UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` **no existía**; no se eligió otro `UNTESTED-*`. Fase TESTING: **`CLOSED-…` → `TESTING-…`** al inicio de este run; informe añadido mientras el archivo era `TESTING-…`; tras **PASS**, **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: fallo/bloqueo → `WIP-…`, no `TESTED-…`.)
- **Comandos ejecutados:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (crate biblioteca: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — sin coincidencias (la tarea cita `src/` en la raíz; Rust en `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — coincidencias en `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Criterios de aceptación:** Cumplidos.
- **Resultado:** **PASS** — nombre final: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29 UTC (`003-tester/TESTER.md`, operator target `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date:** 2026-03-29 (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; the canonical file was `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`. No other `UNTESTED-*` file was used. **TESTING phase:** renamed **`CLOSED-…` → `TESTING-…`** at the start of this run; this report was appended while the filename was **`TESTING-…`**; after **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: fail/block → **`WIP-…`**; the operator message mentioned **`TESTED-…`** on fail — repo procedure uses **`WIP-…`**.)
- **Commands run:**
  - `cd src-tauri && cargo check` — pass
  - `cd src-tauri && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task verification lists top-level `src/`; Rust is under `src-tauri/src/`)
  - Workspace search (same patterns): matches in `src-tauri/src/commands/tool_loop.rs`, `ollama.rs`, `turn_lifecycle.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget; router / turn-lifecycle log strings in source; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29 UTC (`003-tester/TESTER.md`, objetivo operador `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` únicamente; run agente)

- **Fecha:** 2026-03-29 (UTC).
- **Renombre UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` **no existía**; no se usó ningún otro `UNTESTED-*`. Fase TESTING: **`CLOSED-…` → `TESTING-…`** al inicio; informe añadido con el archivo en `TESTING-…`; tras **PASS**, **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: fallo/bloqueo → `WIP-…`; el operador pidió `TESTED-…` en fallo — el procedimiento del repo usa `WIP-…`.)
- **Comandos ejecutados:**
  - `cd src-tauri && cargo check && cargo test` — pass (crate biblioteca: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — coincidencias en `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
  - El bloque de verificación de la tarea cita `rg … src/` en la raíz; el Rust vive en `src-tauri/src/` (no se exige coincidencias en `src/` JS).
- **Criterios de aceptación:** Cumplidos.
- **Resultado:** **PASS** — nombre final tras este run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29 UTC (`003-tester/TESTER.md`, operator path `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date:** 2026-03-29 (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** canonical file renamed **`CLOSED-…` → `TESTING-…`** for this run; this subsection appended while the filename was **`TESTING-…`**; after **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: fail/block → **`WIP-…`**; operator message **`TESTED-…`** on fail is superseded by repo procedure.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task body lists top-level `src/`; Rust is under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget seconds; router / turn-lifecycle log strings in source; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29 UTC (`003-tester/TESTER.md`, objetivo operador `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` únicamente; run agente Cursor)

- **Fecha:** 2026-03-29 (UTC).
- **Renombre UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` **no existía**; no se eligió ningún otro `UNTESTED-*`. Fase TESTING: el archivo canónico se renombró **`CLOSED-…` → `TESTING-…`** antes de ejecutar comandos; este bloque se añadió con el nombre **`TESTING-…`**. Tras **PASS**, se restaurará **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: en fallo/bloqueo → `WIP-…`; el operador mencionó `TESTED-…` en fallo — el procedimiento del repo usa `WIP-…`.)
- **Comandos ejecutados:**
  - `cd src-tauri && cargo check && cargo test` — pass (crate biblioteca: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/ src-tauri/src/` — coincidencias solo en `src-tauri/src/` (`commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`); en `src/` (JS) no hay esas cadenas Rust
- **Criterios de aceptación:** Cumplidos (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` con `**Turn timed out**` y presupuesto en segundos; logs en `turn_lifecycle.rs` y `ollama.rs`; `cargo check` / `cargo test` en verde).
- **Resultado:** **PASS** — nombre final tras este run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29 UTC (`003-tester/TESTER.md`, objetivo operador `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` únicamente)

- **Fecha:** 2026-03-29 (UTC).
- **Renombre UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` **no existía**; no se usó ningún otro `UNTESTED-*`. Fase TESTING: el archivo canónico se renombró **`CLOSED-…` → `TESTING-…`** al inicio de este run; este bloque se añadió con el nombre **`TESTING-…`**. Tras **PASS**, **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: en fallo/bloqueo → `WIP-…`; el operador pidió `TESTED-…` en fallo — el procedimiento del repo usa `WIP-…`.)
- **Comandos ejecutados:**
  - `cd src-tauri && cargo check && cargo test` — pass (crate biblioteca: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — sin coincidencias (el bloque de la tarea cita `src/` en la raíz; el Rust está en `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — coincidencias en `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Criterios de aceptación:** Cumplidos (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` con texto que empieza por `**Turn timed out**` y presupuesto en segundos; `turn wall-clock timeout` / `closing output gate and running cleanup` en `turn_lifecycle.rs`; router `closing output gate after turn wall-clock timeout` en `ollama.rs`; `cargo check` / `cargo test` en verde).
- **Resultado:** **PASS** — nombre final tras este run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T05:01:46Z UTC (`003-tester/TESTER.md`, operator target `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date:** 2026-03-29T05:01:46Z (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-…`** for this run; this subsection was appended while the filename was **`TESTING-…`**. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (On failure, operator asked for **`TESTED-…`**; `003-tester/TESTER.md` specifies **`WIP-…`**.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (library crate: **871** passed, 0 failed)
  - Pattern search (task verification): matches in `src-tauri/src/commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs` for `TurnOutputGate`, `finalize_turn_timeout`, `closing output gate after turn wall-clock`; task body lists `rg … src/` at repo root — top-level `src/` (JS) has no Rust matches.
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` text starts `**Turn timed out**` with budget `**{}s**`; `turn wall-clock timeout` / `closing output gate and running cleanup` in `turn_lifecycle.rs`; router `closing output gate after turn wall-clock timeout` in `ollama.rs`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29 UTC (`003-tester/TESTER.md`, operator path `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date:** 2026-03-29 (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-…`** before verification; this subsection was appended while the filename was **`TESTING-…`**. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: fail/block → **`WIP-…`**; operator wording **`TESTED-…`** on fail is superseded by repo procedure.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task body lists top-level `src/`; Rust is under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget seconds; router / turn-lifecycle log strings in source; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T05:29:17Z UTC (`003-tester/TESTER.md`, operator path `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date:** 2026-03-29T05:29:17Z (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-…`** before verification; this subsection was appended while the filename was **`TESTING-…`**. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: fail/block → **`WIP-…`**; operator wording **`TESTED-…`** on fail is superseded by repo procedure.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task body lists top-level `src/`; Rust is under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget seconds; `turn wall-clock timeout` / `closing output gate and running cleanup` in `turn_lifecycle.rs`; router `closing output gate after turn wall-clock timeout` in `ollama.rs`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29 (UTC, agent run) (`003-tester/TESTER.md`, objetivo `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` únicamente)

- **Fecha:** 2026-03-29 (UTC).
- **Renombre UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` **no existía** en el árbol; no se usó ningún otro `UNTESTED-*`. Fase TESTING: **`CLOSED-…` → `TESTING-…`** al inicio de este run; este bloque se añade con el archivo en **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`**. Tras **PASS**, **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: en fallo/bloqueo → **`WIP-…`**; el operador mencionó **`TESTED-…`** en fallo — el procedimiento del repo usa **`WIP-…`**.)
- **Comandos ejecutados:**
  - `cd src-tauri && cargo check && cargo test` — pass (crate biblioteca: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — sin coincidencias (el bloque de verificación de la tarea cita `src/` en la raíz; el Rust está en `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — coincidencias en `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Criterios de aceptación:** Cumplidos (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` con prefijo `**Turn timed out**` y presupuesto en segundos; logs en `turn_lifecycle.rs` y `ollama.rs`; `cargo check` / `cargo test` en verde).
- **Resultado:** **PASS** — nombre final tras este run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T06:09:07Z UTC (`003-tester/TESTER.md`, operator path `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date:** 2026-03-29T06:09:07Z (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** before verification; this subsection was appended while the filename was **`TESTING-…`**. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (On fail, operator asked for **`TESTED-…`**; `003-tester/TESTER.md` specifies **`WIP-…`** — not applicable.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/tool_loop.rs`, `commands/ollama.rs`, `commands/turn_lifecycle.rs`
  - Task body also lists `rg … src/`; top-level `src/` has no Rust matches (implementation under `src-tauri/src/`).
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send` in `tool_loop.rs`; `finalize_turn_timeout` with `**Turn timed out**` and `**{}s**` budget in `turn_lifecycle.rs`; warns include `turn wall-clock timeout` / `closing output gate and running cleanup`; router string `closing output gate after turn wall-clock timeout` in `ollama.rs`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29 UTC (`003-tester/TESTER.md`, operator target `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date:** 2026-03-29 (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** before verification; this subsection was appended while the filename was **`TESTING-…`**. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (On failure, operator asked for **`TESTED-…`**; `003-tester/TESTER.md` specifies **`WIP-…`** — not applicable.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task verification lists top-level `src/`; Rust is under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget; router / turn-lifecycle log strings in source; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T06:54:36Z UTC (`003-tester/TESTER.md`, operator path `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date:** 2026-03-29T06:54:36Z (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** before verification; this subsection was appended while the filename was **`TESTING-…`**. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (On failure, operator asked for **`TESTED-…`**; `003-tester/TESTER.md` specifies **`WIP-…`** — not applicable because this run passed.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task verification lists top-level `src/`; Rust is under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/turn_lifecycle.rs`, `ollama.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget seconds; router / turn-lifecycle log strings in source; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` (operator fail naming `TESTED-…` not used).

### Re-verify — 2026-03-29T07:21:50Z UTC (`003-tester/TESTER.md`, operator path `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date:** 2026-03-29T07:21:50Z (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** before verification; this subsection was appended while the filename was **`TESTING-…`**. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: fail/block → **`WIP-…`**; the operator message mentioned **`TESTED-…`** on fail — repo procedure uses **`WIP-…`**.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task verification lists top-level `src/`; Rust is under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget seconds; `turn wall-clock timeout` / `closing output gate and running cleanup` in `turn_lifecycle.rs`; `closing output gate after turn wall-clock timeout` in `ollama.rs`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T07:35:16Z UTC (`003-tester/TESTER.md`, operator path `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date:** 2026-03-29T07:35:16Z (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** before verification; this subsection was appended while the filename was **`TESTING-…`**. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: fail/block → **`WIP-…`**; the operator message mentioned **`TESTED-…`** on fail — repo procedure uses **`WIP-…`**.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task body lists top-level `src/`; Rust lives under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and `**{}s**` budget; `turn wall-clock timeout` / `closing output gate and running cleanup` in `turn_lifecycle.rs`; `closing output gate after turn wall-clock timeout` in `ollama.rs`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T07:48:22Z UTC (`003-tester/TESTER.md`, operator path `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date:** 2026-03-29T07:48:22Z (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** before verification; this subsection was appended while the filename was **`TESTING-…`**. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: fail/block → **`WIP-…`**; the operator message mentioned **`TESTED-…`** on fail — repo procedure uses **`WIP-…`**, not `TESTED-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task verification lists top-level `src/`; Rust is under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget seconds; router / turn-lifecycle log strings in source; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T08:01:27Z UTC (`003-tester/TESTER.md`, operator path `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date:** 2026-03-29T08:01:27Z (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** before verification; this subsection was appended while the filename was **`TESTING-…`**. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: fail/block → **`WIP-…`**; the operator message mentioned **`TESTED-…`** on fail — repo procedure uses **`WIP-…`**, not `TESTED-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task verification lists top-level `src/`; Rust is under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget seconds; `turn wall-clock timeout` / `closing output gate and running cleanup` in `turn_lifecycle.rs`; `closing output gate after turn wall-clock timeout` in `ollama.rs`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T08:17:56Z UTC (`003-tester/TESTER.md`, operator path `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date:** 2026-03-29T08:17:56Z (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** before verification; this subsection was appended while the filename was **`TESTING-…`**. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: fail/block → **`WIP-…`**; the operator message mentioned **`TESTED-…`** on fail — repo procedure uses **`WIP-…`**, not `TESTED-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (library crate: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task verification lists top-level `src/`; Rust is under `src-tauri/src/`)
  - Workspace search (same patterns): matches in `src-tauri/src/commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget seconds; router / turn-lifecycle log strings in source; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29 (`003-tester/TESTER.md`, operator path `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date:** 2026-03-29 (UTC, approximate at run completion).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** before verification; this subsection was appended while the filename was **`TESTING-…`**. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: fail/block → **`WIP-…`**; el operador pidió **`TESTED-…`** en fallo — el repo usa **`WIP-…`**, no `TESTED-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (`cargo check` ok; library tests: **871** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (el cuerpo del task lista `src/` de nivel superior; el Rust está en `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — coincidencias en `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** Todas cumplidas (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` con `**Turn timed out**` y presupuesto en segundos; cadenas de log en `turn_lifecycle.rs` y `ollama.rs`; `cargo check` / `cargo test` en verde).
- **Outcome:** **PASS** — nombre final tras este run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T08:52:17Z UTC (`003-tester/TESTER.md`, operator path `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date:** 2026-03-29T08:52:17Z (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** before verification; this subsection was appended while the filename was **`TESTING-…`**. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: fail/block → **`WIP-…`**; the operator message mentioned **`TESTED-…`** on fail — repo procedure uses **`WIP-…`**, not `TESTED-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (library crate: **872** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task verification lists top-level `src/`; Rust is under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget seconds; router / turn-lifecycle log strings in source; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29 (`003-tester/TESTER.md`, operator path `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date:** 2026-03-29 (UTC, approximate at run completion).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** before verification; this subsection was appended while the filename was **`TESTING-…`**. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: fail/block → **`WIP-…`**; the operator message mentioned **`TESTED-…`** on fail — repo procedure uses **`WIP-…`**, not `TESTED-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (`cargo check` ok; library tests: **872** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task body lists top-level `src/`; Rust is under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget seconds; `turn wall-clock timeout` / `closing output gate and running cleanup` in `turn_lifecycle.rs`; `closing output gate after turn wall-clock timeout` in `ollama.rs`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T09:23:38Z UTC (`003-tester/TESTER.md`, operator path `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date:** 2026-03-29T09:23:38Z (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** before verification; this subsection was appended while the filename was **`TESTING-…`**. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: fail/block → **`WIP-…`**; the operator message mentioned **`TESTED-…`** on fail — repo procedure uses **`WIP-…`**, not `TESTED-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (`cargo check` ok; library crate: **872** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task body lists top-level `src/`; Rust is under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget seconds; `turn wall-clock timeout` / `closing output gate and running cleanup` in `turn_lifecycle.rs`; `closing output gate after turn wall-clock timeout` in `ollama.rs`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T09:40:55Z UTC (`003-tester/TESTER.md`, objetivo operador `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` únicamente)

- **Date:** 2026-03-29T09:40:55Z (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` **no existía** en el árbol; no se usó ningún otro `UNTESTED-*`. Fase TESTING: el archivo canónico se renombró **`CLOSED-…` → `TESTING-…`** antes de la verificación; este bloque se añadió con el nombre **`TESTING-…`**. Tras **PASS**, **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: en fallo/bloqueo → **`WIP-…`**; el operador pidió **`TESTED-…`** en fallo — el procedimiento del repo sigue **`WIP-…`**, no `TESTED-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (`cargo check` ok; crate biblioteca: **872** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — sin coincidencias (el bloque de verificación de la tarea cita `src/` en la raíz; el Rust está en `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — coincidencias en `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** Cumplidos (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` con prefijo `**Turn timed out**` y presupuesto en segundos; cadenas de log en `turn_lifecycle.rs` y `ollama.rs`; `cargo check` / `cargo test` en verde).
- **Outcome:** **PASS** — nombre final tras este run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T09:57:43Z UTC (`003-tester/TESTER.md`, operator path `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date:** 2026-03-29T09:57:43Z (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** before verification; this subsection was appended while the filename was **`TESTING-…`**. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: fail/block → **`WIP-…`**; the operator message asked for **`TESTED-…`** on fail — not applicable because this run **passed**.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (library crate: **872** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/ src-tauri/src/` — matches only under `src-tauri/src/` (`commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`); top-level `src/` (JS) has no matches
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget seconds; router / turn-lifecycle log strings in source; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T10:10:54Z UTC (`003-tester/TESTER.md`, objetivo operador `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` únicamente)

- **Fecha:** 2026-03-29T10:10:54Z (UTC).
- **Renombre UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` **no existía** en el árbol; no se usó ningún otro `UNTESTED-*`. Fase TESTING: el archivo canónico se renombró **`CLOSED-…` → `TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** antes de ejecutar la verificación; este bloque se añade con el nombre **`TESTING-…`**. Tras **PASS**, se restaura **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md`: en fallo/bloqueo → `WIP-…`; el operador de este run pidió `TESTED-…` en fallo — no aplica.)
- **Comandos ejecutados:**
  - `cd src-tauri && cargo check && cargo test` — pass (`cargo check` ok; crate biblioteca: **872** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — sin coincidencias, código de salida 1 (el bloque de la tarea cita `src/` en la raíz; el Rust está en `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — coincidencias en `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Criterios de aceptación:** Cumplidos (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` con prefijo `**Turn timed out**` y presupuesto en segundos; cadenas de log en `turn_lifecycle.rs` y `ollama.rs`; `cargo check` / `cargo test` en verde).
- **Resultado:** **PASS** — nombre final tras este run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T18:05:44Z UTC (`003-tester/TESTER.md`, operator target `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date:** 2026-03-29T18:05:44Z (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** before verification; this subsection was appended while the filename was **`TESTING-…`**. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. Outcome naming for this operator run: pass → **`CLOSED-…`**; implementation fail → **`TESTED-…`**; defective task instructions / environment spec → **`TESTPLAN-…`** (`003-tester/TESTER.md` still documents **`WIP-…`** for blocked/failed runs in-repo).
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (`cargo check` ok; library crate: **874** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches, exit code 1 (task verification block lists top-level `src/`; Rust strings live under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` with `**Turn timed out**` and budget seconds; router / turn-lifecycle log strings in source; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T18:12:41Z UTC (`003-tester/TESTER.md`, operator target `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date:** 2026-03-29T18:12:41Z (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** before verification; this subsection was appended under **`TESTING-…`**. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (Operator outcomes: pass → `CLOSED-…`; implementation fail → `TESTED-…`; defective instructions / environment spec → `TESTPLAN-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (`cargo check` ok; library crate: **874** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches, exit code 1 (task body lists top-level `src/`; Rust implementation is under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` as `Arc<AtomicBool>`; `gate_allows_send` on status/draft paths in `tool_loop.rs`; `finalize_turn_timeout` returns text starting `**Turn timed out**` with budget seconds in `turn_lifecycle.rs`; router string `closing output gate after turn wall-clock timeout` in `ollama.rs`; turn-lifecycle warn `turn wall-clock timeout` / `closing output gate and running cleanup`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T18:21:36Z UTC (`003-tester/TESTER.md`, objetivo operador `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` únicamente)

- **Fecha:** 2026-03-29T18:21:36Z (UTC).
- **Renombre UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` **no existía**; no se usó ningún otro `UNTESTED-*`. Fase TESTING: `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` se renombró a **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** antes de la verificación; este bloque se añadió con el archivo en **`TESTING-…`**. Tras **PASS**, se restaura **`TESTING-…` → `CLOSED-…`**. (Criterio del operador: pass → `CLOSED-…`; fallo de implementación → `TESTED-…`; instrucciones de prueba / spec de entorno defectuosas → `TESTPLAN-…`; `003-tester/TESTER.md` en repo sigue indicando `WIP-…` para bloqueos genéricos.)
- **Comandos ejecutados:**
  - `cd src-tauri && cargo check && cargo test` — pass (`cargo check` ok; crate biblioteca: **874** passed, 0 failed)
  - Búsqueda equivalente a `rg … src/` (workspace `src/`): sin coincidencias (JS; el Rust está en `src-tauri/src/`)
  - `rg` en `src-tauri/src` para `closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout` — coincidencias en `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Criterios de aceptación:** Cumplidos (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` con `**Turn timed out**` y presupuesto; cadenas de log en `turn_lifecycle.rs` y `ollama.rs`; `cargo check` / `cargo test` en verde).
- **Resultado:** **PASS** — nombre final tras este run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T18:30:17Z UTC (`003-tester/TESTER.md`, objetivo operador `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` únicamente)

- **Fecha / hora:** 2026-03-29T18:30:17Z (UTC).
- **Renombre UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` **no existía** en el árbol; no se usó ningún otro `UNTESTED-*`. Fase TESTING: `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` se renombró a **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** antes de la verificación; este bloque se añadió con el archivo en **`TESTING-…`**. Tras **PASS**, se restaura **`TESTING-…` → `CLOSED-…`**. (Criterio del operador: pass → `CLOSED-…`; fallo de implementación → `TESTED-…`; instrucciones de prueba / spec de entorno defectuosas → `TESTPLAN-…`; `003-tester/TESTER.md` en repo indica `WIP-…` para bloqueos genéricos.)
- **Comandos ejecutados:**
  - `cd src-tauri && cargo check && cargo test` — pass (`cargo check` ok; crate biblioteca: **874** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — sin coincidencias, código de salida 1 (el bloque de verificación de la tarea cita `src/` en la raíz; el Rust está en `src-tauri/src/`)
  - `rg` equivalente en `src-tauri/src` (mismo patrón) — coincidencias en `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Criterios de aceptación:** Cumplidos (`TurnOutputGate` como `Arc<AtomicBool>`; `gate_allows_send` en rutas de estado/borrador en `tool_loop.rs`; `finalize_turn_timeout` con texto que empieza por `**Turn timed out**` y presupuesto en segundos en `turn_lifecycle.rs`; cadena del router `closing output gate after turn wall-clock timeout` en `ollama.rs`; avisos en `turn_lifecycle.rs` con `turn wall-clock timeout` / `closing output gate and running cleanup`; `cargo check` / `cargo test` en verde).
- **Resultado:** **PASS** — nombre final tras este run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T18:37:25Z UTC (`003-tester/TESTER.md`, operator target `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date / time:** 2026-03-29T18:37:25Z (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** before verification; this subsection was appended while the file was **`TESTING-…`**. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (Operator outcomes: pass → `CLOSED-…`; implementation fail → `TESTED-…`; defective instructions / environment spec → `TESTPLAN-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (`cargo check` ok; library crate: **874** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches, exit code 1 (task verification block lists top-level `src/`; Rust sources are under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` as `Arc<AtomicBool>`; `gate_allows_send` on status/draft paths in `tool_loop.rs`; `finalize_turn_timeout` returns text starting `**Turn timed out**` with budget seconds in `turn_lifecycle.rs`; router string `closing output gate after turn wall-clock timeout` in `ollama.rs`; turn-lifecycle warns include `turn wall-clock timeout` / `closing output gate and running cleanup`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T18:46:05Z UTC (`003-tester/TESTER.md`, objetivo operador `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` únicamente)

- **Fecha / hora:** 2026-03-29T18:46:05Z (UTC).
- **Renombre UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` **no existía**; no se usó ningún otro `UNTESTED-*`. Fase TESTING: `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` → **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** antes de verificar; este bloque se añadió con el archivo en **`TESTING-…`**. Tras **PASS**, **`TESTING-…` → `CLOSED-…`**. (Criterio del operador: pass → `CLOSED-…`; fallo de implementación → `TESTED-…`; instrucciones / spec de entorno defectuosas → `TESTPLAN-…`.)
- **Comandos ejecutados:**
  - `cd src-tauri && cargo check && cargo test` — pass (`cargo check` ok; crate biblioteca: **874** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — sin coincidencias, código de salida 1 (la tarea cita `src/` en la raíz; el Rust está en `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — coincidencias en `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Criterios de aceptación:** Cumplidos (`TurnOutputGate` / `gate_allows_send`; `finalize_turn_timeout` con `**Turn timed out**` y presupuesto; log del router y aviso en `turn_lifecycle.rs` con `turn wall-clock timeout` / `closing output gate and running cleanup`; `cargo check` / `cargo test` en verde).
- **Resultado:** **PASS** — nombre final tras este run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T18:53:32Z UTC (`003-tester/TESTER.md`, operator target `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date / time:** 2026-03-29T18:53:32Z (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** before verification; this subsection was appended while the file was **`TESTING-…`**. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (Operator outcomes: pass → `CLOSED-…`; implementation fail → `TESTED-…`; defective test instructions / environment spec → `TESTPLAN-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (`cargo check` ok; library crate: **874** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches, exit code 1 (task verification block lists top-level `src/`; Rust lives under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` as `Arc<AtomicBool>`; `gate_allows_send` respected in `tool_loop.rs`; `finalize_turn_timeout` text starts `**Turn timed out**` with budget seconds; router log string and turn-lifecycle warns with `turn wall-clock timeout` / `closing output gate and running cleanup`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T19:01:57Z UTC (`003-tester/TESTER.md`, operator target `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date / time:** 2026-03-29T19:01:57Z (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** before verification; this subsection was appended while the file was **`TESTING-…`**. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (Operator outcomes for this run: pass → `CLOSED-…`; implementation fail → `TESTED-…`; defective test instructions / environment spec → `TESTPLAN-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (`cargo check` ok; library crate: **874** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches, exit code 1 (task verification block lists top-level `src/`; Rust lives under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` as `Arc<AtomicBool>`; `gate_allows_send` on status/draft paths in `tool_loop.rs`; `finalize_turn_timeout` returns text starting `**Turn timed out**` with budget seconds in `turn_lifecycle.rs`; router string `closing output gate after turn wall-clock timeout` in `ollama.rs`; turn-lifecycle warns include `turn wall-clock timeout` / `closing output gate and running cleanup`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T19:08:52Z UTC (`003-tester/TESTER.md`, operator target `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date / time:** 2026-03-29T19:08:52Z (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** before verification; this subsection was appended while the file was **`TESTING-…`**. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (Operator outcomes: pass → `CLOSED-…`; implementation fail → `TESTED-…`; defective test instructions / environment spec → `TESTPLAN-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (`cargo check` ok; library crate: **874** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches, exit code 1 (task verification block lists top-level `src/`; Rust lives under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` as `Arc<AtomicBool>`; `gate_allows_send` on status/draft paths in `tool_loop.rs`; `finalize_turn_timeout` returns text starting `**Turn timed out**` with budget seconds in `turn_lifecycle.rs`; router string `closing output gate after turn wall-clock timeout` in `ollama.rs`; turn-lifecycle warns include `turn wall-clock timeout` / `closing output gate and running cleanup`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T19:16:02Z UTC (`003-tester/TESTER.md`, operator target `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date / time:** 2026-03-29T19:16:02Z (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** before verification; this subsection was appended while the file was **`TESTING-…`**. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (Operator outcomes: pass → `CLOSED-…`; implementation fail → `TESTED-…`; defective test instructions / environment spec → `TESTPLAN-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (`cargo check` ok; library crate: **874** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches, exit code 1 (task verification block lists top-level `src/`; Rust lives under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` as `Arc<AtomicBool>`; `gate_allows_send` on status/draft paths in `tool_loop.rs`; `finalize_turn_timeout` returns text starting `**Turn timed out**` with budget seconds in `turn_lifecycle.rs`; router string `closing output gate after turn wall-clock timeout` in `ollama.rs`; turn-lifecycle warns include `turn wall-clock timeout` / `closing output gate and running cleanup`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T19:23:15Z UTC (`003-tester/TESTER.md`, operator target `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date / time:** 2026-03-29T19:23:15Z (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** before appending this subsection. (Operator outcomes: pass → `CLOSED-…`; implementation fail → `TESTED-…`; defective test instructions / environment spec → `TESTPLAN-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (`cargo check` ok; library crate: **874** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task verification block lists top-level `src/`; Rust lives under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` as `Arc<AtomicBool>`; `gate_allows_send` on status/draft paths in `tool_loop.rs`; `finalize_turn_timeout` returns text starting `**Turn timed out**` with budget seconds in `turn_lifecycle.rs`; router string `closing output gate after turn wall-clock timeout` in `ollama.rs`; turn-lifecycle warns include `turn wall-clock timeout` / `closing output gate and running cleanup`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — restored filename: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T19:32:13Z UTC (`003-tester/TESTER.md`, operator target `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date / time:** 2026-03-29T19:32:13Z (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** before verification; this subsection was appended while the file was **`TESTING-…`**. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (Operator outcomes for this run: pass → `CLOSED-…`; implementation fail → `TESTED-…`; defective test instructions / environment spec → `TESTPLAN-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (`cargo check` ok; library crate: **874** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches (task verification block lists top-level `src/`; Rust lives under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` as `Arc<AtomicBool>`; `gate_allows_send` on status/draft paths in `tool_loop.rs`; `finalize_turn_timeout` returns text starting `**Turn timed out**` with budget seconds in `turn_lifecycle.rs`; router string `closing output gate after turn wall-clock timeout` in `ollama.rs`; turn-lifecycle warns include `turn wall-clock timeout` / `closing output gate and running cleanup`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T19:40:45Z UTC (`003-tester/TESTER.md`, operator target `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date / time:** 2026-03-29T19:40:45Z (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** before verification; this subsection was appended while the file was **`TESTING-…`**. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (Operator outcomes for this run: pass → `CLOSED-…`; implementation fail → `TESTED-…`; defective test instructions / environment spec → `TESTPLAN-…`.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (`cargo check` ok; library crate: **874** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches, exit code 1 (task verification block lists top-level `src/`; Rust lives under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` as `Arc<AtomicBool>`; `gate_allows_send` on status/draft paths in `tool_loop.rs`; `finalize_turn_timeout` returns text starting `**Turn timed out**` with budget seconds in `turn_lifecycle.rs`; router string `closing output gate after turn wall-clock timeout` in `ollama.rs`; turn-lifecycle warns include `turn wall-clock timeout` / `closing output gate and running cleanup`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T19:47:40Z UTC (`003-tester/TESTER.md`, operator target `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date / time:** 2026-03-29T19:47:40Z (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** before verification; this subsection was appended while the file was **`TESTING-…`**. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (`003-tester/TESTER.md` uses `WIP-…` on fail/block; operator naming for implementation fail / bad test plan: `TESTED-…` / `TESTPLAN-…` — not applicable.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (`cargo check` ok; library crate: **874** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches, exit code **1** (task verification block lists top-level `src/`; Rust lives under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` as `Arc<AtomicBool>`; `gate_allows_send` on status/draft paths in `tool_loop.rs`; `finalize_turn_timeout` returns text starting `**Turn timed out**` with budget seconds in `turn_lifecycle.rs`; router string `closing output gate after turn wall-clock timeout` in `ollama.rs`; turn-lifecycle warns include `turn wall-clock timeout` / `closing output gate and running cleanup`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T19:56:13Z UTC (`003-tester/TESTER.md`, operator target `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date / time:** 2026-03-29T19:56:13Z (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** before verification; this subsection was appended while the file was **`TESTING-…`**. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (Operator outcomes for this run: pass → `CLOSED-…`; implementation fail → `TESTED-…`; defective test instructions / environment spec → `TESTPLAN-…`. `003-tester/TESTER.md` also names `WIP-…` on fail/block.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (`cargo check` ok; library crate: **874** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches, exit code **1** (task verification block lists top-level `src/`; Rust lives under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` as `Arc<AtomicBool>`; `gate_allows_send` on status/draft paths in `tool_loop.rs`; `finalize_turn_timeout` returns text starting `**Turn timed out**` with budget seconds in `turn_lifecycle.rs`; router string `closing output gate after turn wall-clock timeout` in `ollama.rs`; turn-lifecycle warns include `turn wall-clock timeout` / `closing output gate and running cleanup`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T20:05:01Z UTC (`003-tester/TESTER.md`, operator target `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date / time:** 2026-03-29T20:05:01Z (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** before verification; this subsection was appended while the file had the **`TESTING-…`** prefix. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (Operator outcome naming for this run: pass → `CLOSED-…`; implementation fail → `TESTED-…`; defective testing instructions / environment spec → `TESTPLAN-…`. `003-tester/TESTER.md` also lists `WIP-…` on fail/block.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (`cargo check` ok; library crate: **874** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches, exit code **1** (task verification block lists top-level `src/`; Rust lives under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` as `Arc<AtomicBool>`; `gate_allows_send` on status/draft paths in `tool_loop.rs`; `finalize_turn_timeout` returns text starting `**Turn timed out**` with budget seconds in `turn_lifecycle.rs`; router string `closing output gate after turn wall-clock timeout` in `ollama.rs`; turn-lifecycle warns include `turn wall-clock timeout` / `closing output gate and running cleanup`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.

### Re-verify — 2026-03-29T20:13:57Z UTC (`003-tester/TESTER.md`, operator target `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only)

- **Date / time:** 2026-03-29T20:13:57Z (UTC).
- **Rename UNTESTED → TESTING:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was **not present**; no other `UNTESTED-*` file was used. **TESTING phase:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` was renamed to **`TESTING-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** before verification; this subsection was appended while the file had the **`TESTING-…`** prefix. On **PASS**, restore **`TESTING-…` → `CLOSED-…`**. (Operator outcome naming: pass → `CLOSED-…`; implementation fail → `TESTED-…`; defective testing instructions / environment spec → `TESTPLAN-…`. `003-tester/TESTER.md` also lists `WIP-…` on fail/block.)
- **Commands run:**
  - `cd src-tauri && cargo check && cargo test` — pass (`cargo check` ok; library crate: **874** passed, 0 failed)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src/` — no matches, exit code **1** (task verification block lists top-level `src/`; Rust lives under `src-tauri/src/`)
  - `rg -n "closing output gate after turn wall-clock|TurnOutputGate|finalize_turn_timeout" src-tauri/src` — matches in `commands/ollama.rs`, `turn_lifecycle.rs`, `tool_loop.rs`
- **Acceptance criteria:** All satisfied (`TurnOutputGate` as `Arc<AtomicBool>`; `gate_allows_send` on status/draft paths in `tool_loop.rs`; `finalize_turn_timeout` returns text starting `**Turn timed out**` with budget seconds in `turn_lifecycle.rs`; router string `closing output gate after turn wall-clock timeout` in `ollama.rs`; turn-lifecycle warns include `turn wall-clock timeout` / `closing output gate and running cleanup`; `cargo check` / `cargo test` green).
- **Outcome:** **PASS** — final filename after this run: `CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.
