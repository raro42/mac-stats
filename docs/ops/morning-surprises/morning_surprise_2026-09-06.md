# Morning surprise — 2026-09-06

Overnight Track B (autoresearch) kept shipping instant-lane file-size reads so operators skip Ollama for “how big is …” config stats.

## Shipped tonight

| Version | What |
| --- | --- |
| **v0.1.897** | Instant: `scheduler_delivery_awareness.json` size (`delivery awareness size` / `how big is delivery awareness` / `awareness file size`) |
| **v0.1.896** | Instant: `perplexity_last.json` size (`perplexity last size` / `how big is perplexity last` / `last search file size`) |
| **v0.1.895** | Instant: `discord_channels.json` size (`discord channels size` / `how big is discord channels` / `channels.json size`) |
| **v0.1.894** | Instant: `pinned_processes.json` size (`pinned processes size` / `how big is pinned processes` / `pin file size`) |
| **v0.1.893** | Instant: `disk_cleanup.json` size (`disk cleanup size` / `how big is disk cleanup` / `cleanup file size`) |
| **v0.1.892** | Instant: `history.json` size (`history size` / `how big is history` / `metrics history size`) |
| **v0.1.891** | Instant: `monitors.json` size |
| **v0.1.890** | Instant: `schedules.json` size |
| **v0.1.889** | Instant: `config.json` size |
| **v0.1.888** | Instant: `review logs` / `check logs` → `/logs` (cuts digester Slowest Brave waste) |

## Context

- Digester **open** stayed empty (Elmasnow weather + Review logs already stale/shipped).
- Design review **due=false** (grace); AI Chat / Agent Ops feature PNGs still aged — screenshot polish deferred until TCC allows.
- Debug log: no ERROR/WARN/panic clusters in the 3h scan window.
- Next fuel: `user-info.json` size / `credential_accounts.json` size / `.config.env` size, or a design-review polish when screenshots work.

Ralf: p50 path continues — size lanes mirror the path lanes already shipped for the same files.
