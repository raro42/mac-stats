# Morning surprise — 2026-09-03

Overnight autoresearch (Track B) — ships for Ralf.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.838** | Instant lane: `scheduler_delivery_awareness.json` path (`delivery awareness path`, `where is scheduler_delivery_awareness.json`, `awareness file path`) — config only; no list; does not steal `last delivery` / `/schedules` |
| **v0.1.837** | Instant lane: `discord_channels.json` path (`discord channels path`, `where is discord_channels.json`, `channels.json`) — config only; no list/edit; does not steal `/discord` |
| **v0.1.836** | Instant lane: `perplexity_last.json` path (`perplexity last path`, `where is perplexity_last.json`, `last search file`) — config only; no Top/Snippet dump / new search; does not steal `/perplexity` |
| **v0.1.835** | Instant lane: `disk_cleanup.json` path (`disk cleanup path`, `where is disk_cleanup.json`, `cleanup file path`) — config only; no list/reclaim/clean; does not steal `/disk` |
| **v0.1.834** | Instant lane: `history.json` path (`history path`, `where is history.json`, `metrics history file`) — config only; no sparkline dump / chat history |
| **v0.1.833** | Instant lane: `monitors.json` path (`monitors path`, `where is monitors.json`, `monitor file path`) — config only; no list/add/check; does not steal `/monitors` |
| **v0.1.832** | Instant lane: `schedules.json` path (`schedules path`, `where is schedules.json`, `schedule file path`) — config only; no list/count/create; does not steal `/schedules` |
| **v0.1.831** | Instant lane: `pinned_processes.json` path — config only; does not steal `/pinned` |
| **v0.1.830** | Instant lane: cleanup-quarantine directory path — does not steal `/disk` |

## Digester / design review

- Digester **open**: empty (Elmasnow weather already stale/shipped as Open-Meteo).
- Design review: `due=false` (grace); PNGs still stale (`feature-ai-chat` ~20d) — recapture when grace ends.

## Next fuel

- `user-info.json` / alerts config path instant.
- Design-review screenshot + one polish when due.
- Product-owned `debug.log` errors when they appear.
