# Morning surprise — 2026-09-08

Overnight Track B kept shipping **instant file-age** lanes for operator asks (mtime only; no Ollama). Digester open stayed empty; design review was in grace.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.933** | Instant: `user-info.json` age (`user info age`, `how old is user info`, …) |
| **v0.1.932** | Instant: `scheduler_delivery_awareness.json` age |
| **v0.1.931** | Instant: `perplexity_last.json` age |
| **v0.1.930** | Instant: `discord_channels.json` age |
| **v0.1.929** | Instant: `pinned_processes.json` age |
| **v0.1.928** | Instant: `disk_cleanup.json` age |
| **v0.1.927** | Instant: `history.json` age |
| **v0.1.926** | Instant: `monitors.json` age |
| **v0.1.925** | Instant: `schedules.json` age |
| **v0.1.924** | Instant: `config.json` age |

## Also

- Fixed instant detectors so `tail` no longer matches inside `details` (unblocks `user details size` / `user details age`).
- Debug log: no ERROR/WARN/panic clusters in the scan window.
- Next fuel: `credential_accounts.json` age, `.config.env` age, or Hermes insights / session UX; design-review screens when TCC allows.

