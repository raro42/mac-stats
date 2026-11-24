# Morning surprise — 2026-11-25

Overnight autoresearch (Track B) shipped real product lanes, not digester-empty quiet.

## Shipped
- **v0.1.3049** — UPS instant lane: `/ups` · `/ups-settings` · `/pmset-ups` · `/uninterruptible-power` · `/mac-ups` + view/open NL. Reads `pmset -g ups` (haltlevel / haltafter / haltremain). Bare `ups` / `ups settings` stay with the agent; Bat `%`, AC charger, System load, and Power schedule lanes stay intact.
- **v0.1.3048** — System load (`pmset -g sysload`) slash+NL (prior tick).
- **v0.1.3047** — Power schedule (`pmset -g sched`) slash+NL (prior tick).
- **v0.1.3046** — Sleep assertions (`pmset -g assertions`) slash+NL (prior tick).
- **v0.1.3045** — Sleep Disabled slash+NL (prior tick).

## Also this window
- Digester open stayed empty; design-review surfaces in grace (Mac TCC polish-without-capture).
- Layout-daily: data-poster marked 2026-11-24; **Dark** is next calendar quota.
- Ratchet: keep for 3049 @ aef95fa88; backfilled missing keep for 3048.

## Try
- `/ups` or `view ups settings` in Discord / AI Chat (Mac).
- `/sysload` for combined / user / battery / thermal load levels.
- `/powersched` if you want scheduled wake/sleep.
