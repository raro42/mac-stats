# mac-stats

## Overview

mac-stats is a local AI agent for macOS that provides a range of features, including:

* CPU, GPU, RAM, and disk usage monitoring
* Ollama chat and Discord bot functionality
* Task runner and scheduler
* MCP (Model-Driven Predictive) integration

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

Ollama can invoke the following tool agents:

* **FETCH_URL**: Fetches a web page's body as text.
* **BRAVE_SEARCH**: Performs a web search via Brave Search API.
* **RUN_JS**: Executes JavaScript code.

## AI Tasks Roadmap

### Phase 1: Web Navigation and Extraction

* **Backend Fetch**: Tauri command `fetch_page(url)` in `src-tauri/src/commands/browser.rs`.
* **Tool Protocol**: Ollama can request a page fetch by replying with exactly one line: `FETCH_URL: <full URL>`.
* **Flow**: Handled inside `ollama_chat_with_execution`.

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
* Review and refine the AI tasks roadmap.