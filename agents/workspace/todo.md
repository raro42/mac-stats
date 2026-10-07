# WIP-14 — next compositor cut (v0.1.1592)

## Plan
- [x] Pick lowest GitHub FEAT/WIP: `WIP-14` (tauri://localhost CPU)
- [x] Next cut after v0.1.1591: AI Chat filter chips still mixed against transparent glass
- [x] Add opaque washes (`#ffffff` mix) on `.chat-filter-chip` / `.chat-filter-clear` states
- [x] Sync `src/agent-ops.css` → `src-tauri/dist/agent-ops.css`
- [x] Bump `Cargo.toml` → `0.1.1592`; CHANGELOG entry; standing_backlog note
- [x] Prepend Implementation notes; rename WIP → UNTESTED
- [x] `cargo check` in `src-tauri/`
- [x] Commit + push `origin/main`; do not close #14

## Review
Opaque wash on AI Chat filter chips (All · You · Assistant · Errors) and Clear. Same #14 glass-alpha pattern. Issue left open for tester / 004.
