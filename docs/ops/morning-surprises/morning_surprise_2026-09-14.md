# Morning surprise — 2026-09-14

Overnight Track B (mac-stats autoresearch) for Ralf.

## Shipped

| Version | What |
|---------|------|
| **v0.1.1058** | Instant `/keep-p90` · `keep p90` · `p90 keep gap` · `90th percentile keep gap` · `p90 gap between keeps` (+ discard) — p90 gap between consecutive keep/discard rows from `results.tsv` (tonight + all-time; digester Slowest filters). |
| **v0.1.1057** | Instant `/keep-range` · `keep range` · `keep gap range` · `min max keep gap` · `shortest and longest keep gap` (+ discard) — min–max gap between consecutive keep/discard rows from `results.tsv` (tonight + all-time; digester Slowest filters). |
| **v0.1.1056** | Instant `/keep-median` · `keep median` · `median keep gap` · `median gap between keeps` (+ discard) — median gap between consecutive keep/discard rows from `results.tsv` (tonight + all-time; digester Slowest filters). |
| **v0.1.1055** | Instant `/keep-pace` · `keep pace` · `time between keeps` · `average keep gap` (+ discard) — average gap between consecutive keep/discard rows from `results.tsv` (tonight + all-time; digester Slowest filters). |
| **v0.1.1054** | Instant `/first-keep` · `first keep tonight` · `earliest keep` (+ discard) — earliest keep/discard row tonight since 20:00 from `results.tsv` (one description; digester Slowest filters). |
| **v0.1.1053** | Instant `/since-keep` — age since newest keep/discard (age only). |

## Context

- Digester open was empty; design review not due (grace).
- Fuel: standing backlog p50 / Hermes-style ratchet glances.
- Nightly keep minimum met (results.tsv keep rows).

## Try it

In AI Chat or Discord: `/keep-p90` or `90th percentile keep gap` (beside `/keep-pace` average, `/keep-median` median, and `/keep-range` min–max).
