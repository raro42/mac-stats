# UNTESTED-14 / #14 — tauri://localhost CPU (v0.1.1720 icon-line type)

## Plan
- [x] Pick fuel: standing P2 / GitHub #14 after v0.1.1719 (text/muted)
- [x] Opaque shell mix for `.icon-line-item` resting · hover `color` (always-visible Monitors · AI Chat strip glyphs)
- [x] Bump to v0.1.1720 + CHANGELOG + standing_backlog
- [x] `cargo check` in `src-tauri/`
- [x] Rename WIP-14 → UNTESTED-14
- [x] Commit + push `origin/main`
- [x] Leave GitHub issue alone (do not close)

## Review
- Coder: `.icon-line-item` resting/hover were `rgba(12, 12, 16, …)` glass under the always-visible Monitors · AI Chat · … strip glyphs. Now `color-mix` against opaque chip `#ffffff` / hover `#ececf1`.
- Tester next: open CPU window on macOS, warm ≥30s. Confirm strip glyphs stay solid. Gauges/sparklines still update. Activity Monitor Graphics and Media / `tauri://localhost` toward <1%.
- Next fuel after bounce: `.icon-btn` resting/hover `color: rgba(30, 30, 34, …)` on the apple-actions strip; or always-on `.apple-title h1` / Details · Top Processes body `rgba(60, 60, 67, 0.72)`.
