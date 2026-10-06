# WIP-14 tauri://localhost CPU — v0.1.1426

## Plan
- [x] Drop pending DOM rAF batches on blur/pause; re-check occlusion when rAF fires
- [x] Skip mid-flight version tip/update DOM when occluded after fetch
- [x] Skip fetchAppVersion DOM writes while occluded (still cache version)
- [x] Abort history-seed loop / skip seed DOM when occluded mid-flight
- [x] Bump Cargo.toml → 0.1.1426
- [x] sync-dist.sh
- [x] cargo check in src-tauri/
- [x] Update task file + rename WIP → UNTESTED
- [x] Commit + push origin/main
- [x] Do not close GitHub #14

## Review
Mid-flight occlusion cancel after v0.1.1425 idle-handle cancel. Blur clears queued DOM rAF; version IPC and history seed skip paint when parked. `cargo check` pass. Pushed to origin/main.
