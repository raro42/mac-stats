# Morning surprise — 2026-09-21

Overnight Track B (20:00–06:00 window starting 2026-09-20).

## Shipped

| Version | What got better |
|---------|-----------------|
| **v0.1.1197** | Session count answers only inventory asks (“how many sessions”, …). Verb phrases like “edit this session” stay with the agent. |
| **v0.1.1198** | Operator inventory counts (agents, monitors, tasks, skills, plugins, knowledge) answer only inventory asks. “Delete this agent”, “run this skill”, “check this monitor” stay with the agent. |
| **v0.1.1199** | Runs inventory counts answer only inventory asks (“how many runs”, “run count”, …). “Delete these runs”, “clear the runs”, “export runs” stay with the agent. |

## Tried / context

- Digester open stayed empty (no Slowest / open candidates).
- Design review: `feature-agent-ops` in grace (~5d); deferred for inventory-count correctness.
- Debug.log: Having-fun idle Ollama timeout WARN — already soft-pathed; not product-owned ERROR fuel.

## Fitness

Users who say “delete these runs” or “clear the runs” no longer get a bare runs count. Count asks still stay instant.
