# Session todo — WIP-14 tauri://localhost CPU (#14)

GitHub WIP-14 is the lowest-numbered open GitHub task. Continue WebView idle cuts (v0.1.1420).

- [x] Defer focus-resume secondary polls + sparkline unpark (idle ≤30s)
- [x] Cancel pending idle resume on blur/pause
- [x] Clear `windowPollsPaused` on all resume paths
- [x] CHANGELOG + bump `0.1.1420`
- [x] sync-dist + `cargo check` in `src-tauri/`
- [x] Update WIP-14 implementation notes; rename → UNTESTED-14
- [x] Commit + push `origin/main` (next)
- [x] Do **not** close GitHub #14

## Review
Shipped **v0.1.1420** #14 follow-up: focus resume idle-defers secondary polls and sparkline unpark (≤30s); blur cancels pending idle resume. `cargo check` pass. Left as UNTESTED for macOS Activity Monitor pass. Issue #14 stays open.
