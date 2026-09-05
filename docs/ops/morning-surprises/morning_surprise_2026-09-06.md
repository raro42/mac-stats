# Morning surprise — 2026-09-06

Overnight Track B (20:00–06:00 local, night of 2026-09-05 → morning 2026-09-06).

## Shipped

| Version | What |
|---------|------|
| **v0.1.877** | Instant lane: CDP traces directory size (`traces size`, `how big are traces`, `cdp traces size`) — recursive bytes under `~/.mac-stats/traces/` |
| **v0.1.878** | Instant lane: PDF exports directory size (`pdfs size`, `how big are pdfs`) — recursive bytes under `~/.mac-stats/pdfs/` |
| **v0.1.879** | Instant lane: browser-downloads directory size (`browser downloads size`, `how big are browser downloads`) — recursive bytes under `~/.mac-stats/browser-downloads/` |
| **v0.1.880** | Instant lane: agents directory size (`agents size`, `how big are agents`, `agents folder size`) — recursive bytes under `~/.mac-stats/agents/`; does not steal `agents path` / `/agents` |
| **v0.1.881** | Instant lane: skills directory size (`skills size`, `how big are skills`, `skills folder size`) — Hermes skills dir; does not steal `skills path` / `/skills` / skill.md |

## Fuel

- Digester open: empty (Elmasnow weather already stale/shipped).
- Design review: due=false (grace); stale PNGs still aged.
- Experiment this tick: standing backlog p50 — skills dir size after agents (**v0.1.880**).

## Ratchet

- keep @ `a6cdcb4` — v0.1.877 CDP traces dir size
- keep @ `b7c40e6` — v0.1.878 PDF exports dir size
- keep @ `6d80bce` — v0.1.879 browser-downloads dir size
- keep @ `b478923` — v0.1.880 agents dir size
- keep @ `3edb482` — v0.1.881 skills dir size

## Notes

- Nightly minimum satisfied (multiple keeps).
- Next fuel: plugins/prompts/session/task/quarantine dir sizes, or design-review screenshot when TCC allows.
