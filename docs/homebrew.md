# Homebrew / Cask

Install mac-stats like other Mac apps:

```bash
brew tap raro42/mac-stats https://github.com/raro42/mac-stats
brew install --cask mac-stats
```

Requires **Apple Silicon** (the published DMG is `aarch64` only).

## Updating the cask after a release

1. Publish a GitHub Release with `mac-stats_<version>_aarch64.dmg`.
2. Run `./scripts/print-release-checksums.sh v<version>`.
3. Update `Casks/mac-stats.rb` (`version` + `sha256`).
4. Commit and push (or sync into a dedicated `homebrew-mac-stats` tap repo).

## Gatekeeper

Prefer a **notarized** DMG (see [NOTARIZATION.md](NOTARIZATION.md)). Until notarization secrets are configured in CI, first launch may still need Right-click → Open once.
