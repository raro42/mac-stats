# Browser use — CDP health check ping (`1+1`)

## Goal

Before CDP browser tools run, mac-stats must detect a hung or dead Chrome while the WebSocket may still look open: optional child-PID liveness (`kill -0` on Unix), then a lightweight **`Runtime.evaluate("1+1")`** “ping” with a **hard wall-clock timeout** on a **plain `std::thread`** + `mpsc::recv_timeout`. This path must **never** nest Tokio `Handle::block_on` + `tokio::time::timeout` on the app’s shared runtime (current-thread executor would wedge).

## Acceptance criteria

1. `evaluate_one_plus_one_blocking_timeout` runs `tab.evaluate("1+1", false)` on a worker thread and uses `recv_timeout(BROWSER_CDP_HEALTH_CHECK_TIMEOUT)`; errors surface as **Browser unresponsive** messages where applicable.
2. `check_browser_alive` calls that helper and includes an explicit comment forbidding nested `block_on` + Tokio timeout (heartbeat / scheduler rationale).
3. On health-check failure, `clear_browser_session_on_error` clears the cached session for **Browser unresponsive** and for connection-style errors (`is_connection_error`), without conflating with unrelated retry paths (`should_retry_cdp_after_clearing_session` documents health wins over retry).

## Operator filename (TESTER.md / 003-tester)

- **Pick up this path only:** `tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`. Rename **`UNTESTED-` → `TESTING-`** while you execute the steps below, then follow your runbook for **`TESTING-` → `CLOSED-`** or failure naming.
- If **`tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md` is missing**, treat that as a **defective task queue / handoff** (wrong filename in the tree or coder did not publish `UNTESTED-` yet). **Do not** infer pass/fail of the CDP health-check feature from a substitute file unless your operator guide explicitly allows it.
- **`TESTPLAN-`** (same stamp and slug) means the **testing instructions** are under revision; it is **not** the normal operator pickup name. After the coder finishes, the file must be renamed to **`UNTESTED-`** for the next test run.

## Environment

- **Checkout:** mac-stats repository root (directory that contains `src-tauri/`).
- Run every command below from **repository root** unless `cd src-tauri` is shown.
- **`rg`:** ripgrep must be available on `PATH` (same as other task files).

## Testing instructions

### Pass/fail scope (read first)

- **Required for a pass on this task:** steps **1–4** below (static review, compile, **targeted** lib tests, spot-check). All required commands must succeed.
- **Not required:** a full **`cargo test -p mac_stats --no-fail-fast`** over the whole crate. Other tests can fail in some environments (home-directory layout, `pdfs_dir`, scheduler persistence, etc.); those failures are **out of scope** and **must not** be used to fail this task unless the failing test is one of the **`cdp_retry_`** tests in step 3 or clearly implicates `evaluate_one_plus_one_blocking_timeout` / `check_browser_alive` / `clear_browser_session_on_error`.

### 1) Static review (required)

```bash
rg 'evaluate_one_plus_one_blocking_timeout|check_browser_alive|BROWSER_CDP_HEALTH_CHECK_TIMEOUT|clear_browser_session_on_error' src-tauri/src/browser_agent/mod.rs
rg 'block_on|Never use .Handle::block_on' src-tauri/src/browser_agent/mod.rs | head -n 20
```

**Pass:** the first command lists matches in `browser_agent/mod.rs`; the second shows the anti-`block_on` comment / doc near `check_browser_alive` and `evaluate_one_plus_one_blocking_timeout`.

### 2) Compile (required)

```bash
cd src-tauri && cargo check -p mac_stats
```

### 3) Targeted unit tests (required)

Use **only** this filter (do **not** require the entire lib test suite for this task):

```bash
cd src-tauri && cargo test -p mac_stats --lib cdp_retry_ --no-fail-fast
```

**Pass:** Cargo runs the `cdp_retry_`-filtered lib tests (typically **2** tests); **all** of them pass, including at least:

- `browser_agent::tests::cdp_retry_skipped_when_health_error_also_looks_like_connection_error`
- `browser_agent::tests::cdp_retry_allowed_for_plain_connection_error_without_health_prefix`

If additional tests ever share the `cdp_retry_` name prefix, they are in scope too — **every** test executed by this command must pass.

### 4) Spot-check — acceptance criterion 3 (required)

In `src-tauri/src/browser_agent/mod.rs`, locate `should_retry_cdp_after_clearing_session` (search by name). Confirm the comment/doc states that **health-check / “Browser unresponsive”** handling already clears the session and must **not** be treated like the generic “retry after connection error” path.

### Optional — full crate tests (informational only)

```bash
cd src-tauri && cargo test -p mac_stats --no-fail-fast
```

If this fails, **do not** fail the task **unless** the failure is in the **`cdp_retry_`** tests from step 3 or clearly tied to the health-check symbols in step 1. Otherwise record it as **environment / unrelated suite** in your report, not as a regression for this task.

## Canonical history

Cumulative tester reports and older verification notes: **`tasks/CLOSED-20260321-1345-browser-use-cdp-health-check-ping.md`** (reference only).
