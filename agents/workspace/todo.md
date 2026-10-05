# Session todo — GitHub #14 tauri://localhost CPU (follow-up #3)

- [x] Pick lowest GitHub task (WIP-14 / issue 14)
- [x] macOS: `.transparent(false)` on CPU window (Linux already has it)
- [x] Pause monitors / disk-cleanup glance / agent-ops timers on `document.hidden`
- [x] Opaque sparklines (no canvas alpha)
- [x] Kill dark-theme hover infinite `glow-rotate`
- [x] Metrics poll 5s → 8s; history.js 2s → 8s
- [x] Disconnect changelog MutationObserver after wire-up
- [x] Sync dist, bump version, CHANGELOG
- [x] `cargo check` in src-tauri/
- [x] Rough CPU sample with `--cpu` on this host (WebKit still ~99% on Linux blank-floor host)
- [x] Rename WIP-14 → UNTESTED-14
- [ ] Commit and push origin/main (do not close GitHub #14)

## Review

v0.1.1388 cuts remaining idle timers and compositor blends for the macOS `tauri://localhost` / Graphics and Media path. Linux webkit2gtk still shows ~99% WebKitWebProcess after warm (same host floor as prior blank-page A/B); macOS Activity Monitor remains the acceptance gate.
