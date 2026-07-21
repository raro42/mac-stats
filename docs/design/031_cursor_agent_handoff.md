# Cursor Agent / Claude Code Handoff

When local models (Ollama) don’t get the job done, mac-stats can hand off work to the installed **cursor-agent** CLI (or, conceptually, Claude Code) so the user still gets a result.

## Is this a new agent?

**No new directory agent.** The handoff uses:

1. **Existing CURSOR_AGENT tool** — The orchestrator can already reply with `CURSOR_AGENT: <prompt>`; the router runs the cursor-agent CLI and injects the result.
2. **Virtual “cursor-agent” in AGENT list** — When `cursor-agent` is on PATH, the router adds **cursor-agent** to the “Available agents” list. The model can then say `AGENT: cursor-agent <task>`; the router runs the CLI (no Ollama) and injects the result. So “cursor-agent” is a **proxy agent**: same process as CURSOR_AGENT but invocable via `AGENT:` like other agents.
3. **Automatic fallback** — If completion verification says the answer didn’t satisfy the request, the router runs cursor-agent with the original request and returns that result instead of only appending a “we may not have fully met your request” disclaimer. This applies to **any** task (coding or general), e.g. news (La Vanguardia, Barcelona), screenshot/browser tasks that failed (SSL, missing attachments), or coding tasks.

## How it works

### 1. Explicit handoff (CURSOR_AGENT or AGENT: cursor-agent)

- **CURSOR_AGENT: <prompt>** — Already implemented. Model outputs this; router runs `cursor-agent --print --trust --output-format text --workspace <dir> <prompt>` and injects stdout.
- **AGENT: cursor-agent [task]** — Implemented as a proxy: when the selector is `cursor-agent` (or `cursor_agent`), the router does not call Ollama; it runs the cursor-agent CLI with the task (or the full user question if no task text). The “cursor-agent” name is added to the agent list in the prompt when the CLI is available.

So the model can either use the tool explicitly (`CURSOR_AGENT: ...`) or delegate to the proxy agent (`AGENT: cursor-agent ...`).

### 2. Automatic fallback when verification fails

When:

- Completion verification returns **not satisfied**, and
- The router is about to append the “we may not have fully met your request” disclaimer, and
- **cursor-agent** is available on PATH,

the router runs **cursor-agent** with the original user request, replaces the reply with that result, and skips the generic disclaimer. So “local model didn’t get it done” is turned into “handed off to Cursor Agent; here’s what it did.” This applies to both coding tasks (implement, refactor, fix) and general tasks (e.g. “screenshot La Vanguardia”, “Barcelona latest news”, browser/SSL failures).

## Configuration

Unchanged from existing Cursor Agent setup:

- **PATH** — `cursor-agent` must be on PATH when mac-stats runs.
- **CURSOR_AGENT_WORKSPACE** — Env or `~/.mac-stats/.config.env`; default `$HOME/projects/mac-stats` if that dir exists.
- **CURSOR_AGENT_MODEL** — Optional; passed as `--model` to cursor-agent.

See `docs/012_cursor_agent_tasks.md` for install and usage.

## Summary

| Mechanism              | Invocation              | When to use                                      |
|-------------------------|-------------------------|--------------------------------------------------|
| CURSOR_AGENT tool       | `CURSOR_AGENT: <prompt>`| Model chooses to delegate a coding task.        |
| AGENT: cursor-agent     | `AGENT: cursor-agent <task>` | Model routes to “cursor-agent” like other agents. |
| Verification fallback   | Automatic                | Verification failed + cursor-agent CLI available (any task type). |

No new agent directory or new agent type is required; handoff is done via the existing tool, a proxy in the AGENT list, and one automatic fallback path in the router.
