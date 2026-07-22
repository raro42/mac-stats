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

cp -f "$BIN_SRC" "$APP/Contents/MacOS/mac_stats"
# LaunchAgent / older docs may refer to mac-stats; keep a symlink after DMG installs (CFBundleExecutable is mac_stats).
ln -sfn mac_stats "$APP/Contents/MacOS/mac-stats"
if [[ -d "$DIST_SRC" && -d "$DIST_DST" ]]; then
  # Live UI assets for the menu-bar CPU window (dashboard.* is orphaned — do not ship)
  for f in ollama.js cpu.js cpu.html cpu-ui.js discord.js history.js chart-line.js \
           tauri-logger.js agent-ops.js agent-ops.css; do
    [[ -f "$DIST_SRC/$f" ]] && cp -f "$DIST_SRC/$f" "$DIST_DST/"
  done
  # Remove previously installed orphaned dashboard copies if present
  rm -f "$DIST_DST/dashboard.html" "$DIST_DST/dashboard.js" "$DIST_DST/dashboard.css"
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

# Prefer CFBundleExecutable name; fall back to legacy mac-stats symlink.
PLIST="$HOME/Library/LaunchAgents/com.raro42.mac-stats.plist"
if [[ -f "$PLIST" ]]; then
  /usr/libexec/PlistBuddy -c 'Set :ProgramArguments:0 /Applications/mac-stats.app/Contents/MacOS/mac_stats' "$PLIST" 2>/dev/null || true
fi

# codesign --deep can hang on some macOS builds (overnight harness saw multi-minute stalls).
# Time out and continue — binary is already replaced; LaunchAgent restart still proceeds.
CODESIGN_TIMEOUT_SECS="${MAC_STATS_CODESIGN_TIMEOUT_SECS:-45}"
codesign -s - --force --deep "$APP" &
cs_pid=$!
(
  sleep "$CODESIGN_TIMEOUT_SECS"
  if kill -0 "$cs_pid" 2>/dev/null; then
    echo "codesign still running after ${CODESIGN_TIMEOUT_SECS}s — killing and continuing" >&2
    kill "$cs_pid" 2>/dev/null || true
  fi
) &
waiter_pid=$!
set +e
wait "$cs_pid"
cs_status=$?
set -e
kill "$waiter_pid" 2>/dev/null || true
wait "$waiter_pid" 2>/dev/null || true
if [[ "$cs_status" -ne 0 ]]; then
  echo "codesign did not complete cleanly (exit $cs_status) — app binary was still updated" >&2
fi

xattr -dr com.apple.quarantine "$APP" 2>/dev/null || true

PLIST_PATH="${HOME}/Library/LaunchAgents/com.raro42.mac-stats.plist"
# kickstart only works if the agent is already bootstrapped in the GUI domain.
# After reboot / bootout / fresh machine, bootstrap first (idempotent enough with bootout||true).
if ! launchctl print "$LABEL" >/dev/null 2>&1; then
  echo "LaunchAgent not loaded — bootstrapping $PLIST_PATH" >&2
  if [[ -f "$PLIST_PATH" ]]; then
    launchctl bootout "gui/$(id -u)" "$PLIST_PATH" 2>/dev/null || true
    launchctl bootstrap "gui/$(id -u)" "$PLIST_PATH"
    launchctl enable "$LABEL" 2>/dev/null || true
  else
    echo "Missing $PLIST_PATH — start mac-stats manually or recreate the LaunchAgent" >&2
  fi
fi

launchctl kickstart -k "$LABEL" 2>/dev/null || {
  echo "kickstart failed; trying bootstrap+kickstart" >&2
  if [[ -f "$PLIST_PATH" ]]; then
    launchctl bootout "gui/$(id -u)" "$PLIST_PATH" 2>/dev/null || true
    launchctl bootstrap "gui/$(id -u)" "$PLIST_PATH"
    launchctl kickstart -k "$LABEL"
  fi
}
sleep 3
pgrep -fl 'Contents/MacOS/mac_stats|Contents/MacOS/mac-stats' | head -1 || {
  echo "Process not running after kickstart" >&2
  exit 1
}
echo "Installed and restarted. Check: rg 'Bot connected' ~/.mac-stats/debug.log | tail -1"
