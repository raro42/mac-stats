# WIP-14 / #14 — tauri://localhost CPU (v0.1.1689)

## Plan
- [x] Pick lowest GitHub FEAT/WIP: `WIP-14-…` (issue #14)
- [x] Next glass cut: Apple Monitors settings Remove (`.monitor-remove-btn` resting · hover · focus-visible)
- [x] Bump `Cargo.toml` → `0.1.1689`
- [x] Opaque resting / hover / focus washes mixed against `#ffffff` (Apple + shared focus ring)
- [x] CHANGELOG `[0.1.1689]` entry
- [x] Refresh Implementation section → rename `UNTESTED-14-…`
- [x] `cargo check` in `src-tauri/` (green; unused-import warnings pre-existing)
- [ ] Commit + push `origin/main`
- [ ] GitHub comment via `gh-safe.sh` (issue not closed)

## Review
- Cut: Apple `.monitor-remove-btn` resting · hover · focus-visible mixes against opaque `#ffffff` (was `rgba(255,59,48,0.1)` / `0.2` glass). Shared focus ring mixes against `#ffffff` (was `transparent`).
- Parallel to `.force-quit-btn` opaque wash pattern.
- Linux webkit2gtk blank-page floor still applies; macOS Graphics and Media gate remains tester check.
- Task file: `agents/tasks/UNTESTED-14-20261004-1824-bug-tauri-lcoalhost-eating-cpu.md`
