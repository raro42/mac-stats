# WIP-14 — next compositor cut (v0.1.1584)

## Plan
- [x] Pick lowest GitHub FEAT/WIP: `WIP-14` (tauri://localhost CPU)
- [x] Next cut after v0.1.1583: Details value hover / focus-visible / selected still mixed against transparent glass
- [x] Add opaque accent wash (`#ffffff` mix) for `:hover`, `:focus-visible`, `.is-selected`
- [x] Sync `src/agent-ops.css` → `src-tauri/dist/agent-ops.css`
- [x] Bump `Cargo.toml` → `0.1.1584`; CHANGELOG entry; standing_backlog note
- [x] Prepend Implementation notes on task file; rename WIP → UNTESTED
- [x] `cargo check` / ratchet verify
- [x] Commit + push `origin/main`; do not close #14

## Review
Opaque wash on Details value hover / focus-visible / selected. Same pattern as ring/power copy hover. Issue #14 left open for tester / 004.
