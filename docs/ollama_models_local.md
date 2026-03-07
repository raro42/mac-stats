## Global Context
### mac-stats

**The AI agent that just gets it done. All local.**

[![GitHub release](https://img.shields.io/github/v/release/raro42/mac-stats?include_prereleases&style=flat-square)](https://github.com/raro42/mac-stats/releases/latest)

A local AI agent for macOS: Ollama chat, Discord bot, task runner, scheduler, and MCP—all on your Mac. No cloud, no telemetry. Lives in your menu bar—CPU, GPU, RAM, disk at a glance. Real-time, minimal, there when you look. Built with Rust and Tauri.

<img src="screens/data-poster.png" alt="mac-stats Data Poster theme" width="500">

📋 [Changelog](CHANGELOG.md) · 📸 [Screenshots & themes](screens/README.md)

---

## Install

**DMG (recommended):** [Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.

**Build from source:**
```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```
Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

**If macOS blocks the app:** Gatekeeper may show "damaged" or block the unsigned app—the file is fine. Right-click the DMG → **Open**, then confirm. Or after install: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

---

## At a Glance
### Overview

* Menu bar: CPU, GPU, RAM, disk at a glance; click to open the details window.
* AI chat: Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.
* Discord bot: integrated with the app.

---

## Tool Agents
### Invocation

Whenever Ollama is asked to decide which agent to use, the app sends the complete list of active agents. Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

### Agents

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

---

## Local Ollama Models
### Categorization

Review of `ollama list` with rough purpose and size. Exclude the large 30B-class models for default/agent use.

### Do Not Use (big / 30B-class)

| Model | Parameters | Size (disk) | Note |
|-------|------------|-------------|------|
| **openthinker:32b** | 32.8B | 19 GB | Exclude |
| **qwen3-coder:latest** | 30.5B | 18 GB | Exclude |
| **devstral:latest** | 23.6B | 14 GB | Exclude |
| **gpt-oss:20b** | 20.9B | 13 GB | Exclude |
| **huihui_ai/gpt-oss-abliterated:latest** | 20.9B | 13 GB | Exclude |

### Medium (7B–12B) – General and Code

| Model | Parameters | Purpose |
|-------|------------|--------|
| **gemma3:12b** | 12.2B | General |
| **qwen3:latest** | 8.2B | General |
| **command-r7b:latest** | 8.0B | General (Cohere) |
| **qwen2.5-coder:latest** | 7.6B | **Code** (primary code model) |

### Small / Fast (≤3.2B) – Menu Bar, Agents, Quick Tasks

| Model | Parameters | Purpose |
|-------|------------|--------|
| **llama3.2:latest** | 3.2B | General, fast |
| **granite3-dense:latest** | 2.6B | Small, fast |
| **huihui_ai/granite3.2-abliterated:2b** | 2.5B | Small, fast (already used in plan examples) |
| **deepscaler:latest** | 1.8B | Smallest, very fast |

---

## Suggested Use by Role

* Default / orchestrator / Discord: `qwen3:latest` or `llama3.2:latest` (or `command-r7b:latest` if you prefer Cohere).
* Code agent (e.g. agent-002): `qwen2.5-coder:latest` (7.6B; avoid `qwen3-coder:latest` 30B).
* Lightweight / many agents: `huihui_ai/granite3.2-abliterated:2b` or `granite3-dense:latest` or `llama3.2:latest`.
* Never use in config by default: openthinker:32b, qwen3-coder:latest, devstral, gpt-oss:20b, huihui_ai/gpt-oss-abliterated.

*(Generated from local `ollama list` and `ollama show <model>`.)*

---

### Pending Items
* Update README.md with more detailed installation instructions
* Improve documentation for new users
* Investigate better ways to handle Gatekeeper blocking