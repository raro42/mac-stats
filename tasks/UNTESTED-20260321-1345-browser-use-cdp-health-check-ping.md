# Browser use — CDP health check ping (`1+1`)

## Goal

Before CDP browser tools run, mac-stats must detect a hung or dead Chrome while the WebSocket may still look open: optional child-PID liveness (`kill -0` on Unix), then a lightweight **`Runtime.evaluate("1+1")`** “ping” with a **hard wall-clock timeout** on a **plain `std::thread`** + `mpsc::recv_timeout`. This path must **never** nest Tokio `Handle::block_on` + `tokio::time::timeout` on the app’s shared runtime (current-thread executor would wedge).

## Acceptance criteria

1. `evaluate_one_plus_one_blocking_timeout` runs `tab.evaluate("1+1", false)` on a worker thread and uses `recv_timeout(BROWSER_CDP_HEALTH_CHECK_TIMEOUT)`; errors surface as **Browser unresponsive** messages where applicable.
2. `check_browser_alive` calls that helper and includes an explicit comment forbidding nested `block_on` + Tokio timeout (heartbeat / scheduler rationale).
3. On health-check failure, `clear_browser_session_on_error` clears the cached session for **Browser unresponsive** and for connection-style errors (`is_connection_error`), without conflating with unrelated retry paths (`should_retry_cdp_after_clearing_session` documents health wins over retry).

## Operator filename (TESTER.md / 003-tester)

- **Executable queue file (only this name):** `tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`. At the start of a normal test run, rename **`UNTESTED-` → `TESTING-`**, execute the steps below, then follow your runbook for **`TESTING-` → `CLOSED-`** or failure naming.
- **Handoff defect — do not substitute other prefixes for this slug:** If **`UNTESTED-…` is missing** at repo tip, **stop**. Do **not** rename `CLOSED-…` or `TESTPLAN-…` to `TESTING-…` to “run” this task unless your operator guide **explicitly** authorizes that exception. Typical fix: pull/sync to a revision where the coder published **`UNTESTED-…`** (after **`TESTPLAN-` → `UNTESTED-`**), or file a handoff defect. Treating historical **`CLOSED-…`** as the live test target caused false **TESTPLAN-** outcomes (full-suite noise) and bypassed the narrow gate below.
- If **`tasks/TESTPLAN-20260321-1345-browser-use-cdp-health-check-ping.md` exists** (and **`UNTESTED-…` is absent**): testing instructions are **under revision**. Do **not** rename `TESTPLAN-` → `TESTING-` for execution; wait for **`TESTPLAN-` → `UNTESTED-`**, then pick up **`UNTESTED-…`**.
- **`TESTER.md` / templates** that still mandate an **unfiltered** `cargo test` / `cargo test --no-fail-fast` / `cargo test -p mac_stats --no-fail-fast` with **no** `cdp_retry_` filter for this slug are **stale for pass/fail**. **Closure** for this slug depends **only** on **steps 1–4** in **Testing instructions** (see **Pass/fail scope**).
- **Outcome:** If steps **1–4** all pass, the task **passes** for this slug even when an **optional** unfiltered `cargo test -p mac_stats --no-fail-fast` would fail (Discord `pdfs_dir`, scheduler home-directory coupling, etc.). Use **TESTPLAN-** only when **testing instructions** or **queue filenames/handoff** are wrong — **not** when the narrow gate passes but the broad suite fails.

## Environment

- **Checkout:** mac-stats repository root (directory that contains `src-tauri/`).
- **Sanity check (once per session):** from that root, `test -f src-tauri/Cargo.toml` must succeed before running the `rg` lines (paths below assume repo root).
- Run every command below from **repository root** unless `cd src-tauri` is shown.
- **`rg`:** ripgrep must be available on `PATH` (same as other task files).
- **No live Chrome / CDP session** is required for steps **1–4** (static review, compile, lib unit tests, spot-check).
- **Rust:** stable toolchain able to build `mac_stats` (same as normal mac-stats development).

## Testing instructions

### Pass/fail scope (read first)

- **Required for a pass on this task:** steps **1–4** below (static review, compile, **targeted lib tests**, spot-check). All required commands must succeed.
- **Not a required step:** There is **no** step in this document that runs **`cargo test --no-fail-fast`** or **`cargo test -p mac_stats --no-fail-fast`** **without** the **`cdp_retry_`** filter. If your runbook prints such a line as **mandatory** for this slug, treat that line as **obsolete** for pass/fail. You may run an unfiltered suite for diagnostics, but **do not** fail or emit **TESTPLAN-** solely because that optional command failed while step **3** passed.
- **Authoritative gate:** steps **1–4** override any older checklist that treated **unfiltered** crate tests as mandatory for this task. Failures in unrelated modules **do not** invalidate a pass when steps **1–4** succeed.
- **Checklist conflict:** Never rename to **TESTPLAN-** solely because an unfiltered `cargo test` failed while the **`cdp_retry_`** command in step **3** passed.
- **Not required:** a full **`cargo test -p mac_stats --no-fail-fast`** over the entire crate. Other tests can fail in some environments; those failures are **out of scope** unless the failing test is one of the **`cdp_retry_`** tests in step 3 or clearly implicates `evaluate_one_plus_one_blocking_timeout` / `check_browser_alive` / `clear_browser_session_on_error`.

### Copy-paste — full gate (from repository root)

Run this block **as-is** after the sanity check (`test -f src-tauri/Cargo.toml`). It is equivalent to steps **1–3**; still perform step **4** (manual read) afterward.

```bash
set -e
rg 'evaluate_one_plus_one_blocking_timeout|check_browser_alive|BROWSER_CDP_HEALTH_CHECK_TIMEOUT|clear_browser_session_on_error' src-tauri/src/browser_agent/mod.rs
rg 'block_on|Never use .Handle::block_on' src-tauri/src/browser_agent/mod.rs | head -n 20
( cd src-tauri && cargo check -p mac_stats && cargo test -p mac_stats --lib cdp_retry_ --no-fail-fast )
```

The subshell `( cd src-tauri && … )` keeps the **current directory** at repo root after the block (so `rg` paths stay valid if you append more lines). If you already `cd src-tauri`, run `cargo check -p mac_stats` and `cargo test -p mac_stats --lib cdp_retry_ --no-fail-fast` there instead.

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

From `src-tauri/`, `cargo check -p mac_stats` alone is **acceptable** (explicit package avoids ambiguity in multi-package workspaces).

### 3) Targeted unit tests (required)

Use **only** this command for the automated test gate ( **`--lib`** limits scope to library unit tests and avoids pulling unrelated targets into the bar):

```bash
cd src-tauri && cargo test -p mac_stats --lib cdp_retry_ --no-fail-fast
```

**Pass:** Cargo **runs** the `cdp_retry_`-filtered lib tests and reports a **non-zero** count. Expect **at least 2** tests (current names below). If Cargo reports **0** tests executed, **fail** this step (wrong directory, typo in filter, wrong package, or renamed tests — escalate).

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

This is **diagnostic** only. **Do not** use this command as the **required** pass/fail bar for this task. **Do not** list it in your report as a failed **acceptance** step when steps **1–4** passed.

If it fails, **do not** fail the task **unless** the failure is in the **`cdp_retry_`** tests from step 3 or clearly tied to the health-check symbols in step 1. Otherwise record it as **environment / unrelated suite** in your report, not as a regression for this task.

## Canonical history

Cumulative tester reports and older verification notes: **`tasks/CLOSED-20260321-1345-browser-use-cdp-health-check-ping.md`** (reference only).
