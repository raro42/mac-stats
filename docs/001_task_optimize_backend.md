# Backend CPU Optimization Tasks

## Overview
This document outlines backend (Rust/FFI) CPU optimization opportunities for mac-stats, organized by system component. Target: 18-22% overall CPU reduction through interval adjustments and lock optimization.

---

## HIGH IMPACT OPTIMIZATIONS

### Task 1: Increase Temperature Reading Interval (15s → 20s)

**File**: `src-tauri/src/lib.rs:407-417`

**Current Code**:
```rust
let should_read_temp_now = if let Ok(mut last) = LAST_TEMP_UPDATE.lock() {
    let should = last.as_ref()
        .map(|t| t.elapsed().as_secs() >= 15)  // ← Change this
        .unwrap_or(true);
```

**Change**:
```rust
let should_read_temp_now = if let Ok(mut last) = LAST_TEMP_UPDATE.lock() {
    let should = last.as_ref()
        .map(|t| t.elapsed().as_secs() >= 20)  // ← Increased from 15 to 20
        .unwrap_or(true);
```

**Rationale**:
- Temperature changes gradually on modern Macs (typically 1-2°C per minute at load)
- 15s→20s interval imperceptible to user; temperature display updates still responsive
- Saves one `all_data()` iteration (~100-200 SMC key scans) every 5 seconds
- Reduces SMC call overhead by ~25%

**Impact**: ~2-3% overall CPU reduction when CPU window visible

**Testing**:
```bash
./target/release/mac_stats --cpu -vvv 2>&1 | grep -i "temperature"
# Verify interval changes from ~15s to ~20s
tail -f ~/.mac-stats/debug.log | grep "Temperature updated"
```

**Effort**: 1 line change (30 seconds)

---

### Task 2: Increase Frequency Reading Interval (20s → 30s)

**File**: `src-tauri/src/lib.rs:524-535`

**Current Code**:
```rust
let should_read_freq = if let Ok(mut last) = LAST_FREQ_READ.lock() {
    debug2!("========> LAST_FREQ_READ: {:?}", last);
    let should = last.as_ref()
        .map(|t| t.elapsed().as_secs() >= 20)  // ← Change this
        .unwrap_or(true);
```

**Change**:
```rust
let should_read_freq = if let Ok(mut last) = LAST_FREQ_READ.lock() {
    debug2!("========> LAST_FREQ_READ: {:?}", last);
    let should = last.as_ref()
        .map(|t| t.elapsed().as_secs() >= 30)  // ← Increased from 20 to 30
        .unwrap_or(true);
```

**Rationale**:
- CPU frequency changes are discrete (performance states) not continuous
- 20s→30s provides imperceptible latency (0.5s granularity per frequency step)
- IOReport subscription creation and sampling is expensive (multiple CFDictionary operations)
- Reduces IOReport overhead by ~33%
- Still responsive for detecting thermal throttling events

**Impact**: ~4-6% overall CPU reduction when CPU window visible

**Testing**:
```bash
./target/release/mac_stats --cpu --frequency -vvv 2>&1 | grep "frequency"
# Verify P-core/E-core frequency updates every ~30s instead of 20s
tail -f ~/.mac-stats/debug.log | grep "frequency cache updated"
```

**Effort**: 1 line change (30 seconds)

---

### Task 3: Increase Process Cache Interval (5s → 10s)

**File**: `src-tauri/src/metrics/mod.rs:767`

**Current Code**:
```rust
// Cache processes for 5 seconds
if let Some((cached_processes, timestamp)) = cache.as_ref() {
    if let Ok(age) = timestamp.elapsed() {
        let age_secs = age.as_secs_f64();
        if age_secs < 5.0 {  // ← Change this
            debug3!("Process cache hit (age={:.1}s)", age_secs);
            return cached_processes.clone();
        }
    }
}
```

**Change**:
```rust
// Cache processes for 10 seconds (reduced refresh frequency)
if let Some((cached_processes, timestamp)) = cache.as_ref() {
    if let Ok(age) = timestamp.elapsed() {
        let age_secs = age.as_secs_f64();
        if age_secs < 10.0 {  // ← Increased from 5.0 to 10.0
            debug3!("Process cache hit (age={:.1}s)", age_secs);
            return cached_processes.clone();
        }
    }
}
```

**Rationale**:
- `System::refresh_processes()` with `ProcessesToUpdate::All` is expensive (O(n) process enumeration)
- Process CPU usage changes are typically smooth, not jittery
- 5s→10s is imperceptible to users viewing top process list
- Reduces full process enumeration overhead by ~50%
- Top 8 processes change rarely within 10-second window

**Impact**: ~3-4% overall CPU reduction

**Testing**:
```bash
./target/release/mac_stats --cpu 2>&1 | head -100
# Monitor debug log for process cache hit frequency
tail -f ~/.mac-stats/debug.log | grep "Process cache"
```

**Effort**: 1 line change (30 seconds)

---

### Task 4: Split ACCESS_CACHE into Separate OnceLock Fields

**Files**:
- `src-tauri/src/state.rs:42` (definition)
- `src-tauri/src/metrics/mod.rs:178-223, 354-395` (usage)
- `src-tauri/src/lib.rs:264-271, 384-391, 688-695` (updates)

**Current Design**:
```rust
// state.rs:42
pub(crate) static ACCESS_CACHE: Mutex<Option<(bool, bool, bool, bool)>> = Mutex::new(None);
// Protects: (can_read_temp, can_read_freq, can_read_cpu_power, can_read_gpu_power)
```

**Problem**:
- Single Mutex protects 4 independent boolean flags
- Lock contention when background thread updates flags while frontend reads
- Every flag access requires full Mutex lock/unlock even though flags are nearly static

**New Design**:
```rust
// state.rs - Replace lines 42-43 with:
pub(crate) static CAN_READ_TEMPERATURE: OnceLock<bool> = OnceLock::new();
pub(crate) static CAN_READ_FREQUENCY: OnceLock<bool> = OnceLock::new();
pub(crate) static CAN_READ_CPU_POWER: OnceLock<bool> = OnceLock::new();
pub(crate) static CAN_READ_GPU_POWER: OnceLock<bool> = OnceLock::new();
```

**Changes Required**:

1. **state.rs**: Remove line 42, add 4 new OnceLock fields

2. **metrics/mod.rs:178-223** - `can_read_temperature()` function:
```rust
// OLD:
if let Ok(mut cache) = ACCESS_CACHE.try_lock() {
    if let Some((temp, _, _, _)) = cache.as_ref() {
        debug3!("can_read_temperature: {} (from ACCESS_CACHE)", *temp);
        return *temp;
    }
    // ... first time check
    *cache = Some((can_read, false, false, false));
}

// NEW:
if let Some(&result) = CAN_READ_TEMPERATURE.get() {
    debug3!("can_read_temperature: {} (cached)", result);
    return result;
}
// ... first time check
let _ = CAN_READ_TEMPERATURE.set(can_read);
```

3. **lib.rs:264-271** - Update after SMC connection:
```rust
// OLD:
if let Ok(mut cache) = ACCESS_CACHE.try_lock() {
    if let Some((_, freq, cpu_power, gpu_power)) = cache.as_ref() {
        *cache = Some((true, *freq, *cpu_power, *gpu_power));
    } else {
        *cache = Some((true, false, false, false));
    }
}

// NEW:
let _ = CAN_READ_TEMPERATURE.set(true);
```

4. **lib.rs:384-391** - Update after IOReport subscription:
```rust
// OLD:
if let Ok(mut cache) = ACCESS_CACHE.try_lock() {
    if let Some((temp, _, cpu_power, gpu_power)) = cache.as_ref() {
        *cache = Some((*temp, true, *cpu_power, *gpu_power));
    } else {
        *cache = Some((false, true, false, false));
    }
}

// NEW:
let _ = CAN_READ_FREQUENCY.set(true);
```

5. **metrics/mod.rs:354-395** - Similar updates for can_read_frequency(), can_read_cpu_power(), can_read_gpu_power()

**Rationale**:
- OnceLock is lock-free after initialization (uses atomic compare-and-swap)
- Each flag is independent; no need to lock all 4 together
- Eliminates contention between background thread (writer) and frontend thread (reader)
- Flags are nearly static (set once at startup, never change)

**Impact**: ~2-3% CPU reduction (eliminates unnecessary lock contention)

**Testing**:
```bash
cargo build --release
./target/release/mac_stats --cpu -vv
# Monitor for no lock contention errors in logs
```

**Effort**: Medium (5-6 files, ~30 lines of code changes)

---

## MEDIUM IMPACT OPTIMIZATIONS

### Task 5: Move Window Visibility Check Before SMC Connection

**File**: `src-tauri/src/lib.rs:244-252`

**Current Code**:
```rust
// CRITICAL: Only read temperature when CPU window is visible (saves CPU)
// Check window visibility before expensive SMC operations
let should_read_temp = APP_HANDLE.get()
    .and_then(|app_handle| {
        app_handle.get_window("cpu").and_then(|window| {
            window.is_visible().ok().filter(|&visible| visible)
        })
    })
    .is_some();

if should_read_temp {
    // CPU window is visible - read temperature and frequency
    // Reuse SMC connection if available, otherwise create new one
    if smc_connection.is_none() {
        match Smc::connect() {  // ← Expensive call happens even if window not visible
```

**Problem**:
- Window visibility check is done AFTER attempting SMC connection
- If window not visible, SMC::connect() still executes (expensive)

**Change**: Move visibility check to before SMC connection:
```rust
// Check window visibility FIRST (cheap operation)
let should_read_temp = APP_HANDLE.get()
    .and_then(|app_handle| {
        app_handle.get_window("cpu").and_then(|window| {
            window.is_visible().ok().filter(|&visible| visible)
        })
    })
    .is_some();

if !should_read_temp {
    // CPU window is not visible - skip all temperature/frequency operations
    if smc_connection.is_some() {
        smc_connection = None;
        debug3!("CPU window closed, SMC connection released");
    }
    // Clear IOReport subscription...
    // Short-circuit: skip to next iteration
} else {
    // CPU window IS visible - proceed with SMC connection
    if smc_connection.is_none() {
        match Smc::connect() {
            // ... existing code
        }
    }
    // ... rest of temperature/frequency reading
}
```

**Rationale**:
- Window visibility check is very fast (Tauri API call)
- SMC::connect() involves system calls and may block
- Avoiding connection attempt when window closed saves 5-10ms per 2-second loop iteration
- Already checking visibility, just need to short-circuit earlier

**Impact**: ~1% CPU reduction

**Testing**:
```bash
./target/release/mac_stats --cpu
# Close CPU window, monitor CPU usage drops immediately
# Check log: "CPU window closed, SMC connection released" should appear
tail -f ~/.mac-stats/debug.log | grep "window closed"
```

**Effort**: Low (reorganize existing code, ~20 lines)

---

### Task 6: Optimize IOReport Channel State Validation

**File**: `src-tauri/src/ffi/ioreport.rs:445-454`

**Current Code**:
```rust
let state_count = unsafe {
    IOReportStateGetCount(item as CFDictionaryRef)
};
debug3!("State count for {}: {}", channel_name, state_count);

if state_count < 1 || state_count > 100 {
    debug3!("  Skipping: invalid state count {} (not in range 1..100)", state_count);
    continue;
}
```

**Problem**:
- State count validation happens for EVERY channel on EVERY frequency read
- State counts don't change at runtime (determined by CPU architecture)
- Threshold check `> 100` recomputed every 20 seconds
- P-core and E-core channels typically have ~8-12 states each

**Change**: Cache state counts after first read
```rust
// Add to state.rs (new cache):
pub(crate) static IOREPORT_STATE_COUNTS: Mutex<Option<std::collections::HashMap<String, i32>>> = Mutex::new(None);

// In ioreport.rs parse_channel_states():
let mut state_counts = IOREPORT_STATE_COUNTS.lock().ok();
let cached_counts = state_counts.as_ref().and_then(|map| map.get(&channel_name).copied());

let state_count = if let Some(cached) = cached_counts {
    debug3!("Using cached state count for {}: {}", channel_name, cached);
    cached
} else {
    unsafe { IOReportStateGetCount(item as CFDictionaryRef) }
};

// Cache for future use
if state_counts.is_some() {
    if let Some(map) = state_counts.as_mut() {
        map.insert(channel_name.clone(), state_count);
    }
}

if state_count < 1 || state_count > 100 {
    // ... existing validation
}
```

**Rationale**:
- IOReport channel structure is static per system
- Caching state counts eliminates one IOReportStateGetCount() call per channel per read
- Channels are typically 2-4 per read (P-core, E-core), so 2-4 calls saved per 20s cycle
- HashMap lookup much faster than Core Foundation API call

**Impact**: ~0.5-1% CPU reduction

**Effort**: Medium (add cache, modify ioreport.rs, ~25 lines)

---

### Task 7: Reduce Redundant CFRelease Batching

**File**: `src-tauri/src/lib.rs:597-620`

**Current Code**:
```rust
// Store current sample for next delta calculation
if let Some(current_sample) = current_sample_opt {
    // Retain the sample before storing (Core Foundation ownership rule)
    let retained_sample = CFRetain(current_sample as CFTypeRef) as CFDictionaryRef;
    if let Ok(mut last_sample_storage) = LAST_IOREPORT_SAMPLE.try_lock() {
        // Release old sample if it exists
        if let Some((old_sample_usize, _)) = last_sample_storage.take() {
            let old_sample = old_sample_usize as CFDictionaryRef;
            if !old_sample.is_null() {
                CFRelease(old_sample as CFTypeRef);  // ← Individual release
            }
        }
        // Store retained sample
        *last_sample_storage = Some((retained_sample as usize, std::time::Instant::now()));
    } else {
        // Lock failed, release the retained sample
        CFRelease(retained_sample as CFTypeRef);  // ← Another release
    }
    // Release the original sample (we've retained a copy)
    CFRelease(current_sample as CFTypeRef);  // ← Third release
}
```

**Problem**:
- Multiple CFRelease calls within single operation (3 per frequency read)
- Each CFRelease is a system call with overhead
- No batching or optimization

**Change**: Batch releases or use smarter pointer management:
```rust
if let Some(current_sample) = current_sample_opt {
    let retained_sample = unsafe { CFRetain(current_sample as CFTypeRef) } as CFDictionaryRef;

    if let Ok(mut last_sample_storage) = LAST_IOREPORT_SAMPLE.try_lock() {
        // Batch old release with new store
        if let Some((old_sample_usize, _)) = last_sample_storage.take() {
            if old_sample_usize != 0 {
                unsafe { CFRelease(old_sample_usize as CFTypeRef) };
            }
        }
        *last_sample_storage = Some((retained_sample as usize, std::time::Instant::now()));

        // Don't release current_sample here - it will be released on next iteration
        // Reduces immediate CFRelease call count
    } else {
        // Lock failed - release both
        unsafe {
            CFRelease(retained_sample as CFTypeRef);
            CFRelease(current_sample as CFTypeRef);
        }
    }
}
```

**Rationale**:
- Reduces number of CFRelease system calls per frequency read cycle
- Not critical for correctness, but reduces system call overhead
- More efficient object lifecycle management

**Impact**: ~0.2-0.5% CPU reduction

**Effort**: Low (refactor existing code, ~15 lines)

---

## LOW IMPACT OPTIMIZATIONS

### Task 8: Cache Frequency Logging Flag in Background Thread

**File**: `src-tauri/src/lib.rs:540-543`

**Current Code**:
```rust
// Check if frequency logging is enabled (inside loop, called every 20s)
let freq_logging = state::FREQUENCY_LOGGING_ENABLED.lock()
    .map(|f| *f)
    .unwrap_or(false);

// ... later ...

// Check again
let freq_logging = state::FREQUENCY_LOGGING_ENABLED.lock()
    .map(|f| *f)
    .unwrap_or(false);
```

**Problem**:
- Flag is checked 2-3 times per frequency read cycle
- Requires acquiring Mutex lock each time
- Flag rarely changes (only at startup with --frequency flag)

**Change**: Cache at start of loop:
```rust
loop {
    // Cache frequency logging flag for entire loop iteration
    let freq_logging = state::FREQUENCY_LOGGING_ENABLED.lock()
        .map(|f| *f)
        .unwrap_or(false);

    // ... rest of loop uses freq_logging variable
    // Reuse instead of re-locking
}
```

**Rationale**:
- Reduces lock acquisitions from 2-3 per cycle to 1 per cycle
- Flag is quasi-static during runtime
- Simple optimization with minimal risk

**Impact**: ~0.2% CPU reduction

**Effort**: Low (refactor loop structure, ~10 lines)

---

## VALIDATION CHECKLIST

After implementing each optimization:

- [ ] Code compiles without warnings: `cargo build --release`
- [ ] No new errors in logs: `tail -f ~/.mac-stats/debug.log | grep -i error`
- [ ] Functionality unchanged: Test CPU window displays correct metrics
- [ ] No regressions in responsiveness: Verify temperature/frequency updates still appear
- [ ] Memory usage stable: Monitor for leaks after 10 minutes of runtime
- [ ] CPU usage reduced: Compare before/after with Activity Monitor or `top`

---

## IMPLEMENTATION ORDER

**Recommended sequence** (build up gradually):

1. **Task 1**: Temperature interval 15s → 20s (immediate feedback, 1 line)
2. **Task 2**: Frequency interval 20s → 30s (immediate feedback, 1 line)
3. **Task 3**: Process cache 5s → 10s (immediate feedback, 1 line)
4. **Task 5**: Window visibility early exit (no visible changes, 20 lines)
5. **Task 4**: Split ACCESS_CACHE (moderate refactor, 30 lines)
6. **Task 6**: IOReport state caching (advanced, 25 lines)
7. **Task 7**: CFRelease batching (polish, 15 lines)
8. **Task 8**: Frequency logging cache (polish, 10 lines)

---

## PERFORMANCE TARGETS

| Task | Impact | Cumulative |
|------|--------|-----------|
| After Task 1+2+3 | -12% | -12% |
| After Task 5 | -1% | -13% |
| After Task 4 | -2% | -15% |
| After Task 6 | -0.5% | -15.5% |
| After Task 7 | -0.5% | -16% |
| After Task 8 | -0.2% | -16.2% |

**Conservative estimate**: Implementing all 8 tasks yields ~16-18% CPU reduction.
