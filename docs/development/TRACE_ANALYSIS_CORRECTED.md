# Backend Trace Analysis - Corrected Results

## Trace Method
- Used `sudo sample` for CPU profiling (10 seconds)
- Used `sudo dtruss` for syscall tracing (limited by SIP)
- Window was open during trace

## Key Findings

### 1. get_cpu_details() is Called Too Frequently
**61 calls in 10 seconds = ~6 calls per second**

This is **WAY more frequent** than the 8-second polling interval we configured!
- Expected: 1 call every 8 seconds = ~1.25 calls per 10 seconds
- Actual: 61 calls per 10 seconds = **48x more frequent than expected**

**This is the root cause of high CPU usage!**

### 2. Process Refresh Operations
Each `get_cpu_details()` call triggers:
- `refresh_processes()` - 61 calls
- `sysctl` calls - 42 total (28 + 14)
- `__proc_info` calls - 14 total

### 3. Most Common Syscalls (from sample)
- `__workq_kernreturn` - 16,262 (thread pool operations - normal)
- `semaphore_wait_trap` - 8,141 (waiting - normal)
- `__sysctl` - 42 (process info queries - expensive)
- `__proc_info` - 14 (process info queries - expensive)

## The Real Problem

The frontend is calling `get_cpu_details()` **much more frequently** than expected:
- **Expected**: Every 8 seconds (from `setInterval(refresh, 8000)`)
- **Actual**: ~6 times per second (61 calls in 10 seconds)

This could be due to:
1. Multiple `setInterval` calls (not clearing previous ones)
2. Event-driven calls in addition to polling
3. Animation frame callbacks triggering refreshes
4. Multiple windows or event handlers

## Recommendations

### Immediate Fix: Check Frontend Polling
1. **Verify only one `setInterval` is active**
2. **Clear previous intervals** before creating new ones
3. **Check for event-driven calls** (window focus, visibility, etc.)
4. **Check animation callbacks** - they might be calling refresh

### Code Changes Needed
1. **Add logging** to see when `get_cpu_details()` is actually called
2. **Add rate limiting** in the backend (max 1 call per 2-3 seconds)
3. **Fix frontend polling** to ensure only one interval is active

## Next Steps

1. Check `src-tauri/dist/cpu.js` for multiple `setInterval` calls
2. Check for event listeners that might trigger refresh
3. Add backend rate limiting as a safety measure
4. Add logging to track actual call frequency
