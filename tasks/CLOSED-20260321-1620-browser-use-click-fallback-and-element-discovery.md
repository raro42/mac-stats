# CLOSED — browser-use click fallback and element discovery (2026-03-21)

## Goal

Align **BROWSER_CLICK** behaviour with browser-use-style resilience: **CDP pointer → JS `element.click()`** when appropriate; optional **HTTP `click_http`** when CDP fails and `should_use_http_fallback_after_browser_action_error` allows it; **stale-index / DOM reorder** recovery via identity-based **element discovery** (`find_unique_identity_match`).

## References

- `src-tauri/src/browser_agent/mod.rs` — `cdp_js_click_element`, occlusion handling, `find_unique_identity_match`, `click_by_index_inner_with_downloads`
- `src-tauri/src/commands/browser_tool_dispatch.rs` — `BROWSER_CLICK` index path + HTTP fallback branch
- `src-tauri/src/commands/browser_helpers.rs` — `should_use_http_fallback_after_browser_action_error`

## Acceptance criteria

1. **Build:** `cargo check` in `src-tauri/` succeeds.
2. **Tests:** `cargo test` in `src-tauri/` succeeds (no new failures attributable to click / fallback / remapping paths).
3. **Static verification:** Source still contains CDP click JS fallback, HTTP fallback gate for index clicks, and identity remapping helper (spot-check via `rg` or read).

## Verification commands

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test
```

Optional spot-check:

```bash
rg -n "should_use_http_fallback_after_browser_action_error|click_http" src-tauri/src/commands/browser_tool_dispatch.rs
rg -n "find_unique_identity_match|cdp_js_click_element" src-tauri/src/browser_agent/mod.rs
```

## Test report

**Date:** 2026-03-27 (local operator environment).

**Preflight:** The path `tasks/UNTESTED-20260321-1620-browser-use-click-fallback-and-element-discovery.md` was not present in the workspace at the start of this run; the task body was (re)materialized as `UNTESTED-…`, then renamed to `TESTING-…` per `003-tester/TESTER.md` before verification. No other `UNTESTED-*` file was used.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (854 tests in `mac_stats` library; 0 failed; 1 doc-test ignored)

**Static spot-check**

- `browser_tool_dispatch.rs`: `should_use_http_fallback_after_browser_action_error` + `click_http` on the `BROWSER_CLICK` index error path (e.g. lines ~834–839).
- `browser_agent/mod.rs`: `find_unique_identity_match` (~2321), `cdp_js_click_element` (~2884), unit coverage references ~9849+.

**Outcome:** All acceptance criteria satisfied for this verification pass. Live CDP click / HTTP fallback flows were not exercised end-to-end in this automated run (operator smoke optional).

## Test report — 2026-03-27 (local, Cursor tester run)

**Preflight:** `tasks/UNTESTED-20260321-1620-browser-use-click-fallback-and-element-discovery.md` was not in the workspace. The same task content lives at `tasks/CLOSED-20260321-1620-browser-use-click-fallback-and-element-discovery.md`, so the `UNTESTED-…` → `TESTING-…` rename from `003-tester/TESTER.md` could not be applied to that basename. No other `UNTESTED-*` file was used.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (854 tests in `mac_stats` library; 0 failed; 1 doc-test ignored)

**Static spot-check (`rg`)**

- `browser_tool_dispatch.rs`: `should_use_http_fallback_after_browser_action_error` (imports + ~834, ~1041); `click_http` on index error path (~839).
- `browser_agent/mod.rs`: `find_unique_identity_match` (~2320, ~3239, tests ~9877+); `cdp_js_click_element` (~2883, ~3011).

**Outcome:** All acceptance criteria pass. Filename left as `CLOSED-…` (no `WIP-…` rename). End-to-end CDP / HTTP click not exercised here.

## Test report — 2026-03-27 (local)

**Preflight:** `tasks/UNTESTED-20260321-1620-browser-use-click-fallback-and-element-discovery.md` was not present; the task exists only as `tasks/CLOSED-20260321-1620-browser-use-click-fallback-and-element-discovery.md`. Per `003-tester/TESTER.md`, the `UNTESTED-…` → `TESTING-…` rename could not be applied. No other `UNTESTED-*` file was used.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (854 passed in `mac_stats` lib; 0 failed; 1 doc-test ignored)

**Static spot-check (`rg`)**

- `browser_tool_dispatch.rs`: `should_use_http_fallback_after_browser_action_error` (import line 13, uses ~834, ~1041); `click_http` on index error path (~839).
- `browser_agent/mod.rs`: `find_unique_identity_match` (~2320, ~3239, tests ~9877+); `cdp_js_click_element` (~2883, ~3011).

**Outcome:** All acceptance criteria satisfied. Filename remains `CLOSED-…` (not renamed to `WIP-…`). End-to-end CDP / HTTP click not exercised in this run.

## Test report — 2026-03-27 (local, macOS)

**Preflight:** `tasks/UNTESTED-20260321-1620-browser-use-click-fallback-and-element-discovery.md` was not present; the task file was `tasks/CLOSED-20260321-1620-browser-use-click-fallback-and-element-discovery.md`. Per `003-tester/TESTER.md`, it was renamed `CLOSED-…` → `TESTING-…` for verification, then `TESTING-…` → `CLOSED-…` after pass. No other `UNTESTED-*` was used.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (854 passed in `mac_stats` library; 0 failed; 1 doc-test ignored)

**Static spot-check (`rg`)**

- `browser_tool_dispatch.rs`: `should_use_http_fallback_after_browser_action_error` (import L13, L834, L1041); `click_http` on index error path (L839).
- `browser_agent/mod.rs`: `find_unique_identity_match` (L2320, L3239, tests ~9877+); `cdp_js_click_element` (L2883, L3011).

**Outcome:** All acceptance criteria satisfied. End-to-end CDP / HTTP click not exercised in this automated run.

## Test report — 2026-03-27 (local, macOS workspace)

**Preflight:** `tasks/UNTESTED-20260321-1620-browser-use-click-fallback-and-element-discovery.md` was not present (task already on disk as `CLOSED-…`). Per `003-tester/TESTER.md`, the file was renamed `CLOSED-…` → `TESTING-…` for this verification pass, then renamed back to `CLOSED-…` after success. No other `UNTESTED-*` task file was used in this run.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (854 passed in `mac_stats` library; 0 failed; 1 doc-test ignored)

**Static spot-check (`rg`)**

- `browser_tool_dispatch.rs`: `should_use_http_fallback_after_browser_action_error` (L13, L834, L1041); `click_http` on index error path (L839).
- `browser_agent/mod.rs`: `find_unique_identity_match` (L2320, L3239, tests ~9877+); `cdp_js_click_element` (L2883, L3011).

**Outcome:** All acceptance criteria pass. Filename set to `CLOSED-…` after this report. End-to-end CDP / HTTP click not exercised in this run.

## Test report — 2026-03-27 (local, macOS)

**Preflight:** `tasks/UNTESTED-20260321-1620-browser-use-click-fallback-and-element-discovery.md` was not present. The task on disk was `tasks/CLOSED-20260321-1620-browser-use-click-fallback-and-element-discovery.md`; it was renamed to `TESTING-…` for this run per `003-tester/TESTER.md` (same basename after the prefix). No other `UNTESTED-*` file was used.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (854 passed in `mac_stats` library; 0 failed; 1 doc-test ignored)

**Static spot-check (`rg`)**

- `browser_tool_dispatch.rs`: `should_use_http_fallback_after_browser_action_error` (L13, L834, L1041); `click_http` on index error path (L839).
- `browser_agent/mod.rs`: `find_unique_identity_match` (L2320, L3239, tests L9877+); `cdp_js_click_element` (L2883, L3011).

**Outcome:** All acceptance criteria satisfied. Renaming `TESTING-…` → `CLOSED-…` after this report. End-to-end CDP / HTTP click not exercised in this run.

## Test report — 2026-03-27 (local, macOS)

**Preflight:** `tasks/UNTESTED-20260321-1620-browser-use-click-fallback-and-element-discovery.md` was not present. Per `003-tester/TESTER.md`, `tasks/CLOSED-20260321-1620-browser-use-click-fallback-and-element-discovery.md` was renamed to `TESTING-20260321-1620-browser-use-click-fallback-and-element-discovery.md` for this run (same basename after the prefix). No other `UNTESTED-*` file was used.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (854 passed in `mac_stats` library; 0 failed; 1 doc-test ignored)

**Static spot-check (`rg`)**

- `browser_tool_dispatch.rs`: `should_use_http_fallback_after_browser_action_error` (L13, L834, L1041); `click_http` on index error path (L839).
- `browser_agent/mod.rs`: `find_unique_identity_match` (L2320, L3239, tests L9877+); `cdp_js_click_element` (L2883, L3011).

**Outcome:** All acceptance criteria pass. Renaming `TESTING-…` → `CLOSED-…` after this report. End-to-end CDP / HTTP click not exercised in this run.

## Test report — 2026-03-28 (local, macOS)

**Preflight:** `tasks/UNTESTED-20260321-1620-browser-use-click-fallback-and-element-discovery.md` was not present in the workspace. Per `003-tester/TESTER.md`, `tasks/CLOSED-20260321-1620-browser-use-click-fallback-and-element-discovery.md` was renamed to `TESTING-20260321-1620-browser-use-click-fallback-and-element-discovery.md` for this run (same basename after the prefix). No other `UNTESTED-*` file was used.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (854 passed in `mac_stats` library; 0 failed; 1 doc-test ignored)

**Static spot-check (`rg`)**

- `browser_tool_dispatch.rs`: `should_use_http_fallback_after_browser_action_error` (L13, L834, L1041); `click_http` on index error path (L839).
- `browser_agent/mod.rs`: `find_unique_identity_match` (L2320, L3239, tests L9877+); `cdp_js_click_element` (L2883, L3011).

**Outcome:** All acceptance criteria satisfied. Renaming `TESTING-…` → `CLOSED-…` after this report. End-to-end CDP / HTTP click not exercised in this run.

## Test report — 2026-03-28 (local) — 003-tester follow-up

**Preflight:** Same as above: no `UNTESTED-…` path; `CLOSED-…` → `TESTING-…` for this verification. No other `UNTESTED-*` file was used.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (`mac_stats` library: 854 passed, 0 failed, 0 ignored; doc-tests: 1 ignored)
- `rg -n "should_use_http_fallback_after_browser_action_error|click_http" src-tauri/src/commands/browser_tool_dispatch.rs` — **pass**
- `rg -n "find_unique_identity_match|cdp_js_click_element" src-tauri/src/browser_agent/mod.rs` — **pass**

**Outcome:** All acceptance criteria pass. Renaming `TESTING-…` → `CLOSED-…` after this report. End-to-end CDP / HTTP click not exercised in this run.

## Test report — 2026-03-28 (local, macOS) — operator-requested single-task retest

**Preflight:** `tasks/UNTESTED-20260321-1620-browser-use-click-fallback-and-element-discovery.md` was not in the workspace (only this basename exists under `CLOSED-` / `TESTING-`). Per `003-tester/TESTER.md`, the file was taken as `CLOSED-…` → `TESTING-…` for verification (equivalent workflow when `UNTESTED-…` is absent). No other `UNTESTED-*` task file was used in this run.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (`mac_stats` library: 854 passed, 0 failed, 0 ignored; doc-tests: 1 ignored)
- `rg -n "should_use_http_fallback_after_browser_action_error|click_http" src-tauri/src/commands/browser_tool_dispatch.rs` — **pass**
- `rg -n "find_unique_identity_match|cdp_js_click_element" src-tauri/src/browser_agent/mod.rs` — **pass**

**Outcome:** All acceptance criteria satisfied. Renaming `TESTING-…` → `CLOSED-…` after this report. End-to-end CDP / HTTP click not exercised in this run.

## Test report — 2026-03-28 (local, macOS) — `003-tester/TESTER.md` (UNTESTED path named; file absent)

**Preflight:** El operador pidió probar `tasks/UNTESTED-20260321-1620-browser-use-click-fallback-and-element-discovery.md`; ese path no existía (el task ya estaba como `CLOSED-…`). Según `003-tester/TESTER.md`, se renombró `CLOSED-…` → `TESTING-…` para esta pasada (mismo basename tras el prefijo). No se tocó ningún otro `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (biblioteca `mac_stats`: 854 passed, 0 failed, 0 ignored; doc-tests: 1 ignored)
- `rg -n "should_use_http_fallback_after_browser_action_error|click_http" src-tauri/src/commands/browser_tool_dispatch.rs` — **pass**
- `rg -n "find_unique_identity_match|cdp_js_click_element" src-tauri/src/browser_agent/mod.rs` — **pass**

**Outcome:** Todos los criterios de aceptación cumplidos. Renombrar `TESTING-…` → `CLOSED-…` tras este informe. Flujos CDP / HTTP click end-to-end no ejecutados en esta pasada.

## Test report — 2026-03-28 (local, macOS) — `003-tester/TESTER.md` (solo este task)

**Preflight:** El path `tasks/UNTESTED-20260321-1620-browser-use-click-fallback-and-element-discovery.md` no existía; el task estaba como `CLOSED-…`. Se aplicó el flujo equivalente: `CLOSED-…` → `TESTING-…` antes de la verificación. No se usó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (biblioteca `mac_stats`: 854 passed, 0 failed, 0 ignored; doc-tests: 1 ignored)
- `rg -n "should_use_http_fallback_after_browser_action_error|click_http" src-tauri/src/commands/browser_tool_dispatch.rs` — **pass** (coincidencias en L13, L834, L839, L1041)
- `rg -n "find_unique_identity_match|cdp_js_click_element" src-tauri/src/browser_agent/mod.rs` — **pass** (`find_unique_identity_match` L2320, L3239, tests L9877+; `cdp_js_click_element` L2883, L3011)

**Outcome:** Todos los criterios de aceptación cumplidos. Renombrar `TESTING-…` → `CLOSED-…` tras este informe. Flujos CDP / HTTP click end-to-end no ejecutados en esta pasada.

## Test report — 2026-03-28 (local, macOS) — `003-tester/TESTER.md` (UNTESTED path requested; absent)

**Preflight:** `tasks/UNTESTED-20260321-1620-browser-use-click-fallback-and-element-discovery.md` was not present. Per `003-tester/TESTER.md`, the on-disk task was renamed `CLOSED-…` → `TESTING-…` before verification (same basename after the prefix). No other `UNTESTED-*` task file was used in this run.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (`mac_stats` library: 854 passed, 0 failed, 0 ignored; doc-tests: 1 ignored)
- `rg -n "should_use_http_fallback_after_browser_action_error|click_http" src-tauri/src/commands/browser_tool_dispatch.rs` — **pass** (L13, L834, L839, L1041)
- `rg -n "find_unique_identity_match|cdp_js_click_element" src-tauri/src/browser_agent/mod.rs` — **pass** (`find_unique_identity_match` L2320, L3239, tests L9877+; `cdp_js_click_element` L2883, L3011)

**Outcome:** All acceptance criteria satisfied. Renaming `TESTING-…` → `CLOSED-…` after this report. End-to-end CDP / HTTP click not exercised in this run.

## Test report — 2026-03-28 (local, macOS) — `003-tester/TESTER.md` (solo UNTESTED nombrado; ausente)

**Preflight:** El path `tasks/UNTESTED-20260321-1620-browser-use-click-fallback-and-element-discovery.md` no existía en el workspace; el task estaba como `CLOSED-…`. Según `003-tester/TESTER.md` se renombró `CLOSED-…` → `TESTING-…` antes de la verificación (mismo basename tras el prefijo). No se eligió ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (biblioteca `mac_stats`: 854 passed, 0 failed, 0 ignored; doc-tests: 1 ignored)
- `rg -n "should_use_http_fallback_after_browser_action_error|click_http" src-tauri/src/commands/browser_tool_dispatch.rs` — **pass** (L13, L834, L839, L1041)
- `rg -n "find_unique_identity_match|cdp_js_click_element" src-tauri/src/browser_agent/mod.rs` — **pass** (`find_unique_identity_match` L2320, L3239, tests L9877+; `cdp_js_click_element` L2883, L3011)

**Outcome:** Todos los criterios de aceptación cumplidos. Renombrar `TESTING-…` → `CLOSED-…` tras este informe. Flujos CDP / HTTP click end-to-end no ejecutados en esta pasada.

## Test report — 2026-03-28 (local, macOS) — `003-tester/TESTER.md` (operator: test only this UNTESTED path; file absent)

**Preflight:** `tasks/UNTESTED-20260321-1620-browser-use-click-fallback-and-element-discovery.md` was not in the workspace. Per `003-tester/TESTER.md`, the task was renamed `CLOSED-…` → `TESTING-…` before verification (same basename after the prefix). No other `UNTESTED-*` task file was used in this run.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (`mac_stats` library: 854 passed, 0 failed, 0 ignored; doc-tests: 1 ignored)
- `rg -n "should_use_http_fallback_after_browser_action_error|click_http" src-tauri/src/commands/browser_tool_dispatch.rs` — **pass** (L13, L834, L839, L1041)
- `rg -n "find_unique_identity_match|cdp_js_click_element" src-tauri/src/browser_agent/mod.rs` — **pass** (`find_unique_identity_match` L2320, L3239, tests L9877+; `cdp_js_click_element` L2883, L3011)

**Outcome:** All acceptance criteria satisfied. Renaming `TESTING-…` → `CLOSED-…` after this report. End-to-end CDP / HTTP click not exercised in this run.

## Test report — 2026-03-28 (local, macOS) — `003-tester/TESTER.md` (UNTESTED path pedido; inexistente; verificación fresca)

**Preflight:** No existía `tasks/UNTESTED-20260321-1620-browser-use-click-fallback-and-element-discovery.md`. El task ya estaba como `CLOSED-…`; se aplicó `CLOSED-…` → `TESTING-…` para esta pasada según `003-tester/TESTER.md`. No se tocó ningún otro `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (biblioteca `mac_stats`: 854 passed, 0 failed, 0 ignored; doc-tests: 1 ignored)
- `rg -n "should_use_http_fallback_after_browser_action_error|click_http" src-tauri/src/commands/browser_tool_dispatch.rs` — **pass** (L13, L834, L839, L1041)
- `rg -n "find_unique_identity_match|cdp_js_click_element" src-tauri/src/browser_agent/mod.rs` — **pass** (`find_unique_identity_match` L2320, L3239, tests L9877+; `cdp_js_click_element` L2883, L3011)

**Outcome:** Todos los criterios de aceptación cumplidos. Tras este informe: `TESTING-…` → `CLOSED-…`. CDP / HTTP click end-to-end no probados aquí.

## Test report — 2026-03-28 (local, macOS)

**Preflight:** El path `tasks/UNTESTED-20260321-1620-browser-use-click-fallback-and-element-discovery.md` no existía en el workspace. Según `003-tester/TESTER.md`, el archivo en disco se renombró `CLOSED-…` → `TESTING-…` antes de la verificación (mismo basename tras el prefijo). No se usó ningún otro archivo `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (biblioteca `mac_stats`: 854 passed, 0 failed, 0 ignored; doc-tests: 1 ignored)
- Spot-check `rg`: `should_use_http_fallback_after_browser_action_error` / `click_http` en `browser_tool_dispatch.rs` (L13, L834, L839, L1041) — **pass**
- Spot-check `rg`: `find_unique_identity_match` / `cdp_js_click_element` en `browser_agent/mod.rs` (p. ej. L2320, L2883, L3011, L3239, tests ~L9877+) — **pass**

**Outcome:** Cumplen todos los criterios de aceptación. Tras este informe: `TESTING-…` → `CLOSED-…`. Flujo CDP / HTTP click end-to-end no ejecutado en esta pasada automatizada.

## Test report — 2026-03-28 (local, macOS) — `003-tester/TESTER.md` (operator: UNTESTED path only; file absent)

**Preflight:** `tasks/UNTESTED-20260321-1620-browser-use-click-fallback-and-element-discovery.md` was not in the workspace. Per `003-tester/TESTER.md`, `tasks/CLOSED-20260321-1620-browser-use-click-fallback-and-element-discovery.md` was renamed to `TESTING-20260321-1620-browser-use-click-fallback-and-element-discovery.md` before verification (same basename after the prefix). No other `UNTESTED-*` task file was used in this run.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (`mac_stats` library: 854 passed, 0 failed, 0 ignored; doc-tests: 1 ignored)
- `rg -n "should_use_http_fallback_after_browser_action_error|click_http" src-tauri/src/commands/browser_tool_dispatch.rs` — **pass** (L13, L834, L839, L1041)
- `rg -n "find_unique_identity_match|cdp_js_click_element" src-tauri/src/browser_agent/mod.rs` — **pass** (`find_unique_identity_match` L2320, L3239, tests L9877+; `cdp_js_click_element` L2883, L3011)

**Outcome:** All acceptance criteria satisfied. Renaming `TESTING-…` → `CLOSED-…` after this report. End-to-end CDP / HTTP click not exercised in this run.

## Test report — 2026-03-28 (local, macOS) — `003-tester/TESTER.md` (cursor: solo el UNTESTED nombrado; inexistente)

**Preflight:** `tasks/UNTESTED-20260321-1620-browser-use-click-fallback-and-element-discovery.md` no existía; el archivo en disco era `CLOSED-…` y se renombró a `TESTING-…` antes de la verificación. No se eligió ningún otro `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (biblioteca `mac_stats`: 854 passed, 0 failed, 0 ignored; doc-tests: 1 ignored)
- `rg -n "should_use_http_fallback_after_browser_action_error|click_http" src-tauri/src/commands/browser_tool_dispatch.rs` — **pass** (L13, L834, L839, L1041)
- `rg -n "find_unique_identity_match|cdp_js_click_element" src-tauri/src/browser_agent/mod.rs` — **pass** (L2320, L2883, L3011, L3239, tests L9877+)

**Outcome:** Todos los criterios de aceptación cumplidos. Tras este informe: `TESTING-…` → `CLOSED-…`. CDP / HTTP click end-to-end no probados en esta pasada.
