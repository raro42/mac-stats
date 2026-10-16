# Morning surprise — 2026-10-17

Overnight Track B kept shipping WebView idle-cut (#14) opaque type washes on the CPU window shared sheet. Digester was empty; Mac screenshot refresh stayed deferred on the Linux rack.

## Shipped tonight (keeps)

| Version | What |
|---------|------|
| **v0.1.2042** | Top Processes Hot/Pinned empty hint: opaque muted mix (was glass `opacity` 0.9) |
| **v0.1.2041** | Rings Hot-filter empty hint: opaque `color-mix` (was glass `opacity` 0.78) |
| **v0.1.2040** | Perplexity key status (API key line): opaque muted mix (was glass `opacity` 0.7) |
| **v0.1.2039** | Perplexity result snippet (body under a result): opaque `color-mix` (was glass `opacity` 0.7) |
| **v0.1.2038** | Perplexity result list (bullets under a snippet): opaque `color-mix` (was glass `opacity` 0.78) |
| **v0.1.2037** | Perplexity result meta (source · time under a result): opaque `color-mix` (was glass `opacity` 0.55) |
| **v0.1.2036** | Top Processes column header (Pin · name · CPU · GPU): opaque `color-mix` (was glass `opacity` 0.55) |

## Why it matters

Glass `opacity` on those lines forces the compositor to blend every paint. Mixing ink onto an opaque wash keeps the same look and cuts idle WebView work (#14).

## Still open

- Shared `.perplexity-filter-miss-hint` still uses glass `opacity` 0.9 on `--muted`.
- Design-review screen `feature-cpu-metrics.png` is stale (~40d). Needs Mac TCC capture.
- Layout daily review themes (Data Poster / dark / …) still marked due (screens missing on Linux).

## Ratchet

Latest keep: `e68a1fec` — v0.1.2042 Top Processes filter-miss hint opaque.
