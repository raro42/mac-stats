## Global Context

### Overview

mac-stats is a local AI agent for macOS, providing a range of features including Ollama chat, Discord bot, task runner, scheduler, and MCP. The app is built with Rust and Tauri.

### Installation

#### Recommended Method

*   Download the latest release from the GitHub repository: [Download latest release](https://github.com/raro42/mac-stats/releases/latest)
*   Drag the downloaded DMG file to the Applications folder.

#### Building from Source

```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```

Or use a one-liner:

```bash
curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run
```

#### Workaround for Gatekeeper Blocking

*   Right-click the DMG file and select **Open**.
*   Alternatively, after installation, run the following command to bypass Gatekeeper:

```bash
xattr -rd com.apple.quarantine /Applications/mac-stats.app
```

## At a Glance

### Menu Bar

*   Displays CPU, GPU, RAM, and disk usage at a glance.
*   Clicking the menu bar item opens the details window.

### AI Chat

*   Ollama is available in the app or via Discord.
*   Ollama can execute various commands, including:
    *   FETCH_URL (fetches a web page's body as text)
    *   BRAVE_SEARCH (performs a web search via Brave Search API)
    *   RUN_JS (executes JavaScript code)

## All Agents – Overview and Behavior

This document describes the agentic behavior in mac-stats, including the tools that Ollama can invoke and how each entry-point agent works.

### Tool Agents

Whenever Ollama is asked to decide which agent to use, the app sends the complete list of active agents. Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

### Available Tool Agents

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text | `commands/browser.rs` → `fetch_page_content()` |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API | `commands/brave.rs` → `brave_web_search()` |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript | In **CPU window**: executed in

## Brave Search Agent

The Brave Search agent lets Ollama answer questions using live web search via the Brave Search API.

### Overview

*   Agent name: BRAVE_SEARCH
*   Invocation: Ollama replies with one line: `BRAVE_SEARCH: <search query>`
*   The app calls the Brave Search API, then injects the results back into the conversation so Ollama can summarize or answer.

### Setup

1.  Get an API key
    *   Subscribe at [Brave Search API](https://api-dashboard.search.brave.com/) (free tier available) and create an API key in the dashboard.
2.  Provide the key
    *   **BRAVE_API_KEY** environment variable.
    *   **.config.env** in the current working directory, in `src-tauri/` (when run from repo root), or in `~/.mac-stats/.config.env`. Use either variable name:
        ```bash
        BRAVE_API_KEY=your_brave_api_key_here
        # or (in .config.env only):
        BRAVE-API-KEY=your_brave_api_key_here
        ```

### Authentication

The Brave Search API uses a subscription token in the request header:

*   **Header**: `X-Subscription-Token: YOUR_API_KEY`
*   **Endpoint**: `GET https://api.search.brave.com/res/v1/web/search?q=<query>`

### Rate Limiting

The Brave Search API uses a 1-second sliding window and returns rate limit info in response headers. The app logs these so you can monitor usage:

*   **On every response**: `Brave agent: rate limit — limit=... remaining=... reset_sec=... policy=...` (from `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`, `X-RateLimit-Policy`).
*   **On 429 (rate limited)**: `Brave agent: rate limited (429). Retry after N seconds (see X-RateLimit-Reset).`

### Security

*   The API key is read only from the environment or `.config.env`; it is not logged or exposed in the UI.
*   Do not commit `.config.env` or expose the key in client-side code or public repos.

### Where it’s Used

*   **Discord bot**: When a user asks something that benefits from web search, Ollama can output `BRAVE_SEARCH: <query>`. The app runs the search and gives the results back to Ollama for the reply.
*   **Ollama agent flow**: Same pipeline as Discord (planning step + tool loop); BRAVE_SEARCH is one of the three agents (with FETCH_URL and RUN_JS).

### Implementation

*   **Module**: `src-tauri/src/commands/brave.rs`
*   **Key resolution**: `get_brave_api_key()` — env, then `.config.env` (cwd, then `~/.mac-stats/.config.env`).
*   **Search**: `brave_web_search(query, api_key)` — async HTTP request, parses `web.results` (title, url, description), returns formatted text for Ollama.

## Open tasks:

*   Improve the Brave Search API documentation for better user experience.
*   Review Brave-search-specific error handling and edge cases.