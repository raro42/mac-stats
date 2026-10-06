# Session todo — WIP-14 tauri://localhost CPU (#14)

GitHub WIP-14 is the lowest-numbered open GitHub task. Continue WebView idle cuts (v0.1.1422 after v0.1.1421).

- [x] Bump first metrics idle ≤480s; sparkline unpark ≤480s; history seed ≤480s
- [x] Version IPC ≤1200s; monitoring / Agent Ops ≤3600s; late open fallback ≤1200s
- [x] Focus resume secondary polls + chart-line unpark ≤120s; focus refresh ≤120s
- [x] Idle-defer `startRefresh()` on focus (arm in deferred resume / focus refresh, not on focus event)
- [x] CHANGELOG + bump `0.1.1422`
- [x] sync-dist + `cargo check` in `src-tauri/`
- [x] Update WIP-14 implementation notes; rename → UNTESTED-14
- [ ] Commit + push `origin/main`
- [x] Do **not** close GitHub #14

## Review
Shipped **v0.1.1422** #14 follow-up: defer first metrics / sparkline / history / version / monitoring / late fallback further; focus resume secondary polls and chart-line unpark idle ≤120s; focus `get_cpu_details` + metrics-interval re-arm idle-deferred ≤120s (cancelled on blur). `cargo check` pending/pass. Left as UNTESTED for macOS Activity Monitor pass. Issue #14 stays open.
