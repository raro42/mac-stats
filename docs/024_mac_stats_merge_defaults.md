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
If macOS blocks the app: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

## At a Glance
### Menu Bar
- **CPU, GPU, RAM, disk at a glance**: click to open the details window.
- **AI Chat**: Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.

### Discord Bot
- **Discord bot functionality**: integrated with Ollama chat.

## Tool Agents
### Invocation
Whenever Ollama is asked to decide which agent to use, the app sends the complete list of active agents. Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

### Agents
| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

## Updating Defaults
### Merging Defaults
When updating `.mac-stats` from repo defaults:
- **Do not overwrite** existing files.
- **Merge** instead:
  - **New files / new agents**: Create missing files or agent directories and write default content.
  - **Existing files**: Merge default content into the existing file — e.g. add missing sections (by heading), add new bullets to lists, or append new blocks at the end. Preserve the user’s existing content and customizations.

## Automatic Prompt Merge
At startup, `ensure_defaults()` runs. For **prompt files** (`~/.mac-stats/prompts/planning_prompt.md` and `execution_prompt.md`):
- If the file **does not exist**, the default is written (unchanged).
- If the file **exists**, it is **merged** with the bundled default: the file is split into paragraphs (by `\n\n`); each default paragraph is identified by its first-line key (trimmed, up to 80 chars). Any default paragraph whose key is not already present in the file is **appended**. User content is never overwritten; new sections from repo defaults are added automatically.

## Open tasks:
- Add more documentation for the `ensure_defaults()` function.