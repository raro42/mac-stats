# UNTESTED-15 / #15 — restore live CPU-window metrics after #14 idle ratchet

## Plan
- [x] Fix park gate: do not treat `!document.hasFocus()` as occluded (blur/Focused already park)
- [x] Restore focused-window poll cadences (cpu.js / history.js / agent-ops.js)
- [x] Restore PROCESS_CACHE_TTL + get_cpu_details rate floor in metrics
- [x] Arm metrics quickly on open/focus; fix armed-without-interval
- [x] Restore ring skip ~5% and history points
- [x] Soft-fix data-poster light bleeds (monitors summary / AI glance / LPM / chat input)
- [x] Sync dist, bump v0.1.1762, CHANGELOG, standing_backlog
- [x] `cargo check` in src-tauri/
- [x] Rename WIP-15 → UNTESTED-15; commit + push origin/main
- [ ] Leave GitHub #15 open (tester / 004)

## Review
- Coder: Overnight #14 ratchets left focused window at first-paint "None yet" (hasFocus false-positive, 1h polls, 99% ring skip, 600s backend floor). Restored ~2s focused polls, fast arm, sparkline unpark after first paint, park-on-blur only. Data Poster dark washes for Monitors summary + AI Chat glance.
- Tester next: open CPU window on macOS, warm ≥30s. Confirm rings/strip/Details/sparklines leave "None yet". Data Poster: no white/cream Monitors or AI Chat bars. Do not close #15.
