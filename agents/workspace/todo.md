# Coder — #14 v0.1.1442

## Plan
- [x] Discord icon: skip `is_discord_gateway_ready` on monitoring init + focus resume
- [x] Paint last-known connected state from localStorage (no IPC)
- [x] Check gateway on icon click (already) and when Settings opens
- [x] Stop hourly Discord icon poll on the common open path
- [x] Keep v0.1.1441 Settings credential wiring already in the tree
- [x] Bump to v0.1.1442; CHANGELOG; sync-dist; task notes
- [x] `cargo check` in src-tauri/
- [x] Rename WIP-14 → UNTESTED-14; commit + push; do not close #14

## Review
Shipped **v0.1.1442** for GitHub #14 (tree also includes v0.1.1441 Settings credential wiring). Discord icon skips `is_discord_gateway_ready` on monitoring warm-up and focus resume; last-known paint from localStorage. Click and Settings still check. `cargo check` passed. Task file: `UNTESTED-14-…`. Issue left open for tester / 004 (macOS Activity Monitor).
