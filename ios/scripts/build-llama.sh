#!/usr/bin/env bash
# Compila llama.xcframework (iPhone + simulador) desde una etiqueta fijada de llama.cpp.
# El XCFramework de las releases oficiales solo incluye ios-device y macos, así que
# el simulador no podría enlazarlo.
set -euo pipefail

TAG="b11321"
COMMIT="b0aca3c6539e2dd55ea510bbb79591852a4d4b81"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/vendor/llama.cpp"
OUT="$ROOT/vendor/llama.xcframework"

command -v cmake >/dev/null || { echo "Falta cmake: brew install cmake" >&2; exit 1; }

if [ ! -d "$SRC/.git" ]; then
  git clone --quiet --depth 1 --branch "$TAG" https://github.com/ggml-org/llama.cpp.git "$SRC"
fi
actual="$(git -C "$SRC" rev-parse HEAD)"
if [ "$actual" != "$COMMIT" ]; then
  echo "El commit de $TAG no coincide: $actual (esperado $COMMIT)" >&2
  exit 1
fi

(cd "$SRC" && ./build-xcframework.sh ios-device ios-sim)

rm -rf "$OUT"
cp -R "$SRC/build-apple/llama.xcframework" "$OUT"

# llama.framework es dinámico y no trae manifiesto de privacidad; App Store Connect
# exige uno dentro de cada framework que use APIs con motivo declarado (fstat).
for fw in "$OUT"/*/llama.framework; do
  cp "$ROOT/vendor/PrivacyInfo.llama.xcprivacy" "$fw/PrivacyInfo.xcprivacy"
done

printf 'tag=%s\ncommit=%s\n' "$TAG" "$COMMIT" > "$ROOT/vendor/llama.lock"
echo "Listo: $OUT"
ls "$OUT"
