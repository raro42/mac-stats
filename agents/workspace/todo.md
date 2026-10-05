# Session todo — GitHub #14 tauri://localhost CPU (v0.1.1411)

- [x] Defer first metrics/version IPC idle timeout to 120s; defer DOM wiring
- [x] Ring paint skip ~99%; skip updateRingHotStates when signature unchanged
- [x] Chart-line: no first-sample unpark; late idle unpark 120s; wider deadband
- [x] CSS: freeze mix-blend-mode; transform freeze on heavy trees; no ensureGpuHistoryChart init unpark
- [x] Bump version, CHANGELOG, sync-dist, cargo check
- [x] Rename WIP → UNTESTED; commit + push origin/main

## Review
v0.1.1411 open-path cut: metrics/version IPC idle 120s, DOM wiring idle 60s, sparklines stay parked through open (buffer + late 120s/focus unpark), mix-blend-mode freeze + transform freeze on metric/ring/history trees, ring 99%, hot-state signature skip, wider deadband. `cargo check` pass. Hand to tester; do not close #14.
