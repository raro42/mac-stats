# WIP-21: Update returns error

## Plan
- [x] Read issue #21 and locate update / Homebrew upgrade messaging
- [x] Reproduce root cause from code (brew upgrade error path)
- [x] Implement minimal fix
- [x] `cargo check` in `src-tauri/`
- [x] Rename `WIP-21-…` → `UNTESTED-21-…`
- [ ] Commit and push `origin/main` (do not close GitHub issue)

## Notes
GitHub: https://github.com/raro42/mac-stats/issues/21
Banner always offered brew; DMG → “Cask not installed.” Also Homebrew 7 deprecations on cask.

## Review
- v0.1.2352: brew hint gated on Caskroom; cask `postflight_steps` + no `verified:`.
- `cargo check` PASS; unit test `detects_staged_caskroom_dir` PASS.
- Issue #21 left open for tester / 004.
