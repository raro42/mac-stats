# UNTESTED-14 / #14 — tauri://localhost CPU (v0.1.1704)

## Plan
- [x] Pick fuel: standing P2 / GitHub #14 after v0.1.1703
- [x] Next glass cut: Apple History `.history-controls` + `.time-range-dropdown` borders (still `var(--hairline)` glass) after model-text opaque in v0.1.1703
- [x] Bump `Cargo.toml` → `0.1.1704`
- [x] Opaque `#ffffff` mix for History controls / time-range hairline borders
- [x] CHANGELOG `[0.1.1704]` entry
- [x] `cargo check` in `src-tauri/`
- [x] Rename WIP → UNTESTED; commit + push `origin/main`
- [x] Issue left open / not closed by coder (004 closes)

## Review
- Cut: Apple `.history-controls` / `.time-range-dropdown` resting · hover opaque hairline borders (was `var(--hairline)` glass).
- Parallel to time-range focus opaque in v0.1.1646. Model-text hover opaque in v0.1.1703.
- Linux webkit2gtk blank-page floor still applies; macOS Graphics and Media gate remains tester check.
