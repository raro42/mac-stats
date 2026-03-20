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

**Cloud model as default — fallback behavior** (implementation: `commands/ollama.rs` → entry-point model selection, `agents/mod.rs` → `resolve_agent_models`):

When the user configures a cloud model (e.g. `qwen3.5:cloud`) as the default Ollama model, the following rules apply:

| Scenario | Agent config | Behavior |
|----------|-------------|----------|
| Cloud default + `model_role` set + local models exist | `model_role: "general"` | Role resolves to best local model (cloud excluded). Agent uses local model. |
| Cloud default + `model_role` set + **no** local models | `model_role: "general"` | Role resolution finds nothing (all cloud). Agent warned; `model` stays `None`. At chat time, entry-point path prefers first local model; if none, uses cloud default. |
| Cloud default + explicit `model` set (cloud) | `model: "gpt-4o"` | Used as-is if available in catalog. No role resolution needed. |
| Cloud default + explicit `model` set (local) | `model: "qwen3:latest"` | Used as-is if available. Falls to `model_role` if not found. |
| Cloud default + no `model` or `model_role` | (empty) | Entry-point (`answer_with_ollama_and_fetch`): prefers first local model when default is cloud (lines 3731–3739 in ollama.rs). Agent delegation (`run_agent_ollama_session`): uses global default (cloud). |

**Key points:**
- The entry-point (Discord, scheduler, in-app chat) always prefers a local model over a cloud default to avoid requiring ollama.com authentication for automated flows.
- Sub-agent calls via `AGENT: <selector>` use the agent's resolved `model` (usually local from role resolution). When `model` is `None`, the global default (possibly cloud) is used directly — no local-preference override.
- When role resolution silently fails because only cloud models exist, a warning is logged: "no model found for role 'X', leaving unset; cloud default will be used at chat time".

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

## Agent Reset

Reset agent files to bundled defaults (overwrites agent.json, skill.md, testing.md, soul.md):

```bash
mac_stats agent reset-defaults          # Reset all default agents
mac_stats agent reset-defaults 000      # Reset only agent 000 (Orchestrator)
mac_stats agent reset-defaults 002      # Reset only agent 002 (Coder)
```

- Overwrites **all** files for the agent (including agent.json, which `ensure_defaults` normally preserves)
- When resetting all agents, also resets the shared `soul.md`
- User-created agents (ids not in the bundled defaults) are not affected
- User-created files like `mood.md` or `memory.md` are not touched (only bundled files are written)

---

## SKILL vs LLM Agents
| Feature         | SKILL                     | LLM Agent                     |
|-----------------|---------------------------|-------------------------------|
| Model           | Shared default model      | Per-agent model               |
| Prompt          | Simple overlay            | Combined soul/mood/skill prompt |
| Use Case        | Lightweight tasks         | Complex workflows, delegation |

---

## testing.md format

Each agent directory (`~/.mac-stats/agents/agent-<id>/`) contains a `testing.md` file with test prompts for the `mac_stats agent test` CLI command.

### File structure

The file uses Markdown `## ` headers to delimit test sections. Each section is one test prompt:

```markdown
## Test: short description
The actual prompt text sent to the agent.
Can span multiple lines.

## Test: another scenario
Another prompt.
```

**Parsing rules** (implementation: `agents/cli.rs` → `parse_testing_md`):

1. The file is split on `## ` boundaries.
2. For each section, the **first line** (the header text) is discarded — only the **body** (everything after the first newline) is sent as the prompt.
3. If the file contains no `## ` headers, the entire file content is treated as a single prompt.
4. Empty sections (header with no body) are skipped.

### Conventions

- **Header naming**: Use `## Test: <description>` for consistency (not required by the parser, but makes the file scannable).
- **Expected behavior**: Optionally include an `Expected:` line in the body to document what the agent should do. The line is part of the prompt sent to the agent but serves as documentation for humans reviewing test results.
- **Keep prompts focused**: Each section should test one behavior. For orchestrators, test delegation; for specialists, test their domain.

### Running tests

```bash
mac_stats agent test <selector>                 # Uses agent's testing.md
mac_stats agent test <selector> /path/to/file   # Uses custom test file
```

- **Timeout**: 45 seconds per prompt (default). Override with `MAC_STATS_AGENT_TEST_TIMEOUT_SECS` env var or `agentTestTimeoutSecs` in `config.json` (range: 5–300).
- **Selector**: Agent ID (`000`), slug (`orchestrator`), or name (`Orchestrator`).
- **Output**: Each prompt logs response length; final line reports pass/fail count. Full output in `~/.mac-stats/debug.log`.

### Example: specialist agent (Redmine)

```markdown
## Test: review ticket
Review Redmine ticket 7209.
Expected: REDMINE_API: GET /issues/7209.json?include=journals,attachments then summary.

## Test: search issues
Search Redmine for "monitoring".
Expected: REDMINE_API: GET /search.json?q=monitoring&issues=1&limit=100 then summarize.
```

### Example: orchestrator

```markdown
## Test: delegation
I need a Python function that returns the first N Fibonacci numbers. Delegate to the coder agent.
Expected: AGENT: senior-coder <task> or AGENT: 002 <task>.

## Test: direct answer
What is 2 + 2? Answer in one word.
Expected: Direct answer (no delegation needed for trivial questions).
```

---

## Orchestrator routing examples

The orchestrator (`agent-000`) delegates requests to specialist agents using `AGENT: <selector> [task]`. Below are routing patterns the orchestrator should follow.

### Routing table

| User request pattern | Expected routing | Rationale |
|----------------------|-----------------|-----------|
| Code generation / refactoring | `AGENT: senior-coder <task>` | Coder agent (002) has code-oriented model and prompt. |
| Discord API queries (find user, list channels) | `AGENT: discord-expert <task>` | Discord expert (004) has bot token and API access. |
| Redmine tickets (review, search, create, update) | `AGENT: redmine <task>` | Redmine agent (006) uses REDMINE_API tool. |
| Task/schedule management | `AGENT: scheduler <task>` | Task runner (005) handles TASK_* and SCHEDULE commands. |
| General Q&A, trivia, quick facts | Direct answer | No delegation needed for simple questions. |
| Web research (summarize page, search) | Direct answer using FETCH_URL / BRAVE_SEARCH | Orchestrator can invoke tools directly for one-shot fetches. |

### Multi-step routing

When a request involves multiple domains (e.g. "Create a Redmine ticket for the Discord bot bug"), the orchestrator should delegate to the **primary** specialist (Redmine in this case) and include the full context in the task description. The specialist handles tool invocation within its domain.

### Fallback

If no specialist matches, the orchestrator answers directly using its own model. For general questions, this avoids unnecessary delegation overhead.

---

## Open tasks
- ~~Clarify `model_role` resolution logic~~ **Done:** § "model_role resolution logic" above (when it runs, per-agent order, catalog-not-ready, role→pick rules, cloud models). Tracked in 006-feature-coder/FEATURE-CODER.md.
- ~~Add documentation for `AGENT: <selector> [task]` syntax~~ **Done:** § "AGENT: \<selector\> [task] syntax" above (invocation, selector resolution order, optional task, cursor-agent proxy, behaviour). Tracked in 006-feature-coder/FEATURE-CODER.md.
- ~~Implement missing orchestrator routing examples~~ **Done:** § "Orchestrator routing examples" above (routing table, multi-step, fallback). Tracked in 006-feature-coder/FEATURE-CODER.md.
- ~~Document `testing.md` format requirements~~ **Done:** § "testing.md format" above (file structure, parsing rules, conventions, timeout, examples). Tracked in 006-feature-coder/FEATURE-CODER.md.
- ~~Define fallback behavior for cloud model roles~~ **Done:** § "Cloud model as default — fallback behavior" above (scenario table, entry-point vs sub-agent, local-preference, warning log). Tracked in 006-feature-coder/FEATURE-CODER.md.
- ~~Add CLI command for agent reset/defaults~~ **Done:** § "Agent Reset" above; `mac_stats agent reset-defaults [id]` CLI subcommand; `Config::reset_agent_defaults()` force-overwrites bundled default files. Tracked in 006-feature-coder/FEATURE-CODER.md.