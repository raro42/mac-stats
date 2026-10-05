# Session todo — GitHub #14 tauri://localhost CPU (follow-up #10)

- [x] Align polls/TTL to 90s; backend metric loop 30s; sparkline 6 points
- [x] Raise `get_cpu_details` rate limit 2s → 30s; Debug Log auto-refresh 60s
- [x] History availability poll 5m; pause data-poster history on blur
- [x] Defer GPU-sampler warm off the open path; TEMP read interval 30s
- [x] `cargo check` in src-tauri/; bump v0.1.1396 + CHANGELOG
- [x] Rename WIP-14 → UNTESTED-14; commit + push; gh-safe comment (do not close #14)

## Review
v0.1.1396: 90s UI polls, 30s backend/`get_cpu_details` floor, 6 sparkline points, deferred GPU warm, history blur pause, 5m history-availability probe. macOS Activity Monitor remains the <1% acceptance gate.
