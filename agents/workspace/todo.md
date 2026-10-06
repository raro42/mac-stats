# WIP-14 — next compositor cut (v0.1.1563)

## Plan
- [x] Pick lowest GitHub FEAT/WIP: `WIP-14` (tauri://localhost CPU)
- [x] Next cut after v0.1.1562: Agent Ops Schedules kind filter Clear flash still mixes against `transparent`
- [x] Make `.ops-schedules-kind-filter-clear.is-just-saved` opaque (`#ffffff`), `box-shadow: none`
- [x] Sync `src/agent-ops.css` → `src-tauri/dist/agent-ops.css`
- [x] Bump `Cargo.toml` → `0.1.1563`; CHANGELOG entry
- [x] Prepend Implementation notes on task file; rename WIP → UNTESTED
- [x] `cargo check` in `src-tauri/`
- [x] Commit + push `origin/main`; GitHub comment via `gh-safe.sh`; do not close #14

## Review
Opaque wash on Agent Ops Schedules kind filter Clear (`.ops-schedules-kind-filter-clear.is-just-saved`). Same pattern as Agents enabled / Sessions kind Clear flashes. `cargo check` green (warnings only). Needs macOS Activity Monitor pass for #14 acceptance.
