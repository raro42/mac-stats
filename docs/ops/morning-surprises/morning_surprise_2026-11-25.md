# Morning surprise — 2026-11-25

Overnight Track B (mac-stats product ratchet). Digester open empty; design-review grace; layout-daily data-poster marked (Dark next).

## Shipped
- **v0.1.3062** — Raw battery instant status from `pmset -g rawbatt`
  - Slash + NL: `/rawbatt` · `/raw-batt` · `/pmset-rawbatt` · `/raw-battery` · `/battery-raw` · `/mac-rawbatt` (+ view/open/list)
  - Shows Cap · FCC · Design · Cycles · mA (compact; skips Polled / Time / Location)
  - Does not steal `/battery` · `/cycles` · `/battery-time` · `/accps` · `/pmps` · `/pmuuid`
- **v0.1.3061** — Sleep/Wake UUID from `pmset -g uuid` (prior tick)
- **v0.1.3060** — Accessory power from `pmset -g accps` (prior tick)
- Earlier tonight: Power everything / live / adapters / custom / source / … soft-state lane chain

## Fuel choice
Standing backlog after `pmuuid` — Linux skipped `rawbatt` as unused; this Mac returns a live Cap/FCC/Cycles dump, so ship it. `systemstate` / `battlimit` next.

## Next
- Dark layout-daily (next calendar quota)
- `pmset -g systemstate` or `battlimit` if useful
- history only if non-blocking
