# Browser use — CDP health check ping (`1+1`)

> **Queue file:** `tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md` — testers rename **`UNTESTED-` → `TESTING-`** to execute. **Testing instructions** were revised **2026-03-30** (coder pass): (a) the gate was mis-read as an **unfiltered** crate `cargo test`, which fails for **unrelated** tests — **not** a CDP regression; (b) the **optional** full-suite snippet wrongly said “from `src-tauri/`” but still used `cd src-tauri` (always wrong there); (c) `rg … | head` under `set -o pipefail` can fail with a **non-zero** pipeline exit when `head` closes the pipe (**SIGPIPE**), producing a **false** static-review failure — use **`rg -m 20`** instead. The **only** automated test gate remains step **3** with the literal filter **`cdp_retry_`**.

## Goal

Before CDP browser tools run, mac-stats must detect a hung or dead Chrome while the WebSocket may still look open: optional child-PID liveness (`kill -0` on Unix), then a lightweight **`Runtime.evaluate("1+1")`** “ping” with a **hard wall-clock timeout** on a **plain `std::thread`** + `mpsc::recv_timeout`. This path must **never** nest Tokio `Handle::block_on` + `tokio::time::timeout` on the app’s shared runtime (current-thread executor would wedge).

## Acceptance criteria

1. `evaluate_one_plus_one_blocking_timeout` runs `tab.evaluate("1+1", false)` on a worker thread and uses `recv_timeout(BROWSER_CDP_HEALTH_CHECK_TIMEOUT)`; errors surface as **Browser unresponsive** messages where applicable.
2. `check_browser_alive` calls that helper and includes an explicit comment forbidding nested `block_on` + Tokio timeout (heartbeat / scheduler rationale).
3. On health-check failure, `clear_browser_session_on_error` clears the cached session for **Browser unresponsive** and for connection-style errors (`is_connection_error`), without conflating with unrelated retry paths (`should_retry_cdp_after_clearing_session` documents health wins over retry).

## Operator filename (TESTER.md / 003-tester)

- **If the task file on disk is `TESTPLAN-20260321-1345-browser-use-cdp-health-check-ping.md`:** instructions are **under revision**. Testers **must not** rename **`TESTPLAN-` → `TESTING-`** or execute the gate until a coder publishes **`UNTESTED-…`** (same stamp and slug).
- **Executable queue file (only this name for a real run):** `tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`. At the start of a normal test run, rename **`UNTESTED-` → `TESTING-`**, execute the steps below, then follow your runbook for **`TESTING-` → `CLOSED-`** or failure naming.
- **Handoff defect — do not substitute other prefixes for this slug:** If **`UNTESTED-…` is missing** at repo tip, **stop**. Do **not** rename `CLOSED-…` or `TESTPLAN-…` to `TESTING-…` to “run” this task unless your operator guide **explicitly** authorizes that exception. Typical fix: pull/sync to a revision where the coder published **`UNTESTED-…`** (after **`TESTPLAN-` → `UNTESTED-`**), or file a handoff defect. Treating historical **`CLOSED-…`** as the live test target caused false **TESTPLAN-** outcomes (full-suite noise) and bypassed the narrow gate below.
- If **`tasks/TESTPLAN-20260321-1345-browser-use-cdp-health-check-ping.md` exists** (and **`UNTESTED-…` is absent**): testing instructions are **under revision**. Do **not** rename `TESTPLAN-` → `TESTING-` for execution; wait for **`TESTPLAN-` → `UNTESTED-`**, then pick up **`UNTESTED-…`**.
- **`TESTER.md` / templates** that still mandate an **unfiltered** `cargo test` / `cargo test --no-fail-fast` / `cargo test -p mac_stats --no-fail-fast` with **no** `cdp_retry_` filter for this slug are **stale for pass/fail**. **Closure** for this slug depends **only** on **steps 1–4** in **Testing instructions** (see **Pass/fail scope**).
- **Outcome:** If steps **1–4** all pass, the task **passes** for this slug even when an **optional** unfiltered `cargo test -p mac_stats --no-fail-fast` would fail (Discord `pdfs_dir`, scheduler home-directory coupling, etc.). Use **TESTPLAN-** only when **testing instructions** or **queue filenames/handoff** are wrong — **not** when the narrow gate passes but the broad suite fails.

## Environment

- **Checkout:** mac-stats repository root (directory that contains `src-tauri/`). This repo has **no** workspace `Cargo.toml` at the repository root — the package lives under **`src-tauri/`** only.
- **Sanity check (once per session):** from repo root, `test -f src-tauri/Cargo.toml && test -f src-tauri/src/browser_agent/mod.rs` must succeed before the gate. If this fails, you are in the wrong directory (e.g. `tasks/`, another clone, or inside `src-tauri/` with wrong relative paths for `rg`) — `cd` to the repo root first.
- **From `src-tauri/` instead:** if your shell cwd is already **`…/mac-stats/src-tauri`**, use **`test -f Cargo.toml && test -f src/browser_agent/mod.rs`** before running **alternate** commands. Do **not** run `cd src-tauri` when you are already inside `src-tauri/` (that `cd` fails). For **`rg`**, either `cd` back to repo root and keep paths `src-tauri/src/...`, or run `rg … src/browser_agent/mod.rs` from `src-tauri/` (no `src-tauri/` prefix).
- **Hard footgun — no manifest at repo root:** this tree has **no** `Cargo.toml` at the repository root. Running plain `cargo check` or `cargo test` **without** `--manifest-path src-tauri/Cargo.toml` from root typically errors (e.g. *could not find `Cargo.toml`* or picks up the wrong workspace). That is **not** a product failure — fix the invocation.
- **Preferred cwd for the gate:** stay at **repository root** for the whole copy-paste block. Use **`cargo … --manifest-path src-tauri/Cargo.toml`** so you never depend on `cd src-tauri` (a common source of “0 tests” when someone runs `cargo test` from root without a manifest).
- **Alternate cwd:** you may instead `cd` to **`src-tauri/`** once (from repo root) and run `cargo check -p mac_stats` / `cargo test -p mac_stats --lib cdp_retry_ --no-fail-fast` with **no** `--manifest-path`. If you do, stay in `src-tauri/` for those **cargo** commands; use the path rules above for **`rg`** (either return to repo root for `src-tauri/src/browser_agent/mod.rs`, or use `src/browser_agent/mod.rs` from inside `src-tauri/`).
- **Already in `src-tauri/`:** run **`cargo … -p mac_stats`** with **no** leading `cd src-tauri` (that path does not exist inside `src-tauri/`). Same for **`rg`**: paths are **`src/browser_agent/mod.rs`**, not `src-tauri/src/...`.
- **Shell / pipelines:** The copy-paste block is written for **bash**. **zsh** on macOS also supports `set -o pipefail` (or use `setopt pipefail`). **fish** has different pipeline semantics — run the block via `bash -lc '…'` or execute each command manually. Static review uses **`rg -m 20`** (no `| head`) so **`set -o pipefail` does not** turn a successful search into a pipeline failure (**SIGPIPE** when `head` stops reading).
- **`rg`:** ripgrep must be available on `PATH` (same as other task files).
- **No live Chrome / CDP session** is required for steps **1–4** (static review, compile, lib unit tests, spot-check).
- **Rust:** stable toolchain able to build `mac_stats` (same as normal mac-stats development).

## Testing instructions

### Commands by cwd (pick exactly one column)

Use **one** column end-to-end for **steps 1–3** (paths and **cargo** flags must match that cwd).

| Step | **Repository root** (directory containing `src-tauri/`) | **Already inside** `…/mac-stats/src-tauri/` |
|------|--------------------------------------------------------|---------------------------------------------|
| **1 — first `rg`** | `rg 'evaluate_one_plus_one_blocking_timeout\|check_browser_alive\|BROWSER_CDP_HEALTH_CHECK_TIMEOUT\|clear_browser_session_on_error' src-tauri/src/browser_agent/mod.rs` | `rg '…' src/browser_agent/mod.rs` (same pattern, path **without** `src-tauri/` prefix) |
| **1 — second `rg`** | `rg -n -m 20 'Never use.*Handle::block_on\|recv_timeout\(BROWSER_CDP_HEALTH_CHECK_TIMEOUT\)' src-tauri/src/browser_agent/mod.rs` | same pattern, path `src/browser_agent/mod.rs` |
| **2 — `cargo check`** | `cargo check --manifest-path src-tauri/Cargo.toml -p mac_stats` | `cargo check -p mac_stats` (**no** `cd src-tauri`) |
| **3 — gate `cargo test`** | `cargo test --manifest-path src-tauri/Cargo.toml -p mac_stats --lib cdp_retry_ --no-fail-fast` | `cargo test -p mac_stats --lib cdp_retry_ --no-fail-fast` (**no** `cd src-tauri`) |

If you mix columns (e.g. root paths while cwd is `src-tauri/`), you get empty **`rg`** output or **`running 0 tests`** — that is an **execution** mistake, not a product failure.

### Tester TL;DR (pass/fail)

1. **Cwd:** repo root **or** `src-tauri/` — follow **Environment** (including “already in `src-tauri/`” and **no** root `Cargo.toml`).
2. **Gate:** steps **1–4** only. Step **3** must be exactly the **`cdp_retry_`** lib test command (substring visible on the command line).
3. **Ignore** any **`cargo test`** that does **not** include the substring **`cdp_retry_`** on the command line — including **`cargo test -p mac_stats --lib --no-fail-fast`** (runs the **entire** lib suite, hundreds of tests) and **`cargo test -p mac_stats --no-fail-fast`**. Those are **diagnostics only**, not acceptance.
4. **Step 4** is a **manual** read in the editor — it is **not** included in the bash copy-paste block.
5. **Report:** paste the **exact** step **3** command you ran and Cargo lines showing **`running N tests`** with **N ≥ 1** and **`test result: ok`** (see **Report evidence**).

### If something looks wrong (symptom → fix)

| Symptom | Likely cause | Fix |
|--------|----------------|-----|
| `could not find Cargo.toml` (or similar) when running **cargo** from repo root | Invoked **cargo** without `--manifest-path src-tauri/Cargo.toml` | Use the **preferred** commands from root, or `cd src-tauri` and use **alternate** form **without** `--manifest-path`. |
| `running 0 tests` for step **3** | Wrong cwd/manifest, wrong `-p`, or filter typo (`cdp_retry` vs `cdp_retry_`) | Run `cargo test … --lib cdp_retry_ -- --list` (same manifest/cwd as step **3**); expect **≥ 1** listed test. |
| First **`rg`** prints nothing or exits **1** | Wrong directory or wrong path (`src/browser_agent/...` from root instead of `src-tauri/src/...`) | `pwd`; from root paths **must** start with `src-tauri/src/browser_agent/mod.rs`. |
| Second **`rg`** prints nothing | Same cwd/path issue | Same as above. |
| Second **`rg`** pipeline fails with **`set -e` / `pipefail`** though matches exist | Historically `rg \| head`; **`head`** closes the pipe → **`rg`** can exit non-zero | Use **`rg -n -m 20 '…'`** as in the copy-paste block (no ` \| head`). |
| Optional full-suite command errors with **`cd: src-tauri: No such file`** | Copied **“From `src-tauri/`”** block but shell cwd was **already** `src-tauri/` | From **`src-tauri/`**, run **`cargo test -p mac_stats --no-fail-fast`** only — **no** `cd src-tauri`. |

### Authoritative automated test command (step 3 — paste exactly)

The **only** `cargo test` invocation that counts toward pass/fail for this task is the **library** run with the **`cdp_retry_`** name filter (substring, trailing underscore):

```bash
cargo test --manifest-path src-tauri/Cargo.toml -p mac_stats --lib cdp_retry_ --no-fail-fast
```

**From `src-tauri/`:** `cargo test -p mac_stats --lib cdp_retry_ --no-fail-fast`

Your report **must** show this filter (the literal substring `cdp_retry_`) on the test command line. If the command you ran has no `cdp_retry_` token, you have **not** executed the gate — run step **3** again before choosing an outcome.

### What went wrong in the last TESTPLAN cycle (instruction defect, not product)

Mistakes that produced **TESTPLAN-** while the CDP implementation was fine:

1. **Wrong test bar:** Some runs used **`cd src-tauri && cargo test --no-fail-fast`** (entire crate, **no** `cdp_retry_` filter). That command is **not** in steps **1–4** and is **never** the acceptance gate for this slug. Failures there (Discord `pdfs_dir`, scheduler tests touching the real home directory, etc.) are **environment / unrelated suite** noise.
2. **Wrong queue file:** **`tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`** was missing at repo tip; testers retitled **`CLOSED-…`** or **`TESTPLAN-…`** to **`TESTING-…`** instead of waiting for **`UNTESTED-…`**. That bypassed this document’s narrow gate and invited runbook confusion.
3. **Wrong cwd / manifest:** Running **cargo** from repo root **without** `--manifest-path src-tauri/Cargo.toml`, or running **`cd src-tauri`** when already inside **`src-tauri/`**, or using **`rg`** paths that omit the **`src-tauri/`** prefix from repo root — yields “no manifest”, **`cd` errors**, **0 tests**, or empty **`rg`** output. Treat as **instruction/environment execution**, not a code regression.
4. **Wrong optional-suite snippet:** The diagnostic **`cargo test -p mac_stats --no-fail-fast`** for “already in **`src-tauri/`**” must **not** be prefixed with **`cd src-tauri &&`** (that directory only exists **from repo root**). Same rule as steps **2–3**.

**Fix for retest:** Use **`UNTESTED-…`** from the branch you test; run steps **1–4** verbatim (see **Tester TL;DR** and the symptom table); ignore unfiltered suite results for pass/fail.

### Pass/fail scope (read first)

- **Required for a pass on this task:** steps **1–4** below (static review, compile, **targeted lib tests**, spot-check). All required commands must succeed.
- **Not a required step:** There is **no** step in this document that runs **`cargo test --no-fail-fast`** or **`cargo test -p mac_stats --no-fail-fast`** **without** the **`cdp_retry_`** filter. If your runbook prints such a line as **mandatory** for this slug, treat that line as **obsolete** for pass/fail. You may run an unfiltered suite for diagnostics, but **do not** fail or emit **TESTPLAN-** solely because that optional command failed while step **3** passed.
- **Authoritative gate:** steps **1–4** override any older checklist that treated **unfiltered** crate tests as mandatory for this task. Failures in unrelated modules **do not** invalidate a pass when steps **1–4** succeed.
- **Checklist conflict:** Never rename to **TESTPLAN-** solely because an unfiltered `cargo test` failed while the **`cdp_retry_`** command in step **3** passed.
- **Not required:** a full **`cargo test -p mac_stats --no-fail-fast`** over the entire crate. Other tests can fail in some environments; those failures are **out of scope** unless the failing test is one of the **`cdp_retry_`** tests in step 3 or clearly implicates `evaluate_one_plus_one_blocking_timeout` / `check_browser_alive` / `clear_browser_session_on_error`.

### Wrong command vs gate (quick reference)

| Situation | Pass/fail for this task? |
|-----------|---------------------------|
| `cargo test … -p mac_stats --lib cdp_retry_ --no-fail-fast` — all match tests pass | **Required** — this is step **3** |
| `cargo test … -p mac_stats --no-fail-fast` (no `cdp_retry_`, not limited to those tests) | **Informational only** — **do not** fail the task or emit **TESTPLAN-** based solely on this |
| `cd src-tauri && cargo test --no-fail-fast` (historical shorthand in old reports) | Same as row above — **not** the gate |
| Step **3** passes; optional full suite fails in `discord::`, `scheduler::`, etc. | **Pass** steps **1–4**; note unrelated failures separately in the report |
| `cargo test … -p mac_stats --lib --no-fail-fast` (**no** `cdp_retry_` token) | **Not** the gate — runs **all** lib tests (~800+); failures there do **not** override a passing step **3** |

### Report evidence (include in tester notes)

- The **full command line** for step **3**, which **must** contain the substring **`cdp_retry_`** (e.g. `cargo test … --lib cdp_retry_ --no-fail-fast`).
- Cargo output showing **`running N tests`** with **N ≥ 1** and **`test result: ok`** for that command.
- If you also ran an unfiltered suite, label it **“optional / diagnostic”** and do **not** use it as the pass/fail bar.

### Copy-paste — full gate (from repository root)

Run this block **as-is** in **bash** (or `bash -lc '…'` from **fish**) after the sanity check. It covers **numbered steps 1–3** below (static review, compile, targeted tests). **Numbered step 4** is **only** the manual spot-check in the editor — it is **not** in this script.

```bash
set -e
set -o pipefail
test -f src-tauri/Cargo.toml || { echo >&2 "ERROR: run from mac-stats repo root (missing src-tauri/Cargo.toml)."; exit 1; }
test -f src-tauri/src/browser_agent/mod.rs || { echo >&2 "ERROR: missing src-tauri/src/browser_agent/mod.rs"; exit 1; }
rg 'evaluate_one_plus_one_blocking_timeout|check_browser_alive|BROWSER_CDP_HEALTH_CHECK_TIMEOUT|clear_browser_session_on_error' src-tauri/src/browser_agent/mod.rs
rg -n -m 20 'Never use.*Handle::block_on|recv_timeout\(BROWSER_CDP_HEALTH_CHECK_TIMEOUT\)' src-tauri/src/browser_agent/mod.rs
cargo check --manifest-path src-tauri/Cargo.toml -p mac_stats
echo ">>> STEP 3 GATE: lib tests filtered by cdp_retry_ only (NOT the full crate suite)"
cargo test --manifest-path src-tauri/Cargo.toml -p mac_stats --lib cdp_retry_ --no-fail-fast
```

**Why `--manifest-path`:** the crate manifest is only under `src-tauri/`. Using `--manifest-path src-tauri/Cargo.toml` lets **`cargo check` / `cargo test` succeed from repo root** without a subshell `cd`, which avoids accidental “0 tests” runs (e.g. invoking `cargo test` from root without a manifest, or from a directory that resolves to the wrong package).

**Alternate — pick one:**

- From **repo root**, you may run: `cd src-tauri && cargo check -p mac_stats && cargo test -p mac_stats --lib cdp_retry_ --no-fail-fast` (single subshell `cd` from root is OK).
- If your shell is **already** **`…/mac-stats/src-tauri`**, run **without** `cd src-tauri`: `cargo check -p mac_stats` then `cargo test -p mac_stats --lib cdp_retry_ --no-fail-fast`.

### 1) Static review (required)

From repo root, with **pipefail** enabled (see **Environment**), run:

```bash
set -o pipefail
rg 'evaluate_one_plus_one_blocking_timeout|check_browser_alive|BROWSER_CDP_HEALTH_CHECK_TIMEOUT|clear_browser_session_on_error' src-tauri/src/browser_agent/mod.rs
rg -n -m 20 'Never use.*Handle::block_on|recv_timeout\(BROWSER_CDP_HEALTH_CHECK_TIMEOUT\)' src-tauri/src/browser_agent/mod.rs
```

**Pass:** the first command exits **0** and prints **non-empty** lines from `browser_agent/mod.rs`. If it prints nothing or exits non-zero, **fail** (wrong cwd, wrong path, or symbols removed). The second command exits **0** and prints **at least one** line: the in-file comment that forbids nested **`Handle::block_on`** (matches `Never use` … `Handle::block_on`) **and/or** the `recv_timeout(BROWSER_CDP_HEALTH_CHECK_TIMEOUT)` call in `evaluate_one_plus_one_blocking_timeout`. If you need more context, re-run the second command with a larger **`-m`** value or omit **`-m`** (may print many lines).

**From `src-tauri/` cwd:** use `src/browser_agent/mod.rs` as the path for both **`rg`** commands (not `src-tauri/src/...`).

### 2) Compile (required)

**Preferred (repo root):**

```bash
cargo check --manifest-path src-tauri/Cargo.toml -p mac_stats
```

**Alternate — from repo root (one-shot):**

```bash
cd src-tauri && cargo check -p mac_stats
```

**Alternate — cwd already `src-tauri/`:** `cargo check -p mac_stats` (do **not** prefix `cd src-tauri`).

Explicit **`-p mac_stats`** avoids ambiguity if other manifests appear in the tree.

### 3) Targeted unit tests (required)

Use **only** this command for the automated test gate (`--lib` limits scope to library unit tests). The filter is a **substring** of the test name: **`cdp_retry_`** (trailing underscore).

**Preferred (repo root):**

```bash
cargo test --manifest-path src-tauri/Cargo.toml -p mac_stats --lib cdp_retry_ --no-fail-fast
```

**Alternate — from repo root (one-shot):**

```bash
cd src-tauri && cargo test -p mac_stats --lib cdp_retry_ --no-fail-fast
```

**Alternate — cwd already `src-tauri/`:** `cargo test -p mac_stats --lib cdp_retry_ --no-fail-fast` (do **not** prefix `cd src-tauri`).

**Pass:** Cargo **runs** the `cdp_retry_`-filtered lib tests and reports a **non-zero** count. Expect **at least 2** tests (current names below). If Cargo reports **0** tests executed, **fail** this step and verify: (1) cwd + invocation match one of the two forms above (from root you **must** use `--manifest-path src-tauri/Cargo.toml`; there is no `Cargo.toml` at repo root), (2) the filter is exactly `cdp_retry_`, not `cdp_retry`, (3) the package is `-p mac_stats`. Optional diagnostic from repo root:

```bash
cargo test --manifest-path src-tauri/Cargo.toml -p mac_stats --lib cdp_retry_ -- --list
```

You should see lines containing `cdp_retry_` before re-running the full test command.

- `browser_agent::tests::cdp_retry_skipped_when_health_error_also_looks_like_connection_error`
- `browser_agent::tests::cdp_retry_allowed_for_plain_connection_error_without_health_prefix`

**All** executed tests must pass. If additional tests share the `cdp_retry_` name prefix, they are in scope too — **every** test run by this exact command must pass.

**Copy/paste note:** the filter is the substring `cdp_retry_` (underscore at the end), not `cdp_retry` alone.

**Expected shape (illustrative):** Cargo should print a line like `running 2 tests`, then `test result: ok. 2 passed; …` and a **large** `N filtered out` count (hundreds) is normal — that means only the `cdp_retry_` tests ran. **Do not** require `N filtered out` to match a specific number across checkouts.

### 4) Spot-check — acceptance criterion 3 (required)

In `src-tauri/src/browser_agent/mod.rs`, locate `should_retry_cdp_after_clearing_session` (search by name). Confirm the comment/doc states that **health-check / “Browser unresponsive”** handling already clears the session and must **not** be treated like the generic “retry after connection error” path.

### Optional — full crate tests (informational only)

**From repo root:**

```bash
cargo test --manifest-path src-tauri/Cargo.toml -p mac_stats --no-fail-fast
```

**From `src-tauri/`** (cwd is already `…/mac-stats/src-tauri`; **do not** `cd src-tauri`):

```bash
cargo test -p mac_stats --no-fail-fast
```

This is **diagnostic** only. **Do not** use this command as the **required** pass/fail bar for this task. **Do not** list it in your report as a failed **acceptance** step when steps **1–4** passed.

If it fails, **do not** fail the task **unless** the failure is in the **`cdp_retry_`** tests from step 3 or clearly tied to the health-check symbols in step 1. Otherwise record it as **environment / unrelated suite** in your report, not as a regression for this task.

## Canonical history

Cumulative tester reports and older verification notes: **`tasks/CLOSED-20260321-1345-browser-use-cdp-health-check-ping.md`** (reference only).

## Publication workflow (TESTPLAN → UNTESTED)

While instructions are edited, the task may temporarily live as **`TESTPLAN-20260321-1345-browser-use-cdp-health-check-ping.md`**. When ready for **`003-tester`**, the coder renames **`TESTPLAN-` → `UNTESTED-`** (same stamp and slug). After that rename, **`tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`** is the only executable queue filename for this slug (see **Operator filename** above).

If the branch already contains **`UNTESTED-…`** (no **`TESTPLAN-…`** file), a coder may **edit `UNTESTED-…` in place** to fix instructions; that is equivalent to publishing a fresh **`UNTESTED-`** after a **`TESTPLAN-`** handoff, without an extra rename step.
