# CLOSED — OpenClaw-style poisoned-cache prevention (Ollama model list) (2026-03-21)

## Goal

Ensure cached `GET /api/tags` does not **poison** state: **failed** responses and **empty** model lists must not overwrite a prior **non-empty** successful list; background refresh must follow the same rule; operators can grep `[ollama/model_cache]` in logs.

## References

- `src-tauri/src/ollama/model_list_cache.rs` — TTL, stale-while-revalidate, in-flight dedup, poisoned-cache branches
- `docs/015_ollama_api.md` — caching / no poisoned cache documentation
- `src-tauri/src/commands/ollama_models.rs`, `ollama_config.rs` — `fetch_tags_cached` / `clear_all` / `clear_endpoint`

## Acceptance criteria

1. **Build:** `cargo check` in `src-tauri/` succeeds.
2. **Tests:** `cargo test` in `src-tauri/` succeeds.
3. **Static verification:** `model_list_cache.rs` contains explicit “do not replace / not updating cache” handling for empty `Ok` and `Err` paths, and `MCACHE_LOG_TAG` for log grep.

## Verification commands

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test
```

```bash
rg -n "not replacing cached|not updating cache|empty model list|MCACHE_LOG_TAG" src-tauri/src/ollama/model_list_cache.rs
```

## Test report

**Date:** 2026-03-27 (local, America-friendly operator environment; wall clock not guaranteed UTC).

**Preflight:** `tasks/UNTESTED-20260321-1855-openclaw-poisoned-cache-prevention.md` was not on disk when this run started; the task body was written to that path, then renamed to `TESTING-20260321-1855-openclaw-poisoned-cache-prevention.md` per `003-tester/TESTER.md`. No other `UNTESTED-*` task was used.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (854 tests in `mac_stats` library; 0 failed; 1 doc-test ignored)
- `rg -n "not replacing cached|not updating cache|empty model list|MCACHE_LOG_TAG" src-tauri/src/ollama/model_list_cache.rs` — **pass** (matches `MCACHE_LOG_TAG`, empty-list and fetch-error warn paths with “not replacing cached data” / “not updating cache”)

**Notes:** No dedicated unit tests target `model_list_cache.rs`; verification is build + suite + static read of branches in `fetch_tags_cached` / `run_bg_refresh`. Live Ollama empty/error responses against a running daemon were not exercised in this run.

**Outcome:** All acceptance criteria satisfied → closed.

## Test report

**Date:** 2026-03-27 (local workspace time; not UTC).

**Preflight:** `tasks/UNTESTED-20260321-1855-openclaw-poisoned-cache-prevention.md` was not on disk; this cycle started from `tasks/CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md`, renamed to `TESTING-20260321-1855-openclaw-poisoned-cache-prevention.md` per `003-tester/TESTER.md` step 2 (UNTESTED→TESTING). No other `UNTESTED-*` task was used.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed in `mac_stats` library; 1 doc-test ignored)
- `rg -n "not replacing cached|not updating cache|empty model list|MCACHE_LOG_TAG" src-tauri/src/ollama/model_list_cache.rs` — **pass**

**Notes:** Same scope as prior report: no live Ollama daemon exercised for empty/error responses.

**Outcome:** All acceptance criteria satisfied → `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md`.

## Test report

**Date:** 2026-03-27 (local workspace time; not UTC).

**Preflight:** `tasks/UNTESTED-20260321-1855-openclaw-poisoned-cache-prevention.md` was not present; this run used the same task content under `tasks/CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md`, renamed to `TESTING-20260321-1855-openclaw-poisoned-cache-prevention.md` per `003-tester/TESTER.md` step 2. No other `UNTESTED-*` task was used.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed in `mac_stats` library; 1 doc-test ignored)
- `rg -n "not replacing cached|not updating cache|empty model list|MCACHE_LOG_TAG" src-tauri/src/ollama/model_list_cache.rs` — **pass**

**Notes:** No live Ollama daemon exercised for empty/error responses; scope matches prior reports.

**Outcome:** All acceptance criteria satisfied → `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md`.

## Test report

**Date:** 2026-03-27 (local workspace time; not UTC).

**Preflight:** `tasks/UNTESTED-20260321-1855-openclaw-poisoned-cache-prevention.md` was not present; `003-tester/TESTER.md` step 2 was applied by renaming `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md` → `TESTING-20260321-1855-openclaw-poisoned-cache-prevention.md`. No other `UNTESTED-*` task was used.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed in `mac_stats` library; 1 doc-test ignored)
- `rg -n "not replacing cached|not updating cache|empty model list|MCACHE_LOG_TAG" src-tauri/src/ollama/model_list_cache.rs` — **pass**

**Notes:** No live Ollama daemon exercised for empty/error responses; verification matches task acceptance criteria.

**Outcome:** All acceptance criteria satisfied → `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md`.

## Test report

**Date:** 2026-03-27 (local workspace time; not UTC).

**Preflight:** `tasks/UNTESTED-20260321-1855-openclaw-poisoned-cache-prevention.md` was not present; `003-tester/TESTER.md` step 2 was applied by renaming `tasks/CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md` → `tasks/TESTING-20260321-1855-openclaw-poisoned-cache-prevention.md`. No other `UNTESTED-*` task was used.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed in `mac_stats` library tests; 1 doc-test ignored)
- `rg -n "not replacing cached|not updating cache|empty model list|MCACHE_LOG_TAG" src-tauri/src/ollama/model_list_cache.rs` — **pass** (matches `MCACHE_LOG_TAG`, empty-list “not replacing cached data”, fetch-error “not updating cache”)

**Notes:** No live Ollama daemon exercised for empty/error HTTP responses; scope matches task acceptance criteria.

**Outcome:** All acceptance criteria satisfied → `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md`.

## Test report

**Date:** 2026-03-27 (hora local del workspace; no UTC).

**Preflight:** `tasks/UNTESTED-20260321-1855-openclaw-poisoned-cache-prevention.md` no existía; el archivo activo era `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md`, renombrado a `TESTING-20260321-1855-openclaw-poisoned-cache-prevention.md` según `003-tester/TESTER.md` (paso UNTESTED→TESTING). No se usó ningún otro `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en tests de la librería `mac_stats`; 1 doc-test ignored)
- `rg -n "not replacing cached|not updating cache|empty model list|MCACHE_LOG_TAG" src-tauri/src/ollama/model_list_cache.rs` — **pass**

**Notes:** Sin prueba en vivo contra Ollama con listas vacías o errores HTTP; coincide con el alcance de los criterios de aceptación.

**Outcome:** Todos los criterios de aceptación cumplidos → `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md`.

## Test report

**Date:** 2026-03-27 (local workspace time; not UTC).

**Preflight:** `tasks/UNTESTED-20260321-1855-openclaw-poisoned-cache-prevention.md` was absent; `tasks/CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md` was renamed to `TESTING-20260321-1855-openclaw-poisoned-cache-prevention.md` per `003-tester/TESTER.md` step 2 (equivalent UNTESTED→TESTING for this task id). No other `UNTESTED-*` task was used.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed in `mac_stats` library tests; 1 doc-test ignored)
- `rg -n "not replacing cached|not updating cache|empty model list|MCACHE_LOG_TAG" src-tauri/src/ollama/model_list_cache.rs` — **pass** (matches `MCACHE_LOG_TAG`, empty-list and error paths with “not replacing cached data” / “not updating cache”)

**Notes:** No live Ollama daemon exercised for empty/error HTTP responses; scope matches task acceptance criteria.

**Outcome:** All acceptance criteria satisfied → `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md`.

## Test report

**Date:** 2026-03-28 (hora local del workspace; no UTC).

**Preflight:** `tasks/UNTESTED-20260321-1855-openclaw-poisoned-cache-prevention.md` no existía; `tasks/CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md` se renombró a `TESTING-20260321-1855-openclaw-poisoned-cache-prevention.md` según `003-tester/TESTER.md` (paso equivalente UNTESTED→TESTING para este id). No se usó ningún otro `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en tests de la librería `mac_stats`; 1 doc-test ignored)
- `rg -n "not replacing cached|not updating cache|empty model list|MCACHE_LOG_TAG" src-tauri/src/ollama/model_list_cache.rs` — **pass** (coincide `MCACHE_LOG_TAG`, lista vacía «not replacing cached data», error de fetch «not updating cache»)

**Notes:** Sin prueba en vivo contra Ollama con respuestas vacías o error HTTP; el alcance coincide con los criterios de aceptación de la tarea.

**Outcome:** Todos los criterios de aceptación cumplidos → `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md`.

## Test report

**Date:** 2026-03-28 (hora local del workspace; no UTC).

**Preflight:** `tasks/UNTESTED-20260321-1855-openclaw-poisoned-cache-prevention.md` no existía en disco; `tasks/CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md` se renombró a `TESTING-20260321-1855-openclaw-poisoned-cache-prevention.md` según `003-tester/TESTER.md` (paso 2, equivalente UNTESTED→TESTING para este id). No se usó ningún otro `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en tests de la librería `mac_stats`; 1 doc-test ignored en `Doc-tests mac_stats`)
- `rg -n "not replacing cached|not updating cache|empty model list|MCACHE_LOG_TAG" src-tauri/src/ollama/model_list_cache.rs` — **pass** (coincide `MCACHE_LOG_TAG`, «not replacing cached data», «not updating cache», lista vacía / error de fetch)

**Notes:** Sin prueba en vivo contra Ollama con respuestas vacías o error HTTP; el alcance coincide con los criterios de aceptación.

**Outcome:** Todos los criterios de aceptación cumplidos → `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md`.

## Test report

**Date:** 2026-03-28 (hora local del workspace; no UTC).

**Preflight:** Misma tarea que `UNTESTED-20260321-1855-openclaw-poisoned-cache-prevention.md` (archivo inexistente); ciclo `003-tester/TESTER.md` aplicado renombrando `CLOSED-…` → `TESTING-…` → (tras informe) `CLOSED-…`. Verificación solicitada explícitamente en esta sesión; ningún otro `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en la librería `mac_stats`; 1 doc-test ignored)
- `rg -n "not replacing cached|not updating cache|empty model list|MCACHE_LOG_TAG" src-tauri/src/ollama/model_list_cache.rs` — **pass**

**Notes:** Sin daemon Ollama en vivo para vacío/error HTTP; criterios de la tarea cubiertos por build, suite y grep estático.

**Outcome:** Todos los criterios de aceptación cumplidos → `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md`.

## Test report

**Date:** 2026-03-28 (hora local del workspace; no UTC).

**Preflight:** El operador nombró `tasks/UNTESTED-20260321-1855-openclaw-poisoned-cache-prevention.md`, que **no existía**; se aplicó `003-tester/TESTER.md` renombrando `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md` → `TESTING-20260321-1855-openclaw-poisoned-cache-prevention.md` (equivalente al paso UNTESTED→TESTING). No se usó ningún otro `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed en la librería `mac_stats`; 1 doc-test ignored en `Doc-tests mac_stats`)
- `rg -n "not replacing cached|not updating cache|empty model list|MCACHE_LOG_TAG" src-tauri/src/ollama/model_list_cache.rs` — **pass** (`MCACHE_LOG_TAG`, lista vacía «not replacing cached data», error «not updating cache»)

**Notes:** Sin prueba en vivo contra Ollama con respuestas vacías o error HTTP; coincide con los criterios de aceptación de la tarea.

**Outcome:** Todos los criterios de aceptación cumplidos → `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md`.

## Test report

**Date:** 2026-03-28 (hora local del workspace; no UTC).

**Preflight:** `tasks/UNTESTED-20260321-1855-openclaw-poisoned-cache-prevention.md` no estaba en disco; se siguió `003-tester/TESTER.md` renombrando `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md` → `TESTING-20260321-1855-openclaw-poisoned-cache-prevention.md` (equivalente UNTESTED→TESTING). No se usó ningún otro `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (librería `mac_stats`: 854 passed, 0 failed, 0 ignored, 0 filtered out, ~1.16s; `Doc-tests mac_stats`: 1 ignored)
- `rg -n "not replacing cached|not updating cache|empty model list|MCACHE_LOG_TAG" src-tauri/src/ollama/model_list_cache.rs` — **pass**

**Notes:** Sin Ollama en vivo para vacío/error HTTP; verificación = criterios 1–3 de la tarea.

**Outcome:** Todos los criterios de aceptación cumplidos → `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md`.

## Test report

**Date:** 2026-03-28 (hora local del workspace; no UTC).

**Preflight:** `tasks/UNTESTED-20260321-1855-openclaw-poisoned-cache-prevention.md` no existía; `003-tester/TESTER.md` paso 2: `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md` → `TESTING-20260321-1855-openclaw-poisoned-cache-prevention.md` (equivalente UNTESTED→TESTING). No se usó ningún otro `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (librería `mac_stats`: 854 passed, 0 failed, 0 ignored, 0 filtered out, ~1.16s; `Doc-tests mac_stats`: 1 ignored)
- `rg -n "not replacing cached|not updating cache|empty model list|MCACHE_LOG_TAG" src-tauri/src/ollama/model_list_cache.rs` — **pass** (`MCACHE_LOG_TAG`, «not replacing cached data», «not updating cache»)

**Notes:** Sin Ollama en vivo para vacío/error HTTP; alcance = criterios 1–3 de la tarea (re-ejecución solicitada en esta sesión).

**Outcome:** Todos los criterios de aceptación cumplidos → `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md`.

## Test report

**Date:** 2026-03-28 (hora local del workspace; no UTC).

**Preflight:** `tasks/UNTESTED-20260321-1855-openclaw-poisoned-cache-prevention.md` no existía; `003-tester/TESTER.md` paso 2: `CLOSED-…` → `TESTING-20260321-1855-openclaw-poisoned-cache-prevention.md` (equivalente UNTESTED→TESTING). Ciclo pedido explícitamente en esta sesión; ningún otro `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass** (`Finished dev profile` en ~0.20s)
- `cd src-tauri && cargo test` — **pass** (librería `mac_stats`: 854 passed, 0 failed, 0 ignored, ~1.16s; `Doc-tests mac_stats`: 1 ignored)
- `rg -n "not replacing cached|not updating cache|empty model list|MCACHE_LOG_TAG" src-tauri/src/ollama/model_list_cache.rs` — **pass** (`MCACHE_LOG_TAG`, «not replacing cached data», «not updating cache»)

**Notes:** Sin daemon Ollama en vivo para vacío/error HTTP; criterios 1–3 de la tarea cubiertos.

**Outcome:** Todos los criterios de aceptación cumplidos → `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md`.

## Test report

**Date:** 2026-03-28 (hora local del workspace; no UTC).

**Preflight:** `tasks/UNTESTED-20260321-1855-openclaw-poisoned-cache-prevention.md` no existía; se aplicó `003-tester/TESTER.md` renombrando `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md` → `TESTING-20260321-1855-openclaw-poisoned-cache-prevention.md` (equivalente UNTESTED→TESTING). Tarea explícita del operador; ningún otro `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (librería `mac_stats`: 854 passed, 0 failed; `Doc-tests mac_stats`: 1 ignored)
- `rg -n "not replacing cached|not updating cache|empty model list|MCACHE_LOG_TAG" src-tauri/src/ollama/model_list_cache.rs` — **pass**

**Notes:** Sin prueba en vivo contra Ollama con respuestas vacías o error HTTP; criterios 1–3 de la tarea cumplidos.

**Outcome:** Todos los criterios de aceptación cumplidos → `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md`.

## Test report

**Date:** 2026-03-28 (hora local del workspace; no UTC).

**Preflight:** `tasks/UNTESTED-20260321-1855-openclaw-poisoned-cache-prevention.md` no existía; `003-tester/TESTER.md` paso 2: `CLOSED-…` → `TESTING-20260321-1855-openclaw-poisoned-cache-prevention.md` (equivalente UNTESTED→TESTING). Tarea nombrada por el operador en esta sesión; ningún otro `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (librería `mac_stats`: 854 passed, 0 failed, 0 ignored, ~1.16s; `Doc-tests mac_stats`: 1 ignored)
- `rg -n "not replacing cached|not updating cache|empty model list|MCACHE_LOG_TAG" src-tauri/src/ollama/model_list_cache.rs` — **pass** (`MCACHE_LOG_TAG`, «not replacing cached data», «not updating cache»)

**Notes:** Sin daemon Ollama en vivo para vacío/error HTTP; criterios 1–3 de la tarea cubiertos.

**Outcome:** Todos los criterios de aceptación cumplidos → `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md`.

## Test report

**Date:** 2026-03-28 (hora local del workspace; no UTC).

**Preflight:** `tasks/UNTESTED-20260321-1855-openclaw-poisoned-cache-prevention.md` no existía; `003-tester/TESTER.md` paso 2 aplicado como `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md` → `TESTING-20260321-1855-openclaw-poisoned-cache-prevention.md` (equivalente UNTESTED→TESTING). Solicitud explícita del operador de probar solo ese id; ningún otro `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (librería `mac_stats`: 854 passed, 0 failed, 0 ignored, ~1.16s; `Doc-tests mac_stats`: 1 ignored)
- `rg -n "not replacing cached|not updating cache|empty model list|MCACHE_LOG_TAG" src-tauri/src/ollama/model_list_cache.rs` — **pass** (`MCACHE_LOG_TAG`, «not replacing cached data», «not updating cache»)

**Notes:** Sin Ollama en vivo para vacío/error HTTP; alcance acorde a criterios 1–3 de la tarea.

**Outcome:** Todos los criterios de aceptación cumplidos → `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md`.

## Test report

**Date:** 2026-03-28 (local workspace time; not UTC).

**Preflight:** `tasks/UNTESTED-20260321-1855-openclaw-poisoned-cache-prevention.md` was not on disk; per `003-tester/TESTER.md` step 2, `tasks/CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md` was renamed to `tasks/TESTING-20260321-1855-openclaw-poisoned-cache-prevention.md` (equivalent UNTESTED→TESTING for this task id). Operator asked to test only this task; no other `UNTESTED-*` file was used.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (`mac_stats` library: 854 passed, 0 failed, 0 ignored; `Doc-tests mac_stats`: 1 ignored)
- `rg -n "not replacing cached|not updating cache|empty model list|MCACHE_LOG_TAG" src-tauri/src/ollama/model_list_cache.rs` — **pass** (matches `MCACHE_LOG_TAG`, empty-list “not replacing cached data”, error path “not updating cache”)

**Notes:** Live Ollama daemon not exercised for empty/error HTTP responses; scope matches task criteria 1–3.

**Outcome:** All acceptance criteria satisfied → `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md`.

## Test report

**Date:** 2026-03-28 (local workspace time; not UTC).

**Preflight:** `tasks/UNTESTED-20260321-1855-openclaw-poisoned-cache-prevention.md` was absent; per `003-tester/TESTER.md` step 2, `tasks/CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md` was renamed to `tasks/TESTING-20260321-1855-openclaw-poisoned-cache-prevention.md` (equivalent UNTESTED→TESTING for this task id). Operator mandated testing only this task; no other `UNTESTED-*` file was used.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (`mac_stats` library: 854 passed, 0 failed, 0 ignored, finished ~1.16s; `Doc-tests mac_stats`: 1 ignored)
- `rg -n "not replacing cached|not updating cache|empty model list|MCACHE_LOG_TAG" src-tauri/src/ollama/model_list_cache.rs` — **pass**

**Notes:** No live Ollama for empty/error HTTP paths; acceptance criteria 1–3 only.

**Outcome:** All acceptance criteria satisfied → `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md`.

## Test report

**Date:** 2026-03-28 (hora local del workspace; no UTC).

**Preflight:** `tasks/UNTESTED-20260321-1855-openclaw-poisoned-cache-prevention.md` no existía; `003-tester/TESTER.md` paso 2: `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md` → `TESTING-20260321-1855-openclaw-poisoned-cache-prevention.md` (equivalente UNTESTED→TESTING). Solo esta tarea; ningún otro `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (librería `mac_stats`: 854 passed, 0 failed, 0 ignored, ~1.16s; `Doc-tests mac_stats`: 1 ignored)
- `rg -n "not replacing cached|not updating cache|empty model list|MCACHE_LOG_TAG" src-tauri/src/ollama/model_list_cache.rs` — **pass** (`MCACHE_LOG_TAG`, «not replacing cached data», «not updating cache»)

**Notes:** Sin daemon Ollama en vivo para vacío/error HTTP; criterios 1–3 de la tarea cumplidos.

**Outcome:** Todos los criterios de aceptación cumplidos → `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md`.

## Test report

**Date:** 2026-03-28 (hora local del workspace; no UTC).

**Preflight:** `tasks/UNTESTED-20260321-1855-openclaw-poisoned-cache-prevention.md` no existía en el arranque de esta ejecución; se aplicó `003-tester/TESTER.md` paso 2 renombrando `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md` → `TESTING-20260321-1855-openclaw-poisoned-cache-prevention.md` (equivalente UNTESTED→TESTING). El operador pidió probar solo ese id; no se usó ningún otro `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass** (`Finished dev profile` en ~0.22s)
- `cd src-tauri && cargo test` — **pass** (librería `mac_stats`: 854 passed, 0 failed, 0 ignored, ~1.16s; `Doc-tests mac_stats`: 1 ignored)
- `rg -n "not replacing cached|not updating cache|empty model list|MCACHE_LOG_TAG" src-tauri/src/ollama/model_list_cache.rs` — **pass** (`MCACHE_LOG_TAG`, lista vacía «not replacing cached data», error «not updating cache»)

**Notes:** Sin prueba en vivo contra Ollama con respuestas vacías o error HTTP; alcance acorde a criterios 1–3 de la tarea.

**Outcome:** Todos los criterios de aceptación cumplidos → `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md`.

## Test report

**Date:** 2026-03-28 (hora local del workspace; no UTC).

**Preflight:** `tasks/UNTESTED-20260321-1855-openclaw-poisoned-cache-prevention.md` no existía al inicio de esta ejecución; se aplicó `003-tester/TESTER.md` paso 2 renombrando `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md` → `TESTING-20260321-1855-openclaw-poisoned-cache-prevention.md` (equivalente UNTESTED→TESTING para este id). El operador pidió probar solo ese archivo; no se usó ningún otro `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (librería `mac_stats`: 854 passed, 0 failed, 0 ignored, ~1.16s; `Doc-tests mac_stats`: 1 ignored)
- `rg -n "not replacing cached|not updating cache|empty model list|MCACHE_LOG_TAG" src-tauri/src/ollama/model_list_cache.rs` — **pass** (`MCACHE_LOG_TAG`, lista vacía «not replacing cached data», error «not updating cache»)

**Notes:** Sin daemon Ollama en vivo para respuestas vacías o error HTTP; alcance acorde a criterios 1–3 de la tarea.

**Outcome:** Todos los criterios de aceptación cumplidos → `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md`.

## Test report

**Date:** 2026-03-28 (local workspace time; not UTC).

**Preflight:** `tasks/UNTESTED-20260321-1855-openclaw-poisoned-cache-prevention.md` was not present at run start; per `003-tester/TESTER.md` step 2, `tasks/CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md` was renamed to `tasks/TESTING-20260321-1855-openclaw-poisoned-cache-prevention.md` (equivalent UNTESTED→TESTING for this task id). Operator requested testing only this task; no other `UNTESTED-*` file was used.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (`mac_stats` library: 854 passed, 0 failed, 0 ignored; `Doc-tests mac_stats`: 1 ignored)
- `rg -n "not replacing cached|not updating cache|empty model list|MCACHE_LOG_TAG" src-tauri/src/ollama/model_list_cache.rs` — **pass** (matches `MCACHE_LOG_TAG`, empty-list “not replacing cached data”, fetch-error “not updating cache”)

**Notes:** Live Ollama daemon not exercised for empty or error HTTP responses; scope matches acceptance criteria 1–3.

**Outcome:** All acceptance criteria satisfied → `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md`.

## Test report

**Date:** 2026-03-28 (hora local del workspace; no UTC).

**Preflight:** `tasks/UNTESTED-20260321-1855-openclaw-poisoned-cache-prevention.md` no existía; `003-tester/TESTER.md` paso 2: `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md` → `TESTING-20260321-1855-openclaw-poisoned-cache-prevention.md` (equivalente UNTESTED→TESTING). Solo esta tarea; ningún otro `UNTESTED-*`.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (librería `mac_stats`: 854 passed, 0 failed, 0 ignored, ~1.16s; `Doc-tests mac_stats`: 1 ignored)
- `rg -n "not replacing cached|not updating cache|empty model list|MCACHE_LOG_TAG" src-tauri/src/ollama/model_list_cache.rs` — **pass** (`MCACHE_LOG_TAG`, lista vacía «not replacing cached data», error «not updating cache»)

**Notes:** Sin daemon Ollama en vivo para respuestas vacías o error HTTP; criterios 1–3 de la tarea cumplidos.

**Outcome:** Todos los criterios de aceptación cumplidos → `CLOSED-20260321-1855-openclaw-poisoned-cache-prevention.md`.
