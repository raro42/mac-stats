# Morning surprise — 2026-09-14

Overnight autoresearch (20:00–06:00) for Ralf.

## Shipped tonight

| Version | What |
| --- | --- |
| **v0.1.1069** | Instant `/keep-histogram` · `keep histogram` · `gap histogram` · `keep gap histogram` · `histogram keep gap` (+ discard) — minute-bucket histogram (`<5m` · `5–10m` · `10–20m` · `20–30m` · `30–60m` · `≥60m`) of consecutive keep/discard gaps from `results.tsv` (tonight + all-time; digester Slowest filters). |
| **v0.1.1068** | Instant `/keep-gini` · `keep gini` · `gini keep gap` · `gini coefficient keep gap` · `gini gap between keeps` (+ discard) — Gini coefficient (0–1) of consecutive keep/discard gap lengths from `results.tsv` (tonight + all-time; digester Slowest filters). |
| **v0.1.1067** | Design review / CPU metrics: Temp ring + TEMP sparkline Fair thermal wash (Apple thermal Fair below ≥70°C hot pulse). Serious/Critical thermal marks Temp hot under 70°C — power-strip Heat parity. Recapture deferred (Screen Recording TCC); polish grace marked. |
| **v0.1.1066** | Instant `/keep-mode` · `keep mode` · `mode keep gap` · `modal keep gap` · `mode gap between keeps` (+ discard) — modal (most frequent minute-bucketed) consecutive keep/discard gaps from `results.tsv` (tonight + all-time; digester Slowest filters). |
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

Open the CPU window: when Heat is Fair and Temp is under 70°C, the Temperature ring and TEMP sparkline show a soft amber wash (no pulse). Serious/Critical still pulse hot. Instant ratchet: `/keep-histogram` or `gap histogram` (minute buckets of gap lengths). Also `/keep-gini` for inequality (0 = even pace).

## Notes

- Digester open empty this tick; design review due=false (grace). Fuel = standing backlog `/keep-histogram` after keep-gini.
- Install/kickstart after v0.1.1069; confirm Discord Ready.
