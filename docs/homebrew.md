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

`brew install` installs the same **ad-hoc / not-notarized** `.app` as the GitHub DMG. Recent macOS may still say the app is **“damaged”** after a successful brew install.

```bash
xattr -rd com.apple.quarantine /Applications/mac-stats.app
open -a mac-stats
```

Or Right-click → **Open** once. The cask `postflight` clears quarantine on install; run `xattr` by hand if an older cask was used. Full notes: [NOTARIZATION.md](NOTARIZATION.md).

