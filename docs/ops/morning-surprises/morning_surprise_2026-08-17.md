# Morning surprise — 2026-08-17

Updated: 2026-08-16 23:09 (overnight harness)

## Shipped tonight (Track B)

| Version | What |
|---------|------|
| **v0.1.476** | Agent Ops **Agents** Load into AI Chat — open soul/skill/mood → composer (Enter / double-click; Loaded flash) |
| v0.1.475 | Agent Ops Knowledge Load into AI Chat |
| v0.1.474 | Agent Ops Schedules Load into AI Chat |
| v0.1.473 | Agent Ops Runs Load into AI Chat |
| v0.1.472 | Agent Ops Schedules/deliveries id copy chip |
| v0.1.471 | Agent Ops Runs request-id copy chip |

## Design review

- Digester open = stale `feature-agent-ops.png` (~4.6d); `overnight_design_review.py` due=true.
- Recapture still deferred when Screen Recording TCC blocks Quartz window list.
- Polish continued on Agent Ops Agents tab (parity with Sessions/Knowledge load).

## Digester / debug

- Latency n/a (instant noise filtered); no Slowest open product candidates beyond design-review.
- `debug.log`: single-instance WARN only (expected KeepAlive / launch race).

## Next

- Prefer non-agents-load-adjacent fuel when digester empty (e.g. processes keyboard polish, screenshot when TCC allows, or Discord traffic).
- Finish deferred `docs/screens/feature-agent-ops.png` / `feature-cpu-metrics.png` when a CPU window is capturable.
