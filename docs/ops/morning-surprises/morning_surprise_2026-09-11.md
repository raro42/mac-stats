# Morning surprise — 2026-09-11

Overnight Track B kept shipping operator instant lanes for harness observability.

## Shipped tonight
- **v0.1.995** — Instant: harness loop stdout **age** (`overnight_harness_loop.stdout.log` mtime).
- **v0.1.994** — Instant: harness loop stdout **size**.
- **v0.1.993** — Instant: harness loop stdout **path**.
- **v0.1.992** — Instant: overnight agent log **age**.
- **v0.1.991** — Instant: overnight agent log **size**.
- **v0.1.990** — Instant: overnight agent log **path**.
- **v0.1.989** — LaunchAgent WorkingDirectory (repo/HOME, not `/`).
- **v0.1.988** — RUN_CMD workspace cwd (relative `python3 scripts/…`).

## Tried / notes
- Digester open stayed empty (stale “Review logs” + wake-up noise).
- Design review not due (CPU metrics still in grace).
- Next fuel: harness loop **stderr** path·size·age.

## Why it matters
Ralf can ask “how old is overnight_harness_loop.stdout.log?” and get an instant mtime answer — no LLM, no dump.
