# Force-quit / “not responding” — 2026-09-15 ~12:17 CEST

## What happened

Around **12:17 CEST** the user force-quit mac-stats because the app was **not responding**. LaunchAgent restarted it at `2026-09-15T10:17:22Z`.

No panic / ERROR cluster in `~/.mac-stats/debug.log`. Background threads still logged until seconds before the kill (Having fun idle thought at `10:17:18Z`). That pattern fits a **UI hang** (WebView / Tauri invoke blocked), not a crash.

## Likely cause

Yesterday’s weekly Disk Cleanup scopes (`rebuild-dir`, `docker-prune`, `tmutil-thin`) were scanned on **every** `get_disk_cleanup_status` call — including shallow glance polls (`deep: false`).

Evidence after restart:

| Time (UTC) | Event |
|------------|--------|
| 10:17:22 | Process start |
| 10:17:22 | Temp scope cleaned (~0.4s) |
| 10:17:50 | `tmutil thinlocalsnapshots` finished (~**28 s**) |
| 10:17:51 | Disk cleanup (startup) complete |

`tmutil thin` alone can take tens of seconds. Walking multi-GB `target/debug` trees for size has no hard time budget before the fix. With the CPU window open (Ollama config traffic at ~10:15Z), Disk Cleanup refresh / glance could block the invoke and beachball the window.

## Fix (v0.1.1079)

1. Shallow status / glance: **do not** walk rebuild dirs or call docker/tmutil (stub: “not scanned — Refresh or Clean now”).
2. Dir size walks: **1.5 s** wall budget.
3. `tmutil` / `docker`: **8 s** timeout; SIGKILL on overrun.

## Sources

- `~/.mac-stats/debug.log` (restart + Disk cleanup lines)
- `~/.mac-stats/disk_cleanup.json` lastRun `startup` at 10:17:51Z
