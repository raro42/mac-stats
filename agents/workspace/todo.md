# WIP-14 / #14 — tauri://localhost CPU (v0.1.1688)

## Plan
- [x] Pick lowest GitHub FEAT/WIP: `WIP-14-…` (issue #14)
- [x] Next glass cut: Apple Ollama settings Save/Cancel (`.popover-btn-primary` · `.popover-btn-secondary`)
- [x] Bump `Cargo.toml` → `0.1.1688`
- [x] Opaque resting / hover / focus washes mixed against `#ffffff`
- [x] CHANGELOG `[0.1.1688]` entry
- [x] Refresh Implementation section → rename `UNTESTED-14-…`
- [x] `cargo check` in `src-tauri/` (green; unused-import warnings pre-existing)
- [ ] Commit + push `origin/main`
- [ ] GitHub comment via `gh-safe.sh` (issue not closed)

## Review
- Cut: Apple `.ollama-settings-popover .popover-btn-primary` · `.popover-btn-secondary` resting · hover · focus-visible mixes against opaque `#ffffff` (was `rgba(0,122,255,0.9)` / `rgba(255,255,255,0.5)` / `0.7` + transparent focus).
- Parallel to `#chat-send-btn` / `.settings-btn` opaque wash pattern.
- Linux webkit2gtk blank-page floor still applies; macOS Graphics and Media gate remains tester check.
- Task file: `agents/tasks/UNTESTED-14-20261004-1824-bug-tauri-lcoalhost-eating-cpu.md`
