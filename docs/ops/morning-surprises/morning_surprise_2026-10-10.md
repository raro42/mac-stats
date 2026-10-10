# Morning surprise — 2026-10-10

## Shipped overnight

- **v0.1.1823** — Apple Agent Ops empty-tab hint (`.ops-empty-tab-hint`) uses opaque `color-mix` (63% ink on `#ffffff`). No glass `opacity` on the empty-tab hint line (WebView idle / GitHub #14).
- **v0.1.1822** — Apple Agent Ops Updated stamp (`.ops-updated-ago`) uses opaque `color-mix` (63% / 69% / 81% ink on `#f7f7fa` for resting · visible · hover). No glass `opacity` on the Updated line.
- **v0.1.1821** — Apple Perplexity setup keyboard hint (`.perplexity-setup-kb-hint`) uses opaque `color-mix` (63% ink on `#ffffff`). No glass `opacity` on the key · Save key move hint.
- **v0.1.1820** — Apple Debug Log list keyboard hint (`.logs-kb-hint`) uses opaque `color-mix` (63% ink on `#f7f7fa`). No glass `opacity` on the log-line move hint.
- **v0.1.1819** — Apple Debug Log toolbar keyboard hint (`.logs-toolbar-kb-hint`) uses opaque `color-mix` (63% ink on `#f7f7fa`). No glass `opacity` on the Refresh · Open in editor · Auto-refresh move hint.
- **v0.1.1818** — Apple Perplexity search-box keyboard hint (`.perplexity-search-kb-hint`) uses opaque `color-mix` (63% ink on white). No glass `opacity` on the query · Search move hint.
- **v0.1.1817** — Apple Perplexity results keyboard hint (`.perplexity-kb-hint`) uses opaque `color-mix` (63% ink on white). No glass `opacity` on the results-list move hint.
- **v0.1.1816** — Apple AI Chat message-list keyboard hint (`.chat-kb-hint`) uses opaque `color-mix` (63% ink on white). No glass `opacity` on the All · You · Assistant · list move hint.

## Earlier same night (already on main before this tick)

- Through **v0.1.1815** — Apple kb-hint opaque chain continued (footer/header toolbars, Agent Ops hints, chat-empty, chat-composer, …).
- **v0.1.1795** — Data Poster / Dark AI Chat + Monitors white-slab contrast remaps.
- **v0.1.1794** — Layout daily review helper + Werner schedule.

## Tried / discarded

- Local footer-toolbar experiment raced earlier; discard logged.
- Local `perplexity-kb-hint` experiment raced with another tick that already shipped **v0.1.1817**; discard logged; later ticks continued with search-box and Debug Log hints.

## Digester

- Open candidates: none. Slow turns were MEMORY / MEMORY_APPEND note asks (direct lane).

## Next fuel

- Next shared `opacity: 0.72` type without `.apple-shell` overrides: `.history-chart-caption`, `.logs-viewer-prefix`, `.disk-cleanup-item-path`, `.disk-cleanup-scope-path`.
- Layout daily: dark theme still due tomorrow.
