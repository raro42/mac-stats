# WIP-14 / #14 — tauri://localhost CPU (v0.1.1694)

## Plan
- [x] Pick lowest GitHub FEAT/WIP: `WIP-14-…` (issue #14)
- [x] Next glass cut: Apple AI Chat thinking shell (`.chat-message.thinking`) — still mixed against `transparent` after exec/answer opaque in v0.1.1693
- [x] Bump `Cargo.toml` → `0.1.1694`
- [x] Mix wash / dashed border against `#ffffff`
- [x] CHANGELOG `[0.1.1694]` entry
- [x] Refresh Implementation section → rename `WIP-` → `UNTESTED-14-…`
- [x] `cargo check` in `src-tauri/`
- [x] Commit + push `origin/main` (`cfffa04e` / docs `79f2aa37`; HTTPS after SSH key deny)
- [x] GitHub comment via `gh-safe.sh` (issue not closed)

## Review
- Cut: Apple `.chat-message.thinking` mix against opaque `#ffffff` (was `transparent` glass on dashed border + wash).
- Parallel to Apple message-bubble / empty-shell / exec-answer "glass put back" cuts. Exec / answer cards opaque in v0.1.1693.
- Linux webkit2gtk blank-page floor still applies; macOS Graphics and Media gate remains tester check.
- Task file: `agents/tasks/UNTESTED-14-20261004-1824-bug-tauri-lcoalhost-eating-cpu.md`
