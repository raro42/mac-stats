# WIP-14 / #14 — tauri://localhost CPU (v0.1.1695)

## Plan
- [x] Pick lowest GitHub FEAT/WIP: `WIP-14-…` (issue #14)
- [x] Next glass cut: Apple Changelog loading / error (`.changelog-loading` · `.changelog-error`) — still mixed against `transparent` after thinking shell opaque in v0.1.1694
- [x] Bump `Cargo.toml` → `0.1.1695`
- [x] Mix wash / dashed border / soft-alert against `#ffffff`
- [x] CHANGELOG `[0.1.1695]` entry
- [x] Refresh Implementation section → rename `WIP-` → `UNTESTED-14-…`
- [x] `cargo check` in `src-tauri/`
- [x] Commit + push `origin/main` (`f5899d26`; HTTPS after SSH key deny)
- [x] Issue left open (004 closes)

## Review
- Cut: Apple `.changelog-loading` / `.changelog-error` mix against opaque `#ffffff` (was `transparent` glass on dashed border + wash + soft-alert).
- Parallel to Apple `.process-empty` / `.monitors-empty` opaque empty shells. Thinking shell opaque in v0.1.1694.
- Linux webkit2gtk blank-page floor still applies; macOS Graphics and Media gate remains tester check.
- Task file: `agents/tasks/UNTESTED-14-20261004-1824-bug-tauri-lcoalhost-eating-cpu.md`
