# Morning surprise — 2026-09-12

Overnight Track B (20:00–06:00 local, night of 11→12). Digester open stayed empty; design review was in grace. Fuel: p50 operator NL after Ops Clear chips closed.

## Shipped

| Version | What |
|---------|------|
| **v0.1.1019** | Instant `/monitors` NL: `view monitors` · `see monitors` · `show me the monitors` · `open monitors` · `list the monitors` (and close variants); Up · Down · Slow filters unchanged |
| **v0.1.1018** | Instant `/processes` NL: `view` / `see` / `show me` / `open` / `list the` processes |
| **v0.1.1017** | Instant `/logs` NL + digester Slowest filters for Review logs / Instant wake-ups |
| **v0.1.1016** | Agent Ops Schedules Clear chip (Jobs · Deliveries) — last Ops Clear gap |
| **v0.1.1015** | Agent Ops Knowledge Clear chip (Discord · Core) |
| **v0.1.1014** | Agent Ops Runs Clear chip (Instant · Lite · Direct · Slow · Fail) |
| **v0.1.1013** | Agent Ops Agents Clear chip (On · Off) |
| **v0.1.1012** | Agent Ops Sessions Clear chip (Live · Files) |

## Tried / notes

- No digester open candidates; no debug.log ERROR/WARN clusters in the scan window.
- Design review `due=false` (grace); recommended surface still CPU metrics (~4.9d) when TCC allows a recapture.
- After Clear chips closed, ticks extended slash NL: `/logs` → `/processes` → `/monitors`. Next candidate: `/disk` view/see/show me/open.

## Head

`78f3032a` on `main` — Cargo.toml **0.1.1019**.
