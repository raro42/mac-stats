# Coder — #14 v0.1.1455

## Plan
- [x] Skip chart-line / history / poster canvas bind+park on open (GPU until hover)
- [x] Hide history chart containers until `is-history-gpu-unparked`
- [x] Late-open fallback: real timer when occluded, not rIC deadline
- [x] Bump v0.1.1455; CHANGELOG; sync-dist
- [x] `cargo check` in src-tauri
- [ ] Rename WIP-14 → UNTESTED; commit + push; do not close #14

## Review
Shipped **v0.1.1455** for GitHub #14. Open no longer binds sparkline/poster canvases or sets `canvas.width`. History containers stay out of the compositor until hover or Refresh. Late-open fallback no longer uses idle-callback (that fired immediately). Issue left open for tester / 004 (macOS Activity Monitor).
