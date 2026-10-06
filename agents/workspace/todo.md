# Coder — #14 v0.1.1432

## Plan
- [x] Park Settings credential status refreshes (Brave…Slack) mid-flight
- [x] Park Agent Ops digest refresh mid-flight; skip flash while away
- [x] Buffer AI Chat stream chunks while parked; flush on resume
- [x] Bump to v0.1.1432; CHANGELOG; sync-dist
- [x] `cargo check` in src-tauri/
- [x] Rename WIP-14 → UNTESTED-14; commit + push; do not close #14

## Review
Shipped **v0.1.1432** for GitHub #14. Settings status / digest / chat-stream park while occluded; resume flushes stream buffer. Task file: `UNTESTED-14-…`. Issue left open for tester / 004 (macOS Activity Monitor).
