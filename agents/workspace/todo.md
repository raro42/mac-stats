# WIP-22: Hover layout broken (white metric copy wash)

## Plan
- [x] Read issue #22 + locate click-to-copy hover styles
- [x] Replace white `#ffffff` hover/Copied wash with accent glow on `--panel`
- [x] Sync `src/cpu.js` → `src-tauri/dist/`, bump version + CHANGELOG
- [ ] `cargo check` in `src-tauri/`
- [x] Rename `WIP-22-…` → `UNTESTED-22-…`
- [ ] Commit and push `origin/main` (do not close GitHub issue)

## Notes
GitHub: https://github.com/raro42/mac-stats/issues/22
Screenshot: GPU / Freq / Temp on Data Poster — hover paints a white slab over the value ("Click to copy").
Cause: `ensureMetricValueCopyStyles()` mixes accent into `#ffffff`.

## Review
- v0.1.2561: hover / focus / Copied mix into `--panel` / `--bg` + flat 1px accent ring.
- Issue #22 left open for tester / 004.
