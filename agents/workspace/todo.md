# Coder — #14 v0.1.1434

## Plan
- [x] Defer Settings credential status IPC (Brave…Signal) from open to Settings open
- [x] Defer decorations preference load to Settings open; resume if Settings stays open
- [x] Skip collapsed Perplexity key-status IPC; drop version MutationObserver
- [x] Park Process Details open + Settings Monitors list mid-flight; resume list rebuild
- [x] Bump to v0.1.1434; CHANGELOG; sync-dist
- [x] `cargo check` / ratchet verify
- [x] Rename WIP-14 → UNTESTED-14; commit + push; do not close #14

## Review
Shipped **v0.1.1434** for GitHub #14. Settings credential/decorations deferred to Settings open; Process Details open + Settings Monitors list park mid-flight; resume rebuilds Monitors list. Task file: `UNTESTED-14-…`. Issue left open for tester / 004 (macOS Activity Monitor).
