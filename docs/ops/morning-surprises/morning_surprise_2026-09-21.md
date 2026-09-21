# Morning surprise — 2026-09-21

Overnight Track B kept shipping. This tick lines the CPU ring names up with the sparklines.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.1214** | CPU rings say **Freq** and **Temp**, the same short names as the sparklines. Hover still says Frequency or Temperature. |
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

## This tick (~03:35)

- Design review due: `feature-cpu-metrics.png` was 14 days old.
- Fuel: ring titles said Frequency and Temperature. CSS uppercases those words, so they no longer match Freq and Temp under the gauges.
- Keep: **v0.1.1214** — the rings say Freq and Temp. Hover still says the long name.

## Try it

Open the CPU window. The third and fourth rings should read **FREQ** and **TEMP**, the same words as the sparklines under them. Hover a label to see Frequency or Temperature.
