# Ollama context window, model/params, and skills

This document describes how mac-stats uses **per-model context window size**, **Discord message overrides** (model, temperature, num_ctx), **context-aware content reduction** for FETCH_URL, and **skills** from `~/.mac-stats/skills/` so different agents can respond differently.

---

## 1. Context window size per model

- **Source:** Ollama exposes model details via `POST /api/show` with body `{"name": "model_name"}`. The response includes `parameters` (e.g. `num_ctx 4096`) or `model_info` (e.g. `context_length`).
- **Implementation:** `ollama/mod.rs` defines `ModelInfo { context_size_tokens }` and `get_model_info(endpoint, model_name, api_key)`, which calls `/api/show` and parses context size. Results are **cached** per `(endpoint, model)` so we don’t re-fetch on every request.
- **Startup:** When the app starts, `ensure_ollama_agent_ready_at_startup()` configures the default Ollama client (if not already configured) and calls `get_model_info` for the current model so the context size is known before the first chat.
- **Usage:** The agent pipeline uses this to decide how much content can be injected (e.g. after FETCH_URL) and when to summarize or truncate.

---

## 2. Load different models via Discord

- **Convention:** In a Discord message, use a leading line `model: <name>` or `model=<name>` (case-insensitive). The rest of the message is the question. Example: `model: llama3.2\nWhat is 2+2?`
- **Behaviour:** The app parses the message in `discord/mod.rs` (`parse_discord_ollama_overrides`). If a model override is present, it is validated against `GET /api/tags` (model must exist). Then `answer_with_ollama_and_fetch` uses that model for that request only; the global client config is unchanged.
- **Rust:** `commands/ollama.rs` → `answer_with_ollama_and_fetch(..., model_override, ...)` and `send_ollama_chat_messages(messages, model_override, options_override)`.

---

## 3. Model parameters (temperature, context window size)

- **Ollama API:** `POST /api/chat` accepts an `options` object with e.g. `temperature` and `num_ctx`.
- **Config:** `OllamaConfig` (and `OllamaConfigRequest`) have optional `temperature` and `num_ctx`. When set, they are sent as default options with every chat request unless overridden per request.
- **Discord overrides:** Leading lines in the message (stripped before sending the question):
  - `temperature: 0.7` or `temperature=0.7`
  - `num_ctx: 8192` or `num_ctx=8192`
  - Or one line: `params: temperature=0.7 num_ctx=8192`
- **Rust:** `ollama/mod.rs` → `ChatOptions { temperature, num_ctx }` and `ChatRequest.options`. `send_ollama_chat_messages` merges config defaults with per-request overrides.

---

## 4. Context-aware content reduction (FETCH_URL)

- **Problem:** A fetched page (HTML/text) can be larger than the model’s context window. Injecting it in full can overflow or truncate the reply.
- **Approach:** Before injecting “Here is the page content: …” we estimate token usage (chars/4). We reserve 512 tokens for the reply and subtract the current conversation size. If the page content would exceed the remaining space:
  1. **Summarization (preferred):** One extra Ollama request: “Summarize the following web page content in under N tokens…” with the (possibly pre-truncated) body. The summary is then injected into the main conversation.
  2. **Fallback:** If summarization fails, we truncate the text to fit and append “(content truncated due to context limit).”
- **Rust:** `commands/ollama.rs` → `reduce_fetched_content_to_fit(...)` and the FETCH_URL branch in the tool loop. Uses `model_info.context_size_tokens` from the cache.

---

## 5. Skills (~/.mac-stats/skills/)

- **Purpose:** Different “agents” (behaviours) per request by attaching different system-prompt overlays. Skills are Markdown files that are prepended to the system prompt when selected.
- **Path:** `~/.mac-stats/skills/` (see `Config::skills_dir()` in `config/mod.rs`).
- **Naming:** `skill-<number>-<topic>.md`, e.g. `skill-1-summarize.md`, `skill-2-code.md`. The number and topic are parsed from the filename for listing and selection.
- **Discord:** Leading line `skill: 2` or `skill: code` (case-insensitive for topic). The app loads all skills from the directory, finds the one matching the number or topic, and passes its content as `skill_content` into `answer_with_ollama_and_fetch`. The content is prepended to both the planning and execution system prompts as “Additional instructions from skill: …”.
- **Rust:** `skills.rs` → `load_skills()`, `find_skill_by_number_or_topic()`. `commands/ollama.rs` → `answer_with_ollama_and_fetch(..., skill_content)` injects skill text into the system message.

---

## 6. Ollama agent at startup

- **Behaviour:** The app no longer requires the user to open the CPU window for the Ollama agent to be available. On startup, `ensure_ollama_agent_ready_at_startup()` runs in a background task: if the client is not configured, it applies the default endpoint (`http://localhost:11434`) and model (`llama2`), then checks the connection and fetches model info (context size) for the default model. Discord, the scheduler, and the CPU window can then use the agent immediately when needed.

---

## References

- **All agents overview:** `docs/100_all_agents.md`
- **Discord agent:** `docs/007_discord_agent.md`
- **Ollama API (list, version, pull, delete, embeddings, load/unload):** `docs/015_ollama_api.md`
- **Code:** `src-tauri/src/ollama/mod.rs` (ModelInfo, ChatOptions, get_model_info), `src-tauri/src/commands/ollama.rs` (answer_with_ollama_and_fetch, reduce_fetched_content_to_fit, send_ollama_chat_messages), `src-tauri/src/discord/mod.rs` (parse_discord_ollama_overrides), `src-tauri/src/skills.rs` (load_skills, find_skill_by_number_or_topic), `src-tauri/src/config/mod.rs` (skills_dir).
