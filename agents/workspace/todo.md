# WIP-14 / #14 — tauri://localhost CPU (v0.1.1687)

## Plan
- [x] Pick lowest GitHub FEAT/WIP: `WIP-14-…` (issue #14)
- [x] Next glass cut: Apple Ollama settings system-prompt textarea (`.ollama-settings-popover .popover-body textarea`)
- [x] Bump `Cargo.toml` → `0.1.1687`
- [x] Opaque resting / focus washes mixed against `#ffffff`
- [x] CHANGELOG `[0.1.1687]` entry
- [x] Refresh Implementation section → rename `UNTESTED-14-…`
- [x] `cargo check` in `src-tauri/` (green; unused-import warnings pre-existing)
- [x] Commit + push `origin/main` (`359c525c`)
- [x] GitHub comment via `gh-safe.sh` (issue not closed)

## Review
- Cut: Apple `.ollama-settings-popover .popover-body textarea` resting · focus mixes against opaque `#ffffff` (was `rgba(255,255,255,0.7)` / `0.9`).
- Parallel to `.add-monitor-form input` / `#chat-input` opaque wash pattern.
- Linux webkit2gtk blank-page floor still applies; macOS Graphics and Media gate remains tester check.
- Task file: `agents/tasks/UNTESTED-14-20261004-1824-bug-tauri-lcoalhost-eating-cpu.md`
