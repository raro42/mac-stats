# Brutal CPU Optimization Plan - Native macOS APIs

## Analysis Summary

The other AI's recommendations are **spot-on** and align with professional macOS monitoring apps. However, we need to balance:
- **Impact**: How much CPU will this save?
- **Complexity**: How hard is it to implement?
- **Risk**: Will this break existing functionality?

## Current State Assessment

### ✅ What's Working
- Basic metrics collection (CPU, RAM, temperature)
- Caching system (reduces redundant calls)
- Rate limiting (prevents excessive calls)
- WebKit rendering optimizations (just completed)

### ❌ Critical Issues
1. **Shell commands** (`sh -c`) for sysctl/system_profiler - brittle, slow, risky
2. **IOReport used directly** - not through safety wrappers, crash-prone
3. **"Nominal frequency" is misleading** - not real CPU frequency
4. **No crash capture** - can't debug crashes
5. **sysinfo for processes** - slower than libproc, less control

## Phased Implementation Plan

---

## Phase 1: Quick Wins (High Impact, Low Risk) - **START HERE**

**Goal**: Replace shell commands, fix immediate issues  
**Estimated CPU savings**: 0.5-1%  
**Time**: 2-3 hours  
**Risk**: Low

### 1.1 Replace Shell Commands with Direct sysctl Calls
**Current**: `sh -c "sysctl -n hw.tbfrequency"`
**Target**: Direct sysctl FFI or `std::process::Command` with full path

**Files to change**:
- `src-tauri/src/metrics/mod.rs`:
  - `get_chip_info()` - replace `system_profiler` shell call
  - `get_nominal_frequency()` - replace `sysctl` shell calls

**Implementation**:
```rust
// Instead of: Command::new("sh").arg("-c").arg("sysctl -n hw.tbfrequency")
// Use: Command::new("/usr/sbin/sysctl").arg("-n").arg("hw.tbfrequency")
```

**Benefits**:
- Faster (no shell overhead)
- Safer (no PATH/env issues)
- More reliable (direct binary calls)

### 1.2 Fix "Nominal Frequency" Documentation
**Current**: Returns `hw.tbfrequency * kern.clockrate.hz` as "frequency"
**Target**: Clearly label as "nominal/base frequency, not dynamic"

**Files to change**:
- `src-tauri/src/metrics/mod.rs` - `get_nominal_frequency()` comments
- Frontend - clarify this is base frequency, not current

**Benefits**:
- Users understand what they're seeing
- Prevents confusion about "stuck" frequency

### 1.3 Add Explicit "Unknown" States
**Current**: Returns `0.0` when cache is locked or unavailable
**Target**: Return `Option<f32>` or add "stale" flags

**Files to change**:
- `src-tauri/src/metrics/mod.rs` - `CpuDetails` struct
- Frontend - handle "unknown" states gracefully

**Benefits**:
- Better error visibility
- Can detect contention issues

---

## Phase 2: IOReport Hardening (Medium Impact, Medium Risk)

**Goal**: Consolidate IOReport usage, add crash protection  
**Estimated CPU savings**: 0.3-0.5% (from fewer crashes/restarts)  
**Time**: 4-6 hours  
**Risk**: Medium (IOReport is crash-prone)

### 2.1 Route All IOReport Through ffi/ioreport.rs
**Current**: Direct `extern "C"` calls in `lib.rs`
**Target**: All IOReport calls go through safety wrappers

**Files to change**:
- `src-tauri/src/lib.rs` - replace direct IOReport calls
- `src-tauri/src/ffi/ioreport.rs` - complete wrapper implementations
- Remove `#[allow(dead_code)]` from IOReport wrappers

**Benefits**:
- Consistent null/type checks
- Single place to debug IOReport issues
- Better error handling

### 2.2 Fix CF Ownership/Lifetimes
**Current**: Raw `CFDictionaryRef` returned, no RAII
**Target**: Return owned `CFDictionary` with proper `Drop`

**Files to change**:
- `src-tauri/src/ffi/ioreport.rs` - wrapper return types
- Use `CFDictionary` instead of `CFDictionaryRef`

**Benefits**:
- Prevents memory leaks
- Safer lifetime management

### 2.3 Add Breadcrumb Logging for IOReport
**Current**: No visibility into IOReport failures
**Target**: Ring buffer of last 200 IOReport operations

**Files to create**:
- `src-tauri/src/ffi/ioreport_breadcrumbs.rs` - ring buffer
- Log: channel names, CF types, pointers, return codes

**Benefits**:
- Can debug crashes after they happen
- Understand what IOReport was doing when it crashed

---

## Phase 3: Crash Capture (Medium Impact, Low Risk)

**Goal**: Capture crashes for debugging  
**Estimated CPU savings**: 0% (but saves debugging time)  
**Time**: 3-4 hours  
**Risk**: Low

### 3.1 Add Panic Hook
**Current**: No panic capture
**Target**: Panic hook that writes backtrace + breadcrumbs

**Files to change**:
- `src-tauri/src/main.rs` - add panic hook
- Write to: `~/.mac-stats/crashes/panic-{timestamp}.log`

**Benefits**:
- Can debug Rust panics
- See backtrace when app crashes

### 3.2 Add Crash Log Collection
**Current**: No crash log collection
**Target**: Read macOS crash logs on startup

**Files to create**:
- `src-tauri/src/crash_logs.rs` - collect crash logs
- Read from: `/Library/Logs/DiagnosticReports/`, `/var/root/Library/Logs/DiagnosticReports/`

**Benefits**:
- Can see what caused EXC_BAD_ACCESS
- Understand foreign exception crashes

---

## Phase 4: Native APIs (High Impact, High Complexity) - **OPTIONAL**

**Goal**: Replace sysinfo with native Mach/libproc APIs  
**Estimated CPU savings**: 1-2%  
**Time**: 8-12 hours  
**Risk**: High (major refactor)

### 4.1 Replace sysinfo with libproc for Processes
**Current**: `sysinfo::System::refresh_processes()`
**Target**: `libproc` FFI calls (`proc_listpids`, `proc_pid_rusage`)

**Files to create**:
- `src-tauri/src/native/proc.rs` - libproc wrappers
- `src-tauri/src/native/mach.rs` - Mach API wrappers

**Benefits**:
- Faster (no sysinfo overhead)
- More control (delta-based CPU calculation)
- Better for top-N selection

### 4.2 Use Mach APIs for CPU Usage
**Current**: `sysinfo::System::global_cpu_usage()`
**Target**: `host_statistics64(HOST_CPU_LOAD_INFO)`

**Files to change**:
- `src-tauri/src/native/mach.rs` - Mach CPU wrappers
- `src-tauri/src/metrics/mod.rs` - use Mach instead of sysinfo

**Benefits**:
- Faster (direct kernel calls)
- More accurate (delta-based)
- Less memory allocation

### 4.3 Implement Delta-Based Process CPU
**Current**: `sysinfo` gives instantaneous CPU
**Target**: Track PID → last CPU time, compute deltas

**Files to change**:
- `src-tauri/src/native/proc.rs` - PID → CPU time map
- Compute: `(current_cpu_time - last_cpu_time) / elapsed_time`

**Benefits**:
- More accurate CPU percentages
- Matches Activity Monitor behavior

---

## Phase 5: Architecture Refactor (Very High Impact, Very High Complexity) - **FUTURE**

**Goal**: Split into sampler/reducer/UI publisher  
**Estimated CPU savings**: 2-3%  
**Time**: 20-30 hours  
**Risk**: Very High (major architecture change)

### 5.1 Create Helper Process for IOReport/SMC
**Current**: IOReport/SMC in main process
**Target**: Separate helper process (isolated)

**Files to create**:
- `src-tauri/src/helper/main.rs` - helper process
- Communication via IPC (Tauri events or custom)

**Benefits**:
- IOReport crashes don't kill main app
- Better isolation
- Can restart helper independently

### 5.2 Implement Sampler/Reducer/Publisher Pattern
**Current**: Everything in one thread
**Target**: 
- Sampler: collects raw counters (250-500ms)
- Reducer: computes deltas/percentages (on-demand)
- Publisher: emits snapshots (2-10 Hz)

**Files to create**:
- `src-tauri/src/sampler.rs` - raw data collection
- `src-tauri/src/reducer.rs` - delta computation
- `src-tauri/src/publisher.rs` - UI updates

**Benefits**:
- Clear separation of concerns
- Optimized update rates
- Better CPU efficiency

---

## Recommended Implementation Order

### Immediate (This Week)
1. ✅ **Phase 1.1**: Replace shell commands (2-3 hours)
2. ✅ **Phase 1.2**: Fix nominal frequency docs (30 min)
3. ✅ **Phase 1.3**: Add explicit "unknown" states (1-2 hours)

### Short Term (Next Week)
4. ✅ **Phase 2.1**: Route IOReport through wrappers (3-4 hours)
5. ✅ **Phase 2.2**: Fix CF ownership (2-3 hours)
6. ✅ **Phase 3.1**: Add panic hook (1-2 hours)

### Medium Term (Next Month)
7. ⚠️ **Phase 2.3**: Breadcrumb logging (2-3 hours)
8. ⚠️ **Phase 3.2**: Crash log collection (2-3 hours)
9. ⚠️ **Phase 4.1**: libproc for processes (4-6 hours) - **IF** process collection is still slow

### Long Term (Future)
10. 🔮 **Phase 4.2-4.3**: Full Mach API migration (8-12 hours)
11. 🔮 **Phase 5**: Architecture refactor (20-30 hours)

---

## Decision Points

### Should we do Phase 4 (Native APIs)?
**Pros**:
- Faster (1-2% CPU savings)
- More accurate
- Professional-grade

**Cons**:
- High complexity
- More FFI code to maintain
- sysinfo works fine for most use cases

**Recommendation**: **Only if** process collection is still a bottleneck after Phase 1-3.

### Should we do Phase 5 (Helper Process)?
**Pros**:
- Best isolation
- Can restart helper independently
- Matches Stats app architecture

**Cons**:
- Very high complexity
- IPC overhead
- More moving parts

**Recommendation**: **Only if** IOReport crashes are frequent and blocking.

---

## Expected CPU Usage After Each Phase

| Phase | Current | After Phase | Savings |
|-------|---------|-------------|---------|
| Baseline | ~1.3% | - | - |
| Phase 1 | ~1.3% | ~0.8% | -0.5% |
| Phase 2 | ~0.8% | ~0.5% | -0.3% |
| Phase 3 | ~0.5% | ~0.5% | 0% (debugging) |
| Phase 4 | ~0.5% | ~0.3% | -0.2% |
| Phase 5 | ~0.3% | ~0.1% | -0.2% |

**Target**: <0.5% average CPU usage (currently 1.3%)

---

## Next Steps

1. **Review this plan** - confirm priorities
2. **Start Phase 1.1** - replace shell commands
3. **Test after each phase** - measure CPU usage
4. **Decide on Phase 4/5** - based on results
