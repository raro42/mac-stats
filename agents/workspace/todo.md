# Session todo — GitHub #14 tauri://localhost CPU

- [x] Pick lowest GitHub task (WIP-14 / issue 14)
- [x] Kill live backdrop-filter + infinite hot-ring CSS (WebKit compositor)
- [x] Ring gauges: no 60fps rAF throttle loop
- [x] Pause metrics poll when the window is hidden
- [x] Sparklines: skip idle redraws, cap canvas DPR
- [x] cargo check in src-tauri/
- [x] CHANGELOG + version bump
- [x] Rename WIP-14 → UNTESTED-14
- [ ] Commit and push origin/main (do not close the GitHub issue)

## Review

WKWebView `tauri://localhost` / Graphics and Media stayed hot while the CPU window was open. Opaque window still ran live `backdrop-filter`. Hot rings (GPU ≥ 15%) ran infinite `filter` animations. Ring tween kept `requestAnimationFrame` at display refresh. `cargo check` pass (v0.1.1356).
