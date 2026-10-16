# Morning surprise — 2026-10-17

Overnight Track B kept shipping WebView idle-cut (#14) opaque type washes on the CPU window shared sheet. Digester was empty; Mac screenshot refresh stayed deferred on the Linux rack.

## Shipped tonight (keeps)

| Version | What |
|---------|------|
| **v0.1.2036** | Top Processes column header (Pin · name · CPU · GPU) — opaque `color-mix` (was glass `opacity` 0.55) |

## Why it matters

Glass `opacity` on the process list header forces the compositor to blend every paint. Mixing ink onto an opaque wash keeps the same look and cuts idle WebView work (#14).

## Still open

- Shared `.perplexity-result-meta` still uses glass `opacity` 0.55.
- Design-review screen `feature-cpu-metrics.png` is stale (~40d) — needs Mac TCC capture.
- Layout daily review themes (Data Poster / dark / …) still marked due (screens missing on Linux).

## Ratchet

Latest keep: `80c2615` — v0.1.2036 Top Processes column header opaque.
