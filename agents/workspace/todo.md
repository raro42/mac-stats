# Session todo — WIP-14 tauri://localhost CPU (#14)

GitHub WIP-14 is the lowest-numbered open GitHub task. Continue WebView idle cuts (v0.1.1423 after v0.1.1422).

- [x] Bump first metrics idle ≤960s; sparkline unpark ≤960s; history seed ≤960s
- [x] Version IPC ≤2400s; monitoring / Agent Ops ≤7200s; late open fallback ≤2400s
- [x] Focus resume secondary polls + chart-line unpark ≤240s; focus refresh ≤240s
- [x] CHANGELOG + bump `0.1.1423`
- [x] sync-dist + `cargo check` in `src-tauri/`
- [x] Update WIP-14 implementation notes; rename → UNTESTED-14
- [x] Commit + push `origin/main`
- [x] Do **not** close GitHub #14

## Review
Shipped **v0.1.1423** #14 follow-up: defer first metrics / sparkline / history / version / monitoring / late fallback further; focus resume secondary polls and chart-line unpark idle ≤240s; focus `get_cpu_details` + metrics-interval re-arm idle-deferred ≤240s (cancelled on blur). `cargo check` pass. Left as UNTESTED for macOS Activity Monitor pass. Issue #14 stays open.
