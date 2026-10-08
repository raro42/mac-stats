# Morning surprise — 2026-10-09

Overnight Track B opened on design-review fuel (stale `feature-cpu-metrics`) then continued Apple theme WebView idle cuts for GitHub **#14**.

## Shipped tonight (keeps)

| Version | What |
|---------|------|
| **v0.1.1763** | `.power-label` · `.lpm-label` opaque `color-mix` 45% `#0c0c10` on `#ececf1` (Power · LPM captions on the strip; was shell `--muted` mixed on `#f7f7fa`) |
| **v0.1.1764** | `.mastodon-settings-toolbar-kb-hint` opaque `color-mix` 63% `#0c0c10` on `#ffffff` (URL · token · Save · Clear hint; no glass `opacity: 0.72`) |
| **v0.1.1765** | `.mcp-settings-toolbar-kb-hint` opaque `color-mix` 63% `#0c0c10` on `#ffffff` (URL · stdio · Save · Clear hint; no glass `opacity: 0.72`) |
| **v0.1.1766** | `.browser-settings-toolbar-kb-hint` opaque `color-mix` 63% `#0c0c10` on `#ffffff` (path · port · Save · Clear hint; no glass `opacity: 0.72`) |
| **v0.1.1767** | `.cursor-agent-settings-toolbar-kb-hint` opaque `color-mix` 63% `#0c0c10` on `#ffffff` (workspace · executable · Save · Clear hint; no glass `opacity: 0.72`) |
| **v0.1.1768** | `.telegram-settings-toolbar-kb-hint` opaque `color-mix` 63% `#0c0c10` on `#ffffff` (token · chat id · Save · Clear hint; no glass `opacity: 0.72`) |
| **v0.1.1771** | `.slack-settings-toolbar-kb-hint` opaque `color-mix` 63% `#0c0c10` on `#ffffff` (webhook · Save · Clear hint; no glass `opacity: 0.72`) |
| **v0.1.1773** | `.theme-list-kb-hint` opaque `color-mix` 63% `#0c0c10` on `#ffffff` (Appearance theme-list hint; no glass `opacity: 0.72`) |
| **v0.1.1775** | `.appearance-setting-kb-hint` opaque `color-mix` 63% `#0c0c10` on `#ffffff` (Appearance section hint; no glass `opacity: 0.72`) |
| **v0.1.1776** | `.product-setting-kb-hint` opaque `color-mix` 63% `#0c0c10` on `#ffffff` (Product section hint; no glass `opacity: 0.72`) |
| **v0.1.1777** | `.credentials-section-kb-hint` opaque `color-mix` 63% `#0c0c10` on `#ffffff` (Credentials section hint; no glass `opacity: 0.72`) |

Earlier today (before this overnight window): **v0.1.1762** restored live CPU-window metrics after the #14 idle ratchet stuck gauges on "None yet" (GitHub **#15**). Same window also landed **v0.1.1769** (vim keys no longer steal from text fields), **v0.1.1770** (seed sparklines from history on open), **v0.1.1772** (menu bar / glass UI percentages no longer stuck at 0%), and **v0.1.1774** (footer version paints on open instead of "None yet").

## Fuel notes

- Digester open: empty; design review due=false (grace). Standing P2 / #14 continued.
- Debug.log quiet (no ERROR/WARN clusters in 180m window).
- Mac host `192.168.2.20` still unreachable from the Linux rack; `feature-cpu-metrics` screenshot deferred.
- Next: `.settings-header-kb-hint` glass type (shared `opacity: 0.72`); capture `feature-cpu-metrics` when the Mac is back.

Latest keep: **v0.1.1777** @ `115345a1`.
