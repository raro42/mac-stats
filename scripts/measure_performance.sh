#!/bin/bash

# mac-stats Performance Measurement Script
# Measures CPU, GPU, RAM, and other metrics for mac-stats process
# Usage: ./measure_performance.sh [duration_seconds] [interval_seconds] [mode]

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

# Ensure app is running
if ! pgrep -f "mac_stats" > /dev/null; then
    echo -e "${RED}Error: mac_stats is not running${NC}"
    echo "Start the app first:"
    echo "  ./target/release/mac_stats --cpu  # With window open"
    echo "  ./target/release/mac_stats        # Idle (menu bar only)"
    exit 1
fi

# Get PID
PID=$(pgrep -f "mac_stats" | head -1)

# Output file
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
OUTPUT_FILE="performance_${MODE}_${TIMESTAMP}.txt"
CSV_FILE="performance_${MODE}_${TIMESTAMP}.csv"

echo -e "${BLUE}════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  mac-stats Performance Measurement${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════${NC}"
echo ""
echo "Configuration:"
echo "  Process: mac_stats (PID: $PID)"
echo "  Mode: $MODE"
echo "  Duration: ${DURATION}s"
echo "  Interval: ${INTERVAL}s"
echo "  Output: $OUTPUT_FILE"
echo "  CSV: $CSV_FILE"
echo ""

# Create headers
{
    echo "=== mac-stats Performance Measurement ==="
    echo "Date: $(date)"
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

    # Get metrics using ps
    metrics=$(ps -p $PID -o %cpu=,%mem=,rss=,vsz=,nlwp= 2>/dev/null || echo "0 0 0 0 0")

    cpu=$(echo $metrics | awk '{print $1}')
    mem=$(echo $metrics | awk '{print $2}')
    rss=$(echo $metrics | awk '{print $3}')
    vsz=$(echo $metrics | awk '{print $4}')
    threads=$(echo $metrics | awk '{print $5}')

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

    # Sleep between measurements
    sleep "$INTERVAL"
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
