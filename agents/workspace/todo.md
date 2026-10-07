# Coder todo — UNTESTED-14 (tauri://localhost CPU)

## Plan
- [x] Pick lowest GitHub WIP/FEAT under `agents/tasks/` → WIP-14
- [x] Read FEATURE-CODER.md + lessons.md
- [x] v0.1.1607: Agent Ops base `.ops-row` (resting · hover · focus · selected) opaque wash
- [x] Sync `src-tauri/dist/agent-ops.css`
- [x] Bump Cargo.toml / lock / CHANGELOG / standing_backlog / Implementation notes
- [x] `cargo check` in `src-tauri/`
- [x] Commit + push `origin/main`
- [x] Rename WIP-14 → UNTESTED-14
- [ ] Do **not** close GitHub #14

## Review
- v0.1.1607 base `.ops-row` resting/hover/focus-visible/selected: opaque wash (soft hover blur shadow dropped).
- Lane-specific Lite · Slow · Fail row washes already opaque (v0.1.1606).
- Tester still needs macOS Activity Monitor gate (`tauri://localhost` / Graphics and Media toward <1%).
- Task is `agents/tasks/UNTESTED-14-20261004-1824-bug-tauri-lcoalhost-eating-cpu.md`.
