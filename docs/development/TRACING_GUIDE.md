# Backend Tracing Guide

## Quick Analysis Results

From the CPU sample, we can see:
- **3967 samples** were taken over 5 seconds
- **Almost all CPU time** is spent in the main event loop (`mach_msg` calls)
- The backend is mostly **idle**, waiting for events
- Most work happens in **WebKit/AppKit** (the webview rendering)

This suggests the high CPU usage might be from:
1. **WebKit rendering** (the `tauri://localhost` process)
2. **Frequent event processing** in the event loop
3. **Background threads** doing periodic work

## Tracing Tools Available

### 1. CPU Sampling (sample)
Shows which functions are consuming CPU:
```bash
# Sample for 10 seconds
sudo sample <PID> 10 -f backend_sample.txt

# View results
cat backend_sample.txt | grep -E "^[0-9]+ " | head -30
```

### 2. System Call Tracing (dtruss)
Shows all system calls (like strace on Linux):
```bash
# Trace all syscalls
sudo dtruss -p <PID> 2>&1 | tee backend_syscalls.log

# Trace only expensive syscalls
sudo dtruss -p <PID> 2>&1 | grep -E "(read|write|open|close|stat|mach_msg|select|poll)" | tee backend_syscalls_filtered.log
```

### 3. Advanced Tracing (dtrace)
Count syscalls by type:
```bash
sudo dtrace -n 'syscall:::entry /pid == <PID>/ { @[probefunc] = count(); }'
```

## Using the Tracing Script

Run the interactive script:
```bash
./trace_backend.sh
```

This will:
1. Find the backend process automatically
2. Let you choose a tracing method
3. Save output to files for analysis

## What to Look For

When tracing, look for:
- **Frequent `mach_msg` calls** - Event loop activity (normal)
- **Frequent `read`/`write`** - File I/O operations
- **Frequent `stat`/`open`** - File system access
- **Frequent `select`/`poll`** - Waiting on file descriptors
- **High CPU in specific functions** - From sample output

## Next Steps

1. **Run the tracing script** while the CPU window is open
2. **Look for patterns** - Are there syscalls happening every few seconds?
3. **Check the webview process** - The `tauri://localhost` process might be the culprit
4. **Compare with window closed** - Run tracing with window closed vs open

## Current Optimizations Applied

- Process refresh: Every 20 seconds (was 15s)
- Frontend polling: Every 8 seconds (was 5s)
- Temperature reading: Every 15 seconds (was 5s)
- IOReport frequency: Every 20 seconds (was 10s)
- Menu bar updates: Every 5 seconds (was 2s)
- System refresh: Every 5 seconds (was 2s)

If CPU is still high, the issue might be:
- WebKit rendering/animations
- Too frequent event loop processing
- Background thread work
