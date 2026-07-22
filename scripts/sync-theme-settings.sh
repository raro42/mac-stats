#!/usr/bin/env bash
# Copy #settings-modal from apple theme into all other theme cpu.html files.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
THEMES="$ROOT/src-tauri/dist/themes"

python3 - "$THEMES" <<'PY'
import re, sys
from pathlib import Path

themes = Path(sys.argv[1])
apple = (themes / "apple" / "cpu.html").read_text()
m = re.search(
    r'(    <div id="settings-modal"[\s\S]*?</div>\n\n)(?=    <div id="process-details-modal")',
    apple,
)
if not m:
    raise SystemExit("could not extract apple settings-modal")
block = m.group(1)

for html_path in sorted(themes.glob("*/cpu.html")):
    if html_path.parent.name == "apple":
        continue
    text = html_path.read_text()
    new, n = re.subn(
        r'    <div id="settings-modal"[\s\S]*?</div>\n\n(?=    <div id="process-details-modal")',
        block,
        text,
        count=1,
    )
    if n != 1:
        new, n = re.subn(
            r'    <div id="settings-modal"[\s\S]*?</div>\n(?=    <div id="process-details-modal")',
            block,
            text,
            count=1,
        )
    if n != 1:
        print("FAIL", html_path.parent.name)
        sys.exit(1)
    html_path.write_text(new)
    print("synced", html_path.parent.name)
print("done")
PY

bash "$ROOT/scripts/check-theme-settings-sync.sh"
