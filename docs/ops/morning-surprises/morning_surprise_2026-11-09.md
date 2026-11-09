# Morning surprise — 2026-11-09

Overnight Track B kept shipping **#14 opaque chrome** (WebView glass → `color-mix` on paper + CRT remaps). Digester open stayed empty; design-review stayed in grace (TCC blocks Mac capture).

## Shipped tonight (highlights)

| Version | Keep | What |
|---------|------|------|
| **v0.1.2529** | Settings / Ollama header focus | `.settings-header h2[tabindex]:focus` · Ollama popover `h3[tabindex]:focus` off `--focus-ring` rgba glass |
| **v0.1.2528** | Changelog focus | `.changelog-h2` / loading / error tabindex focus opaque |
| **v0.1.2527** | First-launch tip + update banner | tip/banner border · underline · glyph · dismiss · code chip |
| **v0.1.2526** | History sparkline focus | chart container focus-visible |
| **v0.1.2525** | Metric-strip flash | RAM/CPU/GPU/Temp/Freq/Disk/Uptime highlights |
| **v0.1.2524–2517** | Disk Cleanup / Agent Ops / Perplexity / Monitors / Processes | remaining interactive glass batches |

## Tried / deferred

- Intentional **scrollbar** hover thumbs still use `color-mix(…, transparent)` — left alone (near-invisible chrome).
- Mac `screencapture -l` for stale `feature-*.png` — Screen Recording TCC still blocks.

## Next fuel

1. Scrollbar opaque only if a clear Graphics-and-Media win shows up.
2. Mac capture when TCC allows (`feature-cpu-metrics` / `feature-agent-ops` / `feature-disk-cleanup`).
3. Sibling ports (session resume / Discord reconnect) once glass #14 idle is quiet.

_Generated ~03:55 CET by overnight autoresearch tick._
