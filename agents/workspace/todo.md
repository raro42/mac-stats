# WIP-14 — next compositor cut (v0.1.1578)

## Plan
- [x] Pick lowest GitHub FEAT/WIP: `WIP-14` (tauri://localhost CPU)
- [x] Next cut after v0.1.1577: Perplexity result Copied flash (`.perplexity-result-item[role='option'].is-just-copied`) still mixed against transparent
- [x] Add opaque green wash (`#ffffff` mix), `box-shadow: none`
- [x] Sync `src/agent-ops.css` → `src-tauri/dist/agent-ops.css`
- [x] Bump `Cargo.toml` → `0.1.1578`; CHANGELOG entry; standing_backlog note
- [x] Prepend Implementation notes on task file; keep UNTESTED
- [x] `cargo check` / ratchet verify
- [x] Commit + push `origin/main`; do not close #14

## Review
Opaque wash on Perplexity result row Copied flash. Same pattern as monitor-item / process-row / disk-cleanup row. Ratchet keep @ `fc91df64`. Issue #14 left open for tester / 004.
