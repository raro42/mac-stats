# WIP-23: Widgets with white background (Data Poster power strip)

## Plan
- [x] Read issue #23 + screenshot (battery / power / LPM / remaining pills)
- [x] Remap `#ececf1` power-strip washes to `--panel` in `ensureRamStripStyles()`
- [x] Sync `src/cpu.js` / `agent-ops.css` → `src-tauri/dist/`, bump version + CHANGELOG
- [x] `cargo check` in `src-tauri/`
- [x] Rename `WIP-23-…` → `UNTESTED-23-…`
- [x] Commit and push `origin/main` (do not close GitHub issue)

## Notes
GitHub: https://github.com/raro42/mac-stats/issues/23
Cause: shared injected strip styles mixed status washes into Apple `#ececf1` (reads as white on Data Poster).
Fix: mix into `var(--panel, #ececf1)` so dark themes stay dark.

## Review
- v0.1.2564: power-strip + flash rings use `--panel`.
- `cargo check` PASS.
- Issue #23 left open for tester / 004.
