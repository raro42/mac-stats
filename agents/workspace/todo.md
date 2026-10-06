# Coder — #14 v0.1.1438

## Plan
- [x] Add `list_monitor_statuses` Tauri command (one IPC: id/name/url + cached status)
- [x] `updateMonitorsSummary` / `loadMonitors` / settings list use bulk statuses
- [x] Defer 24h history availability probe until sparkline unpark / seed
- [x] Bump to v0.1.1438; CHANGELOG; sync-dist; task notes
- [x] `cargo check`
- [x] Rename WIP-14 → UNTESTED-14; commit + push; do not close #14

## Review
Shipped **v0.1.1438** for GitHub #14. One `list_monitor_statuses` IPC for Monitors summary/list/settings; 24h history probe waits for sparkline unpark/seed. Task file: `UNTESTED-14-…`. Issue left open for tester / 004 (macOS Activity Monitor).
