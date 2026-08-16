# Morning surprise — 2026-08-17

Updated: 2026-08-16 23:38 (overnight harness)

## Shipped tonight (Track B)

| Version | What |
|---------|------|
| **v0.1.477** | Agent Ops **overview Schedules** click → Schedules tab + full task preview (Live/Knowledge/Recent parity) |
| v0.1.476 | Agent Ops Agents Load into AI Chat |
| v0.1.475 | Agent Ops Knowledge Load into AI Chat |
| v0.1.474 | Agent Ops Schedules Load into AI Chat |
| v0.1.473 | Agent Ops Runs Load into AI Chat |
| v0.1.472 | Agent Ops Schedules/deliveries id copy chip |
| v0.1.471 | Agent Ops Runs request-id copy chip |

## Design review

- Digester open = stale `feature-agent-ops.png` (~4.6d); `overnight_design_review.py` due=true.
- Polish: overview Schedules rows no longer only switch tabs — they open preview + select the matching list row (non-load-adjacent).
- Recapture still deferred: `screencapture -l` → `could not create image from window` (Screen Recording TCC). Prior Aug 12 asset kept.

## Digester / debug

- Latency n/a (instant noise filtered); no Slowest open product candidates beyond design-review.
- `debug.log`: single-instance WARN only (expected KeepAlive / launch race).
- Discord Ready after install/kickstart (Werner_Amvara).

## Next

- Prefer non-overview-schedules-adjacent fuel (e.g. processes polish, `feature-processes.png` when TCC allows, or Discord traffic).
- Finish deferred `docs/screens/feature-agent-ops.png` / `feature-cpu-metrics.png` when Screen Recording permits.
