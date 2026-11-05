# WIP-20 — Data Poster light backgrounds (#20)

## Plan
- [x] Reproduce from issue screenshot: cream Agent Ops attention glances on Data Poster
- [x] Identify root cause: `.ops-*-attention-glance` still mix Apple `#ffffff` washes
- [x] Remap all 15 Agent Ops attention glances onto `#0b0b10` for Data Poster / Dark
- [x] Sync `src/agent-ops.css` → `src-tauri/dist/agent-ops.css`
- [x] Bump version, CHANGELOG, layout_daily selectors
- [x] `cargo check` in `src-tauri/`
- [x] Rename `WIP-20-…` → `UNTESTED-20-…`
- [x] Commit + push `origin/main` (do not close GitHub #20)

## Review
- Root cause: Agent Ops attention glances kept Apple `#ffffff` opaque washes; on Data Poster they rendered as cream slabs (issue screenshot).
- Fix: theme remaps for resting/hover/focus + fail/slow/warn/not-set/filter states onto `#0b0b10`.
- Left GitHub #20 open for tester / 004.
