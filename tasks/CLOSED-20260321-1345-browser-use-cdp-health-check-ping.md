# Browser use — CDP health check ping (`1+1`)

## Goal

Before CDP browser tools run, mac-stats must detect a hung or dead Chrome while the WebSocket may still look open: optional child-PID liveness (`kill -0` on Unix), then a lightweight **`Runtime.evaluate("1+1")`** “ping” with a **hard wall-clock timeout** on a **plain `std::thread`** + `mpsc::recv_timeout`. This path must **never** nest Tokio `Handle::block_on` + `tokio::time::timeout` on the app’s shared runtime (current-thread executor would wedge).

## Acceptance criteria

1. `evaluate_one_plus_one_blocking_timeout` runs `tab.evaluate("1+1", false)` on a worker thread and uses `recv_timeout(BROWSER_CDP_HEALTH_CHECK_TIMEOUT)`; errors surface as **Browser unresponsive** messages where applicable.
2. `check_browser_alive` calls that helper and includes an explicit comment forbidding nested `block_on` + Tokio timeout (heartbeat / scheduler rationale).
3. On health-check failure, `clear_browser_session_on_error` clears the cached session for **Browser unresponsive** and for connection-style errors (`is_connection_error`), without conflating with unrelated retry paths (`should_retry_cdp_after_clearing_session` documents health wins over retry).

## Verification commands

```bash
rg 'evaluate_one_plus_one_blocking_timeout|check_browser_alive|BROWSER_CDP_HEALTH_CHECK_TIMEOUT|clear_browser_session_on_error' src-tauri/src/browser_agent/mod.rs
rg 'block_on|Never use .Handle::block_on' src-tauri/src/browser_agent/mod.rs | head -n 20
cd src-tauri && cargo check && cargo test --no-fail-fast
```

## Test report

**Fecha:** 2026-03-27 20:12 UTC

**Preflight:** El fichero `tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md` no estaba en el árbol del repo; se creó con el alcance inferido de `src-tauri/src/browser_agent/mod.rs` (`check_browser_alive`, `evaluate_one_plus_one_blocking_timeout`, `clear_browser_session_on_error`, comentarios sobre no anidar `block_on`), y se aplicó el flujo TESTER (UNTESTED → TESTING → este informe → CLOSED).

**Commands run**

- `rg 'evaluate_one_plus_one_blocking_timeout|check_browser_alive|BROWSER_CDP_HEALTH_CHECK_TIMEOUT|clear_browser_session_on_error' src-tauri/src/browser_agent/mod.rs` — **pass**
- `rg 'block_on|Never use .Handle::block_on' src-tauri/src/browser_agent/mod.rs | head -n 20` — **pass** (comentario explícito en `check_browser_alive` prohibiendo `Handle::block_on` + `tokio::time::timeout`; doc en `evaluate_one_plus_one_blocking_timeout` sobre no anidar Tokio `block_on`)
- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test --no-fail-fast` — **pass** (854 tests en el crate lib; 0 fallidos)

**Outcome:** Todos los criterios de aceptación verificados — **CLOSED**.

---

## Test report

**Fecha:** 2026-03-27 20:47 UTC

**Flujo de nombres (TESTER.md):** El operador citó `tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`, que **no existe** en el árbol; la tarea ya estaba en `tasks/CLOSED-20260321-1345-browser-use-cdp-health-check-ping.md`. No se renombró a `TESTING-` porque no había prefijo `UNTESTED-` que mover; no se tocó otro fichero `UNTESTED-*`.

**Commands run**

- `rg 'evaluate_one_plus_one_blocking_timeout|check_browser_alive|BROWSER_CDP_HEALTH_CHECK_TIMEOUT|clear_browser_session_on_error' src-tauri/src/browser_agent/mod.rs` — **pass**
- `rg 'block_on|Never use .Handle::block_on' src-tauri/src/browser_agent/mod.rs | head -n 20` — **pass**
- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test --no-fail-fast` — **pass** (854 passed, 0 failed en el crate lib)

**Outcome:** Criterios de aceptación siguen cumplidos — el fichero de tarea permanece **`CLOSED-20260321-1345-browser-use-cdp-health-check-ping.md`** (sin cambio a `WIP-`).

---

## Test report

**Fecha:** 2026-03-27 21:17 UTC

**Flujo TESTER.md:** El operador pidió `tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`; ese path **no existe** en el repo (solo `tasks/CLOSED-20260321-1345-browser-use-cdp-health-check-ping.md`). No se aplicó renombre `UNTESTED-` → `TESTING-` por ausencia del prefijo. No se probó ningún otro fichero `UNTESTED-*`.

**Commands run**

- `rg 'evaluate_one_plus_one_blocking_timeout|check_browser_alive|BROWSER_CDP_HEALTH_CHECK_TIMEOUT|clear_browser_session_on_error' src-tauri/src/browser_agent/mod.rs` — **pass**
- `rg 'block_on|Never use .Handle::block_on' src-tauri/src/browser_agent/mod.rs | head -n 20` — **pass**
- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test --no-fail-fast` — **pass** (854 passed, 0 failed en crate lib; demás targets 0 tests o ignored doc-test)

**Outcome:** Criterios de aceptación verificados de nuevo — el fichero permanece **`CLOSED-20260321-1345-browser-use-cdp-health-check-ping.md`**.

---

## Test report

**Fecha:** 2026-03-27 21:48 UTC

**Flujo TESTER.md:** `tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md` **no existe** en el repo. La tarea única con ese slug estaba como `CLOSED-…`; se renombró **`CLOSED-` → `TESTING-`** para ejecutar el ciclo de prueba sin tocar ningún otro `UNTESTED-*`.

**Commands run**

- `rg 'evaluate_one_plus_one_blocking_timeout|check_browser_alive|BROWSER_CDP_HEALTH_CHECK_TIMEOUT|clear_browser_session_on_error' src-tauri/src/browser_agent/mod.rs` — **pass**
- `rg 'block_on|Never use .Handle::block_on' src-tauri/src/browser_agent/mod.rs | head -n 20` — **pass**
- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test --no-fail-fast` — **pass** (854 passed, 0 failed en crate `mac_stats` lib; otros binarios 0 tests; 1 doc-test ignored)

**Outcome:** Criterios de aceptación cumplidos — renombrar de vuelta a **`CLOSED-20260321-1345-browser-use-cdp-health-check-ping.md`**.

---

## Test report

**Fecha:** 2026-03-28 01:26 UTC

**Flujo TESTER.md:** El operador pidió `tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`; ese path **no existe** en el repo. La tarea con ese slug estaba como `CLOSED-…`; se aplicó **`CLOSED-` → `TESTING-`** para el ciclo de prueba (equivalente a `UNTESTED-` → `TESTING-` cuando no hay prefijo `UNTESTED-`). No se tocó ningún otro fichero `UNTESTED-*`.

**Commands run**

- `rg 'evaluate_one_plus_one_blocking_timeout|check_browser_alive|BROWSER_CDP_HEALTH_CHECK_TIMEOUT|clear_browser_session_on_error' src-tauri/src/browser_agent/mod.rs` — **pass**
- `rg 'block_on|Never use .Handle::block_on' src-tauri/src/browser_agent/mod.rs | head -n 20` — **pass** (comentario en `check_browser_alive` que prohíbe `Handle::block_on` + `tokio::time::timeout`; doc en `evaluate_one_plus_one_blocking_timeout` sobre no anidar Tokio `block_on`)
- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test --no-fail-fast` — **pass** (854 passed, 0 failed en crate lib `mac_stats`; otros bins 0 tests; 1 doc-test ignored)

**Outcome:** Criterios de aceptación cumplidos — renombrar de vuelta a **`CLOSED-20260321-1345-browser-use-cdp-health-check-ping.md`**.


---

## Test report

**Fecha:** 2026-03-27 22:14 UTC

**Flujo TESTER.md:** El operador citó `tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`, que **no existe** en el repo. La tarea con ese slug estaba como `CLOSED-…`; se aplicó **`CLOSED-` → `TESTING-`** para el ciclo de prueba (equivalente funcional a `UNTESTED-` → `TESTING-` cuando la tarea ya estaba cerrada). No se tocó ningún otro fichero `UNTESTED-*`.

**Commands run**

- `rg 'evaluate_one_plus_one_blocking_timeout|check_browser_alive|BROWSER_CDP_HEALTH_CHECK_TIMEOUT|clear_browser_session_on_error' src-tauri/src/browser_agent/mod.rs` — **pass**
- `rg 'block_on|Never use .Handle::block_on' src-tauri/src/browser_agent/mod.rs | head -n 20` — **pass**
- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test --no-fail-fast` — **pass** (854 passed, 0 failed en crate lib `mac_stats`; otros bins 0 tests; 1 doc-test ignored)

**Outcome:** Criterios de aceptación cumplidos — renombrar de vuelta a **`CLOSED-20260321-1345-browser-use-cdp-health-check-ping.md`**.

---

## Test report

**Fecha:** 2026-03-28 01:26 UTC

**Flujo TESTER.md:** El operador pidió `tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`; ese path **no existe** en el repo. La tarea con ese slug estaba como `CLOSED-…`; se aplicó **`CLOSED-` → `TESTING-`** para el ciclo de prueba (equivalente a `UNTESTED-` → `TESTING-` cuando no hay prefijo `UNTESTED-`). No se tocó ningún otro fichero `UNTESTED-*`.

**Commands run**

- `rg 'evaluate_one_plus_one_blocking_timeout|check_browser_alive|BROWSER_CDP_HEALTH_CHECK_TIMEOUT|clear_browser_session_on_error' src-tauri/src/browser_agent/mod.rs` — **pass**
- `rg 'block_on|Never use .Handle::block_on' src-tauri/src/browser_agent/mod.rs | head -n 20` — **pass** (comentario en `check_browser_alive` que prohíbe `Handle::block_on` + `tokio::time::timeout`; doc en `evaluate_one_plus_one_blocking_timeout` sobre no anidar Tokio `block_on`)
- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test --no-fail-fast` — **pass** (854 passed, 0 failed en crate lib `mac_stats`; otros bins 0 tests; 1 doc-test ignored)

**Outcome:** Criterios de aceptación cumplidos — renombrar de vuelta a **`CLOSED-20260321-1345-browser-use-cdp-health-check-ping.md`**.


---

## Test report

**Fecha:** 2026-03-27 22:44 UTC

**Flujo TESTER.md:** El operador citó `tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`, que **no existe** en el repo. La tarea con ese slug estaba como `CLOSED-…`; se aplicó **`CLOSED-` → `TESTING-`** para el ciclo de prueba (equivalente a `UNTESTED-` → `TESTING-` cuando la tarea ya estaba cerrada). No se tocó ningún otro fichero `UNTESTED-*`.

**Commands run**

- `rg 'evaluate_one_plus_one_blocking_timeout|check_browser_alive|BROWSER_CDP_HEALTH_CHECK_TIMEOUT|clear_browser_session_on_error' src-tauri/src/browser_agent/mod.rs` — **pass**
- `rg 'block_on|Never use .Handle::block_on' src-tauri/src/browser_agent/mod.rs | head -n 20` — **pass** (comentario en `check_browser_alive` que prohíbe `Handle::block_on` + `tokio::time::timeout`; doc en `evaluate_one_plus_one_blocking_timeout` sobre no anidar Tokio `block_on`)
- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test --no-fail-fast` — **pass** (854 passed, 0 failed en crate lib `mac_stats`; otros bins 0 tests; 1 doc-test ignored)

**Outcome:** Criterios de aceptación cumplidos — renombrar de vuelta a **`CLOSED-20260321-1345-browser-use-cdp-health-check-ping.md`**.

---

## Test report

**Fecha:** 2026-03-28 01:26 UTC

**Flujo TESTER.md:** El operador pidió `tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`; ese path **no existe** en el repo. La tarea con ese slug estaba como `CLOSED-…`; se aplicó **`CLOSED-` → `TESTING-`** para el ciclo de prueba (equivalente a `UNTESTED-` → `TESTING-` cuando no hay prefijo `UNTESTED-`). No se tocó ningún otro fichero `UNTESTED-*`.

**Commands run**

- `rg 'evaluate_one_plus_one_blocking_timeout|check_browser_alive|BROWSER_CDP_HEALTH_CHECK_TIMEOUT|clear_browser_session_on_error' src-tauri/src/browser_agent/mod.rs` — **pass**
- `rg 'block_on|Never use .Handle::block_on' src-tauri/src/browser_agent/mod.rs | head -n 20` — **pass** (comentario en `check_browser_alive` que prohíbe `Handle::block_on` + `tokio::time::timeout`; doc en `evaluate_one_plus_one_blocking_timeout` sobre no anidar Tokio `block_on`)
- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test --no-fail-fast` — **pass** (854 passed, 0 failed en crate lib `mac_stats`; otros bins 0 tests; 1 doc-test ignored)

**Outcome:** Criterios de aceptación cumplidos — renombrar de vuelta a **`CLOSED-20260321-1345-browser-use-cdp-health-check-ping.md`**.


---

## Test report

**Fecha:** 2026-03-27 23:14 UTC

**Flujo TESTER.md:** El operador pidió `tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`; ese path **no existe** en el repo (solo la tarea con el mismo slug). Se aplicó **`CLOSED-` → `TESTING-`** para el ciclo de prueba (equivalente a `UNTESTED-` → `TESTING-` cuando no hay prefijo `UNTESTED-`). No se tocó ningún otro fichero `UNTESTED-*`.

**Commands run**

- `rg 'evaluate_one_plus_one_blocking_timeout|check_browser_alive|BROWSER_CDP_HEALTH_CHECK_TIMEOUT|clear_browser_session_on_error' src-tauri/src/browser_agent/mod.rs` — **pass**
- `rg 'block_on|Never use .Handle::block_on' src-tauri/src/browser_agent/mod.rs | head -n 20` — **pass**
- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test --no-fail-fast` — **pass** (854 passed, 0 failed en crate lib `mac_stats`; otros bins 0 tests; 1 doc-test ignored)

**Outcome:** Criterios de aceptación cumplidos — renombrar de vuelta a **`CLOSED-20260321-1345-browser-use-cdp-health-check-ping.md`**.

---

## Test report

**Fecha:** 2026-03-28 01:26 UTC

**Flujo TESTER.md:** El operador pidió `tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`; ese path **no existe** en el repo. La tarea con ese slug estaba como `CLOSED-…`; se aplicó **`CLOSED-` → `TESTING-`** para el ciclo de prueba (equivalente a `UNTESTED-` → `TESTING-` cuando no hay prefijo `UNTESTED-`). No se tocó ningún otro fichero `UNTESTED-*`.

**Commands run**

- `rg 'evaluate_one_plus_one_blocking_timeout|check_browser_alive|BROWSER_CDP_HEALTH_CHECK_TIMEOUT|clear_browser_session_on_error' src-tauri/src/browser_agent/mod.rs` — **pass**
- `rg 'block_on|Never use .Handle::block_on' src-tauri/src/browser_agent/mod.rs | head -n 20` — **pass** (comentario en `check_browser_alive` que prohíbe `Handle::block_on` + `tokio::time::timeout`; doc en `evaluate_one_plus_one_blocking_timeout` sobre no anidar Tokio `block_on`)
- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test --no-fail-fast` — **pass** (854 passed, 0 failed en crate lib `mac_stats`; otros bins 0 tests; 1 doc-test ignored)

**Outcome:** Criterios de aceptación cumplidos — renombrar de vuelta a **`CLOSED-20260321-1345-browser-use-cdp-health-check-ping.md`**.


---

## Test report

**Fecha:** 2026-03-27 23:43 UTC

**Flujo TESTER.md:** El operador pidió `tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`; ese path **no existe** en el repo. La tarea con ese slug estaba como `CLOSED-…`; se aplicó **`CLOSED-` → `TESTING-`** para el ciclo de prueba (equivalente a `UNTESTED-` → `TESTING-` cuando no hay prefijo `UNTESTED-`). No se tocó ningún otro fichero `UNTESTED-*`.

**Commands run**

- `rg 'evaluate_one_plus_one_blocking_timeout|check_browser_alive|BROWSER_CDP_HEALTH_CHECK_TIMEOUT|clear_browser_session_on_error' src-tauri/src/browser_agent/mod.rs` — **pass**
- `rg 'block_on|Never use .Handle::block_on' src-tauri/src/browser_agent/mod.rs | head -n 20` — **pass** (comentario en `check_browser_alive` que prohíbe `Handle::block_on` + `tokio::time::timeout`; doc en `evaluate_one_plus_one_blocking_timeout` sobre no anidar Tokio `block_on`)
- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test --no-fail-fast` — **pass** (854 passed, 0 failed en crate lib `mac_stats`; otros bins 0 tests; 1 doc-test ignored)

**Outcome:** Criterios de aceptación cumplidos — renombrar de vuelta a **`CLOSED-20260321-1345-browser-use-cdp-health-check-ping.md`**.

---

## Test report

**Fecha:** 2026-03-28 01:26 UTC

**Flujo TESTER.md:** El operador pidió `tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`; ese path **no existe** en el repo. La tarea con ese slug estaba como `CLOSED-…`; se aplicó **`CLOSED-` → `TESTING-`** para el ciclo de prueba (equivalente a `UNTESTED-` → `TESTING-` cuando no hay prefijo `UNTESTED-`). No se tocó ningún otro fichero `UNTESTED-*`.

**Commands run**

- `rg 'evaluate_one_plus_one_blocking_timeout|check_browser_alive|BROWSER_CDP_HEALTH_CHECK_TIMEOUT|clear_browser_session_on_error' src-tauri/src/browser_agent/mod.rs` — **pass**
- `rg 'block_on|Never use .Handle::block_on' src-tauri/src/browser_agent/mod.rs | head -n 20` — **pass** (comentario en `check_browser_alive` que prohíbe `Handle::block_on` + `tokio::time::timeout`; doc en `evaluate_one_plus_one_blocking_timeout` sobre no anidar Tokio `block_on`)
- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test --no-fail-fast` — **pass** (854 passed, 0 failed en crate lib `mac_stats`; otros bins 0 tests; 1 doc-test ignored)

**Outcome:** Criterios de aceptación cumplidos — renombrar de vuelta a **`CLOSED-20260321-1345-browser-use-cdp-health-check-ping.md`**.


---

## Test report

**Fecha:** 2026-03-28 00:28 UTC

**Flujo TESTER.md:** El operador citó `tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`; ese path **no existe** en el repo. La tarea con ese slug estaba como `CLOSED-…`; se aplicó **`CLOSED-` → `TESTING-`** para el ciclo de prueba (equivalente a `UNTESTED-` → `TESTING-` cuando no hay prefijo `UNTESTED-`). No se tocó ningún otro fichero `UNTESTED-*`.

**Commands run**

- `rg 'evaluate_one_plus_one_blocking_timeout|check_browser_alive|BROWSER_CDP_HEALTH_CHECK_TIMEOUT|clear_browser_session_on_error' src-tauri/src/browser_agent/mod.rs` — **pass**
- `rg 'block_on|Never use .Handle::block_on' src-tauri/src/browser_agent/mod.rs | head -n 20` — **pass** (comentario en `check_browser_alive` que prohíbe `Handle::block_on` + `tokio::time::timeout`; doc en `evaluate_one_plus_one_blocking_timeout` sobre no anidar Tokio `block_on`)
- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test --no-fail-fast` — **pass** (854 passed, 0 failed en crate lib `mac_stats`; otros bins 0 tests; 1 doc-test ignored)

**Outcome:** Criterios de aceptación cumplidos — renombrar de vuelta a **`CLOSED-20260321-1345-browser-use-cdp-health-check-ping.md`**.

---

## Test report

**Fecha:** 2026-03-28 01:26 UTC

**Flujo TESTER.md:** El operador pidió `tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`; ese path **no existe** en el repo. La tarea con ese slug estaba como `CLOSED-…`; se aplicó **`CLOSED-` → `TESTING-`** para el ciclo de prueba (equivalente a `UNTESTED-` → `TESTING-` cuando no hay prefijo `UNTESTED-`). No se tocó ningún otro fichero `UNTESTED-*`.

**Commands run**

- `rg 'evaluate_one_plus_one_blocking_timeout|check_browser_alive|BROWSER_CDP_HEALTH_CHECK_TIMEOUT|clear_browser_session_on_error' src-tauri/src/browser_agent/mod.rs` — **pass**
- `rg 'block_on|Never use .Handle::block_on' src-tauri/src/browser_agent/mod.rs | head -n 20` — **pass** (comentario en `check_browser_alive` que prohíbe `Handle::block_on` + `tokio::time::timeout`; doc en `evaluate_one_plus_one_blocking_timeout` sobre no anidar Tokio `block_on`)
- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test --no-fail-fast` — **pass** (854 passed, 0 failed en crate lib `mac_stats`; otros bins 0 tests; 1 doc-test ignored)

**Outcome:** Criterios de aceptación cumplidos — renombrar de vuelta a **`CLOSED-20260321-1345-browser-use-cdp-health-check-ping.md`**.

---

## Test report

**Fecha:** 2026-03-28 02:00 UTC

**Flujo TESTER.md:** El operador citó `tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`; ese path **no existe** en el repo. La tarea con ese slug ya estaba como **`TESTING-20260321-1345-browser-use-cdp-health-check-ping.md`** (equivalente a haber aplicado `UNTESTED-` → `TESTING-` antes de esta ejecución). No se renombró ningún otro fichero `UNTESTED-*`.

**Commands run**

- `rg 'evaluate_one_plus_one_blocking_timeout|check_browser_alive|BROWSER_CDP_HEALTH_CHECK_TIMEOUT|clear_browser_session_on_error' src-tauri/src/browser_agent/mod.rs` — **pass**
- `rg 'block_on|Never use .Handle::block_on' src-tauri/src/browser_agent/mod.rs | head -n 20` — **pass** (comentario en `check_browser_alive` que prohíbe `Handle::block_on` + `tokio::time::timeout`; doc en `evaluate_one_plus_one_blocking_timeout` sobre no anidar Tokio `block_on`)
- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test --no-fail-fast` — **pass** (854 passed, 0 failed en crate lib `mac_stats`; otros bins 0 tests; 1 doc-test ignored)

**Outcome:** Criterios de aceptación cumplidos — renombrar a **`CLOSED-20260321-1345-browser-use-cdp-health-check-ping.md`**.
