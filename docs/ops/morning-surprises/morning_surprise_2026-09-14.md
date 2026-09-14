# Morning surprise — 2026-09-14

Overnight autoresearch (20:00–06:00) for Ralf.

## Shipped tonight

| Version | What |
| --- | --- |
| **v0.1.1065** | Instant `/keep-entropy` · `keep entropy` · `entropy keep gap` · `shannon entropy keep gap` · `entropy gap between keeps` (+ discard) — Shannon entropy (bits) of minute-bucketed consecutive keep/discard gaps from `results.tsv` (tonight + all-time; digester Slowest filters). |
| **v0.1.1064** | Instant `/keep-kurtosis` · `keep kurtosis` · `kurtosis keep gap` · `excess kurtosis keep gap` · `kurtosis gap between keeps` (+ discard) — Fisher excess kurtosis G2 of consecutive keep/discard gaps from `results.tsv` (tonight + all-time; digester Slowest filters). |
| **v0.1.1063** | Instant `/keep-skew` · `keep skew` · `skew keep gap` · `skewness keep gap` · `skew gap between keeps` (+ discard) — Fisher–Pearson G1 skewness of consecutive keep/discard gaps from `results.tsv` (tonight + all-time; digester Slowest filters). |
| **v0.1.1062** | Instant `/keep-cv` · `keep cv` · `cv keep gap` · `coefficient of variation keep gap` · `cv gap between keeps` (+ discard) — CV (sample std/mean %) of consecutive keep/discard gaps from `results.tsv` (tonight + all-time; digester Slowest filters). |
| **v0.1.1061** | Instant `/keep-mad` · `keep mad` · `mad keep gap` · `median absolute deviation keep gap` · `mad gap between keeps` (+ discard) — MAD of consecutive keep/discard gaps from `results.tsv` (tonight + all-time; digester Slowest filters). |
| **v0.1.1060** | Instant `/keep-std` · `keep std` · `std keep gap` · `standard deviation keep gap` · `std gap between keeps` (+ discard) — sample std gap between consecutive keep/discard rows from `results.tsv` (tonight + all-time; digester Slowest filters). |
| **v0.1.1059** | Instant `/keep-iqr` · `keep iqr` · `iqr keep gap` · `interquartile keep gap` · `iqr gap between keeps` (+ discard) — IQR (Q3−Q1) gap between consecutive keep/discard rows from `results.tsv` (tonight + all-time; digester Slowest filters). |
| **v0.1.1058** | Instant `/keep-p90` · `keep p90` · `p90 keep gap` · `90th percentile keep gap` · `p90 gap between keeps` (+ discard) — p90 gap between consecutive keep/discard rows from `results.tsv` (tonight + all-time; digester Slowest filters). |
| **v0.1.1057** | Instant `/keep-range` min–max gap between keeps. |
| **v0.1.1056** | Instant `/keep-median` median gap between keeps. |
| **v0.1.1055** | Instant `/keep-pace` avg gap between keeps. |
| **v0.1.1054** | Instant `/first-keep` earliest tonight keep. |
| **v0.1.1053** | Instant `/since-keep` age since last keep. |

## Try it

In AI Chat or Discord: `/keep-entropy` or `shannon entropy keep gap` (beside `/keep-kurtosis` excess kurtosis, `/keep-skew` skewness, `/keep-cv` CV, `/keep-mad` MAD, `/keep-std` sample std, `/keep-pace` average, `/keep-median` median, `/keep-range` min–max, `/keep-p90` p90, and `/keep-iqr` IQR).

## Notes

- Digester open stayed empty; fuel from standing backlog + Hermes insights lens.
- Design review still in grace (CPU metrics ~6.9d recommended when due).
- Install/kickstart after v0.1.1065; Discord Ready.
