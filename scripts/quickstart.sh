#!/usr/bin/env bash
# One-click Quick Start: seed ~/.mac-stats, check Ollama, optionally install to /Applications.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
HOME_CFG="${HOME}/.mac-stats"
mkdir -p "$HOME_CFG"

echo "==> mac-stats Quick Start (Apple Silicon monitor; AI optional)"

if [[ ! -f "$HOME_CFG/config.json" ]]; then
  cp "$ROOT/config.minimal.json" "$HOME_CFG/config.json"
  echo "    Wrote $HOME_CFG/config.json (monitor-only defaults)"
else
  echo "    Keeping existing $HOME_CFG/config.json"
fi

if [[ ! -f "$HOME_CFG/schedules.json" ]]; then
  cp "$ROOT/src-tauri/defaults/schedules.example.json" "$HOME_CFG/schedules.json"
  echo "    Seeded schedules.json template (disabled/example)"
fi

if [[ ! -f "$HOME_CFG/discord_channels.json" ]]; then
  cp "$ROOT/src-tauri/defaults/discord_channels.example.json" "$HOME_CFG/discord_channels.json"
  echo "    Seeded discord_channels.json template"
fi

if command -v ollama >/dev/null 2>&1; then
  echo "    Ollama: found ($(ollama --version 2>/dev/null | head -1 || echo ok))"
  if ! ollama list 2>/dev/null | tail -n +2 | grep -q .; then
    echo "    Tip: ollama pull llama3.2   # only needed for AI path"
  fi
else
  echo "    Ollama: not installed (OK for monitor-only). AI path: https://ollama.com"
fi

if [[ -x "$ROOT/scripts/install-to-applications.sh" ]]; then
  read -r -p "Install/update /Applications/mac-stats.app from this tree? [y/N] " ans || true
  if [[ "${ans:-}" =~ ^[Yy]$ ]]; then
    "$ROOT/scripts/install-to-applications.sh"
  else
    echo "    Skip install. Use: brew install --cask mac-stats   or ./scripts/install-to-applications.sh"
  fi
else
  echo "    Install: brew install --cask mac-stats"
fi

echo "Done. Open the app → menu bar. Enable AI later in Settings if you want Ollama/Discord."
