# FEAT/WIP-14 — tauri://localhost CPU (coder)

## Plan
- [x] Pick lowest GitHub FEAT/WIP (`WIP-14`)
- [x] Opaque Apple `.icon-btn` hover / focus-visible / active washes (always-visible icon strip)
- [x] Bump to v0.1.1640; CHANGELOG; standing_backlog; WIP Implementation notes
- [x] `cargo check` in `src-tauri/`
- [x] Commit + push `origin/main`
- [x] Rename `WIP-14-…` → `UNTESTED-14-…`; comment via `gh-safe.sh` (do not close #14)

## Review
- v0.1.1640: Apple `.icon-btn` hover · focus-visible · active mix against `#ffffff` (no glass alpha).
- `cargo check` clean (pre-existing unused import warning in `feature_health.rs` only).
- Pushed to `origin/main`; issue #14 left open for tester / 004.
