# Morning surprise — 2026-08-31

Overnight Track B (20:00–06:00 local) — product ratchet for Ralf.

## Shipped

| Version | What |
|---------|------|
| **v0.1.748** | **External / Monitors Down/Slow attention glance** — **Monitors · N down · M slow** strip under the summary when any site is DOWN or Slow (≥2000 ms). Red wash for Down, amber for Slow-only. Click / Enter → Down (prefer) or Slow filter + first matching row. Design review / `feature-monitors` (Fail·Slow / Hot / Errors parity). |
| **v0.1.747** | Top Processes Hot attention glance — amber **Hot · N hot** strip; click → Hot filter. |
| **v0.1.746** | Agent Ops Runs Fail/Slow glance under Refresh. |
| **v0.1.745** | AI Chat Errors glance for failed turns. |
| **v0.1.744** | `/voice` · `/stt` Ready / Partial / Not set chip. |

## Tried / context

- Digester open empty most of the night (instant-lane noise filtered).
- Design review in grace (`due=false`); still polished stale surfaces in priority order (AI Chat → Agent Ops → Processes → Monitors).
- Screenshot recapture deferred (TCC / need on-screen CPU window).

## Next

- Recapture `feature-monitors` / `feature-processes` / `feature-agent-ops` / `feature-ai-chat` when the CPU window is visible.
- Rotate to `feature-cpu-metrics` or Disk Cleanup attention polish.
- Digester open / product `debug.log` errors when they appear.
