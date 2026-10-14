# WIP-19 — Note Linux as development/testing host in README (#19)

## Plan
- [x] Add a short README note: product target remains macOS Apple Silicon; agent/dev/test work often runs on Linux
- [x] CHANGELOG `[Unreleased]` docs line
- [x] `cargo check` in `src-tauri/`
- [x] Commit + push `origin/main`
- [x] Rename `WIP-19-…` → `UNTESTED-19-…` (do not close GitHub issue)

## Review
- Issue asked only to document that development/testing often runs on Linux, not to ship a Linux menu-bar product.
- README callout under the Apple Silicon note + Build from source sentence; CHANGELOG `[Unreleased]`.
- `cargo check` OK (existing Linux stub warnings only).
