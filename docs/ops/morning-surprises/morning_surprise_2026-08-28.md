# Morning surprise — 2026-08-28

Overnight Track B (20:00–06:00 local) shipped collapsed-glance keyboard chains, keep-header fixes, filter chips across sections, Perplexity Top/Snippet, Agent Ops Runs Slow/Fail, `/failed` + `/slow` instant operators.

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
| **v0.1.691** | **Disk Cleanup categories All · Reclaim · Big · Clean** — Big filter for categories ≥50 MiB reclaimable; red row wash; Reclaimable now card opens Big when any qualify (Monitors Slow / Top Processes Hot parity; design review / `feature-disk-cleanup`) |
| **v0.1.692** | **Perplexity Search All · Top · Snippet** — Top filter for first 3 hits; Snippet filter for preview text; accent top-row wash; last-search glance opens Top when >3 results (Monitors Slow / AI Chat Errors parity; design review grace) |
| **v0.1.693** | **Agent Ops Runs All · Instant · Direct · Slow** — Slow filter for runs ≥2000 ms wall time; amber row wash; overview Opens → Slow when any qualify (Monitors Slow / p50 latency parity) |
| **v0.1.694** | **Agent Ops Runs Fail filter** — Fail filter for ok=false runs; red row wash; overview Opens → Fail when any failed (AI Chat Errors parity) |
| **v0.1.695** | **`/failed` instant operator** — `/failed` + NL (`what failed`, `failed runs`) return recent ok=false turns from runs.jsonl with error text (Discord + AI Chat; Agent Ops Fail parity; p50 latency) |
| **v0.1.696** | **`/slow` instant operator** — `/slow` + NL (`what's slow`, `slow runs`) return recent turns ≥2000 ms from runs.jsonl (Discord + AI Chat; Agent Ops Slow parity; p50 latency) |

## Why it matters

Keyboard users can move **icon → collapsed glance → footer** without opening the full section. Filter chips now cover **Top Processes Hot**, **Monitors Slow**, **AI Chat Errors**, **CPU rings Hot**, **Disk Cleanup On/Off + Big**, **Perplexity Top/Snippet**, **Agent Ops Runs Slow + Fail**. Ask **`/failed`** or **what failed** for a zero-LLM failed-run report. Ask **`/slow`** or **what's slow** for slow-turn latency.

## Digester / design review

- Digester open empty → standing backlog / design review grace.
- Polish grace still marked for stale feature screens.
- Screenshot recapture deferred if Screen Recording TCC blocks.

## Next fuel

- Digester open / debug.log product errors when present.
- Design review when due again after grace (`feature-ai-chat` recommended).
- p50 direct latency — more tool-heavy patterns remain.
