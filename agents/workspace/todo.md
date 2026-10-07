# UNTESTED-14 / #14 — tauri://localhost CPU (v0.1.1723 battery-icon)

## Plan
- [x] Pick fuel: standing P2 / GitHub #14 after v0.1.1722 (apple-title)
- [x] Opaque strip mix for `.battery-icon` resting · charging `color` (always-visible Bat glyph)
- [x] Bump to v0.1.1723 + CHANGELOG + standing_backlog
- [x] `cargo check` / ratchet verify
- [x] Rename WIP-14 → UNTESTED-14
- [x] Commit + push `origin/main`
- [x] Leave GitHub issue alone (do not close)

## Review
- Coder: `.battery-icon` resting/charging were `rgba(12, 12, 16, 0.50)` / `rgba(52, 199, 89, 0.8)` glass on the always-visible power strip. Now `color-mix` against opaque strip `#ececf1`.
- Tester next: open CPU window on macOS, warm ≥30s. Confirm Bat glyph stays solid (resting + charging). Gauges/sparklines still update. Activity Monitor Graphics and Media / `tauri://localhost` toward <1%.
- Next fuel after bounce: Details · Top Processes body `rgba(60, 60, 67, 0.72)`; icon-line status-good/warning/bad type rgba.
