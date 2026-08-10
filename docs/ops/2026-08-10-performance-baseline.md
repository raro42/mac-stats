# Performance baseline — 2026-08-10

Measured with `./scripts/measure_performance.sh` after fixing macOS sampling
(`ps %cpu` is lifetime average; Darwin now uses `top -l 2` interval samples).

Host: Apple Silicon, `/Applications/mac-stats.app` **v0.1.367**, LaunchAgent `-vv` (no `--cpu` for idle).

| Mode | Duration | Samples | CPU avg | CPU min | CPU max | RSS avg |
|------|----------|---------|---------|---------|---------|---------|
| **idle** (menu bar only) | 60s | 31 | **0.17%** | 0.0% | 0.7% | 98.9 MB |
| **window** (`--cpu`, warmed 35s) | 30s | 15 | **0.20%** | 0.0% | 0.7% | 141.6 MB |

Artifacts (gitignored):
- `performance_idle_20260810_193318.txt` / `.csv`
- `performance_window_20260810_193503.txt` / `.csv`

## Context for HN

Motivation for building mac-stats: iStat Menus was often **6–15% CPU** just to show CPU.
This run: idle **~0.17%**, window open **~0.20%** (same machine, same evening).

Caveats:
- Window run briefly had a second LaunchAgent instance; script later prefers `--cpu` PID in window mode. RSS ~142 MB vs ~99 MB idle supports WebView-present process.
- Activity Monitor “% CPU” and these `top` samples are not identical to iStat’s own accounting, but the order-of-magnitude gap vs 6–15% is the point.

## Script fixes shipped with this baseline

- macOS: sample CPU via `top`, not lifetime `ps %cpu`
- Thread count via `ps -M` on Darwin
- Prefer correct instance when idle vs `--cpu` coexist
