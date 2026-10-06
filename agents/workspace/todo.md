# Session todo — GitHub #14 tauri://localhost CPU (v0.1.1413)

- [x] Rename UNTESTED→WIP
- [x] Gate first metrics + sparkline GPU on focus (no 120s idle wake)
- [x] Guard init against double-arm; slow waitForTauri poll
- [x] Bump 0.1.1413, CHANGELOG, sync-dist
- [x] cargo check; rename WIP→UNTESTED
- [ ] commit + push origin/main

## Review
Focus-gated first metrics; no idle sparkline unpark; slower waitForTauri. `cargo check` pass.
