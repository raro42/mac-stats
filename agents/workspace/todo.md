# Coder — #14 v0.1.1451

## Plan
- [x] Bind+hide sparkline canvases on boot (empty map skipped park)
- [x] Keep park through window resize / geometry restore
- [x] Defer Discord Settings wiring, version DOM walk, collapsed Ollama 250ms timer
- [x] Bump to v0.1.1451; CHANGELOG; sync-dist
- [x] `cargo check` in src-tauri
- [x] Rename WIP-14 → UNTESTED; commit + push; do not close #14

## Review
Shipped **v0.1.1451** for GitHub #14. Open-path resize no longer allocates sparkline GPU. Issue left open for tester / 004 (macOS Activity Monitor).
