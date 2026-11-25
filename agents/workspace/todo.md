# WIP-28 Layout problems

## Plan
- [x] Locate time-range selector markup/CSS/JS across themes
- [x] Remove time selector UI from all themes (keep default history window)
- [x] Fix neon theme: scroll + widen with window
- [x] Rename top "CPU" label/title to "mac-stats" (warm title rule)
- [x] cargo check in src-tauri/
- [x] CHANGELOG under [Unreleased]
- [x] Rename WIP-28 → UNTESTED-28; commit + push; do not close issue

## Review
- Root cause: `#history-controls` sat inside the 5-col `.history-section` grid; when shown (`display:flex` after >24h history), it stole a column and misaligned sparklines under gauges.
- Neon: `.neon-panel` had `max-width: 520px` and no `overflow-y: auto` (scroll lived on decorative `.neon-frame` with `pointer-events: none`).
- Removed 24h history availability poll (only existed to show the dropdown).
