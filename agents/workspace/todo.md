# Coder — #14 v0.1.1462

## Plan
- [x] Opaque LPM toggle track (no glass alpha / inset / knob shadow)
- [x] On-state mixes against opaque fills
- [x] Bump v0.1.1462 after origin v0.1.1461; CHANGELOG; standing backlog
- [x] `cargo check` in src-tauri
- [x] Rename WIP-14 → UNTESTED; commit + push; do not close #14

## Review
Shipped **v0.1.1462** for GitHub #14 (origin already had v0.1.1461 section-icon glass). Low Power Mode toggle no longer keeps a glass compositor blend on the open CPU window. Issue left open for tester / 004 (macOS Activity Monitor).
