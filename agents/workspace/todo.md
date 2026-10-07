# UNTESTED-14 / #14 — tauri://localhost CPU (v0.1.1703)

## Plan
- [x] Pick fuel: standing P2 / GitHub #14 after v0.1.1702
- [x] Next glass cut: Apple AI Chat `.model-text:hover` (still `var(--hairline)`) + `.connection-indicator:focus-visible` (still `transparent` mix) after Close opaque in v0.1.1702
- [x] Bump `Cargo.toml` → `0.1.1703`
- [x] Opaque `#ffffff` mix for model-text hover + connection focus ring
- [x] CHANGELOG `[0.1.1703]` entry
- [x] `cargo check` in `src-tauri/`
- [x] Rename WIP → UNTESTED; commit + push `origin/main`
- [x] Issue left open / not closed by coder (004 closes)

## Review
- Cut: Apple `.model-text:hover` opaque wash (was hairline glass); `.connection-indicator:focus-visible` opaque ring (was transparent mix).
- Parallel to model-text focus-visible opaque in v0.1.1692. Popover Close hover opaque in v0.1.1702.
- Linux webkit2gtk blank-page floor still applies; macOS Graphics and Media gate remains tester check.
