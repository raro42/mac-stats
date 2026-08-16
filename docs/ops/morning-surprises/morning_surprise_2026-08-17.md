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

## Tried / deferred

- Recapture of `docs/screens/feature-agent-ops.png` still deferred — no window id / Screen Recording TCC (`screencapture -l`).
- Install `cmp -s` on ~50MB Mach-O sometimes gets SIGKILL overnight; install script now checks size after `cp` instead.

## Ratchet

Keeps in `~/.mac-stats/improvements/autoresearch/results.tsv` include `0f35805` (v0.1.472 Schedules id chip). Nightly minimum met.
