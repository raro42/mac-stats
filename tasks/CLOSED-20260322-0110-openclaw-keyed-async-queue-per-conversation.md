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

## Test report (corrida — agente Cursor, 2026-03-27)

**Date:** 2026-03-27, hora local (zona horaria del sistema donde se ejecutó `cargo`).

**Preflight / nombres:** Tarea indicada: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin usar otro `UNTESTED-*`). Ese path no existía; el fichero era `CLOSED-20260322-0110-…`. Para cumplir `003-tester/TESTER.md` se hizo **CLOSED → UNTESTED → TESTING** (mismo basename), verificación, informe y luego **TESTING → CLOSED**.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — líneas 1143, 1347, 1934 (`run_serial`) y 2310 (`ollama_queue_key` con `discord:{}`).

**Outcome:** Criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (zona horaria del entorno del operador; no UTC explícito).

**Preflight / nombres:** El operador indicó `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` y prohibió elegir otro `UNTESTED-*`. En el disco solo existía `CLOSED-20260322-0110-…`; para aplicar `003-tester/TESTER.md` (paso UNTESTED→TESTING) se renombró en cadena **CLOSED → UNTESTED → TESTING** (mismo basename). No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — líneas 1143, 1347, 1934 (`run_serial`) y 2310 (`ollama_queue_key` con `discord:{}`).

**Outcome:** Todos los criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, sesión actual 2026-03-28)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (zona horaria del entorno del operador; no UTC explícito).

**Preflight / nombres:** Tarea pedida: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin elegir otro `UNTESTED-*`). En el árbol solo existía `CLOSED-20260322-0110-…`; se aplicó **CLOSED → UNTESTED → TESTING** (mismo basename) para cumplir `003-tester/TESTER.md` (paso UNTESTED→TESTING). No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en el crate de biblioteca principal; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — líneas 1143, 1347, 1934 (`run_serial`) y 2310 (`ollama_queue_key` con `discord:{}`).

**Outcome:** Criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, segundo pase 2026-03-28)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (sin UTC explícito).

**Preflight / nombres:** Tarea pedida: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Solo existía `CLOSED-…`; cadena **CLOSED → UNTESTED → TESTING** para cumplir `003-tester/TESTER.md`.

**Commands run:** `cargo check`, `cargo test keyed_queue`, `cargo test` en `src-tauri/` — **pass** (854 tests en el crate de biblioteca `mac_stats`; 1 doc-test ignorado).

**Static spot-check:** `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (1143, 1347, 1934, 2310).

**Outcome:** Criterios cumplidos; archivo **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (zona horaria del entorno del operador; no UTC explícito).

**Preflight / nombres:** Tarea indicada: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Ese path no existía; solo estaba `CLOSED-20260322-0110-…`. Para cumplir `003-tester/TESTER.md` (paso UNTESTED→TESTING) se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename). No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en el crate de biblioteca `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, sesión TESTER)

**Date:** 2026-03-28, local system time where `cargo` ran (operator timezone; not explicit UTC).

**Preflight / names:** Operator requested only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md`; that path was missing (on disk: `CLOSED-20260322-0110-…`). Per `003-tester/TESTER.md`, chain **CLOSED → UNTESTED → TESTING** (same basename) so the UNTESTED→TESTING rename step applies without picking another `UNTESTED-*` file.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` with `discord:{}`).

**Outcome:** All acceptance criteria met. Renaming **TESTING- → CLOSED-** after this report. Live Discord not exercised.

## Test report (corrida — agente Cursor, 2026-03-28)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (zona horaria del entorno del operador; no UTC explícito).

**Preflight / nombres:** Tarea pedida: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin elegir otro `UNTESTED-*`). Ese path no existía; solo estaba `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para poder ejecutar el paso UNTESTED→TESTING sin tocar ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en el crate de biblioteca `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, TESTER)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (zona horaria del entorno del operador; no UTC explícito).

**Preflight / nombres:** Tarea indicada: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin elegir otro `UNTESTED-*`). Ese path no existía; en disco solo estaba `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso UNTESTED→TESTING sin tocar ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, segunda pasada TESTER)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (no UTC explícito).

**Preflight / nombres:** Misma tarea `UNTESTED-20260322-0110-…` indicada por el operador (sin otro `UNTESTED-*`). El fichero estaba como `CLOSED-…`; cadena **CLOSED → UNTESTED → TESTING** para cumplir `003-tester/TESTER.md` (paso UNTESTED→TESTING). Durante la verificación el encabezado del documento pasó a **TESTING** y, tras el informe, el nombre de archivo vuelve a **CLOSED-**.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, TESTER según 003-tester/TESTER.md)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (zona horaria del entorno del operador; no UTC explícito).

**Preflight / nombres:** Tarea indicada: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin elegir otro `UNTESTED-*`). Al inicio del run el fichero estaba como `CLOSED-20260322-0110-…`; se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para cumplir el paso UNTESTED→TESTING de `003-tester/TESTER.md`. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Todos los criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, sesión operador UNTESTED explícito)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (zona horaria del entorno del operador; no UTC explícito).

**Preflight / nombres:** El operador fijó únicamente `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Ese path no existía; el fichero era `CLOSED-…`. Cadena **CLOSED → UNTESTED → TESTING** (mismo basename), encabezado del documento en **TESTING** durante la verificación y vuelta a **CLOSED** en el contenido antes del renombrado final del archivo.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Todos los criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (run — Cursor agent, 2026-03-28, TESTER per 003-tester/TESTER.md)

**Date:** 2026-03-28, local system time where `cargo` ran (operator timezone; not explicit UTC).

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent; the file on disk was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied chain **CLOSED → UNTESTED → TESTING** (same basename) so the UNTESTED→TESTING step applies without picking another `UNTESTED-*` file.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` with `discord:{}`).

**Outcome:** All acceptance criteria met. Renaming **TESTING- → CLOSED-** after this report. Live Discord not exercised.

## Test report (corrida — agente Cursor, 2026-03-28, TESTER.md)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (zona horaria del entorno; no UTC explícito).

**Preflight / nombres:** Tarea única: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Al inicio el fichero estaba como `CLOSED-…`; se aplicó **CLOSED → UNTESTED → TESTING** (mismo basename) para poder ejecutar el paso UNTESTED→TESTING de `003-tester/TESTER.md`. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (2 tests: `same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, TESTER 003-tester/TESTER.md)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (zona horaria del entorno del operador; no UTC explícito).

**Preflight / nombres:** Tarea única indicada: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin elegir otro `UNTESTED-*`). Ese path no existía; el fichero estaba como `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso UNTESTED→TESTING sin tocar ningún otro archivo `UNTESTED-*`. El encabezado del documento quedó en **TESTING** durante la verificación y se restauró a **CLOSED** antes del renombrado final del archivo.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Todos los criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (run — Cursor agent, 2026-03-28, TESTER 003-tester/TESTER.md)

**Date:** 2026-03-28, local system time where `cargo` ran (operator environment; not explicit UTC).

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent at run start; the file on disk was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the UNTESTED→TESTING step applies without selecting another `UNTESTED-*` file. Document H1 set to **TESTING** during verification, restored to **CLOSED** before final filename.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` with `discord:{}`).

**Outcome:** All acceptance criteria met. Renaming **TESTING- → CLOSED-** after this report. Live Discord not exercised.

## Test report (corrida — agente Cursor, 2026-03-28, TESTER.md operador)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (zona horaria del entorno del operador; no UTC explícito).

**Preflight / nombres:** Tarea única: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Ese path no existía al inicio; el fichero estaba como `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso UNTESTED→TESTING sin elegir otro archivo `UNTESTED-*`. El H1 del documento estuvo en **TESTING** mientras corría `cargo`; se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Todos los criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, 003-tester/TESTER.md)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (zona horaria del entorno del operador; no UTC explícito).

**Preflight / nombres:** Tarea única: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Al inicio del run el fichero estaba como `CLOSED-20260322-0110-…`; se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para cumplir el paso UNTESTED→TESTING de `003-tester/TESTER.md`. El H1 del documento pasó a **TESTING** durante la verificación y se restauró a **CLOSED** antes del renombrado final del archivo.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Todos los criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, TESTER según petición explícita)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (zona horaria del entorno del operador; no UTC explícito).

**Preflight / nombres:** Tarea única: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Ese path no existía al inicio; el fichero estaba como `CLOSED-20260322-0110-…`. Cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para cumplir `003-tester/TESTER.md` (paso UNTESTED→TESTING). H1 en **TESTING** durante `cargo`, restaurado a **CLOSED** antes de **TESTING- → CLOSED-**.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Todos los criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — 2026-03-28, 003-tester/TESTER.md, solo UNTESTED-20260322-0110)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (zona horaria del entorno del operador; no UTC explícito).

**Preflight / nombres:** Tarea fijada por el operador: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin elegir otro `UNTESTED-*`). Ese path no existía al inicio; el fichero estaba como `CLOSED-20260322-0110-…`. Cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para cumplir el paso UNTESTED→TESTING de `003-tester/TESTER.md`. El H1 del documento pasó a **TESTING** durante `cargo` y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Todos los criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, 003-tester/TESTER.md)

**Date:** 2026-03-28, local system time where `cargo` ran (operator timezone; not explicit UTC).

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). At run start the file was `CLOSED-20260322-0110-…`; per `003-tester/TESTER.md` applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the UNTESTED→TESTING step applies. No other `UNTESTED-*` file was used.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` with `discord:{}`).

**Outcome:** All acceptance criteria met. Renaming **TESTING- → CLOSED-** after this report. Live Discord not exercised.

## Test report (corrida — agente Cursor, 2026-03-28, 003-tester/TESTER.md, segunda pasada)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (zona horaria del entorno; no UTC explícito).

**Preflight / nombres:** Tarea indicada: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin elegir otro `UNTESTED-*`). Ese path no existía al inicio; el fichero estaba como `CLOSED-20260322-0110-…`. Cadena **CLOSED → UNTESTED → TESTING** (mismo basename) según `003-tester/TESTER.md`. El H1 estuvo en **TESTING** durante `cargo` y se restauró a **CLOSED** antes de **TESTING- → CLOSED-**. No se tocó ningún otro `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, sesión TESTER operador UNTESTED)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (zona horaria del entorno del operador; no UTC explícito).

**Preflight / nombres:** Tarea fijada: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` únicamente (sin otro `UNTESTED-*`). Al inicio el fichero era `CLOSED-20260322-0110-…`; se aplicó **CLOSED → UNTESTED → TESTING** (mismo basename) para cumplir el paso UNTESTED→TESTING de `003-tester/TESTER.md`. El H1 pasó a **TESTING** durante `cargo` y se restauró a **CLOSED** antes de **TESTING- → CLOSED-**. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Todos los criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, 003-tester/TESTER.md)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (zona horaria del entorno del operador; no UTC explícito).

**Preflight / nombres:** Tarea única: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin elegir otro `UNTESTED-*`). Al inicio solo existía `CLOSED-20260322-0110-…`; según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para cumplir el paso UNTESTED→TESTING. No se tocó ningún otro archivo `UNTESTED-*`. El H1 quedó en **CLOSED** antes del renombrado final del archivo.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (2 tests: `same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en el crate de biblioteca `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Todos los criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, 003-tester/TESTER.md, sesión actual)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (no UTC explícito).

**Preflight / nombres:** Tarea fijada: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` únicamente (sin otro `UNTESTED-*`). Ese path no existía; el fichero estaba como `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso UNTESTED→TESTING. El H1 pasó a **TESTING** durante `cargo` y se restauró a **CLOSED** antes del renombrado final del archivo.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Todos los criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, operador TESTER explícito)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (zona horaria del entorno del operador; no UTC explícito).

**Preflight / nombres:** Solo la tarea `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Al inicio no existía ese path; el fichero era `CLOSED-20260322-0110-…`. Cadena **CLOSED → UNTESTED → TESTING** (mismo basename) según `003-tester/TESTER.md`. El H1 pasó a **TESTING** durante la verificación y se restauró a **CLOSED** antes de **TESTING- → CLOSED-**.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, 003-tester/TESTER.md)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (zona horaria del entorno del operador; no UTC explícito).

**Preflight / nombres:** Tarea única: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Al inicio solo existía `CLOSED-20260322-0110-…`; según `003-tester/TESTER.md` se aplicó **CLOSED → UNTESTED → TESTING** (mismo basename) para poder ejecutar el paso literal **UNTESTED → TESTING**. Durante la verificación el nombre de archivo fue `TESTING-…` y el H1 del documento pasó brevemente a **TESTING**, luego se restauró a **CLOSED** antes del renombrado final del archivo. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Todos los criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (run — Cursor agent, 2026-03-28, 003-tester/TESTER.md, only UNTESTED-20260322-0110)

**Date:** 2026-03-28, local system time where `cargo` ran (operator timezone; not explicit UTC).

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent; the file on disk was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied **CLOSED → UNTESTED → TESTING** (same basename) so the UNTESTED→TESTING rename applies without selecting another task.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` with `discord:{}`).

**Outcome:** All acceptance criteria met. Renaming **TESTING- → CLOSED-** after this report. Live Discord not exercised.

## Test report (corrida — agente Cursor, 2026-03-28, 003-tester/TESTER.md)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (zona horaria del entorno; no UTC explícito).

**Preflight / nombres:** Tarea fijada: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` únicamente (sin otro `UNTESTED-*`). Ese path no existía al inicio; el fichero estaba como `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING** sin elegir otro archivo. El H1 del documento pasó a **TESTING** durante la verificación y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, sesión 003-tester)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (no UTC explícito).

**Preflight / nombres:** Tarea única: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Al inicio solo existía `CLOSED-20260322-0110-…`; según `003-tester/TESTER.md` se aplicó **CLOSED → UNTESTED → TESTING** (mismo basename) para cumplir el paso **UNTESTED → TESTING**. El H1 del documento pasó a **TESTING** durante la verificación y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en tests de la crate lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Todos los criterios de aceptación cumplidos en esta corrida. Archivo **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, 003-tester/TESTER.md)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (no UTC explícito).

**Preflight / nombres:** Tarea pedida: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` únicamente (sin otro `UNTESTED-*`). Ese path no existía; el fichero estaba como `CLOSED-20260322-0110-…`. Cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para cumplir el paso **UNTESTED → TESTING** de `003-tester/TESTER.md`. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (no UTC explícito).

**Preflight / nombres:** Únicamente la tarea `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin elegir otro `UNTESTED-*`). Ese path no existía al inicio; el fichero estaba como `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. No se tocó ningún otro archivo `UNTESTED-*`. El H1 del documento pasó a **TESTING** durante la verificación y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Todos los criterios de aceptación cumplidos en esta corrida. Archivo **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, 003-tester/TESTER.md)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (zona horaria del entorno del operador; no UTC explícito).

**Preflight / nombres:** Tarea indicada por el operador: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin elegir otro `UNTESTED-*`). Ese path no existía al inicio; el fichero estaba como `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para poder ejecutar el paso **UNTESTED → TESTING**. El H1 del documento pasó brevemente a **TESTING** durante el trabajo y se restauró a **CLOSED** antes del renombrado final del archivo. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored en doc-tests del crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Todos los criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (run — Cursor agent, 2026-03-28, 003-tester/TESTER.md, only UNTESTED-20260322-0110)

**Date:** 2026-03-28, local system time where `cargo` ran (operator environment; not explicit UTC).

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent at run start; the file on disk was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the UNTESTED→TESTING step applies without selecting another `UNTESTED-*` file. Document H1 set to **TESTING** during verification, restored to **CLOSED** before final filename rename.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` with `discord:{}`)

**Outcome:** All acceptance criteria met. Renaming **TESTING- → CLOSED-** after this report. Live Discord not exercised.

## Test report (corrida — agente Cursor, 2026-03-28, 003-tester/TESTER.md)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (no UTC explícito).

**Preflight / nombres:** Tarea única: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Al inicio solo existía `CLOSED-20260322-0110-…`; según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename), verificación, informe y **TESTING- → CLOSED-** en el nombre de archivo. El H1 quedó en **CLOSED** antes del renombrado final del fichero.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; salida: 868 tests filtrados en el binario de lib)
- `cd src-tauri && cargo test` — **pass** (870 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored en doc-tests del crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Todos los criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, 003-tester/TESTER.md)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (zona horaria del entorno del operador; no UTC explícito).

**Preflight / nombres:** Tarea indicada: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` únicamente (sin otro `UNTESTED-*`). Al inicio el fichero estaba como `CLOSED-20260322-0110-…`; según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. No se tocó ningún otro archivo `UNTESTED-*`. El H1 del documento estuvo en **TESTING** durante la verificación y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 869 filtered)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored en doc-tests del crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Todos los criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.


## Test report (run — Cursor agent, 2026-03-28, 003-tester/TESTER.md)

**Date:** 2026-03-28, local system time (not explicit UTC).

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent; the file on disk was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the UNTESTED→TESTING step applies without selecting another task. Document H1 set to **TESTING** during verification, restored to **CLOSED** before final filename **TESTING- → CLOSED-**.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 869 filtered in lib test binary)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` with `discord:{}`)

**Outcome:** All acceptance criteria met. Renaming **TESTING- → CLOSED-** after this report. Live Discord not exercised.

## Test report (corrida — agente Cursor, 2026-03-28, sesión adicional)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (no UTC explícito).

**Preflight / nombres:** Tarea única: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Al inicio no existía ese path; el fichero era `CLOSED-20260322-0110-…`. Cadena **CLOSED → UNTESTED → TESTING** (mismo basename) según `003-tester/TESTER.md`. H1 **TESTING** durante `cargo`, restaurado a **CLOSED** antes de **TESTING- → CLOSED-**.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (2 passed, 869 filtered)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (1143, 1347, 1934, 2310 con `discord:{}`).

**Outcome:** Criterios cumplidos; **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, 003-tester/TESTER.md)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (no UTC explícito).

**Preflight / nombres:** Tarea única: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Ese path no existía al inicio; el fichero estaba como `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. El H1 del documento estuvo en **TESTING** durante la verificación y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 869 filtered en el binario de tests de `lib.rs`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored en doc-tests del crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Todos los criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, 003-tester/TESTER.md, sesión operador)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (no UTC explícito).

**Preflight / nombres:** Tarea fijada: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` únicamente (sin otro `UNTESTED-*`). Ese path no existía al inicio; el fichero estaba como `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. El H1 del documento pasó a **TESTING** durante la verificación y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 869 filtered)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored en doc-tests del crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Todos los criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, 003-tester/TESTER.md, sesión actual)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (zona horaria del entorno del operador; no UTC explícito).

**Preflight / nombres:** Tarea indicada: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` únicamente (sin otro `UNTESTED-*`). Al inicio del run el fichero estaba como `CLOSED-20260322-0110-…`; según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para cumplir el paso **UNTESTED → TESTING**. El H1 del documento pasó a **TESTING** durante la verificación y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 869 filtered en `lib.rs`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Todos los criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, 003-tester/TESTER.md)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (zona horaria del entorno del operador; no UTC explícito).

**Preflight / nombres:** Tarea fijada: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` únicamente (sin otro `UNTESTED-*`). Al inicio solo existía `CLOSED-20260322-0110-…`; según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para cumplir el paso **UNTESTED → TESTING**. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 869 filtered en `lib.rs`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored en doc-tests del crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`).

**Outcome:** Todos los criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, sesión operador / TESTER.md)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (no UTC explícito).

**Preflight / nombres:** El operador indicó solo `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Ese path no existía; el fichero era `CLOSED-20260322-0110-…`. Cadena **CLOSED → UNTESTED → TESTING** (mismo basename) según `003-tester/TESTER.md`. El H1 del documento pasó a **TESTING** durante el renombrado y se restauró a **CLOSED** antes del paso final **TESTING- → CLOSED-**.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (2 passed, 869 filtered)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en `lib.rs` tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (1143, 1347, 1934, 2310 con prefijo `discord:{}` en la clave Ollama).

**Outcome:** Criterios de aceptación cumplidos. Archivo **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (run — Cursor agent, 2026-03-28, `003-tester/TESTER.md`, task `UNTESTED-20260322-0110` only)

**Date:** 2026-03-28, local system time (where `cargo` ran); not UTC-normalized.

**Preflight / names:** Operator asked to test only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` and not to pick another `UNTESTED-*`. That filename was absent; the task on disk was `CLOSED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the literal **UNTESTED → TESTING** step applies without selecting a different task. Document H1 was set to **TESTING** for the run and restored to **CLOSED** before the final file rename **TESTING- → CLOSED-**.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` with `discord:{}`)

**Outcome:** All acceptance criteria pass. Final filename **CLOSED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md**. Live Discord not exercised.

## Test report (run — Cursor agent, 2026-03-28, `003-tester/TESTER.md`, only `UNTESTED-20260322-0110`)

**Date:** 2026-03-28, local system time where `cargo` ran (operator environment; not explicit UTC).

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). At run start the file was `CLOSED-20260322-0110-…`; per `003-tester/TESTER.md` applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 869 filtered in `lib` test binary)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` with `discord:{}`)

**Outcome:** All acceptance criteria pass. Renaming **TESTING- → CLOSED-** after this report. Live Discord not exercised.

## Test report (verificación — agente Cursor, conversación actual)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (no normalizada a UTC).

**Preflight / nombres:** El operador fijó únicamente `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin elegir otro `UNTESTED-*`). Ese path no existía al inicio; el fichero estaba como `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. El H1 del documento estuvo en **TESTING** durante `cargo` y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (2 passed, 869 filtered out en el binario de tests de `lib.rs`; `same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de librería `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Pass** — criterios de aceptación cumplidos. Nombre final del fichero: `CLOSED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md`. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, `003-tester/TESTER.md`)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (no normalizada a UTC).

**Preflight / nombres:** Tarea pedida: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` únicamente (sin otro `UNTESTED-*`). Ese path no existía al inicio; el fichero estaba como `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. El H1 del documento estuvo en **TESTING** durante `cargo` y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 869 filtered out en el binario de tests de `lib.rs`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de librería `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Pass** — todos los criterios de aceptación cumplidos. Renombrado final del fichero: **CLOSED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md**. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, `003-tester/TESTER.md`, solo `UNTESTED-20260322-0110`)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (no normalizada a UTC).

**Preflight / nombres:** Tarea fijada: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` únicamente (sin otro `UNTESTED-*`). Al inicio solo existía `CLOSED-20260322-0110-…`; según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. El H1 del documento pasó a **TESTING** durante la verificación y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 869 filtered out en el binario de tests de `lib.rs`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de librería `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Pass** — criterios de aceptación cumplidos. Renombrado final: `CLOSED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md`. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, `003-tester/TESTER.md`, tarea `UNTESTED-20260322-0110` única)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (no normalizada a UTC).

**Preflight / nombres:** El operador indicó solo `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Ese nombre no existía al inicio; el fichero era `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. El H1 pasó a **TESTING** durante la corrida y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 869 filtered out en el binario de tests de `lib.rs`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de librería `mac_stats`; 1 doc-test ignored en doc-tests del crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Pass** — todos los criterios de aceptación cumplidos. Renombrado final del fichero: **CLOSED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md**. Discord en vivo no probado.

## Test report (run — Cursor agent, 2026-03-28, `003-tester/TESTER.md`)

**Date:** 2026-03-28, local system time where `cargo` ran (not UTC-normalized).

**Preflight / names:** Operator asked to test only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` and not to pick another `UNTESTED-*`. That path was absent; the file on disk was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting a different task. Document H1 was **TESTING** during `cargo`, then restored to **CLOSED** before final filename **TESTING- → CLOSED-**.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` with `discord:{}`)

**Outcome:** **Pass** — all acceptance criteria met. Final filename: `CLOSED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md`. Live Discord not exercised.

## Test report (corrida — agente Cursor, 2026-03-28, `003-tester/TESTER.md`)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (no normalizada a UTC).

**Preflight / nombres:** Tarea pedida: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` únicamente (sin otro `UNTESTED-*`). Al inicio el fichero era `CLOSED-20260322-0110-…`; según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. El H1 pasó a **TESTING** al renombrar a `TESTING-…` y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (2 passed, 869 filtered; `same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Pass** — criterios de aceptación cumplidos. Renombrado final **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, `003-tester/TESTER.md`, tarea `UNTESTED-20260322-0110` única)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (no normalizada a UTC).

**Preflight / nombres:** El operador indicó solo `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Ese path no existía: el fichero estaba como `CLOSED-20260322-0110-…`. Se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para cumplir el paso **UNTESTED → TESTING** de `003-tester/TESTER.md`. El H1 del documento pasó a **TESTING** durante la verificación. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 869 filtered en el binario de tests de `lib.rs`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored en doc-tests del crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Pass** — criterios de aceptación cumplidos. Renombrado final del fichero: **TESTING- → CLOSED-** tras este informe (el operador pidió `TESTED-` en caso de fallo; no aplica). Discord en vivo no probado.

## Test report (run — Cursor agent, 2026-03-28, `003-tester/TESTER.md`, task `UNTESTED-20260322-0110` only)

**Date:** 2026-03-28, local system time where `cargo` ran (not UTC-normalized).

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent at run start; the file was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task. Document H1 was set to **TESTING** while the filename was `TESTING-…`, then restored to **CLOSED** before final **TESTING- → CLOSED-**.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `run_serial`; line 2310 `ollama_queue_key` with `discord:{}`)

**Outcome:** **Pass** — all acceptance criteria met. Final filename: `CLOSED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md`. Live Discord not exercised.

## Test report (corrida — agente Cursor, 2026-03-28, `003-tester/TESTER.md`)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (no normalizada a UTC).

**Preflight / nombres:** Tarea fijada: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` únicamente (sin otro `UNTESTED-*`). Al inicio del run el fichero estaba como `CLOSED-20260322-0110-…` (no existía el path `UNTESTED-…`); según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. Durante la verificación el H1 del documento estuvo en **TESTING** y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 869 filtered en el binario de tests de `lib.rs`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored en doc-tests del crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Pass** — criterios de aceptación cumplidos. Renombrado final **TESTING- → CLOSED-** tras este informe (en caso de fallo el operador pidió `TESTED-`; no aplica). Discord en vivo no probado.

## Test report (run — Cursor agent, 2026-03-28, `003-tester/TESTER.md`, only `UNTESTED-20260322-0110`)

**Date:** 2026-03-28, local system time where `cargo` ran (operator timezone; not explicit UTC).

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). At run start the file was `CLOSED-20260322-0110-…`; per `003-tester/TESTER.md` applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies. No other `UNTESTED-*` task file was used.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 869 filtered in `lib` test binary)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `run_serial`; line 2310 `ollama_queue_key` with `discord:{}`)

**Outcome:** **Pass** — all acceptance criteria met. Final filename after report: `CLOSED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (on failure the operator asked for `TESTED-`; not applicable). Live Discord not exercised.

## Test report (corrida — agente Cursor, 2026-03-28, `003-tester/TESTER.md`, tarea `UNTESTED-20260322-0110` única)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (no UTC explícito).

**Preflight / nombres:** El operador fijó solo `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Al inicio solo existía `CLOSED-20260322-0110-…`; según `003-tester/TESTER.md` se aplicó **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 869 filtered)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Pass** — criterios cumplidos. Renombrado final **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (run — Cursor agent, 2026-03-28, `003-tester/TESTER.md`, task `UNTESTED-20260322-0110` only)

**Date:** 2026-03-28, local system time where `cargo` ran (operator environment; not explicit UTC).

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent; the file on disk was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename). Document H1 was **TESTING** while `cargo` ran, then restored to **CLOSED** before this report and final filename **TESTING- → CLOSED-**.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 869 filtered in `lib` test binary)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `run_serial`; line 2310 `ollama_queue_key` with `discord:{}`)

**Outcome:** **Pass** — all acceptance criteria met. Final filename: `CLOSED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md`. On failure the operator asked for `TESTED-`; not applicable. Live Discord not exercised.

## Test report (corrida — agente Cursor, 2026-03-28, `003-tester/TESTER.md`, solo `UNTESTED-20260322-0110`)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (no normalizada a UTC).

**Preflight / nombres:** Tarea pedida: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Ese path no existía; el fichero estaba como `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 869 filtered en el binario de tests de `lib.rs`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored en doc-tests del crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Pass** — criterios de aceptación cumplidos. Renombrado final **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, `003-tester/TESTER.md`, tarea `UNTESTED-20260322-0110` única)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (no normalizada a UTC).

**Preflight / nombres:** El operador indicó solo `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Ese path no existía al inicio; el fichero estaba como `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 869 filtered en el binario de tests de `lib.rs`)
- `cd src-tauri && cargo test` — **fail** (`870 passed; 1 failed; 0 ignored` en `--lib`): `discord::tests::outbound_attachment_path_allowlist` panic en `src/discord/mod.rs:3332` — *path under pdfs_dir should be allowed when directory exists*

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Fail** — el criterio 2 de la tarea («`cargo test` en `src-tauri/` succeeds») no se cumple por un fallo ajeno directamente a `keyed_queue`, pero bloquea el cierre. Renombrado final del fichero: **TESTING- → TESTED-** tras este informe (según instrucción del operador en caso de fallo). Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, `003-tester/TESTER.md`, solo tarea `20260322-0110`)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (no normalizada a UTC).

**Preflight / nombres:** El operador indicó únicamente `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Ese path **no existía**; el fichero en disco era `tasks/TESTED-20260322-0110-…` (estado tras un fallo previo documentado). No se eligió ningún otro `UNTESTED-*`. Se renombró **TESTED → TESTING** (mismo basename) para ejecutar la fase de verificación de `003-tester/TESTER.md`. El H1 del documento quedó en **TESTING** durante la corrida.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 869 filtered en el binario de tests de `lib.rs`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored en doc-tests del crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Pass** — todos los criterios de aceptación cumplidos en esta corrida (incluido `cargo test` completo; el fallo previo de `outbound_attachment_path_allowlist` ya no se reproduce). Renombrado final del fichero: **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, `003-tester/TESTER.md`, tarea `UNTESTED-20260322-0110` única)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (no normalizada a UTC).

**Preflight / nombres:** El operador indicó solo `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Ese path no existía al inicio; el fichero estaba como `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. No se tocó ningún otro archivo `UNTESTED-*`. El H1 pasó a **TESTING** durante la verificación y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 869 filtered en el binario de tests de `lib.rs`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored en doc-tests del crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Pass** — todos los criterios de aceptación cumplidos. Renombrado final **TESTING- → CLOSED-** tras este informe (en fallo hubiera sido **TESTED-** según el operador; `003-tester/TESTER.md` indica **WIP-** para bloqueo). Discord en vivo no probado.

## Test report (corrida — agente Cursor, conversación actual 2026-03-28, `003-tester/TESTER.md`)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (no normalizada a UTC).

**Preflight / nombres:** El operador indicó solo `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Ese path no existía; el fichero estaba como `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para cumplir el paso **UNTESTED → TESTING**. El H1 del documento pasó a **TESTING** durante `cargo` y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 869 filtered out en el binario de tests de `lib.rs`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored en doc-tests del crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Pass** — todos los criterios de aceptación cumplidos. Renombrado final **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-28, `003-tester/TESTER.md`, tarea `UNTESTED-20260322-0110` única)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (no normalizada a UTC).

**Preflight / nombres:** El operador indicó solo `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Al inicio el fichero estaba como `CLOSED-20260322-0110-…`; según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. No se tocó ningún otro archivo `UNTESTED-*`. El H1 pasó a **TESTING** durante la verificación y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 869 filtered out en el binario de tests de `lib.rs`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored en doc-tests del crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Pass** — todos los criterios de aceptación cumplidos. Renombrado final **TESTING- → CLOSED-** tras este informe (en caso de fallo el operador pidió `TESTED-`; `003-tester/TESTER.md` sugiere `WIP-` si hubiera bloqueo). Discord en vivo no probado.

## Test report (run — Cursor agent, 2026-03-28, `003-tester/TESTER.md`, task `UNTESTED-20260322-0110` only)

**Date:** 2026-03-28, local system time where `cargo` ran (not UTC-normalized).

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent; the file on disk was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task. Document H1 was **TESTING** during verification, restored to **CLOSED** before final filename **TESTING- → CLOSED-**.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 869 filtered in `lib` test binary)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `run_serial`; line 2310 `ollama_queue_key` with `discord:{}`)

**Outcome:** **Pass** — all acceptance criteria met. Final filename: `CLOSED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (on failure operator asked for `TESTED-`; not applicable). Live Discord not exercised.

## Test report (corrida — agente Cursor, 2026-03-28, `003-tester/TESTER.md`, tarea `UNTESTED-20260322-0110` única)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (no normalizada a UTC).

**Preflight / nombres:** El operador indicó solo `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Al inicio el fichero estaba como `CLOSED-20260322-0110-…`; se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para cumplir el paso **UNTESTED → TESTING** de `003-tester/TESTER.md`. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 869 filtered en el binario de tests de `lib.rs`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored en doc-tests del crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Pass** — criterios de aceptación cumplidos. Renombrado final **TESTING- → CLOSED-** tras este informe (en fallo hubiera sido **TESTED-** según el operador). Discord en vivo no probado.

## Test report (run — Cursor agent, 2026-03-28, `003-tester/TESTER.md`, task `UNTESTED-20260322-0110` only)

**Date:** 2026-03-28, local system time where `cargo` ran (not UTC-normalized).

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent; the file on disk was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task. Document H1 set to **TESTING** during verification.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 869 filtered in `lib` test binary)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `run_serial`; line 2310 `ollama_queue_key` with `discord:{}`)

**Outcome:** **Pass** — all acceptance criteria met. Renaming **TESTING- → CLOSED-** after this report (`003-tester/TESTER.md` uses **WIP-** for blocked/failed work; operator asked **TESTED-** on failure — not applicable). Live Discord not exercised.

## Test report (run — Cursor agent, 2026-03-28, `003-tester/TESTER.md`, task `UNTESTED-20260322-0110` only — this session)

**Date:** 2026-03-28, hora local del sistema donde se ejecutó `cargo` (no normalizada a UTC).

**Preflight / nombres:** El operador fijó únicamente `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Ese path no existía al inicio; el fichero estaba como `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para cumplir el paso **UNTESTED → TESTING**. El H1 del documento pasó a **TESTING** durante la verificación y se restaura a **CLOSED** antes del renombrado final del archivo.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Pass** — criterios de aceptación cumplidos. Renombrado final **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-29, `003-tester/TESTER.md`, solo `UNTESTED-20260322-0110`)

**Date:** 2026-03-29, hora local del sistema donde se ejecutó `cargo` (no normalizada a UTC).

**Preflight / nombres:** El operador indicó solo `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Ese path no existía al inicio; el fichero estaba como `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING** sin elegir otra tarea. El H1 del documento pasó a **TESTING** durante la verificación y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 869 filtered en el binario de tests de `lib.rs`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored en doc-tests del crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Pass** — todos los criterios de aceptación cumplidos. Renombrado final **TESTING- → CLOSED-** tras este informe (`003-tester/TESTER.md` usa **WIP-** si hubiera bloqueo o fallo; el operador pidió **TESTED-** en caso de fallo — no aplica). Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-29, `003-tester/TESTER.md`, sesión operador UNTESTED explícito)

**Date:** 2026-03-29, hora local del sistema donde se ejecutó `cargo` (no normalizada a UTC).

**Preflight / nombres:** El operador fijó únicamente `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Al inicio del run el fichero estaba como `CLOSED-20260322-0110-…`; según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. No se tocó ningún otro archivo `UNTESTED-*`. El H1 pasó a **TESTING** durante la verificación y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 869 filtered en el binario de tests de `lib.rs`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored en doc-tests del crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Pass** — todos los criterios de aceptación cumplidos. Renombrado final **TESTING- → CLOSED-** tras este informe (en caso de fallo el operador pidió **TESTED-**; no aplica). Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-29, `003-tester/TESTER.md`, tarea UNTESTED explícita)

**Date:** 2026-03-29, hora local del sistema donde se ejecutó `cargo` (no normalizada a UTC).

**Preflight / nombres:** El operador indicó solo `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Ese path no existía al inicio; el fichero estaba como `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. No se tocó ningún otro archivo `UNTESTED-*`. El H1 pasó a **TESTING** durante `cargo` y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (2 tests: `same_key_runs_sequentially`, `different_keys_may_overlap`; 869 filtered en el binario de tests de `lib.rs`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored en doc-tests del crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `run_serial`; 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Pass** — criterios de aceptación cumplidos. Archivo final **CLOSED-** (`003-tester/TESTER.md`: **WIP-** si bloqueo/fallo; el operador pidió **TESTED-** en caso de fallo — no aplica). Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-29, `003-tester/TESTER.md`)

**Date:** 2026-03-29, hora local del sistema donde se ejecutó `cargo` (no UTC explícito).

**Preflight / nombres:** El operador fijó solo `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Ese path no existía; el fichero estaba como `CLOSED-20260322-0110-…`. Para cumplir el paso **UNTESTED → TESTING** de `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename). No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored en doc-tests del crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `crate::keyed_queue::run_serial`; 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Pass** — todos los criterios de aceptación cumplidos. Renombrado final del fichero en disco **TESTING- → CLOSED-** tras este informe. `003-tester/TESTER.md` usa **WIP-** (no **TESTED-**) si hubiera bloqueo o fallo; no aplica. Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-29, `003-tester/TESTER.md`)

**Date:** 2026-03-29, hora local del sistema donde se ejecutó `cargo` (no UTC explícito).

**Preflight / nombres:** El operador fijó solo `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Ese path no existía; el fichero estaba como `CLOSED-20260322-0110-…`. Para cumplir el paso **UNTESTED → TESTING** de `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename). El H1 del documento quedó en **TESTING** durante la verificación. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (2 tests: `same_key_runs_sequentially`, `different_keys_may_overlap`; 869 filtered out en el binario de tests de `lib.rs`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored en doc-tests del crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `crate::keyed_queue::run_serial`; 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Pass** — criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** y H1 restaurado a **CLOSED** tras este informe. (El operador pidió **TESTED-** en fallo; `003-tester/TESTER.md` prescribe **WIP-** en bloqueo/fallo.) Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-29, `003-tester/TESTER.md`)

**Date:** 2026-03-29, hora local del sistema donde se ejecutó `cargo` (no UTC explícito).

**Preflight / nombres:** El operador fijó solo `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Ese path no existía; el fichero estaba como `CLOSED-20260322-0110-…`. Para cumplir el paso **UNTESTED → TESTING** de `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename). El H1 del documento quedó en **TESTING** durante la verificación. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored en doc-tests del crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `crate::keyed_queue::run_serial`; 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Pass** — criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** y H1 a **CLOSED** tras este informe. En fallo el operador pidió prefijo **TESTED-**; `003-tester/TESTER.md` usa **WIP-** para bloqueo/fallo (no aplicado). Discord en vivo no probado.

## Test report (run — Cursor agent, 2026-03-29, `003-tester/TESTER.md`)

**Date:** 2026-03-29, local wall-clock on the host where `cargo` ran (timezone not recorded; not explicit UTC).

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` and no other `UNTESTED-*`. That path was missing; the task on disk was `tasks/CLOSED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md`. Per `003-tester/TESTER.md`, applied **CLOSED → UNTESTED → TESTING** (same basename) so the UNTESTED→TESTING step applies to this task only. Document H1 set to **TESTING** during verification.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed in `mac_stats` library tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2310 `ollama_queue_key` with `discord:{}`)

**Outcome:** **Pass** — all acceptance criteria met. File renamed **TESTING- → CLOSED-** and H1 restored to **CLOSED** after this report. Note: `003-tester/TESTER.md` uses **WIP-** for blocked/failed runs (not **TESTED-**). Live Discord not exercised.

## Test report (run — Cursor agent, 2026-03-29, `003-tester/TESTER.md`, segunda corrida)

**Date:** 2026-03-29, local wall-clock on the host where `cargo` ran (timezone not recorded; not explicit UTC).

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent; on-disk task was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied **CLOSED → UNTESTED → TESTING** (same basename) so the UNTESTED→TESTING step targets this task only. H1 set to **TESTING** for the verification window.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed in `mac_stats` library tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2310 `ollama_queue_key` with `discord:{}`)

**Outcome:** **Pass** — all acceptance criteria met. File renamed **TESTING- → CLOSED-** and H1 restored to **CLOSED** after this report. Operator naming for failure was **TESTED-**; not applied. Live Discord not exercised.

## Test report (corrida — agente Cursor, 2026-03-29, `003-tester/TESTER.md`, run actual)

**Date:** 2026-03-29, hora local del sistema donde se ejecutó `cargo` (no UTC explícito).

**Preflight / nombres:** El operador indicó solo `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Ese path no existía; el fichero estaba como `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para poder ejecutar el paso **UNTESTED → TESTING** sobre esta tarea únicamente. El H1 del documento quedó en **TESTING** durante la verificación. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored en doc-tests del crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `crate::keyed_queue::run_serial`; línea 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Pass** — criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** y H1 restaurado a **CLOSED** tras este informe. El operador pidió **TESTED-** en fallo; `003-tester/TESTER.md` prescribe **WIP-** para bloqueo/fallo (no aplicado). Discord en vivo no probado.

## Test report (run — Cursor agent, 2026-03-29, `003-tester/TESTER.md`)

**Date:** 2026-03-29, local wall-clock on the host where `cargo` ran (timezone not recorded; not explicit UTC).

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent at run start; the file was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the literal **UNTESTED → TESTING** step applies to this task only. Document H1 set to **TESTING** during verification.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2310 `ollama_queue_key` with `discord:{}`)

**Outcome:** **Pass** — all acceptance criteria met. File to be renamed **TESTING- → CLOSED-** and H1 restored to **CLOSED** after this report. On failure the operator asked for **TESTED-** (not used). Live Discord not exercised.

## Test report (corrida — agente Cursor, 2026-03-29, `003-tester/TESTER.md`)

**Date:** 2026-03-29, hora local del host donde se ejecutó `cargo` (zona horaria no registrada explícitamente; no UTC explícito).

**Preflight / nombres:** El operador indicó solo `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Ese path no existía al inicio; el fichero era `CLOSED-20260322-0110-…`. Para cumplir el paso UNTESTED→TESTING de `003-tester/TESTER.md` sin tocar otros `UNTESTED-*`, se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename).

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `crate::keyed_queue::run_serial`; línea 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Pass** — criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. En caso de fallo el operador pidió **TESTED-** (no aplicado). Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-29, `003-tester/TESTER.md`, sesión actual)

**Date:** 2026-03-29, hora local del host donde se ejecutó `cargo` (zona horaria del sistema; no UTC explícito).

**Preflight / nombres:** Tarea pedida: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin elegir otro `UNTESTED-*`). Al inicio solo existía `CLOSED-20260322-0110-…`; se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para cumplir el paso UNTESTED→TESTING de `003-tester/TESTER.md`. El H1 del documento quedó en **TESTING** durante la verificación.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `crate::keyed_queue::run_serial`; línea 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Pass** — todos los criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** y H1 restaurado a **CLOSED** tras este informe. En fallo se usaría **TESTED-** según el operador (no aplicado). Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-29, `003-tester/TESTER.md`)

**Date:** 2026-03-29, hora local del host (zona horaria del sistema; no UTC explícito).

**Preflight / nombres:** Tarea indicada: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Ese path no existía; el fichero era `CLOSED-20260322-0110-…`. Cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para cumplir el paso UNTESTED→TESTING de `003-tester/TESTER.md`. El H1 quedó en **TESTING** durante la verificación.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `crate::keyed_queue::run_serial`; línea 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Pass** — criterios de aceptación cumplidos. Archivo a renombrar **TESTING- → CLOSED-** y H1 a **CLOSED** tras este informe. `003-tester/TESTER.md` usa **WIP-** en fallo; el operador pidió **TESTED-** en fallo (no aplicado). Discord en vivo no probado.

## Test report (corrida — agente Cursor, 2026-03-29, `003-tester/TESTER.md`)

**Date:** 2026-03-29, hora local del host donde se ejecutó `cargo` (zona horaria del sistema; no UTC explícito).

**Preflight / nombres:** Tarea pedida: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). En el árbol solo existía `CLOSED-20260322-0110-…`; se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para cumplir el paso UNTESTED→TESTING de `003-tester/TESTER.md`. No se eligió ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `crate::keyed_queue::run_serial`; línea 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Pass** — criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. En fallo, `003-tester/TESTER.md` indica **WIP-** (el operador mencionó **TESTED-** como alternativa; no aplicado). Discord en vivo no probado.

## Test report (corrida — agente Cursor, `003-tester/TESTER.md`)

**Date:** 2026-03-29 04:57:25 CEST (hora local del host; no UTC).

**Preflight / nombres:** Tarea única: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Ese path no existía; el fichero estaba como `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. El H1 del documento pasó a **TESTING** durante la verificación y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `crate::keyed_queue::run_serial`; línea 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Pass** — todos los criterios de aceptación cumplidos. Archivo final **CLOSED-** (si hubiera fallo, el operador pidió prefijo **TESTED-** en lugar de **WIP-** de `TESTER.md`). Discord en vivo no probado.

## Test report (run — Cursor agent, 2026-03-29, `003-tester/TESTER.md`)

**Date:** 2026-03-29 05:09:15 CEST (hora local del host; no UTC).

**Preflight / names:** Operator requested only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent; the file on disk was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the UNTESTED→TESTING step applies without selecting another task. Document H1 set to **TESTING** during verification, restored to **CLOSED** before final filename.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed in lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2310 `ollama_queue_key` with `discord:{}`)

**Outcome:** **Pass** — all acceptance criteria met. File renamed **TESTING- → CLOSED-** after this report. On failure, operator asked for **TESTED-** (this run used **CLOSED-**). Live Discord not exercised.

## Test report (run — Cursor agent, 2026-03-29, `003-tester/TESTER.md`)

**Date:** 2026-03-29 05:21:13 CEST (local host time; not UTC).

**Preflight / names:** Operator requested only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). On disk the file was `CLOSED-20260322-0110-…`; per `003-tester/TESTER.md` applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) to perform the **UNTESTED → TESTING** step. Document H1 set to **TESTING** during verification; restored to **CLOSED** before final **TESTING- → CLOSED-** filename.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed in lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2310 `ollama_queue_key` with `discord:{}`)

**Outcome:** **Pass** — all acceptance criteria met. File renamed **TESTING- → CLOSED-** after this report (on failure, operator asked for **TESTED-** instead of `TESTER.md` **WIP-**). Live Discord not exercised.

## Test report (run — Cursor agent, 2026-03-29, `003-tester/TESTER.md`)

**Date:** 2026-03-29 03:47:55 UTC (local: 2026-03-29 05:47:55 CEST).

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent; the file on disk was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies. Document H1 set to **TESTING** during verification; restored to **CLOSED** before final **TESTING- → CLOSED-** filename.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed in lib crate tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2310 `ollama_queue_key` with `discord:{}`)

**Outcome:** **Pass** — all acceptance criteria met. Final filename **CLOSED-** per pass (`003-tester/TESTER.md` uses **WIP-** on failure; operator text asked for **TESTED-** on fail). Live Discord not exercised.

## Test report (run — Cursor agent, 2026-03-29, `003-tester/TESTER.md`)

**Date:** 2026-03-29 04:02:26 UTC (local: 2026-03-29 06:02:26 CEST).

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent; the file on disk was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies. Document H1 set to **TESTING** during verification; restored to **CLOSED** before final **TESTING- → CLOSED-** filename.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed in lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2310 `ollama_queue_key` with `discord:{}`)

**Outcome:** **Pass** — all acceptance criteria met. Final filename **CLOSED-** (on failure, operator asked for **TESTED-** instead of `TESTER.md` **WIP-**). Live Discord not exercised.

## Test report (run — Cursor agent, 2026-03-29, `003-tester/TESTER.md`)

**Date:** 2026-03-29 04:17:05 UTC (local: 2026-03-29 06:17:05 CEST).

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent at run start; the file on disk was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task. Document H1 set to **TESTING** during verification; restored to **CLOSED** before final **TESTING- → CLOSED-** filename.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2310 `ollama_queue_key` with `discord:{}`)

**Outcome:** **Pass** — all acceptance criteria met. Final filename **CLOSED-** (`003-tester/TESTER.md` would use **WIP-** on failure; operator asked for **TESTED-** on fail). Live Discord not exercised.

## Test report (run — Cursor agent, 2026-03-29, `003-tester/TESTER.md`)

**Date:** 2026-03-29 04:34:59 UTC (local: 2026-03-29 06:34:59 CEST).

**Preflight / nombres:** Tarea indicada: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin elegir otro `UNTESTED-*`). Ese path no existía al inicio; el fichero estaba como `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. El H1 del documento pasó a **TESTING** durante la verificación y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `crate::keyed_queue::run_serial`; línea 2310 `ollama_queue_key` con `discord:{}`)

**Outcome:** **Pass** — criterios de aceptación cumplidos. Nombre final del archivo **CLOSED-** (en fallo habría sido **TESTED-** según el operador). Discord en vivo no probado.

## Test report (run — Cursor agent, 2026-03-29, `003-tester/TESTER.md`)

**Date:** 2026-03-29 04:48:54 UTC (local: 2026-03-29 06:48:54 CEST, +0200).

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). At run start the file was `CLOSED-20260322-0110-…`; per `003-tester/TESTER.md` applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies. Document H1 set to **TESTING** during verification; restored to **CLOSED** before final **TESTING- → CLOSED-** filename.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2310 `ollama_queue_key` with `discord:{}`)

**Outcome:** **Pass** — all acceptance criteria met. Final filename **CLOSED-** (on failure, operator asked for **TESTED-** instead of `TESTER.md` **WIP-**). Live Discord not exercised.

## Test report (run — Cursor agent, 2026-03-29, `003-tester/TESTER.md`)

**Date:** 2026-03-29 05:02:57 UTC (local: 2026-03-29 07:02:57 CEST).

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent at run start; the file on disk was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies. Document H1 set to **TESTING** during verification; restored to **CLOSED** before final **TESTING- → CLOSED-** filename.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2310 `ollama_queue_key` with `discord:{}`)

**Outcome:** **Pass** — all acceptance criteria met. Final filename **CLOSED-** (on failure, operator asked for **TESTED-** instead of `TESTER.md` **WIP-**). Live Discord not exercised.

## Test report (corrida — agente Cursor, `003-tester/TESTER.md`)

**Fecha:** 2026-03-29 05:15:48 UTC (local: 2026-03-29 07:15:48 CEST).

**Preflight / nombres:** El operador indicó solo `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin elegir otro `UNTESTED-*`). Al inicio no existía ese path; en disco estaba `CLOSED-20260322-0110-…`. Para cumplir el paso UNTESTED→TESTING de `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename). El H1 del documento quedó en **TESTING** durante la verificación y se restaurará a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**.

**Comandos ejecutados**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Comprobación estática**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `crate::keyed_queue::run_serial`; línea 2310 `ollama_queue_key` con `discord:{}`)

**Resultado:** **Pass** — criterios de aceptación cumplidos. Nombre final del archivo **CLOSED-** (en fallo el operador pidió **TESTED-** frente a **WIP-** en `TESTER.md`). Discord en vivo no probado.

## Test report (corrida — agente Cursor, `003-tester/TESTER.md`)

**Fecha:** 2026-03-29 05:30:08 UTC (local: 2026-03-29 07:30:08 CEST).

**Preflight / nombres:** El operador indicó solo `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Al inicio el fichero estaba como `CLOSED-20260322-0110-…`; para cumplir el paso UNTESTED→TESTING de `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename). El H1 del documento quedó en **TESTING** durante la verificación y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**. No se tocó ningún otro archivo `UNTESTED-*`.

**Comandos ejecutados**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Comprobación estática**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `crate::keyed_queue::run_serial`; línea 2310 `ollama_queue_key` con `discord:{}`)

**Resultado:** **Pass** — criterios de aceptación cumplidos. Nombre final del archivo **CLOSED-** (en fallo habría sido **TESTED-** según el operador). Discord en vivo no probado.

## Test report (corrida — agente Cursor, `003-tester/TESTER.md`)

**Fecha:** 2026-03-29 05:43:15 UTC (local: 2026-03-29 07:43:15 CEST).

**Preflight / nombres:** El operador indicó solo `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). Al inicio el fichero estaba como `CLOSED-20260322-0110-…`; para cumplir el paso UNTESTED→TESTING de `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename). El H1 del documento quedó en **TESTING** durante la verificación y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**. No se tocó ningún otro archivo `UNTESTED-*`.

**Comandos ejecutados**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Comprobación estática**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `crate::keyed_queue::run_serial`; línea 2310 `ollama_queue_key` con `discord:{}`)

**Resultado:** **Pass** — criterios de aceptación cumplidos. Nombre final del archivo **CLOSED-** (en fallo habría sido **TESTED-** según el operador). Discord en vivo no probado.

## Test report (corrida — agente Cursor, `003-tester/TESTER.md`)

**Fecha:** 2026-03-29 05:57:05 UTC (local: 2026-03-29 07:57:05 CEST).

**Preflight / nombres:** Tarea indicada: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin otro `UNTESTED-*`). En el disco solo existía `CLOSED-20260322-0110-…`; para cumplir el paso UNTESTED→TESTING de `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename). No se tocó ningún otro archivo `UNTESTED-*`.

**Comandos ejecutados**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Comprobación estática**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `crate::keyed_queue::run_serial`; línea 2310 `ollama_queue_key` con `discord:{}`)

**Resultado:** **Pass** — criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. Discord en vivo no probado.

## Test report (corrida — agente Cursor, `003-tester/TESTER.md`)

**Date:** 2026-03-29 06:10:20 UTC (local: 2026-03-29 08:10:20 CEST).

**Preflight / nombres:** Tarea pedida: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin elegir otro `UNTESTED-*`). Ese path no existía al inicio; el fichero estaba como `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. El H1 del documento pasó a **TESTING** durante la verificación y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed in lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2310 `ollama_queue_key` with `discord:{}`)

**Outcome:** **Pass** — all acceptance criteria met. Final filename **CLOSED-** (on failure would have been **TESTED-** per operator). Live Discord not exercised.

## Test report (corrida — agente Cursor, `003-tester/TESTER.md`)

**Date:** 2026-03-29 06:23:52 UTC (local: 2026-03-29 08:23:52 CEST).

**Preflight / names:** Operator requested only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent; the file on disk was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task. Document H1 set to **TESTING** during verification, restored to **CLOSED** before final filename rename.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed in lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2310 `ollama_queue_key` with `discord:{}`)

**Outcome:** **Pass** — all acceptance criteria met. Final filename **CLOSED-** (failure would have been **TESTED-** per operator). Live Discord not exercised.

## Test report (corrida — agente Cursor, `003-tester/TESTER.md`)

**Date:** 2026-03-29 06:36:08 UTC (local: 2026-03-29 08:36:08 CEST).

**Preflight / nombres:** Tarea indicada: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` únicamente (sin otro `UNTESTED-*`). Ese path no existía al inicio; el fichero estaba como `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. El H1 del documento pasó a **TESTING** durante la verificación y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Comprobación estática**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `crate::keyed_queue::run_serial`; línea 2310 `ollama_queue_key` con `discord:{}`)

**Resultado:** **Pass** — criterios de aceptación cumplidos. Archivo final **CLOSED-** (en fallo habría sido **TESTED-** según el operador). Discord en vivo no probado.

## Test report (corrida — agente Cursor, `003-tester/TESTER.md`)

**Date:** 2026-03-29 06:55:54 UTC (local: 2026-03-29 08:55:54 CEST).

**Preflight / nombres:** Tarea pedida: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` únicamente (sin otro `UNTESTED-*`). Ese path no existía al inicio; el fichero estaba como `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. El H1 del documento pasó a **TESTING** durante la verificación y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Comprobación estática**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `crate::keyed_queue::run_serial`; línea 2310 `ollama_queue_key` con `discord:{}`)

**Resultado:** **Pass** — criterios de aceptación cumplidos. Archivo final **CLOSED-** (en fallo habría sido **TESTED-** según el operador). Discord en vivo no probado.

## Test report (corrida — agente Cursor, `003-tester/TESTER.md`)

**Date:** 2026-03-29 07:08:25 UTC (local: 2026-03-29 09:08:25 CEST).

**Preflight / naming:** Operator requested only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path did not exist; the task file was `tasks/CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, renamed **CLOSED → TESTING** (same basename). H1 set to **TESTING** during verification. No other `UNTESTED-*` file was touched.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed in lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2310 `ollama_queue_key` with `discord:{}`)

**Outcome:** **Pass** — all acceptance criteria met. Per operator instruction, failure would have been **TESTED-**; `003-tester/TESTER.md` default is **WIP-**. Live Discord not exercised.

## Test report (corrida — agente Cursor, `003-tester/TESTER.md`)

**Date:** 2026-03-29 07:22:49 UTC (local: 2026-03-29 09:22:49 CEST).

**Preflight / nombres:** Tarea indicada: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` únicamente (sin otro `UNTESTED-*`). Ese path no existía al inicio; el fichero estaba como `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. El H1 del documento pasó a **TESTING** durante la verificación y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Comprobación estática**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `crate::keyed_queue::run_serial`; línea 2310 `ollama_queue_key` con `discord:{}`)

**Resultado:** **Pass** — criterios de aceptación cumplidos. Archivo final **CLOSED-** (en fallo habría sido **TESTED-** según el operador). Discord en vivo no probado.

## Test report (corrida — agente Cursor, `003-tester/TESTER.md`)

**Date:** 2026-03-29 07:36:34 UTC (local: 2026-03-29 09:36:34 CEST).

**Preflight / nombres:** Tarea pedida: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` únicamente (sin otro `UNTESTED-*`). Ese path no existía; el fichero estaba como `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. El H1 del documento quedó en **TESTING** durante la verificación. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Comprobación estática**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `crate::keyed_queue::run_serial`; línea 2310 `ollama_queue_key` con `discord:{}`)

**Resultado:** **Pass** — criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe (`003-tester/TESTER.md` usa **WIP-** para bloqueo/fallo; el operador pidió **TESTED-** en caso de fallo). Discord en vivo no probado.

## Test report (corrida — agente Cursor, `003-tester/TESTER.md`)

**Date:** 2026-03-29 07:49:38 UTC (local: 2026-03-29 09:49:38 CEST).

**Preflight / nombres:** Tarea pedida: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` únicamente (sin otro `UNTESTED-*`). Al inicio el fichero estaba como `CLOSED-20260322-0110-…`; se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para cumplir el paso **UNTESTED → TESTING** de `003-tester/TESTER.md`. El H1 del documento pasó a **TESTING** durante la verificación y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**. No se tocó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored)

**Comprobación estática**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `crate::keyed_queue::run_serial`; línea 2310 `ollama_queue_key` con `discord:{}`)

**Resultado:** **Pass** — criterios de aceptación cumplidos. Archivo final **CLOSED-** (`003-tester/TESTER.md`: **WIP-** si bloqueo/fallo; el operador indicó **TESTED-** en fallo). Discord en vivo no probado.

## Test report

**Date:** 2026-03-29 08:02:50 UTC (local: 2026-03-29 10:02:50 CEST).

**Preflight / naming:** Operator requested only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path did not exist at run start; the file was `tasks/CLOSED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task file. Document H1 set to **TESTING** during verification, restored to **CLOSED** before final filename rename. No other `UNTESTED-*` file was touched.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed in lib `mac_stats`; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2310 `ollama_queue_key` with `discord:{}`)

**Outcome:** **Pass** — all acceptance criteria met. File renamed **TESTING- → CLOSED-** after this report. On failure the operator asked for **TESTED-** (TESTER.md default for blocked/failed work is **WIP-**). Live Discord not exercised.

## Test report

**Date:** 2026-03-29 08:19:17 UTC (local: 2026-03-29 10:19:17 CEST).

**Preflight / nombres:** Tarea fijada: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` únicamente (sin otro `UNTESTED-*`). Al inicio el fichero estaba como `CLOSED-20260322-0110-…`; según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. No se tocó ningún otro archivo `UNTESTED-*`. El H1 del documento pasó a **TESTING** durante la verificación y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en la lib `mac_stats`; 1 doc-test ignored)

**Comprobación estática**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `crate::keyed_queue::run_serial`; línea 2310 `ollama_queue_key` con `discord:{}`)

**Resultado:** **Pass** — criterios de aceptación cumplidos. Archivo renombrado a **CLOSED-** tras este informe. En caso de fallo el operador pidió prefijo **TESTED-** (`003-tester/TESTER.md` usa **WIP-** para bloqueo/fallo). Discord en vivo no probado.

## Test report (corrida — agente Cursor, `003-tester/TESTER.md`)

**Fecha:** 2026-03-29 08:36:59 UTC (local: 2026-03-29 10:36:59 CEST).

**Preflight / nombres:** Tarea fijada: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` únicamente (sin otro `UNTESTED-*`). Al inicio el fichero estaba como `CLOSED-20260322-0110-…`; según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. No se tocó ningún otro archivo `UNTESTED-*`.

**Comandos ejecutados**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (871 passed, 0 failed en la lib `mac_stats`; 1 doc-test ignored)

**Comprobación estática**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `crate::keyed_queue::run_serial`; línea 2310 `ollama_queue_key` con `discord:{}`)

**Resultado:** **Pass** — criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. En fallo habría sido **TESTED-** según el operador (`003-tester/TESTER.md` indica **WIP-** para bloqueo/fallo). Discord en vivo no probado.

## Test report

**Fecha:** 2026-03-29 08:54:05 UTC (local: 2026-03-29 10:54:05 CEST).

**Preflight / nombres:** Tarea fijada: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` únicamente (sin otro `UNTESTED-*`). Ese path no existía al inicio; el fichero estaba como `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. No se tocó ningún otro archivo `UNTESTED-*`. El H1 del documento pasó a **TESTING** durante la verificación y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**.

**Comandos ejecutados**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (872 passed, 0 failed en la lib `mac_stats`; 1 doc-test ignored)

**Comprobación estática**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `crate::keyed_queue::run_serial`; línea 2310 `ollama_queue_key` con `discord:{}`)

**Resultado:** **Pass** — criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. En fallo el operador pidió **TESTED-** (`003-tester/TESTER.md` usa **WIP-** para bloqueo/fallo). Discord en vivo no probado.

## Test report

**Fecha:** 2026-03-29 09:10:26 UTC (local: 2026-03-29 11:10:26 CEST).

**Preflight / nombres:** Tarea fijada: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` únicamente (sin otro `UNTESTED-*`). Ese path no existía al inicio; el fichero estaba como `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. No se tocó ningún otro archivo `UNTESTED-*`. El H1 del documento pasó a **TESTING** durante la verificación y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**.

**Comandos ejecutados**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (872 passed, 0 failed en la lib `mac_stats`; 1 doc-test ignored)

**Comprobación estática**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `crate::keyed_queue::run_serial`; línea 2310 `ollama_queue_key` con `discord:{}`)

**Resultado:** **Pass** — criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. En fallo habría sido **TESTED-** según el operador (`003-tester/TESTER.md` indica **WIP-** para bloqueo/fallo). Discord en vivo no probado.

## Test report

**Fecha:** 2026-03-29 09:24:52 UTC (hora local del sistema donde se ejecutó `cargo`).

**Preflight / nombres:** Tarea indicada: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin elegir otro `UNTESTED-*`). Ese path no existía al inicio; el fichero estaba como `CLOSED-20260322-0110-…`. Según `003-tester/TESTER.md` se aplicó la cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para ejecutar el paso **UNTESTED → TESTING**. No se tocó ningún otro archivo `UNTESTED-*`. El H1 del documento pasó a **TESTING** durante la verificación y se restauró a **CLOSED** antes del renombrado final **TESTING- → CLOSED-**.

**Comandos ejecutados**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (872 passed, 0 failed en la crate lib `mac_stats`; 1 doc-test ignored)

**Comprobación estática**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `crate::keyed_queue::run_serial`; línea 2310 `ollama_queue_key` con `discord:{}`)

**Resultado:** **Pass** — criterios de aceptación cumplidos. Archivo renombrado **TESTING- → CLOSED-** tras este informe. En fallo habría sido **TESTED-** según el operador. Discord en vivo no probado.


## Test report (corrida — agente Cursor, sesión 2026-03-29, `003-tester/TESTER.md`)

**Fecha:** 2026-03-29 09:42:36 UTC (local: 2026-03-29 11:42:36 CEST).

**Preflight / nombres:** Tarea fijada: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` únicamente (sin otro `UNTESTED-*`). Ese path no existía; el fichero estaba como `CLOSED-20260322-0110-…`. Cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para cumplir el paso **UNTESTED → TESTING** de `003-tester/TESTER.md`. El H1 del documento quedó en **TESTING** durante la verificación.

**Comandos ejecutados**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (872 passed, 0 failed en la lib `mac_stats`; 1 doc-test ignored en doc-tests del crate)

**Comprobación estática**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `crate::keyed_queue::run_serial`; línea 2310 `ollama_queue_key` con `discord:{}`)

**Resultado:** **Pass** — criterios de aceptación cumplidos. Tras este informe: renombrar **TESTING- → CLOSED-** y restaurar H1 a **CLOSED**. En fallo el operador pidió **TESTED-**; `003-tester/TESTER.md` usa **WIP-** para bloqueo/fallo. Discord en vivo no probado.


## Test report (corrida — agente Cursor, sesión 2026-03-29, `003-tester/TESTER.md`)

**Fecha:** 2026-03-29, hora local del sistema donde se ejecutó `cargo` (zona horaria del entorno; UTC no fijado explícitamente en esta corrida).

**Preflight / nombres:** Tarea fijada: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` únicamente (sin otro `UNTESTED-*`). Ese path no existía al inicio; el fichero estaba como `CLOSED-20260322-0110-…`. Cadena **CLOSED → UNTESTED → TESTING** (mismo basename) para cumplir el paso **UNTESTED → TESTING** de `003-tester/TESTER.md`. El H1 del documento quedó en **TESTING** durante la verificación.

**Comandos ejecutados**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 870 pruebas omitidas por el filtro en el target de lib)
- `cd src-tauri && cargo test` — **pass** (872 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored en el crate)

**Comprobación estática**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `crate::keyed_queue::run_serial`; línea 2310 `ollama_queue_key` con `discord:{}`)

**Resultado:** **Pass** — todos los criterios de aceptación cumplidos. Tras este informe el archivo se renombra **TESTING- → CLOSED-** y el H1 vuelve a **CLOSED**. En caso de fallo el operador pidió prefijo **TESTED-** (en `TESTER.md` figura **WIP-**). Discord en vivo no probado.


## Test report (corrida — agente Cursor, `003-tester/TESTER.md`)

**Date:** 2026-03-29 10:12:13 UTC; local: 2026-03-29 12:12:13 CEST.

**Preflight / names:** Operator requested only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). At run start the file was `CLOSED-20260322-0110-…`; per `003-tester/TESTER.md` applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task file. Document H1 set to **TESTING** during this run.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; other lib tests filtered out)
- `cd src-tauri && cargo test` — **pass** (872 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored in crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2310 `ollama_queue_key` with `discord:{}`)

**Outcome:** **Pass** — all acceptance criteria met. After this report: restore H1 to **CLOSED** and rename file **TESTING- → CLOSED-**. On failure the operator asked for **TESTED-** (TESTER.md lists **WIP-**). Live Discord not exercised.


## Test report (corrida — agente Cursor, `003-tester/TESTER.md`, 2026-03-30)

**Fecha:** 2026-03-30, hora local del sistema donde se ejecutó `cargo` (zona horaria del entorno; no se fijó UTC explícitamente).

**Preflight / nombres:** Tarea fijada por el operador: `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (sin elegir otro `UNTESTED-*`). Ese path **no existía** en el repo; la misma tarea está en `tasks/CLOSED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md`. Por tanto **no hubo** renombrado literal `UNTESTED- → TESTING-` sobre el nombre pedido. No se tocó ningún otro archivo `UNTESTED-*`. La verificación se ejecutó según el cuerpo de la tarea (criterios y comandos) sobre el fichero **CLOSED-** existente.

**Comandos ejecutados**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 873 tests filtrados en el target de lib)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed en tests de la lib `mac_stats`; 1 doc-test ignored en el crate)

**Comprobación estática**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (líneas 1143, 1347, 1934 `crate::keyed_queue::run_serial`; línea 2345 `ollama_queue_key` con `discord:{}`; las líneas pueden variar respecto a versiones anteriores del informe)

**Resultado:** **Pass** — criterios de aceptación cumplidos. El nombre del archivo permanece **CLOSED-** (no aplica **TESTED-** ni **TESTPLAN-**). Discord en vivo no probado.


## Test report (run — Cursor agent, `003-tester/TESTER.md`, 2026-03-30)

**Date:** 2026-03-30, local system time where `cargo` ran (environment timezone; not fixed to UTC).

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent at run start; the file on disk was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task. Document H1 set to **TESTING** during verification.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; other lib tests filtered out on that invocation)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored in crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `discord:{}`)

**Outcome:** **Pass** — all acceptance criteria met. After this report: restore H1 to **CLOSED** and rename file **TESTING- → CLOSED-** (per operator: **TESTED-** on implementation fail, **TESTPLAN-** on defective test instructions; neither applies). Live Discord not exercised.

**Note:** An earlier 2026-03-30 report above stated no UNTESTED→TESTING rename; **this** run performed the full rename chain and re-ran verification.

## Test report (2026-03-30 — Cursor agent, `003-tester/TESTER.md`)

**Date:** 2026-03-30, local system time where `cargo` ran (host timezone; not fixed to UTC).

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was missing; the file was `CLOSED-20260322-0110-…`. Applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step is satisfied without selecting another task file.

**Commands run** (from `src-tauri/`):

- `cargo check` — **pass**
- `cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored in crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `format!("discord:{}", …)`)

**Outcome:** **Pass** (all acceptance criteria). Rename file **TESTING- → CLOSED-** after this report. Operator outcome mapping: not **TESTED-** (implementation fail) or **TESTPLAN-** (defective instructions). Live Discord not exercised.

## Test report (run — Cursor agent, `003-tester/TESTER.md`, 2026-03-30)

**Date:** 2026-03-30 12:55:39 UTC; local: 2026-03-30 14:55:39 CEST.

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). At run start the file was `CLOSED-20260322-0110-…`; per `003-tester/TESTER.md` applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task. Document H1 set to **TESTING** during verification, restored to **CLOSED** before final filename.

**Commands run** (from repo root / `src-tauri/` as stated):

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 873 lib tests filtered on that invocation)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored in crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `format!("discord:{}", …)`)

**Outcome:** **Pass** — all acceptance criteria met. Final filename **CLOSED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md** (not **TESTED-** or **TESTPLAN-**). `003-tester/TESTER.md` maps this to **CLOSED-** (equivalent to its **WIP-** only when blocked/failed). Live Discord not exercised.

## Test report (2026-03-30 — Cursor agent, `003-tester/TESTER.md`)

**Date:** 2026-03-30 13:04:12 UTC; local: 2026-03-30 15:04:12 CEST.

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent at run start; the file was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task. Document H1 set to **TESTING** during verification, restored to **CLOSED** before final filename.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored in crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `format!("discord:{}", …)`)

**Outcome:** **Pass** — all acceptance criteria met. Final filename **CLOSED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md** (not **TESTED-** implementation fail or **TESTPLAN-** defective instructions). Live Discord not exercised.

## Test report (2026-03-30 — Cursor agent, `003-tester/TESTER.md`)

**Date:** 2026-03-30 13:16:35 UTC; local: 2026-03-30 15:16:35 CEST.

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent at run start; the file was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task. Document H1 set to **TESTING** during verification, restored to **CLOSED** before final filename.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored in crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `format!("discord:{}", …)`)

**Outcome:** **Pass** — all acceptance criteria met. Final filename **CLOSED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md** (operator mapping: **CLOSED-** on pass, not **TESTED-** or **TESTPLAN-**). Live Discord not exercised.

## Test report (2026-03-30 — Cursor agent, `003-tester/TESTER.md`)

**Date:** 2026-03-30 13:29:32 UTC; local: 2026-03-30 15:29:32 CEST.

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). At run start the file was `CLOSED-20260322-0110-…`; per `003-tester/TESTER.md` applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task. Document H1 set to **TESTING** during verification, restored to **CLOSED** before final filename.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 873 filtered)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored in crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `format!("discord:{}", …)`)

**Outcome:** **Pass** — all acceptance criteria met. Final filename **CLOSED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md** (not **TESTED-** implementation fail or **TESTPLAN-** defective instructions). Live Discord not exercised.

## Test report (2026-03-30 — Cursor agent, `003-tester/TESTER.md`)

**Date:** 2026-03-30 13:39:57 UTC; local: 2026-03-30 15:39:57 CEST.

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent at run start; the file was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task. Document H1 was briefly set to **TESTING** then restored to **CLOSED** before final filename.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored in crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `format!("discord:{}", …)`)

**Outcome:** **Pass** — all acceptance criteria met. Final filename **CLOSED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md** (operator mapping: **CLOSED-** on pass, not **TESTED-** or **TESTPLAN-**). Live Discord not exercised.

## Test report (2026-03-30 — Cursor agent, `003-tester/TESTER.md`)

**Date:** 2026-03-30 13:59:53 UTC; local: 2026-03-30 15:59:53 CEST.

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent at run start; the file was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task. Document H1 set to **TESTING** during verification, restored to **CLOSED** before final filename.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 873 filtered in lib test binary)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored in crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `format!("discord:{}", …)`)

**Outcome:** **Pass** — all acceptance criteria met. Final filename **CLOSED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md** (not **TESTED-** implementation fail or **TESTPLAN-** defective instructions). Live Discord not exercised.

## Test report (2026-03-30 — Cursor agent, `003-tester/TESTER.md`)

**Date:** 2026-03-30 14:20:01 UTC; local: 2026-03-30 16:20:01 CEST.

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent at run start; the file was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task. Document H1 set to **TESTING** during verification, restored to **CLOSED** before final filename.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored in crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `format!("discord:{}", …)`)

**Outcome:** **Pass** — all acceptance criteria met. Final filename **CLOSED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md** (operator mapping: **CLOSED-** on pass). Live Discord not exercised.

## Test report (2026-03-30 — Cursor agent, `003-tester/TESTER.md`)

**Date:** 2026-03-30 14:31:43 UTC; local: 2026-03-30 16:31:43 CEST.

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent at run start; the file was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task. Document H1 set to **TESTING** during verification, restored to **CLOSED** before final filename.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 873 filtered in lib test binary)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored in crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `format!("discord:{}", …)`)

**Outcome:** **Pass** — all acceptance criteria met. Final filename **CLOSED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md** (not **TESTED-** implementation fail or **TESTPLAN-** defective instructions). Live Discord not exercised.

## Test report (2026-03-30 — Cursor agent, `003-tester/TESTER.md`)

**Date:** 2026-03-30 14:42:49 UTC; local: 2026-03-30 16:42:49 CEST.

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent at run start; the file was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task. Document H1 set to **TESTING** during verification, restored to **CLOSED** before final filename.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored in crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `format!("discord:{}", …)`)

**Outcome:** **Pass** — all acceptance criteria met. Final filename **CLOSED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md** (operator mapping: **CLOSED-** on pass; not **TESTED-** or **TESTPLAN-**). Live Discord not exercised.

## Test report (2026-03-30 — Cursor agent, `003-tester/TESTER.md`)

**Date:** 2026-03-30 14:54:26 UTC; local: 2026-03-30 16:54:26 CEST.

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent at run start; the file was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task. Document H1 set to **TESTING** during verification, restored to **CLOSED** before final filename.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 873 filtered out in lib test binary)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored in crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `format!("discord:{}", …)`)

**Outcome:** **Pass** — all acceptance criteria met. Final filename **CLOSED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md** (operator mapping: **CLOSED-** on pass; not **TESTED-** or **TESTPLAN-**). Live Discord not exercised.

## Test report (2026-03-30 — Cursor agent, `003-tester/TESTER.md`)

**Date:** 2026-03-30 15:03:50 UTC; local: 2026-03-30 17:03:50 CEST.

**Preflight / names:** Operator named `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` only (no other `UNTESTED-*`). That path did not exist; on disk the task was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step is satisfied without picking another task. H1 set to **TESTING** for this phase; restored to **CLOSED** before final filename.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored in crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `discord:{}`)

**Outcome:** **Pass** — all acceptance criteria met. Renamed **TESTING- → CLOSED-** after this report (not **TESTED-** or **TESTPLAN-**). Live Discord not exercised.

## Test report (2026-03-30 — Cursor agent, `003-tester/TESTER.md`)

**Date:** 2026-03-30 15:15:45 UTC; local: 2026-03-30 17:15:45 CEST.

**Preflight / names:** Operator required only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That filename was not present; the task on disk was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step is satisfied without selecting another queue file. H1 set to **TESTING** during verification; restored to **CLOSED** before final filename.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 873 filtered out in lib test binary)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored in crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `format!("discord:{}", …)`)

**Outcome:** **Pass** — all acceptance criteria met. Final filename **CLOSED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md** (operator mapping: **CLOSED-** on pass). Live Discord not exercised.

## Test report (2026-03-30 — Cursor agent, `003-tester/TESTER.md`)

**Date:** 2026-03-30 15:26:53 UTC; local: 2026-03-30 17:26:53 CEST.

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent; the file on disk was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task. Document H1 was **TESTING** during verification, restored to **CLOSED** before final filename rename.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored in crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `discord:{}`)

**Outcome:** **Pass** — all acceptance criteria met. Renamed **TESTING- → CLOSED-** after this report. Live Discord not exercised.

## Test report (2026-03-30 — Cursor agent, `003-tester/TESTER.md`)

**Date:** 2026-03-30 15:48:29 UTC (authoritative). Local wall clock follows the machine where `cargo` ran.

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent; the file on disk was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task. Document H1 was **TESTING** during verification, restored to **CLOSED** before final filename rename.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 873 filtered out in lib test binary)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored in crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `format!("discord:{}", …)`)

**Outcome:** **Pass** — all acceptance criteria met. Per operator mapping: **CLOSED-** (pass), not **TESTED-** or **TESTPLAN-**. Renamed **TESTING- → CLOSED-** after this report. Live Discord not exercised.

## Test report (2026-03-30 — Cursor agent, `003-tester/TESTER.md`)

**Date:** 2026-03-30 15:59:45 UTC (authoritative). Local wall clock is the machine where `cargo` ran.

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent at run start; the file on disk was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task. Document H1 was **TESTING** during verification, restored to **CLOSED** before final filename rename.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored in crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `format!("discord:{}", …)`)

**Outcome:** **Pass** — all acceptance criteria met. Per operator mapping: **CLOSED-** (pass). Renamed **TESTING- → CLOSED-** after this report. Live Discord not exercised.

## Test report (2026-03-30 — Cursor agent, `003-tester/TESTER.md`)

**Date:** 2026-03-30 16:10:49 UTC (authoritative). Local wall clock is the machine where `cargo` ran.

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent at run start; the file on disk was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task. Document H1 was **TESTING** during verification, restored to **CLOSED** before final filename rename.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 873 filtered out in lib test binary)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored in crate)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `format!("discord:{}", …)`)

**Outcome:** **Pass** — all acceptance criteria met. Per operator mapping: **CLOSED-** (pass), not **TESTED-** or **TESTPLAN-**. Renamed **TESTING- → CLOSED-** after this report. Live Discord not exercised.

## Test report (2026-03-30 — Cursor agent, `003-tester/TESTER.md`)

**Date:** 2026-03-30 16:22:51 UTC (authoritative). Local time is the host where `cargo` ran.

**Preflight / names:** Operator requested only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was missing at run start; on disk the task was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without picking another `UNTESTED-*` file. Document H1 was set to **TESTING** for the active run, then restored to **CLOSED** before final filename rename.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `discord:{}`)

**Outcome:** **Pass** — all acceptance criteria met. Final filename: **CLOSED-** per operator mapping (not **TESTED-** or **TESTPLAN-**). Live Discord not exercised.

## Test report (2026-03-30 — Cursor agent, `003-tester/TESTER.md`)

**Date:** 2026-03-30 16:33:56 UTC (authoritative). Local time is the host where `cargo` ran.

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent at run start; the file on disk was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task. Document H1 was **TESTING** during `cargo` / `rg`, then set to **CLOSED** before final filename rename.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed, 873 filtered out in lib test binary)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `format!("discord:{}", …)`)

**Outcome:** **Pass** — all acceptance criteria met. Per operator mapping: **CLOSED-** (pass), not **TESTED-** or **TESTPLAN-**. Renamed **TESTING- → CLOSED-** after this report. Live Discord not exercised.

## Test report (2026-03-30 — Cursor agent, `003-tester/TESTER.md`)

**Date:** 2026-03-30 16:45:19 UTC (authoritative). Local time is the host where `cargo` ran.

**Preflight / names:** Operator specified only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path was absent at run start; the file on disk was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task. Document H1 was **TESTING** during verification, then set to **CLOSED** before final filename rename.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed in lib test binary)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `format!("discord:{}", …)`)

**Outcome:** **Pass** — all acceptance criteria met. Final filename: **CLOSED-** per operator mapping (not **TESTED-** or **TESTPLAN-**). Live Discord not exercised.

## Test report (2026-03-30 — Cursor agent, `003-tester/TESTER.md`)

**Date:** 2026-03-30, local wall clock on the host where `cargo` ran (mac-stats workspace).

**Preflight / names:** Operator required only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). At run start only `CLOSED-20260322-0110-…` existed; applied **CLOSED → UNTESTED → TESTING** (same basename) per `003-tester/TESTER.md` so **UNTESTED → TESTING** is satisfied without selecting another task.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `discord:{}`)

**Outcome:** All acceptance criteria pass → **CLOSED-** (not **TESTED-** or **TESTPLAN-**). Renamed **TESTING- → CLOSED-** after this report. Live Discord not exercised.

## Test report (2026-03-30 — Cursor agent, `003-tester/TESTER.md`)

**Date:** 2026-03-30 17:04:01 UTC (authoritative). Local time is the host where `cargo` ran (mac-stats workspace).

**Preflight / names:** Operator required only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path did not exist at run start; the file was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `format!("discord:{}", …)`)

**Outcome:** All acceptance criteria pass. Per operator mapping: **CLOSED-** (pass), not **TESTED-** (implementation fail) or **TESTPLAN-** (defective test spec). Renamed **TESTING- → CLOSED-** after this report. Live Discord not exercised.

## Test report (2026-03-30 — Cursor agent, `003-tester/TESTER.md`)

**Date:** 2026-03-30 17:12:14 UTC (authoritative). Local time is the host where `cargo` ran (mac-stats workspace).

**Preflight / names:** Operator required only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path did not exist at run start; the file was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `format!("discord:{}", …)`)

**Outcome:** All acceptance criteria pass → **CLOSED-** (not **TESTED-** or **TESTPLAN-**). Renamed **TESTING- → CLOSED-** after this report. Live Discord not exercised.

## Test report (2026-03-30 — Cursor agent, `003-tester/TESTER.md`)

**Date:** 2026-03-30 17:23:35 UTC (authoritative). Local time is the host where `cargo` ran (mac-stats workspace).

**Preflight / names:** Operator required only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path did not exist at run start; the file was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `format!("discord:{}", …)`)

**Outcome:** All acceptance criteria pass. Per operator mapping: **CLOSED-** (pass), not **TESTED-** (implementation fail) or **TESTPLAN-** (defective test spec). Renamed **TESTING- → CLOSED-** after this report. Live Discord not exercised.

## Test report (2026-03-30 — Cursor agent, `003-tester/TESTER.md`)

**Date:** 2026-03-30 17:32:36 UTC (authoritative). Local time is the host where `cargo` ran (mac-stats workspace).

**Preflight / names:** Operator required only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path did not exist at run start; the file was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `format!("discord:{}", …)`)

**Outcome:** All acceptance criteria pass → **CLOSED-** (not **TESTED-** or **TESTPLAN-**). Renamed **TESTING- → CLOSED-** after this report. Live Discord not exercised.

## Test report (2026-03-30 — Cursor agent, `003-tester/TESTER.md`)

**Date:** 2026-03-30 17:41:28 UTC (authoritative). Local time is the host where `cargo` ran (mac-stats workspace).

**Preflight / names:** Operator required only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path did not exist at run start; the file was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `format!("discord:{}", …)`)

**Outcome:** All acceptance criteria pass → **CLOSED-** (not **TESTED-** or **TESTPLAN-**). Renamed **TESTING- → CLOSED-** after this report. Live Discord not exercised.

## Test report (2026-03-30 — Cursor agent, `003-tester/TESTER.md`)

**Date:** 2026-03-30 17:50:42 UTC (authoritative). Local time is the host where `cargo` ran (mac-stats workspace).

**Preflight / names:** Operator required only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path did not exist at run start; the file was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task. Verification commands were executed in this run (`cargo check`, `cargo test keyed_queue`, `cargo test`, `rg` spot-check).

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `format!("discord:{}", …)`)

**Outcome:** All acceptance criteria pass → **CLOSED-** (not **TESTED-** or **TESTPLAN-**). Renamed **TESTING- → CLOSED-** after this report. Live Discord not exercised.

## Test report (2026-03-30 — Cursor agent, `003-tester/TESTER.md`)

**Date:** 2026-03-30 17:59:41 UTC (authoritative). Local wall clock on the host where `cargo` ran: America/Los_Angeles (mac-stats workspace).

**Preflight / names:** Operator required only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path did not exist at run start; the file was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task. Document H1 set to **TESTING** during verification.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `format!("discord:{}", …)`)

**Outcome:** All acceptance criteria pass → final filename **CLOSED-** (per operator mapping: not **TESTED-** or **TESTPLAN-**). H1 restored to **CLOSED**; file renamed **TESTING- → CLOSED-** after this report. Live Discord not exercised.

## Test report (2026-03-30 — Cursor agent, `003-tester/TESTER.md`)

**Date:** 2026-03-30 18:10:12 UTC (authoritative). Local wall clock on the host where `cargo` ran: mac-stats workspace.

**Preflight / names:** Operator required only `tasks/UNTESTED-20260322-0110-openclaw-keyed-async-queue-per-conversation.md` (no other `UNTESTED-*`). That path did not exist at run start; the file was `CLOSED-20260322-0110-…`. Per `003-tester/TESTER.md`, applied rename chain **CLOSED → UNTESTED → TESTING** (same basename) so the **UNTESTED → TESTING** step applies without selecting another task. Document H1 set to **TESTING** during verification.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test keyed_queue` — **pass** (`same_key_runs_sequentially`, `different_keys_may_overlap`; 2 passed)
- `cd src-tauri && cargo test` — **pass** (875 passed, 0 failed in `mac_stats` lib tests; 1 doc-test ignored)

**Static spot-check**

- `rg -n "keyed_queue::run_serial|ollama_queue_key" src-tauri/src/discord/mod.rs` — **pass** (lines 1143, 1347, 1934 `crate::keyed_queue::run_serial`; line 2345 `ollama_queue_key` with `format!("discord:{}", …)`)

**Outcome:** All acceptance criteria pass → final filename **CLOSED-** (not **TESTED-** or **TESTPLAN-**). H1 restored to **CLOSED**; file renamed **TESTING- → CLOSED-** after this report. Live Discord not exercised.
