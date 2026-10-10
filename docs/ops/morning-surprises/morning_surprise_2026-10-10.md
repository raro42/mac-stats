# Morning surprise — 2026-10-10

Overnight Track B kept shipping Apple WebView idle cut (#14) opaque-type remaps.

## Shipped tonight (local 2026-10-09 evening → 2026-10-10)

| Version | What |
|---------|------|
| **v0.1.1826** | Apple `.logs-viewer-prefix` opaque color-mix (63% ink on `#ffffff`); no glass opacity on the Debug Log path/prefix line |
| **v0.1.1825** | Apple `.filter-chip-kb-hint` opaque color-mix (63% ink on `#f7f7fa`); no glass opacity on All · On · Off / lane chip move hints |
| **v0.1.1824** | Apple `.history-chart-caption` opaque color-mix (63% ink on `#ffffff`) |
| **v0.1.1823** | Apple `.ops-empty-tab-hint` opaque color-mix |
| **v0.1.1822** | Apple `.ops-updated-ago` opaque color-mix (resting · visible · hover) |
| **v0.1.1821** | Apple `.perplexity-setup-kb-hint` opaque color-mix |
| **v0.1.1820** | Apple `.logs-kb-hint` opaque color-mix |
| **v0.1.1819** | Apple `.logs-toolbar-kb-hint` opaque color-mix |
| **v0.1.1818** | Apple `.perplexity-search-kb-hint` opaque color-mix |
| **v0.1.1817** | Apple `.perplexity-kb-hint` opaque color-mix |
| **v0.1.1816** | Apple `.chat-kb-hint` opaque color-mix |

## Also this window
- Digester open stayed empty (MEMORY / MEMORY_APPEND slow turns already have instant/direct lanes).
- Design review not due (grace).
- Layout daily: Data Poster checked earlier; Dark left for another day (one layout experiment/day).
- Debug.log: quiet (no ERROR/WARN/panic clusters in the scan window).

## Fitness
Debug Log viewer prefix no longer uses glass `opacity` on Apple — less compositor work for #14, same readable muted type via `color-mix`.

## Next fuel
Remaining shared `opacity: 0.72` type without `.apple-shell` overrides — `.disk-cleanup-item-path`, `.disk-cleanup-scope-path` (hover already goes to opacity 1). Icon-strip images stay on opacity. `#chat-send-btn:disabled` (0.55) is a disabled state. `.monitor-item.is-checking` and `.is-pending` stay whole-row transient opacity.
