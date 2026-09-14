# Morning surprise — 2026-09-14

Overnight autoresearch (20:00–06:00) for Ralf.

## Shipped tonight

| Version | What |
| --- | --- |
| **v0.1.1073** | Design review / Agent Ops filter-miss calm — warm title (“Nothing here yet”) + solid accent wash when a list filter finds no rows; Fail lane empty uses soft green (“No failed turns”). Recapture of `feature-agent-ops.png` deferred (Screen Recording TCC); polish grace. |
| **v0.1.1072** | Instant `/keep-p25` · `keep p25` · `p25 keep gap` · `25th percentile keep gap` · `p25 gap between keeps` (+ discard) — nearest-rank p25 (floor quartile) of consecutive keep/discard gaps from `results.tsv` (tonight + all-time; digester Slowest filters). |
| **v0.1.1071** | Instant `/keep-p10` · `keep p10` · `p10 keep gap` · `10th percentile keep gap` · `p10 gap between keeps` (+ discard) — nearest-rank p10 of consecutive keep/discard gaps from `results.tsv` (tonight + all-time; digester Slowest filters). |
| **v0.1.1070** | Instant `/keep-p95` · `keep p95` · `p95 keep gap` · `95th percentile keep gap` · `p95 gap between keeps` (+ discard) — nearest-rank p95 of consecutive keep/discard gaps from `results.tsv` (tonight + all-time; digester Slowest filters). |
| **v0.1.1069** | Instant `/keep-histogram` · `keep histogram` · `gap histogram` · `keep gap histogram` · `histogram keep gap` (+ discard) — minute-bucket histogram of consecutive keep/discard gaps from `results.tsv` (tonight + all-time; digester Slowest filters). |
| **v0.1.1068** | Instant `/keep-gini` · `keep gini` · `gini keep gap` · `gini coefficient keep gap` · `gini gap between keeps` (+ discard) — Gini coefficient (0–1) of consecutive keep/discard gap lengths from `results.tsv` (tonight + all-time; digester Slowest filters). |
| **v0.1.1067** | Design review / CPU metrics: Temp ring + TEMP sparkline Fair thermal wash (Apple thermal Fair below ≥70°C hot pulse). Serious/Critical thermal marks Temp hot under 70°C — power-strip Heat parity. Recapture deferred (Screen Recording TCC); polish grace marked. |
| **v0.1.1066** | Instant `/keep-mode` modal gap between keeps. |
| **v0.1.1065** | Instant `/keep-entropy` Shannon entropy of keep gaps. |
| **v0.1.1064** | Instant `/keep-kurtosis` excess kurtosis gap between keeps. |
| **v0.1.1063** | Instant `/keep-skew` skewness gap between keeps. |
| **v0.1.1062** | Instant `/keep-cv` CV gap between keeps. |
| **v0.1.1061** | Instant `/keep-mad` MAD gap between keeps. |
| **v0.1.1060** | Instant `/keep-std` sample std gap between keeps. |
| **v0.1.1059** | Instant `/keep-iqr` IQR gap between keeps. |
| **v0.1.1058** | Instant `/keep-p90` p90 gap between keeps. |
| **v0.1.1057** | Instant `/keep-range` min–max gap between keeps. |
| **v0.1.1056** | Instant `/keep-median` median gap between keeps. |
| **v0.1.1055** | Instant `/keep-pace` avg gap between keeps. |
| **v0.1.1054** | Instant `/first-keep` earliest tonight keep. |
| **v0.1.1053** | Instant `/since-keep` age since last keep. |

## Try it

Open Agent Ops → pick a lane filter with zero rows (e.g. Fail when nothing failed). You should see “Nothing here yet” with a calm accent wash (green on Fail empty), not a dashed muted box. Temp Fair wash still applies on the CPU rings. Instant ratchet: `/keep-p25` for the floor quartile.

## Notes

- Final overnight tick fuel = design review (stale `feature-agent-ops.png`); digester open pointed at the same surface.
- Install/kickstart after v0.1.1073; Discord Ready confirmed.
- Screen Recording TCC still blocks `screencapture -l` for marketing PNGs — grace kept on Aug 12 Agent Ops asset.
