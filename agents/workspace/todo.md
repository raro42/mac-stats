# Session todo — GitHub #14 tauri://localhost CPU (follow-up #5)

- [x] Pick WIP-14 (lowest GitHub FEAT/WIP)
- [x] WebView idle cuts: slower polls, backend cadence, CSS contain, chart points
- [x] Fix harness_ops `contains("age")` false positives (tester gate)
- [x] `cargo check` in src-tauri/
- [x] Version bump + CHANGELOG + task → UNTESTED-
- [x] Commit and push origin/main (do not close GitHub #14)

## Review
v0.1.1390: further cut open-window work (15s polls, 5s backend loop when visible, CSS contain, fewer sparkline points) and fix age-token matcher false positives so `cargo test` harness_ops passes on Linux. macOS Activity Monitor remains the <1% acceptance gate.
