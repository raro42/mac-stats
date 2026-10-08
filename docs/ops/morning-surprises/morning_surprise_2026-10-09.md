# Morning surprise — 2026-10-09

Overnight Track B opened on design-review fuel (stale `feature-cpu-metrics`) and kept shipping Apple theme WebView idle cuts for GitHub **#14**.

## Shipped tonight (keeps)

| Version | What |
|---------|------|
| **v0.1.1763** | `.power-label` · `.lpm-label` opaque `color-mix` 45% `#0c0c10` on `#ececf1` (Power · LPM captions on the strip; was shell `--muted` mixed on `#f7f7fa`) |

Earlier today (before this overnight window): **v0.1.1762** restored live CPU-window metrics after the #14 idle ratchet stuck gauges on "None yet" (GitHub **#15**).

## Fuel notes

- Digester open: design-review stale `feature-cpu-metrics.png` (~31.7d). Mac host `192.168.2.20` unreachable from the Linux rack, so screenshot deferred; polish still shipped on the power strip.
- Debug.log quiet (no ERROR/WARN clusters in 180m window).
- Next: `.mastodon-settings-toolbar-kb-hint` glass type (shared `opacity: 0.72`); capture `feature-cpu-metrics` when the Mac is back; icon-strip images stay on opacity.

Latest keep: **v0.1.1763** @ `73a9db73`.
