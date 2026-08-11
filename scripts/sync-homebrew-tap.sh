#!/usr/bin/env bash
# Push Casks/mac-stats.rb from this monorepo into raro42/homebrew-mac-stats.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/Casks/mac-stats.rb"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/mac-stats-tap.XXXXXX")"
cleanup() { rm -rf "$TMP"; }
trap cleanup EXIT

[[ -f "$SRC" ]] || { echo "missing $SRC" >&2; exit 1; }

git clone --depth 1 git@github.com:raro42/homebrew-mac-stats.git "$TMP/tap"
mkdir -p "$TMP/tap/Casks"
cp "$SRC" "$TMP/tap/Casks/mac-stats.rb"
# Keep tap README if present; refresh from monorepo stub when useful
if [[ -f "$ROOT/homebrew-tap/README.md" ]]; then
  cp "$ROOT/homebrew-tap/README.md" "$TMP/tap/README.md"
fi

cd "$TMP/tap"
git add Casks/mac-stats.rb README.md
if git diff --cached --quiet; then
  echo "Tap already up to date."
  exit 0
fi
ver="$(grep -E '^\s*version "' Casks/mac-stats.rb | head -1 | sed -E 's/.*"([^"]+)".*/\1/')"
git commit -m "Bump mac-stats cask to v${ver}."
git push origin HEAD
echo "Pushed https://github.com/raro42/homebrew-mac-stats (v${ver})"
