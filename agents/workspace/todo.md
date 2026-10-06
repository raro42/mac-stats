# Coder — #14 v0.1.1429

## Plan
- [x] Align leftover `document.hidden`-only timers with shared pause (`windowWorkPaused` / `agentOpsWorkPaused`)
- [x] Mid-flight: skip DOM after parked awaits (disk glance, logs catch, monitors catch/height)
- [x] `updateRingGauge` uses `windowWorkPaused` (match chart-line)
- [x] Agent Ops: abort remaining IPC batches when parked mid-flight
- [x] Bump to v0.1.1429; CHANGELOG; sync-dist
- [x] `cargo check` in src-tauri/
- [x] Rename WIP-14 → UNTESTED-14; commit + push; do not close #14

## Review
Shipped **v0.1.1429** (`717efa1e`) for GitHub #14. Shared-pause holdouts, mid-flight DOM skips, Agent Ops batched IPC abort. Task file: `UNTESTED-14-…`. Issue left open for tester / 004 (macOS Activity Monitor).
