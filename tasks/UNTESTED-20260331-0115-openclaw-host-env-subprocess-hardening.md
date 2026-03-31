---
## Triage summary (TOP)

- **Coder (UTC):** 2026-03-31 — **FEATURE-CODER** (`006-feature-coder/FEATURE-CODER.md`): stem `20260331-0115-openclaw-host-env-subprocess-hardening`. **Pickup:** `tasks/FEAT-20260331-0115-….md` **absent**; used **`UNTESTED-…` → `WIP-…` → (verify) → `UNTESTED-…`** per §6. **Implementation:** in-tree — `security::host_exec_env` (`apply_host_exec_env_hardening` / `_tokio`), all §4 call sites including **`is_cursor_agent_available`** (`which cursor-agent`). **§2 / §3 / §4** satisfied; **no Rust code changes** this run. **Section 6** testing instructions present. **Verification (this run):** `cargo check`, `cargo test host_exec_env`, `cargo test pipeline_date_wc`, `cargo test --lib` — **pass** (**878** `mac_stats` lib tests).
- **Next step:** Tester runs **Section 6** (after **Section 5**).
---

# UNTESTED: OpenClaw parity — host env subprocess hardening (RUN_CMD / agents / MCP)

**Created (UTC):** 2026-03-31 01:15  
**Source:** OpenClaw `host-env-security-policy` + `sanitizeHostExecEnv` (agent / host execution)  
**Topic:** Strip dangerous inherited environment variables from mac-stats–spawned subprocesses so agent tools cannot inherit `DYLD_*`, `PYTHONPATH`, `NODE_OPTIONS`, etc.

---

## 1. Summary

OpenClaw filters the parent environment before running host-executed commands. mac-stats spawns several subprocesses with the default inherited environment (`RUN_CMD` via `sh`, `PYTHON_SCRIPT`, `CURSOR_AGENT`, plugins, Node skill reduction, MCP stdio servers, browser launch, lifecycle hooks). Align with the same **blocked keys + blocked prefixes** policy (OpenClaw `isDangerousHostEnvVarName`) and apply it at spawn time.

---

## 2. Acceptance criteria

1. Central helper (e.g. `security::host_exec_env`) implements removal of variables matching OpenClaw **`blockedKeys` + `blockedPrefixes`** (uppercase / prefix rules), plus **`BROWSER`**, **`GIT_EDITOR`**, **`GIT_SEQUENCE_EDITOR`** (recent OpenClaw infra parity).
2. Helper is applied to **agent-adjacent** spawns: `RUN_CMD` (`sh -c`), `PYTHON_SCRIPT`, `CURSOR_AGENT` main invocation, plugin scripts, `content_reduction` Node invoke, MCP stdio `tokio::process::Command`, visible Chromium launch, Ori prefetch, compaction / session-memory hook shells.
3. `cd src-tauri && cargo check && cargo test` pass; unit tests prove at least one blocked key (e.g. `DYLD_*` or `PYTHONPATH`) is not visible in a child after hardening.

---

## 3. Notes

- Do **not** strip `HOME` / normal path (OpenClaw base sanitization does not use `blockedOverrideKeys` for inherited env).
- `which cursor-agent` probe (`is_cursor_agent_available`) uses **`apply_host_exec_env_hardening`** like the main `cursor-agent` spawn.
- Well-known **`DYLD_*`** / **`LD_*`** library-injection names are also removed unconditionally so variables set only on the `Command` (not present in the parent process) are still stripped.

---

## 4. Implementation (mac-stats)

- **`src-tauri/src/security/host_exec_env.rs`** — `apply_host_exec_env_hardening` / `apply_host_exec_env_hardening_tokio`: strip `BLOCKED_ENV_KEYS` (OpenClaw `blockedKeys` + `BROWSER` / `GIT_*` editors + common `DYLD_*` / `LD_*` injection keys) and any parent env var whose uppercase name matches `blockedPrefixes` (`DYLD_`, `LD_`, `BASH_FUNC_`).
- **`src-tauri/src/security/mod.rs`** — `pub mod host_exec_env`.
- **Call sites:** `commands/run_cmd.rs` (`sh -c` for RUN_CMD), `commands/python_agent.rs`, `commands/cursor_agent.rs` (main invocation + `which` availability probe), `commands/content_reduction.rs` (Node), `commands/compaction_hooks.rs`, `session_memory.rs` (hook shells), `commands/ori_lifecycle.rs` (Ori prefetch), `plugins/mod.rs`, `mcp/mod.rs` (stdio MCP), `browser_agent/mod.rs` (visible Chromium launch).

---

## 5. Verification (automated)

```bash
cd src-tauri && cargo check && cargo test host_exec_env
```

Full unit suite (acceptance criterion 3):

```bash
cd src-tauri && cargo test
```

```bash
rg -n "apply_host_exec_env_hardening" src-tauri/src/security/host_exec_env.rs src-tauri/src/commands/run_cmd.rs src-tauri/src/commands/python_agent.rs src-tauri/src/commands/cursor_agent.rs src-tauri/src/commands/content_reduction.rs src-tauri/src/commands/compaction_hooks.rs src-tauri/src/commands/ori_lifecycle.rs src-tauri/src/plugins/mod.rs src-tauri/src/mcp/mod.rs src-tauri/src/browser_agent/mod.rs src-tauri/src/session_memory.rs
```

Optional: `cd src-tauri && cargo clippy`

---

## 6. Testing instructions

**FEATURE-CODER task-file lifecycle (same stem `20260331-0115-openclaw-host-env-subprocess-hardening`):** `tasks/FEAT-….md` → rename to `WIP-….md` while coding → add/keep this section → rename to `UNTESTED-….md` for tester handoff. If **`FEAT-…` does not exist** but **`UNTESTED-…` does** (coder pickup), rename **`UNTESTED-…` → `WIP-…`** first, then finish work and **`WIP-…` → `UNTESTED-…`** again for handoff.

**Assigned `FEAT-…` path missing:** When instructions name `tasks/FEAT-20260331-0115-….md` but that file is absent and `tasks/UNTESTED-….md` exists (same stem), use the **`UNTESTED → WIP → UNTESTED`** flow above — do not substitute a different FEAT task.

**On-disk handoff path:** after the coder **`WIP → UNTESTED`** rename, this task lives at `tasks/UNTESTED-20260331-0115-openclaw-host-env-subprocess-hardening.md` (same stem as `FEAT-…` / `WIP-…`).

**Canonical handoff:** this section is the source of truth for the tester after the **`WIP → UNTESTED`** rename (do not remove it when changing task-file prefixes).

**FEATURE-CODER:** After the coder run (**on-disk `FEAT-…` → `WIP-…` → `UNTESTED-…`**), the **tester** owns this section. Run **Section 5** before manual checks. **Coder:** leave reproducible smoke, checklist, and pass/fail criteria below; do not strip this section when renaming task-file prefixes.

**Required order:** run **Section 5 — Verification (automated)** first, then manual or optional runtime steps below.

### Coder verification (this run)

From repo root (same as **Minimal smoke**, plus full suite):

```bash
cd src-tauri && cargo check && cargo test host_exec_env && cargo test pipeline_date_wc && cargo test
```

**Result:** all passed (`host_exec_env`: 3 tests; `pipeline_date_wc`: 1 test; `cargo test --lib`: **878** tests passed, as of this FEATURE-CODER run). For all targets including bins, use `cd src-tauri && cargo test` (Section 5).

### Tester checklist (quick)

1. Run **Section 5** automated commands from repo root (`cargo check`, `cargo test host_exec_env`, full `cargo test`).
2. Run **Minimal smoke** below (`host_exec_env` + `pipeline_date_wc`).
3. Optionally run the **Section 5** `rg` one-liner to confirm all expected call sites still invoke `apply_host_exec_env_hardening` / `_tokio`.
4. Optionally perform **Manual / runtime** steps if you rely on RUN_CMD, MCP, PYTHON_SCRIPT, or hooks.
5. Apply **Pass / fail** below and update the task prefix to **`TESTING-`** / **`CLOSED-`** per your tester workflow.

### Minimal smoke (copy-paste)

From the repo root:

```bash
cd src-tauri && cargo check && cargo test host_exec_env && cargo test pipeline_date_wc
```

**Expect:** `host_exec_env` runs **3** unit tests (DYLD / PYTHONPATH / `LD_` prefix); all pass. `pipeline_date_wc` passes.

`pipeline_date_wc` is `commands::run_cmd::run_cmd_stage_validate_tests::pipeline_date_wc_integration`: exercises the hardened **`sh -c`** RUN_CMD path (`date | wc -c`) so PATH and shell execution still work after env stripping.

For the full crate test suite, use **Section 5** (`cargo test` with no filter).

### Manual / runtime (optional)

1. With mac-stats running, trigger a trivial **RUN_CMD** (e.g. from the agent: `RUN_CMD: date`) and confirm it still succeeds (PATH and `HOME` remain available to the shell).
2. If you use **MCP stdio** or **PYTHON_SCRIPT**, run a short flow and confirm no regression (spawn still works).
3. If **compaction hooks** or **before_reset** hooks are configured, trigger one compaction/reset and confirm the hook still runs (dangerous env stripped; `MAC_STATS_*` vars still set by the app).

### Pass / fail

- **Pass:** Section 5 commands succeed; optional manual steps behave as before for normal workflows.
- **Fail:** Any `cargo test` failure, or agent `RUN_CMD` / MCP / hooks consistently failing to spawn.
