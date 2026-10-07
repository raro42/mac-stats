# UNTESTED-14 — tauri://localhost CPU (coder)

## Plan
- [x] Read FEATURE-CODER.md + agents.md + lessons.md
- [x] Pick lowest GitHub WIP/FEAT (`WIP-14`)
- [x] Opaque wash: footer GitHub + app-version focus (always-visible collapsed layout)
- [x] Bump to v0.1.1643 + CHANGELOG + standing_backlog
- [x] `cargo check` in `src-tauri/` (in progress / see review)
- [x] Rename WIP → UNTESTED; commit + push origin/main
- [x] Do not close GitHub #14

## Review
- **v0.1.1643**: Footer `#github-link` hover · focus-visible and Apple `.app-version:focus-visible` · `.apple-github-link:focus-visible` mix against opaque `#ffffff`.
- Always-visible on the default collapsed layout.
- Linux cannot prove `<1%` Graphics and Media; macOS Activity Monitor remains the gate.
- Task file: `agents/tasks/UNTESTED-14-20261004-1824-bug-tauri-lcoalhost-eating-cpu.md`.
