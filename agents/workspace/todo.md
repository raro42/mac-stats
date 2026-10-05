# Session todo — GitHub #14 tauri://localhost CPU (follow-up #7)

- [x] Cut hot-path IPC: tauri-logger warn/error only; drop CPU Power console.log spam
- [x] Slower open-window work: metrics/history 30s, 16 sparkline points, process details 30s, backend 12s
- [x] Apple/Light/Dark CSS: drop expensive icon `img` filter chains
- [x] Fix Linux unused `cpu_window_visible` in metric loop
- [x] `cargo check` in src-tauri/
- [x] Version bump + CHANGELOG + task → UNTESTED-
- [x] Commit and push origin/main (do not close GitHub #14)

## Review
v0.1.1392: cut open-window IPC and compositor work (30s polls, 12s backend loop, 16 sparkline points, warn/error-only tauri-logger, opacity-only icon imgs). macOS Activity Monitor remains the <1% acceptance gate.
