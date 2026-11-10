# FEAT-25: Remove first horizontal line below icons

## Plan
- [x] Read issue #25 + task file (hairline under icon row)
- [x] Remove first post-icon-line divider/rule in theme `cpu.html` files (skip futuristic)
- [x] Bump version + CHANGELOG, `cargo check` / ratchet verify
- [x] Rename `FEAT-25-…` → `UNTESTED-25-…`
- [x] Commit and push `origin/main` (do not close GitHub issue)

## Notes
GitHub: https://github.com/raro42/mac-stats/issues/25
Cause: each theme painted a divider (`apple-divider` / `poster-rule` / …) immediately under `#icon-line`.
Fix: drop that first rule so the icon row sits flush above Monitors / empty space.

## Review
- v0.1.2567: first under-icon hairline removed (#25).
- Issue #25 left open for tester / 004.
