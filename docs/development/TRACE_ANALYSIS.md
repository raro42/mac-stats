# Backend Trace Analysis Results

## Summary

**Backend CPU Usage: 0.0%** (mostly idle, waiting in event loop)

However, when the CPU window is open, the backend experiences **CPU spikes** during process refresh operations.

## Key Findings

### 1. Backend is Mostly Idle
- Shows **0.0% CPU** in `ps` and `top` when idle
- Most time spent in event loop (`mach_msg` calls) - this is normal
- 7969 samples over 10 seconds, mostly waiting

### 2. CPU Spikes During Process Refresh
When `get_cpu_details()` is called (every 8 seconds), it triggers:
- `sysinfo::unix::apple::system::SystemInner::refresh_processes_specifics`
- **60+ `sysctl` calls** per refresh
- Multiple `proc_pidinfo` and `__proc_info` calls
- This is the expensive operation!

### 3. WebKit Processes Running
- `com.apple.WebKit.WebContent` (multiple instances)
- `com.apple.WebKit.Networking`
- `com.apple.WebKit.GPU`
- These are the webview processes (normal for Tauri)

## The Problem

The backend itself is efficient (0.0% CPU when idle), but:
1. **Process refresh is expensive** - 60+ sysctl calls every 20 seconds
2. **Frontend polling** - Calls `get_cpu_details()` every 8 seconds
3. **Process collection** - Even with caching, when cache expires, it does full refresh

## Current Optimizations Applied

✅ Process cache: 20 seconds  
✅ Frontend polling: 8 seconds  
✅ Temperature reading: 15 seconds  
✅ IOReport frequency: 20 seconds  
✅ Menu bar updates: 5 seconds  
✅ System refresh: 5 seconds  

## Recommendations

### Option 1: Further Increase Cache Times
- Process cache: 20s → 30s
- Frontend polling: 8s → 10s

### Option 2: Optimize Process Collection
- Only collect top 5 processes instead of 8
- Use incremental refresh if possible (but `OnlyNew` doesn't exist in sysinfo)

### Option 3: Reduce Process Collection Scope
- Only collect processes when user scrolls to process list
- Lazy load process list

### Option 4: Profile Webview Process
The `tauri://localhost` process might be using more CPU than the backend. We should trace that separately.

## Next Steps

1. **Trace the webview process** (`tauri://localhost`) to see if it's the real culprit
2. **Increase cache times further** if acceptable
3. **Consider lazy loading** the process list
4. **Profile with Instruments** for more detailed analysis
