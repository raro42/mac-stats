# Notarization & code signing

## Current status

GitHub Actions ([`.github/workflows/release.yml`](../.github/workflows/release.yml)) **builds a DMG** and can **sign** when these repository secrets are set:

| Secret | Purpose |
|--------|---------|
| `APPLE_CERTIFICATE` | Base64 `.p12` Developer ID Application cert |
| `APPLE_CERTIFICATE_PASSWORD` | Password for the `.p12` |
| `KEYCHAIN_PASSWORD` | Optional temp keychain password |
| `APPLE_ID` / `APPLE_TEAM_ID` / `APPLE_APP_SPECIFIC_PASSWORD` | For `notarytool` (add when ready) |

If secrets are **missing**, CI still ships an **ad-hoc / unsigned** DMG. On Sequoia+, Gatekeeper often says the app is **“damaged”** — including after `brew install --cask`. That is quarantine + missing notarization, not a corrupt file.

## Users today

1. After brew or DMG install, run:

```bash
xattr -rd com.apple.quarantine /Applications/mac-stats.app
open -a mac-stats
```

2. Or **Right-click** → **Open** on `mac-stats.app` once.

The Homebrew cask `postflight` also clears quarantine; if an older cask is cached, run the `xattr` line by hand. Do not disable Gatekeeper system-wide.

Unsigned ≠ malicious. The maintainer does not have an Apple Developer account to sign and notarize builds yet — help is welcome (see secrets table above).

## Goal

1. **Sign** with Developer ID Application.
2. **Notarize** with `xcrun notarytool submit … --wait` (CI step when `APPLE_ID` / `APPLE_TEAM_ID` / `APPLE_APP_SPECIFIC_PASSWORD` are set).
3. **Staple** the ticket to the DMG / `.app`.

Release notes then state: **Notarized for macOS Sequoia+**.

Helper script (run on a Mac with credentials loaded):

```bash
./scripts/notarize-dmg.sh path/to/mac-stats_X.Y.Z_aarch64.dmg
```
