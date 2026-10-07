# WIP-14 / #14 — tauri://localhost CPU (v0.1.1698)

## Plan
- [x] Pick lowest GitHub FEAT/WIP: `WIP-14-…` (issue #14)
- [x] Next glass cut: Apple AI Chat markdown (`blockquote` · `code` · `pre` · `table th`) — still `rgba(12,12,16,0.03/0.05/0.08)` glass after Monitors Add opaque in v0.1.1697
- [x] Bump `Cargo.toml` → `0.1.1698`
- [x] Mix washes against `#ffffff`
- [x] CHANGELOG `[0.1.1698]` entry
- [x] Refresh Implementation section → rename `WIP-` → `UNTESTED-14-…`
- [x] `cargo check` in `src-tauri/`
- [x] Commit + push `origin/main`
- [x] Issue left open / not closed by coder (004 closes)

## Review
- Cut: Apple `.chat-message .markdown` blockquote / code / pre / table th mix against opaque `#ffffff` (was dark rgba glass + hairline glass on pre).
- Parallel to Changelog inline code opaque in v0.1.1696 and `.chat-exec-code` opaque in v0.1.1693. Monitors Add opaque in v0.1.1697.
- Linux webkit2gtk blank-page floor still applies; macOS Graphics and Media gate remains tester check.
- Task file: `agents/tasks/UNTESTED-14-20261004-1824-bug-tauri-lcoalhost-eating-cpu.md`
