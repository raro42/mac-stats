# UNTESTED-14 / #14 — tauri://localhost CPU (v0.1.1705)

## Plan
- [x] Pick fuel: standing P2 / GitHub #14 after v0.1.1704
- [x] Next glass cut: Apple battery / power strip hairline borders (still `var(--hairline)` glass; always on default idle view). Dropped a dead Details/Processes scrollbar-thumb draft (shared `agent-ops.css` `!important` thumbs win).
- [x] Bump `Cargo.toml` → `0.1.1705`
- [x] Opaque `#ececf1` / `#e4e4ea` mix for `.battery-power-strip` resting · hover borders
- [x] CHANGELOG `[0.1.1705]` entry
- [x] `cargo check` in `src-tauri/`
- [x] Rename WIP → UNTESTED; commit + push `origin/main`
- [x] Issue left open / not closed by coder (004 closes)

## Review
- Cut: Apple `.battery-power-strip` resting · hover opaque hairline borders (was `var(--hairline)` glass).
- Parallel to History controls opaque in v0.1.1704. Focus-within opaque in v0.1.1644. Details/Processes section hairlines still glass (next fuel).
- Note: `origin/main` commit subject for v0.1.1705 still says scrollbar thumbs; tree / CHANGELOG / task file document the battery-strip cut.
- Linux webkit2gtk blank-page floor still applies; macOS Graphics and Media gate remains tester check.
