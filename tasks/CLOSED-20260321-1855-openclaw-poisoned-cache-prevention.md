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
