# Coder — #14 v0.1.1446

## Plan
- [x] Skip `get_cpu_window_ui_state` on monitoring init / focus resume (localStorage SoT)
- [x] Wire monitoring sections without awaiting UI-state IPC
- [x] Agent Ops: no wait loop; one-shot `take_open_ui_section`
- [x] Collapsed Disk Cleanup: do not start glance poll on resume
- [x] Bump to v0.1.1446; CHANGELOG; sync-dist; task notes
- [x] `cargo check`
- [x] Commit + push; do not close #14

## Review
Shipped **v0.1.1446** for GitHub #14. Open path skips `get_cpu_window_ui_state`; Agent Ops capture is one invoke; collapsed Disk Cleanup skips glance IPC on resume. Issue left open for tester / 004 (macOS Activity Monitor).
