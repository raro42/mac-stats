#!/usr/bin/env bash
# Merge selected secrets from repo-local .config.env into ~/.mac-stats/.config.env
# so LaunchAgent /Applications launches see the same keys as `cargo run` from src-tauri.
#
# Does not print secret values. Safe to re-run (idempotent merge).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="${MAC_STATS_CONFIG_ENV_SRC:-$ROOT/src-tauri/.config.env}"
DST="${HOME}/.mac-stats/.config.env"

KEYS=(
  REDMINE_URL
  REDMINE_API_KEY
  BRAVE_API_KEY
  PERPLEXITY_API_KEY
)

if [[ ! -f "$SRC" ]]; then
  echo "sync-home-config-env: no source at $SRC (skip)"
  exit 0
fi

mkdir -p "$(dirname "$DST")"
python3 - "$SRC" "$DST" "${KEYS[@]}" <<'PY'
import sys
from pathlib import Path

src_path = Path(sys.argv[1])
dst_path = Path(sys.argv[2])
keys = sys.argv[3:]

def parse(path: Path) -> dict[str, str]:
    out: dict[str, str] = {}
    if not path.is_file():
        return out
    for line in path.read_text().splitlines():
        t = line.strip()
        if not t or t.startswith("#") or "=" not in t:
            continue
        k, v = t.split("=", 1)
        out[k.strip()] = v.strip()
    return out

src = parse(src_path)
need = {k: src[k] for k in keys if k in src and src[k].strip()}
if not need:
    print("sync-home-config-env: nothing to merge")
    raise SystemExit(0)

raw = dst_path.read_text() if dst_path.is_file() else ""
dst = parse(dst_path)
wrote = []
for k, v in need.items():
    if dst.get(k) == v:
        continue
    lines = [
        ln
        for ln in raw.splitlines()
        if not ln.strip().startswith(k + "=")
    ]
    raw = "\n".join(lines).rstrip() + "\n"
    if "# Synced from src-tauri/.config.env" not in raw:
        raw += "\n# Synced from src-tauri/.config.env (for /Applications + LaunchAgent)\n"
    raw += f"{k}={v}\n"
    wrote.append(k)

if not wrote:
    print("sync-home-config-env: already up to date")
    raise SystemExit(0)

dst_path.write_text(raw if raw.endswith("\n") else raw + "\n")
print("sync-home-config-env: updated " + ", ".join(wrote))
PY
