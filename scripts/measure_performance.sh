#!/bin/bash

# Performance Measurement Script
# Measures CPU/RAM for mac-stats or exelban Stats (same sampling method).
# Usage: ./measure_performance.sh [duration_seconds] [interval_seconds] [mode] [target]
#   target: mac-stats (default) | stats

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DURATION=${1:-30}  # Default: 30 seconds
INTERVAL=${2:-1}   # Default: 1 second between measurements
MODE=${3:-"window"}  # "idle" or "window" (window = CPU window open)
# Target: mac-stats (default) | stats (exelban Stats.app)
TARGET=${4:-${MEASURE_TARGET:-mac-stats}}

resolve_pid_mac_stats() {
    local prefer_cpu=0
    [[ "$MODE" == "window" ]] && prefer_cpu=1
    local pids
    pids=$(pgrep -f '/Contents/MacOS/mac_stats|/target/release/mac_stats' || true)
    if [[ -z "$pids" ]]; then
        pgrep -x mac_stats || true
        return
    fi
    local best="" fallback=""
    local p args
    for p in $pids; do
        args=$(ps -p "$p" -o args= 2>/dev/null || true)
        if [[ "$args" == *"--cpu"* ]]; then
            if [[ "$prefer_cpu" -eq 1 ]]; then
                echo "$p"
                return
            fi
            fallback=${fallback:-$p}
        else
            if [[ "$prefer_cpu" -eq 0 ]]; then
                echo "$p"
                return
            fi
            fallback=${fallback:-$p}
        fi
        best=${best:-$p}
    done
    echo "${fallback:-$best}"
}

resolve_pid_stats() {
    # exelban Stats — main binary only (not widget extensions / MonitorStats)
    pgrep -f '/Applications/Stats\.app/Contents/MacOS/Stats$' \
        || pgrep -x Stats \
        || true
}

case "$TARGET" in
    mac-stats|mac_stats)
        TARGET_LABEL="mac-stats"
        if ! pgrep -f '/Contents/MacOS/mac_stats|/target/release/mac_stats' >/dev/null \
            && ! pgrep -x mac_stats >/dev/null; then
            echo -e "${RED}Error: mac_stats is not running${NC}"
            echo "Start the app first (menu bar or --cpu)."
            exit 1
        fi
        PID=$(resolve_pid_mac_stats)
        ;;
    stats|Stats|exelban-stats)
        TARGET_LABEL="stats"
        if ! pgrep -f '/Applications/Stats\.app/Contents/MacOS/Stats' >/dev/null \
            && ! pgrep -x Stats >/dev/null; then
            echo -e "${RED}Error: Stats.app is not running${NC}"
            echo "Start it: open -a Stats"
            exit 1
        fi
        PID=$(resolve_pid_stats | head -1)
        ;;
    *)
        echo -e "${RED}Unknown target: $TARGET (use mac-stats or stats)${NC}"
        exit 1
        ;;
esac

if [ -z "${PID}" ]; then
    echo -e "${RED}Error: could not resolve PID for $TARGET_LABEL${NC}"
    exit 1
fi
echo "Resolved PID: $PID  ($(ps -p "$PID" -o args= 2>/dev/null | tr -s ' '))"

# Darwin vs Linux sampling:
# - macOS `ps %cpu` is lifetime average (useless for a long-lived menu-bar app).
# - Prefer `top -l 2 -s 1` for an interval sample on Darwin.
IS_DARWIN=0
[[ "$(uname -s)" == "Darwin" ]] && IS_DARWIN=1

# Output file (repo root; gitignored)
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
OUTPUT_FILE="performance_${TARGET_LABEL}_${MODE}_${TIMESTAMP}.txt"
CSV_FILE="performance_${TARGET_LABEL}_${MODE}_${TIMESTAMP}.csv"

echo -e "${BLUE}════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Performance Measurement ($TARGET_LABEL)${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════${NC}"
echo ""
echo "Configuration:"
echo "  Process: $TARGET_LABEL (PID: $PID)"
echo "  Mode: $MODE"
echo "  Duration: ${DURATION}s"
echo "  Interval: ${INTERVAL}s"
echo "  Output: $OUTPUT_FILE"
echo "  CSV: $CSV_FILE"
echo ""

# Create headers
{
    echo "=== Performance Measurement ($TARGET_LABEL) ==="
    echo "Date: $(date)"
    echo "Target: $TARGET_LABEL"
    echo "Mode: $MODE"
    echo "Duration: ${DURATION}s"
    echo "Interval: ${INTERVAL}s"
    echo "PID: $PID"
    echo ""
    echo "Measurements over time:"
    echo "Timestamp | CPU(%) | Threads | RSS(MB) | VSZ(MB) | MEM(%) |"
} > "$OUTPUT_FILE"

# Create CSV header
{
    echo "timestamp,cpu_percent,threads,rss_mb,vsz_mb,mem_percent"
} > "$CSV_FILE"

# Measurement arrays
cpu_values=()
mem_values=()
rss_values=()
threads_values=()

echo -e "${YELLOW}Measuring...${NC}"
echo ""

# Measure for specified duration
start_time=$(date +%s)
measurement_count=0

while true; do
    current_time=$(date +%s)
    elapsed=$((current_time - start_time))

    if [ $elapsed -ge $DURATION ]; then
        break
    fi

    # Memory / VSZ from ps (KB)
    mem_metrics=$(ps -p "$PID" -o %mem=,rss=,vsz= 2>/dev/null || echo "0 0 0")
    mem=$(echo "$mem_metrics" | awk '{print $1}')
    rss=$(echo "$mem_metrics" | awk '{print $2}')
    vsz=$(echo "$mem_metrics" | awk '{print $3}')

    # Threads
    if [[ "$IS_DARWIN" -eq 1 ]]; then
        threads=$(ps -M -p "$PID" 2>/dev/null | tail -n +2 | wc -l | tr -d ' ')
    else
        threads=$(ps -p "$PID" -o nlwp= 2>/dev/null | tr -d ' ')
        threads=${threads:-0}
    fi

    # CPU: interval sample on Darwin; ps %cpu elsewhere
    if [[ "$IS_DARWIN" -eq 1 ]]; then
        # Second sample of top is the interval measurement (-s uses INTERVAL when >=1)
        top_s="$INTERVAL"
        [[ "$top_s" -lt 1 ]] && top_s=1
        cpu=$(top -l 2 -s "$top_s" -pid "$PID" -stats pid,cpu 2>/dev/null \
            | awk -v pid="$PID" '$1 == pid { cpu=$2 } END { if (cpu == "") print 0; else print cpu }')
    else
        cpu=$(ps -p "$PID" -o %cpu= 2>/dev/null | tr -d ' ')
        cpu=${cpu:-0}
        sleep "$INTERVAL"
    fi

    # Convert KB to MB
    rss_mb=$(echo "scale=1; $rss / 1024" | bc)
    vsz_mb=$(echo "scale=1; $vsz / 1024" | bc)

    # Store values
    cpu_values+=("$cpu")
    mem_values+=("$mem")
    rss_values+=("$rss_mb")
    threads_values+=("$threads")

    # Print live output
    timestamp=$(date '+%H:%M:%S')
    printf "%s | %5.1f%% | %7d | %7.1f | %7.1f | %5.1f%% |\n" \
        "$timestamp" "$cpu" "$threads" "$rss_mb" "$vsz_mb" "$mem" \
        | tee -a "$OUTPUT_FILE"

    # Append to CSV
    echo "$timestamp,$cpu,$threads,$rss_mb,$vsz_mb,$mem" >> "$CSV_FILE"

    measurement_count=$((measurement_count + 1))
    # Darwin path already waited via top -s; Linux slept above
done

echo ""
echo -e "${YELLOW}Measurement complete. Calculating statistics...${NC}"
echo ""

# Calculate statistics
calc_avg() {
    echo "$@" | awk '{sum=0; for(i=1;i<=NF;i++) sum+=$i; print sum/NF}'
}

calc_min() {
    echo "$@" | awk '{min=$1; for(i=2;i<=NF;i++) if($i<min) min=$i; print min}'
}

calc_max() {
    echo "$@" | awk '{max=$1; for(i=2;i<=NF;i++) if($i>max) max=$i; print max}'
}

# Convert array to space-separated string
cpu_str=$(printf '%s ' "${cpu_values[@]}")
mem_str=$(printf '%s ' "${mem_values[@]}")
rss_str=$(printf '%s ' "${rss_values[@]}")
threads_str=$(printf '%s ' "${threads_values[@]}")

# Calculate statistics
cpu_avg=$(calc_avg $cpu_str)
cpu_min=$(calc_min $cpu_str)
cpu_max=$(calc_max $cpu_str)

mem_avg=$(calc_avg $mem_str)
mem_min=$(calc_min $mem_str)
mem_max=$(calc_max $mem_str)

rss_avg=$(calc_avg $rss_str)
rss_min=$(calc_min $rss_str)
rss_max=$(calc_max $rss_str)

threads_avg=$(calc_avg $threads_str)
threads_min=$(calc_min $threads_str)
threads_max=$(calc_max $threads_str)

# Print summary
{
    echo ""
    echo "=== Summary Statistics ==="
    echo ""
    echo "CPU Usage:"
    echo "  Average: ${cpu_avg}%"
    echo "  Min: ${cpu_min}%"
    echo "  Max: ${cpu_max}%"
    echo ""
    echo "Memory:"
    echo "  Average: ${mem_avg}%"
    echo "  Min: ${mem_min}%"
    echo "  Max: ${mem_max}%"
    echo ""
    echo "RSS (Resident Set Size):"
    echo "  Average: ${rss_avg} MB"
    echo "  Min: ${rss_min} MB"
    echo "  Max: ${rss_max} MB"
    echo ""
    echo "Threads:"
    echo "  Average: ${threads_avg}"
    echo "  Min: ${threads_min}"
    echo "  Max: ${threads_max}"
    echo ""
    echo "Measurements: $measurement_count samples over ${DURATION}s"
} | tee -a "$OUTPUT_FILE"

# Print summary to console
echo ""
echo -e "${GREEN}════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}  RESULTS SUMMARY${NC}"
echo -e "${GREEN}════════════════════════════════════════════════════${NC}"
echo ""
printf "CPU Usage:     %6.2f%% (min: %5.2f%%, max: %5.2f%%)\n" "$cpu_avg" "$cpu_min" "$cpu_max"
printf "Memory:        %6.2f%% (min: %5.2f%%, max: %5.2f%%)\n" "$mem_avg" "$mem_min" "$mem_max"
printf "RSS:           %6.1f MB (min: %5.1f MB, max: %5.1f MB)\n" "$rss_avg" "$rss_min" "$rss_max"
printf "Threads:       %6.0f (min: %5.0f, max: %5.0f)\n" "$threads_avg" "$threads_min" "$threads_max"
echo ""
echo "Measurements: $measurement_count samples"
echo "Duration: ${DURATION}s"
echo "Interval: ${INTERVAL}s"
echo ""
echo -e "${GREEN}════════════════════════════════════════════════════${NC}"
echo ""

# GPU usage (if available via system_profiler)
echo -e "${YELLOW}Checking GPU usage...${NC}"
if command -v system_profiler &> /dev/null; then
    gpu_info=$(system_profiler SPDisplaysDataType 2>/dev/null | grep -i "used" || echo "GPU info: Not available")
    echo "GPU: $gpu_info"
else
    echo "GPU: system_profiler not available"
fi

echo ""
echo "Files saved:"
echo "  Text: $OUTPUT_FILE"
echo "  CSV:  $CSV_FILE"
echo ""
echo -e "${GREEN}✅ Measurement complete!${NC}"
