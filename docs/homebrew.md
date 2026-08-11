# Homebrew / Cask

## Recommended: one-liner

```bash
curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/main/install.sh | bash
```

[`install.sh`](../install.sh) runs tap + Homebrew 6 trust + cask install (or DMG fallback), clears Gatekeeper quarantine, and opens the app.

## Manual Homebrew

```bash
brew tap raro42/mac-stats https://github.com/raro42/mac-stats
brew trust --cask raro42/mac-stats/mac-stats
brew install --cask mac-stats
xattr -rd com.apple.quarantine /Applications/mac-stats.app
open -a mac-stats
```

Requires **Apple Silicon** (the published DMG is `aarch64` only).

## Homebrew 6+ tap trust

Since Homebrew 6.0, non-official taps are **untrusted by default**. Without trust, install fails with:

```text
Refusing to start from untrusted tap raro42/mac-stats
```

Fix: trust only this cask (preferred), then install by short name:

```bash
brew trust --cask raro42/mac-stats/mac-stats
brew install --cask mac-stats
```

Or trust the whole tap: `brew trust raro42/mac-stats`.

You must tap with the **repo URL** (`https://github.com/raro42/mac-stats`). A plain `brew install --cask raro42/mac-stats/mac-stats` without that tap looks for a missing `homebrew-mac-stats` repo.

Docs: [Homebrew Tap Trust](https://docs.brew.sh/Tap-Trust).

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
