# Coder todo — UNTESTED-14 (tauri://localhost CPU)

## Plan
- [x] Pick lowest GitHub WIP/FEAT under `agents/tasks/` → WIP-14
- [x] Read FEATURE-CODER.md + lessons.md
- [x] v0.1.1613: Agent Ops close button resting · hover opaque wash (after remote took 1612 loading)
- [x] Sync `src-tauri/dist/agent-ops.css`
- [x] Bump Cargo.toml / lock / CHANGELOG / standing_backlog / Implementation notes
- [x] `cargo check` in `src-tauri/`
- [x] Commit + push `origin/main`
- [x] Rename WIP-14 → UNTESTED-14
- [x] Do **not** close GitHub #14

## Review
- v0.1.1613 `.ops-close-btn` resting · hover: opaque wash (no glass alpha).
- Remote already shipped v0.1.1612 `.ops-loading` opaque wash; kept both.
- Tester still needs macOS Activity Monitor gate (`tauri://localhost` / Graphics and Media toward <1%).
- Task is `agents/tasks/UNTESTED-14-20261004-1824-bug-tauri-lcoalhost-eating-cpu.md`.
