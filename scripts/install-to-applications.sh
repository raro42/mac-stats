#!/usr/bin/env bash
# Install release binary + frontend dist into /Applications/mac-stats.app and restart LaunchAgent.
# Always deep-signs the full .app — signing only the Mach-O can exit with OS_REASON_CODESIGNING.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP="/Applications/mac-stats.app"
BIN_SRC="$ROOT/src-tauri/target/release/mac_stats"
DIST_SRC="$ROOT/src-tauri/dist"
LABEL="gui/$(id -u)/com.raro42.mac-stats"

if [[ ! -x "$BIN_SRC" ]]; then
  echo "Missing $BIN_SRC — run: cd src-tauri && cargo build --release" >&2
  exit 1
fi
if [[ ! -d "$APP/Contents/MacOS" ]]; then
  echo "Missing $APP — install the .app bundle first" >&2
  exit 1
fi

cp -f "$BIN_SRC" "$APP/Contents/MacOS/mac-stats"
if [[ -d "$DIST_SRC" && -d "$APP/Contents/Resources/dist" ]]; then
  cp -f "$DIST_SRC"/dashboard.html "$DIST_SRC"/dashboard.js "$DIST_SRC"/dashboard.css \
    "$DIST_SRC"/ollama.js "$APP/Contents/Resources/dist/" 2>/dev/null || true
fi

codesign -s - --force --deep "$APP"
xattr -dr com.apple.quarantine "$APP" 2>/dev/null || true

launchctl kickstart -k "$LABEL"
sleep 3
pgrep -fl 'Contents/MacOS/mac-stats' | head -1 || {
  echo "Process not running after kickstart" >&2
  exit 1
}
echo "Installed and restarted. Check: rg 'Bot connected' ~/.mac-stats/debug.log | tail -1"
