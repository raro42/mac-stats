# Morning surprise — 2026-11-25

Overnight 2026-11-24/25 shipped more pmset instant-status lanes.

## Shipped
- **v0.1.3056** Power custom profiles — `/pmcustom` · `/power-profiles` (+ aliases) show Battery vs AC diffs from `pmset -g custom` (path-like values skipped).
- **v0.1.3055** Power source — `/pmps` · `/power-source` from `pmset -g ps` (remote tick earlier tonight).
- Earlier tonight: User activity `/useractivity` (3054), Power capabilities `/pmcap` (3053), Power stats `/pmstats` (3052), and the SoftDisabled → assertions → powersched → sysload → ups → pmlog → therm chain.

## Tried / context
- Digester open stayed empty all night.
- Design review due=false (grace; Mac TCC — polish-without-capture).
- Layout daily: data-poster marked 2026-11-24; Dark due next calendar day.
- One remote race on 3055 (Power source) while this tick targeted Power custom — rebased to 3056.

## Next
- Dark layout-daily next calendar day.
- Other pmset adapters if still useful (`history` / `live` / `everything`); avoid keep-pNN soft-state clones.

Generated: 2026-11-25T00:05:30+01:00
