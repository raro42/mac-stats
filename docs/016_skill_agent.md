# SKILL Agent (Specialized-Skill Tool)

The SKILL agent lets Ollama run a **specialized skill** (a Markdown system-prompt overlay) in a **separate Ollama session** with no main conversation history. The skill’s reply is injected back into the main conversation so the model can use it to answer the user.

## Overview

- **Agent name**: SKILL  
- **Invocation**: Ollama replies with one line: `SKILL: <number or topic> [optional task]`  
  - Examples: `SKILL: 2`, `SKILL: summarize`, `SKILL: code fix the indentation`, `SKILL: 1 Summarize the following: ...`  
- **Selector**: Either a **number** (e.g. `2`) or a **topic** slug (e.g. `summarize`, `code`). Topic matching is case-insensitive; spaces in topic filenames can be written as `-`.  
- **Task**: Optional. If present (text after the first space), it is the user message for the skill session. If omitted, the **current user question** is used as the user message.  
- **Execution**: The app loads the skill content from `~/.mac-stats/skills/`, runs one Ollama request (system = skill content, user = task or question), and injects the result as `Skill "<number>-<topic>" result:\n\n<result>` into the main conversation.

When there are **no skills** in `~/.mac-stats/skills/`, the SKILL agent is **not** added to the list Ollama sees, so the model cannot invoke it. For per-agent **model** and **soul/mood/skill** (orchestrator and specialists), use **LLM agents** in `~/.mac-stats/agents/` instead; see `docs/017_llm_agents.md`.

## Skill files

- **Directory**: `~/.mac-stats/skills/` (see `Config::skills_dir()` in `config/mod.rs`).  
- **Naming**: `skill-<number>-<topic>.md`, e.g. `skill-1-summarize.md`, `skill-2-code.md`.  
- **Default skills**: When the skills directory is empty, the app creates two default skills: **1-summarize** (summarization) and **2-code** (code help). You can edit or remove them.
- **Content**: Markdown (or plain text) used as the **system prompt** for the skill session. The file is read and trimmed; empty files are skipped.  
- **Listing**: Skills are loaded at runtime; the app builds the “Available skills: 1-summarize, 2-code, …” list from filenames and passes it to Ollama so it can recommend and invoke by number or topic.

See **Discord/skill override** in `docs/012_ollama_context_skills.md`: a Discord message can also start with `skill: 2` or `skill: code` to **prepend** that skill’s content to the **main** system prompt for the whole request. That is different from the **SKILL tool**, which runs a separate session and injects the result.

**Logging:** Each time skills are loaded, the app writes to the log file (`~/.mac-stats/debug.log` when configured): at **info** level (visible with `-vv` or `-vvv`) the directory path, count, and list (e.g. `Skills: loaded 2 from …: 1-summarize, 2-code`), or that the directory is missing/empty; at **warn** (visible with `-v` or higher) directory or file read errors. Any future code that creates or modifies skill files should also log and consider notifying the user (e.g. status or Tauri event).

## When to use SKILL (for Ollama / planning)

- Prefer **SKILL** when the user asks for a **single focused outcome** that matches an available skill (e.g. “summarize this”, “make a joke”, “what’s the time”, “format this code”).  
- Each skill runs in a **separate session** (no main chat history), so it’s best for one-off, well-scoped tasks.  
- The result is always injected back into the main conversation so the model can cite it or refine the answer.

## Behaviour

- When at least one skill file exists, the agent list sent to Ollama in the planning and execution steps includes **SKILL** with the list of available skills (e.g. “Available skills: 1-summarize, 2-code”).  
- Ollama can reply with e.g. `SKILL: 2` or `SKILL: summarize Short summary of …`. The app finds the skill by number or topic, runs `run_skill_ollama_session(skill_content, user_message, ...)`, and injects the result (or an error message) into the conversation.  
- The tool loop (Discord, scheduler, and when wired CPU-window flow) supports SKILL like other tools; each SKILL call counts as one tool iteration (max 5).

## Rust API (for context)

- **Module**: `src-tauri/src/skills.rs`  
- **Types**: `Skill { number: u32, topic: String, content: String }`  
- **Functions**:  
  - `load_skills() -> Vec<Skill>` — reads all `skill-<number>-<topic>.md` from `~/.mac-stats/skills/`.  
  - `find_skill_by_number_or_topic(skills, selector) -> Option<&Skill>` — match by number or by topic (case-insensitive, `-`/space normalized).  
- **Execution**: `commands/ollama.rs` → `run_skill_ollama_session(skill_content, user_message, model_override, options_override)` — one system message (skill content), one user message; returns the assistant reply or error string.

## Where it’s used

- **Discord bot**: When skills exist, Ollama can output `SKILL: <number or topic> [task]`. The app runs the skill session and gives the result back to Ollama.  
- **Scheduler**: Same pipeline; scheduled tasks that go through Ollama can use SKILL.  
- **CPU window chat**: When the CPU-window flow uses the same tool loop, SKILL is available there too.

## References

- **Code:** `src-tauri/src/skills.rs`, `src-tauri/src/commands/ollama.rs` (tool loop, `build_skill_agent_description`, `run_skill_ollama_session`)
- **All agents:** `docs/100_all_agents.md`
- **Skills dir and Discord skill override:** `docs/012_ollama_context_skills.md`
