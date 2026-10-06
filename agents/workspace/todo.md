# Session todo — GitHub #14 tauri://localhost CPU (v0.1.1415)

- [x] Defer first metrics on focus/init: 5s → idle ≤30s (no immediate startCpuWindowMetricsOnce)
- [x] Stop `_forceProcessUpdate` on every focus resume (alt-tab process rebuild)
- [x] Do not schedule monitoring from init when focused (resume / late only)
- [x] Monitoring + Agent Ops idle timeout 120s → 300s
- [x] Defer version/update IPC idle ≤120s after metrics arm
- [x] Bump 0.1.1415, CHANGELOG, sync-dist
- [x] cargo check; rename WIP→UNTESTED
- [x] commit + push origin/main

## Review
Shipped v0.1.1415. Task at `UNTESTED-14-…`. Issue #14 left open.
