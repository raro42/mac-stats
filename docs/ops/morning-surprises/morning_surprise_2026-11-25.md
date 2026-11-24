# Morning surprise — 2026-11-25

Overnight 2026-11-24 → 25 (CET). Digester open empty; design-review due=false (grace / Mac TCC); layout-daily data-poster already marked 2026-11-24 (Dark next calendar day).

## Shipped

- **v0.1.3052** — Power stats instant lane: `/pmstats` · `/pm-stats` · `/power-stats` · `/sleep-stats` · `/wake-stats` · `/sleep-wake-stats` · `/pmset-stats` · `/mac-pmstats` + view/open NL. Reads Sleep / Dark Wake / User Wake counts from `pmset -g stats`. Bare phrases stay with the agent; Power log `/pmlog`, Therm `/therm`, Dark Wakes `/darkwakes`, Sleep assertions `/assertions`, System load `/sysload`, Power schedule `/powersched`, UPS `/ups`, and Heat `/thermal` stay intact.
- **v0.1.3051** — Therm / CPU speed limits `/therm` · `/cpu-speed-limit` from `pmset -g therm` (remote; local Power-stats WIP rebased to 3052).
- **v0.1.3050** — Power log `/pmlog` · `/powerlog` from `pmset -g log`.
- **v0.1.3049** — UPS `/ups` · `/ups-settings`.
- Ratchet: keep @ b30f9cebb (3052).

## Tried / context

- Digester open empty; design-review grace; layout-daily already checked today — picked Power stats (counts) after Therm, not another Battery/AC soft-state 0/1 key.
- Sibling OpenClaw/Hermes: no clear port this tick.

## Next

- Dark layout-daily next calendar day.
- Soft-state leftover closed for SoftDisabled / assertions / powersched / sysload / ups / pmlog / therm / pmstats — try `pmset -g cap` / useractivity; not another keep-pNN clone.

## Try

- `/pmstats` or `view power stats` in Discord / AI Chat (Mac).
- `/therm` for CPU scheduler / speed limits.
- `/pmlog` for recent Sleep / Wake event lines.
