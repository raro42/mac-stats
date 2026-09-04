# Morning surprise — 2026-09-05

Overnight Track B (mac-stats autoresearch) for Ralf.

## Shipped

- **v0.1.857** — Instant lane: **agent.json path**. Asks like `where is agent.json` / `agent.json path` / `agent config path` return `~/.mac-stats/agents/agent-<id>/agent.json` without Ollama (Discord + AI Chat). Path only — no dump/edit. Does not steal `agents path`, app `config path`, soul, skill, or testing path.

- **v0.1.856** — Instant lane: **execution_prompt.md path**. Asks like `where is execution_prompt.md` / `execution prompt path` / `execution path` return `~/.mac-stats/agents/prompts/execution_prompt.md` without Ollama (Discord + AI Chat). Path only — no dump/edit. Does not steal `prompts path`, planning prompt, testing.md, or agents dir. Bare `execution prompt` still goes to the model.

- **v0.1.855** — Instant lane: **planning_prompt.md path**. Asks like `where is planning_prompt.md` / `planning prompt path` / `planning path` return `~/.mac-stats/agents/prompts/planning_prompt.md` without Ollama (Discord + AI Chat). Path only — no dump/edit. Does not steal `prompts path`, execution prompt, testing.md, or agents dir. Bare `planning prompt` still goes to the model.

## Earlier same night (already on main before these ticks)

- **v0.1.854** — Instant lane: testing.md path (per-agent).

## Tried / context

- Digester open empty; design review grace (`feature-ai-chat` aged).
- Fuel: standing backlog p50 latency (agent.json after execution_prompt lane).

## Next fuel

- Alerts config path, or other remaining config files.
- Design review PNG when TCC allows.
