# WIP-14 / #14 — tauri://localhost CPU (v0.1.1692)

## Plan
- [x] Pick lowest GitHub FEAT/WIP: `WIP-14-…` (issue #14)
- [x] Next glass cut: Apple AI Chat model-select dropdown (`.model-select-dropdown` shell · option hover; `.model-text` focus-visible)
- [x] Bump `Cargo.toml` → `0.1.1692`
- [x] Opaque fill / border; drop soft glass shadow; focus rings against `#ffffff`
- [x] CHANGELOG `[0.1.1692]` entry
- [x] Refresh Implementation section → rename `UNTESTED-14-…`
- [x] `cargo check` in `src-tauri/` (green; unused-import warnings pre-existing)
- [ ] Commit + push `origin/main` (commit `4c1ce291` done; push blocked by GitHub HTTP 500 / receive-pack)
- [ ] GitHub comment via `gh-safe.sh` (issue not closed; API writes also 500)

## Review
- Cut: Apple `.model-select-dropdown` opaque `#ffffff` (was `var(--panel)` / `var(--panel-shadow)` glass). Option hover and model-text focus ring mix against `#ffffff` (was `var(--hairline)` / transparent).
- Parallel to Monitors / AI Chat overflow menu opaque cuts in v0.1.1691 / v0.1.1690. Model select control opaque in v0.1.1681.
- Linux webkit2gtk blank-page floor still applies; macOS Graphics and Media gate remains tester check.
- Task file: `agents/tasks/UNTESTED-14-20261004-1824-bug-tauri-lcoalhost-eating-cpu.md`
- Local commit `4c1ce291` on `main`. Push to `origin/main` failed repeatedly (GitHub receive-pack HTTP 500). Issue #14 left open.
