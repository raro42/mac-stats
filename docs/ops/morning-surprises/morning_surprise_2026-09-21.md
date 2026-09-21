# Morning surprise — 2026-09-21

Overnight Track B kept shipping Agent Ops Insights empty-state calm.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.1212** | Insights header **mean n/a · max n/a** when the latency sample is noise-filtered |
| **v0.1.1211** | Insights **Lanes** empty calm — “No lanes yet” wash when by_lane is empty |
| **v0.1.1210** | Insights **Top tools** empty calm — “No tools yet” wash when digester Top tools is clear |
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

## This tick (~02:40)

- Digester open empty; design review grace on `feature-agent-ops`.
- Fuel: Insights header still said mean 0 ms · max 0 ms after the Latency block already said nothing to measure.
- Keep: header uses mean n/a · max n/a when the sample is empty. Real numbers stay when turns are in the sample.

## Try it

Open **Agent Ops → Runs**. On a quiet digester night the Insights header should say **mean n/a · max n/a**, and the Latency block should still say **Nothing to measure**. When a real latency sample exists, mean and max show milliseconds again.
