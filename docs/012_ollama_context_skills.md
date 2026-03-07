## Global Context

### Install

* **DMG (recommended):** [Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.
* **Build from source:**
```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```
Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

* **If macOS blocks the app:** Gatekeeper may show "damaged" or block the unsigned app—the file is fine. Right-click the DMG → **Open**, then confirm. Or after install: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

## At a Glance

* **Menu bar** — CPU, GPU, RAM, disk at a glance; click to open the details window.
* **AI chat** — Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.

## Tool Agents

Whenever Ollama is asked to decide which agent to use, the app sends the **complete list of active agents**: the invocable tools below plus the **SCHEDULER** (informational; Ollama can recommend it for recurring or delayed tasks but cannot invoke it with a tool line). Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

## Ollama Context Window, Model/Params, and Skills

This document describes how mac-stats uses **per-model context window size**, **Discord message overrides** (model, temperature, num_ctx), **context-aware content reduction** for FETCH_URL, and **skills** from `~/.mac-stats/skills/` so different agents can respond differently.

### Context Window Size per Model

- **Source:** Ollama exposes model details via `POST /api/show` with body `{"name": "model_name"}`. The response includes `parameters` (e.g. `num_ctx 4096`) or `model_info` (e.g. `context_length`).
- **Implementation:** `ollama/mod.rs` defines `ModelInfo { context_size_tokens }` and `get_model_info(endpoint, model_name, api_key)`, which calls `/api/show` and parses context size. Results are **cached** per `(endpoint, model)` so we don’t re-fetch on every request.
- **Startup:** When the app starts, `ensure_ollama_agent_ready_at_startup()` configures the default Ollama client (if not already configured) and calls `get_model_info` for the current model so the context size is known before the first chat.
- **Usage:** The agent pipeline uses this to decide how much content can be injected (e.g. after FETCH_URL) and when to summarize or truncate.

### Load Different Models via Discord

- **Convention:** In a Discord message, use a leading line `model: <name>` or `model=<name>` (case-insensitive). The rest of the message is the question. Example: `model: llama3.2\nWhat is 2+2?`
- **Behaviour:** The app parses the message in `discord/mod.rs` (`parse_discord_ollama_overrides`). If a model override is present, it is validated against `GET /api/tags` (model must exist). Then `answer_with_ollama_and_fetch` uses that model for that request only; the global client config is unchanged.
- **Rust:** `commands/ollama.rs` → `answer_with_ollama_and_fetch(..., model_override, ...)` and `send_ollama_chat_messages(messages, model_override, options_override)`.

### When Model Data Is Missing or the Override Is Invalid

- If a request selects a model override that does not exist, validation fails before the main run and the request returns a user-facing error instead of silently falling back to a different model.
- If `/api/show` does not provide a usable context size, the backend falls back to a safe default context budget rather than aborting the request.
- Context-size lookups are cached per `(endpoint, model)` so once the backend learns a model's window, later requests reuse it.

### Model Parameters (Temperature, Context Window Size)

- **Ollama API:** `POST /api/chat` accepts an `options` object with e.g. `temperature` and `num_ctx`.
- **Config:** `OllamaConfig` (and `OllamaConfigRequest`) have optional `temperature` and `num_ctx`. When set, they are sent as default options with every chat request unless overridden per request.
- **Discord overrides:** Leading lines in the message (stripped before sending the question):
  - `temperature: 0.7` or `temperature=0.7`
  - `num_ctx: 8192` or `num_ctx=8192`
  - Or one line: `params: temperature=0.7 num_ctx=8192`
- **Rust:** `ollama/mod.rs` → `ChatOptions { temperature, num_ctx }` and `ChatRequest.options`. `send_ollama_chat_messages` merges config defaults with per-request overrides.

### Custom Overrides in Skill/Context Flows

- Skill-based requests still use the same per-request model/options pipeline as normal chat.
- A Discord request can combine `model:`, `temperature:`, `num_ctx:`, and `skill:` headers; the backend strips those headers, resolves the selected skill, and applies the override/options only to that request.
- Config-level defaults remain the baseline; request-level overrides win when present.

### Context-Aware Content Reduction (FETCH_URL)

- **Problem:** A fetched page (HTML/text) can be larger than the model’s context window. Injecting it in full can overflow or truncate the reply.
- **Approach:** Before injecting “Here is the page content: …” we estimate token usage (chars/4). We reserve 512 tokens for the reply and subtract the current conversation size. If the page content would exceed the remaining space:
  1. **Summarization (preferred):** One extra Ollama request: “Summarize the following web page content in under N tokens…” with the (possibly pre-truncated) body. The summary is then injected into the main conversation.
  2. **Fallback:** If summarization fails, we truncate the text to fit and append “(content truncated due to context limit).”
- **Rust:** `commands/ollama.rs` → `reduce_fetched_content_to_fit(...)` and the FETCH_URL branch in the tool loop. Uses `model_info.context_size_tokens` from the cache.

### Skills (~/.mac-stats/skills/)

- **Purpose:** Different “agents” (behaviours) per request by attaching different system-prompt overlays. Skills are Markdown files that are prepended to the system prompt when selected.
- **Path:** `~/.mac-stats/skills/` (see `Config::skills_dir()` in `config/mod.rs`).
- **Naming:** `skill-<number>-<topic>.md`, e.g. `skill-1-summarize.md`, `skill-2-code.md`. The number and topic are parsed from the filename for listing and selection.
- **Discord:** Leading line `skill: 2` or `skill: code` (case-insensitive for topic). The app loads all skills from the directory, finds the one matching the number or topic, and passes its content as `skill_content` into `answer_with_ollama_and_fetch`. The content is prepended to both the planning and execution system prompts as “Additional instructions from skill: …”.
- **Rust:** `skills.rs` → `load_skills()`, `find_skill_by_number_or_topic()`. `commands/ollama.rs` → `answer_with_ollama_and_fetch(..., skill_content)` injects skill text into the system message.

### Ollama Agent at Startup

- **Behaviour:** The app no longer requires the user to open the CPU window for the Ollama agent to be available. On startup, `ensure_ollama_agent_ready_at_startup()` runs in a background task: if the client is not configured, it applies the default endpoint (`http://localhost:11434`) and auto-detects the first available model (respecting any configured model override), then checks the connection and fetches model info (context size) for the effective model. Discord, the scheduler, and the CPU window can then use the agent immediately when needed.

## References

- **All agents overview:** `docs/100_all_agents.md`
- **Discord agent:** `docs/007_discord_agent.md`
- **Ollama API (list, version, pull, delete, embeddings, load/unload):** `docs/015_ollama_api.md`
- **Code:** `src-tauri/src/ollama/mod.rs` (ModelInfo, ChatOptions, get_model_info), `src-tauri/src/commands/ollama.rs` (answer_with_ollama_and_fetch, reduce_fetched_content_to_fit, send_ollama_chat_messages), `src-tauri/src/discord/mod.rs` (parse_discord_ollama_overrides), `src-tauri/src/skills.rs` (load_skills, find_skill_by_number_or_topic), `src-tauri/src/config/mod.rs` (skills_dir).

## Open tasks:

- Improve Ollama error handling in the skill/context pipeline.
- Improve `FETCH_URL` content reduction performance.