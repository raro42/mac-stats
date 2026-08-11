This folder is the in-repo Homebrew tap layout.

Preferred install:

```bash
curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/main/install.sh | bash
```

Manual tap install: see [docs/homebrew.md](../docs/homebrew.md).

Or copy `Casks/mac-stats.rb` into a dedicated `homebrew-mac-stats` repository (standard tap name: `raro42/homebrew-mac-stats`).

Update `version` and `sha256` whenever you publish a new DMG release (`scripts/print-release-checksums.sh`).
