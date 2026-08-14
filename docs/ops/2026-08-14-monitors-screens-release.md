# 2026-08-14 — Monitors UX, screens, release notes

## Product

- Monitors (v0.1.394–397): downtime start (`down_since`), row meta, click detail panel, per-tick custom tip (not native `title`), ticks stay in card.
- Dark first-paint (v0.1.398): inline `theme-boot-paint` + `color-scheme` so dark themes do not flash white.
- Open section (v0.1.399): retry Tauri invoke; `block: start` scroll; AI Chat expands without toggle-close; `agent-ops.js` cache-bust on theme shells.

## Screenshot capture (installed app)

1. Stop LaunchAgent / `mac_stats` before capture so `MAC_STATS_OPEN_SECTION` reaches the process that owns the CPU window.
2. Theme localStorage for `/Applications/mac-stats.app` is under WebKit **`com.raro42.mac-stats`**, not the legacy `mac_stats` folder.
3. Window owner for `CGWindowList` / `screencapture -l` is **`mac-stats`** (hyphen).
4. Warm ≥30s. Frontend changes need **`cargo build` / `build-dmg` + install** — Tauri embeds `frontendDist`; editing `Resources/dist` alone does not change the WebView.

## Surfaces refreshed

- `docs/screens/feature-monitors.png`
- `docs/screens/feature-ai-chat.png`
- `docs/screens/dark-tui.png`
