# mac-stats Project Overview

## Installation
- **DMG (Recommended):**  
  [Download latest release](https://github.com/raro42/mac-stats/releases/latest) → Drag to Applications  
- **Build from source:**  
  ```bash
  git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
  ```  
  Or one-liner:  
  `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`  
- **Gatekeeper Note:**  
  If blocked, right-click DMG → **Open**, or run:  
  `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

---

## At a Glance
- **Menu Bar:** CPU, GPU, RAM, disk metrics; click to open details window  
- **AI Chat:**  
  - Ollama integration (via app or Discord)  
  - Tools: FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP  
- **Discord Bot:**  
  - Task runner, scheduler, and MCP integration  
  - Lives in menu bar with real-time metrics  

---

## Agents Overview
### Tool Agents (Ollama Invocations)
| Agent         | Invocation         | Purpose                          | Implementation               |
|---------------|--------------------|----------------------------------|------------------------------|
| FETCH_URL     | `FETCH_URL: <URL>` | Fetch web page content           | `commands/browser.rs`        |
| BRAVE_SEARCH  | `BRAVE_SEARCH: <query>` | Brave Search API results       | `commands/brave.rs`          |
| RUN_CMD       | `RUN_CMD: <command>` | Execute shell commands          | `run_cmd.rs` (allowlist: ps, wc, uptime, cursor-agent) |
| PERPLEXITY_SEARCH | `PERPLEXITY_SEARCH: <query>` | Perplexity API results         | `perplexity/` module         |

### Entry-Point Agents
- **Discord:** Triggers Ollama with `from_remote=true`  
- **CPU Window:** Direct chat with Ollama and execution  
- **Cursor Agent:** Task runner with workspace defaults  

---

## Documentation vs Code Review

### Performance / Intervals
| Doc Claim       | Code Reality       | Severity |
|-----------------|--------------------|----------|
| Temperature: 15s | 20s (lib.rs)       | Medium   |
| Frequency: 20s   | 30s (lib.rs)       | Medium   |
| Process Cache: 30s | 30s (state.rs)   | OK       |
**Resolution:** Updated docs to reflect 20s/30s intervals.

---

### Directory Structure
| Doc Claim               | Code Reality                     | Severity |
|-------------------------|----------------------------------|----------|
| `ui/bridge/`, `cli/`    | Not present (use `ui/`, `cli/` instead) | Medium |
| `commands/browser.rs`   | Correct (fetch_page_content)     | OK       |
**Resolution:** AGENTS.md updated to reflect actual structure.

---

### Tool Loop & Discord
| Doc Claim                          | Code Reality                          | Severity |
|------------------------------------|---------------------------------------|----------|
| Max tool iterations: 5 (Discord)   | Default: 15 (overridable per agent)   | High     |
| `TASK:` and `TASK_RUN:` parsing    | Supported in `scheduler/mod.rs`       | OK       |
**Resolution:** Updated to 15 iterations (default).

---

### RUN_CMD Allowlist
| Doc Claim                          | Code Reality                          | Severity |
|------------------------------------|---------------------------------------|----------|
| Allowlist: cat, head, tail, ls, grep | Includes ps, wc, uptime, cursor-agent | Low      |
**Recommendation:** Add note about expanded allowlist in docs.

---

### Frontend & Paths
| Doc Claim                          | Code Reality                          | Severity |
|------------------------------------|---------------------------------------|----------|
| `src/main.js`, `src/index.html`    | Exist                                | OK       |
| CPU idle: <0.1%                    | ~0.5% (CLAUDE.md)                    | Low      |
**Resolution:** Updated CPU line to match CLAUDE.

---

### Version & Build
| Doc Claim                          | Code Reality                          | Severity |
|------------------------------------|---------------------------------------|----------|
| Version in `Cargo.toml`           | Used in `get_app_version()`          | OK       |
| Stale branch name (feat/theming)   | Removed                              | Low      |

---

## Fixes Applied
- CLAUDE.md: Temperature 20s, frequency 30s  
- AGENTS.md: Updated directory structure  
- 100_all_agents: Max tool iterations → 15  
- Removed stale branch line from CLAUDE.md  
- CPU idle line aligned with CLAUDE  

---

## Post-Review Iterations
- **Headless Mode:**  
  - `from_remote=true` → prefer_headless = !wants_visible_browser(question)  
  - Retries stay headless  
- **Verification:**  
  - `cargo check`  
  - `mac_stats -vv discord run-ollama 'What is 2+2?'` → Expected "4"  

---

## Open Tasks:
- **RUN_CMD Allowlist:** Add note in 100_all_agents or 011 about expanded allowlist  
- **Stale Branch:** Remove or update CLAUDE.md line about "feat/theming" branch  
- **Discord Headless Logic:** Verify `prefer_headless` behavior in edge cases  
- **Docs Sync:** Ensure 100_all_agents reflects RUN_CMD allowlist updates
