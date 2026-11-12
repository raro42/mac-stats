# FEAT-26: External monitors empty flash on fresh open

## Plan
- [x] Read issue #26 + WIP task + lessons
- [x] Persist last monitor rows to localStorage on successful `loadMonitors`
- [x] Seed list from local cache synchronously when Monitors is expanded (before IPC)
- [x] Call seed on early open path (`applyIconPaneVisibilityFromLocalEarly` / hydrate)
- [x] Bump version + CHANGELOG, sync `src-tauri/dist/cpu.js`, `cargo check`
- [x] Rename `WIP-26-…` → `UNTESTED-26-…`
- [x] Commit and push `origin/main` (do not close GitHub issue)

## Notes
GitHub: https://github.com/raro42/mac-stats/issues/26
Cause: theme HTML ships "Nothing watching yet"; #24 starts hydrate when expanded, but list still waits on history + `list_monitor_statuses` IPC, so configured sites flash empty for a few seconds.
Fix: last-known rows in localStorage (icon-wash parity); paint list IPC before history ticks.

## Review
- v0.1.2641: local seed + list-before-history (#26).
- Handoff: tester / 004 (issue left open).
