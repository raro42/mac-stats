# Morning surprise — 2026-08-31

Overnight Track B (20:00–06:00 local) — product ratchet for Ralf.

## Shipped

| Version | What |
|---------|------|
| **v0.1.749** | **Disk Cleanup Reclaim/Due attention glance** — **Disk · N big · M reclaim · Due** strip above category filters when reclaimable or due. Amber for Big/Reclaim, green for Due-only. Click / Enter → Big (prefer) or Reclaim filter + first matching row, or Clean now when only Due. Design review / `feature-disk-cleanup` (Monitors Down/Slow / Hot parity). |
| **v0.1.748** | External / Monitors Down/Slow attention glance — red/amber strip; click → Down or Slow filter. |
| **v0.1.747** | Top Processes Hot attention glance — amber **Hot · N hot** strip; click → Hot filter. |
| **v0.1.746** | Agent Ops Runs Fail/Slow glance under Refresh. |
| **v0.1.745** | AI Chat Errors glance for failed turns. |
| **v0.1.744** | `/voice` · `/stt` Ready / Partial / Not set chip. |

## Tried / context

- Digester open empty most of the night (instant-lane noise filtered).
- Design review in grace (`due=false`); still polished stale surfaces in priority order (AI Chat → Agent Ops → Processes → Monitors → Disk Cleanup).
- Screenshot recapture deferred (TCC / need on-screen CPU window).

## Next

- Recapture stale feature screens when the CPU window is visible.
- Rotate to `feature-cpu-metrics` attention polish (rings Hot strip).
- Digester open / product `debug.log` errors when they appear.
