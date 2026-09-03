# Morning surprise — 2026-09-03

Overnight Track B kept shipping path/instant + uptime fixes.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.830** | Instant lane: **cleanup-quarantine directory path** (`cleanup quarantine path` / `where is cleanup-quarantine` / `quarantine folder`) → `~/.mac-stats/cleanup-quarantine/` without Ollama (does not steal `/disk`) |
| **v0.1.829** | Instant lane: **browser-downloads directory path** (`browser downloads path` / `where are browser downloads` / `browser-downloads`) → `~/.mac-stats/browser-downloads/` without Ollama (does not steal `/downloads` organizer) |
| **v0.1.828** | Instant lane: **browser storage-state / cookies path** (`storage state path` / `where are browser cookies` / `browser_storage_state.json`) → `~/.mac-stats/browser_storage_state.json` without Ollama |
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
- Design review still in grace; `feature-ai-chat` PNG ~19.6d stale.
- Next: more p50 path lanes (e.g. history.json), or design-review screenshot when grace ends.
