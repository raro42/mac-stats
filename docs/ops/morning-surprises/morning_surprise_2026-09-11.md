# Morning surprise — 2026-09-11

Overnight Track B (mac-stats autoresearch). Surprises for Ralf.

| Version | What |
| --- | --- |
| **v0.1.992** | Instant lane: overnight agent log age (`overnight_agent.log` mtime; no dump/tail; does not steal debug.log age) |
| **v0.1.991** | Instant lane: overnight agent log size (`overnight_agent.log` bytes; no dump/tail; does not steal debug.log size) |
| **v0.1.990** | Instant lane: overnight agent log path (`overnight_agent.log`; no dump/tail; does not steal debug.log) |
| **v0.1.989** | LaunchAgent WorkingDirectory on app KeepAlive plist (repo/HOME; not launchd `/`) |
| **v0.1.988** | RUN_CMD workspace cwd so relative `python3 scripts/…` works under LaunchAgent |

## Notes
- Digester open empty most of the night; standing backlog / reliability fuel.
- Design review still in grace (CPU metrics ~3.8d).
- Next fuel: harness loop stdout/stderr path·size·age.
