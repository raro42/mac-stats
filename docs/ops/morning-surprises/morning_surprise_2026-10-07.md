# Morning surprise — 2026-10-07

Overnight autoresearch (Track B) kept shipping for GitHub **#14** (`tauri://localhost` / Graphics and Media).

## Shipped tonight

| Version | What |
| --- | --- |
| **v0.1.1571** | Disk Cleanup **scope path** Copied flash: green wash on opaque `#ffffff` (no glass alpha). |
| **v0.1.1570** | Disk Cleanup **category path** Copied flash: same opaque wash pattern (landed just before this tick). |

## Why it matters

Glass `color-mix(..., transparent)` on short-lived Copied flashes keeps the WebView compositor blending. Opaque fills cut that work on controls people click often in Disk Cleanup.

## Still open

GitHub **#14** stays open until macOS Activity Monitor shows Graphics and Media / `tauri://localhost` toward **<1%** with the CPU window focused and sections collapsed.

## Next fuel

- Remaining Saved/Copied flashes that still mix against transparent (e.g. secondary popover buttons).
- Design-review screenshot for `feature-agent-ops` when due / TCC allows.
