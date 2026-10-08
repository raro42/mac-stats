# Morning surprise — 2026-10-08

Overnight autoresearch (Track B) kept real ships. Digester open was empty; design review was in grace; fuel was standing P2 / GitHub #14 opaque glass cuts.

## Shipped

- **v0.1.1730** — Apple ring metric labels (`.metric-label`) mix type color against an opaque card fill. No glass `opacity` on the always-visible CPU · GPU · Freq · Temp captions.
- **v0.1.1729** — Apple Settings theme-list type (`.theme-item`) mixes against an opaque panel fill. No glass alpha on the theme button label fallback (`var(--text, #0c0c10)`). Clears the last `rgba(` in Apple `cpu.css`.
- **v0.1.1728** — Apple AI Chat markdown link type (`.chat-message .markdown a`) mixes against an opaque panel fill. No glass alpha on markdown link color.
- **v0.1.1727** — Apple Force Quit control type (`.force-quit-btn` resting) mixes against an opaque panel fill. No glass alpha on Force Quit label type.
- **v0.1.1726** — Apple Changelog body type (`.changelog-error` · `.changelog-h2` · `.changelog-version` · `.changelog-h3` · `.changelog-paragraph` · `.changelog-item` · bullet · `.changelog-code` · `strong`) mixes against an opaque panel fill. No glass alpha on Changelog copy or the error accent.
- **v0.1.1725** — Apple icon-line status washes (`.icon-line-item.status-good` · `.status-warning` · `.status-bad` resting · hover) mix type color against an opaque chip fill. No glass alpha on Ready / Slow / Down strip status type.
- **v0.1.1724** — Apple Details / Top Processes body type (`.details-grid` · `.process-table`) mixes against an opaque panel fill.
- **v0.1.1723** — Apple battery strip glyph (`.battery-icon` resting · charging) mixes type color against an opaque strip fill.
- **v0.1.1722** — Apple window title (`.apple-title h1`) mixes type color against an opaque shell fill.
- **v0.1.1721** — Apple section icon strip glyphs (`.icon-btn` resting · hover) mix type color against an opaque chip fill.
- **v0.1.1720** — Apple icon-line strip glyphs (`.icon-line-item` resting · hover) mix type color against an opaque chip fill.
- **v0.1.1719** — Apple primary / muted type tokens (`--text` · `--muted`) and leftover panel tokens mix against an opaque shell fill.
- **v0.1.1718** — Apple Settings / Monitors / AI Chat modal dimmers (`--modal-backdrop`) mix against an opaque fill.
- **v0.1.1717** — Apple ring gauge tracks (`.ring-track` / `--ring-track`) mix against an opaque fill.
- **v0.1.1716** — Apple AI Chat markdown table cells and horizontal rules mix hairline borders against an opaque fill.
- **v0.1.1715** — Apple Monitors settings Add form and row history hairlines opaque.
- **v0.1.1714** — Apple Monitors / AI Chat settings popover headers opaque hairline borders.
- **v0.1.1713** — Apple AI Chat message list (`.chat-messages`) hairline border opaque.
- **v0.1.1712** — Apple icon-strip dividers and section hairlines opaque.
- **v0.1.1711** — Apple outer window shell opaque hairline border.
- **v0.1.1710** — Apple History sparkline shells opaque hairline borders.
- **v0.1.1709** — Apple ring metric cards opaque hairline borders.
- **v0.1.1708** — Apple Details / Top Processes shells opaque hairline borders.
- **v0.1.1707** — Apple Changelog scrollbar track / thumbs opaque.
- **v0.1.1706** — Apple Details / Top Processes scrollbar thumbs opaque.
- **v0.1.1705** — Apple battery / power strip opaque hairline borders.
- **v0.1.1704** — Apple History controls opaque hairline borders.
- **v0.1.1703** — Apple AI Chat model-text hover + connection focus opaque.
- **v0.1.1702** — Apple settings popover Close hover opaque.
- **v0.1.1701** — Apple overflow menu-btn hover opaque.
- **v0.1.1700** — Apple section collapse-btn opaque.
- **v0.1.1699** — Apple History sparkline tooltip opaque.

## Tried / context

- Digester: no open candidates.
- Design review: due=false (feature-agent-ops still recommended when screenshot TCC allows).
- Debug.log: quiet in the 180m window.
- Sibling harnesses: OpenClaw / Hermes paths missing on this host.
- 02:41 tick: shipped ring `.metric-label` opacity → opaque color-mix as v0.1.1730 (first always-visible opacity cut after rgba cleared).
- 02:17 tick: shipped Settings theme-item type fallback as v0.1.1729 (last `rgba(` in Apple `cpu.css`).
- 01:55 tick: shipped AI Chat markdown link type as v0.1.1728.
- 01:35 tick: shipped Force Quit type as v0.1.1727.
- 01:10 tick: shipped Changelog body type as v0.1.1726.
- 00:45 tick: shipped icon-line status type as v0.1.1725; logged missing keep for v0.1.1724.

## Next

- More Apple always-visible `opacity` on chrome (`.metric-subtext`, `.history-chart-caption`, `#chip-info::before`, footer). Then other themes / shared CSS for #14 glass leftovers, or screenshot feature-agent-ops when due and TCC allows.
