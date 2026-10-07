# WIP-14 / #14 — tauri://localhost CPU (v0.1.1685)

## Plan
- [x] Pick lowest GitHub FEAT/WIP: `WIP-14-…` (issue #14)
- [x] Next glass cut: Apple Monitors settings list rows (`.monitor-settings-item`)
- [x] Bump `Cargo.toml` → `0.1.1685`
- [x] Opaque resting / hover washes mixed against `#ffffff`
- [x] CHANGELOG `[0.1.1685]` entry
- [x] Refresh Implementation section → rename `UNTESTED-14-…`
- [x] `cargo check` in `src-tauri/` (green; unused-import warning pre-existing)
- [x] Commit + push `origin/main` (`871f5add`)
- [x] GitHub comment via `gh-safe.sh` (issue not closed)

## Review
- Cut: Apple `.monitor-settings-item` resting · hover mixes against opaque `#ffffff` (was `rgba(255,255,255,0.5)` / `0.7`).
- Parallel to `.monitor-item` opaque wash pattern.
- Linux webkit2gtk blank-page floor still applies; macOS Graphics and Media gate remains tester check.
- Task file: `agents/tasks/UNTESTED-14-20261004-1824-bug-tauri-lcoalhost-eating-cpu.md`
