# Coder — #14 v0.1.1477

## Plan
- [x] Opaque AI Chat turn glance washes
- [x] Mix against opaque fill; drop hover / focus box-shadow
- [x] Bump v0.1.1477; CHANGELOG
- [x] `cargo check` in src-tauri; sync dist
- [x] Rename WIP-14 → UNTESTED; commit + push; do not close #14

## Review
Shipped **v0.1.1477** for GitHub #14 (after origin v0.1.1476 hover lift cut). AI Chat turn glance washes no longer keep a glass compositor blend. Issue left open for tester / 004 (macOS Activity Monitor). Linux webkit2gtk still has a host floor on a blank page. `cargo check` in src-tauri finished with warnings only.
