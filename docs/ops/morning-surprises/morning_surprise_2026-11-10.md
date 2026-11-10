# Morning surprise — 2026-11-10

Overnight autoresearch kept shipping the keep/discard gap percentile ladder (digester empty; design-review/layout grace on the Linux rack).

## Shipped tonight
- **v0.1.2544** — Instant `/keep-p30` (nearest-rank p30 gap between consecutive keep/discard rows; densifies between `/keep-p25` and `/keep-p40`).
- Earlier keeps same night: `/keep-p70` (v0.1.2543), `/keep-p60` (v0.1.2542), `/keep-p40` (v0.1.2541), plus the densify ladder from `/keep-p1` … `/keep-p85`.

## Also fixed
- Corrected a leftover `keep gap p60` match typo inside `/keep-p70` detector while cloning p30.
- Wired `#[test]` neighbor cross-steals for p25/p40/p70 ↔ p30.

## Deferred
- Dark layout-daily capture (theme remaps green; `theme-*.png` / Screen Recording TCC still blocked on this rack).
- Intentional scrollbar glass left alone.
