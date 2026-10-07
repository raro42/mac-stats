# WIP-14 / #14 — tauri://localhost CPU (v0.1.1691)

## Plan
- [x] Pick lowest GitHub FEAT/WIP: `WIP-14-…` (issue #14)
- [x] Next glass cut: Apple Monitors overflow menu (`.monitors-menu` shell · item hover · focus-visible)
- [x] Bump `Cargo.toml` → `0.1.1691`
- [x] Opaque fill / border; drop soft glass shadow; focus rings against `#ffffff`
- [x] CHANGELOG `[0.1.1691]` entry
- [x] Refresh Implementation section → rename `UNTESTED-14-…`
- [x] `cargo check` in `src-tauri/` (green; unused-import warnings pre-existing)
- [ ] Commit + push `origin/main`
- [ ] GitHub comment via `gh-safe.sh` (issue not closed)

## Review
- Cut: Apple `.monitors-menu` opaque `#ffffff` (was `var(--panel)` / `var(--panel-shadow)` glass). Item hover / focus and menu-button focus ring mix against `#ffffff` (was transparent).
- Parallel to AI Chat overflow menu opaque cut in v0.1.1690.
- Linux webkit2gtk blank-page floor still applies; macOS Graphics and Media gate remains tester check.
- Task file: `agents/tasks/UNTESTED-14-20261004-1824-bug-tauri-lcoalhost-eating-cpu.md`
