# UNTESTED-14 / #14 — tauri://localhost CPU (v0.1.1713 shipped)

## Plan
- [x] Pick fuel: standing P2 / GitHub #14 after v0.1.1712 (icon-strip / section dividers)
- [x] Opaque `#ffffff` mix for `.chat-messages` resting border (v0.1.1713)
- [x] `cargo check` in `src-tauri/`
- [x] Rename `WIP-14-…` → `UNTESTED-14-…`
- [x] Commit + push `origin/main`
- [x] Leave GitHub issue alone (004 closes; #14 already closed on GitHub)

## Review
- Coder: Apple AI Chat `.chat-messages` resting hairline mixes against opaque `#ffffff` (was `var(--hairline)` glass on the already-opaque panel fill). Icon-strip / section dividers opaque in v0.1.1712.
- Tester next: expand AI Chat on macOS, warm ≥30s; confirm message-list border solid; Activity Monitor Graphics and Media / `tauri://localhost` toward <1%.
- Next fuel after bounce: remaining theme glass (`var(--hairline)` on popover headers / markdown table / monitor form).
