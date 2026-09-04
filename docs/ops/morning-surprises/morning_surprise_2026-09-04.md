# Morning surprise — 2026-09-04

Overnight Track B kept shipping instant-lane path wins (digester open empty; design review in grace).

## Shipped tonight (keep)

| Version | What |
|---------|------|
| **v0.1.862** | Instant lane: Discord channel memory path (`discord memory path` / `memory-discord path`) — path only; no steal of `/knowledge discord` |
| **v0.1.861** | Instant lane: before-compaction transcript path |
| **v0.1.860** | Instant lane: before-reset transcript path |
| **v0.1.859** | Instant lane: Ori vault path |

## Also tried / context

- Digester Slowest still shows stale Elmasnow Brave weather (already filtered as shipped).
- Design review PNGs aged (~21–23d); TCC capture deferred; polish-without-capture grace active.
- Debug log: no ERROR/WARN clusters in 180m window.

## Fitness

Operators can ask where Discord channel memory files live without a 10s+ Ollama round-trip, and without landing on the notes-folder path.
