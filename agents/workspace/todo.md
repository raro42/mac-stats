# WIP-14 — next compositor cut (v0.1.1574)

## Plan
- [x] Pick lowest GitHub FEAT/WIP: `WIP-14` (tauri://localhost CPU)
- [x] Next cut after v0.1.1573: Details value Copied flash (`.details-grid > .detail-value[role='option'].is-just-copied`) still mixed against transparent
- [x] Add opaque green wash (`#ffffff` mix), `box-shadow: none`
- [x] Sync `src/agent-ops.css` → `src-tauri/dist/agent-ops.css`
- [x] Bump `Cargo.toml` → `0.1.1574`; CHANGELOG entry
- [x] Prepend Implementation notes on task file; rename WIP → UNTESTED
- [x] `cargo check` / ratchet verify in `src-tauri/`
- [x] Commit + push `origin/main`; GitHub comment via `gh-safe.sh`; do not close #14

## Review
Opaque wash on Details value Copied flash (`.details-grid > .detail-value[role='option'].is-just-copied`). Same pattern as other opaque flashes. `cargo check` green. Issue #14 left open for tester / 004.
