# 2026-08-28 — Temp sparkline, kb-hints, GitHub tooltip

## Temp graph blank under Temperature gauge

**Cause (frontend):**

1. Temperature samples were gated on the 3s DOM/ring throttle (`includeTemperature`), so the Temp sparkline often got no points while the gauge already showed °C.
2. `chart-line.js` dropped samples when the canvas context was not ready yet (zero-size layout / late GPU chart inject).
3. `feedThemeHistoryCharts` called every theme alias (`appleHistory`, `darkHistory`, …) even though they are the same `themeHistory` object.

**Fix (v0.1.702):** feed temp every refresh when `temperature > 0`; buffer before canvas setup; call each chart API once.

## Soft keyboard tips shifting layout

**Cause:** focus-within CSS revealed long `*-kb-hint` lines (footer, filters, rings, …). Clicking a control (e.g. GitHub) made the essay appear and push the UI.

**Fix:** keep all `*-kb-hint` / `.ops-keyboard-hint` at `display: none !important`. Keyboard shortcuts still work; tips stay out of the layout.

## GitHub icon tooltip

**Was:** `title="GitHub"`. **Now:** `title` = the repo URL (`https://github.com/raro42/mac-stats`).
