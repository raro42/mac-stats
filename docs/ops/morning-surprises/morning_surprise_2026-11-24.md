# Morning surprise — 2026-11-24

Overnight autoresearch kept shipping pmset slash+NL lanes while digester stayed empty.

## Shipped tonight (highlights)
- **v0.1.3022** — Half Dim `/halfdim` · `/half-dim` · `/half-brightness` · `/mac-halfdim` + view/open NL (often absent on Apple Silicon → calm empty reading). Bare `halfdim` / `half dim` / `half brightness` stay with the agent; `/lessbright` · `/displaysleep` · `/powermode` · `/dim-display` intact; hibernatefile stays out.
- **v0.1.3021** — Power Mode `/powermode` · `/power-mode` · `/mac-powermode` + view/open NL (0 Automatic · 1 Low Power · 2 High Power). Bare `powermode` / `power mode` stay with the agent; `/lpm` · `/powernap` · `/power` · `/lessbright` intact.
- **v0.1.3020** — Less Bright `/lessbright` · `/slightly-dim` · `/dim-display`
- **v0.1.3019** — Sleep On Power Button `/sopb`
- **v0.1.3018** — Standby `/standby`
- Earlier: Hibernate · Network over sleep · TCP Keep Alive · System Sleep · Data Poster focus fill · …

## Tried / discarded
- Local races against remote on a few pmset lanes (hibernate / systemsleep / sopb) — discarded duplicates, then shipped the next open key.
- This tick pulled remote Power Mode v0.1.3021 first, then shipped Half Dim (no duplicate race).

## Still soft / next
- cream/Swiss Send fill white-on-blue intentional
- Dark layout-daily next calendar quota
- hibernatefile (or standbydelay / autopoweroff) product fuel — not another keep-pNN clone

Generated: 2026-11-24T01:56:50+01:00
