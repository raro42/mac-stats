# mac-stats

## Install

### DMG (recommended)
[Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.

### Build from source
```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```
Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

### If macOS blocks the app:
Gatekeeper may show "damaged" or block the unsigned app—the file is fine. Right-click the DMG → **Open**, then confirm. Or after install: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

## At a Glance

- **Menu bar** — CPU, GPU, RAM, disk at a glance; click to open the details window.
- **AI chat** — Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.
- **Discord**

## Tool Agents (what Ollama can invoke)

Whenever Ollama is asked to decide which agent to use (planning step in Discord and scheduler flow), the app sends the **complete list of active agents**: the invocable tools below plus the **SCHEDULER** (informational; Ollama can recommend it for recurring or delayed tasks but cannot invoke it with a tool line). Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

## Light Browser-Agent Plan (browser-use–style, minimal weight)

### Current State (Feb 2026)

- **BROWSER_SCREENSHOT** is implemented via CDP (`browser_agent/mod.rs`, `headless_chrome`): connect to Chrome on port 9222 (or launch it), navigate to URL, capture PNG, save under `~/.mac-stats/screenshots/`. The agent tool returns the path; when running from Discord, Werner attaches the screenshot to the channel. URL parsing strips trailing punctuation (e.g. `https://example.com.` → `https://example.com`) to avoid 404s from model output. See `docs/027_discord_log_review.md` for CDP log lines and `docs/028_discord_attachmentsDone.md` for attachment behaviour.

### Browser Automation (CDP) — User Guide

- **What mac-stats needs:** Chrome on port 9222. Either start Chrome yourself with `--remote-debugging-port=9222`, or let mac-stats **launch** Chrome on 9222 when nothing is listening. See **`docs/029_browser_automation.md`** for details.
- In chat (CPU window or Discord) you can say e.g. "Go to google.com and search for XYZ"; the agent will use BROWSER_NAVIGATE, then BROWSER_CLICK (e.g. cookie consent by index), then BROWSER_INPUT (search box index and text), then BROWSER_CLICK (submit). The same session and tab are reused for the whole conversation. Cookie consent and search flows are supported by clicking by index.

## Architecture (High Level)

1. **“Browser” State** (in-memory, per session or per task):
   - `current_url: String`
   - `page_content: String` (raw HTML or cleaned text)
   - `parsed_interactables: Vec<Interactable>` (links, form fields, buttons), each with an index for the LLM.

2. **Interactable** (minimal):
   - Link: `{ index, href, text }`
   - Form: `{ index, action, method, fields: [{ name, type, value? }] }`
   - Button: `{ index, type (submit/button), name?, text }`

3. **Tools** (new, alongside existing FETCH_URL / BRAVE_SEARCH / RUN_JS / etc.):
   - **BROWSER_NAVIGATE** `url` – Set current URL, fetch page, parse into links/forms, store in state, return summarized “page” (URL + numbered list) to LLM.
   - **BROWSER_CLICK** `index` – Resolve index to link or button; if link, treat as NAVIGATE(href); if submit button, submit form (POST/GET).
   - **BROWSER_FILL** `index_or_selector, value` – Fill a form field by index (or simple selector); update in-memory form state.
   - **BROWSER_SUBMIT** `form_index` – Submit form by index (build request from current field values).
   - **BROWSER_EXTRACT** `query?` – Return extracted text (e.g. “all visible text” or simple selector); no screenshot.
   - **BROWSER_BACK** – (Optional) Keep a short history stack, “go back” = previous URL + re-fetch/parse.

4. **Agent Loop** – Reuse existing Ollama execution loop (e.g. in `answer_with_ollama_and_fetch`): add the BROWSER_* tools to the tool list; when the model returns `BROWSER_NAVIGATE: https://...`, run the navigate logic, then feed back “Current page: …” and continue.

5. **Page Parsing** – Use a small HTML parser in Rust (e.g. `scraper` crate, already common) to:
   - Extract `<a href="...">` (with visible text),
   - Extract `<form action method>` and `<input name type>`, `<button>`.
   - Optionally strip script/style, then “main text” for EXTRACT (e.g. body text or a simple readability-style pass).

## Phases

### Phase 1: HTTP-only “browser” (no Chromium)

- **State**: Add a small `BrowserState` (current_url, parsed links/forms, optional history for BACK).
- **Fetch**: Reuse `commands::browser::fetch_page_content` for GET; add a thin wrapper for form POST if needed.
- **Parse**: New module (e.g. `commands/browser_agent.rs` or `browser/parser.rs`) using `scraper`: links, forms, inputs, buttons → `Vec<Interactable>` with indices.
- **Tools**: Implement BROWSER_NAVIGATE, BROWSER_CLICK, BROWSER_FILL, BROWSER_SUBMIT, BROWSER_EXTRACT; register in agent tool list and in the execution loop (parse “BROWSER_NAVIGATE: …” etc. like FETCH_URL).
- **Prompt**: Extend agent/planning prompt: “You have a lightweight browser. Use BROWSER_NAVIGATE to open a URL; then BROWSER_CLICK to follow a link (by index) or BROWSER_FILL + BROWSER_SUBMIT for forms. BROWSER_EXTRACT to get page text.”
- **Scope**: Single-tab, no cookies/session persistence in v1 (each NAVIGATE is a fresh GET unless we add a minimal cookie jar later).
- **No new binaries, no new runtimes** – only Rust crates (e.g. `scraper`, maybe `html5ever` if we need more).

### Phase 2 (Optional): CDP to User’s Chrome

- **Requirement**: User launches Chrome with `--remote-debugging-port=9222` (or we document it). We do **not** spawn Chromium.
- **Client**: Add a small CDP client in Rust (e.g. `headless_chrome` or minimal `cdp` crate) to:
  - Connect to `localhost:9222`,
  - Navigate, get DOM or accessibility tree, optionally screenshot,
  - Execute click (by selector or coordinates), type, scroll.
- **State**: Either keep a “CDP mode” state (current page URL, last snapshot) or reuse the same BROWSER_* tool names and under the hood call CDP when connected, else fall back to Phase 1.
- **Still no bundled browser** – no install of Chromium by us; user brings their own Chrome.

### Phase 3 (Optional): Smarter Parsing and Robustness

- **Readability**: Use a simple “main content” extractor (e.g. heuristic or a small readability port) so BROWSER_EXTRACT returns article text, not full HTML.
- **Cookies/session**: Minimal cookie jar for same-domain requests so login flows work across NAVIGATE/SUBMIT.
- **Rate limiting / politeness**: Max requests per minute, respect robots.txt for optional future crawls.

### Phase 4 (Optional): HTTP-only "browser" Fallback (no Chromium) — **Implemented**

- When CDP/Chrome is not available: BROWSER_NAVIGATE/CLICK/INPUT/EXTRACT fall back to HTTP fetch + `scraper` (links, forms, body text). BROWSER_CLICK follows links or submits forms; BROWSER_INPUT fills form fields. No JS execution. State in `browser_agent::http_fallback`.