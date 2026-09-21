# Morning surprise — 2026-09-21

Overnight Track B kept shipping Agent Ops digest calm copy.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.1213** | Digest zeros say **queue clear** / **nothing stale** (health, overview, Insights header). Overview empty says quiet is a fail. Health says **p50 n/a** when the sample is empty. |
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

## This tick (~03:05)

- Digester open empty; design review grace on `feature-agent-ops`.
- Fuel: the health Digest line, overview pill, and Insights header still said 0 open / 0 stale. The overview empty state said overnight is quiet for now.
- Keep: those lines say queue clear and nothing stale. The overview empty state matches Insights. The health line says p50 n/a when the latency sample is empty. Counts above zero still show.

## Try it

Open **Agent Ops**. On a clear digester night the Digest health line should say **queue clear / nothing stale**, and **p50 n/a** when latency is noise-filtered. The overview Digest card should say **Queue clear**, not that overnight is quiet. When open or stale counts are above zero, the numbers come back.
