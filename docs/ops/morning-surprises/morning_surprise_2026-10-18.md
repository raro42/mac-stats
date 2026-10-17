# Morning surprise — 2026-10-18

Overnight Track B kept shipping WebView idle-cut (#14) opaque type washes on the CPU window shared sheet. Digester was empty; Mac screenshot refresh stayed deferred on the Linux rack.

## Shipped tonight (keeps)

| Version | What |
|---------|------|
| **v0.1.2069** | Disk Cleanup disabled scope row: row type uses an opaque ink mix (was glass `opacity` 0.55); dark shells remap onto `#0e0e14`. Off toggle stays clickable |
| **v0.1.2068** | Monitors removing row: row type uses an opaque ink mix (was glass `opacity` 0.55); dark shells remap onto `#0e0e14`. Clicks stay off. Ollama and MCP streams use `Response::chunk()` so a clean cargo check builds |
| **v0.1.2067** | Monitors pending row: row type uses an opaque ink mix (was glass `opacity` 0.78); dark shells remap onto `#0e0e14`. Dashed border stays |
| **v0.1.2066** | Monitors checking row: row type uses an opaque ink mix (was glass `opacity` 0.72); dark shells remap onto `#0e0e14`. Latency stays accent |
| **v0.1.2065** | AI Chat collapsed empty glance: empty glance uses an opaque ink mix (was glass `opacity` 0.85); dark shells remap onto `#0e0e14` |
| **v0.1.2064** | Agent Ops collapsed empty glance: empty glance uses an opaque ink mix (was glass `opacity` 0.85); dark shells remap onto `#0e0e14` |
| **v0.1.2063** | Monitors collapsed empty glance: empty glance uses an opaque ink mix (was glass `opacity` 0.85); dark shells remap onto `#0e0e14` |
| **v0.1.2062** | Monitors empty summary: empty summary line uses an opaque ink mix (was glass `opacity` 0.85); dark shells remap onto `#0e0e14` |
| **v0.1.2061** | Debug Log empty viewer: empty log line uses an opaque ink mix (was glass `opacity` 0.75); dark shells remap onto `#0e0e14`. Filter-miss copy stays its own colors |
| **v0.1.2060** | Disk Cleanup empty shell: load-error line uses an opaque ink mix (was glass `opacity` 0.75); dark shells remap onto `#0e0e14`. List-empty and filter-miss copy stay their own colors |

## Why it matters

Glass `opacity` on that shell forces the compositor to blend every paint. Mixing ink onto an opaque wash keeps the same look and cuts idle WebView work (#14).

## Still open

- `button.ops-badge:disabled` still uses glass `opacity` 0.55.
- Design-review screen `feature-cpu-metrics.png` is stale (~41d). Needs Mac TCC capture.
