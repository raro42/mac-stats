#!/bin/zsh
# Test DMG file before release
# This script mounts the DMG, installs the app, and runs basic checks

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT/src-tauri"

# Find the DMG file
DMG_DIR="target/release/bundle/dmg"
DMG_FILES=($DMG_DIR/mac-stats_*.dmg(N))

if [[ ${#DMG_FILES[@]} -eq 0 ]]; then
    echo "❌ No DMG file found. Build it first with: ./scripts/build-dmg.sh"
    exit 1
fi

DMG_FILE="${DMG_FILES[1]}"
echo "📦 Testing DMG: $DMG_FILE"
echo ""

# Remove quarantine attribute (if present)
echo "🔓 Removing quarantine attribute..."
xattr -d com.apple.quarantine "$DMG_FILE" 2>/dev/null || true

# Create a temporary mount point
MOUNT_POINT=$(mktemp -d)
echo "📂 Mount point: $MOUNT_POINT"
echo ""

# Mount the DMG
echo "💿 Mounting DMG..."
hdiutil attach "$DMG_FILE" -mountpoint "$MOUNT_POINT" -quiet

# Find the app
APP_PATH="$MOUNT_POINT/mac-stats.app"

if [[ ! -d "$APP_PATH" ]]; then
    echo "❌ App not found in DMG at: $APP_PATH"
    hdiutil detach "$MOUNT_POINT" -quiet
    exit 1
fi

echo "✅ App found in DMG"
echo ""

# Check app structure
echo "🔍 Checking app structure..."
if [[ -d "$APP_PATH/Contents/MacOS" ]]; then
    echo "  ✅ MacOS directory exists"
else
    echo "  ❌ MacOS directory missing"
fi

if [[ -d "$APP_PATH/Contents/Resources" ]]; then
    echo "  ✅ Resources directory exists"
    
    # Check for assets (they're in dist/assets/)
    if [[ -f "$APP_PATH/Contents/Resources/dist/assets/ollama.svg" ]]; then
        echo "  ✅ Ollama icon found"
    else
        echo "  ❌ Ollama icon missing!"
    fi
    
    # Check for theme chart.js files
    THEME_CHARTS=$(find "$APP_PATH/Contents/Resources/themes" -name "chart.js" 2>/dev/null | wc -l)
    if [[ $THEME_CHARTS -gt 0 ]]; then
        echo "  ✅ Found $THEME_CHARTS chart.js files in themes"
    else
        echo "  ❌ No chart.js files found in themes!"
    fi
else
    echo "  ❌ Resources directory missing"
fi

echo ""

# Unmount DMG
echo "💿 Unmounting DMG..."
hdiutil detach "$MOUNT_POINT" -quiet

# Install to a test location
TEST_INSTALL_DIR="$HOME/.mac-stats-test"
mkdir -p "$TEST_INSTALL_DIR"

echo "📥 Installing app to test location: $TEST_INSTALL_DIR"
cp -R "$APP_PATH" "$TEST_INSTALL_DIR/"

# Remove quarantine from installed app
echo "🔓 Removing quarantine from installed app..."
xattr -rd com.apple.quarantine "$TEST_INSTALL_DIR/mac-stats.app" 2>/dev/null || true

INSTALLED_APP="$TEST_INSTALL_DIR/mac-stats.app/Contents/MacOS/mac_stats"

if [[ ! -f "$INSTALLED_APP" ]]; then
    echo "❌ App binary not found after installation"
    exit 1
fi

echo ""
echo "✅ DMG structure looks good!"
echo ""
echo "🧪 To test the app:"
echo "   1. Open the DMG: open \"$DMG_FILE\""
echo "   2. Drag mac-stats.app to Applications (or test location)"
echo "   3. Run: \"$INSTALLED_APP\" --cpu"
echo "   4. Check:"
echo "      - History charts are visible and drawing"
echo "      - Ollama icon appears in the icon line"
echo "      - No console errors in browser devtools (Cmd+Option+I)"
echo ""
echo "💡 To test from installed app:"
echo "   \"$INSTALLED_APP\" --cpu"
echo ""
echo "💡 To check console logs:"
echo "   \"$INSTALLED_APP\" -vvv --cpu 2>&1 | tee /tmp/mac-stats-test.log"
echo ""
echo "🧹 Cleanup test installation:"
echo "   rm -rf \"$TEST_INSTALL_DIR\""
