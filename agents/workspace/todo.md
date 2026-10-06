# WIP-14 — next compositor cut (v0.1.1576)

## Plan
- [x] Pick lowest GitHub FEAT/WIP: `WIP-14` (tauri://localhost CPU)
- [x] Next cut after v0.1.1575: Monitors row Copied flash (`.monitor-item.is-just-copied`) still mixed against transparent
- [x] Add opaque green wash (`#ffffff` mix), opaque border, `box-shadow: none`
- [x] Sync `src/agent-ops.css` → `src-tauri/dist/agent-ops.css`
- [x] Bump `Cargo.toml` → `0.1.1576`; CHANGELOG entry; standing_backlog note
- [x] Prepend Implementation notes on task file; rename WIP → UNTESTED
- [x] `cargo check` in `src-tauri/`
- [x] Commit + push `origin/main`; GitHub comment via `gh-safe.sh`; do not close #14

## Review
Opaque wash on Monitors row Copied flash (`.monitor-item.is-just-copied`). Same pattern as process-row / other opaque flashes. `cargo check` green. Pushed `4d5afe93`. Issue #14 left open for tester / 004.
