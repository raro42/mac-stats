# WIP-14 / #14 — tauri://localhost CPU (v0.1.1693)

## Plan
- [x] Pick lowest GitHub FEAT/WIP: `WIP-14-…` (issue #14)
- [x] Next glass cut: Apple AI Chat exec / answer cards (`.chat-exec-card` · `.chat-exec-code` · `.chat-answer-part` · final) — shared sheet opaque in v0.1.1621; Apple put `transparent` glass back
- [x] Bump `Cargo.toml` → `0.1.1693`
- [x] Mix washes / borders against `#ffffff` (match shared `agent-ops.css`)
- [x] CHANGELOG `[0.1.1693]` entry
- [x] Refresh Implementation section → rename `WIP-` → `UNTESTED-14-…`
- [x] `cargo check` in `src-tauri/` (green; unused-import warnings pre-existing)
- [x] Commit + push `origin/main` (`11d8b0e4`; HTTPS push after SSH key deny)
- [x] GitHub comment via `gh-safe.sh` (issue not closed)

## Review
- Cut: Apple `.chat-exec-card` · `.chat-exec-code` · `.chat-answer-part` · final mix against opaque `#ffffff` (was `transparent` glass). Matches shared `agent-ops.css` from v0.1.1621.
- Parallel to Apple message-bubble / empty-shell "glass put back" cuts. Model-select dropdown opaque in v0.1.1692.
- Linux webkit2gtk blank-page floor still applies; macOS Graphics and Media gate remains tester check.
- Task file: `agents/tasks/UNTESTED-14-20261004-1824-bug-tauri-lcoalhost-eating-cpu.md`
