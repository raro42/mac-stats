#!/usr/bin/env bash
# Print SHA256 for a GitHub release DMG (for Homebrew cask updates).
set -euo pipefail
TAG="${1:-}"
if [[ -z "$TAG" ]]; then
  echo "Usage: $0 v0.1.88" >&2
  exit 1
fi
ASSET=$(gh release view "$TAG" --json assets -q '.assets[] | select(.name | endswith(".dmg")) | .name' | head -1)
if [[ -z "$ASSET" ]]; then
  echo "No DMG on release $TAG" >&2
  exit 1
fi
DIGEST=$(gh release view "$TAG" --json assets -q ".assets[] | select(.name==\"$ASSET\") | .digest")
echo "tag=$TAG"
echo "asset=$ASSET"
echo "sha256=${DIGEST#sha256:}"
echo ""
echo "Paste into Casks/mac-stats.rb:"
echo "  version \"${TAG#v}\""
echo "  sha256 arm: \"${DIGEST#sha256:}\""
