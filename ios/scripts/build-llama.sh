#!/usr/bin/env bash
# Builds llama.xcframework (iPhone + simulator) from a pinned llama.cpp tag.
# The XCFramework in the official releases only includes ios-device and macos, so
# the simulator could not link against it.
set -euo pipefail

TAG="b11321"
COMMIT="b0aca3c6539e2dd55ea510bbb79591852a4d4b81"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/vendor/llama.cpp"
OUT="$ROOT/vendor/llama.xcframework"

command -v cmake >/dev/null || { echo "cmake is missing: brew install cmake" >&2; exit 1; }

if [ ! -d "$SRC/.git" ]; then
  git clone --quiet --depth 1 --branch "$TAG" https://github.com/ggml-org/llama.cpp.git "$SRC"
fi
actual="$(git -C "$SRC" rev-parse HEAD)"
if [ "$actual" != "$COMMIT" ]; then
  echo "Commit mismatch for $TAG: $actual (expected $COMMIT)" >&2
  exit 1
fi

(cd "$SRC" && ./build-xcframework.sh ios-device ios-sim)

rm -rf "$OUT"
cp -R "$SRC/build-apple/llama.xcframework" "$OUT"

# llama.framework is dynamic and ships without a privacy manifest; App Store Connect
# requires one inside every framework that uses required-reason APIs (fstat).
for fw in "$OUT"/*/llama.framework; do
  cp "$ROOT/vendor/PrivacyInfo.llama.xcprivacy" "$fw/PrivacyInfo.xcprivacy"
done

printf 'tag=%s\ncommit=%s\n' "$TAG" "$COMMIT" > "$ROOT/vendor/llama.lock"
echo "Done: $OUT"
ls "$OUT"
