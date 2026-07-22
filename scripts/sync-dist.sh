#!/bin/bash
# Sync frontend source files from src/ → src-tauri/dist/ (Tauri frontendDist).
# Themes live only under src-tauri/dist/themes/ and are never overwritten from src/.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DEST="$PROJECT_ROOT/src-tauri/dist"

if [[ ! -d "$PROJECT_ROOT/src-tauri" ]]; then
  echo "error: src-tauri/ not found" >&2
  exit 1
fi

echo "Syncing src/ → src-tauri/dist/ ..."
mkdir -p "$DEST"

# Shared UI (CPU window + Agent Ops + Ollama). Do not sync orphaned dashboard.* here.
for f in \
  cpu.js cpu-ui.js discord.js history.js \
  ollama.js tauri-logger.js agent-ops.js agent-ops.css \
  cpu.html index.html main.js styles.css; do
  if [[ -f "$PROJECT_ROOT/src/$f" ]]; then
    cp "$PROJECT_ROOT/src/$f" "$DEST/"
  fi
done

# Optional assets
if [[ -d "$PROJECT_ROOT/src/assets" ]]; then
  mkdir -p "$DEST/assets"
  rsync -a "$PROJECT_ROOT/src/assets/" "$DEST/assets/"
fi

# Shared line-chart module (themes import ../../chart-line.js)
if [[ -f "$PROJECT_ROOT/src/chart-line.js" ]]; then
  cp "$PROJECT_ROOT/src/chart-line.js" "$DEST/"
fi

echo "✓ Synced to src-tauri/dist/ (themes untouched; dashboard.* not synced)"
