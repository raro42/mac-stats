# LLM Agents (directory-based, per-agent model and prompt)

This document describes **LLM agents**: directory-based entities under `~/.mac-stats/agents/` with their own **model**, **skill**, optional **mood** and **soul**, and optional **orchestrator** role. They are distinct from the **SKILL** tool (simple prompt overlays in `~/.mac-stats/skills/`). Use LLM agents when you need a per-agent model and a combined identity (soul + mood + skill).

## Path and layout

- **Path**: `~/.mac-stats/agents/` (see `Config::agents_dir()` in `config/mod.rs`).
- **Per agent**: One subdirectory per agent: `agent-<id>/` (e.g. `agent-001`, `agent-002`).

| File         | Purpose                                                                 |
| ------------ | ----------------------------------------------------------------------- |
| **agent.json** | Required. `{ "name": "...", "slug": "...", "model": "...", "orchestrator": true/false, "enabled": true/false }`. `model` optional (fallback: default Ollama config). |
| **skill.md**   | Required. What the agent is good at (system-prompt overlay).            |
| **mood.md**   | Optional. Mood / tone / additional context.                            |
| **soul.md**   | Optional. “Soul” of the agent (identity, principles).                   |
| **testing.md**| Required for `mac_stats agent test`. Test prompts (one per `## ` section or whole file). |

**Combined prompt** for an agent: concatenate in order **soul → mood → skill**. Empty files are skipped.

**Naming**: Agent **id** comes from the directory (`agent-001` → id `"001"`). **Name** and optional **slug** come from `agent.json`. **Slug** is a short natural-language identifier (e.g. `humble-coder`, `senior-coder`) used for display and for **AGENT: <selector>** resolution: match by **slug** (case-insensitive) first, then by **name**, then by **id**.

## AGENT tool (Ollama tool loop)

When at least one enabled agent exists, the app adds an **AGENT** tool to the list Ollama sees:

- **Invocation**: `AGENT: <id or name or slug> [optional task]`
- **Behaviour**: The app runs that agent in a **separate Ollama session**: system prompt = agent’s combined prompt (soul + mood + skill), user message = task or current question, model = agent’s `model` or default. The result is injected back so the main model (e.g. orchestrator) can use it.

**Orchestrator**: One or more agents can have `"orchestrator": true` in `agent.json`. Their `skill.md` typically instructs them to delegate via **AGENT: <id or name> <task>** when a specialized agent is needed. When the orchestrator outputs `AGENT: 002 write a hello world`, the app runs agent `002` and returns its reply to the orchestrator. The orchestrator (e.g. agent-000) should include a **Router API Commands** section in its `skill.md` so it can ask and use all router tools (AGENT, FETCH_URL, TASK_*, SCHEDULE, RUN_CMD, OLLAMA_API, etc.); see `docs/agent_000_router_commands_snippet.md` for a copy-paste snippet.

## Entry-point agent override (Discord)

- **Discord**: A message can start with `agent: 001` or `agent: General` (same style as `model:`, `skill:`). The app resolves the selector to an agent and passes it as **agent_override** into `answer_with_ollama_and_fetch`. The **first** response is then made by that agent (its model + combined prompt); the tool list still includes **AGENT:** so the orchestrator (or that agent) can call other agents.

## Agent test CLI

- **Command**: `mac_stats agent test <selector> [path]`
- **Behaviour**: Resolves the agent by id/slug/name, reads test prompts from `path` or from `~/.mac-stats/agents/agent-<id>/testing.md`, and runs each prompt through that agent’s session (same as AGENT tool). Each agent **must** have a `testing.md` when using the default path.

## Rust API (for context)

- **Module**: `src-tauri/src/agents/mod.rs` (and `agents/cli.rs`, `agents/watch.rs`).
- **Types**: `Agent { id, name, slug, model, orchestrator, enabled, combined_prompt }`, `AgentConfig` (from agent.json).
- **Functions**: `load_agents() -> Vec<Agent>`, `find_agent_by_id_or_name(agents, selector) -> Option<&Agent>`, `load_all_agents()`, `get_agent_dir(id)`.
- **Execution**: `commands/ollama.rs` → `run_agent_ollama_session(agent, user_message, status_tx)`; tool loop handles `AGENT:` and calls it.

## Default agents

Four agents ship as defaults, embedded in the binary via `include_str!` from `src-tauri/defaults/agents/`. On first launch (or if missing), `Config::ensure_defaults()` writes them to `~/.mac-stats/agents/`:

| Dir | Name | Slug | Model | Role |
|-----|------|------|-------|------|
| `agent-000` | Orchestrator | `orchestrator` | qwen3:latest | Routes to specialists, has full Router API Commands in skill.md |
| `agent-001` | General Assistant | `general-purpose-mommy` | qwen3:latest | General-purpose Q&A |
| `agent-002` | Coder | `senior-coder` | qwen2.5-coder:latest | Code generation, refactoring, debugging |
| `agent-003` | Generalist | `humble-generalist` | huihui_ai/granite3.2-abliterated:2b | Non-code topics, discussion, reflection |

Each includes `agent.json`, `skill.md`, and `testing.md`. Users can edit any file — `ensure_defaults()` never overwrites existing files. To reset an agent to defaults, delete its directory and restart.

The default source files live in `src-tauri/defaults/agents/` and are easy to edit in the repo (clean Markdown diffs).

## Relationship to SKILL

- **SKILL** (see `docs/016_skill_agent.md`): Simple prompt overlays in `~/.mac-stats/skills/` (e.g. `skill-1-summarize.md`). No per-skill model; one default model, skill content as system prompt. Use for lightweight, single-session tasks.
- **LLM agents**: Richer entities with optional per-agent model, soul, mood, and skill. Use when you need different models or a clear identity (orchestrator vs specialists) and delegation via **AGENT:**.

## References

- **Plan**: LLM agent orchestration with skill/mood/soul and per-agent models.
- **All agents overview**: `docs/100_all_agents.md`
- **Discord**: `docs/007_discord_agent.md` (agent override: `agent: 001`)
