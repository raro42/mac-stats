# Browser use — CDP health check ping (`1+1`)

## Goal

Before CDP browser tools run, mac-stats must detect a hung or dead Chrome while the WebSocket may still look open: optional child-PID liveness (`kill -0` on Unix), then a lightweight **`Runtime.evaluate("1+1")`** “ping” with a **hard wall-clock timeout** on a **plain `std::thread`** + `mpsc::recv_timeout`. This path must **never** nest Tokio `Handle::block_on` + `tokio::time::timeout` on the app’s shared runtime (current-thread executor would wedge).

## Acceptance criteria

1. `evaluate_one_plus_one_blocking_timeout` runs `tab.evaluate("1+1", false)` on a worker thread and uses `recv_timeout(BROWSER_CDP_HEALTH_CHECK_TIMEOUT)`; errors surface as **Browser unresponsive** messages where applicable.
2. `check_browser_alive` calls that helper and includes an explicit comment forbidding nested `block_on` + Tokio timeout (heartbeat / scheduler rationale).
3. On health-check failure, `clear_browser_session_on_error` clears the cached session for **Browser unresponsive** and for connection-style errors (`is_connection_error`), without conflating with unrelated retry paths (`should_retry_cdp_after_clearing_session` documents health wins over retry).

## Operator filename (TESTER.md)

- **This queue item:** after coder handoff, the file is **`tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`**. Follow your **`UNTESTED-` → `TESTING-`** rename while executing the test run.
- **`TESTPLAN-`** is only used while the **testing instructions** are being revised; it is **not** the name the operator should wait for in `tasks/`.

## Environment

- **Checkout:** mac-stats repository root (directory that contains `src-tauri/`).
- Run every command below from **repository root** unless `cd src-tauri` is shown.

## Testing instructions

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

**Do not** require a full **`cargo test --no-fail-fast`** over the entire `mac_stats` crate for this task. Unrelated tests may depend on home-directory layout, optional fixtures, or other modules; failures there are **out of scope** for this CDP health-check verification.

Gate the change with the **CDP retry vs health-check** unit tests only (same source file as the symbols above):

```bash
cd src-tauri && cargo test -p mac_stats --lib cdp_retry_
```

**Pass:** at least these tests run and succeed:

- `browser_agent::tests::cdp_retry_skipped_when_health_error_also_looks_like_connection_error`
- `browser_agent::tests::cdp_retry_allowed_for_plain_connection_error_without_health_prefix`

(Cargo may run additional tests whose names share the `cdp_retry_` prefix; **all** tests executed by this filter must pass.)

### 4) Spot-check — acceptance criterion 3 (required)

In `src-tauri/src/browser_agent/mod.rs`, locate `should_retry_cdp_after_clearing_session` (search by name). Confirm the comment/doc states that **health-check / “Browser unresponsive”** handling already clears the session and must **not** be treated like the generic “retry after connection error” path.

### Optional — full crate tests (informational only)

```bash
cd src-tauri && cargo test -p mac_stats --no-fail-fast
```

If this fails, **do not** fail the task **unless** the failing tests are the **`cdp_retry_`** tests above or clearly implicate `evaluate_one_plus_one_blocking_timeout` / `check_browser_alive` / `clear_browser_session_on_error`. Otherwise record the failure as **environment / unrelated suite**, not as a regression in this task.

## Canonical history

Cumulative tester reports and older verification notes: **`tasks/CLOSED-20260321-1345-browser-use-cdp-health-check-ping.md`** (reference only).
