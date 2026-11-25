# Morning surprise — 2026-11-25

Overnight Track B (mac-stats product ratchet). Digester open empty; design-review grace; layout-daily data-poster marked (Dark next).

## Shipped
- **v0.1.3064** — Battery charge limit instant status from `pmset -g battlimit`
  - Slash + NL: `/battlimit` · `/batt-limit` · `/pmset-battlimit` · `/battery-limit` · `/charge-limit` · `/battery-level-limits` · `/mac-battlimit` (+ view/open/list)
  - Shows lower/upper % or `none set`
  - Does not steal `/battery` · `/cycles` · `/battery-time` · `/rawbatt` · `/accps` · `/systemstate`
- **v0.1.3063** — System state from `pmset -g systemstate` (prior tick)
- **v0.1.3062** — Raw battery from `pmset -g rawbatt` (prior tick)
- Earlier tonight: Sleep/Wake UUID · Accessory power · Power everything / live / adapters / custom / source / … soft-state lane chain

## Fuel choice
Standing backlog after `systemstate` — `profiles` is invalid on this Mac; `history` empty; `battlimit` returns a live none-set/bounds dump, so ship it.

## Next
- Dark layout-daily (next calendar quota)
- Remaining useful `pmset -g` adapters only if non-blocking
- not another keep-pNN clone
