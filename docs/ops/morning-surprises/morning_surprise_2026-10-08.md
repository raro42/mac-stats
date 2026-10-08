# Morning surprise — 2026-10-08

Overnight Track B kept shipping Apple theme WebView idle cuts for GitHub **#14** (opaque type / no glass alpha on always-visible chrome).

## Shipped tonight (keeps)

| Version | What |
|---------|------|
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
- Apple `cpu.css` has no remaining `rgb(`/`rgba(` type. History range label shipped in v0.1.1737. Next always-visible `opacity:` candidates: `.battery-status`, `.time-remaining`.
- feature-agent-ops screenshot when TCC allows.

Latest keep: **v0.1.1737** (History range label).
