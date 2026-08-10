# task-004: Scheduler: reduce log and disk churn when polling schedules

**Source:** `~/.mac-stats/debug.log` (monitoring run)

**Observed:**
- `INFO Scheduler: loaded 2 entries from "/Users/raro42/.mac-stats/schedules.json"` every ~2 seconds (when `scheduler_check_interval_secs` is set to 2 or similar).
- Same message repeated every loop iteration; next run is far in the future (e.g. next day), so the loop sleeps only up to `check_interval_secs` then reloads again.

**Problem:** The scheduler loop calls `load_schedules()` every iteration and logs at INFO every time. When the next run is far away, the loop sleeps only `min(wait_ms, check_interval_secs * 1000)` ms, so with a short interval (e.g. 2s) it reloads and logs every 2s. This causes:
- Noisy logs (dozens of "loaded N entries" per minute).
- Unnecessary disk reads of `schedules.json` when nothing changed.

**Required:**
1. **Log level or frequency:** Do not log "Scheduler: loaded N entries from ..." at INFO on every load. Options:
   - Log at DEBUG, or
   - Log at INFO only when the number of entries or the set of next-run times actually changes (e.g. compare with previous load).
2. **Optional:** When next run is far in the future (e.g. > 5 minutes), sleep in larger chunks (e.g. cap sleep at 60s) and only call `load_schedules()` after each sleep or when file mtime changes. This reduces disk reads; current design already checks mtime after sleep, so the main fix is the log level or conditional log.

**Relevant code:**
- `src-tauri/src/scheduler/mod.rs`: `scheduler_loop()`, `load_schedules()` (the `info!(...)` at lines 136–140).

**Acceptance:** Default or short `scheduler_check_interval_secs` no longer produces a full "loaded N entries" INFO line every few seconds; either DEBUG-only or only when schedule set/next-run changes.
