# mac-stats: OpenClaw-style managed tab cap (prune excess CDP tabs)

## Summary

When `browserMaxPageTabs` / `MAC_STATS_BROWSER_MAX_PAGE_TABS` is a positive integer, after successful browser tools (navigate with optional `new_tab`, click, hover, drag, screenshot-with-URL), mac-stats prunes **other** page tabs until the count is at most the cap, keeping the focused automation tab. Aligns with OpenClaw-style tab discipline.

## Acceptance criteria

1. `browser_agent/mod.rs` implements `try_enforce_browser_tab_limit` and invokes it after successful operations that can grow or touch the tab set (navigate paths, etc.).
2. `Config::browser_max_page_tabs()` reads config/env (default 0 = disabled; positive enables enforcement).
3. `examples/managed_tab_cap_smoke.rs` documents and exercises cap enforcement across sequential `new_tab` navigations.
4. `cargo check` and `cargo test --lib` succeed in `src-tauri/`.

## Verification (automated)

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test --lib
rg -n "fn try_enforce_browser_tab_limit" src/browser_agent/mod.rs
rg -n "try_enforce_browser_tab_limit\\(" src/browser_agent/mod.rs
```

## Verification (optional — needs Chromium with CDP, e.g. port 9222)

```bash
cd src-tauri && MAC_STATS_BROWSER_MAX_PAGE_TABS=3 cargo run --example managed_tab_cap_smoke
```

Skip or note **blocked** if no CDP browser is available; automated criteria 1–4 still gate **CLOSED**.

## Test report

- **Date:** 2026-03-27, hora local del entorno donde se ejecutaron los comandos (no UTC fijo).
- **Preflight:** En el árbol de trabajo no existía `tasks/UNTESTED-20260322-1901-openclaw-managed-tab-cap-prune.md`; se creó el cuerpo de la tarea como `UNTESTED-…` y se siguió `003-tester/TESTER.md` (renombrado a `TESTING-…`). No se usó ningún otro `UNTESTED-*` en esta corrida.

| Paso | Comando | Resultado |
|------|---------|-----------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed |
| Símbolo `fn` | `rg -n "fn try_enforce_browser_tab_limit" src/browser_agent/mod.rs` (cwd `src-tauri/`) | **pass** — línea 3716 |
| Llamadas | `rg -n "try_enforce_browser_tab_limit\\(" src/browser_agent/mod.rs` | **pass** — 7 coincidencias (definición + usos) |
| Criterio 2 | `Config::browser_max_page_tabs` en `src/config/mod.rs` | **pass** — presente (~1974) |
| Criterio 3 | `examples/managed_tab_cap_smoke.rs` | **pass** — presente y documentado |
| Opcional CDP | `MAC_STATS_BROWSER_MAX_PAGE_TABS=3 cargo run --example managed_tab_cap_smoke` | **no completado** — tras ~120s sin salida adicional tras el arranque CDP (bootstrap `about:blank`); proceso detenido manualmente. No bloquea cierre: los criterios automatizados 1–4 cumplen. |

- **Outcome:** Criterios de aceptación 1–4 satisfechos → **`CLOSED-…`**.

---

### Test report — corrida TESTER (2026-03-27, hora local del entorno)

- **Preflight:** No existía `tasks/UNTESTED-20260322-1901-openclaw-managed-tab-cap-prune.md` en el árbol; el fichero estaba como `CLOSED-…`. Se renombró a `TESTING-…` y se ejecutó la verificación (misma tarea, sin otro `UNTESTED-*`).

| Paso | Comando | Resultado |
|------|---------|-----------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed |
| Símbolo `fn` | `rg -n "fn try_enforce_browser_tab_limit" src/browser_agent/mod.rs` (desde repo) | **pass** — línea 3715 |
| Llamadas | `rg -n "try_enforce_browser_tab_limit\\(" src/browser_agent/mod.rs` | **pass** — 7 coincidencias |
| Criterio 2 | `Config::browser_max_page_tabs` en `src/config/mod.rs` | **pass** — ~1987 |
| Criterio 3 | `examples/managed_tab_cap_smoke.rs` | **pass** — presente |
| Opcional CDP | `MAC_STATS_BROWSER_MAX_PAGE_TABS=3 cargo run --example managed_tab_cap_smoke` | **omitido** (no solicitado para cierre; criterios 1–4 automatizados cumplen) |

- **Outcome:** Criterios 1–4 OK → renombrar a **`CLOSED-…`**.

---

### Test report — corrida TESTER (2026-03-27, hora local del entorno)

- **Preflight:** La ruta pedida `tasks/UNTESTED-20260322-1901-openclaw-managed-tab-cap-prune.md` **no existe** en el árbol; la tarea está en `tasks/CLOSED-20260322-1901-openclaw-managed-tab-cap-prune.md`. No se aplicó el paso UNTESTED→TESTING (imposible sin fichero UNTESTED). No se tocó ningún otro `UNTESTED-*`. Verificación ejecutada sobre este mismo contenido de tarea.

| Paso | Comando | Resultado |
|------|---------|-----------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed |
| Símbolo `fn` | `rg -n "fn try_enforce_browser_tab_limit" src/browser_agent/mod.rs` (cwd `src-tauri`) | **pass** — línea 3715 |
| Llamadas | `rg -n "try_enforce_browser_tab_limit\\(" src/browser_agent/mod.rs` | **pass** — 7 coincidencias (definición + 6 usos) |
| Criterio 2 | `Config::browser_max_page_tabs` en `src/config/mod.rs` | **pass** — línea 1987 |
| Criterio 3 | `examples/managed_tab_cap_smoke.rs` | **pass** — presente |
| Opcional CDP | `MAC_STATS_BROWSER_MAX_PAGE_TABS=3 cargo run --example managed_tab_cap_smoke` | **omitido** (sin Chromium/CDP en esta corrida; no bloquea criterios 1–4) |

- **Outcome:** Criterios de aceptación 1–4 satisfechos → el nombre del fichero permanece **`CLOSED-…`** (ya estaba cerrado).
