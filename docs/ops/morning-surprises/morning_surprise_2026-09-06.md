# Morning surprise — 2026-09-06

Overnight autoresearch (Track B) for Ralf.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.888** | Instant lane: `review logs` / `check logs` / `look at logs` / `read logs` → existing `/logs` Debug Log tail (no Ollama / Brave; digester Slowest had a ~40s Brave waste on bare `Review logs`). |
| **v0.1.887** | Instant lane: notes / memory folder size (`notes size`, `how big are notes`, `memory folder size`) — recursive bytes under `~/.mac-stats/agents/notes/`; rejects bare `memory size` (RAM). |
| **v0.1.886** | Instant lane: cleanup-quarantine directory size. |
| **v0.1.885** | Instant lane: task directory size. |
| **v0.1.884** | Instant lane: session directory size. |

## Context

- Digester open stayed empty (weather candidate already stale/shipped).
- Slowest fuel: bare `Review logs` (lite + BRAVE_SEARCH ~40s) missed `/logs`.
- Design review in grace (`feature-ai-chat` recommended when TCC allows).
- Fuel: digester Slowest → standing backlog p50 latency.

## Try it

```text
Review logs
check logs
look at the logs
read logs
/logs error
```

Also still: `notes size` / `how big are notes`. Path lane: `memory path` / `notes path`. Bare `memory size` is not the notes-size lane (RAM).
