# CPU window hang — GPU `ioreg` on main thread (2026-09-15 ~13:45 CEST)

## Symptom

mac-stats UI “not responding” again. Process still alive (~1.8% CPU). User did not force-quit; agent killed PID 47904.

## Sample evidence

`sample mac_stats 3` showed the **main thread** stuck in:

`get_cpu_details` → `enrich_processes_with_gpu` → `gpu_usage_by_pid` → `Command::output` (`/usr/sbin/ioreg -r -l -c AGXAccelerator`)

Tauri URL-scheme IPC dispatches sync commands on the CFRunLoop. A wedged or huge `ioreg` dump beachballs the whole app.

Not Disk Cleanup this time (v0.1.1079 already capped that path).

## Fix (v0.1.1080)

In `metrics/gpu_processes.rs`:

1. **800 ms** timeout on `ioreg`; SIGKILL on overrun  
2. **In-flight guard** — stacked invokes return the cache  
3. On timeout / empty sample, **keep last good % cache**

## Related

- `docs/ops/2026-09-15-disk-cleanup-ui-hang.md` — earlier stall from `tmutil` / rebuild scans
