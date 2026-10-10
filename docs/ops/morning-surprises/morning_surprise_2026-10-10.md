# Morning surprise — 2026-10-10

Overnight Track B kept shipping Apple WebView idle cut (#14) opaque-type remaps.

## Shipped tonight (local 2026-10-09 evening → 2026-10-10)

| Version | What |
|---------|------|
| **v0.1.1828** | Apple `.disk-cleanup-scope-path` opaque color-mix (63% ink on `#ffffff`); no glass opacity on Disk Cleanup scope paths |
| **v0.1.1827** | Apple `.disk-cleanup-item-path` opaque color-mix (63% ink on `#ffffff`); no glass opacity on Disk Cleanup category paths |
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
- Layout daily: Data Poster marked checked (remaps_missing=none); continued #14 opaque chain instead of a contrast remap.
- Debug.log: quiet (no ERROR/WARN/panic clusters in the scan window).

## Fitness
Disk Cleanup scope path type uses opaque `color-mix` on Apple (no glass opacity).
Disk Cleanup scope paths no longer use glass `opacity` on Apple — less compositor work for #14, same readable muted type via `color-mix`.

## Next fuel

`.force-quit-advanced > summary` and `.disk-cleanup-section .disk-cleanup-summary` (shared opacity 0.72). Icon-strip images stay on opacity. `#chat-send-btn:disabled` is a disabled state. `.monitor-item.is-checking` / `.is-pending` stay whole-row transient opacity. Hover `.ops-tab-digit` opacity stays interaction state.
