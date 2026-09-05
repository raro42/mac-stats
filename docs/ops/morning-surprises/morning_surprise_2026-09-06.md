# Morning surprise — 2026-09-06

Overnight autoresearch (Track B) for Ralf.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.887** | Instant lane: notes / memory folder size (`notes size`, `how big are notes`, `memory folder size`) — recursive bytes under `~/.mac-stats/agents/notes/`; rejects bare `memory size` (RAM). |
| **v0.1.886** | Instant lane: cleanup-quarantine directory size. |
| **v0.1.885** | Instant lane: task directory size. |
| **v0.1.884** | Instant lane: session directory size. |

## Context

- Digester open stayed empty (weather candidate already stale/shipped).
- Design review in grace (`feature-ai-chat` recommended when TCC allows).
- Fuel: standing backlog p50 latency → directory-size instant lanes.

## Try it

```text
notes size
how big are notes
memory folder size
notes folder size
```

Path lane still: `memory path` / `notes path`. Bare `memory size` is not this lane (RAM).
