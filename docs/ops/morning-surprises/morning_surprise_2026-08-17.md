# Morning surprise — 2026-08-17

Overnight Track B (design review due: stale `feature-agent-ops.png`).

## Shipped

| Version | What |
|---------|------|
| **v0.1.484** | Agent Ops **Runs Insights** Digest open hints are clickable — preview + Load into AI Chat; health Digest card opens the first hint (Slowest/Candidates parity). |
| v0.1.483 | Health Next schedule / Last delivery click-to-preview |
| v0.1.482 | Runs Insights Slowest/Candidates click-to-preview |
| v0.1.481 | Overview Last delivery click-to-preview |
| v0.1.480 | Overview Knowledge click-to-preview |
| v0.1.479 | Overview Live click-to-preview |
| v0.1.478 | Overview Recent click-to-preview |
| v0.1.477 | Overview Schedules click-to-preview |
| v0.1.476 | Agents Load into AI Chat |

## Screenshot

`docs/screens/feature-agent-ops.png` recapture still deferred when Screen Recording TCC / no listable CPU window blocks `screencapture -l`.

## Digester

Open fuel was design-review only (latency sample empty after noise filters). Debug.log: single-instance WARN only.

## Next fuel

Prefer non-digest-hint-adjacent: Digester Discord traffic, rotate to stale `feature-processes` / `feature-cpu-metrics` when TCC allows a capture, or a sibling/OpenClaw port that maps to sessions/tools.
