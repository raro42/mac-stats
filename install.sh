#!/usr/bin/env bash
# mac-stats one-liner installer (Apple Silicon).
#
#   curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/main/install.sh | bash
#
# Does: Homebrew tap + trust (Homebrew 6+) + cask install when brew exists,
# otherwise downloads the latest arm64 DMG from GitHub Releases.
# Always clears Gatekeeper quarantine on /Applications/mac-stats.app (until notarized).
#
# Env:
#   MAC_STATS_NO_OPEN=1       skip `open -a mac-stats`
#   MAC_STATS_NO_BREW=1       force DMG path even if brew exists
#   MAC_STATS_NO_AI=1         do not auto-enable AI even if Ollama is running
#   MAC_STATS_REPO=owner/repo default raro42/mac-stats
#   OLLAMA_HOST               Ollama base URL (default http://127.0.0.1:11434)
set -euo pipefail

REPO="${MAC_STATS_REPO:-raro42/mac-stats}"
APP="/Applications/mac-stats.app"
TAP="raro42/mac-stats"
TAP_URL="https://github.com/${REPO}"
RAW="https://raw.githubusercontent.com/${REPO}/main"
API="https://api.github.com/repos/${REPO}/releases/latest"

log()  { printf '==> %s\n' "$*"; }
warn() { printf 'warning: %s\n' "$*" >&2; }
die()  { printf 'error: %s\n' "$*" >&2; exit 1; }

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "need '$1' on PATH"
}

clear_quarantine() {
  if [[ -d "$APP" ]]; then
    /usr/bin/xattr -dr com.apple.quarantine "$APP" 2>/dev/null || true
    log "Cleared Gatekeeper quarantine on $APP"
  fi
}

# True when the local Ollama HTTP API answers (default 127.0.0.1:11434).
ollama_is_running() {
  local host="${OLLAMA_HOST:-http://127.0.0.1:11434}"
  # OLLAMA_HOST may be host:port without scheme.
  case "$host" in
    http://*|https://*) ;;
    *) host="http://${host}" ;;
  esac
  host="${host%/}"
  curl -fsS --connect-timeout 1 --max-time 2 "${host}/api/tags" >/dev/null 2>&1
}

# Merge aiAgentEnabled into ~/.mac-stats/config.json (create from minimal if missing).
set_ai_agent_enabled() {
  local enabled="$1" # true|false
  local cfg_dir="${HOME}/.mac-stats"
  local cfg="${cfg_dir}/config.json"
  mkdir -p "$cfg_dir"

  if [[ ! -f "$cfg" ]]; then
    if ! curl -fsSL "${RAW}/config.minimal.json" -o "$cfg"; then
      warn "Could not download config.minimal.json — writing minimal stub"
      printf '%s\n' '{"aiAgentEnabled":false,"menuBarCompact":true,"windowDecorations":true}' >"$cfg"
    fi
  fi

  ENABLED="$enabled" CFG="$cfg" python3 - <<'PY'
import json, os, pathlib, sys
path = pathlib.Path(os.environ["CFG"])
enabled = os.environ["ENABLED"].lower() == "true"
try:
    data = json.loads(path.read_text(encoding="utf-8"))
except Exception as e:
    sys.stderr.write(f"config parse failed: {e}\n")
    sys.exit(1)
if not isinstance(data, dict):
    sys.stderr.write("config.json is not an object\n")
    sys.exit(1)
prev = data.get("aiAgentEnabled")
data["aiAgentEnabled"] = enabled
tmp = path.with_suffix(".json.tmp")
tmp.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
tmp.replace(path)
print(f"prev={prev!r}")
print(f"now={enabled}")
PY
}

seed_home_config() {
  local cfg="${HOME}/.mac-stats"
  local cfg_json="${cfg}/config.json"
  mkdir -p "$cfg"

  local enable_ai=false
  if [[ "${MAC_STATS_NO_AI:-}" == "1" ]]; then
    log "MAC_STATS_NO_AI=1 — leaving AI disabled"
  elif ollama_is_running; then
    log "Ollama API is reachable — enabling local AI agent (aiAgentEnabled=true)"
    enable_ai=true
  else
    log "Ollama not running — AI stays off (start Ollama later, then enable in Settings)"
  fi

  if [[ ! -f "$cfg_json" ]]; then
    if [[ "$enable_ai" == "true" ]]; then
      set_ai_agent_enabled true >/dev/null
      log "Wrote $cfg_json with aiAgentEnabled=true"
    else
      if curl -fsSL "${RAW}/config.minimal.json" -o "$cfg_json"; then
        log "Wrote $cfg_json (monitor-only defaults; AI off)"
      else
        warn "Could not download config.minimal.json — app still runs with built-in defaults"
        rm -f "$cfg_json"
      fi
    fi
  else
    if [[ "$enable_ai" == "true" ]]; then
      local out
      out="$(set_ai_agent_enabled true | tr '\n' ' ')"
      log "Updated $cfg_json ($out)"
      log "AI stack starts automatically (config watcher) — no restart needed"
    else
      log "Keeping existing $cfg_json"
    fi
  fi
}

install_via_brew() {
  need_cmd brew
  log "Using Homebrew"

  if ! brew tap | grep -qx "$TAP"; then
    log "Tapping $TAP ($TAP_URL)"
    brew tap "$TAP" "$TAP_URL"
  else
    log "Tap $TAP already present"
  fi

  # Homebrew 6+: third-party taps refuse to load until trusted.
  if brew trust --help >/dev/null 2>&1; then
    log "Trusting cask ${TAP}/mac-stats (Homebrew tap trust)"
    brew trust --cask "${TAP}/mac-stats" >/dev/null
  fi

  if brew list --cask mac-stats >/dev/null 2>&1; then
    log "Upgrading cask mac-stats"
    brew upgrade --cask mac-stats
  else
    log "Installing cask mac-stats"
    brew install --cask mac-stats
  fi
}

resolve_latest_dmg() {
  curl -fsSL "$API" | python3 -c '
import json, sys
data = json.load(sys.stdin)
assets = [a for a in data.get("assets") or [] if str(a.get("name", "")).endswith("_aarch64.dmg")]
if not assets:
    assets = [a for a in data.get("assets") or [] if str(a.get("name", "")).endswith(".dmg")]
if not assets:
    sys.stderr.write("no DMG asset on latest release\n")
    sys.exit(1)
a = assets[0]
print(a["browser_download_url"])
print(a["name"])
print(data.get("tag_name") or "")
'
}

install_via_dmg() {
  need_cmd curl
  need_cmd python3
  need_cmd hdiutil
  need_cmd ditto

  log "Homebrew not used — installing from GitHub Releases DMG"

  local url name tag tmp mount attached=0
  {
    read -r url
    read -r name
    read -r tag
  } < <(resolve_latest_dmg)

  [[ -n "${url}" && -n "${name}" ]] || die "Could not resolve latest DMG from GitHub Releases"

  log "Downloading ${name} (${tag})"
  tmp="$(mktemp -d "${TMPDIR:-/tmp}/mac-stats-install.XXXXXX")"
  cleanup() {
    if [[ "${attached}" == "1" && -n "${mount:-}" ]]; then
      hdiutil detach -quiet "$mount" 2>/dev/null || true
    fi
    rm -rf "$tmp"
  }
  trap cleanup EXIT

  curl -fL --progress-bar -o "${tmp}/${name}" "$url"

  log "Mounting DMG"
  mount="$(hdiutil attach -nobrowse -readonly "${tmp}/${name}" | awk 'END { print $NF }')"
  attached=1
  [[ -d "${mount}/mac-stats.app" ]] || die "DMG missing mac-stats.app (mount=${mount})"

  log "Installing to $APP"
  rm -rf "$APP"
  ditto "${mount}/mac-stats.app" "$APP"
  hdiutil detach -quiet "$mount" || true
  attached=0
  trap - EXIT
  rm -rf "$tmp"
}

# --- main ---

main() {
  [[ "$(uname -s)" == "Darwin" ]] || die "mac-stats requires macOS"
  [[ "$(uname -m)" == "arm64" ]] || die "mac-stats requires Apple Silicon (arm64); Intel is not supported"

  log "mac-stats installer (repo ${REPO})"

  if [[ "${MAC_STATS_NO_BREW:-}" != "1" ]] && command -v brew >/dev/null 2>&1; then
    install_via_brew
  else
    if [[ "${MAC_STATS_NO_BREW:-}" == "1" ]]; then
      log "MAC_STATS_NO_BREW=1 — skipping Homebrew"
    else
      warn "brew not found — falling back to DMG install"
      warn "Install Homebrew from https://brew.sh for easier upgrades"
    fi
    install_via_dmg
  fi

  [[ -d "$APP" ]] || die "Install finished but $APP is missing"
  clear_quarantine
  seed_home_config

  if [[ "${MAC_STATS_NO_OPEN:-}" == "1" ]]; then
    log "Skipping open (MAC_STATS_NO_OPEN=1)"
  else
    log "Launching mac-stats"
    open -a mac-stats || warn "open -a mac-stats failed — launch from Applications"
  fi

  log "Done. Menu bar should show CPU (and °C when known)."
  if [[ -f "${HOME}/.mac-stats/config.json" ]] && grep -q '"aiAgentEnabled"[[:space:]]*:[[:space:]]*true' "${HOME}/.mac-stats/config.json"; then
    log "AI agent enabled (Ollama was running at install time)."
  else
    log "AI stays off until Settings → Enable local AI agent (or re-run install with Ollama up)."
  fi
  log "If Finder still says “damaged”: xattr -rd com.apple.quarantine ${APP}"
}

if [[ "${BASH_SOURCE[0]-}" == "$0" ]]; then
  main "$@"
fi
