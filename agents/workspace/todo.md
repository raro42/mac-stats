# Coder — #14 v0.1.1461

## Plan
- [x] Flat opaque section icon chips (no glass alpha / inset / hover shadow)
- [x] Status washes mix against opaque fills (not transparent)
- [x] Bump v0.1.1461 after origin v0.1.1460; CHANGELOG; standing backlog
- [x] `cargo check` in src-tauri
- [x] Rename WIP-14 → UNTESTED; commit + push; do not close #14

## Review
Shipped **v0.1.1461** for GitHub #14 (origin already had v0.1.1460 ring-number centering). Section icon chips no longer keep a glass compositor blend on the open CPU window. Issue left open for tester / 004 (macOS Activity Monitor).
