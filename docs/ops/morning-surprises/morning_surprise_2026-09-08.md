# Morning surprise — 2026-09-08

Overnight Track B (20:00–06:00) shipped more **p50 instant** age lanes for config files.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.929** | Instant: `pinned_processes.json` age (`pinned processes age` / `how old is pinned processes`) |
| **v0.1.928** | Instant: `disk_cleanup.json` age |
| **v0.1.927** | Instant: `history.json` age |
| **v0.1.926** | Instant: `monitors.json` age |
| **v0.1.925** | Instant: `schedules.json` age |
| **v0.1.924** | Instant: `config.json` age |

## Fuel

- Digester **open** stayed empty (stale Elmasnow weather; Review logs already instant).
- Design review **not due** (grace).
- Standing backlog p50 → next JSON age after disk_cleanup: **pinned_processes**.

## Try

```text
how old is pinned processes
pinned_processes.json age
when was pinned processes updated
```

Expect a last-write age without Ollama. Path / size / `/pinned` stay separate.
