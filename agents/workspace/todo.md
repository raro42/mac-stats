# WIP-14 / #14 — tauri://localhost CPU (v0.1.1696)

## Plan
- [x] Pick lowest GitHub FEAT/WIP: `WIP-14-…` (issue #14)
- [x] Next glass cut: Apple Changelog inline code (`.changelog-code`) — still `rgba(0,0,0,0.06)` glass after loading/error opaque in v0.1.1695
- [x] Bump `Cargo.toml` → `0.1.1696`
- [x] Mix wash against `#ffffff`
- [x] CHANGELOG `[0.1.1696]` entry
- [x] Refresh Implementation section → rename `WIP-` → `UNTESTED-14-…`
- [x] `cargo check` in `src-tauri/`
- [x] Commit + push `origin/main` (`57f92d58`; HTTPS after SSH key deny)
- [x] Issue left open (004 closes)

## Review
- Cut: Apple `.changelog-code` mix against opaque `#ffffff` (was `rgba(0,0,0,0.06)` glass).
- Parallel to Apple `.chat-exec-code` opaque code wash. Loading / error shells opaque in v0.1.1695.
- Linux webkit2gtk blank-page floor still applies; macOS Graphics and Media gate remains tester check.
- Task file: `agents/tasks/UNTESTED-14-20261004-1824-bug-tauri-lcoalhost-eating-cpu.md`
