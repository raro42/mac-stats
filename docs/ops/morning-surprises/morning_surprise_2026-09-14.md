# Morning surprise — 2026-09-14

Overnight autoresearch (20:00–06:00) for Ralf.

## Shipped tonight

| Version | What |
| --- | --- |
| **v0.1.1059** | Instant `/keep-iqr` · `keep iqr` · `iqr keep gap` · `interquartile keep gap` · `iqr gap between keeps` (+ discard) — IQR (Q3−Q1) gap between consecutive keep/discard rows from `results.tsv` (tonight + all-time; digester Slowest filters). |
| **v0.1.1058** | Instant `/keep-p90` · `keep p90` · `p90 keep gap` · `90th percentile keep gap` · `p90 gap between keeps` (+ discard) — p90 gap between consecutive keep/discard rows from `results.tsv` (tonight + all-time; digester Slowest filters). |
| **v0.1.1057** | Instant `/keep-range` min–max gap between keeps. |
| **v0.1.1056** | Instant `/keep-median` median gap between keeps. |
| **v0.1.1055** | Instant `/keep-pace` avg gap between keeps. |
| **v0.1.1054** | Instant `/first-keep` earliest tonight keep. |
| **v0.1.1053** | Instant `/since-keep` age since last keep. |

## Try it

In AI Chat or Discord: `/keep-iqr` or `interquartile keep gap` (beside `/keep-pace` average, `/keep-median` median, `/keep-range` min–max, and `/keep-p90` p90).

## Notes

- Digester open stayed empty; fuel from standing backlog + Hermes insights lens.
- Design review still in grace (CPU metrics ~6.8d recommended when due).
- Install/kickstart after v0.1.1059; Discord Ready.
