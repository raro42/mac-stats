# WIP-14 tauri://localhost CPU — v0.1.1427

## Plan
- [x] cancelAnimationFrame for pending DOM rAF on blur/pause
- [x] Mid-flight skip: Discord icon, logs glance/viewer, history availability
- [x] Mid-flight skip: monitors summary/load loops, disk cleanup panel, update banner
- [x] Pause process-details refresh interval on blur; resume if modal still open
- [x] Agent Ops refresh: skip paint when occluded mid-flight
- [x] Bump Cargo.toml → 0.1.1427
- [x] sync-dist.sh
- [x] cargo check in src-tauri/
- [x] Update task file + rename WIP → UNTESTED
- [x] Commit + push origin/main
- [x] Do not close GitHub #14

## Review
Mid-flight secondary IPC cancel after v0.1.1426. Shared `windowWorkPaused` gate; rAF cancel on blur; Discord/monitors/history/logs/disk/update banner/Process Details/Agent Ops skip paint when parked. `cargo check` pass. Pushed `54bc357a` to origin/main. Issue #14 left open for tester/004.
