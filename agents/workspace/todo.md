# Coder — #14 v0.1.1473

## Plan
- [x] Opaque AI Chat collapsed glance (keep-header)
- [x] Mix against opaque fill; drop hover / focus box-shadow
- [x] Bump v0.1.1473; CHANGELOG
- [x] `cargo check` in src-tauri; sync dist
- [x] Rename WIP-14 → UNTESTED; commit + push; do not close #14

## Review
Shipped **v0.1.1473** for GitHub #14 (after v0.1.1472 press/hover transform cut). AI Chat collapsed glance online / offline / active / error washes no longer keep a glass compositor blend on the open CPU window. Issue left open for tester / 004 (macOS Activity Monitor). Linux webkit2gtk still has a host floor on a blank page.
