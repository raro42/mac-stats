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

Also earlier same night: `/knowledge` (**v0.1.707**), `/sessions` (**v0.1.706**), and related Agent Ops operator parity.

## Tried / context

- Digester **open** stayed empty (fast instant or filtered turns).
- Design review **due=false** (grace); recommended surface still `feature-ai-chat`.
- `debug.log` scan: no ERROR/WARN/panic clusters in the last window.
- Fuel: standing backlog **p50** — UI filters without Discord/AI Chat instant lists.
- Pinned processes stay UI-only (localStorage); Discord gets All · Hot only.
- Last Perplexity results now persist from the CPU window search and from `PERPLEXITY_SEARCH` tool runs.

## Why it helps

Asks like `last search`, `/perplexity top`, or `snippet results` skip Ollama and return the last Perplexity hits immediately — same idea as `/processes` / `/logs` / `/monitors` / `/disk` / Agent Ops list operators.

## Next fuel

- Optional: pin sync to `~/.mac-stats` if Discord Pinned parity is worth it.
- Design review when grace ends (stale feature screens).
- Digester open / product-owned `debug.log` errors when present.
- Other tool-heavy p50 patterns when digester surfaces them.
