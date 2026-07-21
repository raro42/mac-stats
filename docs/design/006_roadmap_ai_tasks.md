# mac-stats — AI tasks roadmap

For the **full product overview** (what ships today), see the root **[README.md](../README.md)**. This doc tracks **tool agents** and **future channel** ideas.

## Overview

mac-stats is a local AI agent for macOS that provides a range of features, including:

* CPU, GPU, RAM, and disk usage monitoring
* Ollama chat and Discord bot functionality
* Task runner and scheduler
* MCP integration

## Installation

### DMG (Recommended)

Download the latest release from [GitHub](https://github.com/raro42/mac-stats/releases/latest) and drag the app to the Applications folder.

### Build from Source

```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```
Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

## At a Glance

* **Menu Bar**: Displays CPU, GPU, RAM, and disk usage at a glance. Click to open the details window.
* **AI Chat**: Ollama chat and Discord bot functionality.
* **Discord Agent**: Handles Discord DMs and @mentions.

## Tool Agents

Ollama invokes tools by replying with one line: `TOOL_NAME: <argument>`. The app sends the full list of active agents (including **SCHEDULER** as informational only). Implemented tools include:

* **FETCH_URL**: Fetch a web page's body as text (server-side, no CORS). `commands/browser.rs`.
* **BRAVE_SEARCH**: Web search via Brave Search API. `commands/brave.rs`; requires `BRAVE_API_KEY`.
* **PERPLEXITY_SEARCH**: Perplexity API search; results injected for Ollama to summarize.
* **RUN_CMD**: Run allowlisted shell commands (e.g. `ps`, `wc`, `uptime`, `cursor-agent`). See `docs/033_docs_vs_code_review.md` for allowlist.
* **RUN_JS**: Execute JavaScript (e.g. in CPU window).
* **BROWSER_SCREENSHOT**: Open URL via CDP, save PNG to `~/.mac-stats/screenshots/`; used by Discord and chat.
* **SCHEDULER**: Informational; Ollama can recommend recurring/one-shot tasks; actual add/remove via SCHEDULE / REMOVE_SCHEDULE.

For the full table and invocation details, see **docs/README.md** (Tool Agents) and **docs/007_discord_agent.md**.

## AI Tasks Roadmap

### Phase 1: Web Navigation and Extraction (done)

* **Backend Fetch**: Tauri command `fetch_page` in `src-tauri/src/commands/browser.rs`; `FETCH_URL: <url>` in chat and Discord.
* **BROWSER_SCREENSHOT**: CDP-based screenshot to `~/.mac-stats/screenshots/`; invoked via `BROWSER_SCREENSHOT: <url>`.

### Phase 2 and Beyond

* **Mail**: IMAP/OAuth integration via a dedicated module.
* **WhatsApp**: WhatsApp Business API or unofficial APIs.
* **Google Docs**: Google APIs (OAuth) integration.

## Discord Agent

* **Module**: `src-tauri/src/discord/mod.rs`.
* **Credentials**: Discord Bot Token in Keychain (`discord_bot_token`).
* **Reply Pipeline**: Discord handler uses the shared "answer with Ollama + fetch" API.

## Ollama Connection / Session Handling

* **Current Behaviour**: A single `OllamaClient` is stored when the user configures Ollama.
* **Done**: `send_ollama_chat_messages` now uses the stored `OllamaClient`'s HTTP client (via `OllamaClient::http_client()`) instead of creating a new `reqwest::Client` per request. The stored client is built with the app's chat timeout (`Config::ollama_chat_timeout_secs()`) via `OllamaConfig::timeout_secs`.

## Open tasks

See **006-feature-coder/FEATURE-CODER.md** for the current FEAT backlog. Remaining items:

* Implement Mail, WhatsApp, and Google Docs integrations.