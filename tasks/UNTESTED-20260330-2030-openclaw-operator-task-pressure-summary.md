---
## Triage summary (TOP)

- **Coder (UTC):** 2026-03-30 — **FEATURE-CODER** (`006-feature-coder/FEATURE-CODER.md`): completed **`FEAT → WIP → UNTESTED`** for **`20260330-2030-openclaw-operator-task-pressure-summary`**. Pick-up filename was **`UNTESTED-…`**; renamed **`UNTESTED → FEAT → WIP`** to satisfy the FEAT step, then **`WIP → UNTESTED`** after verification. §4 already implemented in **`src-tauri`** (no code changes this run). **`cd src-tauri && cargo check && cargo test operator_task_pressure`** passed (`task::tests::operator_task_pressure_summary_empty_dir`). **§6 — Testing instructions** retained for the tester.
- **Next step:** Tester runs **§6** (after **§5**) on **`UNTESTED-20260330-2030-openclaw-operator-task-pressure-summary.md`**.
---

# UNTESTED: OpenClaw parity — operator task pressure summary in execution context

**Created (UTC):** 2026-03-30 20:30  
**Source:** OpenClaw vs mac-stats review  
**Topic:** Give the execution-time model compact task backlog + review-loop limits next to live metrics.

---

## 1. Summary

mac-stats already logs `Task scan: open=…` in the review loop, but the **agent router execution system prompt** only included **hardware metrics**. Operators and the model had no inline view of **how many tasks** are open / WIP / paused or how aggressively the review loop drains the queue. A short **Task backlog (operator)** block is appended after **Current system metrics:** in the shared execution fragment so Discord, scheduler, and task-runner paths stay aligned.

---

## 2. Acceptance criteria

1. Execution system prompt (via `context_assembler::fragments::live_metrics_execution_system_section`) appends a compact section with **counts** from `task::count_tasks_by_status` (open, wip, paused, finished, unsuccessful) and **review-loop parameters** (interval, max open tasks per cycle, WIP stale timeout) sourced from the same constants as `task/review.rs`.
2. On `count_tasks_by_status` error, the section degrades to a single line explaining unavailability (no panic).
3. `cd src-tauri && cargo check && cargo test` succeed; at least one unit test covers formatting (empty task dir or controlled `MAC_STATS_TASK_DIR`).

---

## 3. Notes

- **having_fun** casual Discord prompts are unchanged; this task targets the **agent router / execution** path only.

---

## 4. Implementation (mac-stats)

- **`src-tauri/src/task/review.rs`** — `TASK_REVIEW_INTERVAL_SECS`, `TASK_WIP_STALE_TIMEOUT_SECS`, `TASK_REVIEW_MAX_OPEN_PER_CYCLE` (`pub(crate)`), used by stale-WIP logic and the new summary text.
- **`src-tauri/src/task/mod.rs`** — `format_operator_task_pressure_summary()`.
- **`src-tauri/src/commands/context_assembler.rs`** — `live_metrics_execution_system_section()` appends the task block after metrics.
- **`src-tauri/src/commands/ollama.rs`** — agent-router execution path sets `metrics_for_system` from `fragments::live_metrics_execution_system_section()` so Discord / scheduler / task-runner turns include the task backlog block.

---

## 5. Verification (automated)

```bash
cd src-tauri && cargo check && cargo test operator_task_pressure
```

Full unit suite (acceptance criterion 3); run after the focused test or in CI:

```bash
cd src-tauri && cargo test
```

```bash
rg -n "format_operator_task_pressure_summary|live_metrics_execution_system_section" src-tauri/src/task/mod.rs src-tauri/src/commands/context_assembler.rs
```

---

## 6. Testing instructions

**FEATURE-CODER:** After the coder run (**`FEAT → WIP → UNTESTED`**), the **tester** owns this section. Run **Section 5** before manual checks. **Coder:** leave reproducible smoke, checklist, and pass/fail criteria below; do not strip this section when renaming task-file prefixes.

**Required order:** run **Section 5 — Verification (automated)** first, then manual or optional runtime steps below.

### Minimal smoke (copy-paste)

From the repo root:

```bash
cd src-tauri && cargo check && cargo test operator_task_pressure
```

Optional full suite (acceptance criterion 3): `cd src-tauri && cargo test`

Optional ripgrep sanity: `rg -n "format_operator_task_pressure_summary|live_metrics_execution_system_section" src-tauri/src/task/mod.rs src-tauri/src/commands/context_assembler.rs`

### Quick checklist (tester)

- [ ] `cd src-tauri && cargo check` succeeds.
- [ ] `cargo test operator_task_pressure` (or `cargo test task::tests::operator_task_pressure_summary_empty_dir -- --exact`) passes.
- [ ] (Optional) full `cargo test` for acceptance criterion 3.
- [ ] Manual or runtime: execution system prompt shows **`## Task backlog (operator)`** after live metrics; counts match `~/.mac-stats/task/`; **CPU:** / **Load** lines unchanged above the new block.

### What to verify

- Any **agent-router** Ollama execution (Discord agent channel, **TASK_RUN** / scheduler with tools, CPU-window chat that uses the execution stack) receives a system prompt whose **`Current system metrics:`** block is followed by **`## Task backlog (operator)`**, **`Counts: open=…`**, and a **`Review loop:`** line whose numbers match **`task/review.rs`** (`TASK_REVIEW_INTERVAL_SECS`, `TASK_REVIEW_MAX_OPEN_PER_CYCLE`, `TASK_WIP_STALE_TIMEOUT_SECS`). As of this task, expect **`every 60 s`**, **`up to 3 open task(s) started per cycle`**, and **`30 min`** stale WIP wording.
- With several tasks in **`~/.mac-stats/task/`**, counts in the block match reality (compare to **`mac_stats --task list`** or **`TASK_LIST`**).
- **Regression:** Live metrics lines (**CPU:**, **Load**, etc.) still appear unchanged above the new block.

### How to test

1. Run **Section 5 — Verification (automated)**. The filter **`operator_task_pressure`** runs **`task::tests::operator_task_pressure_summary_empty_dir`**, which sets **`MAC_STATS_TASK_DIR`** to an empty temp directory and asserts the summary contains **`## Task backlog (operator)`**, zero **`open=`** / **`wip=`** counts, **`Review loop: every 60 s`** (aligned with **`TASK_REVIEW_INTERVAL_SECS`**), **`up to 3 open task(s) started per cycle`** (aligned with **`TASK_REVIEW_MAX_OPEN_PER_CYCLE`**), and **`30 min may be closed`** (stale WIP hint from **`TASK_WIP_STALE_TIMEOUT_SECS`**). **Single-test variant (same assertions):** `cd src-tauri && cargo test task::tests::operator_task_pressure_summary_empty_dir -- --exact`. Optionally run the full **`cargo test`** block in Section 5 to satisfy acceptance criterion 3 end-to-end.
2. **Static inspection:** In **`context_assembler.rs`**, confirm **`live_metrics_execution_system_section`** formats **`live_metrics_for_prompt()`** first, then **`format_operator_task_pressure_summary()`** (metrics block first, task block second).
3. **Error path (optional):** If the task directory cannot be read, the prompt must still include **`## Task backlog (operator)`** with **`(unavailable:`** and an error hint — no panic. Code review **`format_operator_task_pressure_summary`** in **`task/mod.rs`** (`Err` branch).
4. **Optional (runtime):** Start mac-stats with **`-vvv`**, trigger one short Discord or CPU-window turn that uses **`answer_with_ollama_and_fetch`**, then confirm logs or debugger inspection of **`metrics_for_system`** / **`build_execution_system_content`** shows both **Current system metrics** and **Task backlog (operator)** in one string.

### Pass / fail criteria

- **Pass:** Automated tests green; manual or optional runtime check shows the dual block in the execution system path; having_fun behaviour unchanged.
- **Fail:** Missing section, wrong counts vs filesystem, metrics section broken, or panic on unreadable task dir.
