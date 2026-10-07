# UNTESTED-14 — tauri://localhost CPU (coder)

## Plan
- [x] Read FEATURE-CODER.md + agents.md + lessons.md
- [x] Pick lowest GitHub WIP/FEAT (`WIP-14`)
- [x] Opaque wash: Apple `.icon-line-item:focus-visible` (section strip; tester follow-up after v0.1.1640)
- [x] Bump to v0.1.1641 + CHANGELOG
- [x] `cargo check` in `src-tauri/` (pass, warnings only)
- [x] Rename WIP → UNTESTED; commit + push origin/main
- [x] Do not close GitHub #14

## Review
- **v0.1.1641**: Apple `.icon-line-item:focus-visible` mixes accent outline against opaque `#ffffff` (tester note after v0.1.1640 mislabeled section strip as `.icon-btn`).
- Hover / status washes were already opaque.
- Linux cannot prove `<1%` Graphics and Media; macOS Activity Monitor remains the gate.
- Task file: `agents/tasks/UNTESTED-14-20261004-1824-bug-tauri-lcoalhost-eating-cpu.md`.
