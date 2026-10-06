# WIP-14 — next compositor cut (v0.1.1566)

## Plan
- [x] Pick lowest GitHub FEAT/WIP: `WIP-14` (tauri://localhost CPU)
- [x] Next cut after v0.1.1565: Perplexity filter Clear flash still mixes against `transparent`
- [x] Make `.perplexity-filter-clear.is-just-saved` opaque (`#ffffff`), `box-shadow: none`
- [x] Sync `src/agent-ops.css` → `src-tauri/dist/agent-ops.css`
- [x] Bump `Cargo.toml` → `0.1.1566`; CHANGELOG entry
- [x] Prepend Implementation notes on task file; rename WIP → UNTESTED
- [x] `cargo check` in `src-tauri/`
- [x] Commit + push `origin/main`; GitHub comment via `gh-safe.sh`; do not close #14

## Review
Opaque wash on Perplexity filter Clear (`.perplexity-filter-clear.is-just-saved`). Same pattern as Agent Ops Runs / Knowledge / Schedules Clear flashes. `cargo check` green (warnings only). Needs macOS Activity Monitor pass for #14 acceptance.
