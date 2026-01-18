# CPU Optimization Implementation Checklist

Use this checklist to track progress through the optimization tasks.

---

## PHASE 1: Quick Wins (5 minutes, -12-18% CPU)

### Backend Changes (3 tasks)

- [ ] **Task 1: Temperature Interval 15s → 20s**
  - File: `src-tauri/src/lib.rs:409`
  - Change: Line 409, `>= 15` → `>= 20`
  - Status: ☐ Not started ☐ In progress ☐ Complete
  - Tested: ☐ Compiles ☐ No new errors ☐ CPU reduced

- [ ] **Task 2: Frequency Interval 20s → 30s**
  - File: `src-tauri/src/lib.rs:527`
  - Change: Line 527, `>= 20` → `>= 30`
  - Status: ☐ Not started ☐ In progress ☐ Complete
  - Tested: ☐ Compiles ☐ No new errors ☐ CPU reduced

- [ ] **Task 3: Process Cache 5s → 10s**
  - File: `src-tauri/src/metrics/mod.rs:767`
  - Change: Line 767, `< 5.0` → `< 10.0`
  - Status: ☐ Not started ☐ In progress ☐ Complete
  - Tested: ☐ Compiles ☐ No new errors ☐ CPU reduced

### Frontend Changes (2 tasks)

- [ ] **Task F1: Gauge Threshold 2% → 5%**
  - File: `src-tauri/dist/cpu.js:96`
  - Change: Line 96, `0.02` → `0.05`
  - Status: ☐ Not started ☐ In progress ☐ Complete
  - Tested: ☐ No JS errors ☐ Gauge smooth ☐ CPU reduced

- [ ] **Task F2: Animation Threshold 15% → 20%**
  - File: `src-tauri/dist/cpu.js:103`
  - Change: Line 103, `0.15` → `0.20`
  - Status: ☐ Not started ☐ In progress ☐ Complete
  - Tested: ☐ No JS errors ☐ Animation smooth ☐ CPU reduced

### Phase 1 Completion

- [ ] All 5 changes implemented
- [ ] Build successful: `cargo build --release`
- [ ] No compiler errors
- [ ] No warnings (unless pre-existing)
- [ ] App starts: `./target/release/mac_stats --cpu`
- [ ] CPU window opens and displays metrics
- [ ] All gauges update smoothly
- [ ] Activity Monitor shows CPU reduced
- [ ] Ready for Phase 2

**Phase 1 Complete**: `________________` Date

---

## PHASE 2: Easy Wins (30 minutes, -1.5-2% CPU)

### Backend Changes (1 task)

- [ ] **Task 5: Window Visibility Early Exit**
  - File: `src-tauri/src/lib.rs:244-252`
  - Changes: Move window visibility check before SMC connection attempt (~20 lines)
  - Status: ☐ Not started ☐ In progress ☐ Complete
  - Tested: ☐ Compiles ☐ Window close releases resources ☐ CPU reduced

### Frontend Changes (3 tasks)

- [ ] **Task F4: Replace innerHTML with textContent**
  - Files: `src-tauri/dist/cpu.js:268, 285, 325, 379`
  - Changes: Replace 5-6 innerHTML assignments with textContent (~10 lines)
  - Status: ☐ Not started ☐ In progress ☐ Complete
  - Tested: ☐ No JS errors ☐ Metrics display ☐ DOM updates smooth

- [ ] **Task F5: Cache Version in localStorage**
  - File: `src-tauri/dist/cpu-ui.js:146-175`
  - Changes: Add caching logic (~15 lines)
  - Status: ☐ Not started ☐ In progress ☐ Complete
  - Tested: ☐ Version cached ☐ No Tauri invoke on reload ☐ Fallback works

- [ ] **Task F6: Add Cleanup Listeners**
  - File: `src-tauri/dist/cpu.js:594`
  - Changes: Add beforeunload cleanup (~5 lines)
  - Status: ☐ Not started ☐ In progress ☐ Complete
  - Tested: ☐ Cleanup fires on window close ☐ No memory leaks

### Phase 2 Integration Testing

- [ ] Build successful: `cargo build --release`
- [ ] No new compiler errors
- [ ] App runs: `./target/release/mac_stats --cpu`
- [ ] CPU window responsive
- [ ] All metrics display correctly
- [ ] Theme switching works
- [ ] No console errors (F12 > Console)
- [ ] Activity Monitor shows further CPU reduction
- [ ] Run for 5 minutes without errors

**Phase 2 Complete**: `________________` Date

---

## PHASE 3: Core Refactoring (1-2 hours, -2.5-3% CPU)

### Backend Changes (1 task)

- [ ] **Task 4: Split ACCESS_CACHE into OnceLock Fields**
  - Files: `src-tauri/src/state.rs:42`, `src-tauri/src/metrics/mod.rs:178-223, 354-395`, `src-tauri/src/lib.rs:264-271, 384-391, 688-695`
  - Changes: Replace single Mutex<(bool,bool,bool,bool)> with 4 OnceLock fields (~30 lines total)
  - Status: ☐ Not started ☐ In progress ☐ Complete
  - Substeps:
    - [ ] Add 4 OnceLock fields to state.rs
    - [ ] Update can_read_temperature() in metrics/mod.rs
    - [ ] Update can_read_frequency() in metrics/mod.rs
    - [ ] Update can_read_cpu_power() in metrics/mod.rs
    - [ ] Update can_read_gpu_power() in metrics/mod.rs
    - [ ] Update cache writes in lib.rs (3 locations)
  - Tested: ☐ Compiles ☐ No new errors ☐ Lock contention reduced ☐ CPU reduced

### Frontend Changes (2 tasks)

- [ ] **Task F3: Defer Slow Metrics to 5s Interval**
  - File: `src-tauri/dist/cpu.js:414-448`
  - Changes: Split updates into fast (1s) and slow (5s) (~40 lines)
  - Metrics to defer: load 1m/5m/15m, uptime, CPU power, GPU power
  - Status: ☐ Not started ☐ In progress ☐ Complete
  - Tested: ☐ Fast metrics update 1s ☐ Slow metrics update 5s ☐ All metrics display

- [ ] **Task F7: Optimize Process List DOM Updates**
  - File: `src-tauri/dist/cpu.js:500-557`
  - Changes: Use document fragment and textContent (~25 lines)
  - Status: ☐ Not started ☐ In progress ☐ Complete
  - Tested: ☐ Process list updates smoothly ☐ Fewer reflows ☐ DOM efficient

### Phase 3 Integration Testing

- [ ] Build successful: `cargo build --release`
- [ ] No compiler warnings (unless pre-existing)
- [ ] All ACCESS_CACHE updates working correctly
- [ ] Backward compatibility maintained (no breaking changes)
- [ ] App runs: `./target/release/mac_stats --cpu`
- [ ] All metrics display and update correctly
- [ ] Slow metrics (load average, uptime) visible and correct
- [ ] No console errors or warnings
- [ ] DevTools Performance shows improved frame times
- [ ] Run for 10 minutes, check logs for errors
- [ ] Activity Monitor shows cumulative ~20% CPU reduction

**Phase 3 Complete**: `________________` Date

---

## PHASE 4: Advanced Optimizations (2-4 hours, -1-1.5% CPU)

### Backend Changes (3 tasks)

- [ ] **Task 6: IOReport State Count Caching**
  - File: `src-tauri/src/ffi/ioreport.rs:445-454`
  - Changes: Add HashMap cache for state counts (~25 lines)
  - Status: ☐ Not started ☐ In progress ☐ Complete
  - Tested: ☐ Compiles ☐ State counts cached ☐ No functional change

- [ ] **Task 7: CFRelease Call Batching**
  - File: `src-tauri/src/lib.rs:597-620`
  - Changes: Reduce redundant CFRelease calls (~15 lines)
  - Status: ☐ Not started ☐ In progress ☐ Complete
  - Tested: ☐ Compiles ☐ Memory managed correctly ☐ No leaks

- [ ] **Task 8: Frequency Logging Cache**
  - File: `src-tauri/src/lib.rs:540-543`
  - Changes: Cache flag at loop start instead of re-locking (~10 lines)
  - Status: ☐ Not started ☐ In progress ☐ Complete
  - Tested: ☐ Compiles ☐ Flag cached correctly ☐ Behavior unchanged

### Phase 4 Integration Testing

- [ ] Build successful: `cargo build --release`
- [ ] All advanced optimizations integrated
- [ ] App runs: `./target/release/mac_stats --cpu`
- [ ] Frequency logging works: `./target/release/mac_stats --cpu --frequency`
- [ ] No regression in any metrics
- [ ] Run for 15 minutes, monitor for errors
- [ ] Log file clean: `grep -i error ~/.mac-stats/debug.log`
- [ ] Memory stable: No growth over time
- [ ] Activity Monitor shows final ~20-24% CPU reduction

**Phase 4 Complete**: `________________` Date

---

## FINAL VALIDATION

### Correctness Testing

- [ ] All metrics display correctly
- [ ] Temperature updates (every 20s after Task 1)
- [ ] Frequency updates (every 30s after Task 2)
- [ ] Process list updates (every 10s after Task 3)
- [ ] Load averages display (updated every 5s after Task F3)
- [ ] Uptime display correct (updated every 5s after Task F3)
- [ ] Power metrics display (updated every 5s after Task F3)

### Performance Testing

- [ ] CPU idle: ~0.5% (no significant change expected)
- [ ] CPU window: ~0.6-0.8% (down from ~1%)
- [ ] Under load: ~0.9-1.0% (down from ~1.2-1.5%)
- [ ] Memory: ~120-150 MB (stable, no leaks)
- [ ] Responsiveness: All gauges update smoothly, no stuttering

### Logging & Debugging

- [ ] Run with verbose: `./target/release/mac_stats --cpu -vvv`
- [ ] Check debug log: `tail -100 ~/.mac-stats/debug.log`
- [ ] Verify no errors: `grep -i error ~/.mac-stats/debug.log`
- [ ] Verify update intervals as expected
- [ ] No unexpected lock contention warnings

### UI Testing

- [ ] Menu bar updates every 1-2 seconds
- [ ] CPU window opens smoothly
- [ ] All themes load without errors
- [ ] Settings modal works
- [ ] Theme switching responsive
- [ ] No visual glitches or stuttering
- [ ] DevTools Performance: < 16ms per frame (60 FPS capable)

### Regression Testing

- [ ] Compare before/after with same workload
- [ ] Verify CPU reduction measured (use Activity Monitor)
- [ ] Ensure all features still work
- [ ] No new bugs introduced
- [ ] Code review: All changes follow agents.md principles

---

## PERFORMANCE MEASUREMENTS

### Before Optimization
```
Date: _______________
Idle CPU: _____%
Window CPU: _____%
Load CPU: _____%
Memory: ________MB
Notes: ___________________________
```

### After Phase 1
```
Date: _______________
Idle CPU: _____%
Window CPU: _____%
Load CPU: _____%
Memory: ________MB
Reduction: ____% (from Phase 0)
Notes: ___________________________
```

### After Phase 2
```
Date: _______________
Idle CPU: _____%
Window CPU: _____%
Load CPU: _____%
Memory: ________MB
Reduction: ____% (from Phase 0)
Notes: ___________________________
```

### After Phase 3
```
Date: _______________
Idle CPU: _____%
Window CPU: _____%
Load CPU: _____%
Memory: ________MB
Reduction: ____% (from Phase 0)
Notes: ___________________________
```

### After Phase 4
```
Date: _______________
Idle CPU: _____%
Window CPU: _____%
Load CPU: _____%
Memory: ________MB
Reduction: ____% (from Phase 0)
Notes: ___________________________
```

---

## SIGN-OFF

**Developer**: ___________________

**Date Started**: ___________________

**Phase 1 Completed**: ___________________

**Phase 2 Completed**: ___________________

**Phase 3 Completed**: ___________________

**Phase 4 Completed**: ___________________

**Final Testing**: ___________________

**Code Review**: ___________________

**Ready for Release**: ___________________

---

## NOTES & ISSUES

(Use this space to track any issues or blockers during implementation)

```
Issue 1:
Description: _________________________________________
Resolution: _________________________________________
Date Resolved: ________

Issue 2:
Description: _________________________________________
Resolution: _________________________________________
Date Resolved: ________
```

---

## ROLLBACK PLAN (If Needed)

If any phase causes issues:

1. Identify problematic task(s)
2. Edit file to revert specific change
3. Rebuild: `cargo build --release`
4. Test again

**Git History** (if tracked):
```bash
# View changes
git diff src-tauri/src/lib.rs

# Revert specific file
git checkout src-tauri/src/lib.rs

# Rebuild
cargo build --release
```

---

## REFERENCE DOCUMENTS

- **Full Backend Tasks**: `docs/001_task_optimize_backend.md`
- **Full Frontend Tasks**: `docs/002_task_optimize_frontend.md`
- **Summary**: `docs/000_task_optimize_summary.md`
- **Architecture**: `CLAUDE.md`
- **Project Principles**: `agents.md`
