# Morning surprise — 2026-09-11

Overnight Track B (20:00–06:00 local). Digester open stayed empty; design review was in grace. Fuel: p50 Slowest noise + `/logs` NL.

## Shipped

| Version | What |
|---------|------|
| **v0.1.1017** | Instant `/logs` NL: `view logs` · `see logs` · `show me the logs` · `open logs` · `list the logs`; digester drops historical Review logs + Instant wake-ups from Slowest/p50 |
| **v0.1.1016** | Agent Ops Schedules: Clear chip beside All · Jobs · Deliveries (Knowledge / Runs / Sessions parity; Cleared flash; filter-miss Clear) — last Ops Clear gap |
| **v0.1.1015** | Agent Ops Knowledge: Clear chip beside All · Discord · Core (Runs / Sessions / Agents parity; Cleared flash; filter-miss Clear) |
| **v0.1.1014** | Agent Ops Runs: Clear chip beside All · Instant · Lite · Direct · Slow · Fail |
| **v0.1.1013** | Agent Ops Agents: Clear chip beside All · On · Off |
| **v0.1.1012** | Agent Ops Sessions: Clear chip beside All · Live · Files |
| **v0.1.1011** | Disk Cleanup scopes: Clear chip beside All · On · Off |
| **v0.1.1010** | Perplexity Search: Clear chip beside All · Top · Snippet |
| **v0.1.1009** | Debug Log: Clear chip beside All · Error · Warn |

## Tried / notes

- No digester open candidates; no debug.log ERROR/WARN clusters in the scan window.
- Design review `due=false` (grace); recommended surface still CPU metrics (~4.8d) when TCC allows a recapture.
- Digester Slowest cleaned: wake-ups + Review logs no longer inflate p50.

## Head

`06c791d9` on `main` — Cargo.toml **0.1.1017**.
