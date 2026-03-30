# Browser use — CDP health check ping (`1+1`)

> **Read your basename first.** The **only** rename into **`TESTING-`** that starts a valid run for this slug is **`UNTESTED-` → `TESTING-`**. Renaming **`TESTPLAN-` → `TESTING-`** or **`CLOSED-` → `TESTING-`** because the queue slot “was missing” is **invalid** for this task and produced false **TESTPLAN-** outcomes in archived reports (operators treated **`TESTER.md`**’s unfiltered **`cargo test`** as the bar).
>
> **Filename on disk (read the basename):**
> - **`TESTPLAN-20260321-1345-browser-use-cdp-health-check-ping.md`** — **under revision**. Testers **must not** run the gate or rename **`TESTPLAN-` → `TESTING-`**. Wait for the coder to rename **`TESTPLAN-` → `UNTESTED-`** (same stamp and slug).
> - **`UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`** — **executable queue file**. Testers rename **`UNTESTED-` → `TESTING-`** to execute per **`TESTER.md`**.
>
> Prior **`TESTPLAN-`** outcomes reflected **defective testing instructions / operator environment** (wrong bar, wrong cwd, wrong queue filename, stale translated runbooks), **not** a mac-stats CDP health-check implementation failure.
>
> **Scope:** **mac-stats** repository only. All paths refer to **`src-tauri/src/browser_agent/mod.rs`** in that clone. Running these steps in a **different repo** (e.g. a sibling **openclaw** tree) is **out of scope** and invalid — use the mac-stats checkout that contains this `tasks/` file.
>
> **Testing instructions** revised **2026-03-30** (l): **(k)** plus **zero-ambiguity gate** (minimal pass list + explicit “never the gate” one-liner at top of **Testing instructions**); **wrong-command → procedure error** table (symptom = you ran the wrong `cargo test`); **TESTPLAN vs UNTESTED** reminder on this basename while under coder edit.

## Goal

Before CDP browser tools run, mac-stats must detect a hung or dead Chrome while the WebSocket may still look open: optional child-PID liveness (`kill -0` on Unix), then a lightweight **`Runtime.evaluate("1+1")`** “ping” with a **hard wall-clock timeout** on a **plain `std::thread`** + `mpsc::recv_timeout`. This path must **never** nest Tokio `Handle::block_on` + `tokio::time::timeout` on the app’s shared runtime (current-thread executor would wedge).

## Acceptance criteria

1. `evaluate_one_plus_one_blocking_timeout` runs `tab.evaluate("1+1", false)` on a worker thread and uses `recv_timeout(BROWSER_CDP_HEALTH_CHECK_TIMEOUT)`; errors surface as **Browser unresponsive** messages where applicable.
2. `check_browser_alive` calls that helper and includes an explicit comment forbidding nested `block_on` + Tokio timeout (heartbeat / scheduler rationale).
3. On health-check failure, `clear_browser_session_on_error` clears the cached session for **Browser unresponsive** and for connection-style errors (`is_connection_error`), without conflating with unrelated retry paths (`should_retry_cdp_after_clearing_session` documents health wins over retry).

## Operator filename (TESTER.md / 003-tester)

The **Phase 0** table is the short decision guide; the bullets below spell out the same rules for runbook authors.

- **If the task file on disk is `TESTPLAN-20260321-1345-browser-use-cdp-health-check-ping.md`:** instructions are **under revision**. Testers **must not** rename **`TESTPLAN-` → `TESTING-`** or execute the gate until a coder publishes **`UNTESTED-…`** (same stamp and slug).
- **Executable queue file (only this name for a real run):** `tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`. At the start of a normal test run, rename **`UNTESTED-` → `TESTING-`**, execute the steps below, then follow your runbook for **`TESTING-` → `CLOSED-`** or failure naming.
- **Handoff defect — do not substitute other prefixes for this slug:** If **`UNTESTED-…` is missing** at repo tip, **stop**. Do **not** rename `CLOSED-…` or `TESTPLAN-…` to `TESTING-…` to “run” this task unless your operator guide **explicitly** authorizes that exception. **`TESTPLAN-` → `TESTING-`** is **never** the correct workaround for a missing **`UNTESTED-…`** file — that skips coder publication and invites the wrong **`cargo test`** bar. Typical fix: pull/sync to a revision where the coder published **`UNTESTED-…`** (after **`TESTPLAN-` → `UNTESTED-`**), or file a handoff defect. Treating historical **`CLOSED-…`** as the live test target caused false **TESTPLAN-** outcomes (full-suite noise) and bypassed the narrow gate below.
- If **`tasks/TESTPLAN-20260321-1345-browser-use-cdp-health-check-ping.md` exists** (and **`UNTESTED-…` is absent**): testing instructions are **under revision**. Do **not** rename `TESTPLAN-` → `TESTING-` for execution; wait for **`TESTPLAN-` → `UNTESTED-`**, then pick up **`UNTESTED-…`**.
- **`TESTER.md` / templates** that still mandate an **unfiltered** `cargo test` / `cargo test --no-fail-fast` / `cargo test -p mac_stats --no-fail-fast` with **no** `cdp_retry_` filter for this slug are **stale for pass/fail**. **Closure** for this slug depends **only** on **Steps 1–4** (gate) in **Testing instructions** (see **Pass/fail scope**).
- **Outcome:** If **Steps 1–4** all pass, the task **passes** for this slug even when an **optional** unfiltered `cargo test -p mac_stats --no-fail-fast` would fail (Discord `pdfs_dir`, scheduler home-directory coupling, etc.). Use **TESTPLAN-** only when **testing instructions** or **queue filenames/handoff** are wrong — **not** when the narrow gate passes but the broad suite fails.

## Phase 0 — Which file are you holding? (before anything else)

| Filename on disk | What you do |
|------------------|-------------|
| **`TESTPLAN-20260321-1345-browser-use-cdp-health-check-ping.md`** | **Stop.** Instructions are **under coder revision** or this copy is **not** yet published to the tester queue. **Do not** rename **`TESTPLAN-` → `TESTING-`** (that is **not** equivalent to missing **`UNTESTED-…`** — record **handoff / instruction** state, do **not** execute the gate from this basename). Wait until the repo contains **`UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`** (same stamp/slug), then use that file per your runbook. |
| **`UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`** | This is the **executable** queue name. Rename **`UNTESTED-` → `TESTING-`** per **`TESTER.md`**, then run **Preflight** + **Steps 1–4** below. |
| **`CLOSED-20260321-1345-browser-use-cdp-health-check-ping.md` and no `UNTESTED-…` for this slug** | **`CLOSED-`** is **archive** (reports + history), not the executable queue name for a **new** pass. **Stop** — do not rename **`CLOSED-` → `TESTING-`** to simulate **`UNTESTED-` → `TESTING-`** unless your operator guide **explicitly** allows that exception. Pull/sync to a revision that contains **`UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`**, or file a handoff defect. |
| **`TESTING-20260321-1345-browser-use-cdp-health-check-ping.md`** | Run in progress per **`TESTER.md`**. |

**Common defect:** Running the gate while the task still exists only as **`TESTPLAN-…`**, or running **`cargo test`** from the **wrong repository**, then filing **`TESTPLAN-`** against **mac-stats** code. **Phase 0** prevents both.

## Environment

- **Host:** **macOS** with a normal mac-stats dev toolchain (`cargo`, `rustc` on `PATH`). This crate is a **macOS** app; if you are **not** on macOS and **`cargo check -p mac_stats`** fails for linker or Apple-framework reasons, **stop** — record **environment blocked** in your notes. That is **not** a defect in this task’s **Testing instructions** and is **not** grounds for **`TESTPLAN-…`** by itself.
- **Checkout:** mac-stats repository root (directory that contains `src-tauri/`). This repo has **no** workspace `Cargo.toml` at the repository root — the package lives under **`src-tauri/`** only.
- **IDE / terminal cwd:** Many editors open a terminal in `tasks/`, `src/`, or `src-tauri/` by default. **That is not the repo root.** If `test -f src-tauri/Cargo.toml` fails, `cd` up to the directory that **contains** `src-tauri/` (run `pwd` after `cd` so your report states the actual cwd).
- **Sanity check (once per session):** from repo root, `test -f src-tauri/Cargo.toml && test -f src-tauri/src/browser_agent/mod.rs` must succeed before the gate. If this fails, you are in the wrong directory (e.g. `tasks/`, another clone, or inside `src-tauri/` with wrong relative paths for `rg`) — `cd` to the repo root first.
- **From `src-tauri/` instead:** if your shell cwd is already **`…/mac-stats/src-tauri`**, use **`test -f Cargo.toml && test -f src/browser_agent/mod.rs`** before running **alternate** commands. Do **not** run `cd src-tauri` when you are already inside `src-tauri/` (that `cd` fails). For **`rg`**, either `cd` back to repo root and keep paths `src-tauri/src/...`, or run `rg … src/browser_agent/mod.rs` from `src-tauri/` (no `src-tauri/` prefix).
- **Hard footgun — no manifest at repo root:** this tree has **no** `Cargo.toml` at the repository root. Running plain `cargo check` or `cargo test` **without** `--manifest-path src-tauri/Cargo.toml` from root typically errors (e.g. *could not find `Cargo.toml`* or picks up the wrong workspace). That is **not** a product failure — fix the invocation.
- **Preferred cwd for the gate:** stay at **repository root** for the whole copy-paste block. Use **`cargo … --manifest-path src-tauri/Cargo.toml`** so you never depend on `cd src-tauri` (a common source of “0 tests” when someone runs `cargo test` from root without a manifest).
- **Alternate cwd:** you may instead `cd` to **`src-tauri/`** once (from repo root) and run `cargo check -p mac_stats` / `cargo test -p mac_stats --lib cdp_retry_ --no-fail-fast` with **no** `--manifest-path`. If you do, stay in `src-tauri/` for those **cargo** commands; use the path rules above for **`rg`** (either return to repo root for `src-tauri/src/browser_agent/mod.rs`, or use `src/browser_agent/mod.rs` from inside `src-tauri/`).
- **Already in `src-tauri/`:** run **`cargo … -p mac_stats`** with **no** leading `cd src-tauri` (that path does not exist inside `src-tauri/`). Same for **`rg`**: paths are **`src/browser_agent/mod.rs`**, not `src-tauri/src/...`.
- **Shell / pipelines:** The copy-paste block is written for **bash**. **zsh** on macOS also supports `set -o pipefail` (or use `setopt pipefail`). **fish** has different pipeline semantics — run the block via `bash -lc '…'` (use **single quotes** around the whole script so fish does not expand `$(pwd)` / `$()` before bash sees it) or paste the block into a `bash` subshell. Static review uses **`rg -m 20`** (no `| head`) so **`set -o pipefail` does not** turn a successful search into a pipeline failure (**SIGPIPE** when `head` stops reading).
- **`rg`:** ripgrep must be available on `PATH` (same as other task files).
- **No live Chrome / CDP session** is required for **Steps 1–4** (static review, compile, lib unit tests, spot-check).
- **Rust:** stable toolchain able to build `mac_stats` (same as normal mac-stats development).

## Testing instructions

### Zero-ambiguity gate (read this block first)

**While this file is named `TESTPLAN-…` on disk:** instructions are **under coder revision**. **Do not** run the gate, **do not** rename **`TESTPLAN-` → `TESTING-`**. Wait until the repo contains **`UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`** (same stamp/slug), then execute from that file.

**After `TESTPLAN-` → `UNTESTED-` publication, a pass requires all of:**

1. **Queue:** You started from **`tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`** and renamed **`UNTESTED-` → `TESTING-`** per **`TESTER.md`** (not from **`CLOSED-…`** or **`TESTPLAN-…`** stand-ins).
2. **Repo + cwd:** **mac-stats** tree; `test -f src-tauri/Cargo.toml && test -f src-tauri/src/browser_agent/mod.rs` succeeds from **repository root** (or you used the **`src-tauri/`** column in **Paths by cwd** consistently).
3. **Automated gate:** Run the bash block **Copy-paste — full gate** (covers **Steps 1–3**) exactly as written for your cwd column, then **Step 4** in the editor (manual — **not** in the script).
4. **Step 3 shape:** The `cargo test` you treat as mandatory includes **`--lib`**, the literal filter token **`cdp_retry_`** (underscore **required**), and **`--no-fail-fast`**, with **`cdp_retry_` before any `--`** that starts test-binary args. Cargo must print **`running N tests`** with **N ≥ 1** and **`test result: ok`** for **that** command.

**Never the pass/fail bar:** any single `cargo test` invocation whose command line does **not** contain the substring **`cdp_retry_`**. That includes unfiltered **`cargo test --no-fail-fast`**, **`cargo test -p mac_stats --no-fail-fast`**, **`cargo test -p mac_stats --lib --no-fail-fast`**, and **`cd src-tauri && cargo test --no-fail-fast`** — run those only under **Optional — full crate tests**, and **never** as the reason for **fail** or **`TESTPLAN-`** if the **`cdp_retry_`** Step 3 above **passed**.

**Wrong command ⇒ procedure error (not a mac-stats defect):** If the **only** failing command in your notes is a `cargo test` **without** **`cdp_retry_`**, you did **not** complete the gate — re-run **Step 3** from **Authoritative automated test command** or **Copy-paste — full gate**, then decide pass/fail.

---

**One-line gate recap (memorize this):** From **mac-stats repo root**, run the **Copy-paste — full gate** bash block (Steps **1–3**), then **Step 4** in the editor. **Step 3** is **only** `cargo test … -p mac_stats --lib cdp_retry_ --no-fail-fast` (literal substring **`cdp_retry_`** on that line). Anything else is optional or diagnostic.

**Single-sentence gate rule:** This task **never** requires `cargo test` / `cargo test --no-fail-fast` **without** the substring **`cdp_retry_`** on the **same** command line. If **TESTER.md** (or a translated runbook) lists only `cd src-tauri && cargo test --no-fail-fast` as the final verification line, that line is **not** the acceptance bar for this slug — **Step 3** below replaces it. A failing unfiltered suite **does not** override a passing **Step 3**.

**Before you name an outcome (tri-check — all must be true for pass):**

1. **Step 3 shape:** The `cargo test` line you treat as mandatory includes **`--lib`**, **`cdp_retry_`**, and **`--no-fail-fast`**, with **`cdp_retry_` before any `--`** (test-binary args).
2. **Step 3 evidence:** Cargo printed **`running N tests`** with **N ≥ 1** and **`test result: ok`** for that exact command.
3. **No false TESTPLAN:** You are **not** failing the task or emitting **TESTPLAN-** solely because an **unfiltered** `cargo test --no-fail-fast` failed while **Step 3** passed.

**Closure bar — pass/fail for this slug depends only on this table:**

**Single automated test bar (non‑negotiable):** The **only** `cargo test` command that can **fail** this task is **Step 3** — a **library** run whose command line contains **both** `--lib` and the substring **`cdp_retry_`** (as Cargo’s test-name filter, **before** any `--`). If you ran **`cd src-tauri && cargo test --no-fail-fast`**, **`cargo test -p mac_stats --no-fail-fast`**, or **`cargo test -p mac_stats --lib --no-fail-fast`** **without** the token **`cdp_retry_`** on that same line, you did **not** run Step 3; any failure there is **out of scope** for pass/fail (run Step 3 from **Authoritative automated test command** or the copy-paste block).

**TESTER.md / translated checklists:** If your operator template lists **`cargo test --no-fail-fast`** (or **`cd src-tauri && cargo test …`**) **without** **`cdp_retry_`** as a **mandatory** line for this slug, that line is **obsolete for acceptance** — do **not** treat it as “the task verification block.” Replace it mentally (and in your notes) with **Step 3** below. Filing **`TESTPLAN-`** because that unfiltered command failed **while Step 3 passed** is a **procedure error**, not an instruction defect.

| Required for **pass** | **Never** the pass/fail bar by itself |
|------------------------|----------------------------------------|
| **Steps 1–4** below (including **Step 3** with **`--lib`** and literal **`cdp_retry_`** on the **same** `cargo test` line) | **`cargo test` / `cargo test --no-fail-fast` / `cd src-tauri && cargo test --no-fail-fast`** with **no** **`--lib cdp_retry_`** gate shape on that same command |
| Queue file **`tasks/UNTESTED-…`** renamed **`UNTESTED-` → `TESTING-`** before you start | Renaming **`CLOSED-…`** or **`TESTPLAN-…`** → **`TESTING-…`** when **`UNTESTED-…`** is missing |
| Evidence: **Step 3** output shows **`running N tests`** with **N ≥ 1** and **`test result: ok`** | Failing **optional** full-crate tests (Discord `pdfs_dir`, scheduler home fixtures, etc.) while **Step 3** passed |

**Retest preflight (do these before any `cargo` or `rg`):**

1. **Phase 0:** You are holding **`UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`** (after coder publication), **not** only **`CLOSED-…`**. If you only see **`TESTPLAN-…`** for this slug, **stop** — wait for **`TESTPLAN-` → `UNTESTED-`**.
2. **Repo:** **`test -f src-tauri/Cargo.toml`** from **mac-stats** root (see **PF-1**; use **PF-0** if needed).
3. **Gate command shape:** **Step 3** is **`… --lib cdp_retry_ --no-fail-fast`** — both **`--lib`** and the substring **`cdp_retry_`** are mandatory on the **same** command line.
4. **Stale runbooks:** Ignore mandatory lines that run **unfiltered** `cargo test` / `cargo test --no-fail-fast` for this slug; see **Stale runbook override**.

**Authority / language:** Pass/fail for this slug is defined **only** by this task file (English). **TESTER.md**, localized templates, or Discord/summary text that contradict **Steps 1–4** here are **stale for acceptance** — follow **this** file, not a translated or abbreviated checklist. If a summary claimed this task “requires” unfiltered **`cargo test --no-fail-fast`** as the **mandatory** verification, that summary is **wrong for this slug** — **Step 3** has always been the **`cdp_retry_`**-filtered **`--lib`** command in **this** document (see **Authoritative automated test command**).

**Why the last cycle was `TESTPLAN-` (instruction defect):** Testers followed a **stale** bar (**unfiltered** `cargo test --no-fail-fast` — sometimes copied from **TESTER.md** or an older task body — and misread it as “the mandatory verification block”) and/or ran from a tree where **`UNTESTED-…`** was absent and used **`CLOSED-…`** / **`TESTPLAN-…`** as a stand-in. Neither matches **this** document. **Implementation** of the CDP health-check was **not** implicated.

**If `TESTER.md` still lists an unfiltered crate test as a mandatory step for this slug:** treat that line as **obsolete for pass/fail** — run **Step 3** (`cdp_retry_`) from **this** file instead; you may run the full suite **only** under **Optional — full crate tests** and **must not** fail the task based solely on that optional result.

**Naming rule:** **Preflight** items are **`PF-0` … `PF-4`** (`PF-0` optional walk-up; **`PF-1` … `PF-4`** otherwise). **Gate** items are **`Step 1` … `Step 4`** (static review, compile, filtered tests, manual spot-check). **Do not** confuse **PF-3** (fingerprint / self-check — read the table **before** you type Step 3) with **Step 3** (the **actual** `cargo test` command you run in the shell — **PF-3** is not a substitute command).

### Start here (preflight — before Step 1)

**PF-0 — Optional: find repo root from a nested cwd** (use when **PF-1** fails and you know you are **somewhere inside** the mac-stats clone — e.g. terminal opened in `tasks/`, `src/`, or `src-tauri/src/…`):

```bash
while [ "$(pwd)" != "/" ] && [ ! -f src-tauri/Cargo.toml ]; do cd .. || exit 1; done
test -f src-tauri/Cargo.toml && test -f src-tauri/src/browser_agent/mod.rs && echo "OK: mac-stats repo root" || { echo >&2 "ERROR: could not find mac-stats root (no src-tauri/Cargo.toml above this path)."; exit 1; }
pwd
```

If this still fails, you are outside the clone, in the wrong repository, or the checkout is incomplete — **do not** invent a pass; fix the checkout or `cd` to the correct tree.

**PF-1 — Confirm repo root** (must print `OK: mac-stats repo root`):

```bash
pwd
test -f src-tauri/Cargo.toml && test -f src-tauri/src/browser_agent/mod.rs && echo "OK: mac-stats repo root"
```

If that fails, you are not at the **mac-stats** repository root (common: terminal started in `tasks/`, `src/`, or only inside `src-tauri/`). Run **PF-0** above, or `cd` manually to the clone root — the folder whose **direct child** is `src-tauri/`.

**PF-2 — Queue file for a real run:** `tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md` must exist at repo tip (rename **`UNTESTED-` → `TESTING-`** per runbook). If only **`TESTPLAN-…`** exists, **stop** — instructions under revision; wait for **`TESTPLAN-` → `UNTESTED-`**. If only **`CLOSED-…`** exists for this slug (no **`UNTESTED-…`**), **stop** — see **Phase 0**; do not fake the queue with **`CLOSED-` → `TESTING-`**.

**PF-3 — Step 3 fingerprint (read before Step 3):** the automated gate is **not** “any” `cargo test`. Your **Step 3** command line **must** include the substring **`cdp_retry_`** as Cargo’s **test-name filter** (a **positional** argument after `--lib`, **before** any `--` that starts test-binary args). Self-check after typing the command: you should see the three tokens **`--lib`**, **`cdp_retry_`**, and **`--no-fail-fast`** in that order (with only other `cargo` flags between them as in the copy-paste block).

| You ran | Gate? |
|--------|--------|
| `cargo test … -p mac_stats --lib cdp_retry_ --no-fail-fast` | **Yes** — this is **Step 3** |
| `cargo test … -p mac_stats --lib --no-fail-fast` | **No** — runs **all** lib tests (~800+); **never** the acceptance bar |
| `cargo test … -p mac_stats --no-fail-fast` (no `cdp_retry_`) | **No** |
| `cd src-tauri && cargo test --no-fail-fast` (no `cdp_retry_`) | **No** — stale runbook |

**PF-4 — Order:** run **Copy-paste — full gate** (implements **Steps 1–3**), then **Step 4** in the editor. **Do not** substitute an unfiltered full-suite `cargo test` from an old checklist and treat its failure as this task failing.

### Stale runbook override (read before any `cargo test`)

Some **`TESTER.md`** (or operator) checklists still print **only**:

- `cd src-tauri && cargo check`, then  
- `cd src-tauri && cargo test --no-fail-fast` **with no test-name filter**

**That pair is not the pass/fail bar for this slug.** It runs the **entire** crate test surface; failures in `discord::`, `scheduler::`, home-directory fixtures, etc. are **expected in some environments** and **do not** indicate a CDP health-check regression.

**Authoritative bar (closure):** **Steps 1–4** (gate) in **this** document — not **PF-1…PF-4** alone. In particular, **Step 3** **must** be a **`cargo test`** invocation whose command line contains the literal substring **`cdp_retry_`** (see **Authoritative automated test command** and **Copy-paste — full gate**).

**Wrong outcomes to avoid:**

- Emitting **`TESTPLAN-`** or **fail** because an **unfiltered** `cargo test --no-fail-fast` failed **while** **Step 3** (`cdp_retry_` filter) **passed** — that is a **procedure error**, not an instruction defect.
- Renaming **`CLOSED-…`** or **`TESTPLAN-…`** → **`TESTING-…`** when **`tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`** is missing — use the **`UNTESTED-…`** queue file from the branch under test, or record a **handoff defect** (see **Operator filename**).

### Executive summary (default run)

Use this unless you have a documented reason to use the **`src-tauri/`** cwd column in **Paths by cwd**.

| Step | What you do | Pass hint |
|------|-------------|-----------|
| **1** | Static review: two **`rg`** lines (see **Copy-paste — full gate** or **1) Static review**) | Non-empty matches from `browser_agent/mod.rs`; second **`rg`** shows the “no **`Handle::block_on`** …” comment and/or **`recv_timeout(BROWSER_CDP_HEALTH_CHECK_TIMEOUT)`** |
| **2** | **`cargo check … -p mac_stats`** (with **`--manifest-path`** from repo root) | Exits **0** |
| **3** | **`cargo test … --lib cdp_retry_ --no-fail-fast`** — command line **must** contain the literal substring **`cdp_retry_`** | **`running N tests`** with **N ≥ 1** and **`test result: ok`** |
| **4** | Manual: open **`src-tauri/src/browser_agent/mod.rs`**, find **`should_retry_cdp_after_clearing_session`**, confirm health / “Browser unresponsive” vs generic retry semantics | Matches acceptance criterion **3** in this file |

**Not a step (ever):** `cargo test … --no-fail-fast` or `cd src-tauri && cargo test --no-fail-fast` **without** **`cdp_retry_`** on the same command line — run only under **Optional — full crate tests**, and **never** as the reason for **`TESTPLAN-`** if **Step 3** passed.

The bash block under **Copy-paste — full gate** runs **Steps 1–3** (gate) only. **Step 4** is **never** in that script.

**Filter semantics (Step 3):** **`cdp_retry_`** is Cargo’s **test-name filter** (substring). Only tests whose names match run. **`--no-fail-fast`** affects whether Cargo stops after the first failure **among those matched tests**; it does **not** remove the filter or run the whole **`--lib`** suite.

**Token position (Step 3):** **`cdp_retry_`** is a **Cargo** positional filter. It must appear **before** any **`--`** that starts **test-binary** arguments. Wrong: `cargo test … --lib -- cdp_retry_` or `cargo test … -- --list cdp_retry_` — the gate will **not** run the intended filter. Right: `cargo test … --lib cdp_retry_ --no-fail-fast` (and for listing: `cargo test … --lib cdp_retry_ -- --list`).

### Before Step 1 (queue file + tools)

- **Queue filename:** You must be executing from **`tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`** (rename **`UNTESTED-` → `TESTING-`** per operator runbook). If the only file on disk for this slug is **`TESTPLAN-…`**, instructions are still under revision — do **not** run this gate; wait for **`TESTPLAN-` → `UNTESTED-`** (see **Phase 0**).
- **`rg` and `cargo`:** `command -v rg` and `command -v cargo` must succeed on `PATH` before **Steps 1–3**.
- **`set -e` + `rg`:** In the **Copy-paste — full gate** block, **`set -e`** is intentional. **`rg` exits 1 when there are zero matches**; the script then stops at that line. That is **not** a spurious failure — fix **cwd**, **file path** (see **Paths by cwd**), or restore missing symbols. Do **not** “fix” the script by removing **`set -e`** or piping **`rg … \| true`** for the required static review.

### Paths by cwd (pick one column; regexes = copy-paste block only)

Markdown tables cannot safely embed `rg` alternation patterns (`a|b|c`). **Do not copy `rg` regexes from any table that used escaped pipes** — they are wrong in the shell.

Use **one** column end-to-end for **Steps 1–3** (gate). The **authoritative** `rg` command lines are **only** the two `rg` lines in **Copy-paste — full gate** (or **1) Static review** below); substitute **only** the file path:

| Your shell `cwd` | Use this path as the **last argument** to both `rg` commands |
|------------------|--------------------------------------------------------------|
| **Repository root** (directory that contains `src-tauri/`) | `src-tauri/src/browser_agent/mod.rs` |
| **Already inside** `…/mac-stats/src-tauri/` | `src/browser_agent/mod.rs` |

**Cargo** (same column as above):

| Step | **Repository root** | **Already inside** `src-tauri/` |
|------|----------------------|----------------------------------|
| **2 — `cargo check`** | `cargo check --manifest-path src-tauri/Cargo.toml -p mac_stats` | `cargo check -p mac_stats` (**no** `cd src-tauri`) |
| **3 — gate `cargo test`** | `cargo test --manifest-path src-tauri/Cargo.toml -p mac_stats --lib cdp_retry_ --no-fail-fast` | `cargo test -p mac_stats --lib cdp_retry_ --no-fail-fast` (**no** `cd src-tauri`) |

If you mix columns (e.g. root paths while cwd is `src-tauri/`), you get empty **`rg`** output or **`running 0 tests`** — that is an **execution** mistake, not a product failure.

### Tester TL;DR (pass/fail)

1. **Phase 0:** **`UNTESTED-…`** on disk before you start; **never** run the gate from **`TESTPLAN-…`**.
2. **Cwd:** repo root **or** `src-tauri/` — follow **Environment** (including “already in `src-tauri/`” and **no** root `Cargo.toml`). Confirm **mac-stats** (this repo), not another project.
3. **Gate:** **Steps 1–4** only. **Step 3** must be exactly the **`cdp_retry_`** lib test command (substring visible on the command line, and **`cdp_retry_` before any `--`**). Read **PF-3** before you run **Step 3** so you do not copy the wrong `cargo test` line.
4. **Ignore** any **`cargo test`** that does **not** include the substring **`cdp_retry_`** on the command line — including **`cargo test -p mac_stats --lib --no-fail-fast`** (runs the **entire** lib suite, hundreds of tests) and **`cargo test -p mac_stats --no-fail-fast`**. Those are **diagnostics only**, not acceptance.
5. **Step 4** is a **manual** read in the editor — it is **not** included in the bash copy-paste block.
6. **Report:** paste the **exact** **Step 3** command you ran and Cargo lines showing **`running N tests`** with **N ≥ 1** and **`test result: ok`** (see **Report evidence**).

### If something looks wrong (symptom → fix)

| Symptom | Likely cause | Fix |
|--------|----------------|-----|
| `could not find Cargo.toml` (or similar) when running **cargo** from repo root | Invoked **cargo** without `--manifest-path src-tauri/Cargo.toml` | Use the **preferred** commands from root, or `cd src-tauri` and use **alternate** form **without** `--manifest-path`. |
| `running 0 tests` for **Step 3** | Wrong cwd/manifest, wrong `-p`, filter typo (`cdp_retry` vs `cdp_retry_`), or **`cdp_retry_` placed after `--`** (test-binary args, not Cargo’s filter) | Re-run with **`cdp_retry_` before `--`**: `cargo test … --lib cdp_retry_ --no-fail-fast`. Diagnostic: `cargo test … --lib cdp_retry_ -- --list` (same manifest/cwd); expect **≥ 1** line containing **`cdp_retry_`**. |
| First **`rg`** prints nothing or exits **1** | Wrong directory or wrong path (`src/browser_agent/...` from root instead of `src-tauri/src/...`) | `pwd`; from root paths **must** start with `src-tauri/src/browser_agent/mod.rs`. |
| Second **`rg`** prints nothing | Same cwd/path issue | Same as above. |
| Second **`rg`** pipeline fails with **`set -e` / `pipefail`** though matches exist | Historically `rg \| head`; **`head`** closes the pipe → **`rg`** can exit non-zero | Use **`rg -n -m 20 '…'`** as in the copy-paste block (no ` \| head`). |
| Optional full-suite command errors with **`cd: src-tauri: No such file`** | Copied **“From `src-tauri/`”** block but shell cwd was **already** `src-tauri/` | From **`src-tauri/`**, run **`cargo test -p mac_stats --no-fail-fast`** only — **no** `cd src-tauri`. |
| **`bash -lc` from fish** runs but **`$(pwd)`** is wrong / script looks empty or errors | **fish** expanded **`$(…)`** before **`bash`** ran (double quotes or no quotes around the script) | Wrap the **entire** bash script in **single quotes**, or run **`bash`** interactively and paste the block, or save the block to a file and `bash /path/to/script.sh`. |

### Authoritative automated test command (Step 3 — paste exactly)

The **only** `cargo test` invocation that counts toward pass/fail for this task is the **library** run with the **`cdp_retry_`** name filter (substring, trailing underscore):

```bash
cargo test --manifest-path src-tauri/Cargo.toml -p mac_stats --lib cdp_retry_ --no-fail-fast
```

**From `src-tauri/`:** `cargo test -p mac_stats --lib cdp_retry_ --no-fail-fast`

Your report **must** show this filter (the literal substring `cdp_retry_`) on the test command line. If the command you ran has no `cdp_retry_` token, you have **not** executed the gate — run **Step 3** again before choosing an outcome.

### What went wrong in the last TESTPLAN cycle (instruction defect, not product)

Mistakes that produced **TESTPLAN-** while the CDP implementation was fine:

1. **Wrong test bar:** Some runs used **`cd src-tauri && cargo test --no-fail-fast`** (entire crate, **no** `cdp_retry_` filter). That command is **not** in **Steps 1–4** and is **never** the acceptance gate for this slug. Failures there (Discord `pdfs_dir`, scheduler tests touching the real home directory, etc.) are **environment / unrelated suite** noise.
2. **Wrong queue file:** **`tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`** was missing at repo tip; testers retitled **`CLOSED-…`** or **`TESTPLAN-…`** to **`TESTING-…`** instead of waiting for **`UNTESTED-…`**. That bypassed this document’s narrow gate and invited runbook confusion.
3. **Wrong cwd / manifest:** Running **cargo** from repo root **without** `--manifest-path src-tauri/Cargo.toml`, or running **`cd src-tauri`** when already inside **`src-tauri/`**, or using **`rg`** paths that omit the **`src-tauri/`** prefix from repo root — yields “no manifest”, **`cd` errors**, **0 tests**, or empty **`rg`** output. Treat as **instruction/environment execution**, not a code regression.
4. **Wrong optional-suite snippet:** The diagnostic **`cargo test -p mac_stats --no-fail-fast`** for “already in **`src-tauri/`**” must **not** be prefixed with **`cd src-tauri &&`** (that directory only exists **from repo root**). Same rule as **Steps 2–3**.
5. **Copied `rg` from a markdown table with `\|`:** alternation in **`rg`** must use a **single** pipe `|` between alternatives inside the quoted pattern. Escaped `\|` is **not** the same regex and can yield false “no matches” static-review failures.
6. **Filter after `--`:** placing **`cdp_retry_`** after **`--`** sends it to the test binary instead of Cargo’s name filter → **`running 0 tests`** or wrong listing. Keep **`cdp_retry_` before `--`** (see **Token position** under **Executive summary**).
7. **Omitted `--lib` on Step 3:** `cargo test -p mac_stats --no-fail-fast` **without** `--lib` and **without** `cdp_retry_` is **not** Step 3 — it pulls in **more than library unit tests** and is a common source of unrelated failures.
8. **Equating “task verification” with TESTER.md’s last `cargo test` line:** Some runbooks end with **`cd src-tauri && cargo test --no-fail-fast`** and **no** filter. That command is **not** Step 3 for this slug. **Pass/fail** is **only** Steps 1–4 **in this file**; do **not** emit **`TESTPLAN-`** because that unfiltered line failed if **Step 3** (`cdp_retry_`) **passed**.
9. **fish + `bash -lc`:** Passing the gate script in **double quotes** (or unquoted) lets **fish** expand **`$(pwd)`** and other **`$(…)`** before **bash** runs, so the script does not behave like the copy-paste block. Use **single-quoted** `bash -lc '…'`, a heredoc, or a small **`*.sh`** file.
10. **`TESTPLAN-` → `TESTING-`:** When **`UNTESTED-…`** was absent, some operators renamed **`TESTPLAN-…`** (or **`CLOSED-…`**) to **`TESTING-…`** and then followed **`TESTER.md`**’s unfiltered **`cargo test`**. That bypasses this file’s **Step 3** (the **`cdp_retry_`** filter) and is **not** a valid run for this slug — record **handoff defect** or wait for **`TESTPLAN-` → `UNTESTED-`**, then **`UNTESTED-` → `TESTING-`**.

**Fix for retest:** Use **`UNTESTED-…`** from the branch you test; run **Steps 1–4** using the **bash block** and **1) Static review** for `rg` (see **Paths by cwd** for the path column only); ignore unfiltered suite results for pass/fail.

### Pass/fail scope (read first)

- **Required for a pass on this task:** **Steps 1–4** below (static review, compile, **targeted lib tests**, spot-check). All required commands must succeed.
- **Not a required step:** There is **no** step in this document that runs **`cargo test --no-fail-fast`** or **`cargo test -p mac_stats --no-fail-fast`** **without** the **`cdp_retry_`** filter. If your runbook prints such a line as **mandatory** for this slug, treat that line as **obsolete** for pass/fail. You may run an unfiltered suite for diagnostics, but **do not** fail or emit **TESTPLAN-** solely because that optional command failed while **Step 3** passed.
- **Authoritative gate:** **Steps 1–4** override any older checklist that treated **unfiltered** crate tests as mandatory for this task. Failures in unrelated modules **do not** invalidate a pass when **Steps 1–4** succeed.
- **Checklist conflict:** Never rename to **TESTPLAN-** solely because an unfiltered `cargo test` failed while the **`cdp_retry_`** command in **Step 3** passed.
- **Not required:** a full **`cargo test -p mac_stats --no-fail-fast`** over the entire crate. Other tests can fail in some environments; those failures are **out of scope** unless the failing test is one of the **`cdp_retry_`** tests in step 3 or clearly implicates `evaluate_one_plus_one_blocking_timeout` / `check_browser_alive` / `clear_browser_session_on_error`.

### Wrong command vs gate (quick reference)

| Situation | Pass/fail for this task? |
|-----------|---------------------------|
| `cargo test … -p mac_stats --lib cdp_retry_ --no-fail-fast` — all match tests pass | **Required** — this is **Step 3** |
| `cargo test … -p mac_stats --no-fail-fast` (no `cdp_retry_`, not limited to those tests) | **Informational only** — **do not** fail the task or emit **TESTPLAN-** based solely on this |
| `cd src-tauri && cargo test --no-fail-fast` (historical shorthand in old reports) | Same as row above — **not** the gate |
| **Step 3** passes; optional full suite fails in `discord::`, `scheduler::`, etc. | **Pass** **Steps 1–4**; note unrelated failures separately in the report |
| `cargo test … -p mac_stats --lib --no-fail-fast` (**no** `cdp_retry_` token) | **Not** the gate — runs **all** lib tests (~800+); failures there do **not** override a passing **Step 3** |
| `cargo test … -p mac_stats --no-fail-fast` (**no** `--lib`, **no** `cdp_retry_`) | **Not** the gate — runs **lib plus other test targets** (e.g. integration / binary tests) as Cargo discovers them; failures there are **often environmental** and do **not** override **Step 3** |
| **`TESTER.md` (or template) `cargo test` line with no `cdp_retry_` token** | **Not** the gate — **Step 3** in **this** file replaces that line for pass/fail |

### Report evidence (include in tester notes)

- The **basename of the task file** you executed from (must be **`UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`** after publication — if you only had **`CLOSED-…`**, state that as **handoff defect**, not a code fail).
- The **full command line** for **Step 3**, which **must** contain the substrings **`--lib`** and **`cdp_retry_`** (e.g. `cargo test … --lib cdp_retry_ --no-fail-fast`).
- Cargo output showing **`running N tests`** with **N ≥ 1** and **`test result: ok`** for that command.
- If you also ran an unfiltered suite, label it **“optional / diagnostic”** and do **not** use it as the pass/fail bar.

### Copy-paste — full gate (from repository root)

Run this block **as-is** in **bash** after **PF-1** (or **PF-0** then **PF-1**) / the sanity checks. From **fish**, use `bash -lc '…'` with the **entire** script in **single quotes** so fish does not eat `$(pwd)` or other bash syntax. It covers **Steps 1–3** (gate) below (static review, compile, targeted tests). **Step 4** is **only** the manual spot-check in the editor — it is **not** in this script.

```bash
set -e
set -o pipefail
test -f src-tauri/Cargo.toml || { echo >&2 "ERROR: run from mac-stats repo root (missing src-tauri/Cargo.toml)."; exit 1; }
test -f src-tauri/src/browser_agent/mod.rs || { echo >&2 "ERROR: missing src-tauri/src/browser_agent/mod.rs"; exit 1; }
echo ">>> PWD (expect mac-stats repo root): $(pwd)"
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

**Legacy equivalent (optional):** Older reports used `rg 'block_on|Never use .Handle::block_on' …` (sometimes with `| head -n 20`). That pattern is **acceptable** if it returns the anti-`block_on` comment, but **do not** pipe **`rg` to `head`** under **`set -o pipefail`** — use **`rg -n -m 20`** as in the copy-paste block. The lines above also require **`recv_timeout(BROWSER_CDP_HEALTH_CHECK_TIMEOUT)`** as an alternate match so the health-check timeout stays visible in static review.

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

**Pass:** Cargo **runs** the `cdp_retry_`-filtered lib tests and reports a **non-zero** count. Expect **at least 2** tests (current names below). If Cargo reports **0** tests executed, **fail** this step and verify: (1) cwd + invocation match one of the two forms above (from root you **must** use `--manifest-path src-tauri/Cargo.toml`; there is no `Cargo.toml` at repo root), (2) the filter is exactly `cdp_retry_`, not `cdp_retry`, (3) the package is `-p mac_stats`. Optional diagnostic from repo root (confirms Cargo is applying the **name** filter, not passing stray args after **`--`**):

```bash
cargo test --manifest-path src-tauri/Cargo.toml -p mac_stats --lib cdp_retry_ -- --list
```

You should see **≥ 1** line containing **`cdp_retry_`** (typically two tests) before re-running the full test command. If **no** listed test name contains **`cdp_retry_`**, your **`cargo test`** invocation does not match **Step 3** (wrong manifest, wrong `-p`, or filter placed after **`--`**).

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

This is **diagnostic** only. **Do not** use this command as the **required** pass/fail bar for this task. **Do not** list it in your report as a failed **acceptance** step when **Steps 1–4** passed.

If it fails, **do not** fail the task **unless** the failure is in the **`cdp_retry_`** tests from step 3 or clearly tied to the health-check symbols in step 1. Otherwise record it as **environment / unrelated suite** in your report, not as a regression for this task.

## Canonical history

Cumulative tester reports and older verification notes: **`tasks/CLOSED-20260321-1345-browser-use-cdp-health-check-ping.md`** (reference only).

**Do not copy “Commands run” from `CLOSED-…` into a new pass.** Archive blocks often show **`cd src-tauri && cargo test --no-fail-fast`** without **`cdp_retry_`** — that reflects **older operator habit** or **pre-revision** checklists, **not** the current **Steps 1–4** gate. For execution, use **only** this file’s **Copy-paste — full gate** + **Step 4**.

## Publication workflow (TESTPLAN → UNTESTED)

While instructions are edited, the task lives as **`TESTPLAN-20260321-1345-browser-use-cdp-health-check-ping.md`**. When ready for **`003-tester`**, the coder renames **`TESTPLAN-` → `UNTESTED-`** (same stamp and slug). After that rename, **`tasks/UNTESTED-20260321-1345-browser-use-cdp-health-check-ping.md`** is the only executable queue filename for this slug (see **Operator filename** and **Phase 0** above).

If the branch already contains **`UNTESTED-…`** (no **`TESTPLAN-…`** file), a coder may **edit `UNTESTED-…` in place** to fix instructions; that is equivalent to publishing a fresh **`UNTESTED-`** after a **`TESTPLAN-` → `UNTESTED-`** rename, without an extra filesystem rename on that branch.

**This revision (l):** Coder **`TESTPLAN-…` → `UNTESTED-…`** for retest — **zero-ambiguity gate** block at top of **Testing instructions**; **PF-3 vs Step 3** clarified (read vs run); **`cdp_retry_` underscore** called out; **wrong `cargo test` ⇒ procedure error** one-liner. Carries forward **(k)**. Ready for **`003-tester`** (`UNTESTED-` → `TESTING-`).
