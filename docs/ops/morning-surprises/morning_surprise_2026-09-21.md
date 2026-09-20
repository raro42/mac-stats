# Morning surprise — 2026-09-21

Overnight Track B kept shipping Agent Ops Insights empty-state calm.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.1209** | Insights **Latency** empty calm — “Nothing to measure” wash when p50 is noise-filtered |
| **v0.1.1208** | Insights **Stale** empty calm — “Nothing stale” |
| **v0.1.1207** | Insights **Candidates** empty calm — “Nothing open” |
| **v0.1.1206** | Insights **Slowest** empty calm — “Nothing slow” |
| **v0.1.1205** | Digest open **Queue clear** calm |
| **v0.1.1204** | Agent Ops true-empty tab calm |
| **v0.1.1203** | Debug Log inventory counts only |
| **v0.1.1202** | Keep/discard inventory counts only |
| **v0.1.1201** | Schedule/delivery inventory counts only |
| **v0.1.1200** | Digest open inventory counts only |
| **v0.1.1199** | Runs inventory counts only |

## This tick (~01:20)

- Digester open empty; design review grace on `feature-agent-ops`.
- Fuel: Insights Latency was only in the health tooltip; digester showed Latency n/a after filters.
- Keep: warm empty when there is nothing to measure; clickable p50/mean/max line when the sample has turns.

## Try it

Open **Agent Ops → Runs**. With a quiet digester night you should see **Latency · Nothing to measure** under Top tools, beside Slowest / Candidates / Stale calm blocks.
