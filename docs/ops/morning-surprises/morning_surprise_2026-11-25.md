# Morning surprise — 2026-11-25

Overnight 2026-11-24/25 shipped more pmset instant-status lanes, then Power everything.

## Shipped
- **v0.1.3059** Power everything — `/pmeverything` · `/power-everything` · `/all-power-settings` (+ aliases) summarize section labels (Battery · AC · UPS · Sched · Assertions) plus a Currently-in-use snapshot from `pmset -g everything`. Distinct from Power live `/pmlive` and from health `is everything ok`.
- **v0.1.3058** Power live — `/pmlive` · `/power-live` · `/currently-in-use` (+ aliases) from `pmset -g live`.
- Earlier tonight: Power adapters `/pmadapters` (3057), Power custom `/pmcustom` (3056), Power source `/pmps` (3055), User activity `/useractivity` (3054), and the SoftDisabled → assertions → powersched → sysload → ups → pmlog → therm → pmstats → pmcap chain.

## Tried / context
- Digester open stayed empty.
- Design review due=false (grace; Mac TCC — polish-without-capture).
- Layout daily: data-poster audited 2026-11-25 (missing_remaps=none; Dark due next).
- Sibling OpenClaw/Hermes missing on rack; no port.

## Next
- Dark layout-daily next calendar quota.
- Optional `pmset -g history` if non-blocking; avoid keep-pNN soft-state clones.

Generated: 2026-11-25T00:53:53+01:00
