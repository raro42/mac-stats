# raro42/homebrew-mac-stats (tap stub)

This folder is the in-repo Homebrew tap layout. To install from the monorepo:

```bash
brew tap raro42/mac-stats https://github.com/raro42/mac-stats
brew install --cask mac-stats
```

Or copy `Casks/mac-stats.rb` into a dedicated `homebrew-mac-stats` repository (standard tap name: `raro42/homebrew-mac-stats`).

Update `version` and `sha256` whenever you publish a new DMG release (`scripts/print-release-checksums.sh`).
