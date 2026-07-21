## Installation

### DMG (recommended)
[Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.

### Build from source
```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```
Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

### Gatekeeper workaround
If macOS blocks the app, right-click the DMG → **Open**, then confirm. Or after install: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

## At a Glance

- **Menu bar** — CPU, GPU, RAM, disk at a glance; click to open the details window.
- **AI chat** — Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.

## Tool Agents

Whenever Ollama is asked to decide which agent to use, the app sends the **complete list of active agents**: the invocable tools below plus the **SCHEDULER** (informational; Ollama can recommend it for recurring or delayed tasks but cannot invoke it with a tool line). Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

## Plan: Assign Local Models to Agents by Capability

### Goal

When connecting to local Ollama:

1. Use the **local model list** returned by Ollama to classify models by capability (vision, understanding, thinking, cheap/expensive, parameter count).
2. **Assign** each agent a model that matches its role (e.g. orchestrator → thinking or general, discord-expert → cheap/fast, senior-coder → code, future vision tasks → vision).

### Is it Possible?

**Yes.** The codebase already does most of this:

- **Discovery:** At startup we call `GET /api/tags`, get the full model list (with `details.parameter_size`, `details.family`), and build a `ModelCatalog` in `src-tauri/src/ollama/models.rs`.
- **Classification today:** We already classify by:
  - **Capability:** `Code` vs `General` (from name/family: "coder", "code").
  - **Size:** `Small` (<4B), `Medium` (4–15B), `Large` (>15B) from `parameter_size` or file size.
  - **Cloud vs local:** `is_cloud` to prefer local models.
- **Roles today:** Agents declare `model_role` in `agent.json` (`"code"`, `"general"`, `"small"`). `resolve_agent_models()` maps role → concrete model name at load time.
- **First local model:** We already prefer the **first non-cloud model** for the default and for catalog resolution (`eligible_local()`); no need to “run” a model to evaluate others.

### What’s Missing

- **Extension of capabilities and roles** so we can assign by **vision**, **thinking/reasoning**, and explicit **cheap/expensive**, and wire those to the right agents.

## Extended Capabilities and Roles

### 1. Add Capability Flags (in Code)

In `ollama/models.rs`, extend classification:

| Capability   | How to detect (name / family) |
|-------------|--------------------------------|
| **Vision**  | Name/family: `llava`, `vision`, `pixtral`, `llava`, `minicpm-v` |
| **Reasoning / thinking** | Name/family: `deepseek-r1`, `qwen3`, `thinking`, `reason`, `qwq`, `openreason` (or from a small allowlist) |
| **Code**    | Already: `coder`, `code` |
| **General** | Default when none of the above |

Keep **size** (Small/Medium/Large) and **param count** as today; “cheap” = small/fast, “expensive” = large/slow.

### 2. Add Model Roles

Extend `model_role` in `agent.json` and `resolve_role()` so that in addition to `code`, `general`, `small` we support:

| Role        | Meaning | Picks |
|------------|---------|--------|
| `vision`   | Needs image input | First local model with Vision capability |
| `thinking` / `reasoning` | Best for planning/reasoning | First local model with Reasoning (or largest general if none) |
| `cheap`    | Alias for `small` | Smallest local (fast, low resource) |
| `expensive`| Prefer larger | Largest eligible local (e.g. general/medium or general/large) |

Existing roles stay: `code`, `general`, `small`.

### 3. Assign Agents to Roles (Recommended Mapping)

| Agent / use case        | Suggested `model_role` | Rationale |
|-------------------------|-------------------------|-----------|
| **Orchestrator**        | `thinking` or `general` | Planning and routing; can use reasoning model if available. |
| **Discord expert**      | `general` or `cheap`    | Fast replies; `cheap` = small model. |
| **Senior coder**        | `code`                  | Already set. |
| **Scheduler / task runner** | `general` or `cheap` | Simple task parsing. |
| **Future vision agent** | `vision`                | When we have an agent that interprets screenshots/images. |
| **Default (no agent)**  | First local / `OLLAMA_FAST_MODEL` | Already handled. |

Defaults in `defaults/agents/*/agent.json` can be updated to use `thinking` for orchestrator and `cheap` where appropriate; user overrides (e.g. explicit `model`) continue to take precedence.

## Implementation Steps (High Level)

1. **`ollama/models.rs`**
   - Add capability flags or enum: e.g. `Vision`, `Reasoning`, `Code`, `General` (or a bitfield).
   - In `classify_model()`, set Vision from name/family (llava, vision, pixtral, …), Reasoning from name/family (deepseek-r1, thinking, …).
   - In `ModelCatalog`:
     - Add `pick_vision()`, `pick_reasoning()` (prefer local, then by size as needed).
     - In `resolve_role()`, handle `"vision"`, `"thinking"` (or `"reasoning"`), `"cheap"` (= small), `"expensive"` (= general/large).
   - Keep existing `pick_code`, `pick_general`, `pick_small` and size/code logic.

2. **Agent Defaults**
   - Optionally set orchestrator to `model_role: "thinking"` (or keep `"small"` for speed).
   - Optionally set discord-expert / scheduler to `model_role: "cheap"` where desired.

3. **Startup**
   - No change to “first local model” for default or catalog build: we already use the list from `/api/tags` and prefer first local in resolution. No new “evaluation” call.

4. **Docs**
   - Update `docs/100_all_agents.md` or agent docs to list `model_role` options (code, general, small, vision, thinking, cheap, expensive) and how they map to capabilities.