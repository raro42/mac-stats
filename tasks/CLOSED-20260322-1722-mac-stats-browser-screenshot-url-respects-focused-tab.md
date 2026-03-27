# mac-stats: BROWSER_SCREENSHOT URL respects focused CDP tab

## Summary

`BROWSER_SCREENSHOT: <url>` must navigate and capture the **focused** automation tab (same index as `get_current_tab` / `BROWSER_NAVIGATE` / `new_tab`), not an arbitrary first tab, so multi-tab CDP sessions behave correctly.

## Acceptance criteria

1. `browser_agent/mod.rs` `take_screenshot_inner` (URL branch) uses `get_current_tab()` and documents that URL screenshots respect `CURRENT_TAB_INDEX`, not `tabs.first()`.
2. `commands/browser_tool_dispatch.rs` surfaces user-facing text that the URL path applies to the **focused** automation tab (consistent with agent descriptions).
3. `cargo check` and `cargo test --lib` succeed in `src-tauri/`.

## Verification (automated)

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test --lib
rg -n "take_screenshot URL path: using focused tab|get_current_tab\\(\\)|CURRENT_TAB_INDEX" src/browser_agent/mod.rs
rg -n "focused tab|BROWSER_SCREENSHOT.*URL" src/commands/browser_tool_dispatch.rs src/commands/agent_descriptions.rs
```

## Test report

- **Date:** 2026-03-27, local time of the environment where commands ran (not fixed UTC).
- **Preflight:** The path `tasks/UNTESTED-20260322-1722-mac-stats-browser-screenshot-url-respects-focused-tab.md` was **not present** in the working tree at the start of this run. The task body was written as `UNTESTED-…`, then renamed to `TESTING-…` per `003-tester/TESTER.md`. No other `UNTESTED-*` file was used.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed |
| Symbols | `rg -n` patterns from «Verification (automated)» (`cwd` `src-tauri/`) | **pass** — matches in `browser_agent/mod.rs`, `browser_tool_dispatch.rs`, `agent_descriptions.rs` |

- **Outcome:** Acceptance criteria 1–3 satisfied → **`CLOSED-…`**.

### Test report — 2026-03-27 (follow-up run, TESTER.md)

- **Date:** 2026-03-27, local time of the environment where commands ran.
- **Preflight:** `tasks/UNTESTED-20260322-1722-mac-stats-browser-screenshot-url-respects-focused-tab.md` was **not present** (only `CLOSED-20260322-1722-mac-stats-browser-screenshot-url-respects-focused-tab.md` exists). Per operator instruction, **no other** `UNTESTED-*` file was selected. The `UNTESTED-…` → `TESTING-…` rename could not be applied to a missing path; verification below is against this task’s acceptance criteria and automated checks.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed |
| Symbols | `rg` patterns from «Verification (automated)» (`browser_agent/mod.rs`, `browser_tool_dispatch.rs`, `agent_descriptions.rs`) | **pass** — focused-tab URL screenshot path and user-facing copy present |

- **Outcome:** All checks pass; task remains **`CLOSED-20260322-1722-mac-stats-browser-screenshot-url-respects-focused-tab.md`** (no `WIP-` needed).

### Test report — 2026-03-27 (TESTER.md run)

- **Date:** 2026-03-27, local time of the environment where commands ran.
- **Preflight:** Operator asked for `tasks/UNTESTED-20260322-1722-mac-stats-browser-screenshot-url-respects-focused-tab.md`; that path was **not present** (task was `CLOSED-…`). Per instruction, **no other** `UNTESTED-*` file was used. Renamed **`CLOSED-…` → `TESTING-…`** for this run, then verification below.

| Step | Command | Result |
|------|---------|--------|
| Check | `cd src-tauri && cargo check` | **pass** |
| Lib tests | `cd src-tauri && cargo test --lib` | **pass** — 854 passed, 0 failed |
| Symbols | `rg -n` patterns from «Verification (automated)» (`cwd` `src-tauri/`) | **pass** — `take_screenshot URL path: using focused tab` / `get_current_tab()` / `CURRENT_TAB_INDEX` in `browser_agent/mod.rs`; focused-tab copy and `BROWSER_SCREENSHOT` URL messaging in `browser_tool_dispatch.rs` and `agent_descriptions.rs` |

- **Outcome:** Acceptance criteria 1–3 satisfied → rename to **`CLOSED-20260322-1722-mac-stats-browser-screenshot-url-respects-focused-tab.md`**.
