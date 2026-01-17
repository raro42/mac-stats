# Menu Bar Updates & Process Calculation Plan

## Current State

### Menu Bar Updates
- **Current**: 5 seconds (too slow for Stats-like responsiveness)
- **Target**: 1-2 seconds (like Stats app)
- **Location**: `src-tauri/src/lib.rs:256` - `std::thread::sleep(Duration::from_secs(5))`

### Process Collection
- **Current**: `sysinfo::System::refresh_processes()` - 60+ sysctl calls
- **Cache**: 20 seconds
- **Issue**: Expensive, blocks UI

---

## How `top` Calculates Process CPU

### Delta-Based Calculation (What `top` Does)

1. **Read CPU time counters** for each process:
   - `proc_pid_rusage(pid, RUSAGE_INFO_V4)` → `ru_utime` (user) + `ru_stime` (system)
   - Total CPU time = user_time + system_time

2. **Wait for interval** (e.g., 1 second)

3. **Read again** and compute delta:
   - `delta_cpu_time = current_cpu_time - previous_cpu_time`
   - `delta_wall_time = elapsed_time`

4. **Normalize by cores**:
   - `cpu_percent = (delta_cpu_time / delta_wall_time) / num_cores * 100`
   - If process uses 2 cores at 100% each → shows 200% CPU

5. **Sort by CPU usage** and display top N

### Why This Is Better Than `sysinfo`

| Approach | `sysinfo` | `top` (libproc) |
|----------|-----------|-----------------|
| **Calls per process** | Multiple sysctl calls | Single `proc_pid_rusage` call |
| **Accuracy** | Instantaneous (can be noisy) | Delta-based (smoother) |
| **CPU overhead** | High (60+ sysctl calls) | Low (direct kernel calls) |
| **Control** | Limited | Full control over sampling |

---

## Implementation Plan

### Phase 1: Fast Menu Bar Updates (Quick Win) - **START HERE**

**Goal**: Update menu bar every 1-2 seconds like Stats app  
**Time**: 30 minutes  
**Risk**: Low

#### Changes Needed

1. **Update menu bar refresh interval**:
   ```rust
   // src-tauri/src/lib.rs:256
   // Change from:
   std::thread::sleep(std::time::Duration::from_secs(5));
   // To:
   std::thread::sleep(std::time::Duration::from_secs(1)); // or 2 seconds
   ```

2. **Separate fast metrics from slow metrics**:
   - **Fast path (1-2s)**: CPU usage, RAM usage (from sysinfo cache)
   - **Slow path (5-10s)**: Temperature, frequency, processes

3. **Optimize `get_metrics()` for menu bar**:
   - Don't refresh system on every call
   - Use cached values (already implemented)
   - Only refresh if cache is stale

**Expected Impact**:
- Menu bar updates every 1-2 seconds ✅
- Minimal CPU increase (using cached values)
- Better user experience (responsive like Stats)

---

### Phase 2: Top-Style Process Calculation (High Impact)

**Goal**: Replace `sysinfo` process collection with libproc delta-based calculation  
**Time**: 4-6 hours  
**Risk**: Medium (new FFI code)

#### Why This Is Beneficial

1. **Much faster**: Single `proc_pid_rusage` call per process vs multiple sysctl calls
2. **More accurate**: Delta-based calculation matches Activity Monitor
3. **Less CPU**: Direct kernel calls, no sysinfo overhead
4. **Better control**: Can sample at any interval

#### Implementation Steps

1. **Create libproc wrapper module**:
   ```rust
   // src-tauri/src/native/proc.rs
   // FFI bindings for:
   // - proc_listpids() - get all PIDs
   // - proc_pid_rusage() - get CPU time for PID
   // - proc_pidinfo() - get process name
   ```

2. **Implement delta tracking**:
   ```rust
   struct ProcessDelta {
       pid: u32,
       last_cpu_time: u64, // user + system time
       last_timestamp: Instant,
   }
   
   // Map: PID → ProcessDelta
   static PROCESS_DELTAS: Mutex<HashMap<u32, ProcessDelta>> = ...;
   ```

3. **Compute CPU percentage**:
   ```rust
   fn compute_cpu_percent(pid: u32, current_cpu_time: u64, elapsed: Duration) -> f32 {
       let delta_cpu = current_cpu_time - last_cpu_time;
       let delta_wall = elapsed.as_secs_f64();
       let num_cores = num_cpus::get() as f64;
       
       // Normalize by cores: (delta_cpu / delta_wall) / num_cores * 100
       (delta_cpu as f64 / delta_wall) / num_cores * 100.0
   }
   ```

4. **Top-N selection**:
   - Use `select_nth_unstable_by` to get top N without sorting all
   - Only resolve names for top N processes

#### Files to Create/Modify

- **Create**: `src-tauri/src/native/proc.rs` - libproc FFI wrappers
- **Create**: `src-tauri/src/native/mod.rs` - module declaration
- **Modify**: `src-tauri/src/metrics/mod.rs` - use libproc instead of sysinfo for processes
- **Add dependency**: `libc` (already available) or create raw FFI bindings

#### Expected Impact

- **Process collection**: 60+ sysctl calls → ~10-20 `proc_pid_rusage` calls
- **CPU overhead**: ~70% reduction in process collection CPU
- **Accuracy**: Matches Activity Monitor behavior
- **Speed**: Process list loads instantly (no blocking)

---

### Phase 3: Separate Fast/Slow Update Paths

**Goal**: Different refresh rates for different metrics  
**Time**: 1-2 hours  
**Risk**: Low

#### Fast Path (1-2 seconds)
- CPU usage (from sysinfo cache)
- RAM usage (from sysinfo cache)
- Menu bar display

#### Slow Path (5-10 seconds)
- Temperature (SMC - expensive)
- Frequency (IOReport - expensive)
- Processes (libproc - if implemented)

#### Implementation

```rust
// Separate update loops
let fast_metrics = get_fast_metrics(); // CPU, RAM (cached)
update_menu_bar(fast_metrics);

// Slow metrics in background thread
if should_update_slow_metrics() {
    let slow_metrics = get_slow_metrics(); // Temp, freq, processes
    // Update cache, don't block menu bar
}
```

---

## Comparison: Current vs. Top-Style

### Current Approach (sysinfo)

```rust
// Every 20 seconds:
sys.refresh_processes(ProcessesToUpdate::All, true);
// → 60+ sysctl calls
// → Blocks for ~50-100ms
// → Returns instantaneous CPU (noisy)
```

### Top-Style Approach (libproc)

```rust
// Every 1-2 seconds:
let pids = proc_listpids(PROC_ALL_PIDS, ...); // ~1 syscall
for pid in pids {
    let rusage = proc_pid_rusage(pid, RUSAGE_INFO_V4); // ~1 syscall per PID
    let cpu_percent = compute_delta(pid, rusage);
}
// → ~10-20 syscalls total
// → Non-blocking (direct kernel calls)
// → Delta-based (smooth, accurate)
```

---

## Decision: Should We Implement Top-Style?

### ✅ **YES, if**:
- Process collection is still slow after Phase 1
- You want Activity Monitor-level accuracy
- You're willing to write FFI code

### ⚠️ **MAYBE, if**:
- Menu bar updates are fast enough with Phase 1
- Process collection is acceptable (20s cache)
- You want to avoid FFI complexity

### ❌ **NO, if**:
- Current performance is acceptable
- You want to minimize code changes
- sysinfo works fine for your use case

---

## Recommended Implementation Order

### Immediate (Today)
1. ✅ **Phase 1**: Fast menu bar updates (1-2 seconds) - **30 minutes**

### This Week
2. ⚠️ **Phase 2**: Top-style process calculation - **4-6 hours** (if needed)
3. ✅ **Phase 3**: Separate fast/slow paths - **1-2 hours**

---

## Expected Results

### After Phase 1 (Menu Bar Updates)
- Menu bar updates every 1-2 seconds ✅
- CPU increase: ~0.1-0.2% (minimal, using cached values)
- User experience: Responsive like Stats app ✅

### After Phase 2 (Top-Style Processes)
- Process collection: 70% faster
- CPU overhead: 70% reduction
- Accuracy: Matches Activity Monitor
- Process list: Loads instantly

---

## Next Steps

1. **Start with Phase 1** - Quick win, low risk
2. **Test menu bar responsiveness** - Is 1-2 seconds fast enough?
3. **Evaluate Phase 2** - Is process collection still slow?
4. **Implement Phase 2** - Only if needed
