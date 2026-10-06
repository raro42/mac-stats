# Coder — #14 v0.1.1448

## Plan
- [x] Defer Agent Ops `setupAgentOps` until expand / capture open
- [x] Skip attention-glance DOM create while collapsed
- [x] Skip `set_cpu_window_ui_state` when section state is unchanged
- [x] Bump to v0.1.1448; CHANGELOG; sync-dist
- [x] `cargo check` in src-tauri
- [x] Rename WIP-14 → UNTESTED; commit + push; do not close #14

## Review
Shipped **v0.1.1448** for GitHub #14. Collapsed Agent Ops no longer wires filters/keyboard on monitoring idle. Unchanged collapse restore skips persist IPC. Issue left open for tester / 004 (macOS Activity Monitor).
