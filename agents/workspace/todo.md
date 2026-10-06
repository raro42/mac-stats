# WIP-14 tauri://localhost CPU — v0.1.1425

## Plan
- [x] Pick lowest GitHub FEAT/WIP: WIP-14
- [x] Structural cancel: pending version IPC, after-first sparkline unpark, history seed, monitoring features, Agent Ops init on blur/pause
- [x] Bail version / monitoring / Agent Ops start when occluded (do not arm flags)
- [x] Bump Cargo.toml → 0.1.1425
- [x] sync-dist.sh
- [x] cargo check in src-tauri/
- [x] Update task file + rename WIP → UNTESTED
- [x] Commit + push origin/main (`90b7af80`)
- [x] Do not close GitHub #14

## Review
Structural occlusion cancel for remaining untracked idles after v0.1.1424. Blur drops version IPC, after-first unpark, history seed, monitoring, and Agent Ops init. Focus re-schedules. `cargo check` pass. Pushed to origin/main.
