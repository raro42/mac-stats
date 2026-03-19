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
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript in CPU window. | In **CPU window**: executed in... *(incomplete documentation)* |
| **PERPLEXITY_SEARCH** | `PERPLEXITY_SEARCH: <query>` | Perplexity search with result injection. | *(Implementation details missing)* |
| **RUN_CMD** | `RUN_CMD: <command>` | Run terminal commands. | *(Implementation details missing)* |

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

---

## Open tasks:

- **Performance**: Investigate optimizations for sequential execution.  
- **Parallel execution**: Explore feasibility of multi-tool per turn or parallel agent runs.  
- **Documentation**: Update for clarity and completeness.  
- ~~**Discord integration**: Complete description of bot functionality.~~ **Done:** [007_discord_agent.md](007_discord_agent.md) §2 has "Bot functionality at a glance" (triggers, reply pipeline, personalization, session/memory, scheduling, optional features); [docs/README.md](README.md) At a Glance has a one-line Discord bot summary with link to 007.