# CLOSED — browser-use in-page search and CSS query (2026-03-21)

## Goal

Verify **BROWSER_SEARCH_PAGE** (in-page text search with optional `css_scope`) and **BROWSER_QUERY** (CSS `querySelectorAll` with optional `attrs=`).

## References

- `src-tauri/src/commands/browser_tool_dispatch.rs` — `parse_browser_search_page_arg`, `parse_browser_query_arg`, `handle_browser_search_page`, `handle_browser_query`, unit tests
- `src-tauri/src/browser_agent/mod.rs` — `search_page_text`, `browser_query`
- `src-tauri/examples/example_com_search_twice.rs` — optional smoke for repeated search

## Acceptance criteria

1. **Build:** `cargo check` in `src-tauri/` succeeds.
2. **Tests:** `cargo test` in `src-tauri/` succeeds (no new failures attributable to search/query paths).
3. **Static verification:** Dispatch and browser agent still expose search/query handlers, parsers, and parsing unit tests (spot-check via `rg` or read).

## Verification commands

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test
```

Optional spot-check:

```bash
rg -n "handle_browser_search_page|handle_browser_query|parse_browser_search_page_arg|parse_browser_query_arg" src-tauri/src/commands/browser_tool_dispatch.rs
rg -n "fn search_page_text|pub fn browser_query" src-tauri/src/browser_agent/mod.rs
```

## Test report

**Date:** 2026-03-27 (local operator environment).

**Preflight:** The path `tasks/UNTESTED-20260321-1635-browser-use-in-page-search-and-css-query.md` was not present in the workspace at the start of this run; the task body was materialized as `UNTESTED-…`, then renamed to `TESTING-…` per `003-tester/TESTER.md` before verification. No other `UNTESTED-*` file was used.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (854 tests in `mac_stats` library; 0 failed; 1 doc-test ignored)

**Static spot-check**

- `browser_tool_dispatch.rs`: `parse_browser_search_page_arg`, `parse_browser_query_arg`, `handle_browser_search_page`, `handle_browser_query`, and parsing unit tests (e.g. `parses_search_page_css_scope`, `parses_browser_query_attrs`, fused-token regressions).
- `browser_agent/mod.rs`: `search_page_text` (~8631), `browser_query` (~8847), plus `search_page_text_from_plain_text_*` unit tests.

**Outcome:** All acceptance criteria satisfied for this verification pass. Live CDP search/query against real pages was not exercised end-to-end in this automated run (operator may run `cargo run --example example_com_search_twice` optionally).

### Re-verification (2026-03-27, local)

**Rename step:** `tasks/UNTESTED-20260321-1635-browser-use-in-page-search-and-css-query.md` was **not** in the workspace; the task already existed as `tasks/CLOSED-20260321-1635-browser-use-in-page-search-and-css-query.md`. Per `003-tester/TESTER.md`, no `UNTESTED-→TESTING-` rename was performed. No other `UNTESTED-*` task was used.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (854 passed, 0 failed; 1 doc-test ignored)

**Static spot-check (`rg`)**

- `browser_tool_dispatch.rs`: `parse_browser_search_page_arg`, `parse_browser_query_arg`, `handle_browser_search_page`, `handle_browser_query` present.
- `browser_agent/mod.rs`: `search_page_text`, `browser_query` present.

**Outcome:** Acceptance criteria still satisfied. Filename remains **`CLOSED-20260321-1635-browser-use-in-page-search-and-css-query.md`** (no `WIP-`).
