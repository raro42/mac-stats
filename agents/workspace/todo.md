# Session todo — WIP-14 tauri://localhost CPU (#14)

GitHub WIP-14 is the lowest-numbered open GitHub task. Continue WebView idle cuts.

- [x] Defer first metrics idle ≤60s; wire DOM with metrics arm
- [x] Defer sparkline unpark ≤60s after first poll
- [x] Focus-gate Agent Ops (no DOMContentLoaded); monitoring/Agent Ops idle ≤600s
- [x] Defer version IPC on focus resume ≤120s
- [x] CHANGELOG + bump `0.1.1418`
- [x] `cargo check` in `src-tauri/`
- [x] Rename WIP-14 → UNTESTED-14
- [x] Commit + push `origin/main`
- [x] Do **not** close GitHub #14

## Review
Shipped **v0.1.1418** #14 follow-up: metrics idle ≤60s, DOM wire with metrics, deferred sparkline unpark, focus-gated Agent Ops (600s), deferred version IPC on resume. `cargo check` pass. Left as UNTESTED for macOS Activity Monitor pass. Issue #14 stays open.
