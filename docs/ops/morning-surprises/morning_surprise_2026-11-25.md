# Morning surprise 2026-11-25

Overnight autoresearch kept shipping power/pmset instant lanes.

## Shipped
- **v0.1.3063** System state — `/systemstate` · `/sysstate` · `/system-power-state` · `/system-capabilities` (+ view/open NL) from `pmset -g systemstate` (state + CPU/Graphics/Audio/Network)
- **v0.1.3062** Raw battery — `/rawbatt` · `/raw-battery` from `pmset -g rawbatt` (Cap/FCC/Design/Cycles/mA)
- **v0.1.3061** Sleep/Wake UUID — `/pmuuid` · `/sleep-wake-uuid` from `pmset -g uuid`
- **v0.1.3060** Accessory power — `/accps` · `/accessory-power` from `pmset -g accps`
- Earlier tonight: Power everything / live / adapters / custom / source / useractivity / cap / stats / therm / …

## Tried / skipped
- Digester empty (0 runs / 7d)
- Design review due=false (grace; Mac TCC — polish-without-capture)
- Layout daily: data-poster marked today; **Dark** next calendar quota
- `battlimit` not a useful getter yet; blocking `history`/`thermlog`/`rawlog` skipped

## Next
- Dark layout-daily next calendar day
- Optional: `pmset -g profiles` if useful; avoid blocking streams
