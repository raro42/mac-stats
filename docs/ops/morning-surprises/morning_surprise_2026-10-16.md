# Morning surprise — 2026-10-16

Overnight Track B kept shipping WebView idle-cut (#14) opaque type washes on the CPU window shared sheet. Digester was empty; Mac screenshot refresh stayed deferred on the Linux rack.

## Shipped tonight (keeps)

| Version | What |
|---------|------|
| **v0.1.2033** | Disk Cleanup category **size** (bytes · file count) — opaque `color-mix` (was glass `opacity` 0.85) |
| **v0.1.2032** | Disk Cleanup scope **kind** (builtin/custom) — opaque `color-mix` (was glass `opacity` 0.55) |
| **v0.1.2031** | Disk Cleanup category **policy · samples** — opaque `color-mix` (was glass `opacity` 0.62) |
| **v0.1.2030** | Process Details **PID** click-to-copy — opaque wash |
| **v0.1.2029** | AI Chat exec card labels (Code executed · Result · …) |
| **v0.1.2028** | Process Details metric labels |
| **v0.1.2027** | Disk Cleanup meta card titles |
| **v0.1.2026** | Disk Cleanup soft-delete hint |
| **v0.1.2025** | Force Quit hint |
| **v0.1.2024** | Disk Cleanup category/scope paths |

## Why it matters

Glass `opacity` on secondary labels forces the compositor to blend every paint. Mixing ink onto an opaque wash keeps the same look and cuts idle WebView work (#14).

## Still open

- Shared `.ops-health-label` still uses glass `opacity` 0.55; Perplexity setup lead/note still 0.85.
- Design-review screen `feature-cpu-metrics.png` is stale (~39d) — needs Mac TCC capture.
- Layout daily review themes (Data Poster / dark / …) still marked due (screens missing on Linux).

## Ratchet

Latest keep: `6edaa0bd` — v0.1.2033 disk-cleanup category size line opaque.
