#!/usr/bin/env bash
# Install release binary + frontend dist into /Applications/mac-stats.app and restart LaunchAgent.
# Always deep-signs the full .app — signing only the Mach-O can exit with OS_REASON_CODESIGNING.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP="/Applications/mac-stats.app"
BIN_SRC="$ROOT/src-tauri/target/release/mac_stats"
DIST_SRC="$ROOT/src-tauri/dist"
DIST_DST="$APP/Contents/Resources/dist"
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
if [[ -d "$DIST_SRC" && -d "$DIST_DST" ]]; then
  # Root UI assets (dashboard is unused by the menu-bar window but kept in sync)
  for f in dashboard.html dashboard.js dashboard.css \
           ollama.js cpu.js cpu.html cpu-ui.js discord.js \
           tauri-logger.js agent-ops.js agent-ops.css; do
    [[ -f "$DIST_SRC/$f" ]] && cp -f "$DIST_SRC/$f" "$DIST_DST/"
  done
  # Themes power the real CPU window (cpu.html → themes/<theme>/cpu.html)
  if [[ -d "$DIST_SRC/themes" ]]; then
    rsync -a --delete "$DIST_SRC/themes/" "$DIST_DST/themes/"
  fi
  if [[ -d "$DIST_SRC/assets" ]]; then
    rsync -a "$DIST_SRC/assets/" "$DIST_DST/assets/"
  fi
fi

# LaunchAgent cwd is not src-tauri — merge REDMINE/Brave/Perplexity into ~/.mac-stats/.config.env
bash "$ROOT/scripts/sync-home-config-env.sh" || true

codesign -s - --force --deep "$APP"
xattr -dr com.apple.quarantine "$APP" 2>/dev/null || true

launchctl kickstart -k "$LABEL"
sleep 3
pgrep -fl 'Contents/MacOS/mac-stats' | head -1 || {
  echo "Process not running after kickstart" >&2
  exit 1
}
echo "Installed and restarted. Check: rg 'Bot connected' ~/.mac-stats/debug.log | tail -1"
