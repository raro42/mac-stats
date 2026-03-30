# OpenClaw: hung turn wall-clock timeout + output event gate

Full-turn wall-clock timeout stops a hung agent run: output gate closes (no Discord status/draft/ATTACH spam), user-visible **Turn timed out** reply, optional `about:blank` cleanup only if the timed-out `request_id` still owns the coordination slot.

**Scope (read this first):** The words “OpenClaw” / “agent router” in the title describe **product behavior** that is implemented in **this repository (mac-stats)**, not in the sibling checkout at `../openclaw`. For verification you only search and build **mac-stats**. Searching `../openclaw` or expecting symbols there will fail and is **out of scope** for this task.

**“Event gate”** here means **`TurnOutputGate`** in Rust (`src-tauri/src/commands/turn_lifecycle.rs`): a shared flag the tool loop consults so outbound status/drafts stop after a turn timeout.

## Acceptance criteria

1. `TurnOutputGate` is defined as `pub type TurnOutputGate = Arc<AtomicBool>` in `commands/turn_lifecycle.rs`. The tool loop (`commands/tool_loop.rs` and related paths) calls `gate_allows_send` so sends are suppressed after the gate is closed.
2. `finalize_turn_timeout` in `commands/turn_lifecycle.rs` returns `OllamaReply` whose `text` starts with `**Turn timed out**` and includes the budget in seconds (see the `format!` that builds the user message).
3. **Log strings (static check):** The following literals appear in **mac-stats** Rust sources as written below (copy/paste safe; use the verification `rg` commands). A live Discord timeout repro is **optional**, not required for pass.
   - Substring **`closing output gate after turn wall-clock timeout`** — emitted from **`src-tauri/src/commands/ollama.rs`** (router path when the wall-clock limit fires).
   - Substrings **`turn wall-clock timeout`** and **`closing output gate and running cleanup`** — both appear in the **same** warning format string in **`src-tauri/src/commands/turn_lifecycle.rs`** (`finalize_turn_timeout`).
4. `cargo check` and `cargo test` run from **`src-tauri/`** succeed on the verification host (this project targets **macOS**; use a Mac for verification so results match maintainer expectations).

## Testing instructions

### Operator queue / filename (avoid handoff confusion)

- **This slug’s retest queue file in-repo:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` (the file you are reading after a **TESTPLAN → UNTESTED** rename).
- **If that path is missing** but you have **`CLOSED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** or **`TESTPLAN-…`** for the same stamp/slug, follow **`003-tester/TESTER.md`** for renames — **the verification shell commands below do not depend** on which of those filenames you started from. Missing **`UNTESTED-…`** alone is a **queue/handoff** issue, not evidence that the Rust gate is absent.

### Environment

- **Repository:** **mac-stats** only (directory that contains **`src-tauri/Cargo.toml`**, plus top-level `src/` and `src-tauri/`).
- **Host:** **macOS** + stable **Rust** toolchain + **[ripgrep](https://github.com/BurntSushi/ripgrep)** (`rg` on `PATH`). If `rg` is missing, install it or use your editor’s search; the patterns below are the exact substrings to find.
- **Working directory:** Open a terminal and **`cd` to the mac-stats repo root** (the folder that contains `src-tauri/`). The **primary** verification block below assumes **current working directory = that repo root**.

### Two different directories named `src` (critical)

| Path from repo root | What it is |
|---------------------|------------|
| **`src/`** | Frontend (HTML/JS/CSS). **No** `TurnOutputGate` / turn-timeout Rust strings here. |
| **`src-tauri/src/`** | Rust crate (**all** static checks for this task). |

**Common false failure:** From repo root, running `rg "TurnOutputGate" src` searches **only** the frontend tree and prints **no matches**. That does **not** mean the feature is missing — you searched the wrong tree. Always use **`src-tauri/src`** in path arguments when your shell’s cwd is the **repo root**.

### What *not* to do

- Do **not** treat zero matches under top-level **`src/`** as a failure.
- Do **not** verify in **`../openclaw`** or any other repo.

### Preflight

From **repo root**:

```bash
test -f src-tauri/Cargo.toml && echo "OK: repo root"
test -f src-tauri/src/commands/turn_lifecycle.rs && echo "OK: Rust tree"
command -v rg >/dev/null && echo "OK: rg" || echo "WARN: install ripgrep or search manually"
```

### Optional runtime check

To see log lines in a real run, reproduce or simulate a turn timeout and grep **`~/.mac-stats/debug.log`** for the same substrings. This is **not** required if static `rg` + `cargo` checks pass.

## Verification commands

### A — Recommended: repo root as cwd

Shell: **bash** or **zsh**. Copy the whole block; `set -e` stops on the first failing command.

```bash
set -e
cd "$(git rev-parse --show-toplevel 2>/dev/null)" || true
test -f src-tauri/Cargo.toml
( cd src-tauri && cargo check && cargo test )

rg -n "TurnOutputGate|gate_allows_send|finalize_turn_timeout" src-tauri/src

rg -n '\*\*Turn timed out\*\*' src-tauri/src/commands/turn_lifecycle.rs

rg -n "closing output gate after turn wall-clock timeout" src-tauri/src/commands/ollama.rs

rg -n "turn wall-clock timeout" src-tauri/src/commands/turn_lifecycle.rs
rg -n "closing output gate and running cleanup" src-tauri/src/commands/turn_lifecycle.rs
```

**Why two files for log strings:** The router line with **`closing output gate after`** is only in **`ollama.rs`**. The **`turn wall-clock timeout`** / **`closing output gate and running cleanup`** pair lives in **`turn_lifecycle.rs`**. A single `rg` over **`src-tauri/src`** also works; the file-scoped lines above make expected locations obvious.

### B — Alternate: your cwd is already `src-tauri/` (crate root)

Use this block **only** when `pwd` is the directory that contains **`Cargo.toml`** and a **`src/`** subdirectory (that **`src/`** is **Rust**, not the repo’s top-level frontend `src/`).

```bash
set -e
test -f Cargo.toml
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
