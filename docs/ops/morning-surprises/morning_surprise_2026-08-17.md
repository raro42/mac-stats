# Morning surprise — 2026-08-17

Overnight Track B (from 2026-08-16 20:00). Digester open = design-review (stale `feature-agent-ops.png`); design-review due. Product UI shipped — not quiet.

## Shipped

| Version | What |
|---------|------|
| **v0.1.468** | Agent Ops **Runs**: click a run → preview question, tools, lane, wall time, request id (Esc dismisses) |
| **v0.1.469** | Agent Ops **Agents**: click-to-copy slug/id chip under detail meta (Copied flash; Sessions/Knowledge parity) |
| **v0.1.470** | Data Poster: inactive section icons near-white (were nearly invisible on dark UI) + install rebuild-when-dist-newer gate |
| **v0.1.471** | Agent Ops **Runs**: click-to-copy request id chip under run preview (Copied flash; Sessions/Knowledge parity) |
| **v0.1.472** | Agent Ops **Schedules**: click-to-copy schedule id chip under schedule/delivery preview (Copied flash; Sessions/Knowledge parity) |
| **v0.1.473** | Agent Ops **Runs**: Load into AI Chat puts the run question into the composer (Enter / double-click; Loaded flash; Sessions parity) |
| **v0.1.474** | Agent Ops **Schedules**: Load into AI Chat puts the schedule task or delivery summary into the composer (Enter / double-click; Loaded flash; Sessions/Runs parity) |

## Tried / deferred

- Recapture of `docs/screens/feature-agent-ops.png` still deferred — Screen Recording TCC / no on-screen CPU window for capture.
- Install `cmp -s` on ~50MB Mach-O sometimes gets SIGKILL overnight; install script now checks size after `cp` instead.

## Notes

- Digester Slowest empty after filters; open candidate remains design-review.
- Keep logged @ `ca69124` (v0.1.474). Discord Ready after install/kickstart.
