# Morning surprise — 2026-10-17

Overnight Track B kept shipping WebView idle-cut (#14) opaque type washes on the CPU window shared sheet. Digester was empty; Mac screenshot refresh stayed deferred on the Linux rack.

## Shipped tonight (keeps)

| Version | What |
|---------|------|
| **v0.1.2056** | AI Chat filter-miss copy (You · Assistant · Errors empty hint): opaque ink mix (was glass `opacity` 0.88) |
| **v0.1.2055** | Agent Ops health values (Version · Discord · schedule bodies): opaque ink mix (was glass `opacity` 0.95) |
| **v0.1.2054** | Monitors backoff hint (DOWN recheck countdown): opaque muted mix (was glass `opacity` 0.78) |
| **v0.1.2053** | Monitors detail keys: opaque muted mix (was glass `opacity` 0.78) |
| **v0.1.2052** | Agent Ops clickable insight lines: opaque ink mix (was glass `opacity` 0.9; hover/selected full inherit) |
| **v0.1.2051** | Agent Ops insight lines (not clickable): opaque ink mix (was glass `opacity` 0.8) |
| **v0.1.2050** | Monitors latency line: opaque muted mix (was glass `opacity` 0.9) |
| **v0.1.2049** | Perplexity empty search hint: opaque muted mix (was glass `opacity` 0.9) |
| **v0.1.2048** | Monitors empty list hint: opaque muted mix (was glass `opacity` 0.9) |
| **v0.1.2047** | Disk Cleanup empty category/scope hint: opaque muted mix (was glass `opacity` 0.9) |
| **v0.1.2046** | Disk Cleanup Reclaim/Big/Clean empty hint: opaque muted mix (was glass `opacity` 0.9) |
| **v0.1.2045** | Monitors Up/Down/Slow empty hint: opaque muted mix (was glass `opacity` 0.9) |
| **v0.1.2044** | Debug Log Error/Warn empty hint: opaque muted mix (was glass `opacity` 0.9) |
| **v0.1.2043** | Perplexity Top/Snippet empty hint: opaque muted mix (was glass `opacity` 0.9) |
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

- Shared `.perplexity-filter-chip` still uses glass `opacity` 0.88.
- Design-review screen `feature-cpu-metrics.png` is stale (~40d). Needs Mac TCC capture.
- Layout daily review themes (Data Poster / dark / …) still marked due (screens missing on Linux).

## Ratchet

Latest keep: `aeac8fe9`. v0.1.2056 AI Chat filter-miss copy opaque.
