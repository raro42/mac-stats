# Morning surprise — 2026-11-09

Overnight Track B kept baking remaining glass into opaque `color-mix` so CRT / paper shells stay readable without glass alpha in Graphics and Media.

## Shipped tonight

- **v0.1.2525** — Metric-strip flash highlights opaque: `.is-ram-highlight` · `.is-cpu-highlight` · `.is-gpu-highlight` · `.is-temp-highlight` · `.is-freq-highlight` · `.is-disk-highlight` · `.is-uptime-highlight` off transparent glass onto opaque paper `#ffffff` with CRT remaps (Data Poster / Dark / Neon / Futuristic).
- **v0.1.2524** — Disk Cleanup badge halo + Agent Ops jump-flash opaque: `#icon-disk-cleanup .disk-cleanup-badge` · `--ops-jump-flash-ring`.
- **v0.1.2523** — Disk Cleanup toolbar secondary focus opaque: `.disk-cleanup-toolbar .popover-btn-secondary:focus-visible`.
- **v0.1.2522** — Processes + Disk Cleanup path focus opaque: `.process-name:focus-visible` · `.process-detail-name:focus-visible` · `.process-detail-pid:focus-visible` · `.disk-cleanup-item-path:focus-visible` · `.disk-cleanup-scope-path:focus-visible`.
- **v0.1.2521** — Monitors URL / detail focus opaque: `.monitor-url:focus-visible` · `button.monitor-detail-url:focus-visible` · `.monitor-detail-remove:focus-visible`.
- **v0.1.2520** — Monitors settings focus opaque: `.monitor-settings-item:focus-within` · `.monitors-settings-popover .popover-btn-primary:focus-visible` (+ CRT remaps for `.monitor-remove-btn:focus-visible`).
- **v0.1.2519** — Perplexity search / inline-key opaque: `.perplexity-search-box` input · button · `#perplexity-search-btn` hover / focus · `.perplexity-inline-key` · `.perplexity-inline-save`.
- **v0.1.2518** — Agent Ops Insights opaque: `.ops-insights-card` · `.ops-insight-line.is-clickable` hover / focus / selected.
- **v0.1.2517** — Disk Cleanup add-scope opaque: `#disk-cleanup-add-btn` · scope/add inputs · `.disk-cleanup-scope-remove`.
- **v0.1.2516** — Dark `.changelog-date` · `.changelog-paragraph` · `.changelog-item` opacity opaque on `#0a0a0a`.
- **v0.1.2515** — Dark / Neon / Futuristic metric opacity opaque.
- **v0.1.2514** — Material / Swiss metric-label opacity opaque.
- **v0.1.2513** — Light / Material / Swiss `.section-title` off invalid glass `rgb(1,1,1,0.75)`.
- **v0.1.2512** — Light `.metric-unit` / `.cpu-divider` opacity opaque.

## Tried / notes

- Digester empty (open candidates none). Design-review grace; layout-daily already marked today (data-poster).
- Debug.log quiet (no ERROR/WARN/panic clusters in 180m).
- Mac `docs/screens/` recapture still TCC-blocked (`screencapture -l`).
- Intentional thin scrollbar glass left alone (hover thumb).

## Next fuel

History sparkline focus / first-launch banner glass leftovers if any, or Mac capture of stale `feature-cpu-metrics.png` / `feature-agent-ops.png` / `feature-disk-cleanup.png` when Screen Recording TCC allows.
