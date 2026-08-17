# Morning surprise — 2026-08-17

Overnight Track B (design review due: stale `feature-agent-ops.png`).

## Shipped

| Version | What |
|---------|------|
| **v0.1.486** | Agent Ops **health Discord**: click opens Runs with gateway status preview + Load into AI Chat; Runs Insights Discord line is clickable (Digest/Version health parity). |
| v0.1.485 | Agent Ops health Version → primary agent open |
| v0.1.484 | Runs Insights Digest open hints click-to-preview + health Digest card |
| v0.1.483 | Health Next schedule / Last delivery click-to-preview |
| v0.1.482 | Runs Insights Slowest/Candidates click-to-preview |
| v0.1.481 | Overview Last delivery click-to-preview |
| v0.1.480 | Overview Knowledge click-to-preview |
| v0.1.479 | Overview Live click-to-preview |
| v0.1.478 | Overview Recent click-to-preview |
| v0.1.477 | Overview Schedules click-to-preview |
| v0.1.476 | Agents Load into AI Chat |

## Screenshot

`docs/screens/feature-agent-ops.png` recapture still deferred: `screencapture -l` returns “could not create image from window” (Screen Recording TCC). Window was open and listable via Quartz.

## Digester

Open fuel was design-review only (latency sample empty after noise filters). Debug.log: single-instance WARN only.

## Next fuel

Prefer non-discord-health-adjacent: Digester Discord traffic, rotate to stale `feature-processes` / `feature-cpu-metrics` when TCC allows a capture, health Redmine deeper preview, or a sibling/OpenClaw port that maps to sessions/tools.
