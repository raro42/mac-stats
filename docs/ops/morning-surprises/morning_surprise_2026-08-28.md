# Morning surprise — 2026-08-28

Overnight Track B (20:00–06:00 local) shipped collapsed-glance keyboard chains, keep-header fixes, Top Processes Hot, Monitors Slow, AI Chat Errors, CPU rings Hot, then Disk Cleanup scopes On/Off.

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
| **v0.1.688** | **AI Chat All · You · Assistant · Errors** — Errors filter for failed turns (`Error: …`); red bubble wash (Monitors Slow / Top Processes Hot parity; design review / `feature-ai-chat`) |
| **v0.1.689** | **CPU rings All · Hot** — Hot filter at menu-bar amber thresholds (CPU ≥50% / GPU ≥15% / Freq ≥3.5 GHz / Temp ≥70°C); amber card wash + matching history charts (design review / `feature-cpu-metrics`) |
| **v0.1.690** | **Disk Cleanup scopes All · On · Off** — On/Off filter for enabled or disabled scopes; amber Off wash; Enabled scopes card opens Off when any are off (Agents All/On/Off parity; design review / `feature-disk-cleanup`) |

## Why it matters

Keyboard users can move **icon → collapsed glance → footer** without opening the full section. Monitors, Disk Cleanup, Perplexity, Debug Log, Agent Ops, and AI Chat keep the header + glance on screen when collapsed (compact mode still hides). Top Processes has a **Hot** chip; Monitors has a **Slow** chip; AI Chat has an **Errors** chip; CPU rings have a **Hot** chip; Disk Cleanup scopes have **On · Off** so you can find disabled paths fast.

## Digester / design review

- Digester open empty → standing backlog / design review grace → Disk Cleanup scopes On/Off (`feature-disk-cleanup` stale).
- Polish grace marked for `feature-disk-cleanup`.
- Screenshot recapture deferred if Screen Recording TCC blocks.

## Next fuel

- Digester open / debug.log product errors when present.
- Design review when due again after grace.
