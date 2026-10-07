# WIP-14 / #14 — tauri://localhost CPU (v0.1.1684)

## Plan
- [x] Pick lowest GitHub FEAT/WIP: `WIP-14-…` (issue #14)
- [x] Next glass cut: Apple Ollama settings popover shell (mirror Monitors popover v0.1.1683)
- [x] Bump `Cargo.toml` → `0.1.1684`
- [x] Opaque `.ollama-settings-popover .popover-content` + close focus ring mix
- [x] CHANGELOG `[0.1.1684]` entry
- [x] Refresh Implementation section → rename `UNTESTED-14-…`
- [x] `cargo check` in `src-tauri/` (green; unused-import warning pre-existing)
- [ ] Commit + push `origin/main`
- [ ] GitHub comment via `gh-safe.sh` (do not close issue)

## Review
- Cut: Apple `.ollama-settings-popover .popover-content` opaque `#ffffff`, hairline via `color-mix`, `box-shadow: none`; close focus ring mixes against `#ffffff`.
- Parallel to Monitors settings popover in v0.1.1683.
- Linux webkit2gtk blank-page floor still applies; macOS Graphics and Media gate remains tester check.
