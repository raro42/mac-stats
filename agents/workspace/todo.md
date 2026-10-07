# UNTESTED-14 / #14 — tauri://localhost CPU (v0.1.1722 apple-title)

## Plan
- [x] Pick fuel: standing P2 / GitHub #14 after v0.1.1721 (icon-btn type)
- [x] Opaque shell mix for `.apple-title h1` `color` (always-visible product title)
- [x] Bump to v0.1.1722 + CHANGELOG + standing_backlog
- [x] `cargo check` / ratchet verify
- [x] Rename WIP-14 → UNTESTED-14
- [x] Commit + push `origin/main`
- [x] Leave GitHub issue alone (do not close)

## Review
- Coder: `.apple-title h1` was `rgba(28, 28, 30, 0.92)` glass over the shell. Now `color-mix(in srgb, #1c1c1e 92%, #f2f2f6)` against the opaque body fill.
- Tester next: open CPU window on macOS, warm ≥30s. Confirm title stays solid. Gauges/sparklines still update. Activity Monitor Graphics and Media / `tauri://localhost` toward <1%.
- Next fuel after bounce: Details · Top Processes body `rgba(60, 60, 67, 0.72)`; or `.battery-icon` `rgba(12, 12, 16, 0.50)`.
