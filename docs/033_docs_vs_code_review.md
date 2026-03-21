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
**Resolution:** Full allowlist (including ps, wc, uptime, cursor-agent) is documented in docs/011_local_cmd_agent.md and docs/100_all_agents.md; no further doc change needed.

---

### Frontend & Paths
| Doc Claim                          | Code Reality                          | Severity |
|------------------------------------|---------------------------------------|----------|
| `src/main.js`, `src/index.html`    | Exist                                | OK       |
| CPU idle: <0.1%                    | ~0.5% (agents.md)                    | Low      |
**Resolution:** Updated CPU line to match agents.md.

---

### Version & Build
| Doc Claim                          | Code Reality                          | Severity |
|------------------------------------|---------------------------------------|----------|
| Version in `Cargo.toml`           | Used in `get_app_version()`          | OK       |
| Stale branch name (feat/theming)   | Removed                              | Low      |

---

## Fixes Applied
- agents.md (formerly CLAUDE.md): Temperature 20s, frequency 30s  
- AGENTS.md: Updated directory structure  
- 100_all_agents: Max tool iterations → 15  
- Removed stale branch line from project instructions doc  
- CPU idle line aligned with agents.md  
- RUN_CMD allowlist: 100_all_agents and 011_local_cmd_agent already document full list (ps, wc, uptime, cursor-agent).  

---

## Post-Review Iterations
- **Headless Mode:**  
  - `from_remote=true` → prefer_headless = !wants_visible_browser(question)  
  - Retries stay headless  
- **Verification:**  
  - `cargo check`  
  - `mac_stats -vv discord run-ollama 'What is 2+2?'` → Expected "4"  

---

## prefer_headless — Edge cases and verification

**Where it’s set:** Once at the start of the tool loop in `commands/ollama.rs`:  
`prefer_headless = if from_remote { !wants_visible_browser(question) } else { question.to_lowercase().contains("headless") }`.  
Stored in `browser_agent::PREFER_HEADLESS`; used for the whole run (including retries).

**wants_visible_browser(question):** True only if the question (lowercased) contains one of:  
`"visible"`, `"show me the browser"`, `"show me a browser"`, `"i want to see the browser"`, `"open a window"`.

| Scenario | from_remote | question | prefer_headless |
|----------|-------------|----------|-----------------|
| Discord/scheduler, no visible ask | true | e.g. "screenshot example.com" | true |
| Discord, user asks for visible | true | "show me the browser" or "visible" | false |
| Discord, empty/short question | true | "" or "hi" | true (wants_visible_browser false) |
| CPU window, user says headless | false | "use headless" | true |
| CPU window, default | false | "screenshot x" | false |

**Session reuse** (`browser_agent::get_or_create_browser`): Cached session is reused only if within idle timeout **and** `was_headless == prefer_headless`. If the preference changed (e.g. next request from Discord wants visible), the old session is dropped and a new browser is created with the new mode.

**Retries:** Connection-error retry uses the same `prefer_headless_for_run()` value; no re-read of the question, so retries stay headless when the run started headless.

**ensure_chrome_on_port:** When `prefer_headless_for_run()` is true, we skip launching visible Chrome (early return); the next CDP use will go through `get_or_create_browser` and can launch headless if needed.

**Verification checklist:**  
- `from_remote=true`, question without "visible"/"show me the browser" → logs show "user requested headless" or "launching headless Chrome".  
- `from_remote=true`, question "I want to see the browser" → logs show "connecting to Chrome on port" or "launching visible Chrome".  
- After a headless run, a new run with visible preference → log "preference changed (headless true → false), creating new session".  
- `mac_stats -vv discord run-ollama 'Take a screenshot of example.com'` → no visible Chrome window.  
- `mac_stats -vv discord run-ollama 'Open a window and take a screenshot'` → visible Chrome if port 9222 available.

---

## Open Tasks:
- *(None; Discord headless logic verification documented above.)*
