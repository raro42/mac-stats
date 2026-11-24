# Morning surprise — 2026-11-25

Overnight 2026-11-24 → 25 (CET). Digester open empty; design-review due=false (grace / Mac TCC); layout-daily data-poster already marked 2026-11-24 (Dark next calendar day).

## Shipped

- **v0.1.3048** — System load instant lane: `/sysload` · `/sys-load` · `/pmset-sysload` · `/load-level` · `/system-load-level` · `/mac-sysload` + view/open NL. Reads `pmset -g sysload` (combined / user / battery / thermal levels). Bare phrases stay with the agent; Details `/load` · `/load-1m`, Heat `/thermal` · `/heat`, Bat `/battery`, and Power schedule `/powersched` stay intact.
- **v0.1.3047** — Power schedule `/powersched` · `/power-sched` · `/pmset-sched` · `/scheduled-power` (remote raced local WIP; local duplicate discarded + remote keep backfill, then sysload).
- **v0.1.3046** — Sleep assertions `/assertions` · `/sleep-preventers` · `/who-keeps-awake`.
- **v0.1.3045** — Sleep Disabled `/sleepdisabled` · `/disable-sleep`.
- Ratchet: keep @ 81e0b684a (3048); discard local powersched dup @ 34220e9; keep backfill remote 3047 @ 48768a46a.

## Tried / context

- First fuel this tick (Power schedule) collided with remote v0.1.3047 → discard + reset + keep backfill, then shipped System load (not SoftDisabled / assertions / powersched clone).
- Sibling OpenClaw/Hermes: no clear port; insights/cost already present.

## Next

- Dark layout-daily next calendar day.
- Soft-state leftover closed for SoftDisabled / assertions / powersched / sysload — not another keep-pNN clone.

## Try

- `/sysload` or `view load level` in Discord / AI Chat (Mac).
- `/powersched` for scheduled wake/sleep timers.
