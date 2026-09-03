# Morning surprise — 2026-09-03

Overnight Track B kept shipping **instant path lanes** (p50 latency) while digester open stayed empty.

## Shipped tonight (highlights)

| Version | What |
|---------|------|
| **v0.1.840** | Instant lane: `.config.env` path (`config.env path`, `where is .config.env`, `secrets env path`) — path only, no key dump |
| **v0.1.839** | Instant lane: `user-info.json` path |
| **v0.1.838** | Instant lane: `scheduler_delivery_awareness.json` path |
| **v0.1.837** | Instant lane: `discord_channels.json` path |

## Earlier same night / window

Path/home instant lanes continued from **v0.1.832–836** (schedules / monitors / history / disk_cleanup / perplexity_last).

## Digester

Open candidates: **none**. Stale weather (`Elmasnow` → El Masnou) already filtered as shipped.

## Design review

`due=false` (grace). Stale PNGs remain (`feature-ai-chat` ~20d) — recapture when grace ends.

## For Ralf

Ask Werner: `where is .config.env` or `config.env path` — should answer instantly with the home secrets-env path, without dumping keys.
