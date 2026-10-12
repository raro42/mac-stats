# WIP-18 — Data Poster theme white-on-white (#18)

## Plan
- [x] Identify Top Processes white slabs (summary glances + pinned row) from issue screenshot
- [x] Remap Apple `#ffffff` washes onto `#0e0e14` for data-poster (and dark if shared)
- [x] Sync dist if editing `src/`; bump patch + CHANGELOG
- [x] `cargo check` in `src-tauri/`
- [ ] Commit + push `origin/main`
- [x] Rename `WIP-18-…` → `UNTESTED-18-…` (do not close GitHub issue)

## Review
- Cause: shared `agent-ops.css` Top Processes glances / pinned rows / filter chips mixed against `#ffffff` (Apple opaque washes).
- Fix: dark remaps for Data Poster + Dark onto `#0e0e14` (v0.1.1895). Layout daily helper now tracks those selectors.
- `cargo check` OK (warnings only, unrelated Linux stubs).
