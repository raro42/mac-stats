# Morning surprise — 2026-09-07

Overnight Track B (20:00–06:00 local) shipped more operator instant-size lanes.

## Shipped

| Version | What |
|---------|------|
| **v0.1.904** | Instant lane: **testing.md size** (`testing size`, `testing.md size`, `how big is testing.md`, `testing file size`) — on-disk bytes across per-agent `testing.md` (stat only; no dump; does not steal path / run tests) |
| **v0.1.903** | Instant lane: **skill.md size** (`skill.md size`, `skill file size`, `how big is skill.md`) — on-disk bytes across per-agent `skill.md` (stat only; no dump; bare `skill size` stays Hermes skills/) |
| **v0.1.902** | Instant lane: **mood.md size** |
| **v0.1.901** | Instant lane: **soul.md size** |
| **v0.1.900** | Instant lane: **`.config.env` size** |
| **v0.1.899** | Instant lane: **credential_accounts.json size** |
| **v0.1.898** | Instant lane: **user-info.json size** |

## Context

- Digester **open** stayed empty (stale Elmasnow weather + Review logs already filtered).
- Design review still in **grace** (`feature-ai-chat` ~23d aged; TCC screenshot deferred).
- Fuel: standing backlog p50 latency — path lanes without matching size asks.

## Next fuel

- agent.json / planning_prompt.md / execution_prompt.md / escalation_patterns.md size
- Design review screenshot when TCC allows
- Digester open / debug.log product errors when present
