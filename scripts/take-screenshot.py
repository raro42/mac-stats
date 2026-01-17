#!/usr/bin/env python3
"""
Script to take a screenshot of the CPU application window and save it to screen-what-i-see folder

This script will:
1. Try to automatically capture if the window is frontmost
2. Otherwise, prompt you to click on the window
"""

import os
import subprocess
from datetime import datetime
import time

SCREENSHOT_DIR = "screen-what-i-see"
TIMESTAMP = datetime.now().strftime("%Y%m%d-%H%M%S")
FILENAME = f"{SCREENSHOT_DIR}/screenshot-{TIMESTAMP}.png"

# Create directory if it doesn't exist
os.makedirs(SCREENSHOT_DIR, exist_ok=True)

print("Capturing CPU window...")
print("If a crosshair appears, click on the CPU application window")
print("(You have 8 seconds)")

try:
    # Use -w to capture window (waits for click on window)
    # If window is already selected/frontmost, user can click immediately
    result = subprocess.run(
        ["screencapture", "-x", "-w", "-t", "png", FILENAME],
        timeout=8
    )
    
    if result.returncode == 0:
        print(f"\n✓ Window screenshot saved to: {FILENAME}")
        subprocess.run(["open", SCREENSHOT_DIR])
    else:
        print("\nCapture cancelled, using full screen fallback...")
        subprocess.run(["screencapture", "-x", "-t", "png", FILENAME])
        print(f"Screenshot saved to: {FILENAME}")
        
except subprocess.TimeoutExpired:
    print("\nTimeout - no window clicked, using full screen capture...")
    subprocess.run(["screencapture", "-x", "-t", "png", FILENAME])
    print(f"Screenshot saved to: {FILENAME}")
except KeyboardInterrupt:
    print("\nCancelled by user")
except Exception as e:
    print(f"Error: {e}")
    subprocess.run(["screencapture", "-x", "-t", "png", FILENAME])
    print(f"Fallback screenshot saved to: {FILENAME}")
