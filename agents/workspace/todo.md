# WIP-14 — next compositor cut (v0.1.1583)

## Plan
- [x] Pick lowest GitHub FEAT/WIP: `WIP-14` (tauri://localhost CPU)
- [x] Next cut after v0.1.1582: ring / power-strip copy hover + focus-visible still mixed against transparent glass
- [x] Add opaque accent wash (`#ffffff` mix) for `:hover` and `:focus-visible`
- [x] Sync `src/cpu.js` → `src-tauri/dist/cpu.js`
- [x] Bump `Cargo.toml` → `0.1.1583`; CHANGELOG entry; standing_backlog note
- [x] Prepend Implementation notes on task file; rename WIP → UNTESTED
- [x] `cargo check` / ratchet verify
- [x] Commit + push `origin/main`; do not close #14

## Review
Opaque wash on ring and power-strip copy hover / focus-visible. Same pattern as Copied flash. Issue #14 left open for tester / 004.
