# Morning surprise — 2026-09-08

Overnight Track B (20:00–06:00 local, starting 2026-09-07).

## Shipped

| Version | What |
|---------|------|
| **v0.1.928** | Instant lane: `disk_cleanup.json` age (`disk cleanup age` / how old / when updated) — mtime only, no Ollama |
| **v0.1.927** | Instant lane: `history.json` age |
| **v0.1.926** | Instant lane: `monitors.json` age |
| **v0.1.925** | Instant lane: `schedules.json` age |
| **v0.1.924** | Instant lane: `config.json` age |
| **v0.1.923** | Instant lane: Ori vault size |
| **v0.1.922** | Instant lane: before-compaction transcript size |
| **v0.1.921** | Instant lane: before-reset transcript size |
| **v0.1.920** | Instant lane: session-memory size |
| **v0.1.919** | Agent Ops Overview Live idle calm (design review) |

## Tried / context

- Digester **open** stayed empty (stale Elmasnow weather; `Review logs` already instant).
- Design review **due=false** (grace); CPU metrics PNG ~0.85d.
- Fuel stayed on standing-backlog **p50 file age** lanes after path/size coverage.
- ~23:22 tick: disk_cleanup.json age keep @ eafa2e7e; install/kickstart 0.1.928.

## Still open

- pinned_processes.json / discord_channels.json age (and other JSON age siblings)
- Design-review screenshot refresh when TCC allows (agent-ops / ai-chat / processes)
- Sibling Hermes insights / session UX ports
