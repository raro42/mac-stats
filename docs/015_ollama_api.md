# Ollama API – Full coverage in mac-stats

This document describes the **Ollama HTTP API** operations exposed by mac-stats via Tauri commands and the shared `ollama` module. All operations use the **configured Ollama endpoint** (same as chat, Discord agent, and scheduler). Reference: [Ollama API](https://docs.ollama.com/api/tags) (list models), [get version](https://docs.ollama.com/api-reference/get-version.md), [embed](https://docs.ollama.com/api/embed.md), [pull](https://docs.ollama.com/api/pull.md), [delete](https://docs.ollama.com/api/delete.md), [ps](https://docs.ollama.com/api/ps.md).

**Agent tool:** The LLM (Ollama) can invoke these operations itself via the **OLLAMA_API** agent. In the Discord, scheduler, and CPU-window agent flows, the model is given the OLLAMA_API tool and can reply with e.g. `OLLAMA_API: list_models`, `OLLAMA_API: version`, `OLLAMA_API: pull llama3.2`, `OLLAMA_API: embed nomic-embed-text some text`, etc. See §5 below for the invocation format.

---

## 1. List models

| Command | API | Description |
|--------|-----|-------------|
| `list_ollama_models` | GET /api/tags | Returns model **names only** (existing; backward compatible). |
| `list_ollama_models_full` | GET /api/tags | Returns full list with **details**: name, modified_at, size, digest, details (format, family, parameter_size, quantization_level). |

Frontend can use `list_ollama_models_full` for model management UIs (size, family, quantization).

---

## 2. Version and running models

| Command | API | Description |
|--------|-----|-------------|
| `get_ollama_version` | GET /api/version | Returns Ollama server version string. |
| `list_ollama_running_models` | GET /api/ps | Returns models currently **loaded in memory** (model name, size, digest, details, expires_at, size_vram, context_length). |

---

## 3. Pull, delete, load, unload

| Command | API | Description |
|--------|-----|-------------|
| `pull_ollama_model(model, stream)` | POST /api/pull | Download or **update** a model. `stream: true` consumes NDJSON progress; `stream: false` waits for completion. |
| `delete_ollama_model(model)` | DELETE /api/delete | Remove a model from disk. |
| `load_ollama_model(model, keep_alive?)` | POST /api/generate | **Load (warm)** a model into memory. Optional `keep_alive` e.g. `"5m"`; if omitted, model may unload after default timeout. |
| `unload_ollama_model(model)` | POST /api/chat with keep_alive: 0 | **Unload** a model from memory. Ollama has no dedicated unload endpoint; we send a minimal chat request with `keep_alive: 0`. |

Load/unload: Models load on first use (chat/generate/embed). Explicit load reduces latency for the next request; unload frees VRAM when a model is no longer needed.

---

## 4. Embeddings

| Command | API | Description |
|--------|-----|-------------|
| `ollama_embeddings(model, input, options?)` | POST /api/embed | Generate **vector embeddings** for text. `input`: string or array of strings. `options`: optional `truncate`, `dimensions`. Returns model name, embeddings (array of float arrays), total_duration, load_duration, prompt_eval_count. |

Use for semantic search, RAG, or similarity. Requires an embedding-capable model (e.g. `nomic-embed-text`, `mxbai-embed-large`).

---

## 5. OLLAMA_API agent (tool for the LLM)

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

---

## 6. Tauri command summary

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

---

## 7. Code locations

- **Types and client:** `src-tauri/src/ollama/mod.rs` — `ListResponse`, `ModelSummary`, `VersionResponse`, `PsResponse`, `EmbedInput`, `EmbedRequest`, `EmbedResponse`; `OllamaClient::list_models_full`, `get_version`, `list_running_models`, `pull_model`, `delete_model`, `generate_embeddings`, `unload_model`, `load_model`.
- **Tauri commands:** `src-tauri/src/commands/ollama.rs` — commands above; each clones config from the global client, builds a temporary `OllamaClient`, and calls the corresponding method.
- **Registration:** `src-tauri/src/lib.rs` — all commands registered in `tauri::generate_handler![]`.

---

## 8. References

- **Ollama API index:** https://docs.ollama.com/llms.txt  
- **Context/skills (model, temperature, num_ctx):** `docs/012_ollama_context_skills.md`  
- **All agents overview:** `docs/100_all_agents.md`
