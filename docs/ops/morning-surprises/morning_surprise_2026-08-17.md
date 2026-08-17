# Morning surprise — 2026-08-17

Overnight Track B kept shipping Agent Ops Command Center polish (design-review due: stale `feature-agent-ops`).

## Shipped tonight (keeps)

| Version | What |
|---------|------|
| **v0.1.497** | Overview **Live** ok/warn/bad wash (health Discord gateway parity) |
| v0.1.496 | Overview **Digest** ok/warn/bad wash (health Digest fail/open parity) |
| v0.1.495 | Overview **Runs** ok/warn/bad wash (health Digest fail/open parity) |
| v0.1.494 | Overview **Agents** ok/warn/bad wash (health Version agent-count parity) |
| v0.1.493 | Overview **Schedules** ok/warn/bad wash (health schedule/delivery parity) |
| v0.1.492 | Health **Version** ok/warn/bad wash (Discord/Redmine/Digest parity) |
| v0.1.491 | Health **Next schedule / Last delivery** ok/warn/bad wash |
| v0.1.490 | Overview **Digest** card — digester open hints with click-to-preview + Load into AI Chat |
| v0.1.489 | Overview **Runs** card — recent-turn snapshot + click-to-preview |
| v0.1.488 | Overview **Agents** card — enabled/orchestrator snapshot + click-to-open |
| v0.1.487 | Health **Redmine** → Redmine agent open + Load into AI Chat |
| v0.1.486 | Health **Discord** → Runs gateway preview + Load into AI Chat |
| v0.1.485 | Health **Version** → primary agent open |
| v0.1.484 | Digest open hints click-to-preview + health Digest card |
| v0.1.483 | Health Next schedule / Last delivery click-to-preview |
| v0.1.482 | Runs Insights Slowest/Candidates click-to-preview |

Latest keep: `445a895` on `main` (pushed; install/kickstart to **v0.1.497**).

## Tried / deferred

- Recapture `docs/screens/feature-agent-ops.png` — still deferred (no on-screen CPU window for `screencapture -l`).
- Digester Slowest empty (instant noise filtered); night fuel = design review, not quiet.

## For Ralf

Open CPU → Agent Ops overview **Live** card: red when Discord is disconnected; amber on reconnect noise or when nothing is live; green when live sessions show and the gateway is ready. Matches the Discord health wash.
