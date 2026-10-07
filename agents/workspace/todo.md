# WIP-14 / #14 — tauri://localhost CPU (v0.1.1697)

## Plan
- [x] Pick lowest GitHub FEAT/WIP: `WIP-14-…` (issue #14)
- [x] Next glass cut: Apple Monitors Add (`.add-btn-small`) — still `rgba(255,255,255,0.3/0.4/0.5)` glass + transparent focus mix after Changelog inline code opaque in v0.1.1696
- [x] Bump `Cargo.toml` → `0.1.1697`
- [x] Mix fills against `#ffffff`
- [x] CHANGELOG `[0.1.1697]` entry
- [x] Refresh Implementation section → rename `WIP-` → `UNTESTED-14-…`
- [x] `cargo check` in `src-tauri/`
- [x] Commit + push `origin/main`
- [x] Issue left open / not closed by coder (004 closes)

## Review
- Cut: Apple `.add-btn-small` resting / hover / active / focus-visible mix against opaque `#ffffff` (was white rgba glass + transparent focus).
- Parallel to Monitors Add-form URL input opaque in v0.1.1686. Changelog inline code opaque in v0.1.1696.
- Linux webkit2gtk blank-page floor still applies; macOS Graphics and Media gate remains tester check.
- Task file: `agents/tasks/UNTESTED-14-20261004-1824-bug-tauri-lcoalhost-eating-cpu.md`
