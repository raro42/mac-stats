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
