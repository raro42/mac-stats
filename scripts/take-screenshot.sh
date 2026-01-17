#!/bin/bash

# Script to take a screenshot and save it to screen-what-i-see folder

SCREENSHOT_DIR="screen-what-i-see"
TIMESTAMP=$(date +"%Y%m%d-%H%M%S")
FILENAME="${SCREENSHOT_DIR}/screenshot-${TIMESTAMP}.png"

# Create directory if it doesn't exist
mkdir -p "${SCREENSHOT_DIR}"

# Take screenshot (full screen)
# -x = no sound, -t png = PNG format
screencapture -x -t png "${FILENAME}"

if [ $? -eq 0 ]; then
    echo "Screenshot saved to: ${FILENAME}"
    open "${SCREENSHOT_DIR}"
else
    echo "Failed to take screenshot"
    exit 1
fi
