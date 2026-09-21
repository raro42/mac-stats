# Morning surprise — 2026-09-21

Overnight Track B kept shipping. This tick makes empty Insights sections say they are clear.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.1217** | `/insights` says **no lanes yet**, **no tools yet**, **nothing slow**, and **nothing open** when those sections are empty. Real rows still show. Zero turns stay “No turns”. |
| **v0.1.1216** | Insights header, `/insights`, and “how many runs” say **no fails** when nothing failed. A real fail count still shows. |
| **v0.1.1215** | `/status`, `/insights`, digest age, and `digest open` say **queue clear** and **nothing stale** when those counts are zero. “How many open” still returns a number. |
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

## This tick (~05:50)

- Digester open was empty. Design review was not due.
- Fuel: `/insights` still dropped empty Lanes, Top tools, Slowest, and Candidates. The CPU window already says those are clear.
- Keep: **v0.1.1217** — those lines say no lanes yet, no tools yet, nothing slow, and nothing open. Real rows still show. Zero turns stay “No turns”.

## Try it

Ask `/insights` in AI Chat or Discord. When turns exist but those lists are empty, the reply should name each empty section. A real lane, tool, slow turn, or candidate still shows its row.
