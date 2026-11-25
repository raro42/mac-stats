# Morning surprise 2026-11-25

Overnight autoresearch kept shipping power/pmset instant lanes.

## Shipped
- **v0.1.3061** Sleep/Wake UUID — `/pmuuid` · `/sleep-wake-uuid` · `/pmset-uuid` (+ view/open NL) from `pmset -g uuid`
- **v0.1.3060** Accessory power — `/accps` · `/accessory-power` · `/accessory-batteries` from `pmset -g accps`
- Earlier tonight: Power everything / live / adapters / custom / source / useractivity / cap / stats / therm / …

## Tried / skipped
- Digester empty (0 runs / 7d)
- Design review due=false (grace; Mac TCC — polish-without-capture)
- Layout daily: data-poster marked today; **Dark** next calendar quota
- `battlimit` / `rawbatt` not useful pmset getters (rawlog/pslog block; batt ≡ ps already `/pmps`)

## Next
- Dark layout-daily next calendar day
- Optional: `pmset -g systemstate` / `profiles` if useful; avoid blocking `history`/`thermlog`/`rawlog`
