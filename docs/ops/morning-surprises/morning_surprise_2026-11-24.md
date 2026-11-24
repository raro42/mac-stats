# Morning surprise — 2026-11-24

Overnight autoresearch kept shipping pmset slash+NL lanes while digester stayed empty.

## Shipped tonight (highlights)
- **v0.1.3024** — Standby delay `/standbydelay` · `/standby-delay` · `/mac-standbydelay` + view/open NL (seconds; often absent on Apple Silicon → calm empty reading). Bare `standbydelay` / `standby delay` stay with the agent; `/standby` · `/hibernatefile` intact; standbydelayhigh / standbydelaylow / autopoweroff stay out.
- **v0.1.3023** — Hibernate file `/hibernatefile` · `/hibernate-file` · `/sleepimage` · `/mac-hibernatefile` + view/open NL (path to sleepimage from `pmset`). Bare `hibernatefile` / `hibernate file` / `sleepimage` stay with the agent; `/hibernatemode` · `/hibernate` · `/halfdim` · `/standby` intact.
- **v0.1.3022** — Half Dim `/halfdim` · `/half-dim` · `/half-brightness`
- **v0.1.3021** — Power Mode `/powermode`
- **v0.1.3020** — Less Bright `/lessbright`
- Earlier: Sleep On Power Button · Standby · Hibernate · Network over sleep · TCP Keep Alive · System Sleep · Data Poster focus fill · …

## Tried / discarded
- Local Hibernate file race against remote v0.1.3023 — discarded duplicate, backfilled remote keep, then shipped Standby delay.
- Earlier local races on hibernate / systemsleep / sopb — discarded duplicates, then next open key.

## Still soft / next
- cream/Swiss Send fill white-on-blue intentional
- Dark layout-daily next calendar quota
- autopoweroff (or standbydelayhigh/low) product fuel — not another keep-pNN clone

Generated: 2026-11-24T02:26:27+01:00
