# OpenClaw: hung turn wall-clock timeout + output event gate

Full-turn wall-clock timeout stops a hung agent run: output gate closes (no Discord status/draft/ATTACH spam), user-visible **Turn timed out** reply, optional `about:blank` cleanup only if the timed-out `request_id` still owns the coordination slot.

**Scope (read this first):** The words “OpenClaw” / “agent router” in the title describe **product behavior** that is implemented in **this repository (mac-stats)**, not in the sibling checkout at `../openclaw`. For verification you only search and build **mac-stats**. Searching `../openclaw` or expecting symbols there will fail and is **out of scope** for this task.

**“Event gate”** here means **`TurnOutputGate`** in Rust (`src-tauri/src/commands/turn_lifecycle.rs`): a shared flag the tool loop consults so outbound status/drafts stop after a turn timeout.

## Acceptance criteria

1. `TurnOutputGate` is defined as `pub type TurnOutputGate = Arc<AtomicBool>` in `commands/turn_lifecycle.rs`. The tool loop (`commands/tool_loop.rs` and related paths) calls `gate_allows_send` so sends are suppressed after the gate is closed.
2. `finalize_turn_timeout` in `commands/turn_lifecycle.rs` returns `OllamaReply` whose `text` starts with `**Turn timed out**` and includes the budget in seconds (see the `format!` that builds the user message).
3. **Log strings (static check):** The following literals appear in **mac-stats** Rust sources as written below (copy/paste safe; use the verification `rg` commands). A live Discord timeout repro is **optional**, not required for pass.
   - Substring **`closing output gate after turn wall-clock timeout`** — in **`src-tauri/src/commands/ollama.rs`** (router path when the wall-clock limit fires).
   - Substrings **`turn wall-clock timeout`** and **`closing output gate and running cleanup`** — both appear inside the **same** `tracing::warn!` format string in **`src-tauri/src/commands/turn_lifecycle.rs`** (`finalize_turn_timeout`). **Expected:** the two `rg` lines in blocks **A1**/**A2**/**B** may report the **same line number** twice; that still counts as pass.
4. **`cargo check`** and **`cargo test`** for the **`mac_stats`** package succeed (exit **0**, zero failing tests). The Cargo **package** name is **`mac_stats`** (underscore), declared in **`src-tauri/Cargo.toml`**; pass **`-p mac_stats`** whenever you use **`--manifest-path src-tauri/Cargo.toml`**. Equivalent ways to satisfy this: run **Verification commands** block **A1**/**A2** (repo root + `--manifest-path src-tauri/Cargo.toml -p mac_stats`) or block **B** (cwd **`src-tauri/`** + `cargo check` / `cargo test` with **`-p mac_stats`** or default package). This project targets **macOS**; use a Mac so results match maintainer expectations. Linux CI or a non-macOS checkout may fail link steps or skip platform tests — that mismatch is **not** a product failure; rerun on macOS.

## Testing instructions

### Tester quick gate (read first)

1. **Queue file per [`003-tester/TESTER.md`](../003-tester/TESTER.md):** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only. The rename chain is **`UNTESTED-…` → `TESTING-…` → (`CLOSED-…` or `WIP-…`)**. There is **no** approved **`TESTPLAN-…` → `TESTING-…`** shortcut.
2. **If the filename starts with `TESTPLAN-…`:** instructions are **under coder repair**. Testers **must not** run verification or rename to **`TESTING-…`**. Wait until a coder publishes **`UNTESTED-…`** (same stamp + slug).
3. **Inventory (optional sanity check):** from mac-stats repo root,  
   `ls tasks/*20260321-2000*openclaw-hung-turn-timeout-event-gate.md 2>/dev/null || true`  
   For a normal queued run you should see **`UNTESTED-…`** (and may also see **`CLOSED-…`** as history). If **only** **`CLOSED-…`** appears, **stop** — restore or fetch **`UNTESTED-…`**; do **not** treat **`CLOSED-…`** as the queue file.

### Operator filename (`003-tester/TESTER.md`)

- **Executable queue file for a real run:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only. At run start, rename **`UNTESTED-` → `TESTING-`**, run **Verification commands**, then apply outcome naming per **TESTER.md**.
- **Missing `UNTESTED-…` at repo tip:** **Stop.** Do **not** verify from **`CLOSED-…`** alone or invent **`TESTING-…`** from **`CLOSED-…`** unless your operator runbook explicitly allows it. Sync/pull for **`UNTESTED-…`**, or return a **queue / handoff defect** to the coder.
- **Emit `TESTPLAN-` only for instruction or environment-spec defects** (wrong paths, wrong queue file, ambiguous `cargo` cwd). Do **not** use **`TESTPLAN-`** because **`rg`** on top-level **`src/`** returns no matches — that is a **tester path mistake**, not a bad test plan (see **Two different directories named `src`** below).

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

**Current handoff:** After coder repair, the queue file **must** be named exactly **`tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`**.

### Shell compatibility

- The blocks below use **`bash`** syntax (`set -e`, `$(…)`). On macOS, **Terminal.app** defaults to **zsh**, which understands these snippets as written.
- If your login shell is **fish** (or another non-POSIX shell), run the block explicitly with Bash, for example:  
  `bash -lc 'set -e; REPO_ROOT="$(git rev-parse --show-toplevel)"; …'`  
  or paste the block after running **`bash`** interactively. **Do not** run the same script verbatim in **fish**; `set -e` and `$(…)` differ.

### Environment

- **Repository:** **mac-stats** only (directory that contains **`src-tauri/Cargo.toml`**, plus top-level `src/` and `src-tauri/`). There is **no** workspace **`Cargo.toml`** at the repository root — the Rust package is **`mac_stats`** under **`src-tauri/`** only.
- **Host:** **macOS** + stable **Rust** (`cargo` / `rustc` on `PATH`) + **[ripgrep](https://github.com/BurntSushi/ripgrep)** (`rg` on `PATH`). If `rg` is missing, install it or use your editor’s search; the patterns below are the exact substrings to find.
- **Preferred `cargo` cwd (blocks **A1** / **A2**):** stay at **repo root** and use **`cargo … --manifest-path src-tauri/Cargo.toml -p mac_stats`**. That avoids the common mistake of running **`cargo test`** from repo root **without** a manifest (Cargo errors or wrong package) and avoids relying on a subshell **`cd src-tauri`**.
- **Alternate `cargo` cwd (block **B**):** **`src-tauri/`** (crate root). There, use **`cargo check -p mac_stats`** / **`cargo test -p mac_stats`** (or plain **`cargo check`** / **`cargo test`** since this directory is a single-package manifest).
- **Wrong repo:** If `git rev-parse` / `test -f src-tauri/Cargo.toml` fails, stop — fix cwd before **`cargo`** or **`rg`** paths that assume repo root.
- **Typo trap:** the **repo folder** is often **`mac-stats`** (hyphen). The **Cargo package** is **`mac_stats`** (underscore). Do not drop **`-p mac_stats`** when using **`--manifest-path`** from repo root.
- **If `cargo` prints `could not find Cargo.toml`:** you ran **`cargo`** from the **repo root** without **`--manifest-path src-tauri/Cargo.toml`**, or your cwd is not the mac-stats tree. Use block **A1**/**A2** or **`cd src-tauri`** and block **B**.

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
3. **Fish (or non-bash) pasted script** — `set -e` / `$(…)` differ; use **`bash -lc '…'`** or run **zsh** with the block as written.
4. **Treating `CLOSED-…` verification snippets as authoritative** — historical reports may use wrong paths; follow **this** file’s **Verification commands** only.
5. **Using the `git rev-parse` block when `.git` is missing** — use **Verification commands → A2** (full no-git block), not a partial edit of **A1**.
6. **`rg: command not found`** — install [ripgrep](https://github.com/BurntSushi/ripgrep) or search your editor for the **exact** substrings under **`src-tauri/src/`**; the acceptance literals must still be located in the files named in criteria 3.

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

### Optional runtime check

To see log lines in a real run, reproduce or simulate a turn timeout and grep **`~/.mac-stats/debug.log`** for the same substrings. This is **not** required if static `rg` + `cargo` checks pass.

## Verification commands

Paste **one** complete block (**A1**, **A2**, or **B**) in a single shot. Do **not** split a block halfway; **`set -e`** must stop the script if cwd or paths are wrong.

### A1 — Recommended: repo root, **with** `.git`

Use **bash** or **zsh** (or `bash -lc '…'` from fish). **`set -e`** stops on the first failing command — do **not** append `|| true` to `cd` or `git rev-parse` here; a wrong directory must **fail** the script.

**Git prerequisite:** Run from **inside** the mac-stats clone so `git rev-parse --show-toplevel` prints the mac-stats root (the folder that directly contains `src-tauri/`). Starting in `src-tauri/` is OK: git still returns the repo root.

**Why `--manifest-path`:** mac-stats has **no** `Cargo.toml` at repo root. Invoking **`cargo test`** from root without **`--manifest-path`** is a frequent false failure; the lines below pin the **`mac_stats`** package explicitly.

```bash
set -e
REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"
test -f src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml -p mac_stats
cargo test --manifest-path src-tauri/Cargo.toml -p mac_stats

rg -n "TurnOutputGate|gate_allows_send|finalize_turn_timeout" src-tauri/src

rg -n -F "**Turn timed out**" src-tauri/src/commands/turn_lifecycle.rs

rg -n "closing output gate after turn wall-clock timeout" src-tauri/src/commands/ollama.rs

rg -n "turn wall-clock timeout" src-tauri/src/commands/turn_lifecycle.rs
rg -n "closing output gate and running cleanup" src-tauri/src/commands/turn_lifecycle.rs
```

### A2 — Recommended: repo root, **no** `.git` (tarball / export)

Use this when **`git rev-parse` fails** or the tree has no `.git` directory. Edit the **`cd`** line to your mac-stats root (the directory that contains **`src-tauri/`**).

```bash
set -e
cd /path/to/mac-stats
test -f src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml -p mac_stats
cargo test --manifest-path src-tauri/Cargo.toml -p mac_stats

rg -n "TurnOutputGate|gate_allows_send|finalize_turn_timeout" src-tauri/src

rg -n -F "**Turn timed out**" src-tauri/src/commands/turn_lifecycle.rs

rg -n "closing output gate after turn wall-clock timeout" src-tauri/src/commands/ollama.rs

rg -n "turn wall-clock timeout" src-tauri/src/commands/turn_lifecycle.rs
rg -n "closing output gate and running cleanup" src-tauri/src/commands/turn_lifecycle.rs
```

**Alternate (equivalent) cargo one-liner** (after you are at repo root):  
`( cd src-tauri && cargo check -p mac_stats && cargo test -p mac_stats )` — same package as **`--manifest-path`** above.

**Why two files for log strings:** The router line with **`closing output gate after`** is only in **`ollama.rs`**. The **`turn wall-clock timeout`** / **`closing output gate and running cleanup`** pair is in **`turn_lifecycle.rs`** inside **one** format string — **both `rg` commands may print the same line** (same line number twice). Optional single check:  
`rg -n "turn wall-clock timeout|closing output gate and running cleanup" src-tauri/src/commands/turn_lifecycle.rs`  
You should see **one** line containing both substrings. A single broad `rg` over **`src-tauri/src`** also works; the file-scoped lines above make expected locations obvious.

**Runtime:** `cargo test` for this crate can take several minutes on first run (compilation + tests). A long compile is **not** a hang.

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

rg -n -F "**Turn timed out**" src/commands/turn_lifecycle.rs

rg -n "closing output gate after turn wall-clock timeout" src/commands/ollama.rs

rg -n "turn wall-clock timeout" src/commands/turn_lifecycle.rs
rg -n "closing output gate and running cleanup" src/commands/turn_lifecycle.rs
```

**Do not** mix **A** and **B** path styles in one shell session: from repo root, **never** pass bare `src/` to `rg` for this task (that is the frontend tree). From **`src-tauri/`**, **never** pass `src-tauri/src` (there is no such path below the crate root).

**Slow tests:** `cargo test --no-fail-fast` (with the same **`-p mac_stats`** / **`--manifest-path`** as your block) is fine; requirement is **zero failing tests** for this crate.

## Test report

_(Tester: append results only on the **queue** file **`UNTESTED-…` → `TESTING-…`** per [`003-tester/TESTER.md`](../003-tester/TESTER.md). If the file is still **`TESTPLAN-…`**, that is coder repair — no test report yet.)_
