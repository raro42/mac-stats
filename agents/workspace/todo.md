# Coder — #14 v0.1.1459

## Plan
- [x] LPM knob uses left offset, not translate (cpu.js + theme cpu.css)
- [x] Bump v0.1.1459; CHANGELOG; standing backlog
- [x] `cargo check` in src-tauri
- [x] Rename WIP-14 → UNTESTED; commit + push; do not close #14

## Review
Shipped **v0.1.1459** for GitHub #14. Low Power Mode knob no longer keeps a transform compositor layer on the open CPU window (including while LPM is on). Issue left open for tester / 004 (macOS Activity Monitor).
