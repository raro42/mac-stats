# Morning surprise — 2026-08-17

Overnight Track B (design review due: stale `feature-agent-ops.png`).

## Shipped

| Version | What |
|---------|------|
| **v0.1.489** | Agent Ops **overview Runs** card: recent-turn snapshot on the Command Center grid; click opens Runs + preview/Load into AI Chat; Runs tab gets overview active wash (Agents/Schedules/Live/Knowledge/Recent parity). |
| v0.1.488 | Agent Ops overview Agents card (enabled snapshot + click-to-open) |
| v0.1.487 | Agent Ops health Redmine → Redmine agent open |
| v0.1.486 | Agent Ops health Discord → Runs gateway preview + Load into AI Chat |
| v0.1.485 | Agent Ops health Version → primary agent open |
| v0.1.484 | Runs Insights Digest open hints click-to-preview + health Digest card |
| v0.1.483 | Health Next schedule / Last delivery click-to-preview |
| v0.1.482 | Runs Insights Slowest/Candidates click-to-preview |
| v0.1.481 | Overview Last delivery click-to-preview |
| v0.1.480 | Overview Knowledge click-to-preview |
| v0.1.479 | Overview Live click-to-preview |
| v0.1.478 | Overview Recent click-to-preview |
| v0.1.477 | Overview Schedules click-to-preview |

## Screenshot

`docs/screens/feature-agent-ops.png` recapture still deferred: no on-screen CPU window for Quartz/`screencapture -l` (Screen Recording TCC / headless tick). Prior Aug 12 asset kept.

## Digester

Open fuel was design-review only (latency sample empty after noise filters). Debug.log: single-instance WARN only.

## Next fuel

Prefer non-overview-runs-adjacent: Digester Discord traffic, rotate to stale `feature-processes` / `feature-cpu-metrics` when TCC allows a capture, or a sibling/OpenClaw port that maps to sessions/tools.
