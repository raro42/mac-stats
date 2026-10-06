# Coder — #14 v0.1.1440

## Plan
- [x] Collapsed Debug Log: skip `read_debug_log` glance poll on monitoring init + focus resume
- [x] Start/stop glance poll on expand/collapse (mirror Perplexity)
- [x] Harden `startLogsGlancePoll` / `pollLogsGlanceCounts` for collapsed + park
- [x] `ensureLogsSectionExpanded` arms glance poll
- [x] Bump to v0.1.1440; CHANGELOG; sync-dist; task notes
- [x] `cargo check`
- [x] Rename WIP-14 → UNTESTED-14; commit + push; do not close #14

## Review
Shipped **v0.1.1440** for GitHub #14 (v0.1.1439 was taken by idle-thought timeout log). Collapsed Debug Log skips `read_debug_log` glance IPC on monitoring warm-up and focus resume; expand arms the poll. Task file: `UNTESTED-14-…`. Issue left open for tester / 004 (macOS Activity Monitor).
