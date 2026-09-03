# Morning surprise — 2026-09-03

Overnight Track B kept shipping path instant lanes so operator “where is …?” asks stay off Ollama.

## Shipped tonight (keep)

| Version | What |
|---------|------|
| **v0.1.839** | Instant lane: `user-info.json` path (`user info path`, `where is user-info.json`, `user-info path`) — config only; no list/edit |
| **v0.1.838** | Instant lane: `scheduler_delivery_awareness.json` path — does not steal `last delivery` / `/schedules` |
| **v0.1.837** | Instant lane: `discord_channels.json` path — does not steal `/discord` |
| **v0.1.836** | Instant lane: `perplexity_last.json` path — does not steal `/perplexity` |
| **v0.1.835** | Instant lane: `disk_cleanup.json` path — does not steal `/disk` |
| **v0.1.834** | Instant lane: `history.json` path — metrics sparkline buffer file |
| **v0.1.833** | Instant lane: `monitors.json` path — does not steal `/monitors` |
| **v0.1.832** | Instant lane: `schedules.json` path — does not steal `/schedules` |

## Digester / design review

- Digester **open**: empty (Elmasnow weather already stale after **v0.1.818**).
- Design review: **due=false** (grace); recommended surface still `feature-ai-chat` (~20d PNG).

## Note for next night

- Path-detector sibling nesting is getting expensive — prefer string-only excludes for new lanes (as in **v0.1.839**).
- Next fuel candidates: alerts config path, design-review PNG recapture when grace ends, debug.log product errors.
