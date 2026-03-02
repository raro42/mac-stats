# Plan: Assign local models to agents by capability

## Goal

When connecting to local Ollama:

1. Use the **local model list** returned by Ollama to classify models by capability (vision, understanding, thinking, cheap/expensive, parameter count).
2. **Assign** each agent a model that matches its role (e.g. orchestrator → thinking or general, discord-expert → cheap/fast, senior-coder → code, future vision tasks → vision).

## Is it possible?

**Yes.** The codebase already does most of this:

- **Discovery:** At startup we call `GET /api/tags`, get the full model list (with `details.parameter_size`, `details.family`), and build a `ModelCatalog` in `src-tauri/src/ollama/models.rs`.
- **Classification today:** We already classify by:
  - **Capability:** `Code` vs `General` (from name/family: "coder", "code").
  - **Size:** `Small` (<4B), `Medium` (4–15B), `Large` (>15B) from `parameter_size` or file size.
  - **Cloud vs local:** `is_cloud` to prefer local models.
- **Roles today:** Agents declare `model_role` in `agent.json` (`"code"`, `"general"`, `"small"`). `resolve_agent_models()` maps role → concrete model name at load time.
- **First local model:** We already prefer the **first non-cloud model** for the default and for catalog resolution (`eligible_local()`); no need to “run” a model to evaluate others.

What’s **missing** is extending capabilities and roles so we can assign by **vision**, **thinking/reasoning**, and explicit **cheap/expensive**, and wire those to the right agents.

---

## “Use the first local model to evaluate”

Two interpretations:

1. **Use the model list (no LLM call):** We already do this. We take the list from `/api/tags`, classify each model from **static** signals (name, family, parameter_size), and assign agents from that catalog. No “first local model” is invoked to evaluate others.
2. **Use an LLM to classify models:** One could pick the first local model and run a prompt like “Classify these models: …” to get vision/code/reasoning. That would be slow, brittle, and redundant if we can infer from names/families.

**Recommendation:** Keep **static classification** from name/family/parameter_size and optionally a small **built-in capability table** for well-known tags (e.g. `llava` → vision, `deepseek-r1` → thinking). No LLM-based evaluation step.

---

## Extended capabilities and roles

### 1. Add capability flags (in code)

In `ollama/models.rs`, extend classification:

| Capability   | How to detect (name / family) |
|-------------|--------------------------------|
| **Vision**  | Name/family: `llava`, `bakllava`, `vision`, `pixtral`, `llava`, `minicpm-v` |
| **Reasoning / thinking** | Name/family: `deepseek-r1`, `qwen3`, `thinking`, `reason`, `qwq`, `openreason` (or from a small allowlist) |
| **Code**    | Already: `coder`, `code` |
| **General** | Default when none of the above |

Keep **size** (Small/Medium/Large) and **param count** as today; “cheap” = small/fast, “expensive” = large/slow.

### 2. Add model roles

Extend `model_role` in `agent.json` and `resolve_role()` so that in addition to `code`, `general`, `small` we support:

| Role        | Meaning | Picks |
|------------|---------|--------|
| `vision`   | Needs image input | First local model with Vision capability |
| `thinking` / `reasoning` | Best for planning/reasoning | First local model with Reasoning (or largest general if none) |
| `cheap`    | Alias for `small` | Smallest local (fast, low resource) |
| `expensive`| Prefer larger | Largest eligible local (e.g. general/medium or general/large) |

Existing roles stay: `code`, `general`, `small`.

### 3. Assign agents to roles (recommended mapping)

| Agent / use case        | Suggested `model_role` | Rationale |
|-------------------------|-------------------------|-----------|
| **Orchestrator**        | `thinking` or `general` | Planning and routing; can use reasoning model if available. |
| **Discord expert**      | `general` or `cheap`    | Fast replies; `cheap` = small model. |
| **Senior coder**        | `code`                  | Already set. |
| **Scheduler / task runner** | `general` or `cheap` | Simple task parsing. |
| **Future vision agent** | `vision`                | When we have an agent that interprets screenshots/images. |
| **Default (no agent)**  | First local / `OLLAMA_FAST_MODEL` | Already handled. |

Defaults in `defaults/agents/*/agent.json` can be updated to use `thinking` for orchestrator and `cheap` where appropriate; user overrides (e.g. explicit `model`) continue to take precedence.

---

## Implementation steps (high level)

1. **`ollama/models.rs`**
   - Add capability flags or enum: e.g. `Vision`, `Reasoning`, `Code`, `General` (or a bitfield).
   - In `classify_model()`, set Vision from name/family (llava, vision, pixtral, …), Reasoning from name/family (deepseek-r1, thinking, …).
   - In `ModelCatalog`:
     - Add `pick_vision()`, `pick_reasoning()` (prefer local, then by size as needed).
     - In `resolve_role()`, handle `"vision"`, `"thinking"` (or `"reasoning"`), `"cheap"` (= small), `"expensive"` (= general/large).
   - Keep existing `pick_code`, `pick_general`, `pick_small` and size/code logic.

2. **Agent defaults**
   - Optionally set orchestrator to `model_role: "thinking"` (or keep `"small"` for speed).
   - Optionally set discord-expert / scheduler to `model_role: "cheap"` where desired.

3. **Startup**
   - No change to “first local model” for default or catalog build: we already use the list from `/api/tags` and prefer first local in resolution. No new “evaluation” call.

4. **Docs**
   - Update `docs/100_all_agents.md` or agent docs to list `model_role` options (code, general, small, vision, thinking, cheap, expensive) and how they map to capabilities.

---

## Summary

- **Possible:** Yes. We already discover local models, classify by code/general/size, and assign by `model_role`. We only need to extend capabilities (vision, reasoning) and roles (vision, thinking, cheap, expensive) and point agents at them.
- **“First local model to evaluate”:** Implemented as “use the local model list from Ollama to classify and assign”; no extra LLM evaluation step.
## Implementation status (done)

- **`ollama/models.rs`**: Extended `ModelCapability` with `Vision`, `Reasoning`. `detect_capability()` checks vision (llava, pixtral, etc.), then reasoning (deepseek-r1, thinking, etc.), then code, then general. Added `pick_vision()`, `pick_reasoning()`, `pick_expensive()`; `resolve_role()` handles `vision`, `thinking`/`reasoning`, `cheap`, `expensive`.
- **Agent defaults**: Orchestrator `model_role` → `thinking`; Discord Expert → `cheap`.
- **Docs**: `docs/017_llm_agents.md` documents all `model_role` values.
