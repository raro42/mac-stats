#!/bin/bash
# Quick wrapper script for CPU comparison monitoring

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PYTHON_SCRIPT="${SCRIPT_DIR}/monitor_cpu_comparison.py"

# Default values
DURATION=${1:-60}      # How long to monitor after warmup
INTERVAL=${2:-1}       # Sampling interval
WARMUP=${3:-30}        # Warmup/stabilization period

# Parse flags
AUTO_KILL=""
KEEP_RUNNING=""
DEBUG=""

shift 3 2>/dev/null || true
while [[ $# -gt 0 ]]; do
    case $1 in
        --auto-kill)
            AUTO_KILL="--auto-kill"
            shift
            ;;
        --keep-running)
            KEEP_RUNNING="--keep-running"
            shift
            ;;
        --kill-after)
            KEEP_RUNNING="--kill-after"
            shift
            ;;
        --debug)
            DEBUG="--debug"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [duration] [interval] [warmup] [--auto-kill] [--keep-running|--kill-after] [--debug]"
            exit 1
            ;;
    esac
done

# Check if Python 3 is available
if ! command -v python3 &> /dev/null; then
    echo "Error: python3 is required but not found"
    exit 1
fi

# Run the Python script
python3 "$PYTHON_SCRIPT" "$DURATION" "$INTERVAL" "$WARMUP" $AUTO_KILL $KEEP_RUNNING $DEBUG
