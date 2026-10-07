# UNTESTED-14 / #14 — tauri://localhost CPU (v0.1.1721 icon-btn type)

## Plan
- [x] Pick fuel: standing P2 / GitHub #14 after v0.1.1720 (icon-line type)
- [x] Opaque shell mix for `.icon-btn` resting · hover `color` (always-visible Monitors · AI Chat strip icons)
- [x] Bump to v0.1.1721 + CHANGELOG + standing_backlog
- [x] `cargo check` in `src-tauri/`
- [x] Rename WIP-14 → UNTESTED-14
- [x] Commit + push `origin/main`
- [x] Leave GitHub issue alone (do not close)

## Review
- Coder: `.icon-btn` resting/hover were `rgba(30, 30, 34, …)` glass under the always-visible Monitors · AI Chat · … strip icons (over opaque `#ececf1` chip). Now `color-mix` against opaque chip `#ececf1` / hover `#ffffff`.
- Tester next: open CPU window on macOS, warm ≥30s. Confirm strip icons stay solid. Gauges/sparklines still update. Activity Monitor Graphics and Media / `tauri://localhost` toward <1%.
- Next fuel after bounce: always-on `.apple-title h1` `rgba(28, 28, 30, 0.92)`; or Details · Top Processes body `rgba(60, 60, 67, 0.72)`.
