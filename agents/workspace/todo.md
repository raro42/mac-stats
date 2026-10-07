# WIP-14 — next compositor cut (v0.1.1585)

## Plan
- [x] Pick lowest GitHub FEAT/WIP: `WIP-14` (tauri://localhost CPU)
- [x] Next cut after v0.1.1584: Top Processes row pinned / hover / focus / active / selected still mixed against transparent glass
- [x] Add opaque accent wash (`#ffffff` mix); drop hover glass drop shadow
- [x] Sync `src/agent-ops.css` → `src-tauri/dist/agent-ops.css`
- [x] Bump `Cargo.toml` → `0.1.1585`; CHANGELOG entry; standing_backlog note
- [x] Prepend Implementation notes on task file; keep TESTING for Linux host
- [x] `cargo check` / ratchet verify
- [x] Commit + push `origin/main`; do not close #14

## Review
Opaque wash on Top Processes row interaction states. Same pattern as Details value hover. Issue #14 left open for tester / 004.
