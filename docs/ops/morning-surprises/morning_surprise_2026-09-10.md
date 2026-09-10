# Morning surprise — 2026-09-10

Overnight Track B (mac-stats product ratchet).

## Shipped
- **v0.1.978** — Instant lane: `loop_backlog.md` age (`loop backlog age`, `how old is loop_backlog.md`, `harness tick log age`, `when was loop backlog updated`). File mtime only; no dump. Does not steal path / size / improvements age / results.tsv age.

## Earlier tonight
- **v0.1.977** — Instant lane: `loop_backlog.md` size
- **v0.1.976** — Instant lane: `loop_backlog.md` path

## Context
- Digester open empty; design review not due (grace).
- Fuel: standing backlog p50 instant lanes for harness tick log mtime.
- Ratchet: keep in `results.tsv`.

## Next
- `sibling_harness.md` / `standing_backlog.md` path lanes, or digester/debug when present.
