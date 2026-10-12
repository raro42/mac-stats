# WIP-17 — update banner brew hint

## Plan
- [x] Fix in-app update banner: do not imply every install can `brew upgrade --cask mac-stats`
- [x] Lead with Release notes / DMG; brew only if Homebrew cask install
- [x] Align README Updates + docs/homebrew.md
- [x] sync-dist.sh; cargo check in src-tauri/
- [x] Commit + push; rename WIP-17 → UNTESTED-17; do not close #17

## Review
- Root cause: banner always showed `brew upgrade --cask mac-stats`; DMG / install.sh users have no cask.
- Fix: banner + README + homebrew.md clarify DMG path vs Homebrew-only upgrade.
- v0.1.1894. Issue #17 left open for tester / 004.
