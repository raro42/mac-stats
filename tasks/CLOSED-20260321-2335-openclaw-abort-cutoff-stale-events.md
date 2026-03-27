# CLOSED — OpenClaw-style abort cutoff / stale inbound events (2026-03-21)

## Goal

Verify session-scoped **abort cutoff**: after a turn aborts (e.g. wall-clock timeout), inbound work whose event time is stale vs the recorded cutoff is dropped so Discord retries and scheduler runs **due before** the abort do not start a new router turn (OpenClaw-style).

## References

- `src-tauri/src/commands/abort_cutoff.rs` — `record_cutoff`, `clear_cutoff`, `should_skip`, `InboundStaleGuard`, unit tests for stale comparison
- `src-tauri/src/commands/turn_lifecycle.rs` — `record_cutoff` on abort
- `src-tauri/src/commands/ollama.rs` — `should_skip` + `OllamaRunError::StaleInboundAfterAbort`
- `src-tauri/src/commands/ollama_run_error.rs` — `StaleInboundAfterAbort` variant
- `src-tauri/src/discord/mod.rs` — `clear_cutoff`, `should_skip`, stale handling after router
- `src-tauri/src/scheduler/mod.rs` — scheduler Ollama stale inbound path
- `src-tauri/src/scheduler/heartbeat.rs` — heartbeat Ollama stale inbound path
- `src-tauri/src/commands/ollama_frontend_chat.rs` — CPU chat `clear_cutoff` / `should_skip`

## Acceptance criteria

1. **Build:** `cargo check` in `src-tauri/` succeeds.
2. **Tests:** `cargo test` in `src-tauri/` succeeds (including `abort_cutoff` unit tests).
3. **Static verification:** `record_cutoff`, `should_skip`, and `StaleInboundAfterAbort` remain wired in Discord, scheduler, heartbeat, and `ollama.rs` (`rg` spot-check).

## Verification commands

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test abort_cutoff
cd src-tauri && cargo test
```

Optional spot-check:

```bash
rg -n "abort_cutoff::|StaleInboundAfterAbort" src-tauri/src/discord/mod.rs src-tauri/src/scheduler/mod.rs src-tauri/src/scheduler/heartbeat.rs src-tauri/src/commands/ollama.rs
```

## Test report

**Date:** 2026-03-27 (local operator environment), noted in UTC terms as 2026-03-27 for the run timestamp.

**Preflight:** `tasks/UNTESTED-20260321-2335-openclaw-abort-cutoff-stale-events.md` was not present on disk at the start of the run. The task body was written as that path, then renamed to `TESTING-20260321-2335-openclaw-abort-cutoff-stale-events.md` per `003-tester/TESTER.md`. No other `UNTESTED-*` file was used.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test abort_cutoff` — **pass** (4 tests in `commands::abort_cutoff::tests`)
- `cd src-tauri && cargo test` — **pass** (854 tests in `mac_stats` library; 0 failed; 1 doc-test ignored)

**Static spot-check**

- `rg -n "abort_cutoff::|StaleInboundAfterAbort" …` on `discord/mod.rs`, `scheduler/mod.rs`, `scheduler/heartbeat.rs`, `commands/ollama.rs` — matches for `clear_cutoff`, `should_skip`, `InboundStaleGuard`, and `StaleInboundAfterAbort` as expected.

**Outcome:** All acceptance criteria satisfied. Live Discord/scheduler abort and retry ordering against a real Ollama instance was not exercised in this automated run.

## Test report

**Date:** 2026-03-27 (local operator environment; this Cursor tester run).

**Rename note:** `tasks/UNTESTED-20260321-2335-openclaw-abort-cutoff-stale-events.md` was not present on disk. The task file exists only as `CLOSED-20260321-2335-openclaw-abort-cutoff-stale-events.md`, so the `UNTESTED → TESTING` rename from `003-tester/TESTER.md` could not be performed without inventing a duplicate path. Verification was run against this `CLOSED-*` file only; no other `UNTESTED-*` task was used.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test abort_cutoff` — **pass** (4 tests in `commands::abort_cutoff::tests`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed; 1 doc-test ignored)

**Static spot-check**

- `rg -n "abort_cutoff::|StaleInboundAfterAbort"` on `discord/mod.rs`, `scheduler/mod.rs`, `scheduler/heartbeat.rs`, `commands/ollama.rs` — **pass** (matches for `clear_cutoff`, `should_skip`, `InboundStaleGuard`, `StaleInboundAfterAbort`).

**Outcome:** All acceptance criteria satisfied. Filename remains `CLOSED-20260321-2335-openclaw-abort-cutoff-stale-events.md`. Live Discord/scheduler abort ordering against Ollama was not exercised here.

## Test report

**Date:** 2026-03-27 (local operator environment, macOS).

**Rename chain:** At run start, `tasks/UNTESTED-20260321-2335-openclaw-abort-cutoff-stale-events.md` was not on disk (only `CLOSED-*` existed). Per `003-tester/TESTER.md`, the file was renamed `CLOSED-*` → `UNTESTED-*` (header updated) → `TESTING-*` so the `UNTESTED → TESTING` step could be applied without touching any other `UNTESTED-*` task.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test abort_cutoff` — **pass** (4 tests in `commands::abort_cutoff::tests`, also covered in full suite)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed; 1 doc-test ignored)

**Static spot-check**

- `rg -n "abort_cutoff::|StaleInboundAfterAbort"` on `discord/mod.rs`, `scheduler/mod.rs`, `scheduler/heartbeat.rs`, `commands/ollama.rs` — **pass** (`clear_cutoff`, `should_skip`, `InboundStaleGuard`, `StaleInboundAfterAbort` present as expected).

**Outcome:** All acceptance criteria satisfied. Renamed `TESTING-*` → `CLOSED-*`. Live Discord/scheduler abort and retry ordering against a real Ollama instance was not exercised in this run.

## Test report

**Date:** 2026-03-27 (local macOS operator environment).

**Preflight / rename:** `tasks/UNTESTED-20260321-2335-openclaw-abort-cutoff-stale-events.md` was not present. Only `CLOSED-*` existed for this task id; it was renamed to `TESTING-*` (header updated) to follow `003-tester/TESTER.md` without selecting any other `UNTESTED-*` file.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test abort_cutoff` — **pass** (4 tests in `commands::abort_cutoff::tests`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed in library crate; 1 doc-test ignored)

**Static spot-check**

- `rg -n "abort_cutoff::|StaleInboundAfterAbort"` on `discord/mod.rs`, `scheduler/mod.rs`, `scheduler/heartbeat.rs`, `commands/ollama.rs` — **pass** (`clear_cutoff`, `should_skip`, `InboundStaleGuard`, `StaleInboundAfterAbort` present as expected).

**Outcome:** All acceptance criteria satisfied. File renamed `TESTING-*` → `CLOSED-*` for this run. Live Discord/scheduler abort and retry ordering against Ollama was not exercised here.

## Test report

**Date:** 2026-03-27 (local America/Los_Angeles; wall-clock date stated explicitly).

**Rename:** `tasks/UNTESTED-20260321-2335-openclaw-abort-cutoff-stale-events.md` was not present. This task existed as `CLOSED-*`; it was renamed to `TESTING-*` for this run only (no other `UNTESTED-*` file was used), per operator instruction to test this task id.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test abort_cutoff` — **pass** (4 tests in `commands::abort_cutoff::tests`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed in `mac_stats` lib; 1 doc-test ignored)

**Static spot-check**

- `rg -n "abort_cutoff::|StaleInboundAfterAbort"` on `discord/mod.rs`, `scheduler/mod.rs`, `scheduler/heartbeat.rs`, `commands/ollama.rs` — **pass** (`clear_cutoff`, `should_skip`, `InboundStaleGuard`, `StaleInboundAfterAbort` present).

**Outcome:** All acceptance criteria satisfied. Renamed `TESTING-*` → `CLOSED-*` after this report. Live Discord/scheduler abort and retry ordering against a real Ollama instance was not exercised.

## Test report

**Date:** 2026-03-27 (local environment; wall-clock date as in user context).

**Preflight / rename:** At run start, `tasks/UNTESTED-20260321-2335-openclaw-abort-cutoff-stale-events.md` was not on disk (only `CLOSED-*` existed). To apply `003-tester/TESTER.md` step 2 (`UNTESTED → TESTING`) on this task id only, `CLOSED-*` was renamed to `UNTESTED-*` (header updated), then immediately to `TESTING-*`. No other `UNTESTED-*` task file was used.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test abort_cutoff` — **pass** (4 tests in `commands::abort_cutoff::tests`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed in `mac_stats` lib; 1 doc-test ignored)

**Static spot-check**

- `rg -n "abort_cutoff::|StaleInboundAfterAbort"` on `discord/mod.rs`, `scheduler/mod.rs`, `scheduler/heartbeat.rs`, `commands/ollama.rs` — **pass** (`clear_cutoff`, `should_skip`, `InboundStaleGuard`, `StaleInboundAfterAbort` present as expected).

**Outcome:** All acceptance criteria satisfied. Renaming `TESTING-*` → `CLOSED-*` after this report. Live Discord/scheduler abort and retry ordering against a real Ollama instance was not exercised in this run.

## Test report

**Date:** 2026-03-27 (local operator environment, America/Los_Angeles wall-clock; timestamps in prose are local unless noted).

**Rename (`003-tester/TESTER.md`):** At run start the task existed only as `CLOSED-20260321-2335-openclaw-abort-cutoff-stale-events.md` (no `UNTESTED-*` file for this id). To follow step 2 on this task only: `CLOSED-*` → `UNTESTED-*` (header) → `TESTING-*` (header). No other `UNTESTED-*` task was touched.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test abort_cutoff` — **pass** (4 tests in `commands::abort_cutoff::tests`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed in `mac_stats` library crate; 1 doc-test ignored)

**Static spot-check**

- `rg` for `abort_cutoff::` / `StaleInboundAfterAbort` in `src-tauri/src/discord/mod.rs`, `scheduler/mod.rs`, `scheduler/heartbeat.rs`, `commands/ollama.rs` — **pass** (`clear_cutoff`, `should_skip`, `InboundStaleGuard`, `StaleInboundAfterAbort` present).

**Outcome:** All acceptance criteria satisfied. File renamed `TESTING-*` → `CLOSED-*` after this report. Live Discord/scheduler abort and retry ordering against a real Ollama instance was not exercised in this run.

## Test report

**Date:** 2026-03-28 (local wall-clock, America/Los_Angeles; prose dates are local unless noted).

**Rename (`003-tester/TESTER.md`):** El archivo solicitado `tasks/UNTESTED-20260321-2335-openclaw-abort-cutoff-stale-events.md` no existía en disco (solo `CLOSED-*`). Para cumplir el paso UNTESTED→TESTING sin tocar otro `UNTESTED-*`: `CLOSED-*` → `UNTESTED-*` (cabecera) → `TESTING-*` (cabecera); tras este informe → `CLOSED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test abort_cutoff` — **pass** (4 tests en `commands::abort_cutoff::tests`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en crate `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg` de `abort_cutoff::` / `StaleInboundAfterAbort` en `discord/mod.rs`, `scheduler/mod.rs`, `scheduler/heartbeat.rs`, `commands/ollama.rs` — **pass** (`clear_cutoff`, `should_skip`, `InboundStaleGuard`, `StaleInboundAfterAbort` presentes).

**Outcome:** Criterios de aceptación cumplidos → `CLOSED-*`. No se probó en vivo orden abort/retry Discord u Ollama.

