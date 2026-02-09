#!/bin/zsh
# Build script for creating DMG file.
# DMG version is taken from src-tauri/Cargo.toml (tauri.conf.json has no version set).
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
DMG_DIR="target/release/bundle/dmg"
DMG_FILES=($DMG_DIR/mac-stats_*.dmg(N))

if [[ ${#DMG_FILES[@]} -gt 0 ]]; then
    DMG_FILE="${DMG_FILES[1]}"
    echo ""
    echo "✅ DMG created successfully!"
    echo "Location: $DMG_FILE"
    ls -lh "$DMG_FILE"
else
    echo "❌ DMG not found. Build may have failed."
    exit 1
fi
