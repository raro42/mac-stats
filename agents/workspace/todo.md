# UNTESTED-27: RAM usage as ring widget (#27)

## Plan
- [x] Add RAM metric-card + history sparkline to all theme `cpu.html` (after GPU)
- [x] CSS grids: metrics + history `repeat(4)` → `repeat(5)` in theme `cpu.css`
- [x] `chart-line.js` + data-poster `poster-charts.js`: ram buffer / seed / updateRam
- [x] `cpu.js`: live ring, hot wash (85%), seed gauges, feed charts, kb/history maps
- [x] Harness `/rings`: include RAM + hot threshold
- [x] Bump v0.1.2915 + CHANGELOG; sync dist; `cargo check`
- [x] Rename `WIP-27-…` → `UNTESTED-27-…`; commit + push `origin/main` (issue left open)

## Notes
GitHub: https://github.com/raro42/mac-stats/issues/27
RAM already in history.json / Details; gauge chips were pruned from the power strip — ring was missing.

## Review
- v0.1.2915: RAM ring + sparkline across themes; `/rings` lists 5.
- Handoff: tester / 004 (issue left open).
