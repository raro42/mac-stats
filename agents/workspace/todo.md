# Coder — #14 v0.1.1474

## Plan
- [x] Opaque history sparkline washes (CPU · GPU · FREQ · TEMP)
- [x] Mix against opaque fill; drop ring / flash box-shadow
- [x] Bump v0.1.1474; CHANGELOG
- [x] `cargo check` in src-tauri; sync dist
- [x] Rename WIP-14 → UNTESTED; commit + push; do not close #14

## Review
Shipped **v0.1.1474** for GitHub #14 (after v0.1.1473 AI Chat glance). Sparkline hot / calm / Fair washes no longer keep a glass compositor blend on the open CPU window. Issue left open for tester / 004 (macOS Activity Monitor). Linux webkit2gtk still has a host floor on a blank page.
