# Light browser-agent plan (browser-use–style, minimal weight)

## Current state (Feb 2026)

- **BROWSER_SCREENSHOT** is implemented via CDP (`browser_agent/mod.rs`, `headless_chrome`): connect to Chrome on port 9222 (or launch it), navigate to URL, capture PNG, save under `~/.mac-stats/screenshots/`. The agent tool returns the path; when running from Discord, Werner attaches the screenshot to the channel. URL parsing strips trailing punctuation (e.g. `https://example.com.` → `https://example.com`) to avoid 404s from model output. See `docs/027_discord_log_review.md` for CDP log lines and `docs/028_discord_attachments.md` for attachment behaviour.
- The full BROWSER_NAVIGATE / BROWSER_CLICK / BROWSER_FILL / BROWSER_EXTRACT flow (Phase 1 HTTP-only or Phase 2 CDP) is not yet implemented; the plan below still applies for that.

---

## Context

[browser-use](https://github.com/browser-use/browser-use) gives AI agents the ability to automate the web: open a URL, see the page (DOM + optional screenshot), decide actions (click, type, scroll, extract), execute them, repeat. It’s powerful but **heavy**: Python 3.11+, many deps (OpenAI, Anthropic, Google, MCP, etc.), and **Chromium via CDP** (Chrome DevTools Protocol). Install and run cost are high.

**Goal:** Reimplement the same *kind* of capability (task → LLM → browser-like actions → repeat) inside mac-stats, with **minimal install and runtime weight**: no Python, no bundled browser, reuse existing Ollama + agent loop.

---

## What we keep from browser-use (conceptually)

| Concept | browser-use | Our light version |
|--------|--------------|--------------------|
| Task | "Find the number of stars of repo X" | Same (natural language task) |
| Browser | Full Chromium via CDP | Option A: HTTP-only “browser” (fetch + parse HTML). Option B (later): CDP to user’s Chrome. |
| Page representation | DOM + optional screenshot | Parsed links + forms + main text (numbered list for LLM) |
| Actions | click, type, scroll, extract, go_back, etc. | NAVIGATE, CLICK (link index), FILL (form), EXTRACT (text), optional BACK |
| LLM | Many providers + their cloud | **Ollama only** (already in mac-stats) |
| Agent loop | Steps until done or max_steps | Same: plan → tool call → execute → feed back → repeat (reuse existing tool loop) |

---

## What we explicitly avoid (to stay light)

- **No Python** – everything in Rust/Tauri.
- **No bundled Chromium** – no Playwright/CDP browser spawn by default.
- **No vision in v1** – no screenshots, no image encoding, no vision model; text-only page representation.
- **No extra LLM providers** – Ollama only; no new API keys or cloud deps.
- **No new processes** for “browser” in Phase 1 – only HTTP client + HTML parsing.

---

## Architecture (high level)

1. **“Browser” state** (in-memory, per session or per task):
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

4. **Agent loop** – Reuse existing Ollama execution loop (e.g. in `answer_with_ollama_and_fetch`): add the BROWSER_* tools to the tool list; when the model returns `BROWSER_NAVIGATE: https://...`, run the navigate logic, then feed back “Current page: …” and continue.

5. **Page parsing** – Use a small HTML parser in Rust (e.g. `scraper` crate, already common) to:
   - Extract `<a href="...">` (with visible text),
   - Extract `<form action method>` and `<input name type>`, `<button>`.
   - Optionally strip script/style, then “main text” for EXTRACT (e.g. body text or a simple readability-style pass).

No JavaScript execution in Phase 1: we only interpret static HTML. That covers many sites (docs, simple forms, link-heavy pages).

---

## Phases

### Phase 1: HTTP-only “browser” (no Chromium)

- **State**: Add a small `BrowserState` (current_url, parsed links/forms, optional history for BACK).
- **Fetch**: Reuse `commands::browser::fetch_page_content` for GET; add a thin wrapper for form POST if needed.
- **Parse**: New module (e.g. `commands/browser_agent.rs` or `browser/parser.rs`) using `scraper`: links, forms, inputs, buttons → `Vec<Interactable>` with indices.
- **Tools**: Implement BROWSER_NAVIGATE, BROWSER_CLICK, BROWSER_FILL, BROWSER_SUBMIT, BROWSER_EXTRACT; register in agent tool list and in the execution loop (parse “BROWSER_NAVIGATE: …” etc. like FETCH_URL).
- **Prompt**: Extend agent/planning prompt: “You have a lightweight browser. Use BROWSER_NAVIGATE to open a URL; then BROWSER_CLICK to follow a link (by index) or BROWSER_FILL + BROWSER_SUBMIT for forms. BROWSER_EXTRACT to get page text.”
- **Scope**: Single-tab, no cookies/session persistence in v1 (each NAVIGATE is a fresh GET unless we add a minimal cookie jar later).
- **No new binaries, no new runtimes** – only Rust crates (e.g. `scraper`, maybe `html5ever` if we need more).

### Phase 2 (optional): CDP to user’s Chrome

- **Requirement**: User launches Chrome with `--remote-debugging-port=9222` (or we document it). We do **not** spawn Chromium.
- **Client**: Add a small CDP client in Rust (e.g. `headless_chrome` or minimal `cdp` crate) to:
  - Connect to `localhost:9222`,
  - Navigate, get DOM or accessibility tree, optionally screenshot,
  - Execute click (by selector or coordinates), type, scroll.
- **State**: Either keep a “CDP mode” state (current page URL, last snapshot) or reuse the same BROWSER_* tool names and under the hood call CDP when connected, else fall back to Phase 1.
- **Still no bundled browser** – no install of Chromium by us; user brings their own Chrome.

### Phase 3 (optional): Smarter parsing and robustness

- **Readability**: Use a simple “main content” extractor (e.g. heuristic or a small readability port) so BROWSER_EXTRACT returns article text, not full HTML.
- **Cookies/session**: Minimal cookie jar for same-domain requests so login flows work across NAVIGATE/SUBMIT.
- **Rate limiting / politeness**: Max requests per minute, respect robots.txt for optional future crawls.

### Phase 4 (optional): HTTP-only "browser" fallback (no Chromium)

- For environments where Chrome is not available: fetch HTML via HTTP, parse with `scraper` (links, forms), present numbered list to LLM; BROWSER_CLICK = follow link, BROWSER_FILL/SUBMIT = form. No JS execution. See original "Phase 1" description above for details.

---

## File / module layout (proposal)

- **`src-tauri/src/commands/browser.rs`** – Keep as-is for `fetch_page_content` / `fetch_page`.
- **`src-tauri/src/browser_agent/`** (new) or under **`commands/`**:
  - **`mod.rs`** – State (`BrowserState`), tool registration, “run BROWSER_* action” entry.
  - **`parse.rs`** – HTML parsing: links, forms, buttons → `Interactable` list.
  - **`actions.rs`** – Implement NAVIGATE, CLICK, FILL, SUBMIT, EXTRACT (and BACK if done).
- **Agent / Ollama** – In `commands/ollama.rs` (or where tool loop lives): add BROWSER_* to the list of parseable tools; when we get `BROWSER_NAVIGATE: url`, call `browser_agent::navigate(...)`, then append “Current page: …” to the conversation and continue.
- **Prompts** – In `prompts/` or in code: add a short “Light browser tools” section describing BROWSER_NAVIGATE, BROWSER_CLICK, BROWSER_FILL, BROWSER_SUBMIT, BROWSER_EXTRACT (and BROWSER_BACK if implemented).

---

## Dependencies (Phase 1)

- **`headless_chrome`** – Rust CDP client; connect to Chrome via WebSocket, navigate, get page content, run JS in page.
- No bundled browser – user runs Chrome with `--remote-debugging-port=9222` or we spawn it for tests.

---

## Success criteria (Phase 1)

- User can give a task like “Go to https://example.com and tell me the first three link texts.”
- Agent uses BROWSER_NAVIGATE → then BROWSER_EXTRACT (or BROWSER_CLICK by index and repeat).
- No browser window opens; no extra install beyond `cargo build`.
- Same binary as today; feature can be gated by config or by “browser agent” skill if we want.

---

## Out of scope (by design)

- **Captcha / anti-bot** – Not solving these; document that the light agent works best on static/friendly sites.
- **Full JS apps** – Phase 1 (CDP) handles JS-heavy sites; Phase 4 (HTTP fallback) is HTML-only.
- **Cloud browser / stealth** – No equivalent of browser-use cloud; keep everything local and minimal.

---

## References

- browser-use: https://github.com/browser-use/browser-use  
- mac-stats FETCH_URL / agent loop: `src-tauri/src/commands/ollama.rs`, `commands/browser.rs`  
- Agent tools list: `AGENTS.md`, planning/execution prompts in repo  
