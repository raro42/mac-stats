# 2026-08-15 — Per-process GPU usage (Top Processes)

## Need

When the GPU gauge hits ~100%, the Top Processes list only ranked by CPU, so the heavy GPU user was invisible.

## Approach

Apple Silicon exposes AGX `AGXDeviceUserClient` entries in IORegistry with:

- `IOUserClientCreator` → `pid N, name`
- `AppUsage[].accumulatedGPUTime` → cumulative GPU ns

Sample twice, divide delta by wall time → estimated GPU %. No sudo (same family of data Activity Monitor uses). Best-effort; multi-queue sums can exceed 100%.

## UI

Top Processes: **GPU** column (purple bar). List merges high-GPU PIDs and sorts by `max(cpu, gpu)`. Detail modal shows Current GPU.

## Files

- `src-tauri/src/metrics/gpu_processes.rs`
- `ProcessUsage.gpu` / enrich in `metrics/mod.rs`
- `src/cpu.js`, `src/agent-ops.css`

## Fix (v0.1.427)

Empty first-sample cache blocked warm-up; `ioreg` now uses `-l`. Gauge refresh also ticks the process sampler.
