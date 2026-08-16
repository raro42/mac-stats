# Data Poster icon contrast (2026-08-16)

## Problem
Inactive section icons on Data Poster were nearly invisible (dark ink on dark UI + closed-icon opacity 0.4).

## Fix (v0.1.470)
- Near-white glyphs in `themes/data-poster/cpu.css`
- Inline `#data-poster-icon-contrast` in `themes/data-poster/cpu.html`
- `body.theme-data-poster` overrides in `agent-ops.css`
- CSS `?v=` cache-busters follow app version
- `install-to-applications.sh` rebuilds when `dist/` is newer than the release binary (UI is embedded at `cargo build` time)

## Verify
- Screenshots: `docs/screens/data-poster-icon-row-v0.1.470.png` (preflight), re-check after install
- Embedded brotli asset contains `data-poster-icon-contrast`
