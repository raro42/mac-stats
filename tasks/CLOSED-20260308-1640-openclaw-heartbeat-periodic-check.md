# OpenClaw heartbeat — periodic check on app Tokio runtime

## Goal

Periodic agent heartbeat (OpenClaw-style checklist + `HEARTBEAT_OK`) must run on the **same** Tokio runtime as the rest of mac-stats Tauri async work (`tauri::async_runtime::spawn`), so `tokio::time` intervals and Ollama queue waits behave correctly. CDP tab health (`check_browser_alive`) must **not** nest `Handle::block_on` on that runtime (current-thread executor would wedge; timers never fire).

## Acceptance criteria

- `spawn_heartbeat_thread` uses `tauri::async_runtime::spawn` and awaits `heartbeat_loop()` inside the spawned future.
- App startup invokes `scheduler::heartbeat::spawn_heartbeat_thread()` after the Ollama warmup gate (ordering with Discord/scheduler).
- CDP liveness path documents/implements avoidance of nested `block_on` + `tokio::time::timeout` on the shared runtime (blocking eval on a plain `std::thread`).
- `cargo check` and `cargo test scheduler::heartbeat` succeed.

## Verification commands

```bash
rg 'spawn_heartbeat_thread|async_runtime::spawn|heartbeat_loop' src-tauri/src/lib.rs src-tauri/src/scheduler/heartbeat.rs
rg 'block_on|check_browser_alive|evaluate_one_plus_one_blocking_timeout' src-tauri/src/browser_agent/mod.rs | head
cd src-tauri && cargo check && cargo test scheduler::heartbeat --no-fail-fast
```

## Test report

**Date:** 2026-03-27 20:08 UTC

**Preflight:** El fichero `tasks/UNTESTED-20260308-1640-openclaw-heartbeat-periodic-check.md` no existía en el árbol del repo; se creó con el alcance inferido del comentario en `src-tauri/src/scheduler/heartbeat.rs` (líneas 297–301) y del comentario en `browser_agent` sobre `block_on`, y se aplicó el flujo TESTER (UNTESTED → TESTING → este informe → CLOSED).

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test scheduler::heartbeat --no-fail-fast` — **pass** (5 tests: `is_heartbeat_ack` cases)
- `rg 'spawn_heartbeat_thread|async_runtime::spawn|heartbeat_loop' src-tauri/src/lib.rs src-tauri/src/scheduler/heartbeat.rs` — **pass** (`spawn_heartbeat_thread` → `tauri::async_runtime::spawn` + `heartbeat_loop().await`; `lib.rs` llama `scheduler::heartbeat::spawn_heartbeat_thread()`)
- `rg 'block_on|evaluate_one_plus_one_blocking_timeout' src-tauri/src/browser_agent/mod.rs | head` — **pass** (comentario explícito prohibiendo `Handle::block_on` + `tokio::time::timeout` en health check; uso de `evaluate_one_plus_one_blocking_timeout`)

**Outcome:** Todos los criterios de aceptación verificados — **CLOSED**.

---

### Run: 2026-03-27 20:43 UTC

**Preflight:** No existía `tasks/UNTESTED-20260308-1640-openclaw-heartbeat-periodic-check.md` en el árbol; el task ya estaba como `CLOSED-…`. Para seguir `003-tester/TESTER.md` se renombró `CLOSED-…` → `TESTING-…`, se volvieron a ejecutar las verificaciones y se cierra de nuevo como `CLOSED-…`.

**Commands run**

- `rg 'spawn_heartbeat_thread|async_runtime::spawn|heartbeat_loop' src-tauri/src/lib.rs src-tauri/src/scheduler/heartbeat.rs` — **pass** (`spawn_heartbeat_thread` usa `tauri::async_runtime::spawn` y `heartbeat_loop().await`; `lib.rs` invoca `scheduler::heartbeat::spawn_heartbeat_thread()` tras el gate de warmup de Ollama en el mismo bloque async).
- `rg 'block_on|check_browser_alive|evaluate_one_plus_one_blocking_timeout' src-tauri/src/browser_agent/mod.rs | head` — **pass** (comentarios y `evaluate_one_plus_one_blocking_timeout`; sin `Handle::block_on` anidado en el health check).
- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test scheduler::heartbeat --no-fail-fast` — **pass** (5 tests)

**Outcome:** Criterios de aceptación cumplidos — **CLOSED**.
