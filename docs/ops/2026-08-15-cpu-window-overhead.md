# CPU window overhead — 2026-08-15

## Question

Why does mac-stats sometimes show **>5% CPU** with the window open, while the target is **<2%** (ideally **<1%**)?

## Measurement (this machine, v0.1.428 tree before 0.1.429)

`./scripts/measure_performance.sh 20 1 window` with `--cpu`:

| Metric | Value |
|--------|-------|
| CPU avg | **0.21%** |
| CPU max | 0.60% |
| RSS | ~97 MB |

So steady-state window open is already under 1% when idle of agent work. Spikes come from stacked work, not a constant 5% floor.

## Hot paths found

1. **Process Details modal (every 2s)** called `refresh_processes(All)` (~8ms each) plus GPU map. That alone can spike Activity Monitor samples.
2. **`get_gpu_usage()`** also called `gpu_usage_by_pid()` → second `ioreg -l` (~20ms) every ~2s on top of the device-utilization `ioreg`.
3. **Frontend** polled `get_cpu_details` every **1s** while the backend rate-limits full work to **2s** (IPC + WebKit wakeups for little gain).
4. **Background metrics loop** slept **1s** while the window was visible (overlaps frontend).
5. Note: `/Applications` Info.plist can lag `Cargo.toml` (seen **0.1.368** label); trust the install script version check + restart.

## Fixes in v0.1.429

- Details: refresh **only the selected PID**.
- Details UI interval: **5s**.
- Frontend metrics: **2s**.
- Background loop with window open: **2s**.
- Per-process GPU sampler: not tied to GPU gauge; ioreg hold ~**3s** after warm-up.

## suggestd (separate from mac-stats)

`suggestd` is Apple **Core Suggestions**. A live `sample` showed **ProactiveHarvesting** on Biome **ThirdPartyApp** / Safari / Notes streams (not mac-stats). High CPU is Apple indexing/harvest; quitting mac-stats does not stop it. Optional: System Settings → Siri / Apple Intelligence / Suggestions, or wait for harvest to finish.
