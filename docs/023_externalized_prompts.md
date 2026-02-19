# Externalized Prompts

System prompts sent to Ollama are no longer hardcoded in Rust source. They live as editable Markdown files under `~/.mac-stats/` and can be changed at runtime — no rebuild required.

## Files

| File | Purpose | Placeholder |
|------|---------|-------------|
| `~/.mac-stats/agents/soul.md` | Personality and tone rules. Prepended to all system prompts (router and agents). | None |
| `~/.mac-stats/prompts/planning_prompt.md` | Instructions for the planning step (how to produce `RECOMMEND: <plan>`). | None |
| `~/.mac-stats/prompts/execution_prompt.md` | Instructions for the execution step (how to invoke tools, relay results, answer concisely). | `{{AGENTS}}` |

### `{{AGENTS}}` placeholder

The execution prompt contains `{{AGENTS}}` which is replaced at runtime with the dynamically generated tool description list (RUN_JS, FETCH_URL, BRAVE_SEARCH, SCHEDULE, SKILL, RUN_CMD, TASK, OLLAMA_API, PYTHON_SCRIPT, DISCORD_API, AGENT, MCP). This list depends on which tools are enabled/configured at runtime, so it must remain code-generated. Everything else in the prompt is user-editable.

## Defaults

Default content is embedded in the binary via `include_str!` from source files in `src-tauri/defaults/`:

```
src-tauri/defaults/
  agents/
    soul.md
    agent-000/  (orchestrator)
      agent.json, skill.md, testing.md
    agent-001/  (general assistant)
      agent.json, skill.md, testing.md
    agent-002/  (coder)
      agent.json, skill.md, testing.md
    agent-003/  (generalist)
      agent.json, skill.md, testing.md
  prompts/
    planning_prompt.md
    execution_prompt.md
```

On first launch, `Config::ensure_defaults()` writes any missing files. Existing user files are **never overwritten**. To reset a file to its default, delete it and restart the app.

## How the system prompt is assembled

The final system prompt sent to Ollama (for both planning and execution steps) is assembled from these parts:

1. **Soul** — loaded from `~/.mac-stats/agents/soul.md`
2. **Discord user context** — injected when request is from Discord (user name, user ID)
3. **Prompt** — `planning_prompt.md` (planning step) or `execution_prompt.md` with `{{AGENTS}}` expanded (execution step)
4. **Plan** — the recommendation from the planning step (execution step only)

The code that assembles this is in `commands/ollama.rs` → `answer_with_ollama_and_fetch()`.

## Tauri commands (frontend API)

| Command | Arguments | Returns |
|---------|-----------|---------|
| `list_prompt_files` | none | `Vec<{name, path, content}>` for soul, planning_prompt, execution_prompt |
| `save_prompt_file` | `name: String, content: String` | `Ok(())` or error. Name must be `soul`, `planning_prompt`, or `execution_prompt`. |

## Editing tips

- Changes take effect on the **next request** (prompts are loaded fresh each time).
- Keep `{{AGENTS}}` in the execution prompt — removing it means Ollama won't know about available tools.
- The soul is shared with all agents (as fallback) and with the router. Per-agent souls in `agent-<id>/soul.md` override the shared soul for that agent.
- The planning prompt should instruct the model to reply with `RECOMMEND: <plan>` — the router strips this prefix and uses the rest as the plan.

## References

- **Config**: `src-tauri/src/config/mod.rs` — `load_planning_prompt()`, `load_execution_prompt()`, `ensure_defaults()`
- **Router**: `src-tauri/src/commands/ollama.rs` — prompt loading, `{{AGENTS}}` replacement, system prompt assembly
- **Defaults source**: `src-tauri/defaults/`
- **Agent defaults**: `docs/017_llm_agents.md` (default agents table)
- **All agents**: `docs/100_all_agents.md`
