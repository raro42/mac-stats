# WIP-14 / #14 — tauri://localhost CPU (v0.1.1700)

## Plan
- [x] Pick lowest GitHub FEAT/WIP: `WIP-14-…` (issue #14)
- [x] Next glass cut: Apple section collapse control (`.collapse-btn` hover · focus-visible) — still `var(--hairline)` / `transparent` after section headers opaque in v0.1.1635
- [x] Bump `Cargo.toml` → `0.1.1700`
- [x] Opaque `#ffffff` mix for hover wash + focus ring
- [x] CHANGELOG `[0.1.1700]` entry
- [x] Refresh Implementation section → rename `WIP-` → `UNTESTED-14-…`
- [x] `cargo check` in `src-tauri/`
- [x] Commit + push `origin/main`
- [x] Issue left open / not closed by coder (004 closes)

## Review
- Cut: Apple `.collapse-btn` hover / focus-visible opaque wash (was hairline glass + transparent focus mix).
- Parallel to `.section-header-collapsible` opaque in v0.1.1635. History tooltip opaque in v0.1.1699.
- Linux webkit2gtk blank-page floor still applies; macOS Graphics and Media gate remains tester check.
- Task file: `agents/tasks/UNTESTED-14-20261004-1824-bug-tauri-lcoalhost-eating-cpu.md`
