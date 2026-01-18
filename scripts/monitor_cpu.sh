#!/bin/bash
# Quick wrapper script for CPU comparison monitoring

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PYTHON_SCRIPT="${SCRIPT_DIR}/monitor_cpu_comparison.py"

# Default values
DURATION=${1:-60}
INTERVAL=${2:-1}

# Check if Python 3 is available
if ! command -v python3 &> /dev/null; then
    echo "Error: python3 is required but not found"
    exit 1
fi

# Run the Python script
python3 "$PYTHON_SCRIPT" "$DURATION" "$INTERVAL"
