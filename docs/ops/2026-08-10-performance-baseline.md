# Performance baseline — 2026-08-10

Measured with `./scripts/measure_performance.sh` (Darwin: `top -l 2` interval CPU samples).

Host: Apple Silicon · same evening · mac-stats **v0.1.367** (LaunchAgent `-vv`, no `--cpu` when idle) · [exelban Stats](https://github.com/exelban/stats) from `/Applications/Stats.app` (default modules, menu bar only, warmed ~30s).

## Results

| App | Mode | Duration | Samples | CPU avg | CPU min | CPU max | RSS avg |
|-----|------|----------|---------|---------|---------|---------|---------|
| **mac-stats** | idle (menu bar) | 60s | 31 | **0.17%** | 0.0% | 0.7% | 98.9 MB |
| **mac-stats** | window (`--cpu`, warmed 35s) | 30s | 15 | **0.20%** | 0.0% | 0.7% | 141.6 MB |
| **Stats** (exelban) | idle (menu bar) | 60s | 31 | **3.40%** | 2.4% | 6.9% | 96.8 MB |

Same machine, same sampler. Stats was started for this run, then quit afterward.

## Artifacts (gitignored)

- `performance_idle_20260810_193318.*` — mac-stats idle (legacy filename before target prefix)
- `performance_window_20260810_193503.*` — mac-stats window
- `performance_stats_idle_20260810_194035.*` — Stats idle

Re-run:

```bash
./scripts/measure_performance.sh 60 1 idle mac-stats
open -a Stats && sleep 30
./scripts/measure_performance.sh 60 1 idle stats
```

## Context for HN

- Motivation: **iStat Menus** often **6–15% CPU** (anecdotal; not re-measured tonight — not installed).
- Peer check tonight: **Stats ~3.4%** idle vs **mac-stats ~0.17%** idle.
- Stats is excellent and deeper; this is about overhead for a lean menu-bar glance, not “Stats is bad.”

## Caveats

- Window mac-stats run briefly overlapped a LaunchAgent instance; measured PID was the `--cpu` process (RSS ~142 MB).
- Module sets differ (Stats may enable more sensors by default). Fairness = same host + same `top` method, not identical feature surface.
- iStat 6–15% remains memory unless re-measured.

## Script notes

- Target arg: `mac-stats` (default) or `stats`
- macOS: `top` interval samples (not lifetime `ps %cpu`)
- Thread count via `ps -M` on Darwin
