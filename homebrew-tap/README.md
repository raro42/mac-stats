# Homebrew tap stub (mirrored to https://github.com/raro42/homebrew-mac-stats)

Preferred install:

```bash
curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/main/install.sh | bash
```

Manual:

```bash
brew tap raro42/mac-stats
brew trust --cask raro42/mac-stats/mac-stats
brew install --cask mac-stats
```

Do **not** run only `brew install --cask mac-stats` (not in official Homebrew casks).

After a release, update `Casks/mac-stats.rb` here and in `../Casks/mac-stats.rb`, then:

```bash
./scripts/sync-homebrew-tap.sh
```

See [docs/homebrew.md](../docs/homebrew.md).
