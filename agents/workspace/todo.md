# Session todo — GitHub #14 tauri://localhost CPU (v0.1.1409)

- [x] Defer first `get_cpu_details` past first paint; drop init history seed
- [x] Ring skip ~85%; chart-line boot idle 360s; AGX warm 3600s
- [x] Global CSS transition/animation freeze while window open
- [x] Bump version, CHANGELOG, sync-dist, cargo check
- [x] Rename WIP → UNTESTED; commit + push origin/main

## Review
v0.1.1409 open-path cut: idle-deferred first metrics IPC, no init history seed, global CSS transition freeze, ring 85%, sparkline deadband wider, boot idle 360s, AGX warm 3600s, Linux warm cache parity. `cargo check` pass. Hand to tester; do not close #14.
