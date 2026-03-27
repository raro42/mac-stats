# CLOSED — OpenClaw-style partial progress on timeout (2026-03-21)

## Goal

Verify that when router/Ollama work hits a **wall-clock or transport timeout**, mac-stats still surfaces **partial progress** (tool steps + last assistant snippet) to Discord replies, scheduler task failures, and heartbeat logs—mirroring OpenClaw-style visibility.

## References

- `src-tauri/src/commands/partial_progress.rs` — `PartialProgressCapture`, `format_user_summary` (`try_lock` for timeout safety), unit test `format_summary_lists_tools_and_snippet`
- `src-tauri/src/commands/ollama_run_error.rs` — `should_attach_partial_progress`, unit tests `should_attach_partial_progress_*`
- `src-tauri/src/discord/mod.rs` — `partial_progress` + `should_attach_partial_progress` on router errors
- `src-tauri/src/scheduler/mod.rs` — `tokio::time::timeout` + `partial.format_user_summary()` on task timeout
- `src-tauri/src/scheduler/heartbeat.rs` — heartbeat Ollama timeout logs partial progress
- `src-tauri/src/commands/tool_loop.rs` — records into `partial_progress_capture`

## Acceptance criteria

1. **Build:** `cargo check` in `src-tauri/` succeeds.
2. **Tests:** `cargo test` in `src-tauri/` succeeds (including `partial_progress` and `ollama_run_error` partial-progress tests).
3. **Static verification:** Timeout paths still call `format_user_summary` / attach partial progress where documented above (`rg` spot-check).

## Verification commands

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test
```

Optional spot-check:

```bash
rg -n "format_user_summary|should_attach_partial_progress|PartialProgressCapture::new" src-tauri/src/discord/mod.rs src-tauri/src/scheduler/mod.rs src-tauri/src/scheduler/heartbeat.rs
```

## Test report

**Date:** 2026-03-27 (local operator environment).

**Preflight:** The path `tasks/UNTESTED-20260321-1800-openclaw-partial-progress-on-timeout.md` was not present in the workspace at the start of this run; the task body was materialized as `UNTESTED-…`, then renamed to `TESTING-…` per `003-tester/TESTER.md` before verification. No other `UNTESTED-*` file was used.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (854 tests in `mac_stats` library; 0 failed; 1 doc-test ignored)

**Static spot-check**

- `discord/mod.rs`: `PartialProgressCapture::new`, `should_attach_partial_progress` + `format_user_summary` on router error path.
- `scheduler/mod.rs`: `PartialProgressCapture::new` and `format_user_summary` after `tokio::time::timeout` for scheduled tasks.
- `scheduler/heartbeat.rs`: `format_user_summary` on heartbeat Ollama timeout path.

**Outcome:** All acceptance criteria satisfied. Live Discord/scheduler timeouts against a real Ollama instance were not exercised in this automated run.

### Test report — 2026-03-27 (UTC)

**Preflight:** `tasks/UNTESTED-20260321-1800-openclaw-partial-progress-on-timeout.md` was not in the tree; the task was already `CLOSED-…`. Renamed `CLOSED-…` → `TESTING-…` for this run per `003-tester/TESTER.md`. No other `UNTESTED-*` file was used.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (854 tests in `mac_stats` library; 0 failed; 1 doc-test ignored)

**Static spot-check (`rg`)**

- `discord/mod.rs`: `PartialProgressCapture::new`, `should_attach_partial_progress`, `format_user_summary` on router error path (lines ~2287, 2353–2354).
- `scheduler/mod.rs`: `PartialProgressCapture::new`, `format_user_summary` after timeout path (lines ~640, 654).
- `scheduler/heartbeat.rs`: `PartialProgressCapture::new`, `format_user_summary` on timeout path (lines ~136, 206).

**Outcome:** All acceptance criteria satisfied. Renamed `TESTING-…` → `CLOSED-…`.

### Test report — 2026-03-27 (UTC)

**Preflight:** `tasks/UNTESTED-20260321-1800-openclaw-partial-progress-on-timeout.md` was not in the tree (already `CLOSED-…`); renamed `CLOSED-…` → `TESTING-…` for this run per `003-tester/TESTER.md`. No other `UNTESTED-*` file was used.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (854 tests in `mac_stats` library; 0 failed; 1 doc-test ignored)

**Static spot-check (`rg`)**

- `discord/mod.rs`: `PartialProgressCapture::new` (L2287), `should_attach_partial_progress` + `format_user_summary` (L2353–2354).
- `scheduler/mod.rs`: `PartialProgressCapture::new` (L640), `format_user_summary` after timeout (L654).
- `scheduler/heartbeat.rs`: `PartialProgressCapture::new` (L136), `format_user_summary` on timeout (L206).

**Outcome:** All acceptance criteria satisfied. Live Discord/Ollama timeouts were not exercised in this run. Renamed `TESTING-…` → `CLOSED-…`.

### Test report — 2026-03-27 (local macOS operator environment)

**Preflight:** `tasks/UNTESTED-20260321-1800-openclaw-partial-progress-on-timeout.md` no estaba en el árbol (solo existe esta tarea con prefijo `CLOSED-…` / `TESTING-…`). Se renombró `CLOSED-20260321-1800-openclaw-partial-progress-on-timeout.md` → `TESTING-…` para ejecutar la verificación según `003-tester/TESTER.md`. No se usó ningún otro `UNTESTED-*`.

**Comandos**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (854 tests en la librería `mac_stats`; 0 fallidos; 1 doc-test ignorado)

**Comprobación estática (`rg`)**

- `discord/mod.rs`: `PartialProgressCapture::new` (L2287), `should_attach_partial_progress` + `format_user_summary` (L2353–L2354).
- `scheduler/mod.rs`: `PartialProgressCapture::new` (L640), `format_user_summary` tras timeout (L654).
- `scheduler/heartbeat.rs`: `PartialProgressCapture::new` (L136), `format_user_summary` en timeout (L206).

**Resultado:** Criterios de aceptación cumplidos. No se probaron timeouts reales contra Discord/Ollama. Renombrado `TESTING-…` → `CLOSED-…`.
