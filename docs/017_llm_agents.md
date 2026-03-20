# mac-stats: Local AI Agent for macOS

## Overview
A local AI agent for macOS with:
- **Menu bar monitoring**: CPU, GPU, RAM, disk usage
- **AI capabilities**: Ollama chat, Discord bot, task runner, scheduler
- **No cloud/telemetry**: All operations run locally
- **Themes**: Data Poster (default), customizable

[![GitHub release](https://img.shields.io/github/v/release/raro42/mac-stats?include_prereleases&style=flat-square)](https://github.com/raro42/mac-stats/releases/latest)

---

## Installation
**Recommended:**  
[Download latest release](https://github.com/raro42/mac-stats/releases/latest) → Drag to Applications

**From source:**
```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
# Or one-liner: curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run
```

**If blocked by Gatekeeper:**  
Right-click DMG → **Open**, or run:  
`xattr -rd com.apple.quarantine /Applications/mac-stats.app`

---

## Core Features
- **Menu bar**: Click to open detailed metrics window
- **AI chat**: Ollama integration (via app or Discord)
- **Discord bot**: Message parsing, agent routing
- **Task automation**: Scheduler, command execution
- **Real-time monitoring**: CPU/GPU/RAM/disk metrics

---

## Agent System

### Tool Agents (Ollama Invokable)
| Agent         | Command              | Purpose                          |
|---------------|----------------------|----------------------------------|
| FETCH_URL     | `FETCH_URL: <URL>`   | Fetch web page content           |
| BRAVE_SEARCH  | `BRAVE_SEARCH: <q>`  | Web search via Brave API         |
| RUN_JS        | `RUN_JS: <code>`     | Execute JavaScript in CPU window |

**Implementation**:  
- `commands/browser.rs` for FETCH_URL  
- `commands/brave.rs` for BRAVE_SEARCH  
- JavaScript execution in CPU window

---

### LLM Agents (Directory-Based)
**Location**: `~/.mac-stats/agents/`  
Each agent has:
- `agent.json`: Model, role, orchestrator status
- `skill.md`: System prompt (task-specific)
- `mood.md`: Tone/context
- `soul.md`: Identity/principles
- `testing.md`: Test prompts

**Model Roles** (resolved at startup):
| Role       | Description                     |
|------------|---------------------------------|
| `code`     | Code-oriented model (coder, etc)|
| `general`  | General-purpose model           |
| `small`    | Smallest/local model            |
| `vision`   | Multimodal model (LLaVA, etc)   |
| `thinking` | Reasoning model (DeepSeek, etc) |
| `expensive`| Largest/local model             |

**model_role resolution logic** (implementation: `agents/mod.rs` → `resolve_agent_models`, `ollama/models.rs` → `ModelCatalog::resolve_role`):

1. **When resolution runs**: At startup, after Ollama is ready, the app fetches `/api/tags`, builds a `ModelCatalog` (classified by capability and size), and caches it. `load_agents()` then calls `resolve_agent_models(agents, catalog)` so each agent gets an effective `model` when possible.
2. **Per-agent order**:
   - If `model` is set in agent.json and that model is **available** in the catalog → use it (explicit override).
   - If `model` is set but **not available** → log warning, then resolve from `model_role` (if set).
   - If `model_role` is set → resolve via catalog (see role→pick rules below). Only **local** (non-cloud) models are chosen; models above 15B params are excluded from auto-selection except for `expensive`.
   - If neither `model` nor `model_role` is set → agent keeps `model: None`; the **global default** (Ollama config) is used at chat time.
3. **Catalog not ready**: If the catalog is not yet populated (e.g. Ollama not reachable at startup), `load_agents()` logs that `model_role` resolution was skipped; agents keep their `model_role` but `model` may stay unset until the next load after the catalog is set.
4. **Role → model choice** (local, ≤15B only unless noted):
   - `code`: best code-capable model, preferring medium size; else general.
   - `general`: best general-capable medium, then general any, then any local; else smallest local above cap.
   - `small` / `cheap`: smallest local model (first in sorted-by-size list).
   - `vision`: best vision-capable local; else general.
   - `thinking` / `reasoning`: best reasoning-capable medium, then any reasoning; else general.
   - `expensive`: largest local (last in sorted list); may use a model above 15B if no smaller local.
   - **Unknown role**: treated as `general` (with a warning in logs).
5. **Cloud models**: Never chosen by role resolution. They are used only when the user sets an explicit `model` in agent.json or configures a cloud model as the default Ollama model.

**Orchestrator Agents**:
- Delegate tasks to specialized agents via `AGENT: <id>`  
- Must include "Router API Commands" in `skill.md`

---

## AGENT: \<selector\> [task] syntax

The **AGENT** tool lets the entry-point model (e.g. orchestrator) delegate work to a specialized LLM agent or to the Cursor Agent CLI. Only the model in the router tool loop can invoke it; sub-agents cannot call AGENT (see **docs/021_router_and_agents.md**).

**Invocation** (exactly one line in the model’s reply):

```text
AGENT: <selector> [task]
```

- **\<selector\>**: Identifies the agent. Resolved in this order (implementation: `agents/mod.rs` → `find_agent_by_id_or_name`):
  1. **Slug** (case-insensitive), e.g. `discord-expert`, `senior-coder`
  2. **Name** (case-insensitive), e.g. `General Assistant`, `Orchestrator`
  3. **Id** (exact), e.g. `000`, `002`
  4. **Id with prefix** `agent-`, e.g. `agent-000`
- **[task]**: Optional. If present, everything after the first space is the task message sent to the agent. If omitted, the **current user question** is used.

**Examples:**

| Reply line | Effect |
|------------|--------|
| `AGENT: 002` | Run agent 002 (Coder) with the current user question as the task. |
| `AGENT: discord-expert list channels in server X` | Run discord-expert with task "list channels in server X". |
| `AGENT: senior-coder refactor this function` | Run agent whose slug is senior-coder with the given task. |

**Special case — cursor-agent:**  
If the selector is `cursor-agent` or `cursor_agent` and the `cursor-agent` CLI is on PATH, the router runs the CLI with the task (or user question) as the prompt and injects its output. This is a proxy agent (no Ollama); see **docs/031_cursor_agent_handoff.md**.

**Behaviour:**  
The router runs the chosen agent in a **single** Ollama request (that agent’s model and prompt; no tool list). The sub-agent’s reply is injected back into the entry-point conversation. Sub-agents cannot use FETCH_URL, TASK_*, SCHEDULE, or another AGENT.

---

## Default Agents
Pre-installed agents (editable by user):
| ID         | Name               | Role                     |
|------------|--------------------|--------------------------|
| `000`      | Orchestrator       | Routes to specialists    |
| `001`      | General Assistant  | General Q&A              |
| `002`      | Coder              | Code generation          |
| `003`      | Generalist         | Fast replies             |
| `004`      | Discord Expert     | Discord API specialist   |
| `005`      | Task Runner        | Task file execution      |

---

## Agent Testing
```bash
mac_stats agent test <selector> [path]
```
- Resolves agent by ID/name/slug
- Uses `testing.md` for prompts
- Simulates `AGENT:` tool invocation

---

## SKILL vs LLM Agents
| Feature         | SKILL                     | LLM Agent                     |
|-----------------|---------------------------|-------------------------------|
| Model           | Shared default model      | Per-agent model               |
| Prompt          | Simple overlay            | Combined soul/mood/skill prompt |
| Use Case        | Lightweight tasks         | Complex workflows, delegation |

---

## Open tasks:
- ~~Clarify `model_role` resolution logic~~ **Done:** § "model_role resolution logic" above (when it runs, per-agent order, catalog-not-ready, role→pick rules, cloud models). Tracked in 006-feature-coder/FEATURE-CODER.md.
- ~~Add documentation for `AGENT: <selector> [task]` syntax~~ **Done:** § "AGENT: \<selector\> [task] syntax" above (invocation, selector resolution order, optional task, cursor-agent proxy, behaviour). Tracked in 006-feature-coder/FEATURE-CODER.md.
- Implement missing `orchestrator` routing examples
- Define fallback behavior for cloud model roles
- Add CLI command for agent reset/defaults
- Document `testing.md` format requirements