# UNTESTED-14 / #14 — tauri://localhost CPU (v0.1.1715 shipped)

## Plan
- [x] Pick fuel: standing P2 / GitHub #14 after v0.1.1714 (popover headers)
- [x] Opaque `#ffffff` mix for Monitors Add form + row history hairlines (v0.1.1715)
- [x] `cargo check` in `src-tauri/`
- [x] Keep `UNTESTED-14-…` for tester (no FEAT/WIP sibling; claim updated in place)
- [x] Commit + push `origin/main`
- [x] Leave GitHub issue alone (004 closes; #14 already closed on GitHub)

## Review
- Coder: `.add-monitor-form` and `.monitor-history` mix `border-top` against opaque `#ffffff` (was `var(--hairline)` glass). Popover headers opaque in v0.1.1714.
- Tester next: expand Monitors → settings; confirm Add form divider and row history tick divider solid; Activity Monitor Graphics and Media / `tauri://localhost` toward <1%.
- Next fuel after bounce: remaining `var(--hairline)` (markdown table/hr).
