# Development & Measurement Scripts

This directory contains utility scripts for building, testing, and measuring mac-stats performance.

---

## Performance Measurement

### `measure_performance.sh` - CPU, GPU, RAM Tracker

Measures real-time performance metrics for mac-stats process.

**Usage**:
```bash
# Measure with CPU window open (30 seconds)
./scripts/measure_performance.sh 30 1 window

# Measure idle (menu bar only, 60 seconds)
./scripts/measure_performance.sh 60 1 idle

# Custom: 120 seconds, 2-second intervals
./scripts/measure_performance.sh 120 2 idle
```

**Parameters**:
- `duration` (seconds): How long to measure [default: 30]
- `interval` (seconds): Measurement interval [default: 1]
- `mode` (string): "window" or "idle" [default: "window"]

**Measures**:
- CPU usage (%)
- Memory usage (%)
- RSS (Resident Set Size) in MB
- VSZ (Virtual memory) in MB
- Thread count
- GPU usage (if available)

**Output**:
- `performance_window_YYYYMMDD_HHMMSS.txt` - Detailed text report
- `performance_window_YYYYMMDD_HHMMSS.csv` - Spreadsheet-compatible CSV

**Example Workflow**:
```bash
# Start app with window
./target/release/mac_stats --cpu &

# Take baseline measurement
./scripts/measure_performance.sh 30 1 window

# Output: performance_window_20260118_180000.txt

# Apply optimizations...

# Measure again
./scripts/measure_performance.sh 30 1 window

# Compare results: Activity Monitor shows CPU reduction
```

---

## CPU Comparison Monitoring

### `monitor_cpu_comparison.py` - Compare mac_stats vs Stats App

Monitors CPU usage of both `mac_stats` and `Stats` apps simultaneously for side-by-side comparison. Perfect for demonstrating that mac_stats uses less CPU.

**Usage**:
```bash
# Monitor for 60 seconds (default), 1-second intervals
./scripts/monitor_cpu.sh

# Monitor for 120 seconds, 2-second intervals
./scripts/monitor_cpu.sh 120 2

# Or use Python directly
python3 ./scripts/monitor_cpu_comparison.py 60 1
```

**Parameters**:
- `duration` (seconds): How long to monitor [default: 60]
- `interval` (seconds): Sampling interval [default: 1.0]

**Features**:
- Monitors all processes matching "mac_stats" and "Stats"
- Real-time comparison display
- Generates summary report with statistics
- Exports CSV data for further analysis
- Optional screenshot capture

**Output**:
- `cpu-comparison-report-YYYYMMDD_HHMMSS.txt` - Summary report with averages
- `cpu-comparison-data-YYYYMMDD_HHMMSS.csv` - Raw data for analysis
- `cpu-comparison-screenshots/comparison-YYYYMMDD_HHMMSS.png` - Screenshot (optional)

**Example Workflow**:
```bash
# 1. Start both apps
./target/release/mac_stats &
open -a Stats

# 2. Wait a few seconds for apps to stabilize

# 3. Run comparison monitor
./scripts/monitor_cpu.sh 60 1

# 4. Review the generated report
cat cpu-comparison-report-*.txt

# 5. Open CSV in spreadsheet for visualization
open cpu-comparison-data-*.csv
```

**Example Output**:
```
================================================================================
  CPU Usage Comparison Monitor
================================================================================
Started: 2025-01-18 18:00:00

Monitoring for 60 seconds (interval: 1.0s)

Time     | mac_stats CPU | Stats CPU | Comparison
--------------------------------------------------------------------------------
18:00:01 | mac_stats:     0.45% (1 procs) | Stats:     2.30% (3 procs) | ✓ mac_stats uses LESS
18:00:02 | mac_stats:     0.52% (1 procs) | Stats:     2.15% (3 procs) | ✓ mac_stats uses LESS
18:00:03 | mac_stats:     0.48% (1 procs) | Stats:     2.40% (3 procs) | ✓ mac_stats uses LESS
...

================================================================================
CPU Usage Comparison Report
================================================================================
mac_stats App:
  Average CPU: 0.48%
  Minimum CPU: 0.35%
  Maximum CPU: 0.65%

Stats App:
  Average CPU: 2.25%
  Minimum CPU: 1.90%
  Maximum CPU: 2.80%

Comparison:
  ✓ mac_stats uses LESS CPU
  Difference: 1.77%
```

**Process Detection**:
The script automatically finds ALL processes belonging to mac-stats, including:
- Main Rust process (`mac_stats`)
- Tauri WebView processes
- Helper processes spawned by Tauri
- Any child processes

It identifies processes by:
- Process name patterns: `mac_stats`, `mac-stats`
- Bundle identifier: `com.raro42.mac-stats`
- Process paths containing `/mac-stats` or `/mac_stats`
- Parent-child relationships (finds all children of main process)

The script will show all found processes at startup so you can verify it's tracking everything correctly.

**Tips for Best Results**:
1. **Start both apps** before running the monitor
2. **Let apps stabilize** for 10-15 seconds before monitoring
3. **Close other apps** to minimize background noise
4. **Monitor for at least 60 seconds** for meaningful averages
5. **Use the CSV data** to create charts/graphs for presentations
6. **Verify process detection** - check the startup process list to ensure all mac-stats processes are found

---

## Build & Release

### `build-dmg.sh` - Create DMG Installer

Builds a macOS DMG file for distribution.

```bash
./scripts/build-dmg.sh
```

### `run.sh` - Quick Build & Run

Builds and runs the application.

```bash
./scripts/run.sh
```

---

## Debugging & Development

### `trace_backend.sh` - Backend Trace Logging

Enables detailed backend tracing for debugging.

```bash
./scripts/trace_backend.sh
```

### `take-screenshot.sh` - Screenshot Capture

Captures screenshot of CPU window for documentation.

```bash
./scripts/take-screenshot.sh
```

### `take-screenshot.py` - Python Screenshot Tool

Alternative screenshot capture using Python.

```bash
./scripts/take-screenshot.py
```

---

## Performance Analysis Workflow

### 1. Establish Baseline

Before implementing optimizations:

```bash
# Start app
./target/release/mac_stats --cpu &

# Measure current performance
./scripts/measure_performance.sh 30 1 window
./scripts/measure_performance.sh 30 1 idle

# Note the CSV filenames for comparison
```

### 2. Implement Optimizations

See optimization documentation:
- `docs/001_task_optimize_backend.md`
- `docs/002_task_optimize_frontend.md`
- `docs/003_task_optimize_advanced_idle.md`

```bash
# After making changes:
cargo build --release
```

### 3. Measure After Each Phase

```bash
# After Phase 1 (5 min):
./target/release/mac_stats --cpu &
./scripts/measure_performance.sh 30 1 window

# After Phase 2 (30 min):
./target/release/mac_stats --cpu &
./scripts/measure_performance.sh 30 1 window

# After Phase 3 (1-2 hours):
./target/release/mac_stats --cpu &
./scripts/measure_performance.sh 30 1 window
```

### 4. Compare Results

Import CSV files into spreadsheet (Excel, Google Sheets):

```bash
# Files to compare:
ls performance_*.csv

# Open in spreadsheet application
open performance_*.csv
```

Expected improvements:
- Phase 1: -12-18% CPU
- Phase 1+2: -15% CPU
- Phase 1-3: -20% CPU
- Phase 1-4: -22% CPU
- With advanced idle: -60-90% CPU reduction

---

## Measurement Output Example

```
════════════════════════════════════════════════════
  mac-stats Performance Measurement
════════════════════════════════════════════════════

Configuration:
  Process: mac_stats (PID: 12345)
  Mode: window
  Duration: 30s
  Interval: 1s
  Output: performance_window_20260118_180000.txt

Measurements over time:
Timestamp | CPU(%) | Threads | RSS(MB) | VSZ(MB) | MEM(%) |
18:00:00  |   1.2  |       8 |   120.5 |   456.2 |   0.8  |
18:00:01  |   0.9  |       8 |   120.5 |   456.2 |   0.8  |
18:00:02  |   1.1  |       8 |   121.0 |   456.2 |   0.8  |

=== Summary Statistics ===

CPU Usage:
  Average: 1.05%
  Min: 0.8%
  Max: 1.2%

Memory:
  Average: 0.8%
  Min: 0.7%
  Max: 0.9%

RSS (Resident Set Size):
  Average: 120.7 MB
  Min: 120.5 MB
  Max: 121.0 MB

Threads:
  Average: 8
  Min: 8
  Max: 8

Measurements: 31 samples over 30s
```

---

## Tips for Accurate Measurements

1. **Minimize Background Activity**:
   - Close other applications
   - Avoid network activity
   - Let system settle for 30 seconds before measuring

2. **Consistent Test Conditions**:
   - Always measure same duration (e.g., 30s)
   - Same measurement interval (e.g., 1s)
   - Same system state (plugged in, screen on)

3. **Multiple Runs**:
   - Take 3-5 measurements per configuration
   - Average the results
   - Look for consistency

4. **Compare Properly**:
   - Same mode ("window" vs "idle")
   - Same duration
   - Same conditions
   - Use CSV files for comparison

---

## Troubleshooting

**"Error: mac_stats is not running"**
```bash
# Start the app first:
./target/release/mac_stats --cpu  # With window
# or
./target/release/mac_stats        # Idle (menu bar only)
```

**Script not executable**
```bash
chmod +x ./scripts/measure_performance.sh
```

**Output files not created**
- Check write permissions in current directory
- Ensure script has execute permission
- Check for error messages in output

---

## Integration with Optimization Tasks

Use these scripts alongside the optimization documentation:

1. Read optimization summary: `docs/000_task_optimize_summary.md`
2. Take baseline measurement: `./scripts/measure_performance.sh 30 1 window`
3. Implement optimizations from `docs/001_task_optimize_backend.md`, etc.
4. Measure after each phase: `./scripts/measure_performance.sh 30 1 window`
5. Track progress using `docs/OPTIMIZE_CHECKLIST.md`

---

## For More Information

- Performance measurement: `agents.md` (Performance Measurement & Optimization section)
- Optimization tasks: See `docs/` directory
- Architecture: `CLAUDE.md`
