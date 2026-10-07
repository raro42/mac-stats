# Coder todo — WIP-14 (tauri://localhost CPU)

## Plan
- [x] Pick lowest GitHub FEAT/WIP: WIP-14
- [x] Convert Agent Ops filter row (`.ops-filter-input` / `.ops-filter-match` / `.ops-filter-clear`) glass blends to opaque `#ffffff`
- [x] Sync `src-tauri/dist/agent-ops.css`
- [x] Bump `src-tauri/Cargo.toml` → `0.1.1598`
- [x] Update WIP Implementation notes; `cargo check` in `src-tauri/`
- [x] Commit + push `origin/main`; rename `WIP-` → `UNTESTED-`

## Review
- v0.1.1598: Agent Ops filter input / match / Clear / just-cleared opaque washes (#14).
- `cargo check` in `src-tauri/` succeeded (pre-existing unused import warning in `feature_health.rs` only).
- Commit `7636e715` pushed to `origin/main`. Task file: `UNTESTED-14-…`. GitHub #14 left open for tester/004.
