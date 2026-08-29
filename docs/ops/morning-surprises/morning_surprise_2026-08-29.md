# Morning surprise — 2026-08-29

Ralf, overnight Track B shipped product instant lanes (not digester-empty quiet).

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.708** | `/schedules` jobs · deliveries — Agent Ops Schedules list |
| **v0.1.709** | `/monitors` up · down · slow — External / Monitors list |
| **v0.1.710** | `/disk` on · off · reclaim · big · clean — Disk Cleanup scopes/categories |
| **v0.1.711** | `/logs` · `/logs error` · `/logs warn` — Debug Log Error/Warn tail |
| **v0.1.712** | `/processes` · `/processes hot` · `/hot` — Top Processes Hot list (CPU≥15% · GPU≥15% · RAM≥1 GiB) |
| **v0.1.713** | `/perplexity` · `/perplexity top` · `/perplexity snippet` — last Perplexity Top/Snippet list (`perplexity_last.json`) |
| **v0.1.714** | `/processes pinned` · `/pinned` — Top Processes Pinned list (`pinned_processes.json` sync from CPU window) |

Also earlier same night: `/knowledge` (**v0.1.707**), `/sessions` (**v0.1.706**), and related Agent Ops operator parity.

## Tried / context

- Digester **open** stayed empty (fast instant or filtered turns).
- Design review **due=false** (grace); recommended surface still `feature-ai-chat`.
- `debug.log` scan: no ERROR/WARN/panic clusters in the last window.
- Fuel: standing backlog **p50** — UI filters without Discord/AI Chat instant lists; then pin sync.
- Pins now live in `~/.mac-stats/pinned_processes.json` (localStorage still mirrors in the WebView).
- Last Perplexity results persist from the CPU window search and from `PERPLEXITY_SEARCH` tool runs.

## Why it helps

Asks like `/processes pinned`, `show pinned`, or `/pinned` skip Ollama and return starred favorites immediately — same idea as Hot / Monitors / Disk / Agent Ops list operators. Discord sees the same pins as the CPU window.

## Next fuel

- Design review when grace ends (stale feature screens).
- Digester open / product-owned `debug.log` errors when present.
- Other tool-heavy p50 patterns when digester surfaces them (e.g. CPU rings Hot instant).
