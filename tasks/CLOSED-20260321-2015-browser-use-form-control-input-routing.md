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
