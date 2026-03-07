## Global Context
### README.md snippets
#### mac-stats

[![GitHub release](https://img.shields.io/github/v/release/raro42/mac-stats?include_prereleases&style=flat-square)](https://github.com/raro42/mac-stats/releases/latest)

A local AI agent for macOS: Ollama chat, Discord bot, task runner, scheduler, and MCP—all on your Mac. No cloud, no telemetry. Lives in your menu bar—CPU, GPU, RAM, disk at a glance. Real-time, minimal, there when you look. Built with Rust and Tauri.

### Install

#### DMG (recommended)
[Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.

#### Build from source
```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```
Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

#### Gatekeeper workaround
Right-click the DMG → **Open**, then confirm. Or after install: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

## At a Glance
- **Menu bar** — CPU, GPU, RAM, disk at a glance; click to open the details window.
- **AI chat** — Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.

## Tool Agents
Whenever Ollama is asked to decide which agent to use, the app sends the **complete list of active agents**: the invocable tools below plus the **SCHEDULER** (informational; Ollama can recommend it for recurring or delayed tasks but cannot invoke it with a tool line). Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

## Externalized Prompts
System prompts sent to Ollama are no longer hardcoded in Rust source. They live as editable Markdown files under `~/.mac-stats/` and can be changed at runtime — no rebuild required.

### Files

| File | Purpose | Placeholder |
|------|---------|-------------|
| `~/.mac-stats/agents/soul.md` | Personality and tone rules. Prepended to all system prompts (router and agents). | None |
| `~/.mac-stats/prompts/planning_prompt.md` | Instructions for the planning step (how to produce `RECOMMEND: <plan>`). | None |
| `~/.mac-stats/prompts/execution_prompt.md` | Instructions for the execution step (how to invoke tools, relay results, answer concisely). | `{{AGENTS}}` |

### `{{AGENTS}}` placeholder

The execution prompt contains `{{AGENTS}}` which is replaced at runtime with the dynamically generated tool description list (RUN_JS, FETCH_URL, BRAVE_SEARCH, SCHEDULE, SKILL, RUN_CMD, TASK, OLLAMA_API, PYTHON_SCRIPT, DISCORD_API, AGENT, MCP). This list depends on which tools are enabled/configured at runtime, so it must remain code-generated. Everything else in the prompt is user-editable.

## Defaults
Default content is embedded in the binary via `include_str!` from source files in `src-tauri/defaults/`:

```
src-tauri/defaults/
  agents/
    soul.md
    agent-000/  (orchestrator)
      agent.json, skill.md, testing.md
    agent-001/  (general assistant)
      agent.json, skill.md, testing.md
    agent-002/  (coder)
      agent.json, skill.md, testing.md
    agent-003/  (generalist)
      agent.json, skill.md, testing.md
  prompts/
    planning_prompt.md
    execution_prompt.md
```

On first launch, `Config::ensure_defaults()` writes any missing files. Existing user files are **never overwritten**. To reset a file to its default, delete it and restart the app.

## How the System Prompt is Assembled
The final system prompt sent to Ollama (for both planning and execution steps) is assembled from these parts:

1. **Soul** — loaded from `~/.mac-stats/agents/soul.md`
2. **Discord user context** — injected when request is from Discord (user name, user ID)
3. **Prompt** — `planning_prompt.md` (planning step) or `execution_prompt.md` with `{{AGENTS}}` expanded (execution step)
4. **Plan** — the recommendation from the planning step (execution step only)

The code that assembles this is in `commands/ollama.rs` → `answer_with_ollama_and_fetch()`.

## Tauri Commands (frontend API)
| Command | Arguments | Returns |
|---------|-----------|---------|
| `list_prompt_files` | none | `Vec<{name, path, content}>` for soul, planning_prompt, execution_prompt |
| `save_prompt_file` | `name: String, content: String` | `Ok(())` or error. Name must be `soul`, `planning_prompt`, or `execution_prompt`. |

## Editing Tips
- Changes take effect on the **next request** (prompts are loaded fresh each time).
- Keep `{{AGENTS}}` in the execution prompt — removing it means Ollama won't know about available tools.
- The soul is shared with all agents (as fallback) and with the router. Per-agent souls in `agent-<id>/soul.md` override the shared soul for that agent.
- The planning prompt should instruct the model to reply with `RECOMMEND: <plan>` — the router strips this prefix and uses the rest as the plan.

## References
- **Config**: `src-tauri/src/config/mod.rs` — `load_planning_prompt()`, `load_execution_prompt()`, `ensure_defaults()`
- **Router**: `src-tauri/src/commands/ollama.rs` — prompt loading, `{{AGENTS}}` replacement, system prompt assembly
- **Defaults source**: `src-tauri/defaults/`
- **Agent defaults**: `docs/017_llm_agents.md` (default agents table)
- **All agents**: `docs/100_all_agents.md`