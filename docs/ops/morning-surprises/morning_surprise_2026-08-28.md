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

## Why it matters

Keyboard users can move **icon → collapsed glance → footer** without opening the full section. Disk Cleanup, Perplexity, and Debug Log keep the header + glance on screen when collapsed (compact mode still hides).

## Digester / design review

- Digester **open** empty (instant noise filtered).
- Design review in grace after Disk Cleanup polish; next tick used standing backlog (Debug Log keep-header).

## Next fuel

- Agent Ops / Monitors / AI Chat keep-header (same full-hide pattern).
- Digester open / debug.log product errors when present.
