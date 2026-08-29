# 2026-08-29 — Rings filter removed + sparkline history seed

## Rings All · Hot

Removed the All/Hot chip row above the CPU gauges (UI clutter). Amber `is-hot` wash remains. Discord/AI Chat `/rings` is unchanged.

## Sparklines starting empty

Cause: `chart-line.js` only buffered live `get_cpu_details` samples after the CPU window opened, while the menu-bar loop already wrote a tiered history buffer in memory (and never persisted it).

Fix (v0.1.717):

1. Load/save `~/.mac-stats/history.json` from the background metrics loop.
2. On CPU window open/visible, call `get_metrics_history` and `themeHistory.seedFromPoints(...)`.
