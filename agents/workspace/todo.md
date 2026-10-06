# Coder — #14 v0.1.1479

## Plan
- [x] Opaque AI Chat errors glance washes
- [x] Mix against opaque fill; drop hover / focus box-shadow
- [x] Bump v0.1.1479; CHANGELOG
- [x] `cargo check` in src-tauri; sync dist
- [x] Rename WIP-14 → UNTESTED; commit + push; do not close #14

## Review
Shipped **v0.1.1479** for GitHub #14 (after origin v0.1.1478 last-answer glance cut). AI Chat errors glance washes no longer keep a glass compositor blend. Issue left open for tester / 004 (macOS Activity Monitor). Linux webkit2gtk still has a host floor on a blank page. `cargo check` in src-tauri finished with warnings only.
