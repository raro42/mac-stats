# Local Ollama models – categorization (do not use big 30B)

Review of `ollama list` with rough purpose and size. **Exclude** the large 30B-class models for default/agent use.

---

## Do not use (big / 30B-class)

| Model | Parameters | Size (disk) | Note |
|-------|------------|-------------|------|
| **openthinker:32b** | 32.8B | 19 GB | Exclude |
| **qwen3-coder:latest** | 30.5B | 18 GB | Exclude |
| **devstral:latest** | 23.6B | 14 GB | Exclude |
| **gpt-oss:20b** | 20.9B | 13 GB | Exclude |
| **huihui_ai/gpt-oss-abliterated:latest** | 20.9B | 13 GB | Exclude |

---

## Medium (7B–12B) – general and code

| Model | Parameters | Purpose |
|-------|------------|--------|
| **gemma3:12b** | 12.2B | General |
| **qwen3:latest** | 8.2B | General |
| **command-r7b:latest** | 8.0B | General (Cohere) |
| **qwen2.5-coder:latest** | 7.6B | **Code** (primary code model) |

---

## Small / fast (≤3.2B) – menu bar, agents, quick tasks

| Model | Parameters | Purpose |
|-------|------------|--------|
| **llama3.2:latest** | 3.2B | General, fast |
| **granite3-dense:latest** | 2.6B | Small, fast |
| **huihui_ai/granite3.2-abliterated:2b** | 2.5B | Small, fast (already used in plan examples) |
| **deepscaler:latest** | 1.8B | Smallest, very fast |

---

## Suggested use by role

- **Default / orchestrator / Discord:** `qwen3:latest` or `llama3.2:latest` (or `command-r7b:latest` if you prefer Cohere).
- **Code agent (e.g. agent-002):** `qwen2.5-coder:latest` (7.6B; avoid `qwen3-coder:latest` 30B).
- **Lightweight / many agents:** `huihui_ai/granite3.2-abliterated:2b` or `granite3-dense:latest` or `llama3.2:latest`.
- **Never use in config by default:** openthinker:32b, qwen3-coder:latest, devstral, gpt-oss:20b, huihui_ai/gpt-oss-abliterated.

*(Generated from local `ollama list` and `ollama show <model>`.)*
