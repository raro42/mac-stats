# WIP-14 / #14 — tauri://localhost CPU (v0.1.1702)

## Plan
- [x] Pick fuel: standing P2 / GitHub #14 after v0.1.1701
- [x] Next glass cut: Apple Monitors / AI Chat settings popover Close hover (`.popover-close`) — still `var(--hairline)` after menu-btn opaque in v0.1.1701
- [x] Bump `Cargo.toml` → `0.1.1702`
- [x] Opaque `#ffffff` mix for Close hover wash
- [x] CHANGELOG `[0.1.1702]` entry
- [x] `cargo check` via autoresearch_ratchet verify
- [x] Commit + push `origin/main`
- [x] Issue left open / not closed by coder (004 closes)

## Review
- Cut: Apple `.monitors-settings-popover .popover-close` / `.ollama-settings-popover .popover-close` hover opaque wash (was hairline glass).
- Parallel to overflow menu-btn opaque in v0.1.1701. Popover shells opaque in v0.1.1683.
- Linux webkit2gtk blank-page floor still applies; macOS Graphics and Media gate remains tester check.
- Task file: already in `agents/testing/active/TESTING-14-…` from prior ship; leave tester path alone.
