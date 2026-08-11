# Homebrew / Cask

mac-stats is **not** in the official Homebrew cask catalogue. A bare
`brew install --cask mac-stats` will fail with “No Cask with this name exists”
(and may suggest unrelated `mac-sai`). Use the one-liner or tap first.

## Recommended: one-liner

```bash
curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/main/install.sh | bash
```

[`install.sh`](../install.sh) taps [raro42/homebrew-mac-stats](https://github.com/raro42/homebrew-mac-stats), runs Homebrew 6 trust, installs the cask (or DMG fallback), clears Gatekeeper quarantine, and opens the app. If Ollama is already serving locally, it turns on `aiAgentEnabled`.

## Manual Homebrew

```bash
brew tap raro42/mac-stats
brew trust --cask raro42/mac-stats/mac-stats   # Homebrew 6+
brew install --cask mac-stats
xattr -rd com.apple.quarantine /Applications/mac-stats.app
open -a mac-stats
```

Requires **Apple Silicon** (the published DMG is `aarch64` only).

Tap remote: https://github.com/raro42/homebrew-mac-stats  
(`brew tap raro42/mac-stats` maps to that repo by Homebrew convention.)

## Troubleshooting

### `Cask 'mac-stats' is unavailable: No Cask with this name exists`

You ran `brew install --cask mac-stats` **without** tapping. That only searches Homebrew’s official casks. Fix:

```bash
curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/main/install.sh | bash
```

or the three manual steps above. Do **not** install `mac-sai` — that is a different app.

### `Refusing to start from untrusted tap` / `Refusing to load cask … from untrusted tap`

Homebrew 6+ requires an explicit trust step:

```bash
brew trust --cask raro42/mac-stats/mac-stats
brew install --cask mac-stats
```

Docs: [Homebrew Tap Trust](https://docs.brew.sh/Tap-Trust).

## Updating the cask after a release

1. Publish a GitHub Release with `mac-stats_<version>_aarch64.dmg`.
2. Run `./scripts/print-release-checksums.sh v<version>`.
3. Update `Casks/mac-stats.rb` and `homebrew-tap/Casks/mac-stats.rb` (`version` + `sha256`).
4. Commit and push this repo, then sync the dedicated tap:

```bash
./scripts/sync-homebrew-tap.sh
```

## Gatekeeper

`brew install` installs the same **ad-hoc / not-notarized** `.app` as the GitHub DMG. Recent macOS may still say the app is **“damaged”** after a successful brew install.

```bash
xattr -rd com.apple.quarantine /Applications/mac-stats.app
open -a mac-stats
```

Or Right-click → **Open** once. The cask `postflight` clears quarantine on install; run `xattr` by hand if an older cask was used. Full notes: [NOTARIZATION.md](NOTARIZATION.md).
