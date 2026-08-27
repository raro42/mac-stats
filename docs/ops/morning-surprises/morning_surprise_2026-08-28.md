# Morning surprise — 2026-08-28

Overnight Track B (20:00–06:00 local) shipped collapsed-glance keyboard chains, then keep-header fixes so glances stay on screen when sections collapse.

## Shipped tonight

| Version | What |
| --- | --- |
| **v0.1.677** | Monitors collapsed glance↔icon↔footer toolbar chain |
| **v0.1.678** | Disk Cleanup collapsed glance↔icon↔footer toolbar chain |
| **v0.1.679** | AI Chat collapsed glance↔icon↔footer toolbar chain |
| **v0.1.680** | Perplexity collapsed glance↔icon↔footer toolbar chain (**keep-header restored**) |
| **v0.1.681** | **Disk Cleanup keep-header** — collapsed reclaim/due/scopes glance stays visible (design review) |
| **v0.1.682** | **Debug Log keep-header** + Quiet/error/warn glance↔icon↔footer toolbar chain |
| **v0.1.683** | **Agent Ops keep-header** + Discord Ready glance↔icon↔footer toolbar chain (design review) |
| **v0.1.684** | **Monitors keep-header** — collapsed up/down glance stays visible (design review / Disk Cleanup parity) |

## Why it matters

Keyboard users can move **icon → collapsed glance → footer** without opening the full section. Monitors, Disk Cleanup, Perplexity, Debug Log, and Agent Ops keep the header + glance on screen when collapsed (compact mode still hides). Discord Ready / Offline stays visible without opening Agent Ops. External / Monitors up/down status stays visible without opening the list.

## Digester / design review

- Digester **open** empty; design review **due** on stale `feature-monitors.png`.
- Shipped Monitors keep-header; screenshot recapture deferred if Screen Recording TCC blocks; polish grace marked.

## Next fuel

- AI Chat keep-header (same full-hide via `setIconPaneVisibility` if still present).
- Digester open / debug.log product errors when present.
