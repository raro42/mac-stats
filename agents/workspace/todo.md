# Session todo — GitHub #14 tauri://localhost CPU (follow-up #9)

- [x] Raise PROCESS_CACHE stale TTL from 5s/10s → 60s
- [x] Frontend: metrics/history 60s; process list 60s; Discord/monitors 60s; Process Details 60s; Debug Log auto-refresh 30s
- [x] Agent Ops refresh 60s; sparkline points 8; backend metric loop 20s
- [x] `cargo check` in src-tauri/
- [x] Version bump + CHANGELOG + task → UNTESTED-
- [ ] Commit and push origin/main (do not close GitHub #14)

## Review
v0.1.1394: process-cache TTL 60s was the main miss (full process enum every 5–10s while window open). Timers aligned to 60s / backend 20s / 8 sparkline points. macOS Activity Monitor remains the <1% acceptance gate.
