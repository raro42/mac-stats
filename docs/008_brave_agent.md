# Brave Search Agent

The Brave Search agent lets Ollama (and the Discord bot) answer questions using live web search via the [Brave Search API](https://api.search.brave.com/).

## Overview

- **Agent name**: BRAVE_SEARCH  
- **Invocation**: Ollama replies with one line: `BRAVE_SEARCH: <search query>`  
- The app calls the Brave Search API, then injects the results (titles, URLs, snippets) back into the conversation so Ollama can summarize or answer.

## Setup

1. **Get an API key**  
   Subscribe at [Brave Search API](https://api-dashboard.search.brave.com/) (free tier available) and create an API key in the dashboard.

2. **Provide the key** (checked in this order):
   - **BRAVE_API_KEY** environment variable.
   - **.config.env** in the current working directory, in `src-tauri/` (when run from repo root), or in `~/.mac-stats/.config.env`. Use either variable name:
     ```bash
     BRAVE_API_KEY=your_brave_api_key_here
     # or (in .config.env only):
     BRAVE-API-KEY=your_brave_api_key_here
     ```

If no key is found, the agent reports “Brave Search is not configured” and Ollama is asked to answer without search results.

## Authentication

The Brave Search API uses a subscription token in the request header:

- **Header**: `X-Subscription-Token: YOUR_API_KEY`
- **Endpoint**: `GET https://api.search.brave.com/res/v1/web/search?q=<query>`

See [Brave Search API – Authentication](https://api-dashboard.search.brave.com/documentation/guides/authentication#code-examples) for details.

## Rate limiting

The Brave Search API uses a [1-second sliding window](https://api-dashboard.search.brave.com/documentation/guides/rate-limiting) and returns rate limit info in response headers. The app logs these so you can monitor usage:

- **On every response**: `Brave agent: rate limit — limit=... remaining=... reset_sec=... policy=...` (from `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`, `X-RateLimit-Policy`).
- **On 429 (rate limited)**: `Brave agent: rate limited (429). Retry after N seconds (see X-RateLimit-Reset).`

Use `-v` or check `~/.mac-stats/debug.log` to see Brave agent usage and remaining quota.

## Security

- The API key is read only from the environment or `.config.env`; it is not logged or exposed in the UI.
- Do not commit `.config.env` or expose the key in client-side code or public repos.

## Where it’s used

- **Discord bot**: When a user asks something that benefits from web search, Ollama can output `BRAVE_SEARCH: <query>`. The app runs the search and gives the results back to Ollama for the reply.
- **Ollama agent flow**: Same pipeline as Discord (planning step + tool loop); BRAVE_SEARCH is one of the three agents (with FETCH_URL and RUN_JS).

## Implementation

- **Module**: `src-tauri/src/commands/brave.rs`
- **Key resolution**: `get_brave_api_key()` — env, then `.config.env` (cwd, then `~/.mac-stats/.config.env`).
- **Search**: `brave_web_search(query, api_key)` — async HTTP request, parses `web.results` (title, url, description), returns formatted text for Ollama.
