# Morning surprise — 2026-11-25

Overnight Track B (mac-stats product ratchet). Digester open empty; design-review grace (Agent Ops polish already shipped); layout-daily data-poster marked (Dark next).

## Shipped
- **v0.1.3066** — Power user-clients instant status from `pmset -g userclients`
  - Slash + NL: `/userclients` · `/user-clients` · `/pmset-userclients` · `/power-clients` · `/pm-clients` · `/iokit-clients` · `/mac-userclients` (+ view/open/list)
  - Shows hold count · unique procs · top names (×N)
  - Does not steal `/useractivity` · `/assertions` · `/tasks` · `/powerstate`
- **v0.1.3065** — Agent Ops leftover kb-hints (shorter, warmer Tab tips; polish-without-capture)
- **v0.1.3064** — Battery charge limit from `pmset -g battlimit`
- **v0.1.3063** — System state from `pmset -g systemstate`
- **v0.1.3062** — Raw battery from `pmset -g rawbatt`
- Earlier tonight: Sleep/Wake UUID · Accessory power · Power everything / live / adapters / custom / source / … soft-state lane chain

## Fuel choice
Digester empty; design-review already used tonight on Agent Ops; layout-daily Dark waits for next calendar day. Standing backlog asked for useful non-blocking `pmset -g` adapters (not soft-state keep-pNN) — `userclients` returns a live hold list, so ship it.

## Next
- Dark layout-daily (next calendar quota)
- Soft-state leftover closed for SoftDisabled…battlimit/userclients
- Remaining useful non-blocking `pmset -g` adapters only if clearly distinct
- not another keep-pNN clone
