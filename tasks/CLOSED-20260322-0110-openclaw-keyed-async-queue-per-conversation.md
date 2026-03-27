# CLOSED — OpenClaw-style keyed async queue per conversation (2026-03-22)

## Goal

Verify **per-conversation (per-channel) serialization** of full Discord/OpenClaw router turns via an async keyed mutex queue, so concurrent messages on the same channel do not interleave tool loops or session updates; different keys still run in parallel. Ollama HTTP remains additionally keyed via `ollama_queue_key`.

## References

- `src-tauri/src/keyed_queue.rs` — `run_serial`, `is_key_busy`, unit tests (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `src-tauri/src/discord/mod.rs` — `run_serial` around router path; `ollama_queue_key: Some(format!("discord:{}", channel_id_u64))`
- `src-tauri/src/commands/ollama.rs` — `ollama_queue_key` on `OllamaChatRequest`
- `src-tauri/src/ollama_queue.rs` — FIFO per key for `/api/chat`

## Acceptance criteria

1. **Build:** `cargo check` in `src-tauri/` succeeds.
2. **Tests:** `cargo test` in `src-tauri/` succeeds (includes `keyed_queue` module tests).
3. **Static verification:** `keyed_queue::run_serial` and `ollama_queue_key` with `discord:` remain wired from `discord/mod.rs` (`rg` spot-check).

## Verification commands

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test keyed_queue
cd src-tauri && cargo test
```

Optional spot-check:

```bash
rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs
```

## Test report

**Date:** 2026-03-27, hora local; zona horaria del entorno del operador (se asume coherente con el reloj del sistema donde se ejecutó `cargo`).

**Preflight:** `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` **no estaba** en el disco al inicio del run. Se escribió el cuerpo de la tarea en esa ruta y se renombró a `TESTING-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` según `003-tester/TESTER.md`. No se usó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (2 tests: `same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — coincidencias en líneas 1143, 1347, 1934 (`run_serial`) y 2310 (`ollama_queue_key` con `discord:{}`).

**Outcome:** Criterios de aceptación cumplidos en esta corrida automatizada. No se probó Discord en vivo contra un gateway real.

## Test report (corrida adicional — agente Cursor)

**Date:** 2026-03-27, hora local del entorno donde se ejecutó `cargo` (misma convención que el informe anterior).

**Preflight / nombres:** El operador indicó explícitamente `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md`. Ese archivo **no está** en el repositorio; la tarea correspondiente es `tasks/CLOSED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md`. No había ningún `UNTESTED-*` en `tasks/`, por lo que **no se aplicó** el renombrado UNTESTED→TESTING de `003-tester/TESTER.md` en este run. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — coincidencias en 1143, 1347, 1934 (`run_serial`) y 2310 (`ollama_queue_key` con `discord:{}`).

**Outcome:** Criterios de aceptación cumplidos de nuevo. El nombre del archivo permanece **CLOSED-** (no WIP). Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-27)

**Date:** 2026-03-27, hora local del entorno donde se ejecutó `cargo` (zona horaria del sistema del operador).

**Preflight / nombres:** El operador pidió probar `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md`. En el repo solo existía `CLOSED-20260322-0110-…`; se aplicó el flujo `003-tester/TESTER.md` renombrando en cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para cumplir el paso UNTESTED→TESTING sin tocar ningún otro `UNTESTED-*`. El encabezado del documento quedó en estado **TESTING** durante la verificación.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — coincidencias en 1143, 1347, 1934 (`run_serial`) y 2310 (`ollama_queue_key` con `discord:{}`).

**Outcome:** Todos los criterios de aceptación cumplidos. Archivo renombrado a **CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor)

**Date:** 2026-03-27, hora local del entorno donde se ejecutó `cargo` (zona horaria del sistema del operador).

**Preflight / nombres:** El operador indicó `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md`. En el repo solo existía `CLOSED-20260322-0110-…`; para cumplir `003-tester/TESTER.md` (UNTESTED→TESTING sin elegir otro `UNTESTED-*`) se renombró en cadena **CLOSED → UNTESTED → TESTING** con el mismo basename. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — coincidencias en 1143, 1347, 1934 (`run_serial`) y 2310 (`ollama_queue_key` con `discord:{}`).

**Outcome:** Criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-27)

**Date:** 2026-03-27, hora local del entorno donde se ejecutó `cargo` (zona horaria del sistema del operador).

**Preflight / nombres:** El operador indicó `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md`. Ese path **no existe** en el working tree; la tarea ya está en `tasks/CLOSED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md`. Por tanto **no se pudo** aplicar el paso literal de `003-tester/TESTER.md` «renombrar UNTESTED→TESTING» sin inventar otra copia del archivo. No se renombró a `TESTING-` ni a `WIP-`; no se tocó ningún otro `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — coincidencias en 1143, 1347, 1934 (`run_serial`) y 2310 (`ollama_queue_key` con `discord:{}`).

**Outcome:** Criterios de aceptación cumplidos. El nombre del archivo permanece **CLOSED-** (no WIP). Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-27)

**Date:** 2026-03-27, hora local del sistema donde se ejecutó `cargo` (zona horaria del entorno del operador).

**Preflight / nombres:** El operador indicó `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md`. En el repo solo existía `CLOSED-20260322-0110-…`; para cumplir `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename), luego **TESTING → CLOSED** tras el informe. No se usó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — líneas 1143, 1347, 1934 (`run_serial`) y 2310 (`ollama_queue_key` con `discord:{}`).

**Outcome:** Todos los criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.
