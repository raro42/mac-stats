# Morning surprise — 2026-10-07

Overnight autoresearch (Track B) kept shipping for GitHub **#14** (`tauri://localhost` / Graphics and Media).

## Shipped tonight

| Version | What |
| --- | --- |
| **v0.1.1578** | Perplexity **result row** Copied flash: green wash on opaque `#ffffff` (no glass alpha / outline). |
| **v0.1.1577** | Disk Cleanup **row** Copied flash: green wash on opaque `#ffffff` (category + scope rows). |
| **v0.1.1576** | Monitors **row** Copied flash: green wash on opaque `#ffffff`. |
| **v0.1.1575** | Top Processes **row** Copied flash: green wash on opaque `#ffffff`. |
| **v0.1.1574** | Details **value** Copied flash: green wash on opaque `#ffffff`. |
| **v0.1.1573** | Settings **product-toggle** Saved flash: same opaque wash on the label. |
| **v0.1.1572** | Shared Save / secondary-button Saved flash: opaque wash. |
| **v0.1.1571** | Disk Cleanup **scope path** Copied flash: opaque wash. |
| **v0.1.1570** | Disk Cleanup **category path** Copied flash: opaque wash. |

## Why it matters

Glass `color-mix(..., transparent)` on short-lived Copied/Saved flashes keeps the WebView compositor blending. Opaque fills cut that work on controls people click often (Perplexity results, process rows, Details metrics, Settings toggles, Disk Cleanup paths).

## Still open

GitHub **#14** stays open until macOS Activity Monitor shows Graphics and Media / `tauri://localhost` toward **<1%** with the CPU window focused and sections collapsed.

## Next fuel

- Remaining `is-just-copied` flashes that still mix against transparent (Debug Log lines, chat messages, glance Copied flashes).
- Design-review screenshot for `feature-agent-ops` when due / TCC allows.
