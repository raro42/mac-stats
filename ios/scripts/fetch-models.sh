#!/usr/bin/env bash
# Downloads the models in models/catalog.json to the Mac (URLs pinned to a Hugging Face
# commit) and verifies their SHA-256. Usage: scripts/fetch-models.sh [id ...]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIR="$ROOT/models"

entries() {
  python3 - "$DIR/catalog.json" "$@" <<'PY'
import json, sys
wanted = set(sys.argv[2:])
for m in json.load(open(sys.argv[1])):
    if wanted and m["id"] not in wanted:
        continue
    url = f"https://huggingface.co/{m['repo']}/resolve/{m['commit']}/{m['file']}"
    print(f"{url}\t{m['file']}\t{m['sha256']}")
PY
}

sha_of() { shasum -a 256 "$1" | cut -d' ' -f1; }

while IFS=$'\t' read -r url file sha; do
  dest="$DIR/$file"
  if [ -f "$dest" ] && [ "$(sha_of "$dest")" = "$sha" ]; then
    echo "OK (already present) $file"
    continue
  fi
  echo "Downloading ${file}…"
  curl -L --fail --retry 3 -C - -o "$dest.part" "$url"
  got="$(sha_of "$dest.part")"
  if [ "$got" != "$sha" ]; then
    echo "SHA-256 mismatch for $file: $got" >&2
    rm -f "$dest.part"
    exit 1
  fi
  mv "$dest.part" "$dest"
  echo "OK $file"
done < <(entries "$@")
