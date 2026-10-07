# WIP-14 / #14 — tauri://localhost CPU (v0.1.1719 text/muted)

## Plan
- [x] Pick fuel: standing P2 / GitHub #14 after v0.1.1718 (modal-backdrop)
- [x] Opaque shell mix for `--text` · `--muted` (+ leftover `--hairline` / `--panel` tokens) (v0.1.1719)
- [x] `cargo check` in `src-tauri/`
- [x] Rename CLOSED-14 → WIP-14
- [x] Commit + push `origin/main`
- [x] Leave GitHub issue alone (do not close)

## Review
- Coder: `--text` / `--muted` were `rgba(12, 12, 16, …)` glass under every always-on label (rings, strip, Agent Ops, AI Chat). Now `color-mix` against opaque shell `#f7f7fa`. Leftover unused `--hairline` / `--panel` / `--panel-border` / `--panel-shadow` glass tokens flattened the same way so they cannot reintroduce alpha.
- Tester next: open CPU window on macOS, warm ≥30s. Confirm labels stay solid (no glass alpha through to sparklines). Gauges/sparklines still update. Activity Monitor Graphics and Media / `tauri://localhost` toward <1%.
- Next fuel after bounce: icon-line resting/hover `color: rgba(12, 12, 16, …)` on the always-visible Monitors · AI Chat strip; screenshot feature-agent-ops when TCC allows.
