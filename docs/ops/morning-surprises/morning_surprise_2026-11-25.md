# Morning surprise — 2026-11-25

Overnight 2026-11-24 → 25 (CET). Digester open empty; design-review due=false (grace / Mac TCC); layout-daily data-poster already marked 2026-11-24 (Dark next calendar day).

## Shipped

- **v0.1.3050** — Power log instant lane: `/pmlog` · `/pm-log` · `/powerlog` · `/power-log` · `/powermanagement-log` · `/pmset-log` · `/sleep-wake-log` · `/mac-pmlog` + view/open NL. Reads recent Sleep / Wake / DarkWake / Notification rows from `pmset -g log` (skips Assertions spam). Bare phrases stay with the agent; Debug Log `/logs`, Sleep assertions `/assertions`, System load `/sysload`, Power schedule `/powersched`, UPS `/ups`, and Heat `/thermal` stay intact.
- **v0.1.3049** — UPS `/ups` · `/ups-settings` · `/pmset-ups` (remote raced local Power-log WIP; local duplicate discarded + remote keep backfill, then pmlog as 3050).
- **v0.1.3048** — System load `/sysload` · `/load-level` · `/pmset-sysload`.
- Ratchet: keep @ 44ac03f2a (3050); discard local 3049 Power-log race @ a80a831; keep backfill remote 3049 @ 21153dd36.

## Tried / context

- Digester open empty; design-review grace; layout-daily already checked today — picked Power log (history) after System load / UPS, not another Battery/AC soft-state 0/1 key.
- Sibling OpenClaw/Hermes: no clear port this tick.

## Next

- Dark layout-daily next calendar day.
- Soft-state leftover closed for SoftDisabled / assertions / powersched / sysload / ups / pmlog — not another keep-pNN clone.

## Try

- `/pmlog` or `view power log` in Discord / AI Chat (Mac).
- `/ups` for UPS halt settings when a UPS is attached.
