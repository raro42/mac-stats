# WIP-14 / #14 — tauri://localhost CPU (v0.1.1699)

## Plan
- [x] Pick lowest GitHub FEAT/WIP: `WIP-14-…` (issue #14)
- [x] Next glass cut: Apple History sparkline tooltip (`.history-tooltip`) — still `rgba(28,28,30,0.92)` glass + soft shadow after markdown opaque in v0.1.1698
- [x] Bump `Cargo.toml` → `0.1.1699`
- [x] Opaque `#1c1c1e` / `#f5f5f7`; drop soft shadow (monitor-tick-tip parity v0.1.1594)
- [x] CHANGELOG `[0.1.1699]` entry
- [x] Refresh Implementation section → rename `WIP-` → `UNTESTED-14-…`
- [x] `cargo check` in `src-tauri/`
- [x] Commit + push `origin/main`
- [x] Issue left open / not closed by coder (004 closes)

## Review
- Cut: Apple `.history-tooltip` opaque fill (was rgba glass + soft drop shadow).
- Parallel to `.monitor-tick-tip` opaque in v0.1.1594. Markdown shells opaque in v0.1.1698.
- Linux webkit2gtk blank-page floor still applies; macOS Graphics and Media gate remains tester check.
- Task file: `agents/tasks/UNTESTED-14-20261004-1824-bug-tauri-lcoalhost-eating-cpu.md`
