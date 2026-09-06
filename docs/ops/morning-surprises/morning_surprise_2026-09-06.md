# Morning surprise — 2026-09-06

Overnight autoresearch (Track B) for Ralf.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.890** | Instant lane: `schedules size` / `schedules.json size` / `how big is schedules` — on-disk size of `schedules.json` (stat only; no dump; does not steal `schedules path` / `/schedules`). |
| **v0.1.889** | Instant lane: `config size` / `config.json size` / `how big is config` — on-disk size of app `config.json` (stat only; no dump; does not steal `config path` / `.config.env`). |
| **v0.1.888** | Instant lane: `review logs` / `check logs` / `look at logs` / `read logs` → existing `/logs` Debug Log tail (no Ollama / Brave; digester Slowest had a ~40s Brave waste on bare `Review logs`). |
| **v0.1.887** | Instant lane: notes / memory folder size (`notes size`, `how big are notes`, `memory folder size`) — recursive bytes under `~/.mac-stats/agents/notes/`; rejects bare `memory size` (RAM). |
| **v0.1.886** | Instant lane: cleanup-quarantine directory size. |
| **v0.1.885** | Instant lane: task directory size. |
| **v0.1.884** | Instant lane: session directory size. |

## Context

- Digester open stayed empty (weather candidate already stale/shipped).
- Design review in grace (`feature-ai-chat` recommended when TCC allows).
- Fuel: standing backlog p50 latency (config.json size after path lane).

## Try it

```text
config size
config.json size
how big is config
config path
```

Also still: `Review logs` / `notes size`. Bare `memory size` is not the notes-size lane (RAM).
