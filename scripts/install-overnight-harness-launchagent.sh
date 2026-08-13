#!/usr/bin/env bash
# Install LaunchAgent so the overnight harness survives IDE/session exit.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LABEL=com.raro42.mac-stats-overnight-harness
SRC="$ROOT/scripts/${LABEL}.plist"
DST="$HOME/Library/LaunchAgents/${LABEL}.plist"
mkdir -p "$HOME/Library/LaunchAgents" "$HOME/.mac-stats/improvements"

# Render plist with this machine's paths (template uses raro42 defaults; rewrite HOME/ROOT).
python3 - <<PY
from pathlib import Path
import os
home = Path.home()
root = Path("$ROOT")
text = Path("$SRC").read_text()
# Replace common hardcoded paths if present
text = text.replace("/Users/raro42/projects/mac-stats", str(root))
text = text.replace("/Users/raro42", str(home))
# Prefer Homebrew python3 if available
import shutil
py = shutil.which("python3") or "/usr/bin/python3"
text = text.replace("<string>/usr/bin/python3</string>", f"<string>{py}</string>", 1)
Path("$DST").write_text(text)
print(f"wrote {Path('$DST')}")
print(f"python={py}")
PY

UID_NUM="$(id -u)"
launchctl bootout "gui/${UID_NUM}/${LABEL}" 2>/dev/null || true
# Also stop any ad-hoc nohup copy so KeepAlive owns the single instance
pkill -f 'run_overnight_harness_loop.py' 2>/dev/null || true
sleep 1
launchctl bootstrap "gui/${UID_NUM}" "$DST"
launchctl kickstart -k "gui/${UID_NUM}/${LABEL}"
sleep 1
launchctl print "gui/${UID_NUM}/${LABEL}" 2>&1 | head -25
pgrep -fl run_overnight_harness_loop || { echo "ERROR: harness not running"; exit 1; }
echo "OK: overnight harness LaunchAgent loaded ($LABEL)"
