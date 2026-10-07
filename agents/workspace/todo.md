# UNTESTED-14 / #14 — tauri://localhost CPU (v0.1.1716 shipped)

## Plan
- [x] Pick fuel: standing P2 / GitHub #14 after v0.1.1715 (Add form / history)
- [x] Opaque `#ffffff` mix for AI Chat markdown table/hr hairlines (v0.1.1716)
- [x] `cargo check` in `src-tauri/`
- [x] Rename `WIP-14-…` → `UNTESTED-14-…` (lean claim; trim prior 1MB report dump)
- [x] Commit + push `origin/main`
- [x] Leave GitHub issue alone (004 closes; #14 already closed on GitHub)

## Review
- Coder: `.chat-message .markdown table th/td` and `.chat-message .markdown hr` mix hairlines against opaque `#ffffff` (was `var(--hairline)` glass). Add form / history opaque in v0.1.1715. No remaining `var(--hairline)` in Apple `cpu.css`.
- Tester next: expand AI Chat, render a reply with a markdown table and/or `hr`, warm ≥30s. Confirm cell/hr borders stay solid (no glass alpha). Gauges/sparklines still update. Activity Monitor Graphics and Media / `tauri://localhost` toward <1%.
- Next fuel after bounce: other glass alpha (`--panel`, soft shadows, remaining rgba borders) if macOS still over ~1%.
