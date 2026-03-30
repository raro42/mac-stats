# Browser use — CDP health check ping (`1+1`)

## Goal

Before CDP browser tools run, mac-stats must detect a hung or dead Chrome while the WebSocket may still look open: optional child-PID liveness (`kill -0` on Unix), then a lightweight **`Runtime.evaluate("1+1")`** “ping” with a **hard wall-clock timeout** on a **plain `std::thread`** + `mpsc::recv_timeout`. This path must **never** nest Tokio `Handle::block_on` + `tokio::time::timeout` on the app’s shared runtime (current-thread executor would wedge).

## Acceptance criteria

1. `evaluate_one_plus_one_blocking_timeout` runs `tab.evaluate("1+1", false)` on a worker thread and uses `recv_timeout(BROWSER_CDP_HEALTH_CHECK_TIMEOUT)`; errors surface as **Browser unresponsive** messages where applicable.
2. `check_browser_alive` calls that helper and includes an explicit comment forbidding nested `block_on` + Tokio timeout (heartbeat / scheduler rationale).
3. On health-check failure, `clear_browser_session_on_error` clears the cached session for **Browser unresponsive** and for connection-style errors (`is_connection_error`), without conflating with unrelated retry paths (`should_retry_cdp_after_clearing_session` documents health wins over retry).

## Operator filename (TESTER.md / 003-tester)

- **Pick up this path only:** `tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`. Rename **`UNTESTED-` → `TESTING-`** while you execute the steps below, then follow your runbook for **`TESTING-` → `CLOSED-`** or failure naming.
- If **`UNTESTED-…` is missing** but **`tasks/TESTPLAN-20260321-1345-browser-use-cdp-health-check-ping.md` exists:** the task is **not** queued for execution — **testing instructions are under revision**. Do **not** rename `TESTPLAN-` to `TESTING-` for a normal run; wait for the coder to rename **`TESTPLAN-` → `UNTESTED-`**, then pick up **`UNTESTED-…`** as usual.
- If **neither** `UNTESTED-…` **nor** `TESTPLAN-…` exists for this slug, treat that as a **defective task queue / handoff**. **Do not** infer pass/fail of the CDP health-check feature from a substitute file (e.g. `CLOSED-…` renamed to `TESTING-`) unless your operator guide explicitly allows it.
- **`TESTER.md` / templates** that still mandate a **full** `cargo test` / `cargo test --no-fail-fast` with **no** `cdp_retry_` filter for this slug are **superseded** by **steps 1–4** in this file for pass/fail (see **Pass/fail scope** below).
- **Outcome:** if steps **1–4** all pass, the task **passes** for this slug even when an **optional** unfiltered `cargo test -p mac_stats --no-fail-fast` would fail (Discord, scheduler, home-directory coupling, etc.). Use **TESTPLAN-** only when **testing instructions** or **queue filenames** are wrong — **not** when the narrow gate passes but the broad suite fails.

## Environment

- **Checkout:** mac-stats repository root (directory that contains `src-tauri/`).
- **Sanity check (once per session):** from that root, `test -f src-tauri/Cargo.toml` must succeed before running the `rg` lines (paths below assume repo root).
- Run every command below from **repository root** unless `cd src-tauri` is shown.
- **`rg`:** ripgrep must be available on `PATH` (same as other task files).
- **No live Chrome / CDP session** is required for steps **1–4** (static review, compile, lib unit tests, spot-check).
- **Rust:** stable toolchain able to build `mac_stats` (same as normal mac-stats development).

## Testing instructions

### Pass/fail scope (read first)

- **Required for a pass on this task:** steps **1–4** below (static review, compile, **targeted** lib tests, spot-check). All required commands must succeed.
- **Authoritative gate:** these four steps override any older checklist that treated **unfiltered** `cargo test` / `cargo test --no-fail-fast` / `cargo test -p mac_stats --no-fail-fast` as mandatory for this task. Failures in unrelated modules (e.g. Discord `pdfs_dir`, scheduler home-directory persistence) **do not** invalidate a pass when steps **1–4** succeed.
- **Checklist conflict:** if your runbook prints a **mandatory** unfiltered `cargo test` line for this slug, treat it as **stale for pass/fail**. You may still run it for curiosity or logs, but **closure** depends only on steps **1–4**. Never rename to **TESTPLAN-** solely because that unfiltered command failed while step **3** (`cdp_retry_`) passed.
- **Not required:** a full **`cargo test -p mac_stats --no-fail-fast`** (or whole-workspace test) over the entire crate. Other tests can fail in some environments (home-directory layout, `pdfs_dir`, scheduler persistence, etc.); those failures are **out of scope** and **must not** be used to fail this task unless the failing test is one of the **`cdp_retry_`** tests in step 3 or clearly implicates `evaluate_one_plus_one_blocking_timeout` / `check_browser_alive` / `clear_browser_session_on_error`.

### 1) Static review (required)

```bash
rg 'evaluate_one_plus_one_blocking_timeout|check_browser_alive|BROWSER_CDP_HEALTH_CHECK_TIMEOUT|clear_browser_session_on_error' src-tauri/src/browser_agent/mod.rs
rg 'block_on|Never use .Handle::block_on' src-tauri/src/browser_agent/mod.rs | head -n 20
```

**Pass:** the first command lists matches in `browser_agent/mod.rs`; the second’s first lines (up to **20** shown) include the anti-`block_on` comment / doc near `check_browser_alive` and `evaluate_one_plus_one_blocking_timeout`. If you need more context, re-run the second command without `| head -n 20`.

### 2) Compile (required)

```bash
cd src-tauri && cargo check -p mac_stats
```

From `src-tauri/`, `cargo check` alone is **acceptable** if it builds the `mac_stats` package in your checkout (explicit `-p mac_stats` avoids ambiguity in multi-package workspaces).

### 3) Targeted unit tests (required)

Use **only** this filter (do **not** require the entire lib test suite for this task):

```bash
cd src-tauri && cargo test -p mac_stats --lib cdp_retry_ --no-fail-fast
```

**Pass:** Cargo **runs** the `cdp_retry_`-filtered lib tests and reports a **non-zero** count. Expect **at least 2** tests (current names below). If Cargo reports **0** tests executed, **fail** this step (wrong directory, typo in filter, or renamed tests — escalate).

- `browser_agent::tests::cdp_retry_skipped_when_health_error_also_looks_like_connection_error`
- `browser_agent::tests::cdp_retry_allowed_for_plain_connection_error_without_health_prefix`

**All** executed tests must pass. If additional tests share the `cdp_retry_` name prefix, they are in scope too — **every** test run by this exact command must pass.

**Copy/paste note:** the filter is the substring `cdp_retry_` (underscore at the end), not `cdp_retry` alone.

**Expected shape (illustrative):** Cargo should print a line like `running 2 tests`, then `test result: ok. 2 passed; …` and a **large** `N filtered out` count (hundreds) is normal — that means only the `cdp_retry_` tests ran. **Do not** require `N filtered out` to match a specific number across checkouts.

### 4) Spot-check — acceptance criterion 3 (required)

In `src-tauri/src/browser_agent/mod.rs`, locate `should_retry_cdp_after_clearing_session` (search by name). Confirm the comment/doc states that **health-check / “Browser unresponsive”** handling already clears the session and must **not** be treated like the generic “retry after connection error” path.

### Optional — full crate tests (informational only)

```bash
cd src-tauri && cargo test -p mac_stats --no-fail-fast
```

This is **diagnostic noise tolerance** only. **Do not** use this command as the **required** pass/fail bar for this task. **Do not** list it in your report as a failed **acceptance** step when steps **1–4** passed.

If it fails, **do not** fail the task **unless** the failure is in the **`cdp_retry_`** tests from step 3 or clearly tied to the health-check symbols in step 1. Otherwise record it as **environment / unrelated suite** in your report, not as a regression for this task.

## Canonical history

Cumulative tester reports and older verification notes: **`tasks/CLOSED-20260321-1345-browser-use-cdp-health-check-ping.md`** (reference only).
