# Session todo — WIP-14 tauri://localhost CPU (#14)

GitHub WIP-14 is the lowest-numbered open GitHub task. Continue WebView idle cuts.

- [x] Defer first metrics idle ≤120s; version IPC ≤300s; monitoring/Agent Ops ≤900s
- [x] Defer sparkline unpark ≤120s after first poll
- [x] chart-line: idle-defer focus/visibility unpark (park stays immediate)
- [x] Defer history seed on focus resume (idle ≤120s)
- [x] CHANGELOG + bump `0.1.1419`
- [x] sync-dist + `cargo check` in `src-tauri/`
- [x] Rename WIP-14 → UNTESTED-14
- [x] Commit + push `origin/main`
- [x] Do **not** close GitHub #14

## Review
Shipped **v0.1.1419** #14 follow-up: metrics idle ≤120s, deferred sparkline unpark (focus ≤30s / after-poll ≤120s), deferred history seed on resume, version IPC ≤300s, monitoring/Agent Ops ≤900s. `cargo check` pass. Left as UNTESTED for macOS Activity Monitor pass. Issue #14 stays open.
