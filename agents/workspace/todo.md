# Coder — #14 v0.1.1464

## Plan
- [x] Opaque battery / power / LPM / time-remaining status washes (no glass alpha)
- [x] Mix against opaque strip fill; drop ring box-shadow
- [x] Bump v0.1.1464 after origin v0.1.1463; CHANGELOG; standing backlog
- [x] `cargo check` in src-tauri
- [x] Rename WIP-14 → UNTESTED; commit + push; do not close #14

## Review
Shipped **v0.1.1464** for GitHub #14 (origin already had v0.1.1463 settings-knob `left`). Battery, power, LPM, and time-remaining status washes no longer keep a glass compositor blend on the open CPU window. Issue left open for tester / 004 (macOS Activity Monitor).
