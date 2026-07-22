#!/usr/bin/env bash
# Fail if #settings-modal markup differs across themes (normalized whitespace).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
THEMES="$ROOT/src-tauri/dist/themes"
REF="$THEMES/apple/cpu.html"

python3 - "$REF" "$THEMES" <<'PY'
import hashlib, re, sys
from pathlib import Path

ref_path = Path(sys.argv[1])
themes = Path(sys.argv[2])

def extract(html: str) -> str:
    m = re.search(
        r'<div id="settings-modal"[\s\S]*?</div>\s*(?=<div id="process-details-modal")',
        html,
    )
    if not m:
        raise SystemExit(f"settings-modal not found")
    # collapse whitespace for comparison
    return re.sub(r"\s+", " ", m.group(0)).strip()

ref = extract(ref_path.read_text())
ref_hash = hashlib.sha256(ref.encode()).hexdigest()[:12]
failed = []
for html in sorted(themes.glob("*/cpu.html")):
    if html.parent.name == "apple":
        continue
    try:
        body = extract(html.read_text())
    except SystemExit as e:
        failed.append(f"{html.parent.name}: {e}")
        continue
    h = hashlib.sha256(body.encode()).hexdigest()[:12]
    if body != ref:
        failed.append(f"{html.parent.name}: hash {h} != apple {ref_hash}")

if failed:
    print("Theme settings-modal drift detected:")
    for f in failed:
        print(" ", f)
    print("Fix: scripts/sync-theme-settings.sh")
    sys.exit(1)
print(f"OK: settings-modal in sync across themes (apple {ref_hash})")
PY
