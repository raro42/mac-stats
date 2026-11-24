# Morning surprise — 2026-11-25

Overnight 2026-11-24/25 shipped more pmset instant-status lanes, then Power adapters.

## Shipped
- **v0.1.3057** Power adapters — `/pmadapters` · `/power-adapters` · `/adapters` (+ aliases) show wattage · description from `pmset -g adapters` (fallback `pmset -g adapter`). Distinct from AC charger `/adapter` (`system_profiler`).
- **v0.1.3056** Power custom profiles — `/pmcustom` · `/power-profiles` Battery vs AC diffs from `pmset -g custom`.
- Earlier tonight: Power source `/pmps` (3055), User activity `/useractivity` (3054), Power capabilities `/pmcap` (3053), Power stats `/pmstats` (3052), and the SoftDisabled → assertions → powersched → sysload → ups → pmlog → therm chain.

## Tried / context
- Digester open stayed empty.
- Design review due=false (grace; Mac TCC — polish-without-capture).
- Layout daily: data-poster audited 2026-11-25 (missing_remaps=none; Dark due next).
- Sibling OpenClaw/Hermes missing on rack; no port.

## Next
- Dark layout-daily next calendar quota.
- Other pmset getters if useful (`history` / `live` / `everything` only if non-blocking); avoid keep-pNN soft-state clones.

Generated: 2026-11-25T00:26:00+01:00
