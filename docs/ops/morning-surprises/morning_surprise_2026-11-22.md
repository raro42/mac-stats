# Morning surprise — 2026-11-22

Overnight autoresearch (Track B) kept shipping operator instant lanes. Digester open stayed empty; design-review stayed in grace (Mac TCC); layout-daily already marked data-poster for the day.

## Shipped tonight (keep)

| Version | What |
|---------|------|
| **v0.1.2958** | Battery time-to-empty / full `/battery-time` · `/time-remaining` · `/time-to-empty` · `/time-to-full` + view/open NL |
| v0.1.2957 | AC charger · adapter `/charger` · `/adapter` · `/charger-watts` + view/open NL |
| v0.1.2956 | Battery cycles · health · max capacity `/cycles` · `/battery-health` · `/max-capacity` + view/open NL |
| v0.1.2955 | Primary link status · media / speed `/link-status` · `/media` · `/link-speed` + view/open NL |
| earlier | broadcast · iface · netmask · mtu · mac · wifi · dns · gateway · ip · boot · cores · … |

## This tick (~05:50 CET)

- Fuel: standing product after AC charger race — `battery` crate `time_to_empty` / `time_to_full` had no dedicated slash+NL while `/battery`, `/cycles`, and `/charger` already did. Rebased onto remote v0.1.2957 charger as **v0.1.2958**.
- Verify OK → keep → push `main` (also backfilled missing 2956 keep row earlier).
- Bare `time` / `remaining` stay with the agent; `/battery` · `battery left` stay Bat %; `/cycles` stays cycles; `/charger` stays adapter watts.

## Still open

- Dark / Neon / Futuristic layout-daily next calendar day
- cream/Swiss Send fill white-on-blue intentional (leave alone)
- sibling harness ports when digester stays empty
- Ask Werner `/battery-time` or `/charger` for ETA · adapter watts without waking Ollama
