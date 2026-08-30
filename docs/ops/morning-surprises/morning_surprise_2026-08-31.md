# Morning surprise — 2026-08-31

Overnight Track B (20:00–06:00 local) — product ratchet for Ralf.

## Shipped

| Version | What |
|---------|------|
| **v0.1.751** | **Power strip Hot attention glance** — **Hot · Bat · LPM · Heat · Up · RAM · SSD** strip under the slim battery/power row when any `/strip hot` cue fires (Bat≤20% not charging · LPM On · Heat Fair+ · Up≥7d · RAM/SSD≥85%). Click / Enter opens the first cue. Rings keep their own Hot glance. Design review / `feature-cpu-metrics`. |
| **v0.1.750** | **CPU rings Hot attention glance** — **Hot · CPU · Temp** amber strip under the gauges when any ring hits menu-bar amber (CPU ≥50% · GPU ≥15% · Freq ≥3.5 GHz · Temp ≥70°C). Click / Enter focuses the first hot ring with a brief flash. Design review / `feature-cpu-metrics` (Top Processes Hot parity; All · Hot chips stay removed). |
| **v0.1.749** | Disk Cleanup Reclaim/Due attention glance — amber/green strip; click → Big/Reclaim or Clean now. |
| **v0.1.748** | External / Monitors Down/Slow attention glance — red/amber strip; click → Down or Slow filter. |
| **v0.1.747** | Top Processes Hot attention glance — amber **Hot · N hot** strip; click → Hot filter. |
| **v0.1.746** | Agent Ops Runs Fail/Slow glance under Refresh. |
| **v0.1.745** | AI Chat Errors glance for failed turns. |
| **v0.1.744** | `/voice` · `/stt` Ready / Partial / Not set chip. |

## Tried / context

- Digester open empty most of the night (instant-lane noise filtered).
- Design review in grace (`due=false`); still polished stale surfaces in priority order (AI Chat → Agent Ops → Processes → Monitors → Disk Cleanup → CPU rings → power strip).
- Screenshot recapture deferred (TCC / need on-screen CPU window).

## Next

- Recapture stale feature screens when the CPU window is visible.
- Next polish: Debug Log Error/Warn attention (expanded), Perplexity last-search attention, or Details Load/RAM attention.
- Digester open / product `debug.log` errors when they appear.
