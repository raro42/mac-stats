# WIP-24: External monitors show nothing on fresh open

## Plan
- [x] Read issue #24 + task file (empty "Nothing watching yet" until icon toggle)
- [x] Early-hide collapsed icon panes from localStorage (HTML shows monitors expanded)
- [x] Start monitoring features when a pane is already saved expanded
- [x] Resume path: hydrate monitors list immediately when expanded (not only 30s poll)
- [x] Sync dist, bump version + CHANGELOG, `cargo check`
- [x] Rename `WIP-24-…` → `UNTESTED-24-…`
- [x] Commit and push `origin/main` (do not close GitHub issue)

## Notes
GitHub: https://github.com/raro42/mac-stats/issues/24
Cause: theme HTML paints External / Monitors with empty state; `initMonitorsSection` waits for section intent (#14), so configured sites never replace first paint until the icon is toggled.
Fix: `applyIconPaneVisibilityFromLocalEarly()` + start monitoring features when any icon pane is saved expanded; focus resume calls `ensureMonitorsListHydrated()` immediately.

## Review
- v0.1.2566: Monitors empty-on-open fix (#24).
- `cargo check` PASS.
- Pushed `8a6241eb` to `origin/main`.
- Issue #24 left open for tester / 004.
