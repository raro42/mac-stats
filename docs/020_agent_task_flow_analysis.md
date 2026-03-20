# mac-stats

**The AI agent that just gets it done. All local.**

[![GitHub release](https://img.shields.io/github/v/release/raro42/mac-stats?include_prereleases&style=flat-square)](https://github.com/raro42/mac-stats/releases/latest)

A local AI agent for macOS: Ollama chat, Discord bot, task runner, scheduler, and MCP—all on your Mac. No cloud, no telemetry. Lives in your menu bar—CPU, GPU, RAM, disk at a glance. Real-time, minimal, there when you look. Built with Rust and Tauri.

<img src="screens/data-poster.png" alt="mac-stats Data Poster theme" width="500">

📋 [Changelog](CHANGELOG.md) · 📸 [Screenshots & themes](screens/README.md)

---

## Install

**DMG (recommended):**  
[Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.

**Build from source:**
```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```
Or one-liner:  
`curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

**If macOS blocks the app:**  
Right-click the DMG → **Open**, then confirm.  
Or run: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

---

## At a glance

- **Menu bar** — CPU, GPU, RAM, disk at a glance; click to open the details window.
- **AI chat** — Ollama in the app or via Discord; supports `FETCH_URL`, `BRAVE_SEARCH`, `PERPLEXITY_SEARCH`, `RUN_CMD`, code execution, MCP.
- **Discord integration** — Bot for chat and task automation.
- **Scheduler** — Recurring tasks and delayed execution.

---

## All Agents – Overview and Behaviour

### Tool agents (what Ollama can invoke)

Ollama can invoke these tools by replying with `TOOL_NAME: <argument>`. The **SCHEDULER** is informational only (cannot be invoked directly).

| Agent | Invocation | Purpose | Implementation |
|-------|-----------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch web page content (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest, 15s timeout). Used by Discord and CPU-window chat. |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Brave Search API with result injection. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY`. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | **CPU window**: frontend runs code in app context and returns result. **Discord/agent**: `commands/ollama.rs` → `run_js_via_node` (Node.js). In some contexts (e.g. Discord) JS may not be executed and the app returns a message instead. |
| **PERPLEXITY_SEARCH** | `PERPLEXITY_SEARCH: <query>` | Perplexity search with result injection. | `commands/perplexity.rs` → `perplexity_search()`; `perplexity/mod.rs` calls Perplexity Search API. Results shaped and injected in `ollama.rs`. Requires `PERPLEXITY_API_KEY` (env, `.config.env`, or Keychain). Only offered when configured. |
| **RUN_CMD** | `RUN_CMD: <command> [args]` | Run allowlisted local commands (read-only). | `commands/run_cmd.rs`. Allowlist from orchestrator skill "## RUN_CMD allowlist" (or default: cat, head, tail, ls, grep, date, whoami, ps, wc, uptime, cursor-agent). Paths must be under `~/.mac-stats`. Disabled when `ALLOW_LOCAL_CMD=0`. See `docs/011_local_cmd_agent.md`. |

---

## Agent Behavior

### Why agents stop: tool loop limit and sequential execution

#### 1. Max tool iterations
- Each agent has a `max_tool_iterations` limit (default **15** in `agent.json`).
- Ollama stops after reaching the limit, returning the last response.
- Log: `Agent router: max tool iterations reached (N), using last response as final`

#### 2. Sequential execution
- Agents run one at a time; no parallel execution.
- Model outputs one tool line per turn, then waits for results.

#### 3. Task files vs. "working on" a task
- `TASK_CREATE` generates a task file under `~/.mac-stats/task/`.
- Task loop is separate: triggered by scheduler or task review loop (every 1 minute).

For the full tool list (all agents, invocation, and implementation), see **docs/agent_workflow.md** (Tool list) and **docs/README.md** (Tool Agents).

---

## Open tasks:

See **006-feature-coder/FEATURE-CODER.md** for the current FEAT backlog.

- ~~**Performance**: Investigate optimizations for sequential execution.~~ Deferred: future/backlog (current sequential tool loop is simple and correct; optimization tracked in FEATURE-CODER when profiling shows a bottleneck).
- ~~**Parallel execution**: Explore feasibility of multi-tool per turn or parallel agent runs.~~ Deferred: future/backlog (Ollama returns one tool per reply; parallel agents would require significant router changes).
- ~~**Documentation**: Update for clarity and completeness.~~ **Done:** Tool table above completed (RUN_JS, PERPLEXITY_SEARCH, RUN_CMD implementation details); See also added for full list. See 006-feature-coder/FEATURE-CODER.md.
- ~~**Discord integration**: Complete description of bot functionality.~~ **Done:** [007_discord_agent.md](007_discord_agent.md) §2 has "Bot functionality at a glance" (triggers, reply pipeline, personalization, session/memory, scheduling, optional features); [docs/README.md](README.md) At a Glance has a one-line Discord bot summary with link to 007.