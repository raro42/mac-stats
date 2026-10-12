# WIP-16: changelog does not close

- [x] Find changelog popover open/close handlers (X + ESC)
- [x] Fix close on X click and Escape
- [x] `cargo check` in `src-tauri/`
- [x] Rename WIP-16 → UNTESTED-16
- [x] Commit + push origin/main (do not close GitHub issue)

## Review
- Cause: footer version click in `injectAppVersion` opened the modal with `stopPropagation`, so `ensureChangelogModalWired` never ran and Close/Escape stayed unwired.
- Fix: call `ensureChangelogModalWired()` from `openChangelogModal` (and the inject path); Escape looks up the live modal.
- Ship: v0.1.1893 on `main` (`43b3069e`). Issue #16 left open for tester.
