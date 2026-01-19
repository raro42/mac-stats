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

Starts both apps fresh, waits for stabilization, then monitors CPU usage for accurate, reproducible comparison. Perfect for demonstrating that mac_stats uses less CPU.

**Usage**:
```bash
# Monitor for 60 seconds (default), 1-second intervals, 30-second warmup
./scripts/monitor_cpu.sh

# Monitor for 120 seconds, 2-second intervals, 30-second warmup
./scripts/monitor_cpu.sh 120 2

# Custom warmup period (e.g., 15 seconds)
./scripts/monitor_cpu.sh 60 1 15

# Or use Python directly
python3 ./scripts/monitor_cpu_comparison.py 60 1 30
```

**Parameters**:
- `duration` (seconds): How long to monitor after warmup [default: 60]
- `interval` (seconds): Sampling interval [default: 1.0]
- `warmup` (seconds): Warmup/stabilization period [default: 30]

**Features**:
- **Starts apps fresh**: Launches both apps from command line for clean test
- **30-second warmup**: Waits for apps to stabilize before measuring
- **Precise PID tracking**: Captures main PID and tracks all child processes
- **CPU time measurement**: Uses cumulative CPU time (more reliable than instant %)
- **Real-time comparison**: Shows CPU time consumed and percentage reduction
- **Comprehensive reports**: Summary with statistics and CSV export
- **Optional cleanup**: Can kill processes after monitoring

**Output**:
- `cpu-comparison-report-YYYYMMDD_HHMMSS.txt` - Summary report with CPU time comparison
- `cpu-comparison-data-YYYYMMDD_HHMMSS.csv` - Raw data for analysis
- `cpu-comparison-screenshots/comparison-YYYYMMDD_HHMMSS.png` - Screenshot (optional)

**Example Workflow**:
```bash
# Simply run the script - it handles everything:
./scripts/monitor_cpu.sh 60 1

# The script will:
# 1. Check for existing processes (ask to kill or use them)
# 2. Start both apps fresh
# 3. Wait 30 seconds for stabilization (warmup)
# 4. Monitor for 60 seconds
# 5. Generate reports
# 6. Ask if you want to keep apps running

# Review the generated report
cat cpu-comparison-report-*.txt

# Open CSV in spreadsheet for visualization
open cpu-comparison-data-*.csv
```

**Example Output**:
```
================================================================================
  CPU Usage Comparison Monitor
================================================================================
Started: 2025-01-18 18:00:00

Checking for existing processes...
  No existing processes found. Starting fresh...
Starting mac_stats...
  ✓ Started mac_stats (PID 12345)
Starting Stats.app...
  ✓ Started Stats.app (PID 12346)

  mac_stats: 4 process(es) (main PID: 12345)
  Stats: 8 process(es) (main PID: 12346)

⏱️  Warmup period: 30 seconds
   Apps are initializing and stabilizing...
   30 seconds remaining...
   ...
   ✓ Warmup complete!

Capturing baseline CPU times...
  ✓ Baseline captured

📊 Monitoring for 60 seconds (interval: 1.0s)

Time     | mac_stats CPU Time | Stats CPU Time | Reduction
--------------------------------------------------------------------------------
18:00:31 | mac_stats:      2.1s (4 procs) | Stats:     12.5s (8 procs) | ✓ -83.2% CPU time
18:00:32 | mac_stats:      4.3s (4 procs) | Stats:     25.1s (8 procs) | ✓ -82.9% CPU time
...

================================================================================
RESULTS SUMMARY (Based on CPU Time - Most Reliable Metric)
================================================================================
  mac_stats: 45.2s CPU time (0.75% average)
  Stats:     275.8s CPU time (4.60% average)
  Absolute Difference: 230.6s
  
  ✓ mac_stats uses 83.6% LESS CPU time than Stats
  
  🎉
```

**How It Works**:
1. **Starts apps fresh**: Launches both apps from command line, capturing main PIDs immediately
2. **Tracks all processes**: Recursively finds all child processes (Tauri WebView, helpers, etc.)
3. **30-second warmup**: Waits for apps to stabilize (initialization complete, steady-state reached)
4. **Baseline capture**: Records CPU times at end of warmup (our "zero point")
5. **Monitors CPU time**: Measures cumulative CPU time delta (more reliable than instant %)
6. **Fair comparison**: Both apps start at same time, same conditions, same warmup period

**Why CPU Time Instead of Instantaneous %?**
- **More stable**: CPU time is cumulative, doesn't fluctuate wildly
- **More accurate**: Represents actual resource usage over time
- **Fairer**: Not affected by momentary spikes or sampling timing
- **Same as Activity Monitor**: Uses the same metric Activity Monitor shows

The script will show all found processes at startup so you can verify it's tracking everything correctly.

**Tips for Best Results**:
1. **Close other apps** to minimize background noise during testing
2. **Monitor for at least 60 seconds** after warmup for meaningful data
3. **Use default 30-second warmup** to ensure steady-state measurements
4. **Review the process list** at startup to verify all processes are tracked
5. **Use the CSV data** to create charts/graphs for presentations
6. **Multiple runs**: Run 2-3 times and average results for most accurate comparison

**If Apps Are Already Running**:
The script will detect existing processes and ask if you want to:
- Kill them and start fresh (recommended for fair comparison)
- Use existing processes (skips warmup, measures from current state)

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
