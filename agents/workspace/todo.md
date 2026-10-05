# Session todo — GitHub #14 tauri://localhost CPU (follow-up #6)

- [x] Pick WIP-14 (lowest GitHub FEAT/WIP)
- [x] Flatten Light shell gradients; slower polls; fewer sparkline points; process-details 15s
- [x] Isolate rust-native digest test from shared latest.json race
- [x] `cargo check` in src-tauri/
- [x] Version bump + CHANGELOG + task → UNTESTED-
- [ ] Commit and push origin/main (do not close GitHub #14)

## Review
v0.1.1391: further cut open-window work (20s polls, 8s backend loop, 24 sparkline points, Light/Dark compositor flatten, process-details 15s) and isolate native digest test from shared latest.json races. macOS Activity Monitor remains the <1% acceptance gate.
