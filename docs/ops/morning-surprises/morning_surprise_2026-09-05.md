# Morning surprise — 2026-09-05

Overnight Track B (mac-stats product ratchet). Digester open stayed empty; fuel was standing-backlog p50 dir-size instant lanes.

## Shipped

| Version | What |
|---------|------|
| **v0.1.885** | Instant lane: task directory size (`task size`, `how big are tasks`, `task folder size`, …) — recursive file bytes under `~/.mac-stats/task/`; no Ollama; does not steal `task path` / `/tasks` / `TASK_CREATE:` |
| **v0.1.884** | Instant lane: session directory size (`session size`, `how big are sessions`, …) — under `~/.mac-stats/session/`; does not steal path / `/sessions` / session-memory |
| **v0.1.883** | Instant lane: prompts directory size — under `~/.mac-stats/agents/prompts/` |
| **v0.1.882** | Instant lane: plugins/scripts directory size — under `~/.mac-stats/scripts/` |
| **v0.1.881** | Instant lane: skills directory size — Hermes skills folder |
| **v0.1.880** | Instant lane: agents directory size — under `~/.mac-stats/agents/` |
| **v0.1.879** | Instant lane: browser-downloads directory size |
| **v0.1.878** | Instant lane: PDF exports directory size |
| **v0.1.877** | Instant lane: CDP traces directory size |

## Night notes

- Digester open empty (Elmasnow weather already stale/shipped).
- Design review due=false (grace); `feature-ai-chat` / Agent Ops PNGs still aged (~22–24d).
- Latest keep: **v0.1.885** @ `da2a651` (install + Discord Ready).
- Next fuel: cleanup-quarantine / notes dir size, or design-review polish when TCC allows.

Ask Werner: *Any improvements from last night?*
