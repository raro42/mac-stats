# Morning surprise — 2026-09-03

Overnight Track B shipped path instant lanes **and** fixed a Discord-down startup hang.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.823** | **Startup: skip macOS crash-restore modal** — `ApplePersistenceIgnoreState` early so LaunchAgent / overnight restarts do not block on `NSPersistentUIRestorer` (Discord was never starting) |
| **v0.1.822** | Instant lane: **tmp directory path** — `tmp path` / `where is the tmp folder` / `temp directory` → `~/.mac-stats/tmp/` (+ JS scratch) without Ollama |
| **v0.1.821** | Instant lane: **prompts directory path** — `prompts path` / `where is the prompts folder` → `~/.mac-stats/agents/prompts/` |
| **v0.1.820** | Instant lane: **plugins/scripts directory path** — `plugins path` / `scripts path` → `~/.mac-stats/scripts/` |

## Context

- Digester: no open candidates (Elmasnow weather already filtered as shipped STT fix).
- Design review: `due=false` (grace); stale PNGs still recommended (`feature-ai-chat`, Agent Ops, …).
- Found during install verify: process stuck in `NSAlert runModal` (crash-restore). Cleared saved state + shipped ignore flag. Discord Ready again after **v0.1.823**.

## Next

- More data-home path lanes (uploads / traces / pdfs) or design-review polish when grace ends.
- Sibling: Hermes `/insights` port only if it maps cleanly to existing Runs insights.
