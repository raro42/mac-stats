# Morning surprise — 2026-08-17

Overnight Track B (from 2026-08-16 20:00). Digester open = design-review (stale `feature-agent-ops.png`); design-review due. Product UI shipped — not quiet.

## Shipped

| Version | What |
|---------|------|
| **v0.1.468** | Agent Ops **Runs**: click a run → preview question, tools, lane, wall time, request id (Esc dismisses) |
| **v0.1.469** | Agent Ops **Agents**: click-to-copy slug/id chip under detail meta (Copied flash; Sessions/Knowledge parity) |
| **v0.1.470** | Data Poster: inactive section icons near-white (were nearly invisible on dark UI) + install rebuild-when-dist-newer gate |

## Tried / deferred

- Recapture of `docs/screens/feature-agent-ops.png` still deferred when Screen Recording TCC blocks window-only capture.
- Concurrent overnight ticks shared the tree; `pkill -f mac_stats` briefly SIGKILL’d rustc (argv contains `crate-name mac_stats`) — prefer path-scoped kills.

## Ratchet

Keeps in `~/.mac-stats/improvements/autoresearch/results.tsv` include `6169a9a` (v0.1.469 Agents copy chip). Nightly minimum met.
