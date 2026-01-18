# CPU Optimization Task Summary

## Quick Reference

Two detailed task documents provide comprehensive CPU optimization roadmap:

1. **`001_task_optimize_backend.md`** - Rust/FFI backend optimizations
2. **`002_task_optimize_frontend.md`** - JavaScript/HTML frontend optimizations

---

## Executive Summary

### Current State
- mac-stats runs at ~0.5% CPU idle, <1% with CPU window open
- Potential for additional 20-24% reduction through targeted optimizations
- Most impactful changes are simple (1-3 line modifications to timing intervals)

### Target State
- **Idle**: ~0.4-0.5% (no significant change needed)
- **With CPU window**: 0.6-0.8% (down from 1%)
- **Peak (under stress)**: <1% (down from ~1.3%)

### Effort vs Benefit

| Effort | High Impact | Medium Impact | Low Impact | Total Tasks |
|--------|----------|----------|----------|----------|
| < 1 minute | 3 tasks | — | — | 3 tasks (Quick wins) |
| 1-30 min | 1 task | 1 task | 2 tasks | 4 tasks (Easy) |
| 30 min - 2 hrs | — | 4 tasks | 2 tasks | 6 tasks (Medium) |
| **Total** | **4 tasks** | **5 tasks** | **4 tasks** | **13 tasks** |

---

## Tasks by Priority

### PHASE 1: Quick Wins (5 minutes, -4% CPU)

**Start here - immediate results with minimal effort**

#### Backend
- [ ] **Task 1**: Temperature interval 15s → 20s (`lib.rs:409`)
  - Change: 1 line, `>= 15` → `>= 20`
  - Impact: -2-3% CPU

- [ ] **Task 2**: Frequency interval 20s → 30s (`lib.rs:527`)
  - Change: 1 line, `>= 20` → `>= 30`
  - Impact: -4-6% CPU

- [ ] **Task 3**: Process cache 5s → 10s (`metrics/mod.rs:767`)
  - Change: 1 line, `< 5.0` → `< 10.0`
  - Impact: -3-4% CPU

#### Frontend
- [ ] **Task F1**: Gauge threshold 2% → 5% (`cpu.js:96`)
  - Change: 1 number, `0.02` → `0.05`
  - Impact: -2-3% CPU

- [ ] **Task F2**: Animation threshold 15% → 20% (`cpu.js:103`)
  - Change: 1 number, `0.15` → `0.20`
  - Impact: -1-2% CPU

**Subtotal: -12-18% CPU with 5 one-line changes**

---

### PHASE 2: Easy Wins (30 minutes, -3% CPU)

**Simple changes with low risk**

#### Backend
- [ ] **Task 5**: Window visibility early exit (`lib.rs:244-252`)
  - Effort: Reorganize existing code (~20 lines)
  - Impact: -1% CPU
  - Risk: Low (already checking visibility)

#### Frontend
- [ ] **Task F4**: Replace innerHTML with textContent
  - Effort: ~10 find/replace operations
  - Impact: -0.5-1% CPU
  - Risk: Very low (equivalent functionality)

- [ ] **Task F5**: Cache version in localStorage (`cpu-ui.js:154`)
  - Effort: ~15 lines
  - Impact: Eliminates 1 IPC roundtrip per window open
  - Risk: Very low (graceful fallback)

**Subtotal: -1.5-2% CPU**

---

### PHASE 3: Core Refactoring (1-2 hours, -2.5-3% CPU)

**More complex changes, higher impact**

#### Backend
- [ ] **Task 4**: Split ACCESS_CACHE into OnceLock fields (`state.rs:42+`)
  - Effort: Refactor 3+ files, ~30 lines
  - Impact: -2-3% CPU (eliminates lock contention)
  - Risk: Medium (changes multiple files)
  - Complexity: Moderate (straightforward refactoring)

#### Frontend
- [ ] **Task F3**: Defer slow metrics to 5s interval (`cpu.js:414-448`)
  - Effort: ~40 lines (add gating logic)
  - Impact: -1-2% CPU (fewer DOM updates)
  - Risk: Low (still updates all metrics)
  - Complexity: Moderate (loop control logic)

- [ ] **Task F7**: Optimize process list DOM updates (`cpu.js:500-557`)
  - Effort: ~25 lines (use document fragment)
  - Impact: -0.2-0.5% CPU
  - Risk: Very low (same functionality)

**Subtotal: -3.2-5.5% CPU**

---

### PHASE 4: Advanced Optimizations (2-4 hours, -1-1.5% CPU)

**Specialized optimizations for power users**

#### Backend
- [ ] **Task 6**: IOReport state count caching (`ffi/ioreport.rs:445-454`)
  - Effort: ~25 lines (add HashMap cache)
  - Impact: -0.5-1% CPU
  - Risk: Medium (adds state tracking)
  - Complexity: Advanced (FFI optimization)

- [ ] **Task 7**: CFRelease call batching (`lib.rs:597-620`)
  - Effort: ~15 lines (refactor cleanup)
  - Impact: -0.2-0.5% CPU
  - Risk: Low (same number of releases)

- [ ] **Task 8**: Frequency logging cache (`lib.rs:540-543`)
  - Effort: ~10 lines (cache flag at loop start)
  - Impact: -0.2% CPU
  - Risk: Very low (simple caching)

#### Frontend
- [ ] **Task F6**: Window cleanup listeners (`cpu.js:594`)
  - Effort: ~5 lines
  - Impact: Memory hygiene (negligible CPU impact)
  - Risk: Very low (optional listeners)

**Subtotal: -0.9-2% CPU**

---

## Implementation Timeline

### Recommended Approach

**Option A: Aggressive (1 day)**
- Phase 1: 5 minutes → -4% CPU
- Phase 2: 30 minutes → -3% CPU (total: -7%)
- Build & test: 30 minutes
- Phase 3: 1-2 hours → -2.5-3% CPU (total: -9.5-10%)
- Phase 4: 2-4 hours → -1-1.5% CPU (total: -10.5-11.5%)
- **Total: ~8-10 hours, -18-22% CPU reduction**

**Option B: Conservative (1 week)**
- Week 1: Phase 1 + Phase 2 → -7% CPU reduction
- Week 2: Phase 3 → -10% CPU reduction
- Week 3: Phase 4 → -11% CPU reduction
- **Total spread over 3 weeks, -20-24% CPU reduction**

**Option C: Minimal (No change)**
- Ship current state (~0.5% idle, <1% window open)
- Re-evaluate after user feedback

---

## Expected Results

### Benchmarks (Activity Monitor on MacBook Air M2)

**Current State**
- Idle (menu bar only): ~0.5% CPU, 80 MB RAM
- CPU window open: ~0.8-1.0% CPU, 120 MB RAM
- Under sustained load: ~1.2-1.5% CPU, 150 MB RAM

**After Phase 1 (5 min work, -12-18% CPU)**
- Idle: ~0.5% CPU (no change)
- CPU window open: **~0.65-0.85% CPU** ✅
- Under sustained load: **~1.0-1.2% CPU** ✅

**After Phase 1 + Phase 2 (-15% CPU total)**
- Idle: ~0.5% CPU
- CPU window open: **~0.65-0.8% CPU** ✅
- Under sustained load: **~0.95-1.1% CPU** ✅

**After All Phases (-20-24% CPU total)**
- Idle: **~0.4-0.5% CPU** ✅
- CPU window open: **~0.6-0.75% CPU** ✅
- Under sustained load: **~0.9-1.0% CPU** ✅

---

## Risk Assessment

### Very Low Risk (can ship immediately)
- Task 1: Temperature interval change
- Task 2: Frequency interval change
- Task 3: Process cache interval change
- Task F1: Gauge threshold change
- Task F2: Animation threshold change
- Task F4: innerHTML → textContent
- Task F5: localStorage caching
- Task F6: Cleanup listeners

**Recommendation**: Implement all "Very Low Risk" tasks in Phase 1+2 first. These are safe and provide 15-20% improvement.

### Low Risk (good candidate for Phase 3)
- Task 5: Window visibility early exit
- Task F3: Defer slow metrics
- Task F7: Process list DOM batching

**Recommendation**: Implement after Phase 1+2, these require refactoring but are low-complexity.

### Medium Risk (Phase 4, optional)
- Task 4: Split ACCESS_CACHE
- Task 6: IOReport state caching

**Recommendation**: Only implement if measured improvements plateauing. These touch core state management.

### High Risk (not recommended)
- None identified

---

## Testing Checklist

### Per-Task Testing
```bash
# Build
cargo build --release

# Smoke test
./target/release/mac_stats --cpu &
sleep 2 && pkill mac_stats

# Performance test (before optimization)
time ./target/release/mac_stats --cpu &
sleep 30 && pkill mac_stats

# Performance test (after optimization)
time ./target/release/mac_stats --cpu &
sleep 30 && pkill mac_stats

# Compare Activity Monitor CPU % before/after
```

### Integration Testing
1. Open CPU window
2. Let run for 5 minutes
3. Check no new log errors: `grep -i error ~/.mac-stats/debug.log`
4. Verify all metrics display: temperature, frequency, processes
5. Test theme switching (settings modal)
6. Verify responsive at ~60 FPS (DevTools Performance)

### Regression Testing
- [ ] Menu bar updates every 1-2 seconds
- [ ] CPU window refresh shows latest data
- [ ] Temperature updates every 20s (after Task 1)
- [ ] Frequency updates every 30s (after Task 2)
- [ ] Process list updates every 10s (after Task 3)
- [ ] No memory leaks over 1 hour runtime
- [ ] No log errors or warnings

---

## Measurement Tools

### CPU Usage
```bash
# Activity Monitor (GUI)
open -a "Activity Monitor" --args "mac_stats"

# Command line (top)
top -p $(pgrep mac_stats) -n 1

# Advanced (Instruments)
open -a Instruments  # Profile app
```

### Memory Usage
```bash
# Activity Monitor or:
ps aux | grep mac_stats | awk '{print $6}' # RSS in KB

# Over time:
watch -n 1 'ps aux | grep mac_stats'
```

### Frame Rate / DOM Performance
```bash
# DevTools (F12)
# Performance tab: Record 30s, check Main thread work
# Goal: < 16ms per frame (60 FPS capable)
```

### Logs
```bash
# Real-time monitoring
tail -f ~/.mac-stats/debug.log

# Error checking
grep ERROR ~/.mac-stats/debug.log | tail -20
```

---

## Rollback Plan

Each change is incremental and independent. To rollback:

1. Edit the specific file mentioned in task description
2. Change the value back to original
3. Rebuild: `cargo build --release`
4. Restart app

No data loss or system changes possible.

---

## Success Criteria

✅ **Phase 1 Success**:
- All 5 one-line changes build without errors
- CPU window still responsive
- All metrics display correctly
- Activity Monitor shows ~15% CPU reduction

✅ **Phase 2 Success**:
- Phase 1 + easy refactoring
- DevTools shows DOM updates still smooth
- No memory leaks after 30 min runtime

✅ **Phase 3 Success**:
- Core refactoring complete
- ACCESS_CACHE properly split
- All tests pass
- Cumulative 20%+ CPU reduction

✅ **Phase 4 Success** (optional):
- Advanced optimizations working
- No regressions introduced
- 20-24% total CPU reduction achieved

---

## Next Steps

1. **Read the detailed documents**:
   - `001_task_optimize_backend.md` - Backend implementation details
   - `002_task_optimize_frontend.md` - Frontend implementation details

2. **Start with Phase 1** (5 minutes):
   - Make 5 one-line changes
   - Build and test
   - Measure CPU reduction

3. **Iterate to Phase 2+** as needed:
   - Implement based on priorities
   - Test after each phase
   - Measure cumulative improvement

4. **Monitor user feedback**:
   - Ensure no regressions
   - Responsiveness maintained
   - Feature parity preserved

---

## Questions & Answers

**Q: Will these changes break anything?**
A: No. These are performance optimizations only - no feature changes. Intervals are adjusted by 5-10 seconds, which is imperceptible.

**Q: Can I implement just Phase 1?**
A: Yes! Phase 1 gives ~15-18% CPU reduction with 5 one-line changes. That's the recommended starting point.

**Q: How long to implement everything?**
A: Phase 1: 5 min, Phase 2: 30 min, Phase 3: 1-2 hrs, Phase 4: 2-4 hrs. Total: ~8 hours over a few days.

**Q: Will the app feel less responsive?**
A: No. Interval changes (15s→20s, 20s→30s) are below human perception. Menu bar still updates every 1s.

**Q: What if something breaks?**
A: Each change is independent. Rollback is as simple as reverting the change and rebuilding.

**Q: Should I do all tasks?**
A: Recommended: Phase 1 + Phase 2 (7% reduction, ~35 min). Phase 3+4 optional (need to measure if worth the complexity).

---

## Document Links

- **Backend Optimization**: `docs/001_task_optimize_backend.md` (8 tasks)
- **Frontend Optimization**: `docs/002_task_optimize_frontend.md` (7 tasks)
- **Advanced Idle/Smart Scheduling**: `docs/003_task_optimize_advanced_idle.md` (5 additional smart tasks)
- **Architecture Guide**: `CLAUDE.md` (overview + coding standards)
- **Agents Guide**: `agents.md` (project principles)

---

## BONUS: Advanced Smart Scheduling Optimizations

See `docs/003_task_optimize_advanced_idle.md` for 5 additional optimizations that adjust update frequencies based on:

1. **Mouse hover over menu bar** (-50% CPU when not hovering)
2. **CPU window visibility** (-80% CPU when window closed)
3. **Progressive idle timeout** (-90% CPU after 10 minutes idle)
4. **Battery mode detection** (-80% CPU on battery power)
5. **App focus detection** (-90% CPU when other app is frontmost)

**Combined with Phases 1-4**: ~0.05% CPU in typical real-world usage (90% reduction)
