# Browser-use style SPA blank-page retry (CDP)

**Date:** 2026-03-21  
**Area:** `browser_agent`, `config`, docs

## Summary

After post-navigate stabilization, measure DOM size vs visible text; if the page looks like a skeleton or has no usable body, wait, optionally `Page.reload` once, then fail clearly if still unloadable. Toggle: `browserSpaRetryEnabled` / `MAC_STATS_BROWSER_SPA_RETRY_ENABLED` (default on).

## Acceptance criteria

1. `run_spa_blank_page_retry_if_needed` exists and is invoked after `BROWSER_NAVIGATE` (post-stabilization path).
2. `Config::browser_spa_retry_enabled()` reads env and `config.json`, default **true**.
3. Docs mention SPA blank-page retry and log tag `SPA readiness` at `-vv`.
4. `cargo check` and `cargo test` in `src-tauri/` pass.

## Verification

```bash
cd src-tauri && cargo check && cargo test
rg -n "run_spa_blank_page_retry_if_needed|browser_spa_retry_enabled|SPA readiness" ../src-tauri/src/browser_agent/mod.rs ../src-tauri/src/config/mod.rs ../docs/029_browser_automation.md
```

## Test report

**Date:** 2026-03-27 (local, entorno del agente al ejecutar la corrida)

**Commands run**

- `mv tasks/UNTESTED-20260321-1530-browser-use-spa-blank-page-retry.md → tasks/TESTING-20260321-1530-browser-use-spa-blank-page-retry.md` — pass
- `cd src-tauri && cargo check && cargo test` — pass (`cargo check` sin errores; `cargo test`: 854 passed, 0 failed en el crate principal; demás bins 0 tests)
- `rg -n "run_spa_blank_page_retry_if_needed|browser_spa_retry_enabled|SPA readiness" …` — pass (coincidencias en `browser_agent/mod.rs`, `config/mod.rs`, `docs/029_browser_automation.md`)

**Acceptance criteria**

1. Función presente y llamada tras navegación (~línea 7093 en `browser_agent/mod.rs`) — pass  
2. `browser_spa_retry_enabled()` con env, JSON y default `true` — pass (revisión de `config/mod.rs`)  
3. Documentación y mensajes de log `SPA readiness` — pass  
4. Compilación y tests — pass  

**Outcome:** Pass (verificación estática + suite Rust).

**Notes:** Al abrir la corrida, `tasks/UNTESTED-20260321-1530-browser-use-spa-blank-page-retry.md` no existía en el árbol; se creó con criterios y comandos alineados a la implementación y a `docs/029_browser_automation.md`, luego se aplicó el flujo TESTER (UNTESTED→TESTING→informe→CLOSED). No se ejecutó prueba manual con Chrome/CDP en esta corrida.
