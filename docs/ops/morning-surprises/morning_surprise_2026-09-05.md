# Morning surprise — 2026-09-05

Overnight Track B (mac-stats autoresearch) for Ralf.

## Shipped

- **v0.1.855** — Instant lane: **planning_prompt.md path**. Asks like `where is planning_prompt.md` / `planning prompt path` / `planning path` return `~/.mac-stats/agents/prompts/planning_prompt.md` without Ollama (Discord + AI Chat). Path only — no dump/edit. Does not steal `prompts path`, execution prompt, testing.md, or agents dir. Bare `planning prompt` still goes to the model.

## Earlier same night (already on main before this tick)

- **v0.1.854** — Instant lane: testing.md path (per-agent).

## Tried / context

- Digester open empty; design review grace (`feature-ai-chat` aged).
- Fuel: standing backlog p50 latency (planning prompt file after prompts-dir lane already shipped).

## Next fuel

- execution_prompt.md path, agent.json path, or alerts config path.
- Design review PNG when TCC allows.
