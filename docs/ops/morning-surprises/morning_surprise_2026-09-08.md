# Morning surprise — 2026-09-08

Overnight Track B (20:00–06:00) shipped more **p50 instant** age lanes for config files.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.931** | Instant: `perplexity_last.json` age (`perplexity last age` / `how old is perplexity last`) |
| **v0.1.930** | Instant: `discord_channels.json` age (`discord channels age` / `how old is discord channels`) |
| **v0.1.929** | Instant: `pinned_processes.json` age (`pinned processes age` / `how old is pinned processes`) |
| **v0.1.928** | Instant: `disk_cleanup.json` age |
| **v0.1.927** | Instant: `history.json` age |
| **v0.1.926** | Instant: `monitors.json` age |
| **v0.1.925** | Instant: `schedules.json` age |
| **v0.1.924** | Instant: `config.json` age |

## Fuel

- Digester **open** stayed empty (stale Elmasnow weather; Review logs already instant).
- Design review **not due** (grace).
- Standing backlog p50 → next JSON age after discord_channels: **perplexity_last**.

## Try

```text
how old is perplexity last
perplexity_last.json age
when was perplexity last updated
```

Expect a last-write age without Ollama. Path / size / `/perplexity` stay separate.
