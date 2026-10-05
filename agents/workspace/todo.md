# Session todo — GitHub #14 tauri://localhost CPU (follow-up #11)

- [x] Align UI polls/TTL to 120s; backend metric loop 45s; sparkline 4 points
- [x] Raise `get_cpu_details` floor 30s → 45s; TEMP 45s / cache 60s
- [x] History seed skip on focus if recent; HISTORY_POINTS 16; seed maxDisplayPoints 8
- [x] Pause Debug Log auto-refresh on blur; auto-refresh interval 120s
- [x] Defer GPU-sampler warm 5s; Agent Ops updated-ago 120s
- [x] `cargo check` in src-tauri/; bump v0.1.1397 + CHANGELOG; sync-dist
- [x] Rename WIP-14 → UNTESTED-14; commit + push; gh-safe comment (do not close #14)

## Review
v0.1.1397: 120s UI polls, 45s backend/`get_cpu_details` floor, 4 sparkline points, Debug Log auto-refresh pauses on blur, history seed skip on rapid focus, GPU warm deferred 5s. macOS Activity Monitor remains the <1% acceptance gate.
