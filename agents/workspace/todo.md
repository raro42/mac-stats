# Session todo — GitHub #14 tauri://localhost CPU (v0.1.1410)

- [x] Drop init `_forceProcessUpdate`; defer first metrics idle 30s; ring skip ~95%
- [x] Chart-line: start parked, lazy init (no idle auto-boot); wider deadband
- [x] Global CSS freeze filter/box-shadow/will-change; skip AGX warm on open
- [x] Defer `get_app_version` / update check past first paint
- [x] Bump version, CHANGELOG, sync-dist, cargo check
- [x] Rename WIP → UNTESTED; commit + push origin/main

## Review
v0.1.1410 open-path cut: no forced process refresh on init, metrics/version IPC idle 30s, sparklines start parked (lazy unpark), global CSS freezes filters/shadows/will-change, ring 95%, wider deadband, no AGX warm on open. `cargo check` pass. Hand to tester; do not close #14.
