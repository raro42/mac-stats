# OpenClaw: hung turn wall-clock timeout + output event gate

Full-turn wall-clock timeout stops a hung agent run: output gate closes (no Discord status/draft/ATTACH spam), user-visible **Turn timed out** reply, optional `about:blank` cleanup only if the timed-out `request_id` still owns the coordination slot.

**Scope (read this first):** The words “OpenClaw” / “agent router” in the title describe **product behavior** that is implemented in **this repository (mac-stats)**, not in the sibling checkout at `../openclaw`. For verification you only search and build **mac-stats**. Searching `../openclaw` or expecting symbols there will fail and is **out of scope** for this task.

**“Event gate”** here means **`TurnOutputGate`** in Rust (`src-tauri/src/commands/turn_lifecycle.rs`): a shared flag the tool loop consults so outbound status/drafts stop after a turn timeout.

## Acceptance criteria

1. `TurnOutputGate` is defined as `pub type TurnOutputGate = Arc<AtomicBool>` in `commands/turn_lifecycle.rs`. The tool loop (`commands/tool_loop.rs` and related paths) calls `gate_allows_send` so sends are suppressed after the gate is closed.
2. `finalize_turn_timeout` in `commands/turn_lifecycle.rs` returns `OllamaReply` whose `text` starts with `**Turn timed out**` and includes the budget in seconds (see the `format!` that builds the user message).
3. **Log strings (static check):** The following literals appear in **mac-stats** Rust sources as written below (copy/paste safe; use the verification `rg` commands). A live Discord timeout repro is **optional**, not required for pass.
   - Substring **`closing output gate after turn wall-clock timeout`** — in **`src-tauri/src/commands/ollama.rs`** (router path when the wall-clock limit fires).
   - Substrings **`turn wall-clock timeout`** and **`closing output gate and running cleanup`** — both appear inside the **same** `tracing::warn!` format string in **`src-tauri/src/commands/turn_lifecycle.rs`** (`finalize_turn_timeout`). **Expected:** the two `rg` lines in block **A**/**B** may report the **same line number** twice; that still counts as pass.
4. `cargo check` and `cargo test` run from **`src-tauri/`** succeed on the verification host (this project targets **macOS**; use a Mac for verification so results match maintainer expectations).

## Testing instructions

### Which task file to follow (avoid defective copy-paste)

- **Follow this document’s verification blocks only.** Identity is **stamp `20260321-2000`** + slug **`openclaw-hung-turn-timeout-event-gate`**, regardless of whether the filename prefix is `TESTPLAN-`, `UNTESTED-`, or `TESTING-`.
- **`tasks/CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` is not a spec.** It is an **append-only archive** of old test reports. Some appended commands used **`rg … src/`** (top-level frontend), which is **wrong** for this feature and produces **false failures**. Do **not** copy verification commands from `CLOSED-*`; use the blocks under **Verification commands** in **this** file.
- **Tester workflow (`003-tester/TESTER.md`):** The step **rename `UNTESTED-…` → `TESTING-…`** applies to the **queue file**. If the queue still shows **`TESTPLAN-…`**, treat that as **instructions not yet handed back**: ask the coder to finish by renaming **`TESTPLAN-…` → `UNTESTED-…`** (same stamp + slug) before you start testing.

### Operator queue / filename (handoff)

- **Ready for retest:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`.
- **Under instruction repair:** `tasks/TESTPLAN-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` (same body; rename to `UNTESTED-…` when the coder is done).

### Shell compatibility

- The blocks below use **`bash`** syntax (`set -e`, `$(…)`). On macOS, **Terminal.app** defaults to **zsh**, which understands these snippets as written.
- If your login shell is **fish** (or another non-POSIX shell), run the block explicitly with Bash, for example:  
  `bash -lc 'set -e; REPO_ROOT="$(git rev-parse --show-toplevel)"; …'`  
  or paste the block after running **`bash`** interactively. **Do not** run the same script verbatim in **fish**; `set -e` and `$(…)` differ.

### Environment

- **Repository:** **mac-stats** only (directory that contains **`src-tauri/Cargo.toml`**, plus top-level `src/` and `src-tauri/`).
- **Host:** **macOS** + stable **Rust** toolchain + **[ripgrep](https://github.com/BurntSushi/ripgrep)** (`rg` on `PATH`). If `rg` is missing, install it or use your editor’s search; the patterns below are the exact substrings to find.
- **Working directory:** You must end up with **`src-tauri/Cargo.toml`** resolvable from your chosen cwd (see blocks **A** and **B** below). Do **not** assume a successful run if you never confirmed that path exists.

### Two different directories named `src` (critical)

| Path from repo root | What it is |
|---------------------|------------|
| **`src/`** | Frontend (HTML/JS/CSS). **No** `TurnOutputGate` / turn-timeout Rust strings here. |
| **`src-tauri/src/`** | Rust crate (**all** static checks for this task). |

**Common false failure:** From repo root, running `rg "TurnOutputGate" src` searches **only** the frontend tree and prints **no matches**. That does **not** mean the feature is missing — you searched the wrong tree. Always use **`src-tauri/src`** in path arguments when your shell’s cwd is the **repo root**.

### What *not* to do

- Do **not** treat zero matches under top-level **`src/`** as a failure.
- Do **not** verify in **`../openclaw`** or any other repo.

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
| `cargo check` + `cargo test` in `src-tauri/` | Exit **0**; **zero** failing tests. |
| `rg` for gate symbols | At least one match for each **distinct** pattern in block **A** or **B** (see paths for your cwd). For **`turn_lifecycle.rs`**, the two log-string `rg` lines may both hit the **same** source line — that is still pass. |
| Top-level `src/` | May show **no** matches for Rust gate strings — **not** a failure. |

### Optional runtime check

To see log lines in a real run, reproduce or simulate a turn timeout and grep **`~/.mac-stats/debug.log`** for the same substrings. This is **not** required if static `rg` + `cargo` checks pass.

## Verification commands

### A — Recommended: repo root as cwd

Use **bash** or **zsh** (or `bash -lc '…'` from fish). Copy the whole block. **`set -e`** stops on the first failing command — do **not** append `|| true` to `cd` or `git rev-parse` here; a wrong directory must **fail** the script.

```bash
set -e
REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"
test -f src-tauri/Cargo.toml
( cd src-tauri && cargo check && cargo test )

rg -n "TurnOutputGate|gate_allows_send|finalize_turn_timeout" src-tauri/src

rg -n '\*\*Turn timed out\*\*' src-tauri/src/commands/turn_lifecycle.rs

rg -n "closing output gate after turn wall-clock timeout" src-tauri/src/commands/ollama.rs

rg -n "turn wall-clock timeout" src-tauri/src/commands/turn_lifecycle.rs
rg -n "closing output gate and running cleanup" src-tauri/src/commands/turn_lifecycle.rs
```

**Not in a git checkout?** Replace the first three lines with a manual `cd /path/to/mac-stats` (the directory containing `src-tauri/`), then run from `test -f src-tauri/Cargo.toml` onward.

**Why two files for log strings:** The router line with **`closing output gate after`** is only in **`ollama.rs`**. The **`turn wall-clock timeout`** / **`closing output gate and running cleanup`** pair is in **`turn_lifecycle.rs`** inside **one** format string — **both `rg` commands may print the same line** (same line number twice). Optional single check:  
`rg -n "turn wall-clock timeout|closing output gate and running cleanup" src-tauri/src/commands/turn_lifecycle.rs`  
You should see **one** line containing both substrings. A single broad `rg` over **`src-tauri/src`** also works; the file-scoped lines above make expected locations obvious.

**Runtime:** `cargo test` for this crate can take several minutes on first run (compilation + tests).

### B — Alternate: your cwd is already `src-tauri/` (crate root)

Use this block **only** when `pwd` is the directory that contains **`Cargo.toml`** and a **`src/`** subdirectory (that **`src/`** is the **Rust crate source**, not the repo’s top-level frontend **`src/`**). Quick sanity check before `rg`: **`test -f src/commands/turn_lifecycle.rs`** must succeed; if it fails, you are not in `src-tauri/`.

```bash
set -e
test -f Cargo.toml
test -f src/commands/turn_lifecycle.rs
cargo check && cargo test

rg -n "TurnOutputGate|gate_allows_send|finalize_turn_timeout" src

rg -n '\*\*Turn timed out\*\*' src/commands/turn_lifecycle.rs

rg -n "closing output gate after turn wall-clock timeout" src/commands/ollama.rs

rg -n "turn wall-clock timeout" src/commands/turn_lifecycle.rs
rg -n "closing output gate and running cleanup" src/commands/turn_lifecycle.rs
```

**Do not** mix A and B path styles in one shell session: from repo root, **never** pass bare `src/` to `rg` for this task (that is the frontend tree). From **`src-tauri/`**, **never** pass `src-tauri/src` (there is no such path below the crate root).

**Slow tests:** `cargo test --no-fail-fast` is fine; requirement is **zero failing tests** for this crate.

## Test report

_(Tester: append results after `UNTESTED` → `TESTING` per `003-tester/TESTER.md`.)_
