# OpenClaw: hung turn wall-clock timeout + output event gate

**On-disk name (this file):** Same stamp `20260321-2000` and slug `openclaw-hung-turn-timeout-event-gate`; prefix is either:

- **`tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** — **tester queue** (ready to run). Follow [`003-tester/TESTER.md`](../003-tester/TESTER.md): **`UNTESTED-…` → `TESTING-…`** at run start.
- **`tasks/TESTPLAN-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** — **not** on the tester queue; coder is revising **Testing instructions** only. Testers **stop** until the coder renames **`TESTPLAN-…` → `UNTESTED-…`** (same stamp + slug).

**No** mac-stats product code change is required for this task file.

**Instruction revision note:** A prior run flagged **Testing instructions** or the **stated environment** as defective (not a mac-stats implementation failure). This markdown body is the only authoritative spec: follow **Verification commands** below exactly, not snippets copied from **`CLOSED-*`** history (those may use wrong paths such as top-level **`src/`**). **Latest repair (2026-03-30 coder pass):** (1) **Queue banner** matches **`UNTESTED-…`** on disk (no “this file is TESTPLAN” confusion). (2) **Closure checklist** after **Minimal run order** for tick-before-outcome verification. (3) **Network / toolchain** note: `cargo` fetch or missing `rustc` is **environment** (**`WIP-…`** per **TESTER.md**), not **`TESTPLAN-…`**. (4) **Preflight** vs **`cargo --manifest-path`**: preflight can pass from **`tasks/`**; **`cargo`** still needs **`pwd`** at repo root unless you use block **B** from **`src-tauri/`**. (5) **`BLOCK: A1 or A2`** = pick **exactly one** of **A1** or **A2** per run (never both); after step **0** tells you to **`cd`** to repo root, run **`test -f src-tauri/Cargo.toml`** before pasting **A1**/**A2**. **(6) Second pass (2026-03-30):** dual-prefix banner; **Testing instructions** **TL;DR** + **happy-path** list; **A1 vs A2** rule (use **A1** whenever `.git` exists); **expected `rg` output** note; **IDE “Run selection”** warning (partial selection = invalid run).

**Coder handoff (future):** **`UNTESTED-…` → `TESTPLAN-…`** for another instruction repair, edit **Testing instructions** / clarity wording only, then **`TESTPLAN-…` → `UNTESTED-…`**. Testers **only** start from **`UNTESTED-…`** — **not** **`TESTPLAN-…`**.

Full-turn wall-clock timeout stops a hung agent run: output gate closes (no Discord status/draft/ATTACH spam), user-visible **Turn timed out** reply, optional `about:blank` cleanup only if the timed-out `request_id` still owns the coordination slot.

**Scope (read this first):** The words “OpenClaw” / “agent router” in the title describe **product behavior** that is implemented in **this repository (mac-stats)**, not in the sibling checkout at `../openclaw`. For verification you only search and build **mac-stats**. Searching `../openclaw` or expecting symbols there will fail and is **out of scope** for this task.

**“Event gate”** here means **`TurnOutputGate`** in Rust (`src-tauri/src/commands/turn_lifecycle.rs`): a shared flag the tool loop consults so outbound status/drafts stop after a turn timeout.

## Acceptance criteria

1. `TurnOutputGate` is defined as `pub type TurnOutputGate = Arc<AtomicBool>` in `commands/turn_lifecycle.rs`. The tool loop (`commands/tool_loop.rs` and related paths) calls `gate_allows_send` so sends are suppressed after the gate is closed.
2. `finalize_turn_timeout` in `commands/turn_lifecycle.rs` returns `OllamaReply` whose `text` starts with `**Turn timed out**` and includes the budget in seconds (see the `format!` that builds the user message).
3. **Log strings (static check):** The following **substrings** must appear inside **mac-stats** Rust sources (typically inside a longer `format!` / macro string — **`rg` only needs a line-level substring match**, not a whole-line exact copy). Use the **Verification commands** `rg` lines verbatim (**`-F`** only where shown, and **single quotes** around `'**Turn timed out**'` so the shell does not glob `**`). A live Discord timeout repro is **optional**, not required for pass.
   - Substring **`closing output gate after turn wall-clock timeout`** — in **`src-tauri/src/commands/ollama.rs`** (router path when the wall-clock limit fires).
   - Substrings **`turn wall-clock timeout`** and **`closing output gate and running cleanup`** — both appear inside the **same** `tracing::warn!` format string in **`src-tauri/src/commands/turn_lifecycle.rs`** (`finalize_turn_timeout`). **Expected:** the two separate `rg` lines in blocks **A1**/**A2**/**B** may report the **same line number** twice; that still counts as pass.
4. **`cargo check`** and **`cargo test`** for the **`mac_stats`** package succeed (exit **0**, zero failing tests). The Cargo **package** name is **`mac_stats`** (underscore), declared in **`src-tauri/Cargo.toml`**; pass **`-p mac_stats`** whenever you use **`--manifest-path src-tauri/Cargo.toml`**. Equivalent ways to satisfy this: run **Verification commands** block **A1**/**A2** (repo root + `--manifest-path src-tauri/Cargo.toml -p mac_stats`) or block **B** (cwd **`src-tauri/`** + `cargo check` / `cargo test` with **`-p mac_stats`** or default package). This project targets **macOS**; use a Mac so results match maintainer expectations. Linux CI or a non-macOS checkout may fail link steps or skip platform tests — that mismatch is **not** a product failure; rerun on macOS.

## Testing instructions

### TL;DR (static gate)

1. **mac-stats** repo only — not `../openclaw`. Rust paths are under **`src-tauri/src/`** (from repo root) or **`src/`** (only when cwd is **`src-tauri/`** — block **B**).
2. Run **Tester quick gate** step **0** → **`cd`** if told → **`test -f src-tauri/Cargo.toml`** before **A1**/**A2**.
3. Run **Preflight** (git or no-git) once in the **same terminal** you will use for verification.
4. Paste **one** complete block **A1** *or* **A2** *or* **B** from the first **`set -e`** through the **last `rg`** — **no** line omitted, **no** mixing blocks, **no** running **`cargo`**/**`rg`** alone from an IDE snippet.
5. **Pass:** `cargo check` + `cargo test` exit **0**; every **`rg`** in that block prints **≥1 line** (for **`turn_lifecycle.rs`**, two `rg` lines may show the **same** line number — still pass).

### Before you run anything (read once)

| Step | Action |
|------|--------|
| 1 | **Host:** macOS with `cargo`, `rustc`, and `rg` on `PATH`. Linux-only or missing toolchain → stop, report **environment blocked** per **TESTER.md** (typically **`WIP-…`**), **not** **`TESTPLAN-…`**. |
| 2 | **Repo:** You are in the **mac-stats** tree (folder that contains **`src-tauri/Cargo.toml`**). Do **not** search **`../openclaw`** or top-level **`src/`** for Rust gate strings. |
| 3 | **Shell:** Paste verification blocks in **`bash`** (or zsh with `set -e` behaving as documented). **fish** → use `bash -lc '…'`. |
| 4 | **Block choice:** Run **Tester quick gate** step **0** below; obey the printed **BLOCK:** (A1/A2 vs B). Never mix path styles from different blocks in one paste. |
| 5 | **A1 vs A2:** If the probe says **A1 or A2**, use **A1** whenever this tree is a **git clone** (`.git` present and **`git rev-parse --show-toplevel`** works). Use **A2** **only** when there is **no** `.git` (tarball/export) or **`git rev-parse` fails**. **Do not** pick **A2** for convenience if **A1** applies. **Do not** run **A1** and then **A2** in the same session — one successful block is enough. |
| 6 | **One paste:** Run **exactly one** of **A1**, **A2**, or **B** from **`set -e` through the last `rg`** without changing directory mid-block. |
| 7 | **After step 0 says `cd …`:** Run that **`cd`**, then **`test -f src-tauri/Cargo.toml`** (expect success) **before** pasting **A1** or **A2**. If that **`test`** fails, you are not at mac-stats **repo root** yet — fix **`cd`** and retry. |
| 8 | **`--manifest-path` vs Preflight:** **Preflight** uses **`$REPO_ROOT`** absolute paths, so it can succeed from **`tasks/`** or any subfolder. **`cargo --manifest-path src-tauri/Cargo.toml`** is resolved **relative to `pwd`**. If you run **only** the **`cargo`** / **`rg`** lines from **A1**/**A2** while cwd is still a subdirectory, Cargo looks for **`…/tasks/src-tauri/Cargo.toml`** (missing) → **false failure**. Always paste the **entire** block, including the **`cd`** that establishes repo root (or use **B** from **`src-tauri/`**). |
| 9 | **IDE / “run selection”:** Highlighting **`cargo`**…**`rg`** without the leading **`cd`**, **`REPO_ROOT=…`**, and **`test -f`** lines is an **invalid** run — same failure mode as step 8. |

**Fast path (experienced testers, git clone, macOS):** **Tester quick gate** step **0** → if **BLOCK: B**, paste **B** only. If **A1 or A2**, **`cd`** if instructed → **Preflight (git)** → paste **Verification commands → A1** in full (never skip **`cd "$REPO_ROOT"`** / **`test -f src-tauri/Cargo.toml`**). **No `.git`:** **A2** only, with a real absolute path on the **`cd`** line. **Pass** = every command exits **0** and every `rg` prints at least one line.

**Happy path (ordered):** **0** Tester quick gate → **1** optional **`cd`** + **`test -f src-tauri/Cargo.toml`** → **2** Preflight → **3** one full block **A1** or **A2** or **B** → **4** closure checklist (if you already renamed queue to **`TESTING-…`**).

### Coder publication after TESTPLAN repair

- If the task file on disk is **`tasks/TESTPLAN-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`**, fix **Testing instructions** / wording in that file, then **rename** it to **`tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** (same stamp `20260321-2000` and slug `openclaw-hung-turn-timeout-event-gate`). **Do not** change the stamp or slug.
- **Retest queue name is always `UNTESTED-…`**. Testers start from **`UNTESTED-…`**, not **`TESTPLAN-…`**.

### Tester quick gate (read first)

0. **Pick the verification block** (run this **before** copying **A1**/**A2**/**B**; no `set -e` required). This probe works from the **repo root**, from **`src-tauri/`**, and from **any subdirectory** of a git clone (e.g. `tasks/`) — the old one-line check only looked at `./src-tauri/Cargo.toml` and wrongly printed **BLOCK: none** when your shell was not already at repo root.

```bash
GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || GIT_ROOT=""
if test -n "$GIT_ROOT" && test -f "$GIT_ROOT/src-tauri/Cargo.toml"; then
  if test -f src-tauri/Cargo.toml; then
    echo "BLOCK: A1 or A2 (cwd is mac-stats repo root). Do not use B."
  elif test -f Cargo.toml && test -f src/commands/turn_lifecycle.rs; then
    echo "BLOCK: B (cwd is src-tauri/ crate root). Do not use A1/A2."
  else
    echo "BLOCK: A1 or A2 — cwd is inside the clone but not repo root or src-tauri/. Before pasting A1/A2 run: cd $(printf %q "$GIT_ROOT")"
  fi
elif test -f src-tauri/Cargo.toml; then
  echo "BLOCK: A1 or A2 (cwd is mac-stats repo root; no usable git root). Do not use B."
elif test -f Cargo.toml && test -f src/commands/turn_lifecycle.rs; then
  echo "BLOCK: B (cwd is src-tauri/ crate root). Do not use A1/A2."
else
  echo "BLOCK: none — cd to mac-stats repo root (folder containing src-tauri/) or into src-tauri/, then run this probe again."
fi
```

If the script prints **BLOCK: none**, `cd` as indicated and re-run step **0**. If it tells you to **`cd '…'`** (quoted path), run that **`cd`** before pasting **A1** or **A2** (those blocks assume repo root for relative paths). Immediately after that **`cd`**, run **`test -f src-tauri/Cargo.toml && echo "OK: repo root"`**; if it fails, do **not** paste **A1**/**A2** yet.

**Why step 0 matters:** From a subdirectory (e.g. **`tasks/`**), a naive `test -f src-tauri/Cargo.toml` fails even though you are inside the clone. Step **0** uses **`$GIT_ROOT`** so it can tell you to **`cd`** to the real repo root before **A1**/**A2**.

**`BLOCK: A1 or A2` is not two blocks:** It means “you must end up at **repo root** before **`cargo --manifest-path src-tauri/Cargo.toml`**.” Use **A1** *or* **A2** to get there — whichever matches your tree (**git** vs **no git**), then stop.

1. **Queue file per [`003-tester/TESTER.md`](../003-tester/TESTER.md):** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only. The tester rename chain is **`UNTESTED-…` → `TESTING-…` → (`CLOSED-…` or `WIP-…`)**. Testers **must not** rename **`TESTPLAN-…` → `TESTING-…`** (wait for **`UNTESTED-…`** first).
2. **Host and toolchain:** Run on **macOS** with **`cargo`**, **`rustc`**, and **`rg`** on your `PATH`. Criterion **4** requires a full **`cargo check`** + **`cargo test`** for **`mac_stats`** to exit **0** on this platform. If you only have Linux (or CI images without the macOS toolchain), **stop**: append **environment blocked** to the test report and rename the queue file per [`003-tester/TESTER.md`](../003-tester/TESTER.md) (typically **`WIP-…`**). That is **not** a product failure and **not** a reason to bounce the task to **`TESTPLAN-…`**. The **TESTPLAN-** prefix is for bad *instructions* in this task file, not for missing macOS or toolchain.
3. **Inventory (optional sanity check):** from mac-stats repo root,  
   `ls tasks/*20260321-2000*openclaw-hung-turn-timeout-event-gate.md 2>/dev/null || true`  
   For a **ready-to-run** queue you want **`UNTESTED-…`** (and may also see **`CLOSED-…`** as history). **If you see `TESTPLAN-…` instead of `UNTESTED-…`,** the coder is still revising **Testing instructions** — **do not** start [`003-tester/TESTER.md`](../003-tester/TESTER.md); wait for **`TESTPLAN-…` → `UNTESTED-…`**. If **only** **`CLOSED-…`** appears, **stop** — restore or fetch **`UNTESTED-…`**; do **not** treat **`CLOSED-…`** as the queue file.
4. **Run one verification block in one paste:** Choose **A1**, **A2**, or **B** in **Verification commands** and execute that block **from the first `set -e` through the last `rg`** without changing directory between lines. **Do not** run only the **`cargo`** lines or only the **`rg`** lines, and **do not** mix lines from different blocks. Mixing repo-root paths (`src-tauri/src/…`) with crate-root paths (`src/…`) in the same session causes false failures (see **Two different directories named `src`**).
5. **`rg` and `set -e`:** With **`set -e`**, **`rg` exits 1** when a pattern has **no** matches and the script **stops** — that means **fail** for this task. All patterns in the block are required to match somewhere in the given paths (except the documented “same line twice” case for **`turn_lifecycle.rs`**).

### Operator filename (`003-tester/TESTER.md`)

- **Executable queue file for a real run:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only. At run start, rename **`UNTESTED-` → `TESTING-`**, run **Verification commands**, then apply outcome naming per **TESTER.md**.
- **Missing `UNTESTED-…` at repo tip:** **Stop.** Do **not** verify from **`CLOSED-…`** alone or invent **`TESTING-…`** from **`CLOSED-…`** unless your operator runbook explicitly allows it. Sync/pull for **`UNTESTED-…`**, or return a **queue / handoff defect** to the coder.
- **Emit `TESTPLAN-` only when this markdown is wrong** (wrong paths, wrong queue rules, ambiguous `cargo` cwd wording). Do **not** use **`TESTPLAN-`** because **`rg`** on top-level **`src/`** returns no matches — that is a **tester path mistake** (see **Two different directories named `src`**). Do **not** use **`TESTPLAN-`** for “no Mac” / Linux-only runs — use **`WIP-…`** plus an environment note (step **2** above).
- **[`003-tester/TESTER.md`](../003-tester/TESTER.md)** says to prefer **`cargo check` / `cargo test` in `src-tauri/`**. For this task that means **either** block **B** (cwd = `src-tauri/`) **or** blocks **A1**/**A2** (repo root + **`--manifest-path src-tauri/Cargo.toml -p mac_stats`**). It does **not** mean “run plain **`cargo test`** from repo root without a manifest” — that will fail on mac-stats (no root **`Cargo.toml`**).

### Task-file identity (stamp `20260321-2000`, slug `openclaw-hung-turn-timeout-event-gate`)

The **spec** is this markdown body. **Verification commands** live only in **Verification commands** below — not in chat logs, not in `CLOSED-*` archives.

| On-disk prefix | Meaning | Who acts |
|----------------|---------|----------|
| **`TESTPLAN-…`** | Instructions failed a review; coder is revising **Testing instructions** / wording. | **Coder** renames **`TESTPLAN-…` → `UNTESTED-…`** (same stamp + slug) when ready for retest. |
| **`UNTESTED-…`** | Ready for the tester queue. | **Tester** follows [`003-tester/TESTER.md`](../003-tester/TESTER.md) (e.g. **`UNTESTED-…` → `TESTING-…`** at run start). |
| **`TESTING-…`** | A test run is in progress. | Tester finishes per **TESTER.md** and sets the outcome filename. |
| **`CLOSED-…`** | Append-only history for this stamp. | **Not** the live queue file. **Never** use it as a substitute if **`UNTESTED-…`** is missing. |

**Parallel `CLOSED-*` file:** `tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` may exist next to **`UNTESTED-…`** / **`TESTPLAN-…`**. Old appended notes there sometimes used **`rg … src/`** (top-level **frontend** `src/`). For this task that path is **always wrong** and yields **false “missing feature” results**. Do **not** copy commands from **`CLOSED-*`**.

**Queue defects to avoid:**

- **Operator names `UNTESTED-…` but only `CLOSED-…` exists** — Do **not** “verify against CLOSED.” Update your tree, restore **`UNTESTED-…`** from git, or bounce the task to the coder. Appending new results into **`CLOSED-…`** without a live **`UNTESTED-…`**/`TESTING-…` step is out of procedure.
- **Only `TESTPLAN-…` is present** — Instructions are still in repair; wait for **`TESTPLAN-…` → `UNTESTED-…`** before starting the **TESTER.md** rename chain.

**Current handoff:** After coder publication, the **tester-visible** filename is **`tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** — follow **TESTER.md** (**`UNTESTED-…` → `TESTING-…`** at run start). If your tree only has **`TESTPLAN-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`**, **do not** start **TESTER.md**; wait for **`TESTPLAN-…` → `UNTESTED-…`** (same stamp + slug).

### Shell compatibility

- **Preferred:** paste blocks **A1**/**A2**/**B** into **`bash`** (macOS: `/bin/bash` or `bash -lc '…'`) so **`set -e`** aborts the same way as in CI. **zsh** usually runs these snippets correctly; if a failing command does **not** stop the script, re-run the block under **`bash`** before filing a failure.
- The blocks below use **`bash`** syntax (`set -e`, `$(…)`). On macOS, **Terminal.app** defaults to **zsh**.
- **Quote the `**Turn timed out**` pattern for `rg`:** the verification blocks use **single quotes** around the fixed string so no shell treats `**` as a glob. If you type the command by hand, use **`rg -n -F '**Turn timed out**' …`** (do not unquote the pattern).
- If your login shell is **fish** (or another non-POSIX shell), run the block explicitly with Bash, for example:  
  `bash -lc 'set -e; REPO_ROOT="$(git rev-parse --show-toplevel)"; …'`  
  or paste the block after running **`bash`** interactively. **Do not** run the same script verbatim in **fish**; `set -e` and `$(…)` differ.

### Environment

- **Repository:** **mac-stats** only (directory that contains **`src-tauri/Cargo.toml`**, plus top-level `src/` and `src-tauri/`). There is **no** workspace **`Cargo.toml`** at the repository root — the Rust package is **`mac_stats`** under **`src-tauri/`** only.
- **Host:** **macOS** + stable **Rust** (`cargo` / `rustc` on `PATH`) + **[ripgrep](https://github.com/BurntSushi/ripgrep)** (`rg` on `PATH`). If `rg` is missing, install it or use your editor’s search; the patterns below are the exact substrings to find. If **`rustc` / `cargo` is missing**, the wrong toolchain is active, or **`cargo` cannot reach crates.io** (offline sandbox, corporate proxy, blocked DNS), that is an **environment** problem — use **TESTER.md** outcome **`WIP-…`** with a short note, **not** a **`TESTPLAN-…`** bounce ( **`TESTPLAN-`** is only for bad *instructions* in this task file).
- **Preferred `cargo` cwd (blocks **A1** / **A2**):** stay at **repo root** and use **`cargo … --manifest-path src-tauri/Cargo.toml -p mac_stats`**. That avoids the common mistake of running **`cargo test`** from repo root **without** a manifest (Cargo errors or wrong package) and avoids relying on a subshell **`cd src-tauri`**.
- **Alternate `cargo` cwd (block **B**):** **`src-tauri/`** (crate root). There, use **`cargo check -p mac_stats`** / **`cargo test -p mac_stats`** (or plain **`cargo check`** / **`cargo test`** since this directory is a single-package manifest).
- **Block B from repo root:** At repo root, **`test -f Cargo.toml`** (first line of block **B**) **fails** because mac-stats has **no** root **`Cargo.toml`**. That is **not** a broken task — you picked the wrong block; use **A1** or **A2**.
- **Wrong repo:** If `git rev-parse` / `test -f src-tauri/Cargo.toml` fails, stop — fix cwd before **`cargo`** or **`rg`** paths that assume repo root.
- **Typo trap:** the **repo folder** is often **`mac-stats`** (hyphen). The **Cargo package** is **`mac_stats`** (underscore). Do not drop **`-p mac_stats`** when using **`--manifest-path`** from repo root.
- **`--manifest-path` is relative to the shell cwd (critical):** Blocks **A1** / **A2** assume **`pwd`** is the **repo root** (the directory that **contains** `src-tauri/`). After `cd "$REPO_ROOT"` (A1) or `cd /ABSOLUTE/.../mac-stats` (A2), **`test -f src-tauri/Cargo.toml`** must succeed. If your cwd is already **`…/mac-stats/src-tauri`**, the path **`src-tauri/Cargo.toml`** points at a **non-existent** `src-tauri/src-tauri/Cargo.toml` — **do not** paste **A1**/**A2** there. Either **`cd ..`** to the repo root and use **A1**/**A2**, or stay in **`src-tauri/`** and use block **B** only (`cargo check` / `cargo test` without `src-tauri/` prefix on paths).
- **Subdirectory example (`tasks/`, `docs/`, etc.):** From **`…/mac-stats/tasks`**, **`cargo check --manifest-path src-tauri/Cargo.toml`** asks Cargo for **`…/mac-stats/tasks/src-tauri/Cargo.toml`** — wrong. **Preflight** can still print **OK** from there because it checks **`"$REPO_ROOT/src-tauri/Cargo.toml"`**. That does **not** mean you can skip **`cd "$REPO_ROOT"`** in **A1** (or the **`cd /ABSOLUTE/...`** line in **A2**).
- **If `cargo` prints `could not find Cargo.toml`:** you ran **`cargo`** from the **repo root** without **`--manifest-path src-tauri/Cargo.toml`**, your cwd is not the mac-stats tree, you used **A1**/**A2** while cwd was **`src-tauri/`** (see bullet above), **or** you ran **`cargo`** with **`--manifest-path src-tauri/Cargo.toml`** from a **non-root** subdirectory without **`cd`** to repo root first. Use block **A1**/**A2** from repo root or **`cd src-tauri`** and block **B**.

### Two different directories named `src` (critical)

| Path from repo root | What it is |
|---------------------|------------|
| **`src/`** | Frontend (HTML/JS/CSS). **No** `TurnOutputGate` / turn-timeout Rust strings here. |
| **`src-tauri/src/`** | Rust crate (**all** static checks for this task). |

**Common false failure:** From repo root, running `rg "TurnOutputGate" src` searches **only** the frontend tree and prints **no matches**. That does **not** mean the feature is missing — you searched the wrong tree. Always use **`src-tauri/src`** in path arguments when your shell’s cwd is the **repo root**.

### What *not* to do

- Do **not** treat zero matches under top-level **`src/`** as a failure.
- Do **not** verify in **`../openclaw`** or any other repo.

### Common instruction defects (typical `TESTPLAN-` causes)

1. **`rg … src/` from repo root** — searches the **frontend** tree only; Rust lives under **`src-tauri/src/`**. Use block **A1**/**A2** or **B** paths exactly.
2. **`cargo check` / `cargo test` from repo root without `--manifest-path`** — often fails or targets the wrong manifest. Use block **A1**/**A2** (`--manifest-path src-tauri/Cargo.toml -p mac_stats`) or **`cd src-tauri`** then block **B**.
3. **Running A1 or A2 while cwd is `src-tauri/`** — **`--manifest-path src-tauri/Cargo.toml`** is resolved relative to cwd, so Cargo looks for **`src-tauri/src-tauri/Cargo.toml`** and fails. **`cd ..`** to repo root, or use block **B** only.
4. **Fish (or non-bash) pasted script** — `set -e` / `$(…)` differ; use **`bash -lc '…'`** or run **zsh** with the block as written.
5. **Treating `CLOSED-…` verification snippets as authoritative** — historical reports may use wrong paths; follow **this** file’s **Verification commands** only.
6. **Using the `git rev-parse` block when `.git` is missing** — use **Verification commands → A2** (full no-git block), not a partial edit of **A1**.
7. **`rg: command not found`** — install [ripgrep](https://github.com/BurntSushi/ripgrep) or search your editor for the **exact** substrings under **`src-tauri/src/`**; the acceptance literals must still be located in the files named in criteria 3.
8. **Unquoted `**Turn timed out**` in the shell** — some shells glob `**`; always run **`rg -n -F '**Turn timed out**' …`** as in the verification blocks.
9. **Quick gate from `tasks/` (or any subdir) with the old probe** — `./src-tauri/Cargo.toml` is missing from subdirectories, so **BLOCK: none** was a false signal. Use **Tester quick gate** step **0** as written here (git-aware), then **`cd`** to repo root if instructed before **A1**/**A2**.
10. **Preflight OK from a subdir, then only `cargo`/`rg` lines from A1/A2** — Preflight uses **`$REPO_ROOT`**-absolute **`test -f`** checks; **A1**/**A2** **`cargo --manifest-path src-tauri/Cargo.toml`** does **not**. You **must** paste the full block (including **`cd`**) or **`cd`** to repo root yourself before **`cargo`**.

### Preflight (required)

Run **one** of these, depending on whether you have a `.git` directory.

**Inside a git clone of mac-stats (recommended):** from any directory in that clone,

```bash
set -e
REPO_ROOT="$(git rev-parse --show-toplevel)"
test -f "$REPO_ROOT/src-tauri/Cargo.toml"
test -f "$REPO_ROOT/src-tauri/src/commands/turn_lifecycle.rs"
echo "OK: mac-stats repo root = $REPO_ROOT"
command -v rg >/dev/null && echo "OK: rg" || echo "WARN: install ripgrep or search manually"
```

If `git rev-parse` errors, you are **not** in the mac-stats git checkout — fix your cwd or use the tarball path below.

**Same shell as verification:** Preflight may run from **`tasks/`** and pass. **A1**/**A2** still need **`pwd` = repo root** before **`cargo --manifest-path src-tauri/Cargo.toml`** — either paste the full **A1**/**A2** block (starts with **`cd`** to root) or run **`cd "$(git rev-parse --show-toplevel)"`** yourself, then paste **A1** from **`set -e`** onward.

**Tarball / no `.git`:** `cd` manually to the folder that **contains** `src-tauri/Cargo.toml`, then:

```bash
set -e
test -f src-tauri/Cargo.toml
test -f src-tauri/src/commands/turn_lifecycle.rs
echo "OK: cwd is mac-stats root (no git)"
command -v rg >/dev/null && echo "OK: rg" || echo "WARN: install ripgrep or search manually"
```

### Pass / fail summary (static gate)

| Check | Pass |
|--------|------|
| Preflight | Both `test -f` lines succeed; you know your repo root path. |
| `cargo check` + `cargo test` for **`mac_stats`** | Exit **0**; **zero** failing tests — via block **A1**/**A2** (`--manifest-path … -p mac_stats`) **or** block **B** (cwd **`src-tauri/`**, `-p mac_stats` or default). |
| `rg` for gate symbols | At least one match for each **distinct** pattern in block **A1**/**A2** or **B** (see paths for your cwd). For **`turn_lifecycle.rs`**, the two log-string `rg` lines may both hit the **same** source line — that is still pass. |
| Top-level `src/` | May show **no** matches for Rust gate strings — **not** a failure. |

### Minimal run order (single shell session)

Do **not** mix **A1**/**A2** path prefixes (repo root + `src-tauri/…`) with **B** path prefixes (crate root + `src/…`) in one run.

1. Run **Tester quick gate** step **0** (directory probe). If the output includes **`cd '/path/to/mac-stats'`** (or similar), run that **`cd`** so **A1**/**A2** relative paths resolve, then **`test -f src-tauri/Cargo.toml`**. Then run **Preflight (required)** for git (**A1-style**) or no-git (**A2-style**).
2. Paste **exactly one** of **A1**, **A2**, or **B** **in full** from **Verification commands** (same terminal; **`set -e`** should still be active; use **`bash`** if unsure — see **Shell compatibility**). Prefer **A1** if `.git` exists, else **A2**; use **B** only when step **0** says so.
3. If **`cargo`** fails with **`could not find Cargo.toml`**, you are not using **A1**/**A2**/**B** correctly — re-read **Environment** and **Common instruction defects**.

### Closure checklist (tick before outcome naming)

Use this only after the queue file is **`UNTESTED-…`** and you have started the **TESTER.md** rename chain (**`UNTESTED-…` → `TESTING-…`**). If you are still reading **`TESTPLAN-…`**, stop — the coder has not published the queue yet.

1. **Probe:** Ran **Tester quick gate** step **0**; chose **one** of **A1** / **A2** / **B** matching the printed **BLOCK:** (no mixing blocks).
2. **Preflight:** Ran the matching **Preflight (required)** variant (git **or** no-git); both `test -f` lines succeeded.
3. **One paste:** Executed **one** full block from **Verification commands** from the first **`set -e`** through the **last `rg`** without changing directory mid-block.
4. **Cargo:** `cargo check` and `cargo test` for **`mac_stats`** exited **0** with **zero** failing tests.
5. **Ripgrep:** Every pattern in that block matched at least once in the paths given (for **`turn_lifecycle.rs`**, two `rg` lines may show the **same** line number — still pass).

### Optional runtime check

To see log lines in a real run, reproduce or simulate a turn timeout and grep **`~/.mac-stats/debug.log`** for the same substrings. This is **not** required if static `rg` + `cargo` checks pass.

## Verification commands

Use **`bash`** for the blocks below unless you have confirmed **`set -e`** behaves correctly in your shell (see **Shell compatibility**).

**Expected output:** Each **`rg`** line must print **at least one** matching line (file path + line number + text). If **`rg`** prints nothing, it exits **1** and **`set -e`** stops the block — treat as **fail** (wrong cwd, wrong block, or missing code). Exception: the two **`turn_lifecycle.rs`** log-string **`rg`** lines may both point to the **same** source line (same line number twice).

Paste **one** complete block (**A1** *xor* **A2** *xor* **B**) in a single shot. **A1** and **A2** are mutually exclusive ways to reach the same cwd (repo root); running both is redundant and can confuse the report. Do **not** split a block halfway; **`set -e`** must stop the script if cwd or paths are wrong. Do **not** cherry-pick only the **`cargo`** or only the **`rg`** lines — the initial **`cd`** + **`test -f`** lines establish the correct cwd. **`cargo --manifest-path src-tauri/Cargo.toml`** is always relative to **`pwd`**; from **`tasks/`** it fails unless you **`cd`** to repo root first (even if **Preflight** already passed).

**Anti-pattern (common false “failure”):** Running **`cargo check`** / **`cargo test`** from repo **root** without **`--manifest-path src-tauri/Cargo.toml -p mac_stats`**, or running **`rg`** with bare **`src/`** paths while cwd is **repo root** (that tree is the **frontend**, not the Rust crate). Use the **full** block chosen in **Tester quick gate** step **0**.

**Pick a block by where you will stand after `cd`:**

| Block | After the block’s `cd`, this must succeed | Wrong choice if… |
|-------|-------------------------------------------|------------------|
| **A1** / **A2** | `test -f src-tauri/Cargo.toml` | You refuse to leave **`src-tauri/`** — use **B** instead. |
| **B** | `test -f src/commands/turn_lifecycle.rs` | `test -f src-tauri/Cargo.toml` succeeds from **`pwd`** — you are at **repo root**; use **A1**/**A2**, not **B**. |

### A1 — Recommended: repo root, **with** `.git`

Use **A1** when this is a **git** checkout and **`git rev-parse --show-toplevel`** returns the mac-stats root. If you already used **A2** successfully, **do not** also run **A1**.

Use **bash** or **zsh** (or `bash -lc '…'` from fish). **`set -e`** stops on the first failing command — do **not** append `|| true` to `cd` or `git rev-parse` here; a wrong directory must **fail** the script.

**If the block dies on `git rev-parse`:** your shell cwd is **not** inside a git working tree (wrong folder, tarball without `.git`, or nested worktree confusion). Fix: **`cd`** into the mac-stats clone and retry **A1**, or use **A2** with an explicit absolute path to the repo root.

**Git prerequisite:** Run from **inside** the mac-stats clone so `git rev-parse --show-toplevel` prints the mac-stats root (the folder that directly contains `src-tauri/`). Starting in `src-tauri/` is OK: git still returns the repo root, and **`cd "$REPO_ROOT"`** moves you to **repo root** before **`cargo`** — required so **`--manifest-path src-tauri/Cargo.toml`** resolves correctly.

**Why `--manifest-path`:** mac-stats has **no** `Cargo.toml` at repo root. Invoking **`cargo test`** from root without **`--manifest-path`** is a frequent false failure; the lines below pin the **`mac_stats`** package explicitly.

```bash
set -e
REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"
test -f src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml -p mac_stats
cargo test --manifest-path src-tauri/Cargo.toml -p mac_stats

rg -n "TurnOutputGate|gate_allows_send|finalize_turn_timeout" src-tauri/src

rg -n -F '**Turn timed out**' src-tauri/src/commands/turn_lifecycle.rs

rg -n "closing output gate after turn wall-clock timeout" src-tauri/src/commands/ollama.rs

rg -n "turn wall-clock timeout" src-tauri/src/commands/turn_lifecycle.rs
rg -n "closing output gate and running cleanup" src-tauri/src/commands/turn_lifecycle.rs
```

### A2 — Recommended: repo root, **no** `.git` (tarball / export)

Use **A2** when **`git rev-parse` fails** or the tree has no `.git` directory — **not** when **A1** already worked. **Replace** **`/ABSOLUTE/PATH/TO/mac-stats`** on the **`cd`** line with the **absolute path** to your mac-stats root (the directory that **directly** contains **`src-tauri/`** — not `src-tauri/` itself, not a parent that only contains the zip name). **Edit that line before pasting**; leaving the placeholder verbatim will **`cd`** to a non-existent path and fail. After editing, **`test -f src-tauri/Cargo.toml`** must succeed; if it fails, the path is wrong.

```bash
set -e
cd /ABSOLUTE/PATH/TO/mac-stats
test -f src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml -p mac_stats
cargo test --manifest-path src-tauri/Cargo.toml -p mac_stats

rg -n "TurnOutputGate|gate_allows_send|finalize_turn_timeout" src-tauri/src

rg -n -F '**Turn timed out**' src-tauri/src/commands/turn_lifecycle.rs

rg -n "closing output gate after turn wall-clock timeout" src-tauri/src/commands/ollama.rs

rg -n "turn wall-clock timeout" src-tauri/src/commands/turn_lifecycle.rs
rg -n "closing output gate and running cleanup" src-tauri/src/commands/turn_lifecycle.rs
```

**Alternate (equivalent) cargo one-liner** (after you are at repo root):  
`( cd src-tauri && cargo check -p mac_stats && cargo test -p mac_stats )` — same package as **`--manifest-path`** above.

**Why two files for log strings:** The router line with **`closing output gate after`** is only in **`ollama.rs`**. The **`turn wall-clock timeout`** / **`closing output gate and running cleanup`** pair is in **`turn_lifecycle.rs`** inside **one** format string — **both `rg` commands may print the same line** (same line number twice). Optional single check:  
`rg -n "turn wall-clock timeout|closing output gate and running cleanup" src-tauri/src/commands/turn_lifecycle.rs`  
You should see **one** line containing both substrings. A single broad `rg` over **`src-tauri/src`** also works; the file-scoped lines above make expected locations obvious.

**Runtime:** `cargo test` for this crate can take several minutes on first run (compilation + tests). A long compile is **not** a hang. First-time dependency download can print network activity; wait for **`cargo`** to exit **0** before interpreting **`rg`** results.

**Exit codes:** With **`set -e`**, any **`cargo`** failure or **`rg`** with **no matches** (exit **1**) aborts the block — treat that as **verification failed** until you fix cwd/block choice. Most such aborts are **wrong directory or wrong block**, not missing Rust code; re-read **Two different directories named `src`** and **Common instruction defects** before using outcome naming in **TESTER.md**.

### B — Alternate: your cwd is already `src-tauri/` (crate root)

Use this block **only** when `pwd` is the directory that contains **`Cargo.toml`** and a **`src/`** subdirectory (that **`src/`** is the **Rust crate source**, not the repo’s top-level frontend **`src/`**). Quick sanity check before `rg`: **`test -f src/commands/turn_lifecycle.rs`** must succeed; if it fails, you are not in `src-tauri/`.

**How to tell crate root vs repo root:** If `test -f src-tauri/Cargo.toml` succeeds from `pwd`, you are at **repo root** — use block **A1**/**A2**, not **B**. If `test -f Cargo.toml` succeeds and `test -f src/commands/turn_lifecycle.rs` succeeds, you are at **crate root** — block **B** is OK.

```bash
set -e
test -f Cargo.toml
test -f src/commands/turn_lifecycle.rs
cargo check -p mac_stats
cargo test -p mac_stats

rg -n "TurnOutputGate|gate_allows_send|finalize_turn_timeout" src

rg -n -F '**Turn timed out**' src/commands/turn_lifecycle.rs

rg -n "closing output gate after turn wall-clock timeout" src/commands/ollama.rs

rg -n "turn wall-clock timeout" src/commands/turn_lifecycle.rs
rg -n "closing output gate and running cleanup" src/commands/turn_lifecycle.rs
```

**Do not** mix **A1**/**A2** and **B** path styles in one shell session: from repo root, **never** pass bare `src/` to `rg` for this task (that is the frontend tree). From **`src-tauri/`**, **never** pass `src-tauri/src` (there is no such path below the crate root).

**Slow tests:** `cargo test --no-fail-fast` (with the same **`-p mac_stats`** / **`--manifest-path`** as your block) is fine; requirement is **zero failing tests** for this crate.

## Test report

_(Tester: append results only on the **queue** file **`UNTESTED-…` → `TESTING-…`** per [`003-tester/TESTER.md`](../003-tester/TESTER.md). **While the filename is `TESTPLAN-…`**, there is no queue slot — coder repair only; **no** test report.)_
