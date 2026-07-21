# Notarization & code signing

## Current status

GitHub Actions ([`.github/workflows/release.yml`](../.github/workflows/release.yml)) **builds a DMG** and can **sign** when these repository secrets are set:

| Secret | Purpose |
|--------|---------|
| `APPLE_CERTIFICATE` | Base64 `.p12` Developer ID Application cert |
| `APPLE_CERTIFICATE_PASSWORD` | Password for the `.p12` |
| `KEYCHAIN_PASSWORD` | Optional temp keychain password |
| `APPLE_ID` / `APPLE_TEAM_ID` / `APPLE_APP_SPECIFIC_PASSWORD` | For `notarytool` (add when ready) |

If secrets are **missing**, CI still ships an **unsigned** DMG. Prefer **Right-click → Open** on first launch — not random `xattr` advice from the web. Unsigned ≠ malicious; it means Apple Developer credentials are not in CI yet.

## Goal

1. **Sign** with Developer ID Application.
2. **Notarize** with `xcrun notarytool submit … --wait` (CI step when `APPLE_ID` / `APPLE_TEAM_ID` / `APPLE_APP_SPECIFIC_PASSWORD` are set).
3. **Staple** the ticket to the DMG / `.app`.

Release notes then state: **Notarized for macOS Sequoia+**.

Helper script (run on a Mac with credentials loaded):

```bash
./scripts/notarize-dmg.sh path/to/mac-stats_X.Y.Z_aarch64.dmg
```

## Users today

- Prefer **Homebrew cask** or Right-click → **Open** on first launch.
- `xattr -rd com.apple.quarantine /Applications/mac-stats.app` only as a last resort after verifying the download from [GitHub Releases](https://github.com/raro42/mac-stats/releases).
