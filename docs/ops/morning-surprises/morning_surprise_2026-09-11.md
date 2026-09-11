# Morning surprise — 2026-09-11

Overnight Track B kept shipping. Digester open stayed empty; design-review grace covered stale feature screens.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.999** | Instant lane: harness loop **stderr** path (`overnight_harness_loop.stderr.log`) |
| **v0.1.998** | Disk Cleanup filter **Clear** chip (All · Reclaim · Big · Clean) |
| **v0.1.997** | Monitors filter **Clear** chip (All · Up · Down · Slow) |
| **v0.1.996** | Top Processes filter **Clear** chip (All · Pinned · Hot) |
| **v0.1.995** | Instant lane: harness loop stdout **age** |
| **v0.1.994** | Instant lane: harness loop stdout **size** |
| **v0.1.993** | Instant lane: harness loop stdout **path** |

## Tried / notes
- Design review due=false this tick; Clear chips landed earlier while screens were in polish grace.
- Next: stderr size·age, or design review when due / TCC allows recapture.

## Fitness
Operators get zero-LLM path for Track B stderr capture without stealing stdout / overnight_agent / debug.log lanes.
