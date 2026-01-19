#!/bin/bash
# Sync source files to dist directory for Tauri development

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "Syncing files from src/ to dist/..."

# Create dist directory if it doesn't exist
mkdir -p "$PROJECT_ROOT/dist"

# Copy HTML, CSS, JS files
cp "$PROJECT_ROOT/src"/*.html "$PROJECT_ROOT/dist/" 2>/dev/null || true
cp "$PROJECT_ROOT/src"/*.css "$PROJECT_ROOT/dist/" 2>/dev/null || true
cp "$PROJECT_ROOT/src"/*.js "$PROJECT_ROOT/dist/" 2>/dev/null || true

# Copy assets directory
if [ -d "$PROJECT_ROOT/src/assets" ]; then
    cp -r "$PROJECT_ROOT/src/assets" "$PROJECT_ROOT/dist/" 2>/dev/null || true
fi

# Also sync to src-tauri/dist/ (where Tauri actually serves from)
if [ -d "$PROJECT_ROOT/src-tauri" ]; then
    echo "Syncing files to src-tauri/dist/..."
    mkdir -p "$PROJECT_ROOT/src-tauri/dist"
    
    # Copy HTML, CSS, JS files to src-tauri/dist/
    cp "$PROJECT_ROOT/src"/*.html "$PROJECT_ROOT/src-tauri/dist/" 2>/dev/null || true
    cp "$PROJECT_ROOT/src"/*.css "$PROJECT_ROOT/src-tauri/dist/" 2>/dev/null || true
    cp "$PROJECT_ROOT/src"/*.js "$PROJECT_ROOT/src-tauri/dist/" 2>/dev/null || true
    
    # Copy assets directory
    if [ -d "$PROJECT_ROOT/src/assets" ]; then
        cp -r "$PROJECT_ROOT/src/assets" "$PROJECT_ROOT/src-tauri/dist/" 2>/dev/null || true
    fi
fi

echo "✓ Files synced to dist/ and src-tauri/dist/"
