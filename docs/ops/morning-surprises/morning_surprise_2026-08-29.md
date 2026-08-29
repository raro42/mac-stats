# Morning surprise — 2026-08-29

Ralf, overnight Track B shipped product instant lanes (not digester-empty quiet).

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.708** | `/schedules` jobs · deliveries — Agent Ops Schedules list |
| **v0.1.709** | `/monitors` up · down · slow — External / Monitors list |
| **v0.1.710** | `/disk` on · off · reclaim · big · clean — Disk Cleanup scopes/categories |
| **v0.1.711** | `/logs` · `/logs error` · `/logs warn` — Debug Log Error/Warn tail (Discord + AI Chat) |

Also earlier same night: `/knowledge` (**v0.1.707**), `/sessions` (**v0.1.706**), and related Agent Ops operator parity.

## Tried / context

- Digester **open** stayed empty (fast instant or filtered turns).
- Design review **due=false** (grace); recommended surface still `feature-ai-chat`.
- `debug.log` scan: no ERROR/WARN/panic clusters in the last window.
- Fuel: standing backlog **p50** — UI filters without Discord/AI Chat instant lists.

## Why it helps

Asks like `any errors` or `/logs warn` skip Ollama and return the Debug Log tail immediately — same idea as `/monitors` / `/disk` / Agent Ops list operators.

## Next fuel

- Top Processes Hot/Pinned (pins need disk sync for Discord), or Perplexity Top/Snippet instant lists.
- Design review when grace ends (stale feature screens).
- Digester open / product-owned `debug.log` errors when they appear.
