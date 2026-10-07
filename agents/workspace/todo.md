# UNTESTED-14 / #14 — tauri://localhost CPU (v0.1.1724 details/process body type)

## Plan
- [x] Pick fuel: standing P2 / GitHub #14 after v0.1.1723 (battery-icon)
- [x] Opaque panel mix for `.details-grid` · `.process-table` body `color` (Detail labels · process rows)
- [x] Bump to v0.1.1724 + CHANGELOG + standing_backlog
- [x] `cargo check` / ratchet verify
- [x] Rename WIP-14 → UNTESTED-14
- [x] Commit + push `origin/main`
- [x] Leave GitHub issue alone (do not close)

## Review
- Coder: `.details-grid` / `.process-table` were `rgba(60, 60, 67, 0.72)` glass on the opaque white Details / Top Processes panels. Now `color-mix(in srgb, #3c3c43 72%, #ffffff)`.
- Tester next: open CPU window on macOS, warm ≥30s. Confirm Details labels and Top Processes rows stay solid. Gauges/sparklines still update. Activity Monitor Graphics and Media / `tauri://localhost` toward <1%.
- Next fuel after bounce: `.section-title` `rgb(1, 1, 1, 0.75)`; icon-line status-good/warning/bad type rgba; Changelog body type rgba.
