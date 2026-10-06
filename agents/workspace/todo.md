# Coder — #14 v0.1.1454

## Plan
- [x] Stop rIC open-path monitoring / Agent Ops (fires as soon as idle)
- [x] Wire section chrome on pointer/keyboard intent; capture `?open=` still hydrates now
- [x] Defer ring/header keyboard + GPU chart inject until Tab/focus or history unpark
- [x] Skip version/GitHub IPC until footer version click
- [x] Bump to v0.1.1454; CHANGELOG; sync-dist
- [x] `cargo check` in src-tauri
- [ ] Rename WIP-14 → UNTESTED; commit + push; do not close #14

## Review
Shipped **v0.1.1454** for GitHub #14. Collapsed section wiring no longer runs on idle callback. Issue left open for tester / 004 (macOS Activity Monitor).
