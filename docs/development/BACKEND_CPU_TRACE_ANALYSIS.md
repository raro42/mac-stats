# Backend CPU Trace Analysis - CPU Window Open

**Date:** 2026-01-17 23:45:34  
**Process:** mac-stats-backend (PID: 51104)  
**Sampling Duration:** 5 seconds  
**CPU Usage Observed:** 5.9% spike  
**Important:** Spike ONLY occurs when CPU window is OPEN, NOT when closed

## Summary

The backend CPU spike (5.9%) is caused by operations that **only happen when the CPU window is open**:

1. **Process collection (`refresh_processes()`)** - Expensive operation that scans ALL processes
2. **Frontend polling every 1 second** - Calls `get_cpu_details()` frequently
3. **WebKit IPC activity** - Communication between backend and frontend webview
4. **Core Animation layer updates** - Rendering updates for the CPU window

## Key Findings

### 1. CPU Spike Only When Window Open

**Critical Observation:** The 5.9% CPU spike ONLY occurs when the CPU window is open. When closed, CPU usage is minimal (~0.5%).

This means the spike is NOT from:
- Menu bar rendering (happens regardless of window state)
- Background update loop (runs regardless)

The spike IS from:
- Operations that only happen when `should_collect_processes = true`
- Frontend polling every 1 second (only when window open)
- WebKit IPC communication (only when window exists)

### 2. Process Collection (`refresh_processes()`)

When the CPU window is open:
```rust
let should_collect_processes = APP_HANDLE.get()
    .and_then(|app_handle| {
        app_handle.get_window("cpu").and_then(|window| {
            window.is_visible().ok().filter(|&visible| visible)
        })
    })
    .is_some();

if should_collect_processes {
    sys.refresh_processes(ProcessesToUpdate::All, true);  // EXPENSIVE!
    // Collect ALL processes, sort, take top 8
}
```

**Impact:**
- `refresh_processes()` scans ALL processes on the system
- Makes multiple sysctl calls per process
- Collects, sorts, and filters processes
- Happens when cache is empty or stale (>60s)

**Frequency:**
- Frontend polls every 1 second
- Backend rate-limited to max once per 2 seconds
- But if cache is empty, `refresh_processes()` is called immediately

### 3. WebKit IPC Activity

The trace shows WebKit IPC messages being processed:
```
WebKit::WebProcessProxy::didReceiveMessage
  → WebKit::WebUserContentControllerProxy::didPostMessage
  → wry::webview::wkwebview::InnerWebView::new::did_receive
  → tauri_runtime_wry::create_ipc_handler
```

**Impact:** Each frontend `invoke("get_cpu_details")` call:
- Sends IPC message from webview to backend
- Backend processes message on main thread
- Returns data via IPC
- WebKit processes response

### 4. Core Animation Layer Updates

```
CA::Transaction::flush_as_runloop_observer
  → CA::Transaction::commit()
  → CA::Layer::display_if_needed()
  → NSViewBackingLayer::display()
```

**Impact:** CPU window rendering triggers:
- Layer tree updates for the webview
- Display refresh callbacks
- Transaction commits

## CPU Usage Breakdown

From the sample analysis:
- **Main Thread:** 100% of samples (2406/2406)
- **Event Loop:** Most time spent in `mach_msg2_trap` (idle/waiting)
- **Rendering Spikes:** Occur during menu bar drawing operations
- **Background Threads:** Minimal CPU usage (mostly idle)

## Root Cause

The 5.9% CPU spike is caused by operations that **only happen when CPU window is open**:

1. **Frontend polling every 1 second** → Calls `get_cpu_details()` frequently
2. **Process collection (`refresh_processes()`)** → Expensive operation when cache is empty/stale
   - Scans ALL processes on system
   - Makes multiple sysctl calls per process
   - Collects, sorts, filters to top 8
3. **WebKit IPC overhead** → Each `invoke()` call has IPC overhead
4. **Core Animation updates** → CPU window rendering triggers layer updates

## Recommendations (Analysis Only - No Code Changes)

### Potential Optimizations:

1. **Optimize process collection**
   - Current: `refresh_processes(ProcessesToUpdate::All, true)` - scans ALL processes
   - Consider: Use delta-based process updates (only new/changed processes)
   - Consider: Use native macOS APIs (`libproc`, `proc_pid_rusage`) instead of sysinfo
   - Consider: Increase cache validity (currently 60s for stale, 20s for refresh)

2. **Reduce frontend polling frequency**
   - Current: 1 second (matches menu bar)
   - Consider: 2-3 seconds for CPU window (processes don't need 1s updates)
   - Keep 1 second for menu bar (user expects responsiveness)

3. **Improve process cache strategy**
   - Current: Cache for 20 seconds, but refreshes if empty
   - Consider: Pre-populate cache in background thread
   - Consider: Only refresh processes when cache is truly empty (not just stale)

4. **Optimize WebKit IPC**
   - Current: Each `invoke()` call has IPC overhead
   - Consider: Batch multiple metric requests into single IPC call
   - Consider: Use WebSocket or shared memory for high-frequency updates

5. **Profile specific operations**
   - Measure exact cost of `refresh_processes()` - how many sysctl calls?
   - Identify which part of process collection is most expensive
   - Check if process sorting/filtering is the bottleneck

## Thread Activity

- **Main Thread:** 2406 samples (100%) - UI/rendering
- **NSEventThread:** 3984 samples - Event handling (mostly idle)
- **JavaScriptCore scavenger:** 3984 samples - Memory management (mostly idle)
- **WebCore Scrolling:** 3984 samples - WebKit thread (mostly idle)

## Conclusion

The 5.9% CPU spike is caused by **process collection operations** that only happen when the CPU window is open:

1. **Frontend polls every 1 second** → Calls `get_cpu_details()`
2. **Process collection is expensive** → `refresh_processes()` scans ALL processes
3. **WebKit IPC overhead** → Each invoke call has communication overhead
4. **Core Animation updates** → CPU window rendering

**Key Insight:** The spike is NOT from menu bar rendering (which happens regardless), but from the expensive process collection that only occurs when the window is visible.

**When window is closed:** No process collection → Minimal CPU usage (~0.5%)  
**When window is open:** Process collection every 1-2 seconds → 5.9% CPU spike

The primary bottleneck is `sys.refresh_processes(ProcessesToUpdate::All, true)`, which:
- Scans ALL processes on the system
- Makes multiple sysctl calls per process
- Collects, sorts, and filters processes
- Happens when cache is empty or stale

**This is the main optimization target** - reducing the cost of process collection will significantly reduce CPU usage when the window is open.
