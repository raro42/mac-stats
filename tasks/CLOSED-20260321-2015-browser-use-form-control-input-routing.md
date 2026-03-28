# Browser use: form control BROWSER_INPUT routing

## Summary

Form-aware **BROWSER_INPUT** (CDP) routes `<select>` (value or label), HTML5 compound inputs (`date`, `time`, etc.) via native value setter + events, datepicker-like text fields, `contenteditable`, and default text fields via focus + keystrokes. Spec: `docs/029_browser_automation.md` § “Form-aware BROWSER_INPUT”. Manual fixture: `docs/fixtures/browser-input-routing.html`.

## Acceptance criteria

1. `input_by_index` / in-page JS distinguishes routes and returns `ok_select`, `ok_native`, `ok_datepicker`, `ok_contenteditable`, or the default typing path; logs include `route_hint` / `path=datepicker_heuristic` where applicable (`src-tauri/src/browser_agent/mod.rs`).
2. Interactable rows expose `input_type`, `contenteditable`, and `datepicker_like` for LLM snapshots.
3. Fixture `docs/fixtures/browser-input-routing.html` exists (select, `input type="date"`, contenteditable).
4. `cargo check` and `cargo test --lib` succeed in `src-tauri/`.

## Verification (automated)

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test --lib
rg -n "ok_select|ok_native|ok_datepicker|ok_contenteditable" src/browser_agent/mod.rs
```

Optional manual: with CDP Chrome, `BROWSER_NAVIGATE` to `file://…/docs/fixtures/browser-input-routing.html` and exercise **BROWSER_INPUT** on listed indices; click “Read values” to confirm.

## Test report

- **Date:** 2026-03-27 (hora local del entorno donde se ejecutaron los comandos; no UTC fijada).
- **Note:** En el árbol de trabajo no existía `UNTESTED-20260321-2015-browser-use-form-control-input-routing.md`; se creó el cuerpo de la tarea y se aplicó el paso **UNTESTED → TESTING** con `mv` para seguir `003-tester/TESTER.md`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed |
| Routing symbols | `rg -n "ok_select\|ok_native\|ok_datepicker\|ok_contenteditable" src/browser_agent/mod.rs` | **pass** — matches at 8003, 8010, 8033, 8160–8161 |

- **Manual CDP / fixture:** no ejecutado en esta corrida (opcional en la tarea).
- **Outcome:** Criterios automatizados y comprobación de rutas en código cumplidos → **CLOSED**.

### Test report — run 2026-03-27 (segunda corrida, `003-tester/TESTER.md`)

- **Date:** 2026-03-27, hora local del entorno de ejecución (no UTC fijada).
- **Note:** El operador indicó `tasks/UNTESTED-20260321-2015-browser-use-form-control-input-routing.md`, que **no estaba presente**; solo existía `CLOSED-…`. Se aplicó el flujo renombrando **`CLOSED-` → `TESTING-`**, ejecutando la verificación de la tarea y, al pasar todo, **`TESTING-` → `CLOSED-`**. No se tocó ningún otro archivo `UNTESTED-*`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed |
| Routing symbols | `rg -n "ok_select\|ok_native\|ok_datepicker\|ok_contenteditable" src/browser_agent/mod.rs` | **pass** — líneas ~8018, 8025, 8048, 8175 |
| Criterio 2 (campos en filas) | revisión en código: `input_type`, `contenteditable`, `datepicker_like` en `InteractableRow` / snapshot JS | **pass** |
| Criterio 3 (fixture) | `docs/fixtures/browser-input-routing.html` presente | **pass** |
| Manual CDP / fixture | — | **no ejecutado** (opcional) |

- **Outcome:** Todos los criterios automatizados y comprobaciones estáticas cumplidos → **`CLOSED-`**.

### Test report — run 2026-03-27 (tercera corrida, `003-tester/TESTER.md`)

- **Date:** 2026-03-27, hora local del entorno de ejecución (no UTC fijada).
- **Note:** No existía `UNTESTED-20260321-2015-browser-use-form-control-input-routing.md`; el archivo estaba como `CLOSED-…`. Se siguió `TESTER.md` con **`CLOSED-` → `TESTING-`**, verificación, informe y **`TESTING-` → `CLOSED-`**. No se probó ningún otro `UNTESTED-*`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed |
| Routing symbols | `rg -n "ok_select\|ok_native\|ok_datepicker\|ok_contenteditable" src/browser_agent/mod.rs` | **pass** — líneas 8018, 8025, 8048, 8175–8176 |
| Criterio 3 (fixture) | `docs/fixtures/browser-input-routing.html` | **pass** (presente) |
| Manual CDP / fixture | — | **no ejecutado** (opcional) |

- **Outcome:** Criterios automatizados de la tarea cumplidos → **`CLOSED-`**.

### Test report — run 2026-03-27 (cuarta corrida, `003-tester/TESTER.md`)

- **Date:** 2026-03-27, hora local del entorno de ejecución (no UTC fijada).
- **Note:** El operador nombró `tasks/UNTESTED-20260321-2015-browser-use-form-control-input-routing.md`, que **no existía** en el árbol; el artefacto presente era `CLOSED-…`. Flujo aplicado: **`CLOSED-` → `TESTING-`**, verificación según cuerpo de la tarea, informe, **`TESTING-` → `CLOSED-`**. No se probó ningún otro `UNTESTED-*`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed |
| Routing symbols | `rg -n "ok_select\|ok_native\|ok_datepicker\|ok_contenteditable" src/browser_agent/mod.rs` | **pass** — líneas 8018, 8025, 8048, 8175–8176 |
| Criterio 2 (InteractableRow / snapshot) | `input_type`, `contenteditable`, `datepicker_like` presentes en `mod.rs` (struct + JS out.push) | **pass** |
| Criterio 3 (fixture) | `docs/fixtures/browser-input-routing.html` | **pass** (presente) |
| Manual CDP / fixture | — | **no ejecutado** (opcional) |

- **Outcome:** Criterios automatizados y comprobaciones estáticas cumplidos → **`CLOSED-`**.

### Test report — run 2026-03-27 (quinta corrida, `003-tester/TESTER.md`)

- **Date:** 2026-03-27, hora local del entorno de ejecución (no UTC fijada).
- **Note:** El operador indicó `tasks/UNTESTED-20260321-2015-browser-use-form-control-input-routing.md`, que **no existía**; el archivo estaba como `CLOSED-…`. Se aplicó **`CLOSED-` → `TESTING-`**, verificación según el cuerpo de la tarea, informe y **`TESTING-` → `CLOSED-`**. No se probó ningún otro `UNTESTED-*`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed |
| Routing symbols | `rg -n "ok_select\|ok_native\|ok_datepicker\|ok_contenteditable" src/browser_agent/mod.rs` | **pass** — líneas 8018, 8025, 8048, 8175–8176 |
| Criterio 2 (InteractableRow / snapshot) | `input_type`, `contenteditable`, `datepicker_like` en struct/JS | **pass** |
| Criterio 3 (fixture) | `docs/fixtures/browser-input-routing.html` | **pass** (presente) |
| Manual CDP / fixture | — | **no ejecutado** (opcional) |

- **Outcome:** Criterios automatizados y comprobaciones estáticas cumplidos → **`CLOSED-`**.

### Test report — run 2026-03-27 (sexta corrida, `003-tester/TESTER.md`)

- **Date:** 2026-03-27, hora local del entorno de ejecución (no UTC fijada).
- **Note:** El operador indicó `tasks/UNTESTED-20260321-2015-browser-use-form-control-input-routing.md`, que **no existía** en el árbol; el archivo estaba como `CLOSED-…`. Se aplicó **`CLOSED-` → `TESTING-`**, verificación según el cuerpo de la tarea, informe y **`TESTING-` → `CLOSED-`**. No se probó ningún otro `UNTESTED-*`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed |
| Routing symbols | `rg -n "ok_select\|ok_native\|ok_datepicker\|ok_contenteditable" src/browser_agent/mod.rs` | **pass** — líneas 8018, 8025, 8048, 8175–8176 |
| Criterio 2 (InteractableRow / snapshot) | `input_type`, `contenteditable`, `datepicker_like` en struct/JS | **pass** |
| Criterio 3 (fixture) | `docs/fixtures/browser-input-routing.html` | **pass** (presente) |
| Manual CDP / fixture | — | **no ejecutado** (opcional) |

- **Outcome:** Criterios automatizados y comprobaciones estáticas cumplidos → **`CLOSED-`**.

### Test report — run 2026-03-27 (séptima corrida, `003-tester/TESTER.md`)

- **Date:** 2026-03-27, local environment time (not fixed to UTC).
- **Note:** Operator asked for `tasks/UNTESTED-20260321-2015-browser-use-form-control-input-routing.md`; that path **does not exist** in the repo. The task file was `CLOSED-…`. Per `TESTER.md`, workflow applied: **`CLOSED-` → `TESTING-`**, run verification from the task body, append this report, then **`TESTING-` → `CLOSED-`**. No other `UNTESTED-*` file was used.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed |
| Routing symbols | `rg -n "ok_select\|ok_native\|ok_datepicker\|ok_contenteditable" src/browser_agent/mod.rs` | **pass** — lines 8018, 8025, 8048, 8175–8176 |
| Criterion 2 (InteractableRow / snapshot) | `input_type`, `contenteditable`, `datepicker_like` in struct + JS `out.push` | **pass** |
| Criterion 3 (fixture) | `docs/fixtures/browser-input-routing.html` | **pass** (present) |
| Manual CDP / fixture | — | **not run** (optional) |

- **Outcome:** Automated acceptance criteria satisfied → **`CLOSED-`**.

### Test report — run 2026-03-28 (`003-tester/TESTER.md`)

- **Date:** 2026-03-28, hora local del entorno de ejecución (no UTC fijada).
- **Note:** El operador nombró `tasks/UNTESTED-20260321-2015-browser-use-form-control-input-routing.md`, que **no existía**; el archivo estaba como `CLOSED-…`. Flujo `TESTER.md` para esta tarea únicamente: **`CLOSED-` → `TESTING-`**, verificación del cuerpo de la tarea, este informe, **`TESTING-` → `CLOSED-`**. No se usó ningún otro `UNTESTED-*`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed |
| Routing symbols | `rg -n "ok_select\|ok_native\|ok_datepicker\|ok_contenteditable" src/browser_agent/mod.rs` | **pass** — líneas 8018, 8025, 8048, 8175–8176 |
| Criterio 2 (InteractableRow / snapshot) | `input_type`, `contenteditable`, `datepicker_like` en struct + JS | **pass** |
| Criterio 3 (fixture) | `docs/fixtures/browser-input-routing.html` | **pass** (presente) |
| Manual CDP / fixture | — | **no ejecutado** (opcional) |

- **Outcome:** Criterios automatizados y comprobaciones estáticas cumplidos → **`CLOSED-`**.

### Test report — run 2026-03-28 (`003-tester/TESTER.md`, segunda corrida del día, agente)

- **Date:** 2026-03-28, hora local del entorno de ejecución (no UTC fijada).
- **Note:** El operador indicó `tasks/UNTESTED-20260321-2015-browser-use-form-control-input-routing.md`, que **no existe**; el archivo estaba como `CLOSED-…`. Flujo `TESTER.md` para **solo esta tarea**: **`CLOSED-` → `TESTING-`**, verificación del cuerpo de la tarea, este informe, **`TESTING-` → `CLOSED-`**. No se usó ningún otro `UNTESTED-*`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed |
| Routing symbols | `rg -n "ok_select\|ok_native\|ok_datepicker\|ok_contenteditable" src/browser_agent/mod.rs` | **pass** — líneas 8018, 8025, 8048, 8175–8176 |
| Criterio 1 (logs) | `route_hint` / `input_type` / `datepicker_like` en trazas `BROWSER_INPUT` (`mod.rs` ~8078) | **pass** (revisión estática) |
| Criterio 2 (InteractableRow / snapshot) | `input_type`, `contenteditable`, `datepicker_like` en struct + `out.push` | **pass** |
| Criterio 3 (fixture) | `docs/fixtures/browser-input-routing.html` | **pass** (presente) |
| Manual CDP / fixture | — | **no ejecutado** (opcional) |

- **Outcome:** Criterios automatizados y comprobaciones estáticas cumplidos → **`CLOSED-`**.

### Test report — run 2026-03-28 (`003-tester/TESTER.md`, tercera corrida del día)

- **Date:** 2026-03-28, hora local del entorno de ejecución (no UTC fijada).
- **Note:** El operador indicó `tasks/UNTESTED-20260321-2015-browser-use-form-control-input-routing.md`, que **no existe** en el repo; el archivo estaba como `CLOSED-…`. Flujo `TESTER.md` para **solo esta tarea**: **`CLOSED-` → `TESTING-`**, verificación del cuerpo de la tarea, este informe, **`TESTING-` → `CLOSED-`**. No se usó ningún otro `UNTESTED-*`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed |
| Routing symbols | `rg -n "ok_select\|ok_native\|ok_datepicker\|ok_contenteditable" src/browser_agent/mod.rs` | **pass** — líneas 8018, 8025, 8048, 8175–8176 |
| Criterio 1 (logs) | `route_hint` en traza `BROWSER_INPUT` (`mod.rs` ~8078) con `input_type`, `datepicker_like` | **pass** (revisión estática) |
| Criterio 2 (InteractableRow / snapshot) | `input_type`, `contenteditable`, `datepicker_like` en struct + `out.push` | **pass** |
| Criterio 3 (fixture) | `docs/fixtures/browser-input-routing.html` | **pass** (presente) |
| Manual CDP / fixture | — | **no ejecutado** (opcional) |

- **Outcome:** Criterios automatizados y comprobaciones estáticas cumplidos → **`CLOSED-`**.

### Test report — run 2026-03-28 (`003-tester/TESTER.md`, cuarta corrida del día)

- **Date:** 2026-03-28, hora local del entorno de ejecución (no UTC fijada).
- **Note:** El operador indicó `tasks/UNTESTED-20260321-2015-browser-use-form-control-input-routing.md`, que **no existe**; el archivo estaba como `CLOSED-…`. Flujo `TESTER.md` para **solo esta tarea**: **`CLOSED-` → `TESTING-`**, verificación del cuerpo de la tarea, este informe, **`TESTING-` → `CLOSED-`**. No se usó ningún otro `UNTESTED-*`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed |
| Routing symbols | `rg -n "ok_select\|ok_native\|ok_datepicker\|ok_contenteditable" src/browser_agent/mod.rs` | **pass** — líneas 8018, 8025, 8048, 8175–8176 |
| Criterio 1 (logs / rutas) | `route_hint` en traza `BROWSER_INPUT` (`mod.rs` ~8078) con `input_type`, `datepicker_like` | **pass** (revisión estática) |
| Criterio 2 (InteractableRow / snapshot) | `input_type`, `contenteditable`, `datepicker_like` en struct + `out.push` | **pass** |
| Criterio 3 (fixture) | `docs/fixtures/browser-input-routing.html` | **pass** (presente) |
| Manual CDP / fixture | — | **no ejecutado** (opcional) |

- **Outcome:** Criterios automatizados y comprobaciones estáticas cumplidos → **`CLOSED-`**.

### Test report — run 2026-03-28 (`003-tester/TESTER.md`, quinta corrida del día)

- **Date:** 2026-03-28, hora local del entorno de ejecución (no UTC fijada).
- **Note:** El operador indicó `tasks/UNTESTED-20260321-2015-browser-use-form-control-input-routing.md`, que **no existía**; el archivo estaba como `CLOSED-…`. Flujo `TESTER.md` para **solo esta tarea**: **`CLOSED-` → `TESTING-`**, verificación del cuerpo de la tarea, este informe, **`TESTING-` → `CLOSED-`**. No se usó ningún otro `UNTESTED-*`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed |
| Routing symbols | `rg -n "ok_select\|ok_native\|ok_datepicker\|ok_contenteditable" src/browser_agent/mod.rs` | **pass** — líneas 8018, 8025, 8048, 8175–8176 |
| Criterio 1 (logs / rutas) | `route_hint` (~8078), `path=datepicker_heuristic` (~8179) en `mod.rs` | **pass** (revisión estática) |
| Criterio 2 (InteractableRow / snapshot) | `input_type`, `contenteditable`, `datepicker_like` en struct + `out.push` | **pass** |
| Criterio 3 (fixture) | `docs/fixtures/browser-input-routing.html` | **pass** (presente) |
| Manual CDP / fixture | — | **no ejecutado** (opcional) |

- **Outcome:** Criterios automatizados y comprobaciones estáticas cumplidos → **`CLOSED-`**.

### Test report — run 2026-03-28 (`003-tester/TESTER.md`, sesión actual)

- **Date:** 2026-03-28, hora local del entorno de ejecución (no UTC fijada).
- **Note:** El path `tasks/UNTESTED-20260321-2015-browser-use-form-control-input-routing.md` **no existe** en el repo; el artefacto era `CLOSED-…`. Flujo `TESTER.md` para **solo esta tarea**: **`CLOSED-` → `TESTING-`**, verificación del cuerpo de la tarea, este informe, **`TESTING-` → `CLOSED-`**. No se usó ningún otro `UNTESTED-*`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed |
| Routing symbols | `rg -n "ok_select\|ok_native\|ok_datepicker\|ok_contenteditable" src/browser_agent/mod.rs` | **pass** — líneas 8018, 8025, 8048, 8175–8176 |
| Criterio 1 (logs / rutas) | `route_hint` (~8078), `path=datepicker_heuristic` (~8179) en `mod.rs` | **pass** (revisión estática) |
| Criterio 2 (InteractableRow / snapshot) | `input_type`, `contenteditable`, `datepicker_like` en struct + `out.push` | **pass** |
| Criterio 3 (fixture) | `docs/fixtures/browser-input-routing.html` | **pass** (presente) |
| Manual CDP / fixture | — | **no ejecutado** (opcional) |

- **Outcome:** Criterios automatizados y comprobaciones estáticas cumplidos → **`CLOSED-`**.

### Test report — run 2026-03-28 (`003-tester/TESTER.md`, corrida agente Cursor)

- **Date:** 2026-03-28, hora local del entorno de ejecución (no UTC fijada).
- **Note:** El operador indicó `tasks/UNTESTED-20260321-2015-browser-use-form-control-input-routing.md`, que **no existe** en el árbol; el archivo estaba como `CLOSED-…`. Flujo `TESTER.md` para **solo esta tarea**: **`CLOSED-` → `TESTING-`**, verificación del cuerpo de la tarea, este informe, **`TESTING-` → `CLOSED-`**. No se usó ningún otro `UNTESTED-*`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed (≈1.16s) |
| Routing symbols | `rg -n "ok_select\|ok_native\|ok_datepicker\|ok_contenteditable" src/browser_agent/mod.rs` | **pass** — líneas 8018, 8025, 8048, 8175–8176 |
| Criterio 1 (logs) | `route_hint` (~8078), `path=datepicker_heuristic` (~8179) en `mod.rs` | **pass** (revisión estática) |
| Criterio 2 (InteractableRow / snapshot) | `input_type`, `contenteditable`, `datepicker_like` en struct (~79–83) y `out.push` (~1350, ~1800) | **pass** |
| Criterio 3 (fixture) | `docs/fixtures/browser-input-routing.html` | **pass** (presente) |
| Manual CDP / fixture | — | **no ejecutado** (opcional) |

- **Outcome:** Criterios automatizados y comprobaciones estáticas cumplidos → **`CLOSED-`**.

### Test report — run 2026-03-28 (`003-tester/TESTER.md`, sesión Cursor — operador)

- **Date:** 2026-03-28, hora local del entorno de ejecución (no UTC fijada).
- **Note:** `tasks/UNTESTED-20260321-2015-browser-use-form-control-input-routing.md` **no estaba en el repo**; no se probó ningún otro `UNTESTED-*`. Se aplicó **`CLOSED-` → `TESTING-`**, verificación automática del cuerpo de la tarea, este informe y **`TESTING-` → `CLOSED-`**.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed (~1.16s) |
| Routing symbols | `rg -n "ok_select\|ok_native\|ok_datepicker\|ok_contenteditable" src/browser_agent/mod.rs` | **pass** — líneas 8018, 8025, 8048, 8175–8176 |
| Criterio 1 (logs / rutas) | `route_hint` (~8078), `path=datepicker_heuristic` (~8179) en `mod.rs` | **pass** (revisión estática) |
| Criterio 2 (InteractableRow / snapshot) | `input_type`, `contenteditable`, `datepicker_like` en struct (~79–83) y `out.push` (~1350, ~1800) | **pass** |
| Criterio 3 (fixture) | `docs/fixtures/browser-input-routing.html` | **pass** (presente) |
| Manual CDP / fixture | — | **no ejecutado** (opcional) |

- **Outcome:** Criterios automatizados y comprobaciones estáticas cumplidos → **`CLOSED-`**.

### Test report — run 2026-03-28 (`003-tester/TESTER.md`, agente Auto)

- **Date:** 2026-03-28, hora local del entorno de ejecución (no UTC fijada).
- **Note:** `tasks/UNTESTED-20260321-2015-browser-use-form-control-input-routing.md` **no existe** en el repo; la tarea estaba como `CLOSED-…`. Flujo `TESTER.md` para **solo esta tarea**: **`CLOSED-` → `TESTING-`**, verificación del cuerpo de la tarea, este informe, **`TESTING-` → `CLOSED-`**. No se usó ningún otro `UNTESTED-*`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed; finished in 1.16s |
| Routing symbols | `rg -n "ok_select\|ok_native\|ok_datepicker\|ok_contenteditable" src/browser_agent/mod.rs` | **pass** — líneas 8018, 8025, 8048, 8175–8176 |
| Criterio 1 (logs) | `route_hint` (8078), `path=datepicker_heuristic` (8179) en `mod.rs` | **pass** (revisión estática) |
| Criterio 2 (InteractableRow / snapshot) | `input_type`, `contenteditable`, `datepicker_like` en struct (~79–83) y `out.push` (~1350, ~1800) | **pass** |
| Criterio 3 (fixture) | `docs/fixtures/browser-input-routing.html` | **pass** (presente) |
| Manual CDP / fixture | — | **no ejecutado** (opcional) |

- **Outcome:** Criterios automatizados y comprobaciones estáticas cumplidos → **`CLOSED-`**.

### Test report — run 2026-03-28 (`003-tester/TESTER.md`, Cursor)

- **Date:** 2026-03-28, local workspace time (not fixed to UTC).
- **Note:** `tasks/UNTESTED-20260321-2015-browser-use-form-control-input-routing.md` **does not exist** in the repo; this run used **`CLOSED-` → `TESTING-`**, executed verification from the task body, appended this report, then **`TESTING-` → `CLOSED-`**. No other `UNTESTED-*` file was tested.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed; finished in 1.16s |
| Routing symbols | `rg -n "ok_select\|ok_native\|ok_datepicker\|ok_contenteditable" src/browser_agent/mod.rs` | **pass** — lines 8018, 8025, 8048, 8175–8176 |
| Criterion 2 (snapshot fields) | `input_type`, `contenteditable`, `datepicker_like` on `InteractableRow` / snapshot (same as prior static review) | **pass** |
| Criterion 3 (fixture) | `docs/fixtures/browser-input-routing.html` | **pass** (present) |
| Manual CDP / fixture | — | **not run** (optional per task) |

- **Outcome:** Automated acceptance criteria satisfied → **`CLOSED-`**.

### Test report — run 2026-03-28 (`003-tester/TESTER.md`)

- **Date:** 2026-03-28, local workspace time (not fixed to UTC).
- **Note:** `tasks/UNTESTED-20260321-2015-browser-use-form-control-input-routing.md` was **not present**; this task file was `CLOSED-…`. Per `TESTER.md` for this task only: **`CLOSED-` → `TESTING-`**, verification from the task body, this report, then **`TESTING-` → `CLOSED-`**. No other `UNTESTED-*` file was used.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed; finished in 1.16s |
| Routing symbols | `rg -n "ok_select\|ok_native\|ok_datepicker\|ok_contenteditable" src/browser_agent/mod.rs` | **pass** — lines 8018, 8025, 8048, 8175–8176 |
| Criterion 1 (logs) | `route_hint` (8078), `path=datepicker_heuristic` (8179) in `mod.rs` | **pass** (static review) |
| Criterion 2 (snapshot fields) | `input_type`, `contenteditable`, `datepicker_like` on `InteractableRow` / snapshot JS (unchanged vs prior static review) | **pass** |
| Criterion 3 (fixture) | `docs/fixtures/browser-input-routing.html` | **pass** (present) |
| Manual CDP / fixture | — | **not run** (optional per task) |

- **Outcome:** Automated acceptance criteria satisfied → **`CLOSED-`**.

### Test report — run 2026-03-28 (`003-tester/TESTER.md`, segunda corrida del día)

- **Date:** 2026-03-28, hora local del workspace (no fijada a UTC).
- **Note:** El path `tasks/UNTESTED-20260321-2015-browser-use-form-control-input-routing.md` **no existe** en el repo; esta corrida usó solo este archivo: **`CLOSED-` → `TESTING-`**, verificación según el cuerpo de la tarea, este informe y **`TESTING-` → `CLOSED-`**. No se probó ningún otro `UNTESTED-*`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed; finished in 1.16s |
| Routing symbols | `rg -n "ok_select\|ok_native\|ok_datepicker\|ok_contenteditable" src/browser_agent/mod.rs` | **pass** — líneas 8018, 8025, 8048, 8175–8176 |
| Criterio 1 (logs) | `route_hint` (8078), `path=datepicker_heuristic` (8179) en `mod.rs` | **pass** |
| Criterio 2 (filas / snapshot) | `input_type`, `contenteditable`, `datepicker_like` en `InteractableRow` y `out.push` (p. ej. 79–83, 1350) | **pass** |
| Criterio 3 (fixture) | `docs/fixtures/browser-input-routing.html` | **pass** (presente) |
| Manual CDP / fixture | — | **no ejecutado** (opcional en la tarea) |

- **Outcome:** Criterios de aceptación automatizados cumplidos → **`CLOSED-`**.

### Test report — run 2026-03-28 (`003-tester/TESTER.md`, corrida agente)

- **Date:** 2026-03-28, hora local del entorno de ejecución (no fijada a UTC).
- **Note:** El operador indicó `tasks/UNTESTED-20260321-2015-browser-use-form-control-input-routing.md`, que **no existe** en el árbol; el archivo estaba como `CLOSED-…`. Flujo `TESTER.md` para **solo esta tarea**: **`CLOSED-` → `TESTING-`**, verificación del cuerpo de la tarea, este informe, **`TESTING-` → `CLOSED-`**. No se usó ningún otro `UNTESTED-*`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed; finished in 1.16s |
| Routing symbols | `rg -n "ok_select\|ok_native\|ok_datepicker\|ok_contenteditable" src/browser_agent/mod.rs` | **pass** — líneas 8018, 8025, 8048, 8175–8176 |
| Criterio 2 (InteractableRow / snapshot) | revisión estática: `input_type`, `contenteditable`, `datepicker_like` en `mod.rs` | **pass** |
| Criterio 3 (fixture) | `test -f docs/fixtures/browser-input-routing.html` | **pass** |
| Manual CDP / fixture | — | **no ejecutado** (opcional en la tarea) |

- **Outcome:** Criterios automatizados cumplidos → **`CLOSED-`**.

### Test report — run 2026-03-28 (`003-tester/TESTER.md`, corrida agente Auto)

- **Date:** 2026-03-28, hora local del entorno de ejecución (no fijada a UTC).
- **Note:** El path `tasks/UNTESTED-20260321-2015-browser-use-form-control-input-routing.md` **no existe** en el repo; esta corrida aplicó **`CLOSED-` → `TESTING-`**, ejecutó la verificación del cuerpo de la tarea, añadió este informe y **`TESTING-` → `CLOSED-`**. No se probó ningún otro `UNTESTED-*`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed; finished in 1.16s |
| Routing symbols | `rg -n "ok_select\|ok_native\|ok_datepicker\|ok_contenteditable" src/browser_agent/mod.rs` (desde `src-tauri/`) | **pass** — líneas 8018, 8025, 8048, 8175–8176 |
| Criterio 1 (logs / rutas) | `route_hint` (8078), `path=datepicker_heuristic` (8179) en `mod.rs` | **pass** (revisión estática) |
| Criterio 2 (InteractableRow / snapshot) | `input_type`, `contenteditable`, `datepicker_like` en struct (~79–83) y `out.push` (~1350, ~1800) | **pass** |
| Criterio 3 (fixture) | `docs/fixtures/browser-input-routing.html` | **pass** (presente) |
| Manual CDP / fixture | — | **no ejecutado** (opcional en la tarea) |

- **Outcome:** Criterios de aceptación automatizados cumplidos → **`CLOSED-`**.

### Test report — run 2026-03-28 (`003-tester/TESTER.md`)

- **Date:** 2026-03-28, hora local del entorno de ejecución (no fijada a UTC).
- **Note:** El path pedido `tasks/UNTESTED-20260321-2015-browser-use-form-control-input-routing.md` **no existe** en el repo; esta corrida actuó solo sobre esta tarea renombrando **`CLOSED-` → `TESTING-`**, ejecutando la verificación del cuerpo de la tarea, añadiendo este informe y **`TESTING-` → `CLOSED-`**. No se probó ningún otro archivo `UNTESTED-*`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed; finished in 1.17s |
| Routing symbols | `rg -n "ok_select\|ok_native\|ok_datepicker\|ok_contenteditable" src/browser_agent/mod.rs` (desde `src-tauri/`) | **pass** — líneas 8018, 8025, 8048, 8175–8176 |
| Criterio 1 (logs) | `route_hint` (8078), `path=datepicker_heuristic` (8179) en `mod.rs` | **pass** (revisión estática) |
| Criterio 2 (InteractableRow / snapshot) | `input_type`, `contenteditable`, `datepicker_like` en struct (~79–83) y `out.push` (~1350, ~1800) | **pass** |
| Criterio 3 (fixture) | `test -f docs/fixtures/browser-input-routing.html` | **pass** |
| Manual CDP / fixture | — | **no ejecutado** (opcional en la tarea) |

- **Outcome:** Todos los criterios automatizados y comprobaciones estáticas cumplidos → **`CLOSED-`**.

### Test report — run 2026-03-28 (`003-tester/TESTER.md`, repetición misma tarea UNTESTED nombrada)

- **Date:** 2026-03-28, hora local del entorno de ejecución (no fijada a UTC).
- **Note:** `tasks/UNTESTED-20260321-2015-browser-use-form-control-input-routing.md` **no existe**; se aplicó **`CLOSED-` → `TESTING-`**, verificación, este informe y **`TESTING-` → `CLOSED-`**. Ningún otro `UNTESTED-*` probado.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed; finished in 1.16s |
| Routing symbols | `rg -n "ok_select\|ok_native\|ok_datepicker\|ok_contenteditable" src/browser_agent/mod.rs` (desde `src-tauri/`) | **pass** — líneas 8018, 8025, 8048, 8175–8176 |
| Criterio 1 (logs / rutas) | `route_hint` (~8078), `path=datepicker_heuristic` (~8179) en `mod.rs` | **pass** (revisión estática) |
| Criterio 2 (InteractableRow / snapshot) | `input_type`, `contenteditable`, `datepicker_like` en struct (~79–83) y `out.push` (~1350, ~1800) | **pass** |
| Criterio 3 (fixture) | `test -f docs/fixtures/browser-input-routing.html` | **pass** |
| Manual CDP / fixture | — | **no ejecutado** (opcional en la tarea) |

- **Outcome:** Criterios automatizados cumplidos → **`CLOSED-`**.

### Test report — run 2026-03-28 (`003-tester/TESTER.md`, operator-named UNTESTED path)

- **Date:** 2026-03-28, local environment time (not fixed to UTC).
- **Note:** `tasks/UNTESTED-20260321-2015-browser-use-form-control-input-routing.md` **does not exist** in the repo; this run used only this task file via **`CLOSED-` → `TESTING-`**, ran verification from the task body, appended this report, then **`TESTING-` → `CLOSED-`**. No other `UNTESTED-*` file was tested.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed; finished in 1.16s |
| Routing symbols | `rg -n "ok_select\|ok_native\|ok_datepicker\|ok_contenteditable" src/browser_agent/mod.rs` (from `src-tauri/`) | **pass** — lines 8018, 8025, 8048, 8175–8176 |
| Criterion 1 (logs / routes) | `route_hint` (8078), `path=datepicker_heuristic` (8179) in `mod.rs` | **pass** (static review) |
| Criterion 2 (InteractableRow / snapshot) | `input_type`, `contenteditable`, `datepicker_like` on struct (~79–83) and `out.push` (~1350, ~1800) | **pass** |
| Criterion 3 (fixture) | `docs/fixtures/browser-input-routing.html` | **pass** (present) |
| Manual CDP / fixture | — | **not run** (optional in task) |

- **Outcome:** Automated acceptance criteria satisfied → **`CLOSED-`**.

### Test report — run 2026-03-28 (`003-tester/TESTER.md`, corrida Cursor)

- **Date:** 2026-03-28, hora local del entorno de ejecución (no fijada a UTC).
- **Note:** El path `tasks/UNTESTED-20260321-2015-browser-use-form-control-input-routing.md` **no existe** en el repo; la tarea estaba como `CLOSED-…`. Flujo `TESTER.md` para **solo esta tarea**: **`CLOSED-` → `TESTING-`**, verificación del cuerpo de la tarea, este informe, **`TESTING-` → `CLOSED-`**. No se probó ningún otro `UNTESTED-*`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed; finished in 1.16s |
| Routing symbols | `rg -n "ok_select\|ok_native\|ok_datepicker\|ok_contenteditable" src/browser_agent/mod.rs` (desde `src-tauri/`) | **pass** — líneas 8018, 8025, 8048, 8175–8176 |
| Criterio 1 (logs) | `route_hint` (8078), `path=datepicker_heuristic` (8179) en `mod.rs` | **pass** (revisión estática) |
| Criterio 2 (InteractableRow / snapshot) | `input_type`, `contenteditable`, `datepicker_like` en struct y snapshot JS (`mod.rs`) | **pass** (revisión estática) |
| Criterio 3 (fixture) | `test -f docs/fixtures/browser-input-routing.html` | **pass** |
| Manual CDP / fixture | — | **no ejecutado** (opcional en la tarea) |

- **Outcome:** Criterios de aceptación automatizados cumplidos → **`CLOSED-`**.

### Test report — run 2026-03-28 (`003-tester/TESTER.md`, segunda corrida agente Cursor)

- **Date:** 2026-03-28, hora local del entorno de ejecución (no fijada a UTC).
- **Note:** El path `tasks/UNTESTED-20260321-2015-browser-use-form-control-input-routing.md` **no existe** en el repo; la tarea estaba como `CLOSED-…`. Flujo `TESTER.md` para **solo esta tarea**: **`CLOSED-` → `TESTING-`**, verificación del cuerpo de la tarea, este informe, **`TESTING-` → `CLOSED-`**. No se probó ningún otro archivo `UNTESTED-*`.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed; finished in 1.16s |
| Routing symbols | `rg -n "ok_select\|ok_native\|ok_datepicker\|ok_contenteditable" src/browser_agent/mod.rs` (desde `src-tauri/`) | **pass** — líneas 8018, 8025, 8048, 8175–8176 |
| Criterio 1 (logs / rutas) | `route_hint` (8078), `path=datepicker_heuristic` (8179) en `mod.rs` | **pass** (revisión estática) |
| Criterio 2 (InteractableRow / snapshot) | `input_type`, `contenteditable`, `datepicker_like` en struct (~79–83) y `out.push` (~1350, ~1800) | **pass** (revisión estática) |
| Criterio 3 (fixture) | `test -f docs/fixtures/browser-input-routing.html` | **pass** |
| Manual CDP / fixture | — | **no ejecutado** (opcional en la tarea) |

- **Outcome:** Todos los criterios automatizados y comprobaciones estáticas cumplidos → **`CLOSED-`**.
