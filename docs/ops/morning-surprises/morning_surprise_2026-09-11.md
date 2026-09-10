# Morning surprise — 2026-09-11

Overnight Track B (20:00–06:00 local) for Ralf.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.993** | Instant lane: **harness loop stdout path** (`harness loop stdout path`, `where is overnight_harness_loop.stdout.log`) — path only; no dump/tail |
| **v0.1.992** | Instant lane: overnight agent log age |
| **v0.1.991** | Instant lane: overnight agent log size |
| **v0.1.990** | Instant lane: overnight agent log path |
| **v0.1.989** | LaunchAgent WorkingDirectory (repo/HOME, not `/`) |
| **v0.1.988** | RUN_CMD workspace cwd (relative `scripts/…` under LaunchAgent) |
| **v0.1.987** | Instant lane: morning surprise age |

## Why it matters

Operators can ask where the harness loop stdout capture lives without dumping a megabyte log or confusing it with `overnight_agent.log` / `debug.log` / loop backlog.

## Digester / design review

- Digester open: empty (stale “Review logs” + wake-up instant noise).
- Design review: grace (CPU metrics ~3.8d).

## Next fuel

1. Harness loop stdout **size** then **age**.
2. Harness loop stderr path·size·age.
3. Digester open / product-owned `debug.log` errors when present.
4. Design-review recapture when due / TCC allows.
