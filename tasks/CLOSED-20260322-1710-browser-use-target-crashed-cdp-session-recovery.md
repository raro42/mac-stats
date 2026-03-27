# Browser use: Target.targetCrashed → CDP session recovery

## Summary

When the automation tab’s renderer crashes, CDP emits `Target.targetCrashed`. mac-stats uses a side WebSocket (`cdp_target_crash_listener`) with `Target.setDiscoverTargets`, matches the crashed target id to the active automation tab, clears the cached session, logs under `browser/cdp`, and surfaces a one-shot message so the next tool call can reconnect via `with_connection_retry`. Spec: `docs/029_browser_automation.md` (Renderer crash).

## Acceptance criteria

1. `src-tauri/src/browser_agent/cdp_target_crash_listener.rs` spawns the side listener, sends `Target.setDiscoverTargets` **without** invalid `filter: null`, and forwards `Target.targetCrashed` to `notify_target_renderer_crashed_side` when the target id matches tracking.
2. `browser_agent/mod.rs` implements `notify_target_renderer_crashed_side`, `clear_cached_browser_session` on crash, and `debug_page_crash_current_automation_tab` (smoke path used by CLI `--browser-debug-crash-tab`).
3. `main.rs` exposes `--browser-debug-crash-tab` gated on `browserToolsEnabled`.
4. `cargo check` and `cargo test --lib` succeed in `src-tauri/`.

## Verification (automated)

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test --lib
rg -n "targetCrashed|notify_target_renderer_crashed_side|spawn_target_crash_side_listener|debug_page_crash_current_automation_tab" src/browser_agent/cdp_target_crash_listener.rs src/browser_agent/mod.rs src/main.rs
```

Optional smoke (requires browser tools enabled + reachable Chrome on debug port): `cd src-tauri && cargo run --release -- --browser-debug-crash-tab -vv` then confirm `Target.targetCrashed` / session reset lines in `~/.mac-stats/debug.log`.

## Test report

- **Date:** 2026-03-27, hora local del entorno donde se ejecutaron los comandos (no UTC fijada).
- **Preflight:** En el árbol de trabajo **no existía** `tasks/UNTESTED-20260322-1710-browser-use-target-crashed-cdp-session-recovery.md`; se materializó el cuerpo de la tarea como `UNTESTED-…` y se renombró a `TESTING-…` según `003-tester/TESTER.md`. No se usó ningún otro archivo `UNTESTED-*`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed |
| Symbols | `rg -n` con los patrones del bloque «Verification (automated)» sobre `cdp_target_crash_listener.rs`, `browser_agent/mod.rs`, `main.rs` (cwd `src-tauri/`) | **pass** — coincidencias en los tres archivos |

- **Smoke CLI (`--browser-debug-crash-tab`):** se lanzó `target/release/mac_stats --browser-debug-crash-tab -vv`; en consola y en `~/.mac-stats/debug.log` apareció `CDP target-crash side listener: Target.setDiscoverTargets ok (listening for Target.targetCrashed)`. El proceso **no** llegó a imprimir el mensaje final de `Page.crash` / no quedó traza de `Target.targetCrashed` en el log en el tiempo observado (proceso detenido manualmente tras espera prolongada). Se considera **opcional** y **no bloqueante** frente a los criterios 1–4.
- **Outcome:** Criterios de compilación, tests de librería y presencia del cableado CDP/CLI cumplidos → **CLOSED**.

## Test report (2026-03-27, segunda pasada — `003-tester/TESTER.md`)

- **Fecha / zona:** 2026-03-27, hora local del entorno donde se ejecutaron los comandos (no UTC fijada).
- **Preflight:** No existía `tasks/UNTESTED-20260322-1710-browser-use-target-crashed-cdp-session-recovery.md` en el árbol; la tarea estaba como `CLOSED-…`. Se aplicó el flujo renombrando **`CLOSED-…` → `TESTING-…`** (mismo basename). No se tocó ningún otro archivo `UNTESTED-*`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed |
| Symbols | `rg` sobre `cdp_target_crash_listener.rs`, `browser_agent/mod.rs`, `main.rs` con los patrones de «Verification (automated)» | **pass** |

- **Smoke CLI:** No ejecutado en esta pasada (sigue siendo opcional según el cuerpo de la tarea).
- **Outcome:** Criterios 1–4 cumplidos → renombrar **`TESTING-…` → `CLOSED-…`**.

## Test report (2026-03-27 — `003-tester/TESTER.md`, this run)

- **Date / TZ:** 2026-03-27, local time of the environment where commands ran (not fixed UTC).
- **Preflight:** `tasks/UNTESTED-20260322-1710-browser-use-target-crashed-cdp-session-recovery.md` was **not present** in the tree (task was already `CLOSED-…`). No other `UNTESTED-*` file was used. Renamed **`CLOSED-…` → `TESTING-…`** for this verification cycle per `TESTER.md`, then **`TESTING-…` → `CLOSED-…`** after pass.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed |
| Symbols | `rg -n "targetCrashed\|notify_target_renderer_crashed_side\|spawn_target_crash_side_listener\|debug_page_crash_current_automation_tab"` on `src/browser_agent/cdp_target_crash_listener.rs`, `src/browser_agent/mod.rs`, `src/main.rs` (cwd `src-tauri/`) | **pass** |

- **Smoke CLI (`--browser-debug-crash-tab`):** not run (optional per task body).
- **Outcome:** Acceptance criteria 1–4 satisfied → **`CLOSED-…`**.

## Test report (2026-03-27 — `003-tester/TESTER.md`, corrida solicitada por operador)

- **Fecha / zona:** 2026-03-27, hora local del entorno donde se ejecutaron los comandos (no UTC fijada).
- **Preflight:** No existía `tasks/UNTESTED-20260322-1710-browser-use-target-crashed-cdp-session-recovery.md` en el árbol (la tarea estaba como `CLOSED-…`). Se renombró **`CLOSED-…` → `TESTING-…`** para esta verificación. No se tocó ningún otro archivo `UNTESTED-*`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed |
| Symbols | `rg -n "targetCrashed\|notify_target_renderer_crashed_side\|spawn_target_crash_side_listener\|debug_page_crash_current_automation_tab"` sobre `src/browser_agent/cdp_target_crash_listener.rs`, `src/browser_agent/mod.rs`, `src/main.rs` (cwd `src-tauri/`) | **pass** |

- **Smoke CLI (`--browser-debug-crash-tab`):** no ejecutado (opcional según el cuerpo de la tarea).
- **Outcome:** Criterios de aceptación 1–4 cumplidos → renombrar **`TESTING-…` → `CLOSED-…`**.
