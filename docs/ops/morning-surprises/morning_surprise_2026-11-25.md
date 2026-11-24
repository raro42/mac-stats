# Morning surprise — 2026-11-25

Overnight 2026-11-24/25 shipped more pmset instant-status lanes.

## Shipped
- **v0.1.3058** Power live — `/pmlive` · `/power-live` · `/currently-in-use` (+ aliases) show Currently-in-use settings from `pmset -g live` (sleep preventer notes kept).
- **v0.1.3057** Power adapters — `/pmadapters` · `/power-adapters` (remote tick; this Mac raced live-on-3057 and discarded).
- **v0.1.3056** Power custom profiles — `/pmcustom` · `/power-profiles` Battery vs AC diffs.
- Earlier tonight: Power source `/pmps` (3055), User activity `/useractivity` (3054), Power capabilities `/pmcap` (3053), and the SoftDisabled → assertions → powersched → sysload → ups → pmlog → therm → pmstats chain.

## Layout
- Data Poster layout-daily: remaps audited OK (no missing WASH selectors / no live `#0e0e14`); marked checked 2026-11-25. Dark is next calendar quota.

## Tried / context
- Digester open stayed empty.
- Design review due=false (grace; Mac TCC — polish-without-capture).
- Local Power live targeted 3057 while remote shipped adapters 3057 — discarded + rebased to 3058.

## Next
- Dark layout-daily next calendar day.
- Optional `pmset -g everything` summary if still useful; avoid keep-pNN soft-state clones.

Generated: 2026-11-25T00:36:26+01:00
