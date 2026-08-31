# Morning surprise — 2026-08-31

Overnight Track B (20:00–06:00 local) — product ratchet for Ralf.

## Shipped

| Version | What |
|---------|------|
| **v0.1.754** | **Perplexity Top/error attention glance** — **Search · error · …** / **Search · N results · Top** strip above All · Top · Snippet when Perplexity is open and the last search failed or returned more than Top-N hits (Debug Log / Monitors Down·Slow parity). Red wash for error, amber for Top. Click / Enter focuses the query (error) or opens Top + first result. Collapsed keep-header last-search glance stays. |
| **v0.1.753** | **Details Load/RAM Hot attention glance** — **Hot · Load · RAM** amber strip above the Details grid when Details is open and Load ≥4 or RAM ≥85% (`/details hot` / collapsed glance parity). Click / Enter focuses the first hot row (Load flash, or RAM). Hidden while collapsed. Design review / `feature-cpu-metrics`. |
| **v0.1.752** | **Debug Log Error/Warn attention glance** — **Logs · N errors · M warns** strip above the toolbar when Debug Log is open and the tail has ERROR/WARN lines. Red wash for Error, amber for Warn-only. Click / Enter opens Error (prefer) or Warn filter and focuses the first matching line. Collapsed keep-header glance stays as Quiet / error-warn summary. |
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
- Design review in grace (`due=false`); still polished stale surfaces in priority order (AI Chat → Agent Ops → Processes → Monitors → Disk Cleanup → CPU rings → power strip → Debug Log → Details Hot → Perplexity Top/error).
- Screenshot recapture deferred (TCC / need on-screen CPU window).

## Next

- Recapture stale feature screens when the CPU window is visible.
- Next polish: Settings Help / empty-state, or screenshot recapture when due.
- Digester open / product `debug.log` errors when they appear.
