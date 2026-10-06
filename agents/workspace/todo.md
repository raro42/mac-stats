# Session todo — WIP-14 tauri://localhost CPU (#14)

GitHub WIP-14 is the lowest-numbered open GitHub task. Continue WebView idle cuts (v0.1.1421 after v0.1.1420).

- [x] Bump first metrics idle ≤240s; sparkline unpark ≤240s; history seed ≤240s
- [x] Version IPC ≤600s; monitoring / Agent Ops ≤1800s
- [x] Focus resume secondary polls + chart-line unpark ≤60s
- [x] Idle-defer focus `refresh()` (≤60s); cancel on blur/pause
- [x] CHANGELOG + bump `0.1.1421`
- [x] sync-dist + `cargo check` in `src-tauri/`
- [x] Update WIP-14 implementation notes; rename → UNTESTED-14
- [x] Commit + push `origin/main`
- [x] Do **not** close GitHub #14

## Review
Shipped **v0.1.1421** #14 follow-up: defer first metrics / sparkline / history / version / monitoring further; focus resume secondary polls and chart-line unpark idle ≤60s; stale focus `get_cpu_details` idle-deferred ≤60s and cancelled on blur. `cargo check` pass. Left as UNTESTED for macOS Activity Monitor pass. Issue #14 stays open.
