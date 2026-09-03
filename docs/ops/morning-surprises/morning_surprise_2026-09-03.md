# Morning surprise — 2026-09-03

Overnight Track B kept shipping path/instant + uptime fixes.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.827** | Instant lane: **browser credentials path** (`browser credentials path` / `where are browser credentials` / `browser-credentials.toml`) → `~/.mac-stats/browser-credentials.toml` without Ollama |
| **v0.1.826** | Instant lane: **PDF exports directory path** (`pdfs path` / `where is the pdfs folder` / `pdf directory`) → `~/.mac-stats/pdfs/` without Ollama |
| **v0.1.825** | Instant lane: **CDP traces directory path** (`traces path` / `where is the traces folder` / `cdp traces`) → `~/.mac-stats/traces/` without Ollama |
| **v0.1.824** | Instant lane: **uploads directory path** (`uploads path` / `where is the uploads folder`) → `~/.mac-stats/uploads/` without Ollama |
| **v0.1.823** | Startup: skip macOS crash-restore modal (`ApplePersistenceIgnoreState`) so Discord starts after LaunchAgent thrash |
| **v0.1.822** | Instant lane: tmp directory path |
| **v0.1.821** | Instant lane: prompts directory path |
| **v0.1.820** | Instant lane: plugins/scripts directory path |

## Fuel notes

- Digester open empty (Elmasnow weather already stale/shipped).
- Design review still in grace; `feature-ai-chat` PNG ~19.5d stale.
- Next: browser storage-state / cookies path, browser-downloads dir, or design-review screenshot when grace ends.
