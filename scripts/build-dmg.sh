#!/bin/zsh
# Build script for creating DMG file
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT/src-tauri"

echo "Building mac-stats DMG..."
echo ""

# Ensure dist directory exists
if [[ ! -d "dist" ]]; then
    echo "Error: dist directory not found. Frontend files need to be built first."
    exit 1
fi

# Build the DMG
cargo tauri build --bundles dmg

# Show the result
DMG_PATH="target/release/bundle/dmg/mac-stats_*.dmg"
if ls $DMG_PATH 1> /dev/null 2>&1; then
    echo ""
    echo "✅ DMG created successfully!"
    echo "Location: $(ls -1 $DMG_PATH | head -1)"
    ls -lh $DMG_PATH | head -1
else
    echo "❌ DMG not found. Build may have failed."
    exit 1
fi
