# Morning surprise — 2026-11-25

Overnight (2026-11-24 → 2026-11-25) shipped product fuel from standing backlog (digester open empty).

## Shipped
- **v0.1.3054** — User activity instant lane: `/useractivity` · `/user-activity` · `/pmset-useractivity` · `/user-activity-level` · `/activity-level` · `/mac-useractivity` + view/open NL. Reads Level / MostSignificant / idle hint from `pmset -g useractivity`. Bare phrases stay with the agent; Screensaver idle `/idle-time`, Username `/username`, Power capabilities `/pmcap`, and Power stats `/pmstats` stay intact.
- **v0.1.3053** — Power capabilities `/pmcap` · `/power-cap` from `pmset -g cap` (prior tick; remote).
- **v0.1.3052** — Power stats `/pmstats` · `/power-stats` from `pmset -g stats` (prior tick).
- **v0.1.3051** — Therm / CPU speed limits `/therm` · `/cpu-speed-limit` from `pmset -g therm` (prior tick).

## Context
- Digester open empty; design-review due=false (Mac TCC — polish-without-capture).
- Layout-daily: data-poster marked 2026-11-24 (Dark next calendar quota).
- Soft-state leftover closed for SoftDisabled / assertions / powersched / sysload / ups / pmlog / therm / pmstats / pmcap / useractivity — try other pmset adapters next; not another keep-pNN clone.

## Try
- `/useractivity` or `view user activity` in Discord / AI Chat (Mac).
- `/pmcap` for supported power-management caps.
