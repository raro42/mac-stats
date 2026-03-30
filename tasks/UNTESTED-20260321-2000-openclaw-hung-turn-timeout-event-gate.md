# OpenClaw: hung turn wall-clock timeout + output event gate

**Filename = queue state:** The **leading prefix** on this file (`TESTPLAN-…` vs `UNTESTED-…`) is authoritative — read the **basename**, not the IDE tab title.

- **If the basename starts with `TESTPLAN-…`:** **Coder repair / instruction draft.** **Testers:** **do not** run [`003-tester/TESTER.md`](../003-tester/TESTER.md), **do not** rename to **`TESTING-…`**, **do not** run **Verification commands** as the queued task — wait until the coder renames this file to **`UNTESTED-…`** (same stamp + slug).
- **If the basename starts with `UNTESTED-…`:** **Tester queue.** Follow **TESTER.md**: rename **`UNTESTED-…` → `TESTING-…`** at run start, then run **Verification commands** below.

Sections that refer to the “queue file” mean **`UNTESTED-…`** once that filename exists on disk.

**On-disk name (this file):** Same stamp `20260321-2000` and slug `openclaw-hung-turn-timeout-event-gate`; the **leading prefix** is what matters:

- **`tasks/TESTPLAN-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** — **not** on the tester queue; coder is revising **Testing instructions** / clarity only. Testers **do not** start [`003-tester/TESTER.md`](../003-tester/TESTER.md) until the coder renames **`TESTPLAN-…` → `UNTESTED-…`** (same stamp + slug).
- **`tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** — **tester queue** (ready to run). Follow **TESTER.md**: **`UNTESTED-…` → `TESTING-…`** at run start.

**No** mac-stats product code change is required for this task file.

**Instruction revision note:** A prior run flagged **Testing instructions** or the **stated environment** as defective (not a mac-stats implementation failure). This markdown body is the only authoritative spec: follow **Verification commands** below exactly, not snippets copied from **`CLOSED-*`** history (those may use wrong paths such as top-level **`src/`**). **Latest repair (2026-03-30, eighth pass — coder):** Added **Execution recipe (mandatory order)** with explicit **BLOCK: A1/A2** vs **BLOCK: B** flows; stated that **Preflight never `cd`s** your shell (so **`pwd` after step 1 is unchanged**); warned **not** to **`cd` to `$REPO_ROOT` after Preflight when the probe said `BLOCK: B`** (block **B** must run from **`src-tauri/`** crate root); fixed **Before step 2 (A1 or A2)** so it matches **A1**’s walk-up from subdirs (e.g. **`tasks/`**); **Common instruction defects** item **13** (`BLOCK: B` + mistaken **`cd` to repo root**). **Prior (seventh pass):** **Fast path** / **TL;DR** aligned with **0 → 0b → 1 → 2**; **Preflight mandatory** including **`BLOCK: B`**. **Prior (sixth pass):** **BLOCK: B + no-git Preflight** trap; decision table; **Common instruction defects** item **11**.

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

**Invariant: step 1 always —** After **Tester quick gate** (step **0**), you **must** run **Preflight** (step **1**) once before **Verification commands** (step **2**), **for every** printed **BLOCK** (**A1**/**A2** **or** **B**). **Do not** paste block **A1**, **A2**, or **B** immediately after the probe, even if a summary section says “experienced testers.” Skipping Preflight is an **invalid** run.

**Preflight does not change directory:** Neither Preflight snippet runs **`cd`**. After step **1**, your shell’s **`pwd`** is the **same** as before Preflight unless **you** ran a **`cd`** in step **0b**. Block **A1**/**A2** performs the repo-root **`cd`** inside the pasted block; block **B** assumes you are **still** at **`src-tauri/`** crate root after step **0**/**1**.

### Execution recipe (mandatory order)

Use this subsection when the longer text feels contradictory. It restates **Copy-paste order** (**0 → 0b → 1 → 2**) only.

**When step 0 prints `BLOCK: A1 or A2` (you will use verification block A1 or A2):**

1. **0** — Paste **Tester quick gate** (probe).  
2. **0b** — If the probe printed **`cd '…'`**, run that **`cd`**, then run **`test -f src-tauri/Cargo.toml && echo OK`**. If the probe said you are **already** at repo root (no **`cd`** line), still run **`test -f src-tauri/Cargo.toml && echo OK`** from **`pwd`** before step **2**.  
3. **1** — Paste **Preflight**: **git** variant (works from repo root or any subdir under mac-stats, including **`tasks/`**) **or** **no-git** variant (**only** when **`pwd`** is mac-stats **repo root** — **never** from **`src-tauri/`** alone).  
4. **2** — Paste **one** full block **A1** *or* **A2** from **Verification commands** (from **`set -e`** through the **last `rg`**). **Do not** skip the opening lines: they **`cd`** to repo root even if Preflight already printed **`$REPO_ROOT`**. **If `pwd` is still `tasks/`** after Preflight, you **must** either have run step **0b**’s **`cd`** or rely on block **A1**’s walk-up + **`cd "$REPO_ROOT"`** — **do not** run **`cargo --manifest-path src-tauri/Cargo.toml`** from **`tasks/`** without that **`cd`**.

**When step 0 prints `BLOCK: B` (you will use verification block B):**

1. **0** — Paste **Tester quick gate** (probe). Your **`pwd`** should be **`src-tauri/`** crate root.  
2. **0b** — **Skip** the repo-root check **`test -f src-tauri/Cargo.toml`** (from crate root it fails **by design**; see **Copy-paste order** step **0b**).  
3. **1** — Paste **git** Preflight **only** (walk-up still finds mac-stats root for **`test -f`**). **Do not** use the **no-git** Preflight while **`pwd`** is **`src-tauri/`**.  
4. **2** — Paste block **B** in full **without** leaving **`src-tauri/`**. **Do not** **`cd` to `$REPO_ROOT` or the path Preflight printed** before block **B** — block **B**’s **`rg`** paths are **crate-relative** (`src/commands/…`), not repo-root **`src-tauri/src/…`**.

### At a glance (shell order — read this first)

This list is the **same** sequence as **Copy-paste order**; use it when the longer sections feel ambiguous.

- **Step 0 — Directory probe:** Paste the **bash** script under **Tester quick gate** (the fenced block). **No** `set -e` in this probe. It prints **BLOCK: …** — that chooses **A1**/**A2** vs **B** for step **2** only.
- **Step 0b — Align `pwd` (when the probe prints a `cd …` line):** Run that **`cd`**, then run **`test -f src-tauri/Cargo.toml && echo OK`**. When the probe says **BLOCK: B**, skip **0b**’s repo-root test — you will run block **B** in step **2** from **`src-tauri/`** (crate root), where **`test -f src-tauri/Cargo.toml`** is **wrong** (that path would mean `src-tauri/src-tauri/…`).
- **Step 1 — Preflight:** Paste **exactly one** of the two snippets under **Preflight (required)** (git variant **or** no-git variant). Each snippet starts with **`set -e`**. This is **not** verification block **A2** (Preflight has **no** `cargo` / `rg`). **If step 0 printed `BLOCK: B`** (cwd is **`src-tauri/`**), use **only the git Preflight** — **do not** use the no-git snippet until **`pwd`** is mac-stats **repo root** (see **Preflight (required)**).
- **Step 2 — Verification:** Paste **exactly one** complete block **A1**, **A2**, or **B** from **Verification commands**, from the first **`set -e`** through the **last `rg`**, **without** re-running Preflight inside that paste.

**Before step 2 (A1 or A2 only):** **`test -f src-tauri/Cargo.toml`** should succeed **from `pwd`** after step **0b** (repo root) **or** you paste **A1** in full from a **subdirectory** of mac-stats — **A1** walks **up** from **`$PWD`** to locate **`REPO_ROOT`** and then **`cd`s**. **A2** still needs its opening **`cd /ABSOLUTE/.../mac-stats`** to be correct (subdirs like **`tasks/`** are fine **if** that **`cd`** lands on repo root). If **`pwd`** is **`…/mac-stats/src-tauri`** (crate root), you are **not** on repo root: **`cd ..`** for **A1**/**A2**, or stay put and use block **B**.

**Before step 2 (block B only):** Your **`pwd`** must be the **`src-tauri/`** crate root — where **`test -f Cargo.toml`** and **`test -f src/commands/turn_lifecycle.rs`** both succeed.

**Cursor / VS Code:** The integrated terminal often starts in the **workspace root**. If the workspace is a **parent** folder (monorepo) or a different clone path, **`pwd`** may **not** be mac-stats until you **`cd`** — run step **0** in **that** terminal and follow the printed **`cd`**.

### Start here (read before any shell)

- **Repository:** All commands target **this checkout: mac-stats** (the tree that contains **`src-tauri/Cargo.toml`**). The title says “OpenClaw” for **product behavior** only — **do not** `cd` to **`../openclaw`**, **do not** search or build there; that is **out of scope** and will fail.
- **Queue:** If the task file basename is **`TESTPLAN-…`**, you are **not** on the tester queue — **stop** (see header above). If it is **`UNTESTED-…`**, follow **TESTER.md** rename rules, then run the shell sequence below in **one** terminal session (same shell for **0 → 0b → 1 → 2**).
- **Single source of truth for shell order:** Only the **Copy-paste order** table defines steps **0**, **0b**, **1**, **2**. Lists labeled “TL;DR”, “Before you run anything”, “At a glance”, or “Minimal run order” are **checklists or narrative** — they are **not** extra numbered bash steps after **2**.

**Canonical step numbers:** Only the **Copy-paste order** table (**0**, **0b**, **1**, **2**) defines the command sequence. Other sections use bullets or headings — they are explanations or policy, **not** additional numbered steps to interleave with **0**/**1**/**2**.

### Copy-paste order (mandatory)

Run these **in order** in **one terminal** (bash or zsh per **Shell compatibility**). Skip nothing between step **0** and the end of your chosen block.

| Step | What to run |
|------|-------------|
| **0** | **Tester quick gate** probe (below) — **no** `set -e` required. |
| **0b** | If the probe prints **`cd '…'`**, run that **`cd`**, then **`test -f src-tauri/Cargo.toml && echo OK`** (required before **A1**/**A2** in step **2**). If probe says **BLOCK: B**, you will paste block **B** from **`src-tauri/`** in step **2** — **skip** this repo-root **`test -f src-tauri/…`** (from crate root it fails by design). |
| **1** | **Preflight (required)** — **git** variant (from any directory under mac-stats, **including `src-tauri/`** when step **0** said **`BLOCK: B`**) **or** **no-git** variant (**only** when **`pwd`** is mac-stats **repo root** — **never** from **`src-tauri/`** alone, because **`test -f src-tauri/Cargo.toml`** fails there). |
| **2** | Paste **exactly one** of **Verification commands → A1**, **A2**, or **B** from the first **`set -e`** through the **last `rg`**. **Do not** insert **Preflight** again inside this paste — step **1** already ran it once. |

**Naming note:** **Block A2** (verification) always includes **`cd /ABSOLUTE/PATH/TO/mac-stats`** — you edit that line. The **no-git Preflight** is only two `test -f` lines plus `echo`; it does **not** replace block **A2**.

### Probe → Preflight → verification (decision table)

Use this when **Copy-paste order** step **0** is done and you are choosing step **1** + step **2**.

| Step **0** printed | Your cwd after step **0b** (if any) | Preflight (step **1**) | Verification (step **2**) |
|--------------------|-------------------------------------|-------------------------|---------------------------|
| **`BLOCK: B`** | **`src-tauri/`** (crate root) — step **0b** skipped | **Git variant** only (walk-up resolves `$REPO_ROOT`; **Preflight does not `cd` you** — **stay** at crate root; **do not** `cd` into `$REPO_ROOT` before block **B**) | Paste block **B** **without** leaving **`src-tauri/`** (you should still be at crate root). |
| **`BLOCK: A1 or A2`** (at repo root) | Repo root | **Git** or **no-git** | **A1** or **A2** (one paste). |
| **`BLOCK: A1 or A2`** (subdir; you ran printed **`cd`**) | Repo root | **Git** or **no-git** | **A1** or **A2** (one paste). |

**Why `BLOCK: B` forbids no-git Preflight from crate root:** The no-git snippet runs **`test -f src-tauri/Cargo.toml`**, which expects a **`src-tauri/`** child directory. From **`…/mac-stats/src-tauri`**, that path does not exist — the snippet fails even though the checkout is valid.

### TL;DR (static gate)

**These six lines are a skimmable summary.** For the exact paste sequence, use **Copy-paste order** (**0**, **0b**, **1**, **2**) only.

1. **mac-stats** repo only — not `../openclaw`. Rust paths are under **`src-tauri/src/`** (from repo root) or **`src/`** (only when cwd is **`src-tauri/`** — block **B**).
2. **Terminal cwd:** In Cursor/VS Code (or any **multi-root** setup), confirm **`pwd`** is **inside** the mac-stats tree before step **0**. If **`pwd`** is only a **parent** monorepo folder, **`cd`** into **`…/mac-stats`** (the folder that **contains** **`src-tauri/`**) first — otherwise step **0** prints **`BLOCK: none`** even though the project is open in the IDE.
3. Run **Tester quick gate** step **0** (walk-up probe; **no** `git` required) → **`cd`** if told → **`test -f src-tauri/Cargo.toml`** before **A1**/**A2**.
4. Run **Preflight** once in the **same terminal** (**mandatory** after step **0**, including when step **0** said **`BLOCK: B`** — never go probe → block **B** with no Preflight): **git** variant (walk-up from any subdir, **required when step 0 said `BLOCK: B`**) **or** **no-git** variant (**only** when **`pwd`** is repo root — **not** from **`src-tauri/`**). Preflight is **not** a substitute for **Verification commands → A2**.
5. Paste **one** complete block **A1** *or* **A2** *or* **B** from the first **`set -e`** through the **last `rg`** — **no** line omitted, **no** mixing blocks, **no** running **`cargo`**/**`rg`** alone from an IDE snippet.
6. **Pass:** `cargo check` + `cargo test` exit **0**; every **`rg`** in that block prints **≥1 line** (for **`turn_lifecycle.rs`**, two `rg` lines may show the **same** line number — still pass).

### Before you run anything (read once)

**Note:** Rows **1–9** in this table are a **read-once checklist**, not the **Copy-paste order** steps (**0**, **0b**, **1** Preflight, **2** verification block).

| Step | Action |
|------|--------|
| 1 | **Host:** macOS with `cargo`, `rustc`, and `rg` on `PATH`. Linux-only or missing toolchain → stop, report **environment blocked** per **TESTER.md** (typically **`WIP-…`**), **not** **`TESTPLAN-…`**. |
| 2 | **Repo:** **`pwd`** (terminal cwd) is inside the **mac-stats** tree (eventually the folder that contains **`src-tauri/Cargo.toml`**). The IDE workspace root may be a **parent** monorepo — trust the terminal, not the window title. Do **not** search **`../openclaw`** or top-level **`src/`** for Rust gate strings. |
| 3 | **Shell:** Paste verification blocks in **`bash`** (or zsh with `set -e` behaving as documented). **fish** → use `bash -lc '…'`. |
| 4 | **Block choice:** Run **Tester quick gate** step **0** below; obey the printed **BLOCK:** (A1/A2 vs B). Never mix path styles from different blocks in one paste. |
| 5 | **A1 vs A2:** If the probe says **A1 or A2**, use **A1** whenever you have a **normal git clone of mac-stats** where **`git rev-parse --show-toplevel`** is **the same directory** as the mac-stats root (the folder that contains **`src-tauri/`**). Use **A2** when there is **no** `.git`, **`git rev-parse` fails**, or **git’s top-level is a parent monorepo** and you prefer a single explicit **`cd /ABSOLUTE/.../mac-stats`** line over **A1**’s automatic walk-up. **Do not** run **A1** and then **A2** in the same session — one successful block is enough. |
| 6 | **One paste:** Run **exactly one** of **A1**, **A2**, or **B** from **`set -e` through the last `rg`** without changing directory mid-block. |
| 7 | **After step 0 says `cd …`:** Run that **`cd`**, then **`test -f src-tauri/Cargo.toml`** (expect success) **before** pasting **A1** or **A2**. If that **`test`** fails, you are not at mac-stats **repo root** yet — fix **`cd`** and retry. |
| 8 | **`--manifest-path` vs Preflight:** **Preflight** uses **`$REPO_ROOT`** absolute paths, so it can succeed from **`tasks/`** or any subfolder. **`cargo --manifest-path src-tauri/Cargo.toml`** is resolved **relative to `pwd`**. If you run **only** the **`cargo`** / **`rg`** lines from **A1**/**A2** while cwd is still a subdirectory, Cargo looks for **`…/tasks/src-tauri/Cargo.toml`** (missing) → **false failure**. Always paste the **entire** block, including the **`cd`** that establishes repo root (or use **B** from **`src-tauri/`**). |
| 9 | **IDE / “run selection”:** Highlighting **`cargo`**…**`rg`** without the leading **`cd`**, **`REPO_ROOT=…`**, and **`test -f`** lines is an **invalid** run — same failure mode as step 8. |

**Fast path (experienced testers, macOS):** Same **0 → 0b → 1 → 2** as **Copy-paste order** — **no** shortcuts. **Tester quick gate** step **0** → step **0b** when **A1**/**A2** (repo-root **`test -f`**) → step **1** **Preflight** always (**git** variant if **`.git`** exists or when **`BLOCK: B`**; **no-git** only at repo root and **never** from **`src-tauri/`** alone) → step **2** paste **one** full block **A1**, **A2**, or **B** (never skip leading **`cd`** / **`test -f`** lines inside the block). **Tarball / broken git:** **A2** with a real absolute path on the **`cd`** line. **Pass** = every command exits **0** and every **`rg`** prints at least one line.

**Happy path (ordered):** **0** Tester quick gate → **0b** optional **`cd`** + **`test -f src-tauri/Cargo.toml`** (when using **A1**/**A2**) → **1** Preflight (**git** or **no-git** variant — not the same as verification block **A2**) → **2** one full block **A1** or **A2** or **B** → **after bash succeeds:** **Closure checklist** (**TESTER.md** paperwork only — **not** “step 3” of the shell table; **not** another bash paste). Use the checklist only when the queue file is **`UNTESTED-…`** and you already renamed **`UNTESTED-…` → `TESTING-…`** per **TESTER.md**.

### Coder publication after TESTPLAN repair

- If the task file on disk is **`tasks/TESTPLAN-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`**, fix **Testing instructions** / wording in that file, then **rename** it to **`tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** (same stamp `20260321-2000` and slug `openclaw-hung-turn-timeout-event-gate`). **Do not** change the stamp or slug.
- **Retest queue name is always `UNTESTED-…`**. Testers start from **`UNTESTED-…`**, not **`TESTPLAN-…`**.

### Tester quick gate (read first)

**This subsection is Copy-paste order step 0** (directory probe only — **not** Preflight, **not** block **A1**/**A2**/**B**).

**Pick the verification block** by running the script below **before** copying **A1**/**A2**/**B**. **No** `set -e` required. This probe **does not require `git`**: it detects **`src-tauri/`** crate root vs mac-stats **repo root** by walking **up** from **`pwd`** until it finds **`src-tauri/Cargo.toml`** + **`src-tauri/src/commands/turn_lifecycle.rs`**, or prints **BLOCK: none**. That fixes false **BLOCK: none** from **`tasks/`** and avoids assuming **`git rev-parse --show-toplevel`** equals the mac-stats folder (parent monorepos).

```bash
if test -f Cargo.toml && test -f src/commands/turn_lifecycle.rs; then
  echo "BLOCK: B (cwd is src-tauri/ crate root). Do not use A1/A2."
else
  d="$PWD"
  found=""
  while test "$d" != "/"; do
    if test -f "$d/src-tauri/Cargo.toml" && test -f "$d/src-tauri/src/commands/turn_lifecycle.rs"; then
      found="$d"
      break
    fi
    d=$(dirname "$d")
  done
  if test -n "$found"; then
    if test "$(cd "$found" && pwd -P)" = "$(pwd -P)"; then
      echo "BLOCK: A1 or A2 (cwd is mac-stats repo root). Do not use B."
    else
      echo "BLOCK: A1 or A2 — cwd is inside mac-stats but not repo root. Before pasting A1/A2 run: cd $(printf %q "$found")"
    fi
  else
    echo "BLOCK: none — cd into the mac-stats checkout (directory tree that contains src-tauri/), then run this probe again."
  fi
fi
```

If the script prints **BLOCK: none**, **`cd`** into the mac-stats tree and re-run step **0**. If it prints a **`cd '…'`** line, run that **`cd`** before pasting **A1** or **A2** — including when you believe you are “already” in mac-stats: **symlinks** and **`pwd -P`** can make the probe’s **`found`** path differ from your current **`pwd`**, and **`cargo --manifest-path src-tauri/Cargo.toml`** must run with **`pwd`** aligned to that **`found`** directory. Immediately after that **`cd`**, run **`test -f src-tauri/Cargo.toml && echo "OK: repo root"`**; if it fails, do **not** paste **A1**/**A2** yet.

**Why step 0 matters:** Relative paths in **A1**/**A2** (**`src-tauri/Cargo.toml`**, **`src-tauri/src/…`**) only work when **`pwd`** is the mac-stats **repo root**. From **`tasks/`**, **`docs/`**, or a **parent git** worktree, you must **`cd`** to the directory that **contains** **`src-tauri/`** (the probe’s **`found`** path) before **`cargo --manifest-path …`**.

**`BLOCK: A1 or A2` is not two blocks:** It means “you must end up at **repo root** before **`cargo --manifest-path src-tauri/Cargo.toml`**.” Use **A1** *or* **A2** to get there — whichever matches your tree (**git** vs **no git**), then stop.

#### Queue and environment policy (read only — not part of steps 0–2)

These bullets are **not** shell commands to paste after the probe. After the probe, your next **paste** is **Preflight** (**Copy-paste order** step **1**), then **one** block **A1**/**A2**/**B** (step **2**).

- **Queue file** ([`003-tester/TESTER.md`](../003-tester/TESTER.md)): when **`tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** exists, use that file only. The tester rename chain is **`UNTESTED-…` → `TESTING-…` → (`CLOSED-…` or `WIP-…`)**. Testers **must not** rename **`TESTPLAN-…` → `TESTING-…`** (wait for **`UNTESTED-…`** first). **While the only task file for this stamp is `TESTPLAN-…`**, there is **no** queue slot — **do not** start **TESTER.md** on **`TESTPLAN-…`**.

- **Host and toolchain:** Run on **macOS** with **`cargo`**, **`rustc`**, and **`rg`** on your `PATH`. Criterion **4** requires a full **`cargo check`** + **`cargo test`** for **`mac_stats`** to exit **0** on this platform. If you only have Linux (or CI images without the macOS toolchain), **stop**: append **environment blocked** to the test report and rename the queue file per [`003-tester/TESTER.md`](../003-tester/TESTER.md) (typically **`WIP-…`**). That is **not** a product failure and **not** a reason to bounce the task to **`TESTPLAN-…`**. The **TESTPLAN-** prefix is for bad *instructions* in this task file, not for missing macOS or toolchain.

- **Inventory (optional sanity check):** from mac-stats repo root,  
  `ls tasks/*20260321-2000*openclaw-hung-turn-timeout-event-gate.md 2>/dev/null || true`  
  For a **ready-to-run** queue you want **exactly one** live queue prefix: **`UNTESTED-…`** (and may also see **`CLOSED-…`** as history). **If you see `TESTPLAN-…` instead of `UNTESTED-…`,** the coder is still revising **Testing instructions** — **do not** start [`003-tester/TESTER.md`](../003-tester/TESTER.md); wait for **`TESTPLAN-…` → `UNTESTED-…`**. If **`ls`** shows **both** **`TESTPLAN-…`** and **`UNTESTED-…`**, **stop** — handoff defect (see **Queue defects to avoid**). If **only** **`CLOSED-…`** appears, **stop** — restore or fetch **`UNTESTED-…`**; do **not** treat **`CLOSED-…`** as the queue file.

- **One verification block, one paste:** Choose **A1**, **A2**, or **B** in **Verification commands** and execute that block **from the first `set -e` through the last `rg`** without changing directory between lines. **Do not** run only the **`cargo`** lines or only the **`rg`** lines, and **do not** mix lines from different blocks. Mixing repo-root paths (`src-tauri/src/…`) with crate-root paths (`src/…`) in the same session causes false failures (see **Two different directories named `src`**).

- **`rg` and `set -e`:** With **`set -e`**, **`rg` exits 1** when a pattern has **no** matches and the script **stops** — that means **fail** for this task. All patterns in the block are required to match somewhere in the given paths (except the documented “same line twice” case for **`turn_lifecycle.rs`**).

### Operator filename (`003-tester/TESTER.md`)

- **Executable queue file for a real run:** `tasks/UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md` only. At run start, rename **`UNTESTED-` → `TESTING-`**, run **Verification commands**, then apply outcome naming per **TESTER.md**.
- **Missing `UNTESTED-…` at repo tip:** **Stop.** Do **not** verify from **`CLOSED-…`** alone or invent **`TESTING-…`** from **`CLOSED-…`** unless your operator runbook explicitly allows it. Sync/pull for **`UNTESTED-…`**, or return a **queue / handoff defect** to the coder.
- **Emit `TESTPLAN-` only when this markdown is wrong** (wrong paths, wrong queue rules, ambiguous `cargo` cwd wording). Do **not** use **`TESTPLAN-`** because **`rg`** on top-level **`src/`** returns no matches — that is a **tester path mistake** (see **Two different directories named `src`**). Do **not** use **`TESTPLAN-`** for “no Mac” / Linux-only runs — use **`WIP-…`** plus an environment note (see **Queue and environment policy** → **Host and toolchain**, or **Before you run anything** row **1**).
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

- **Both `TESTPLAN-…` and `UNTESTED-…` exist for the same stamp** — **Stop.** That is a **coder handoff error** (two queue states at once). Do **not** pick either file for **TESTER.md** until the tree has **only** **`UNTESTED-…`** (coder removes **`TESTPLAN-…`** after publishing the repair).
- **Operator names `UNTESTED-…` but only `CLOSED-…` exists** — Do **not** “verify against CLOSED.” Update your tree, restore **`UNTESTED-…`** from git, or bounce the task to the coder. Appending new results into **`CLOSED-…`** without a live **`UNTESTED-…`**/`TESTING-…` step is out of procedure.
- **Only `TESTPLAN-…` is present** — Instructions are still in repair; wait for **`TESTPLAN-…` → `UNTESTED-…`** before starting the **TESTER.md** rename chain.

**Current handoff (use the basename of *this* file):**

- If this file is named **`TESTPLAN-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** — **coder repair**; **no** tester queue slot. **Do not** rename **`TESTPLAN-…` → `TESTING-…`**.
- If this file is named **`UNTESTED-20260321-2000-openclaw-hung-turn-timeout-event-gate.md`** — **tester queue**; follow **TESTER.md** (**`UNTESTED-…` → `TESTING-…`** at run start).

### Shell compatibility

- **Preferred:** paste blocks **A1**/**A2**/**B** into **`bash`** (macOS: `/bin/bash` or `bash -lc '…'`) so **`set -e`** aborts the same way as in CI. **zsh** usually runs these snippets correctly; if a failing command does **not** stop the script, re-run the block under **`bash`** before filing a failure.
- **fish (or other non-POSIX shells):** **Do not** paste multiline **`set -e`** blocks directly into **fish** — open **`bash`** first, or wrap the **entire** block in **`bash -lc '…'`** (see the next bullet). The **Tester quick gate** and **Preflight** snippets use **`bash`/`zsh`** syntax too.
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
- **Wrong repo:** If **`test -f src-tauri/Cargo.toml`** fails from the directory you believe is root, stop — fix **`cd`** before **`cargo`** or **`rg`** paths that assume repo root.
- **Parent git / monorepo:** If **`git rev-parse --show-toplevel`** points **above** the mac-stats folder (sibling projects under one **`.git`**), **do not** assume that path is the mac-stats root. **Tester quick gate**, **Preflight**, and **A1** now **walk up from `pwd`** to the directory that contains **`src-tauri/Cargo.toml`** when git’s top-level is wrong; you can still use **A2** with an explicit absolute **`cd`** if you prefer.
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
6. **Tarball / missing `.git` / shell not under mac-stats** — use **Verification commands → A2** (explicit **`cd /ABSOLUTE/.../mac-stats`**), or **`cd`** into the tree first so **A1**’s walk-up can find **`src-tauri/Cargo.toml`**.
7. **`rg: command not found`** — install [ripgrep](https://github.com/BurntSushi/ripgrep) or search your editor for the **exact** substrings under **`src-tauri/src/`**; the acceptance literals must still be located in the files named in criteria 3.
8. **Unquoted `**Turn timed out**` in the shell** — some shells glob `**`; always run **`rg -n -F '**Turn timed out**' …`** as in the verification blocks.
9. **Quick gate from `tasks/` (or any subdir)** — a probe that only checks **`./src-tauri/Cargo.toml`** prints false **BLOCK: none**. Use **Tester quick gate** step **0** as written here (**walk-up** from **`pwd`**), then **`cd`** to the printed **`found`** path before **A1**/**A2** if required.
10. **Preflight OK from a subdir, then only `cargo`/`rg` lines from A1/A2** — Preflight uses **`$REPO_ROOT`**-absolute **`test -f`** checks; **A1**/**A2** **`cargo --manifest-path src-tauri/Cargo.toml`** does **not**. You **must** paste the full block (including **`cd`**) or **`cd`** to repo root yourself before **`cargo`**.
11. **`BLOCK: B` + no-git Preflight from `src-tauri/`** — the no-git snippet’s first **`test -f src-tauri/Cargo.toml`** fails at crate root; use **git** Preflight (step **1**) and stay in **`src-tauri/`** for block **B** (step **2**), or **`cd ..`**, run no-git Preflight, then **`cd src-tauri`** before block **B**.
12. **Skipping Preflight after the probe (especially `BLOCK: B`)** — **Copy-paste order** is **0 → 0b (if needed) → 1 → 2**. Older summaries that said “paste **B** only” after the probe were **wrong**; you **must** run **Preflight** (step **1**) before **any** **A1**/**A2**/**B** paste.
13. **`BLOCK: B` + `cd` to repo root after Preflight** — Preflight prints **`OK: mac-stats repo root = …`** but **does not** change **`pwd`**. If you **`cd` to that path** and then paste block **B**, **`test -f Cargo.toml`** at repo root **fails** (mac-stats has **no** root **`Cargo.toml`**). **Stay in `src-tauri/`** for block **B** (or use **A1**/**A2** from repo root instead of **B**).

### Preflight (required)

Run **one** of these, depending on whether you have a `.git` directory.

**Inside a git clone of mac-stats (recommended):** from any directory **under** the mac-stats tree (repo root, **`tasks/`**, etc.),

```bash
set -e
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || REPO_ROOT=""
if test -z "$REPO_ROOT" || ! test -f "$REPO_ROOT/src-tauri/Cargo.toml"; then
  REPO_ROOT="$PWD"
  while test "$REPO_ROOT" != "/" && ! test -f "$REPO_ROOT/src-tauri/Cargo.toml"; do
    REPO_ROOT=$(dirname "$REPO_ROOT")
  done
fi
test -f "$REPO_ROOT/src-tauri/Cargo.toml"
test -f "$REPO_ROOT/src-tauri/src/commands/turn_lifecycle.rs"
echo "OK: mac-stats repo root = $REPO_ROOT"
command -v rg >/dev/null && echo "OK: rg" || echo "WARN: install ripgrep or search manually"
```

If the **`test -f`** lines fail, your shell is **not** under a mac-stats checkout — fix **`cd`** or use the **no-git** preflight below (tarball path).

**Same shell as verification:** Preflight may run from **`tasks/`** and pass. **A1**/**A2** still need **`pwd` = repo root** before **`cargo --manifest-path src-tauri/Cargo.toml`** — paste the **full** **A1**/**A2** block (it **`cd`s** to **`$REPO_ROOT`**), or **`cd`** manually to the folder that contains **`src-tauri/`** (same path **Tester quick gate** prints), then run **A1**/**A2** from **`set -e`** onward.

**Tarball / no `.git`:** `cd` manually to the folder that **contains** `src-tauri/Cargo.toml` (mac-stats **repo root**), then run the snippet below. **Do not** run this snippet when your **`pwd`** is **`…/mac-stats/src-tauri`** (crate root): **`test -f src-tauri/Cargo.toml`** will fail — use the **git** Preflight variant if `.git` exists, or **`cd ..`** to repo root first, then return to **`src-tauri/`** only if you are about to run block **B**. This confirms paths only — it is **not** **Verification commands → A2** (which adds **`cargo`**, **`rg`**, and a **`set -e`** block).

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

This section repeats **Copy-paste order** in prose. **Shell step numbers are still only 0 / 0b / 1 / 2** in the table above — the items below are **not** “steps 3+” of that table.

Do **not** mix **A1**/**A2** path prefixes (repo root + `src-tauri/…`) with **B** path prefixes (crate root + `src/…`) in one run.

1. **(Same as table 0 / 0b / 1.)** Run **Tester quick gate** step **0** (directory probe). If the output includes **`cd '/path/to/mac-stats'`** (or similar), run that **`cd`** so **A1**/**A2** relative paths resolve, then **`test -f src-tauri/Cargo.toml`**. Then run **Preflight (required)**: the **git** snippet if you have a `.git` tree (can run from **`tasks/`** etc.), **or** the **no-git** snippet after you have manually **`cd`'d** to repo root — **not** the full **Verification commands → A2** block.
2. **(Same as table 2.)** Paste **exactly one** of **A1**, **A2**, or **B** **in full** from **Verification commands** (same terminal; **`set -e`** should still be active; use **`bash`** if unsure — see **Shell compatibility**). Prefer **A1** if `.git` exists and **BLOCK** is **A1 or A2**, else **A2** when you need an explicit absolute **`cd`** line; use **B** only when step **0** says **BLOCK: B**.
3. **Troubleshooting (not a paste step):** If **`cargo`** fails with **`could not find Cargo.toml`**, you are not using **A1**/**A2**/**B** correctly — re-read **Environment** and **Common instruction defects**.

### Closure checklist (tick before outcome naming)

Use this only when the **on-disk** queue file is named **`UNTESTED-…`** and you have started the **TESTER.md** rename chain (**`UNTESTED-…` → `TESTING-…`**). If the file is still named **`TESTPLAN-…`**, stop — the coder has not published the queue yet (no **`TESTING-…`** rename from **`TESTPLAN-…`**).

1. **Probe:** Ran **Tester quick gate** step **0**; chose **one** of **A1** / **A2** / **B** matching the printed **BLOCK:** (no mixing blocks).
2. **Preflight:** Ran the matching **Preflight (required)** variant (git **or** no-git); both `test -f` lines succeeded. If the probe was **`BLOCK: B`**, confirm you did **not** use **no-git** Preflight from **`src-tauri/`** unless you **`cd ..`** first (see **Probe → Preflight → verification**).
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
| **A1** / **A2** | `test -f src-tauri/Cargo.toml` (mac-stats **repo** root) | You will stay in **`src-tauri/`** for **`cargo`** — use **B** instead. |
| **B** | `test -f src/commands/turn_lifecycle.rs` (crate **`src-tauri/`** root) | `test -f src-tauri/Cargo.toml` succeeds from **`pwd`** — you are at **repo root**; use **A1**/**A2**, not **B**. |

### A1 — Recommended: resolve mac-stats repo root (git + walk-up fallback)

Use **A1** for a normal checkout where you will run **`cargo`** / **`rg`** from the mac-stats **repo root** after this block’s **`cd`**. If you already used **A2** successfully, **do not** also run **A1**.

Use **bash** or **zsh** (or `bash -lc '…'` from fish). **`set -e`** stops on the first failing command — do **not** append `|| true` to **`cd`** here; a wrong directory must **fail** the script.

**How `REPO_ROOT` is chosen:** Prefer **`git rev-parse --show-toplevel`** **only when** that path already contains **`src-tauri/Cargo.toml`** (standalone mac-stats clone). Otherwise **walk up** from **`pwd`** until **`src-tauri/Cargo.toml`** exists (subdirectory such as **`tasks/`**, or mac-stats nested under a **parent git** worktree). If neither yields a root, the block exits with an error — use **A2** after **`cd`**ing to the correct folder, or open a shell **inside** the mac-stats tree.

**Why `--manifest-path`:** mac-stats has **no** `Cargo.toml` at repo root. Invoking **`cargo test`** from root without **`--manifest-path`** is a frequent false failure; the lines below pin the **`mac_stats`** package explicitly.

```bash
set -e
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || REPO_ROOT=""
if test -z "$REPO_ROOT" || ! test -f "$REPO_ROOT/src-tauri/Cargo.toml"; then
  REPO_ROOT="$PWD"
  while test "$REPO_ROOT" != "/" && ! test -f "$REPO_ROOT/src-tauri/Cargo.toml"; do
    REPO_ROOT=$(dirname "$REPO_ROOT")
  done
fi
test -f "$REPO_ROOT/src-tauri/Cargo.toml" || { echo >&2 "Could not locate mac-stats root (expected src-tauri/Cargo.toml). cd into the checkout or use block A2."; exit 1; }
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

Use **A2** when **`git rev-parse` fails** or the tree has no `.git` directory — **not** when **A1** already worked. **Replace** **`/ABSOLUTE/PATH/TO/mac-stats`** on the **`cd`** line with the **absolute path** to your mac-stats root (the directory that **directly** contains **`src-tauri/`** — not `src-tauri/` itself, not a parent that only contains the zip name). **`bash`** expands **`$HOME/projects/mac-stats`** and **`~/projects/mac-stats`** on the **`cd`** line — both are fine. **Edit that line before pasting**; leaving the placeholder verbatim will **`cd`** to a non-existent path and fail. After editing, **`test -f src-tauri/Cargo.toml`** must succeed; if it fails, the path is wrong.

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
