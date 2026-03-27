# CLOSED — OpenClaw-style agent session timeout alignment (2026-03-22)

## Goal

Keep the **two-clock** model aligned across code and docs (OpenClaw-style clarity): **Ollama per-request** HTTP timeout (`ollamaChatTimeoutSecs`) vs **agent-router session wall-clock** (`agentRouterTurnTimeoutSecsDiscord` / `Ui` / `Remote` for one full `answer_with_ollama_and_fetch` turn). Defaults and narrative must match between the limit matrix, `Config`, and operator-facing documentation.

## References

- `src-tauri/src/commands/agent_session_limits.rs` — limit matrix table
- `src-tauri/src/config/mod.rs` — `ollama_chat_timeout_secs`, `agent_router_turn_timeout_secs_*`
- `src-tauri/src/commands/turn_lifecycle.rs` — `resolve_turn_budget_secs`
- `docs/019_agent_session_and_memory.md` — § "Agent router time limits"

## Acceptance criteria

1. **Build:** `cargo check` in `src-tauri/` succeeds.
2. **Tests:** `cargo test` in `src-tauri/` succeeds.
3. **Static alignment:** Matrix in `agent_session_limits.rs` matches documented defaults in `docs/019` (300s per-request; Discord/remote wall-clock 300s, in-app 180s; max wall-clock 48h) and `Config` default constants for those timeouts.

## Verification commands

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test
```

Optional spot-check:

```bash
rg -n "ollamaChatTimeoutSecs|agentRouterTurnTimeoutSecs|Two different clocks" docs/019_agent_session_and_memory.md
rg -n "DEFAULT_SECS: u64 = (300|180)" src-tauri/src/config/mod.rs | head
```

## Test report

**Date:** 2026-03-27 (local operator environment, America-friendly; not UTC).

**Preflight:** `tasks/UNTESTED-20260322-1805-openclaw-agent-session-timeout-alignment.md` was absent from the workspace at the start of this run; the task body was written as `UNTESTED-…`, then renamed to `TESTING-…` per `003-tester/TESTER.md`. No other `UNTESTED-*` file was used.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (854 tests in `mac_stats` library; 0 failed; 1 doc-test ignored)

**Static alignment**

- `agent_session_limits.rs` matrix: Ollama HTTP **300s**; session wall-clock **Discord 300s**, **in-app 180s**, **remote 300s**; max tool iterations **15**; matches table rows vs `docs/019_agent_session_and_memory.md` § "Two different clocks" (300s per-request; 300s Discord/remote and 180s in-app wall-clock; 48h cap stated in doc).
- `Config::ollama_chat_timeout_secs` and `agent_router_turn_timeout_secs_{discord,ui,remote}` default `DEFAULT_SECS` lines at 300 / 300 / 180 / 300 respectively (spot-check via `rg`).

**Outcome:** All acceptance criteria satisfied for this verification pass. Live Discord/Ollama long-turn behaviour was not exercised end-to-end here.

### Test report — tester follow-up (2026-03-27, local; not UTC)

**Preflight:** Operator asked to run `003-tester/TESTER.md` against `tasks/UNTESTED-20260322-1805-openclaw-agent-session-timeout-alignment.md`. That `UNTESTED-…` path was **not present** in the workspace (task already lives as `CLOSED-20260322-1805-openclaw-agent-session-timeout-alignment.md`), so the `UNTESTED → TESTING` rename could not be applied without inventing a duplicate filename. No other `UNTESTED-*` task was substituted.

**Commands run**

- `cd src-tauri && cargo check` — **pass**
- `cd src-tauri && cargo test` — **pass** (854 tests in `mac_stats` library; 0 failed; 1 doc-test ignored)

**Optional spot-checks**

- `rg` on `docs/019_agent_session_and_memory.md`: **Two different clocks** section still documents `ollamaChatTimeoutSecs` vs `agentRouterTurnTimeoutSecs*` with 300s per-request and 300s Discord/remote vs 180s in-app wall-clock, 48h cap.
- `rg` on `config/mod.rs`: `DEFAULT_SECS` 300 / 300 / 180 / 300 for the aligned timeout fields (matches prior pass).

**Static alignment:** `agent_session_limits.rs` limit matrix (300s HTTP; Discord 300s / in-app 180s / remote 300s; 15 tool iterations) unchanged and consistent with `docs/019` and `Config` defaults.

**Outcome:** All acceptance criteria still satisfied. Filename left **`CLOSED-…`** (no `WIP-` rename).
