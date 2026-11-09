# Morning surprise — 2026-11-09

Overnight autoresearch kept product chrome opaque (standing P2 #14). Digester open stayed empty; design-review and layout-daily were not due (grace / data-poster already marked).

## Shipped

| Version | Keep | What |
| --- | --- | --- |
| v0.1.2519 | keep | Perplexity search / inline-key opaque — `.perplexity-search-box` input · button · `#perplexity-search-btn` hover/focus · `.perplexity-inline-key` · `.perplexity-inline-save` off transparent glass onto opaque `#ffffff` + CRT remaps |
| v0.1.2518 | keep | Agent Ops Insights opaque — `.ops-insights-card` · `.ops-insight-line.is-clickable` hover / focus / selected → opaque `color-mix` + CRT remaps |
| v0.1.2517 | keep | Disk Cleanup add-scope opaque — `#disk-cleanup-add-btn` · scope/add inputs · `.disk-cleanup-scope-remove` off transparent glass onto opaque `#ffffff` + CRT remaps |
| v0.1.2516 | keep | Dark Changelog opacity opaque — `.changelog-date` · `.changelog-paragraph` · `.changelog-item` → opaque `color-mix` on `#0a0a0a` |
| v0.1.2515 | keep | Dark / Neon / Futuristic metric opacity opaque — Futuristic `.metric-label` · `.ring-track`, Neon `.metric-unit`, Dark `.ring-track` → opaque `color-mix` on CRT shells |
| v0.1.2514 | keep | Material / Swiss `.metric-label` · `.metric-subtext` · Material `.ring-track` opacity → opaque `color-mix` |
| v0.1.2513 | keep | Light / Material / Swiss `.section-title` invalid glass → opaque `color-mix` |
| v0.1.2512 | keep | Light `.metric-unit` · `.cpu-divider` opacity → opaque `color-mix` |
| v0.1.2511 | keep | Settings note underline opaque across themes |

## Tried / discarded

- Parallel race earlier tonight: local section-title as v0.1.2512 discarded when origin already shipped Light metric-unit/divider as 2512 (re-shipped as 2513).
- Material/Swiss metric-label plan raced origin v0.1.2514 — pulled and moved to CRT metric opacity instead (kept as 2515).

## Notes

- Mac feature-screen recapture still deferred (Screen Recording TCC / menu-bar only).
- Next fuel: remaining Perplexity / Agent Ops transparent leftovers if any, or Mac capture when TCC allows.
