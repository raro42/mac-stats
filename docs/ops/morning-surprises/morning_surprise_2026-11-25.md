# Morning surprise — 2026-11-25

Overnight autoresearch (Track B) kept shipping pmset instant lanes.

## Shipped
- **v0.1.3055** Power source — `/pmps` · `/power-source` · `/now-drawing` · `/ps` + view/open NL from `pmset -g ps` (now drawing + battery row). Does not steal `/battery`, `/adapter`, `/ups`, or `/useractivity`.
- **v0.1.3054** User activity — `/useractivity` · `/activity-level` from `pmset -g useractivity` (backfill keep logged this tick).
- Earlier same night (already archived): Power capabilities, Power stats, Therm, Power log, UPS, System load, Power schedule, Sleep assertions, …

## Tried / context
- Digester empty all night; design-review surfaces in grace (Mac TCC — polish-without-capture).
- Layout-daily: Data Poster marked 2026-11-24; **Dark** is next calendar-day quota.
- Sibling OpenClaw/Hermes: no clear port on this rack.
- Debug.log: quiet (no ERROR/WARN clusters).

## Next
- Dark layout-daily (next calendar day).
- `pmset -g adapters` after Power source — not another keep-pNN clone.
