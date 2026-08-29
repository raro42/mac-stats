# Morning surprise — 2026-08-29

Ralf, overnight Track B shipped product instant lanes (not digester-empty quiet).

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.709** | `/monitors` up · down · slow — External / Monitors list (Discord + AI Chat) |
| **v0.1.710** | `/disk` on · off · reclaim · big · clean — Disk Cleanup scopes/categories (Discord + AI Chat; shallow scan) |

Also earlier same night (before this note): `/schedules` jobs/deliveries (**v0.1.708**), `/knowledge` (**v0.1.707**), `/sessions` (**v0.1.706**), and related Agent Ops operator parity.

## Tried / context

- Digester **open** stayed empty (9 turns, all fast instant or filtered).
- Design review **due=false** (grace); recommended surface still `feature-ai-chat`.
- `debug.log` scan: no ERROR/WARN/panic clusters in the last window.
- Fuel: standing backlog **p50** — UI filters without Discord/AI Chat instant lists.

## Why it helps

Asks like `what's reclaimable` or `/disk big` skip Ollama and return Disk Cleanup status immediately — same idea as `/monitors` / Agent Ops list operators.

## Next fuel

- Top Processes Hot/Pinned, Debug Log Error/Warn, or Perplexity Top/Snippet instant lists.
- Design review when grace ends (stale feature screens).
- Digester open / product-owned `debug.log` errors when they appear.
