# WIP-14 (v0.1.1443) — collapsed Monitors localStorage icon

- [x] Pick lowest GitHub FEAT/WIP (`WIP-14`)
- [x] Implement: collapsed Monitors skips `list_monitor_statuses`; paint from localStorage
- [x] `cargo check` in `src-tauri/` (pass; pre-existing warnings only)
- [x] Rename `WIP-14-…` → `UNTESTED-14-…`
- [ ] Commit + push `origin/main` (do not close #14)

## Review
Shipped v0.1.1443: collapsed Monitors paints last-known icon wash from localStorage (no `list_monitor_statuses` / hourly poll). Expand still hydrates. Hand off to tester; leave GitHub #14 open.
