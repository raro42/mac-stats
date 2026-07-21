#!/usr/bin/env bash
# Notarize a signed DMG (requires Apple Developer credentials in the environment).
# Does nothing useful without APPLE_ID / APPLE_TEAM_ID / APPLE_APP_SPECIFIC_PASSWORD.
set -euo pipefail

DMG="${1:-}"
if [[ -z "$DMG" || ! -f "$DMG" ]]; then
  echo "Usage: $0 path/to/mac-stats_*.dmg" >&2
  exit 1
fi

: "${APPLE_ID:?Set APPLE_ID}"
: "${APPLE_TEAM_ID:?Set APPLE_TEAM_ID}"
: "${APPLE_APP_SPECIFIC_PASSWORD:?Set APPLE_APP_SPECIFIC_PASSWORD}"

echo "Submitting $DMG to Apple notary service…"
xcrun notarytool submit "$DMG" \
  --apple-id "$APPLE_ID" \
  --team-id "$APPLE_TEAM_ID" \
  --password "$APPLE_APP_SPECIFIC_PASSWORD" \
  --wait

echo "Stapling…"
xcrun stapler staple "$DMG"
xcrun stapler validate "$DMG"
echo "Done. Distribute this DMG; Gatekeeper should accept it without xattr hacks."
