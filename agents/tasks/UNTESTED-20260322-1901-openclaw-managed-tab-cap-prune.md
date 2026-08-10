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
cargo test --lib managed_tab_cap_focus
```

## Verification (optional — needs Chromium with CDP, e.g. port 9222)

```bash
cd src-tauri && MAC_STATS_BROWSER_MAX_PAGE_TABS=3 cargo run --example managed_tab_cap_smoke
```

Skip or note **blocked** if no CDP browser is available; automated criteria 1–4 still gate closure.

## Implementation notes (coder, 2026-03-28 UTC)

- `try_enforce_browser_tab_limit` logs under target **`browser/cdp`** (`managed tab cap:` before/after counts, per-close lines; one **warn** per process if enforcement is abandoned).
- Call sites after successful tool completion: main **`navigate_and_get_state_inner`**, **`BROWSER_CLICK`** (index and viewport coordinates), **`BROWSER_HOVER`**, **`BROWSER_DRAG`**, and **screenshot path** that navigates to a URL before capture.
- Added **six** `#[cfg(test)]` unit tests for **`new_focus_index_after_close`** (same adjustment as `BROWSER_CLOSE_TAB` when pruning) so tab-index behaviour is covered without a live browser.

---

## Testing instructions

### What to verify

- With cap **> 0**, after the listed tools succeed, open page tab count does not exceed the cap; the **automation-focused** tab is never closed; **`CURRENT_TAB_INDEX`** stays consistent after closes (same rules as after **`BROWSER_CLOSE_TAB`**).
- With cap **0** (default), no pruning runs and behaviour matches previous releases.
- **`Config::browser_max_page_tabs()`** respects env **`MAC_STATS_BROWSER_MAX_PAGE_TABS`** and **`config.json`** key **`browserMaxPageTabs`**, clamped to **0..=64**.
- Automated build/tests pass, including the **`managed_tab_cap_focus_*`** unit tests.

### How to test

1. From repo root: `cd src-tauri && cargo check && cargo test --lib`.
2. Run only the new focus-index tests: `cd src-tauri && cargo test --lib managed_tab_cap_focus`.
3. Confirm wiring: `rg -n "fn try_enforce_browser_tab_limit" src/browser_agent/mod.rs` and `rg -n "try_enforce_browser_tab_limit\\(" src/browser_agent/mod.rs` (definition plus several call sites).
4. **Optional (CDP):** Start Chromium with remote debugging (e.g. port **9222**). Then:
   `cd src-tauri && MAC_STATS_BROWSER_MAX_PAGE_TABS=3 cargo run --example managed_tab_cap_smoke`
   Expect printed tab counts **≤ 3** after each `new_tab` step and **`DONE: managed tab cap smoke passed`**.
5. **Optional (logs):** Run the app with **`-vv`** and trigger a capped session; in **`~/.mac-stats/debug.log`**, search for **`managed tab cap`** lines showing before/after counts.

### Pass/fail criteria

- **Pass:** `cargo check` and `cargo test --lib` succeed; all six **`managed_tab_cap_focus_*`** tests pass; `rg` shows **`try_enforce_browser_tab_limit`** defined and invoked from multiple success paths; optional smoke example completes without **`FAIL:`** and respects cap when CDP is available.
- **Fail:** Build or lib test failures; smoke example reports **`tab_count` exceeds cap**; focused tab incorrectly closed (would show as wrong **`(active)`** tab or broken **`switch_tab_to_index`** in smoke); or enforcement **`warn`** storms (more than one structural skip warn per process for the same failure mode — at most one warn is expected for abandoned enforcement).
