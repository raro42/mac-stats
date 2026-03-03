# Docs vs code – deep review

This document compares documentation (README, CLAUDE.md, AGENTS.md, and numbered docs) to the actual codebase. **Mismatch** = doc says X, code does Y. **Missing** = code has it, doc doesn't. **Obsolete** = doc references something that no longer exists.

**Status:** Review was done; the recommended fixes have been **applied** (see §9). This doc serves as the audit trail and re-verification reference.

### Need vs have (summary)

| Need | Have |
|------|------|
| Discord/scheduler use headless browser by default (no popups) | `from_remote` + `wants_visible_browser()`; Discord/scheduler/task pass `from_remote=true` |
| Docs match code (intervals, modules, tool count, CPU %) | CLAUDE, AGENTS, 100_all_agents, README, lib.rs comments updated |
| Test pipeline without sending a real Discord message | `mac_stats discord run-ollama "<question>"` (ensures defaults + Ollama client, then runs same pipeline) |
| RUN_CMD allowlist documented | 100_all_agents RUN_CMD row includes ps, wc, uptime, cursor-agent |
| How to test browser flow | docs/007 § "Testing the pipeline without Discord" |

---

## 1. Performance / intervals

| Doc | Claim | Code reality | Severity |
|-----|--------|---------------|----------|
| CLAUDE.md, "Selective resource usage" | Temperature read every **15 seconds** | `lib.rs`: threshold is **20 seconds** (`elapsed().as_secs() >= 20`) | Medium |
| CLAUDE.md | Frequency read every **20 seconds** | `lib.rs`: threshold is **30 seconds** (`elapsed().as_secs() >= 30`) | Medium |
| AGENTS.md "Process list" | Cached for **30 seconds** | `state.rs` + `metrics/mod.rs`: 30s process cache – **matches** | OK |

**Resolution:** CLAUDE.md and lib.rs comments updated to 20s / 30s.

---

## 2. Directory structure and modules

| Doc | Claim | Code reality | Severity |
|-----|--------|---------------|----------|
| AGENTS.md "Code organization" | Split into `metrics/`, `ffi/`, **`ui/bridge/`**, **`cli/`**, `logging/` | No `ui/bridge/` or `cli/` in `src-tauri/src/`. Present: `ui/`, `metrics/`, `ffi/`, `logging/`, `commands/`, `discord/`, `browser_agent/`, `agents/`, `task/`, `scheduler/`, `mcp/`, `perplexity/`, `redmine/`, etc. | Medium |
| CLAUDE.md "Code organization" | Split into `metrics/`, `ffi/`, `ui/`, `config/`, `logging/` | Matches (no mention of ui/bridge or cli). | OK |
| AGENTS.md "Directory structure" | `commands/browser.rs` – fetch_page for Ollama web tasks | Correct: `commands/browser.rs` has `fetch_page_content` and `fetch_page`; used by FETCH_URL and scheduler. | OK |
| AGENTS.md "commands" list | `commands/plugins.rs` | Exists. | OK |
| docs/100_all_agents | SKILL "Implementation" `skills.rs` | Module is `src-tauri/src/skills.rs` (top-level, not under `commands/`). | OK |

**Resolution:** AGENTS.md updated to "split into `metrics/`, `ffi/`, `ui/`, `config/`, `logging/`".

---

## 3. Tool loop and Discord

| Doc | Claim | Code reality | Severity |
|-----|--------|---------------|----------|
| docs/100_all_agents §2.1 | "Up to **5** tool iterations" (Discord) | `ollama.rs`: default `max_tool_iterations = 15`; agent override can set different value. | High |
| docs/012_cursor_agent_tasks | "When the scheduler runs `TASK: <path_or_id>` (or `TASK_RUN: <path_or_id>`)" | **Correct**: `scheduler/mod.rs` parses both `TASK:` and `TASK_RUN:` in schedule task body. | OK |
| docs/012 | "RUN_CMD allowlist … cursor-agent" | Default agent skill (e.g. agent-000/skill.md) includes cursor-agent in allowlist. | OK |
| Doc 007 | "Or from devtools: `invoke('configure_discord', { token: 'YOUR_TOKEN' })`" | Rust: `configure_discord(token: Option<String>)`. Tauri typically maps first arg; if frontend passes object, may need to match. | Low |

**Resolution:** 100_all_agents updated to "Up to 15 tool iterations (default; overridable per agent)".

---

## 4. RUN_CMD allowlist

| Doc | Claim | Code reality | Severity |
|-----|--------|---------------|----------|
| docs/100_all_agents RUN_CMD | "Allowed: cat, head, tail, ls, grep, date, whoami" | `run_cmd.rs`: allowlist includes more (e.g. ps, wc, uptime, cursor-agent per agent skill). | Low |
| docs/011_local_cmd_agent | Same short list | Same as above; doc is minimal. | Low |

**Recommendation:** Optional: add "(and others, e.g. ps, wc, uptime, cursor-agent when in skill allowlist)" so docs match behavior.

---

## 5. Frontend and paths

| Doc | Claim | Code reality | Severity |
|-----|--------|---------------|----------|
| AGENTS.md "Frontend" | `src/main.js` "Main entry point", `src/index.html` "App entry HTML" | Both exist. Dashboard is `dashboard.html` / `dashboard.js`; index.html and main.js exist. | OK |
| CLAUDE.md "Frontend" | "src-tauri/dist/: main.js, cpu.js, cpu-ui.js" | dist contains main.js, cpu.js, cpu-ui.js, ollama.js, tauri-logger.js, etc.; themes under dist/themes/. | OK |
| README "At a glance" | "<0.1% with window closed" | CLAUDE.md says "~0.5% idle". Inconsistent. | Low |

**Resolution:** README CPU line updated to match CLAUDE: ~0.5% with window closed, <1% with CPU window open.

---

## 6. Version and build

| Doc | Claim | Code reality | Severity |
|-----|--------|---------------|----------|
| CLAUDE.md "Version management" | Version in `Cargo.toml` only; `get_app_version()` etc. | **Matches**: `metrics/mod.rs` has `get_app_version()` using `env!("CARGO_PKG_VERSION")`. | OK |
| CLAUDE.md "Development Notes" | "Git status: Currently on main branch with changes to … (feat/theming branch)" | Stale branch name; should be updated or removed. | Low |

**Resolution:** Stale line removed from CLAUDE.md.

---

## 7. Discord and scheduler (from_remote / headless)

| Doc | Claim | Code reality | Severity |
|-----|--------|---------------|----------|
| Agent prompts (planning/execution) | "User says headless → no visible window; browser or default → visible Chrome" | **Updated**: When `from_remote` is true (Discord, scheduler, task runner), browser defaults to **headless** unless question asks to "see" the browser. | OK (recent change) |

No doc update strictly required; behavior is in code and planning prompt describes user-visible behavior.

---

## 8. Other docs spot-checks

| Doc | Claim | Code reality | Severity |
|-----|--------|---------------|----------|
| docs/024_mac_stats_merge_defaults | Merge, don't overwrite ~/.mac-stats | Describes intended behavior; no single "merge" function to verify. | OK |
| docs/007_discord_agent | Token from DISCORD_BOT_TOKEN, .config.env, Keychain | Matches discord module and config loading. | OK |
| docs/007 | DISCORD_API GET/POST paths | Matches usage in ollama.rs tool loop. | OK |
| docs/012 | CURSOR_AGENT_WORKSPACE default `$HOME/projects/mac-stats` | `cursor_agent.rs`: same default when dir exists. | OK |
| docs/012 | "cursor-agent … `--print --trust --output-format text`" | Matched in cursor_agent.rs (--print, --trust, output format). | OK |

---

## 9. Fixes applied (done)

| # | Change | Status |
|---|--------|--------|
| 1 | CLAUDE.md: Temperature 20s, frequency 30s; lib.rs comments aligned | Done |
| 2 | AGENTS.md: Code organization – use metrics/, ffi/, ui/, config/, logging/ | Done |
| 3 | docs/100_all_agents.md: Up to 15 tool iterations (default) | Done |
| 4 | CLAUDE.md: Stale feat/theming branch line removed | Done |
| 5 | README CPU line aligned with CLAUDE; lib.rs 20s/30s comments | Done |
| 6 | Headless when from_remote: prefer_headless = !wants_visible_browser(question) when from_remote (retries stay headless) | Done |

**Optional (not done):** RUN_CMD allowlist – add note in 100_all_agents or 011 that allowlist includes ps, wc, uptime, cursor-agent when in skill.

---

## 10. Iteration (post-review)

- **Headless on retry:** When `from_remote` is true (Discord, scheduler, `discord run-ollama`), browser preference is now computed as `prefer_headless = !wants_visible_browser(question)` so retry questions (e.g. "Verification said we didn't fully complete...") never flip to visible. Local (non-remote) still uses the "headless" keyword only.
- **Verification:** `cargo check` and `mac_stats -vv discord run-ollama 'What is 2+2? Reply with one number.'` (exit 0, reply "4", verification YES).

---

## 11. How this review was done

- Grep and file listing for module/dir names.
- Reading `lib.rs` for temperature/frequency intervals and tool loop.
- Reading `ollama.rs` for max_tool_iterations, tool names, and TASK_*.
- Reading `scheduler/mod.rs` for TASK:/TASK_RUN:.
- Reading `cursor_agent.rs` for workspace default and flags.
- Spot-checks of 007, 012, 024, and agent prompts.

To re-verify after changes: run `cargo check`, then `mac_stats discord run-ollama 'What is 2+2? Reply with one number.'` for a quick pipeline test, then re-read the sections above and update this doc.
