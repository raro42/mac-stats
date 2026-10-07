# UNTESTED-14 / #14 — tauri://localhost CPU (v0.1.1714 shipped)

## Plan
- [x] Pick fuel: standing P2 / GitHub #14 after v0.1.1713 (chat-messages border)
- [x] Opaque `#ffffff` mix for Monitors / AI Chat settings popover header hairlines (v0.1.1714)
- [x] `cargo check` in `src-tauri/`
- [x] Keep `UNTESTED-14-…` for tester (no FEAT/WIP sibling; claim updated in place)
- [x] Commit + push `origin/main`
- [x] Leave GitHub issue alone (004 closes; #14 already closed on GitHub)

## Review
- Coder: `.monitors-settings-popover .popover-header` and `.ollama-settings-popover .popover-header` mix `border-bottom` against opaque `#ffffff` (was `var(--hairline)` glass). Message-list border opaque in v0.1.1713.
- Tester next: expand Monitors or AI Chat → open settings popover; confirm title-row hairline solid; Activity Monitor Graphics and Media / `tauri://localhost` toward <1%.
- Next fuel after bounce: remaining `var(--hairline)` (add-monitor form, monitor-history, markdown table/hr).
