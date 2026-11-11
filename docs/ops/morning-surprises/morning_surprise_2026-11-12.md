# Morning surprise — 2026-11-12

Overnight autoresearch (Track B) starting 2026-11-11 20:00 CET.

## Shipped

- **v0.1.2594** (parallel tick) — Digester Slowest + ops-help collapsed onto Instant `/keep-p{N}` (1–99) after harness_ops split.
- **v0.1.2595** — Layout-daily helper: Architect is light paper (spot-check only). Dark-ink due set is `data-poster` / `dark` / `neon` / `futuristic` with CRT remap audit.
- **v0.1.2596** — Footer GitHub hover / focus / busy CRT remaps on Data Poster / Dark / Neon / Futuristic.
- **v0.1.2597** — Serialize `monitor_history.json` RMW behind a process-wide mutex so background checks and “Check now” cannot drop each other’s 24h ticks.
- **v0.1.2598** — CRT remaps for `.logs-path-hint:focus-visible`, `#cpu-usage-card:focus-visible`, and `#perplexity-search-btn:disabled` on Data Poster / Dark / Neon / Futuristic (Apple `#ffffff` → CRT paper).

## Tried / discarded

- Local digester keep-p generator rebuild raced origin v0.1.2594 (same Slowest collapse) → discard.
- Parallel tick already took monitor_history mutex as v0.1.2597 before this tick could start it → re-picked CRT focus/disabled gaps.

## Context

- Digester open empty; design review not due (grace); layout-daily data-poster already marked today (no second layout tick).
- Debug.log: idle-thought Ollama wall-budget WARNs (best-effort; not product ERROR/panic).

## Next fuel

- Dark layout-daily when the day quota opens, Mac capture when TCC allows, or sibling/product fuel (not another keep-pNN clone).
