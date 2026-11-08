# Morning surprise — 2026-11-08

Overnight Track B (mac-stats product ratchet). Window 20:00–06:00 local.

## Shipped

- **v0.1.2468** — Data Poster / Dark / Neon / Futuristic Top Processes bar CRT (`#process-list .process-bar`) off Apple `#ffffff` onto `#0b0b10` / CRT `#0a0a0a` / `#08120c` / `#0b0b10`. Design-review `feature-processes` polish-without-capture (Mac Screen Recording TCC still blocks `screencapture -l`).
- **v0.1.2467** — Data Poster / Dark / Neon / Futuristic Monitors `.monitor-detail-log` CRT on the same opaque shells.
- **v0.1.2466** — Data Poster / Dark / Neon / Futuristic Perplexity result rows + weather card CRT.
- **v0.1.2465** — Disk Cleanup last-run strip + Scan primary CRT.
- **v0.1.2464** — Disk Cleanup meta-card CRT remaps.
- Earlier tonight: logs-line, chat-message, metric Hot/Ok/Fair, process-empty, Monitors pending/removing, and more CRT catch-up (see `results.tsv` / CHANGELOG).

## Tried / discarded

- Duplicate attempt at Monitors `.monitor-detail-log` CRT after a sibling tick already shipped it in v0.1.2467 → hard-reset to origin; no second ship.
- Earlier: duplicate Disk Cleanup `.disk-cleanup-primary` after v0.1.2465 → rebase skipped.

## Why it matters

Top Processes still painted the usage bar track on Apple white after row washes went opaque. Dark / Neon / Futuristic / Data Poster process lists no longer flash that white track slab.

## Still open

- Digester Slowest empty (filtered instant noise). Fuel = design-review + standing P2 #14.
- Layout-daily due (theme screens missing) deferred; CRT remaps continue standing #14.
- Mac Screen Recording TCC still blocks fresh `feature-*.png` captures; polish-without-capture grace continues.
- Next: other Apple `#ffffff` washes still missing Dark remaps (e.g. `.section-header-collapsible:hover`) if any, or Mac capture of stale `feature-processes.png` / `feature-monitors.png` / `feature-agent-ops.png` when TCC allows.

Generated: 2026-11-08T01:08:00
