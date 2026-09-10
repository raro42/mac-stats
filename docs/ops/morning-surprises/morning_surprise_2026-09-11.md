# Morning surprise — 2026-09-11

Overnight Track B kept shipping. Digester Slowest was mostly wake-up noise; design review was **due**.

## Shipped tonight (highlights)

| Version | What |
|---------|------|
| **v0.1.996** | Top Processes: Clear chip beside All · Pinned · Hot (AI Chat / Ops parity; Cleared flash) |
| **v0.1.995** | Instant lane: harness loop stdout age |
| **v0.1.994** | Instant lane: harness loop stdout size |
| **v0.1.993** | Instant lane: harness loop stdout path |
| **v0.1.992** | Instant lane: overnight agent log age |
| **v0.1.991** | Instant lane: overnight agent log size |
| **v0.1.990** | Instant lane: overnight agent log path |
| **v0.1.989** | LaunchAgent WorkingDirectory (repo/HOME, not `/`) |
| **v0.1.988** | RUN_CMD workspace cwd (LaunchAgent `/` fix) |
| **v0.1.987** | Instant lane: morning surprise age |

## Design review

- Surface: `feature-processes` (stale ~28d).
- Polish: filter Clear chip (same pattern as AI Chat **v0.1.960**).
- Screenshot: TCC blocked `screencapture -l` — prior asset kept; polish grace marked.

## Tried / notes

- Debug log: LaunchAgent cwd `/` still shows up in older RUN_CMD failures (`//scripts/…`); **v0.1.988–989** address that path.
- Next fuel: harness loop **stderr** path·size·age; recapture processes when Screen Recording allows.
