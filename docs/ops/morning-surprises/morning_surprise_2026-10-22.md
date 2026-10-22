# Morning surprise — 2026-10-22

Overnight Track B kept shipping Data Poster opaque washes (standing P2 #14 / GitHub #14). Digester stayed empty; design review still due for stale feature screens (Mac capture blocked on this Linux host).

## Shipped tonight (keeps)

| Version | What |
|---------|------|
| **v0.1.2214** | Debug Log toolbar / viewer (`.logs-toolbar button` · `.logs-viewer` · path/autorefresh) opaque `#0b0b10` mix (no glass `var(--hairline)` / rgba) |
| **v0.1.2213** | LPM strip track (`.lpm-info .lpm-toggle`) mixes against `#0b0b10` (dropped early `#0e0e14`) |
| **v0.1.2212** | Collapse / Monitors menu opaque (no glass hairline/panel) |
| **v0.1.2211** | Theme-footer + battery-power-strip hairlines opaque |
| **v0.1.2210** | AI Chat empty shell opaque |
| **v0.1.2209** | AI Chat model/turn/answer/errors/collapsed glances opaque |
| **v0.1.2208** | AI Chat error bubble + offline glance opaque |

## Tried / deferred

- Mac `screencapture -l` for stale `feature-ai-chat` / `feature-monitors` / `feature-cpu-metrics` — unavailable on Linux; TCC still blocks when on Mac.
- Sibling OpenClaw/Hermes ports — git trees missing on this host.

## Next fuel

Data Poster monitor-item / history hairline glass (`var(--hairline)`), or Mac window-only capture when TCC allows.
