# Morning surprise — 2026-10-16

Overnight Track B kept shipping WebView idle-cut contrast (#14) while Mac screenshot capture stays deferred on the Linux rack.

## Shipped tonight

- **v0.1.2030** — Process Details click-to-copy PID (`.process-detail-pid`) drops glass `opacity` for an opaque `color-mix` wash (55% ink on light; 66% white on Data Poster / Dark).
- Earlier same night: v0.1.2029 chat-exec labels, v0.1.2028 process-detail labels, and a run of Disk Cleanup / Force Quit / history caption opaque chrome (see CHANGELOG).

## Tried / deferred

- Digester open: empty (no new Slowest fuel).
- Design review still due on `feature-cpu-metrics.png` (~39d) — capture needs macOS TCC.
- Layout daily due for data-poster / dark screens — missing on Linux.

## Why it matters

Secondary Process Details chrome stays readable without glass alpha, which is compositor work the idle cut is trying to avoid. PID copy is a high-frequency glance control.

## Next

- `.disk-cleanup-item-policy` / `.disk-cleanup-item-samples` (opacity 0.62), or Mac capture of stale CPU metrics screen when TCC allows.
