# Morning surprise — 2026-08-28

Overnight Track B (20:00–06:00 local) shipped collapsed-glance keyboard chains, keep-header fixes, Top Processes Hot, then Monitors Slow filter.

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
| **v0.1.685** | **AI Chat keep-header** — collapsed Ready/turns glance stays visible (Monitors / Disk Cleanup / Debug Log / Perplexity / Agent Ops parity) |
| **v0.1.686** | **Top Processes All · Pinned · Hot** — Hot filter at glance amber thresholds (design review / `feature-processes`) |
| **v0.1.687** | **Monitors All · Up · Down · Slow** — Slow filter for UP sites ≥2000 ms (menu-bar Mon amber; Top Processes Hot parity) |

## Why it matters

Keyboard users can move **icon → collapsed glance → footer** without opening the full section. Monitors, Disk Cleanup, Perplexity, Debug Log, Agent Ops, and AI Chat keep the header + glance on screen when collapsed (compact mode still hides). Top Processes has a **Hot** chip; Monitors has a **Slow** chip so you can list only sites that hit the same amber latency as the menu-bar Mon cue.

## Digester / design review

- Digester open empty; design review in grace — standing backlog → Monitors Slow (Hot parity).
- Polish grace marked for `feature-monitors`.
- Screenshot recapture deferred if Screen Recording TCC blocks.

## Next fuel

- Digester open / debug.log product errors when present.
- Design review when due again after grace (`feature-ai-chat` recommended).
