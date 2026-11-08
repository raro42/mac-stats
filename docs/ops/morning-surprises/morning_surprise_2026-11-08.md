# Morning surprise — 2026-11-08

## Shipped overnight (Track B)

### v0.1.2470 — section-header hover/focus CRT (≈01:32 CET)
- **Fuel:** digester design-review `feature-cpu-metrics` (62d stale) + standing P2 #14 after v0.1.2469.
- **Change:** Data Poster / Dark / Neon / Futuristic `.section-header-collapsible:hover` · `:focus-visible` washes off Apple `#ffffff` → `#0b0b10` / CRT `#0a0a0a` / `#08120c` / `#0b0b10` (`src/agent-ops.css`).
- **Capture:** deferred (Screen Recording TCC / menu-bar only); polish-without-capture grace on `feature-cpu-metrics`.
- **Ratchet:** keep @ `8cd5c9fb`.

### Earlier keeps this night
- **v0.1.2469** — Monitors row status CRT (selected / Down / Slow / focus / Copied) on Dark / Neon / Futuristic.
- **v0.1.2468** — Top Processes `#process-list .process-bar` CRT on Data Poster / Dark / Neon / Futuristic.
- **v0.1.2467** — Monitors `.monitor-detail-log` CRT.
- **v0.1.2466** — Perplexity result-item / weather-card CRT.
- **v0.1.2465** — Disk Cleanup last-run / Scan primary CRT.
- **v0.1.2464** — Disk Cleanup meta-card CRT.

## Still open
- Other Apple `#ffffff` washes (e.g. `.logs-viewer` · `.monitor-detail` · Disk Cleanup selected/reclaim · `.perplexity-empty`) if any.
- Mac window-only recapture of stale `feature-*.png` when Screen Recording TCC allows `screencapture -l`.
- Layout-daily screens still missing for data-poster / dark / neon / futuristic / architect.

## Debug skim
- `scan_debug_log_errors.py` (180m): single-instance lock WARN only — no product ERROR/panic cluster.

