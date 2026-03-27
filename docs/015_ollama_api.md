# mac-stats

## Global Context

A local AI agent for macOS: Ollama chat, Discord bot, task runner, scheduler, and MCP—all on your Mac. No cloud, no telemetry. Lives in your menu bar—CPU, GPU, RAM, disk at a glance. Real-time, minimal, there when you look. Built with Rust and Tauri.

## Install

### DMG (recommended)

*   [Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.

### Build from source

```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```

Or one-liner:

```bash
curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run
```

### If macOS blocks the app

Gatekeeper may show "damaged" or block the unsigned app—the file is fine. Right-click the DMG → **Open**, then confirm. Or after install: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

## At a Glance

-   **Menu bar** — CPU, GPU, RAM, disk at a glance; click to open the details window.
-   **AI chat** — Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.
-   **Discord** — [Discord bot](https://github.com/raro42/mac-stats/discord-bot)

## Tool Agents (What Ollama Can Invoke)

Whenever Ollama is asked to decide which agent to use (planning step in Discord and scheduler flow), the app sends the **complete list of active agents**: the invocable tools below plus the **SCHEDULER** (informational; Ollama can recommend it for recurring or delayed tasks but cannot invoke it with a tool line). Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

## Ollama API – Full Coverage in mac-stats

This document describes the **Ollama HTTP API** operations exposed by mac-stats via Tauri commands and the shared `ollama` module. All operations use the **configured Ollama endpoint** (same as chat, Discord agent, and scheduler). Reference: [Ollama API](https://docs.ollama.com/api/tags) (list models), [get version](https://docs.ollama.com/api-reference/get-version.md), [embed](https://docs.ollama.com/api/embed.md), [pull](https://docs.ollama.com/api/pull.md), [delete](https://docs.ollama.com/api/delete.md), [ps](https://docs.ollama.com/api/ps.md).

## List Models

| Command | API | Description |
|--------|-----|-------------|
| `list_ollama_models` | GET /api/tags | Returns model **names only** (existing; backward compatible). |
| `list_ollama_models_full` | GET /api/tags | Returns full list with **details**: name, modified_at, size, digest, details (format, family, parameter_size, quantization_level). |

Frontend can use `list_ollama_models_full` for model management UIs (size, family, quantization).

**Caching:** Per-endpoint `GET /api/tags` results go through `ollama/model_list_cache.rs`: **5-minute TTL**, **stale-while-revalidate** (after expiry, callers get the last good list immediately while a background refresh runs), **one shared in-flight fetch** per endpoint for concurrent callers, and **no poisoned cache**—empty responses or fetch errors do not overwrite a previous non-empty list. Warnings are logged under `ollama/model_cache` when a stale list is served. Changing Ollama settings clears the cache.

**Circuit breaker:** Ollama HTTP for **`/api/chat`** (all agent/UI/Discord paths via `commands/ollama_chat.rs`) and **`/api/tags`** (`OllamaClient::list_models_full`, `model_list_cache` refresh, connection checks) shares one breaker (`circuit_breaker.rs`, `ollama/mod.rs`): after **3** consecutive infra-style failures (timeouts, connection errors, HTTP 5xx), further calls fail fast until a **30s** reset window allows a single probe. Client-style errors (e.g. `Ollama error: model not found`, HTTP 4xx) do not advance the breaker. Transitions are logged under `mac_stats::circuit`. When the breaker is fully **open**, the menu bar appends a line **`Ollama ✕`**. For manual QA without a real outage, set **`MAC_STATS_DEBUG_FORCE_OPEN_OLLAMA_CIRCUIT=1`** (or `true` / `yes`): Ollama HTTP is blocked immediately and the menu bar shows **`Ollama ✕`**; the process logs once under `circuit` that the debug flag is active (does not exercise the real open/half-open state machine).

**Startup ordering (menu bar app):** `ensure_ollama_agent_ready_at_startup` in `commands/ollama_config.rs` runs to completion on the Tauri async runtime **before** the Discord gateway, scheduler, heartbeat, and task-review threads are spawned (`lib.rs` setup). That avoids racing the first inbound Discord message or due scheduled job against default Ollama config, `GET /api/tags`, and `ModelCatalog` population. Operators can confirm ordering in `~/.mac-stats/debug.log` with target `mac_stats_startup` (e.g. `Ollama startup warmup finished (gate open); spawning Discord…`). Warmup failures are **non-fatal**: one **`WARN`** per failure class (endpoint/model in the message) and automation continues. The first `/api/chat` after process start may perform **one** extra 400 ms cold-start retry when the error looks like a still-starting Ollama (connection refused, model not yet available, etc.); see `ollama::ollama_error_suggests_transient_cold_start` and `OLLAMA_POST_START_COLD_CHAT_RETRY` in `commands/ollama_chat.rs`.

**CPU-window streaming (`stream: true` on `ollama_chat_with_execution`):** NDJSON deltas are coalesced with a short idle window (default **50** ms in `SurfaceChunkPolicy::tauri_ui_default` in `commands/outbound_pipeline.rs`) before emitting **`ollama-chat-chunk`**, reducing event spam. Identical consecutive emitted payloads in the same reply are skipped (dedup). Discord’s ordered outbound behaviour is documented in **`docs/007_discord_agent.md`**.

## Version and Running Models

| Command | API | Description |
|--------|-----|-------------|
| `get_ollama_version` | GET /api/version | Returns Ollama server version string. |
| `list_ollama_running_models` | GET /api/ps | Returns models currently **loaded in memory** (model name, size, digest, details, expires_at, size_vram, context_length). |

## Pull, Delete, Load, Unload

| Command | API | Description |
|--------|-----|-------------|
| `pull_ollama_model(model, stream)` | POST /api/pull | Download or **update** a model. `stream: true` consumes NDJSON progress; `stream: false` waits for completion. |
| `delete_ollama_model(model)` | DELETE /api/delete | Remove a model from disk. |
| `load_ollama_model(model, keep_alive?)` | POST /api/generate | **Load (warm)** a model into memory. Optional `keep_alive` e.g. `"5m"`; if omitted, model may unload after default timeout. |
| `unload_ollama_model(model)` | POST /api/chat with keep_alive: 0 | **Unload** a model from memory. Ollama has no dedicated unload endpoint; we send a minimal chat request with `keep_alive: 0`. |

Load/unload: Models load on first use (chat/generate/embed). Explicit load reduces latency for the next request; unload frees VRAM when a model is no longer needed.

## Embeddings

| Command | API | Description |
|--------|-----|-------------|
| `ollama_embeddings(model, input, options?)` | POST /api/embed | Generate **vector embeddings** for text. `input`: string or array of strings. `options`: optional `truncate`, `dimensions`. Returns model name, embeddings (array of float arrays), total_duration, load_duration, prompt_eval_count. |

Use for semantic search, RAG, or similarity. Requires an embedding-capable model (e.g. `nomic-embed-text`, `mxbai-embed-large`).

## OLLAMA_API Agent (Tool for the LLM)

When using the agent loop (Discord, scheduler, CPU window with tools), the model sees the **OLLAMA_API** agent and can invoke it by replying with exactly one line:

**OLLAMA_API:** `<action>` [args]

| Action | Args | Description |
|--------|------|-------------|
| `list_models` | (none) | Same as `list_ollama_models_full`; returns JSON. |
| `version` | (none) | Same as `get_ollama_version`. |
| `running` | (none) | Same as `list_ollama_running_models`; returns JSON. |
| `pull` | `<model>` [stream true\|false] | Pull or update model; stream defaults to true. |
| `delete` | `<model>` | Delete model from disk. |
| `embed` | `<model>` `<text>` | Generate embeddings for text; returns JSON. |
| `load` | `<model>` [keep_alive] | Load model into memory; optional e.g. `5m`. |
| `unload` | `<model>` | Unload model from memory. |

Example lines: `OLLAMA_API: list_models`, `OLLAMA_API: version`, `OLLAMA_API: pull llama3.2`, `OLLAMA_API: embed nomic-embed-text hello world`, `OLLAMA_API: unload llama3.2`.

## Tauri Command Summary

All commands require Ollama to be **configured** (`configure_ollama`). They use the same endpoint and (if set) API key from Keychain.

| Command | Args | Returns |
|---------|------|--------|
| `list_ollama_models` | — | `Vec<String>` (names) |
| `list_ollama_models_full` | — | `ListResponse` (models with details) |
| `get_ollama_version` | — | `VersionResponse` (version) |
| `list_ollama_running_models` | — | `PsResponse` (running models) |
| `pull_ollama_model` | `model: String`, `stream: bool` | `()` |
| `delete_ollama_model` | `model: String` | `()` |
| `ollama_embeddings` | `model: String`, `input: string \| string[]`, `options?: { truncate?, dimensions? }` | `EmbedResponse` |
| `unload_ollama_model` | `model: String` | `()` |
| `load_ollama_model` | `model: String`, `keep_alive?: String` | `()` |

## Failure Behavior

- If the Ollama client is not configured yet, these commands return a clear error such as `Ollama not configured` instead of attempting the API call.
- If the Ollama server is reachable but a model action fails (for example deleting a missing model), the Tauri command returns the backend error text from the Ollama request path so the caller sees the actual failure instead of a silent no-op.
- The `OLLAMA_API` tool in the agent loop surfaces the same backend result text back to the model/user.

## Code Locations

- **Types and client:** `src-tauri/src/ollama/mod.rs` — `ListResponse`, `ModelSummary`, `VersionResponse`, `PsResponse`, `EmbedInput`, `EmbedRequest`, `EmbedResponse`; `OllamaClient::list_models_full`, `get_version`, `list_running_models`, `pull_model`, `delete_model`, `generate_embeddings`, `unload_model`, `load_model`.
- **Tauri commands:** `src-tauri/src/commands/ollama.rs` — commands above; each clones config from the global client, builds a temporary `OllamaClient`, and calls the corresponding method.
- **Registration:** `src-tauri/src/lib.rs` — all commands registered in `tauri::generate_handler![]`.

## References

- **Ollama API index:** https://docs.ollama.com/llms.txt  
- **Context/skills (model, temperature, num_ctx):** `docs/012_ollama_context_skills.md`  
- **All agents overview:** `docs/100_all_agents.md`

## Open tasks:

See **006-feature-coder/FEATURE-CODER.md** for the current FEAT backlog.

- ~~Improve the user interface for model management and configuration.~~ **Done:** Settings → Ollama tab in the dashboard (endpoint URL, model dropdown via Refresh models, Apply). Backend: `get_ollama_config`, `list_ollama_models_at_endpoint` (see 006-feature-coder/FEATURE-CODER.md).
- ~~Consider support for more advanced model-management features such as fine-tuning or export.~~ Deferred: future/backlog (fine-tuning and export are Ollama CLI features, not in scope for mac-stats).