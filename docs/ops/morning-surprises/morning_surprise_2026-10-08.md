# Morning surprise — 2026-10-08

Overnight Track B kept shipping Apple theme WebView idle cuts for GitHub **#14** (opaque type / no glass alpha on always-visible chrome).

## Shipped tonight (keeps)

| Version | What |
|---------|------|
| **v0.1.1754** | `.apple-shell .monitor-checked-ago` opaque `color-mix` 36% `#0c0c10` on `#ffffff` (Down rows 43%; last-check age; was shared `opacity: 0.72` / Down `0.85` on `--muted`) |
| **v0.1.1753** | `.apple-shell .monitor-latency` opaque `color-mix` 45% `#0c0c10` on `#ffffff` (monitor timing line; was shared `opacity: 0.9` on `--muted`; Slow and Down keep status color) |
| **v0.1.1752** | `.apple-shell .disk-cleanup-empty-hint` · `.disk-cleanup-filter-miss-hint` opaque `color-mix` 45% `#0c0c10` on `#ffffff` (Disk Cleanup empty / filter-miss hints; was shared `opacity: 0.9` on `--muted`) |
| **v0.1.1751** | `.apple-shell .monitors-empty-hint` · `.monitors-filter-miss-hint` opaque `color-mix` 45% `#0c0c10` on `#ffffff` (Monitors empty / filter-miss hints; was shared `opacity: 0.9` on `--muted`) |
| **v0.1.1750** | `.apple-shell .logs-filter-miss-hint` opaque `color-mix` 45% `#0c0c10` on `#ffffff` (Debug Log filter-miss hint; was shared `opacity: 0.9` on `--muted`) |
| **v0.1.1749** | `.apple-shell .processes-filter-miss-hint` opaque `color-mix` 45% `#0c0c10` on `#ffffff` (Top Processes filter-miss hint; was shared `opacity: 0.9` on `--muted`) |
| **v0.1.1748** | `.apple-shell .perplexity-empty-hint` · `.perplexity-filter-miss-hint` opaque `color-mix` 45% `#0c0c10` on `#ffffff` (Perplexity empty / filter-miss hints; was shared `opacity: 0.9` on `--muted`) |
| **v0.1.1747** | `.apple-shell .chat-exec-label` opaque `color-mix` 33% `#0c0c10` on `#ffffff` (AI Chat Code executed / Result captions; was shared `opacity: 0.65` on `--muted`) |
| **v0.1.1746** | `.chat-message.thinking` opaque `color-mix` 75% `#0c0c10` on `#ffffff` (AI Chat waiting bubble; was `opacity: 0.85`) |
| **v0.1.1745** | `.apple-shell .response-time` opaque `color-mix` 36% `#0c0c10` on `#ffffff` (AI Chat latency line; was `opacity: 0.72` on `--muted`) |
| **v0.1.1744** | `.chat-status` opaque `color-mix` 45% `#0c0c10` on `#ffffff` (AI Chat status line; was `opacity: 0.9` on `--muted`) |
| **v0.1.1743** | `.chat-empty` opaque `color-mix` 45% `#0c0c10` on `#ffffff` (AI Chat empty shell; was `opacity: 0.9` on `--muted`) |
| **v0.1.1742** | `.perplexity-result-meta` opaque `color-mix` 38% `#0c0c10` on `#ffffff` (Perplexity source line; was `opacity: 0.75` on `--muted`) |
| **v0.1.1741** | `.logs-path-hint` opaque `color-mix` 43% `#0c0c10` on `#ffffff` (Debug Log path; was `opacity: 0.85` on `--muted`) |
| **v0.1.1740** | `.apple-footer` opaque `color-mix` 28% `#0c0c10` on `#f7f7fa` (hover 44%; version · GitHub line; was `opacity: 0.55` / `0.88` on `--muted`) |
| **v0.1.1739** | `.time-remaining` opaque `color-mix` 45% `#0c0c10` on `#ececf1` (time-left caption; was `opacity: 0.9` on `--muted`) |
| **v0.1.1738** | `.battery-status` opaque `color-mix` 45% `#0c0c10` on `#ececf1` (Bat charging / AC caption; was `opacity: 0.9` on `--muted`) |
| **v0.1.1737** | `.history-controls label` opaque `color-mix` 46% `#0c0c10` on `#ffffff` (History range caption; was `opacity: 0.9` on `--muted`) |
| **v0.1.1736** | `.section-title` opaque `color-mix` 75% `#010101` on `#ffffff` (Details · Top Processes headings; was `rgb(1,1,1,0.75)`) |
| **v0.1.1735** | `.detail-label` opaque `color-mix` 63% `#3c3c43` on `#ffffff` (Details Load · RAM · Up captions) |
| **v0.1.1734** | `#chip-info::before` opaque `color-mix` 32% `#0c0c10` on `#f7f7fa` ( glyph) |
| **v0.1.1733** | `.apple-subtitle` / `#chip-info` opaque `color-mix` 46% `#0c0c10` on `#f7f7fa` |
| **v0.1.1732** | `.history-chart-caption` opaque `color-mix` 36% `#0c0c10` on `#ffffff` |
| **v0.1.1731** | `.metric-subtext` opaque `color-mix` 40% `#0c0c10` on `#ffffff` |
| **v0.1.1730** | `.metric-label` opaque `color-mix` 62% `#0c0c10` on `#ffffff` |
| **v0.1.1729** | Settings `.theme-item` opaque type fallback (Apple `cpu.css` zero `rgba(` left) |

## Fuel notes

- Digester open stayed empty; design review still in grace (stale feature screens need TCC for screenshots).
- Debug.log quiet (no ERROR/WARN clusters in scan windows).
- Monitor latency shipped in v0.1.1753. Last-check age shipped in v0.1.1754 (`opacity: 1` plus 36% ink; Down rows 43%; `.apple-shell` beats the later shared sheet, and the Down selector beats `opacity: 0.85`). Next opacity candidate: `.monitor-detail-k` (shared `opacity: 0.78` on `--muted`). Icon-strip images stay on opacity (a CSS filter is heavier). `#chat-send-btn:disabled` (0.55) is a disabled state. `.monitor-item.is-checking` and `.is-pending` stay whole-row transient opacity.
- feature-agent-ops screenshot when TCC allows.

Latest keep: **v0.1.1754** @ `24ca4199`.
