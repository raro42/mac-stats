# Morning surprise — 2026-11-25

Overnight autoresearch (Track B) shipped real product lanes, not digester-empty quiet.

## Shipped
- **v0.1.3051** — Therm / CPU speed limits instant lane: `/therm` · `/pmset-therm` · `/cpu-speed-limit` · `/speed-limit` · `/thermal-limits` · `/sched-limit` · `/mac-therm` + view/open NL. Reads `pmset -g therm` (scheduler · speed · available CPUs). Bare `therm` / `cpu speed limit` / `thermal limits` stay with the agent; Heat `/thermal` · `/heat`, Power log, System load, and UPS lanes stay intact. Live `thermlog` stream skipped (blocks).
- **v0.1.3050** — Power log (`pmset -g log`) slash+NL (prior tick).
- **v0.1.3049** — UPS (`pmset -g ups`) slash+NL (prior tick).
- **v0.1.3048** — System load (`pmset -g sysload`) slash+NL (prior tick).
- **v0.1.3047** — Power schedule (`pmset -g sched`) slash+NL (prior tick).

## Also this window
- Digester open stayed empty; design-review surfaces in grace (Mac TCC polish-without-capture).
- Layout-daily: data-poster marked 2026-11-24; **Dark** is next calendar quota.
- Ratchet: keep for 3051 @ 29952f6c4; backfilled missing keep for 3050.

## Try
- `/therm` or `view thermal limits` in Discord / AI Chat (Mac).
- `/pmlog` for recent Sleep / Wake / DarkWake / Notification.
- `/ups` for UPS haltlevel / haltafter / haltremain.
