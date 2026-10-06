# Coder — #14 v0.1.1447

## Plan
- [x] Bake MAC_STATS_OPEN_SECTION / openUiSection into cpu.html?open= at window create
- [x] Agent Ops: read URL `open`; skip take_open_ui_section invoke
- [x] Unit tests for sanitize + URL builder
- [x] Bump to v0.1.1447; CHANGELOG; sync-dist
- [x] `cargo check` + config tests
- [x] Commit + push; do not close #14

## Review
Shipped **v0.1.1447** for GitHub #14. Common open path no longer does take_open_ui_section IPC. Capture still works via window URL. Issue left open for tester / 004 (macOS Activity Monitor).
