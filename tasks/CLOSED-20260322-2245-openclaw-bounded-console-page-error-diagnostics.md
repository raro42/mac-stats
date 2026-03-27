# Bounded CDP console / page error diagnostics (OpenClaw-style)

## Goal

During **BROWSER_NAVIGATE** and **BROWSER_EXTRACT**, capture a **small, bounded** slice of CDP **Log** (error/warning) and **Runtime** uncaught-exception events so blank SPAs and JS failures are interpretable in tool output—without unbounded log spam.

## Acceptance criteria

1. **Caps:** At most **10** deduplicated console lines (~**200** chars each after whitespace collapse) and **2** deduplicated uncaught-exception messages (~**200** chars each); oldest dropped when over cap (FIFO via `VecDeque`).
2. **Filtering:** Only `LogEntryAdded` at **error** or **warning** level; ignore verbose/info. `RuntimeExceptionThrown` normalized from `text` or stack description.
3. **Lifecycle:** `try_attach_bounded_cdp_page_diagnostics` enables Log + Runtime, registers a listener; `detach_bounded_cdp_page_diagnostics` removes listener and disables both domains (no leak across tool calls).
4. **Gating:** When `Config::browser_include_diagnostics_in_state()` is false, navigate does not attach diagnostics; extract skips `TabExtractDiagnosticsSession`.
5. **Tool output:** Non-empty capture is appended as a `## Page diagnostics` markdown block (`Console (error/warning):` / `Uncaught exceptions:`).

## Verification

```bash
rg "try_attach_bounded_cdp_page_diagnostics|DIAG_MAX_CONSOLE_LINES|push_bounded_dedup|format_bounded_page_diagnostics_tool_section|browser_include_diagnostics_in_state" src-tauri/src/browser_agent/mod.rs src-tauri/src/config/mod.rs
cd src-tauri && cargo check
cd src-tauri && cargo test --no-run
```

Optional: `cargo test` (full run) if time permits.

## Test report

**Date:** 2026-03-27 (local time, workspace host).

**Preflight:** `tasks/UNTESTED-20260322-2245-openclaw-bounded-console-page-error-diagnostics.md` was **not** on disk at run start; the task body was written to that path per operator instruction, then renamed to `TESTING-20260322-2245-openclaw-bounded-console-page-error-diagnostics.md` per `003-tester/TESTER.md`. No other `UNTESTED-*` file was used in this run.

| Step | Command | Result |
|------|-----------|--------|
| Symbols / wiring | `rg "try_attach_bounded_cdp_page_diagnostics\|DIAG_MAX_CONSOLE_LINES\|push_bounded_dedup\|format_bounded_page_diagnostics_tool_section\|browser_include_diagnostics_in_state" src-tauri/src/browser_agent/mod.rs src-tauri/src/config/mod.rs` | **pass** — hits in `browser_agent/mod.rs` and `config/mod.rs` |
| Compile | `cd src-tauri && cargo check` | **pass** |
| Test binaries | `cd src-tauri && cargo test --no-run` | **pass** |
| Lib tests | `cd src-tauri && cargo test -p mac_stats --lib` | **pass** — 854 passed, 0 failed |

**Code review notes:** Constants `DIAG_MAX_CONSOLE_LINES` (10) / per-line chars (200) and uncaught caps (2 × 200) match acceptance §1. `push_cdp_diagnostic_event` filters Log to error/warning only and handles `RuntimeExceptionThrown` (§2). Attach/detach pair disables Log+Runtime and removes listener (§3). Navigate uses `browser_include_diagnostics_in_state()`; extract uses `TabExtractDiagnosticsSession` only when the same config is true (§4). `format_bounded_page_diagnostics_tool_section` emits `## Page diagnostics` (§5).

**Outcome:** **CLOSED** — acceptance criteria satisfied and build/tests green.

## Test report

**Date:** 2026-03-27 (local time, workspace host).

**Preflight:** `tasks/UNTESTED-20260322-2245-openclaw-bounded-console-page-error-diagnostics.md` was **not** present on disk; the task existed as `CLOSED-20260322-2245-openclaw-bounded-console-page-error-diagnostics.md`. Per operator instruction to test only that basename, the file was renamed `CLOSED-…` → `TESTING-…` for this run (same basename after the prefix). No other `UNTESTED-*` file was used.

| Step | Command | Result |
|------|-----------|--------|
| Symbols / wiring | `rg "try_attach_bounded_cdp_page_diagnostics\|DIAG_MAX_CONSOLE_LINES\|push_bounded_dedup\|format_bounded_page_diagnostics_tool_section\|browser_include_diagnostics_in_state" src-tauri/src/browser_agent/mod.rs src-tauri/src/config/mod.rs` | **pass** |
| Compile | `cd src-tauri && cargo check` | **pass** |
| Test binaries | `cd src-tauri && cargo test --no-run` | **pass** |
| Lib tests | `cd src-tauri && cargo test -p mac_stats --lib` | **pass** — 854 passed, 0 failed |

**Outcome:** **CLOSED** — verification commands and lib tests green; acceptance criteria unchanged from prior review.

## Test report

**Date:** 2026-03-27 (hora local del host del workspace).

**Preflight:** No existía `tasks/UNTESTED-20260322-2245-openclaw-bounded-console-page-error-diagnostics.md`; la tarea estaba como `CLOSED-20260322-2245-openclaw-bounded-console-page-error-diagnostics.md`. Por instrucción de probar solo esa base de nombre, se renombró `CLOSED-…` → `TESTING-…`. No se usó ningún otro `UNTESTED-*` en esta ejecución.

| Paso | Comando | Resultado |
|------|---------|-----------|
| Símbolos / cableado | `rg "try_attach_bounded_cdp_page_diagnostics|DIAG_MAX_CONSOLE_LINES|push_bounded_dedup|format_bounded_page_diagnostics_tool_section|browser_include_diagnostics_in_state" src-tauri/src/browser_agent/mod.rs src-tauri/src/config/mod.rs` | **pass** |
| Compilación | `cd src-tauri && cargo check` | **pass** |
| Binarios de test | `cd src-tauri && cargo test --no-run` | **pass** |
| Tests lib | `cd src-tauri && cargo test -p mac_stats --lib` | **pass** — 854 passed, 0 failed |

**Outcome:** **CLOSED** — criterios de aceptación sin cambios; verificación y tests en verde.

## Test report

**Date:** 2026-03-27 (local time, workspace host).

**Preflight:** `tasks/UNTESTED-20260322-2245-openclaw-bounded-console-page-error-diagnostics.md` was **not** on disk; the task file was `CLOSED-20260322-2245-openclaw-bounded-console-page-error-diagnostics.md`. Per `003-tester/TESTER.md` and operator instruction to test only that basename, it was renamed `CLOSED-…` → `TESTING-…` for this run. No other `UNTESTED-*` file was used.

| Step | Command | Result |
|------|-----------|--------|
| Symbols / wiring | `rg "try_attach_bounded_cdp_page_diagnostics|DIAG_MAX_CONSOLE_LINES|push_bounded_dedup|format_bounded_page_diagnostics_tool_section|browser_include_diagnostics_in_state" src-tauri/src/browser_agent/mod.rs src-tauri/src/config/mod.rs` | **pass** |
| Compile | `cd src-tauri && cargo check` | **pass** |
| Test binaries | `cd src-tauri && cargo test --no-run` | **pass** |
| Lib tests | `cd src-tauri && cargo test -p mac_stats --lib` | **pass** — 854 passed, 0 failed |

**Outcome:** **CLOSED** — verification commands and lib tests green; acceptance criteria unchanged.
