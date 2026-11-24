# Morning surprise — 2026-11-25

Overnight Track B (mac-stats) for Ralf.

## Shipped

- **v0.1.3045** — Sleep Disabled `/sleepdisabled` · `/sleep-disabled` · `/disable-sleep` · `/mac-sleepdisabled` + view/open NL (`pmset` Battery/AC `SleepDisabled`, often absent on Apple Silicon). Bare `sleepdisabled` / `sleep disabled` / `disable sleep` stay with the agent; `/systemsleep` · `/sleep` · `/ring` · `/displaysleep` · `/disksleep` · `/norestoresleep` intact.
- **v0.1.3044** — Wake on Modem Ring `/ring` · `/modem-ring` · `/wake-on-ring` · `/mac-ring` + view/open NL.
- **v0.1.3043** — Sudden Motion Sensor `/sms` · `/sudden-motion` · `/sudden-motion-sensor` · `/mac-sms` + view/open NL.
- **v0.1.3042** — Reduce Brightness `/reducebrightness` · `/reduce-brightness` · `/reducebright` · `/mac-reducebrightness` + view/open NL.
- **v0.1.3041** — No Restore Sleep `/norestoresleep` · `/no-restore-sleep` · `/mac-norestoresleep` + view/open NL.
- **v0.1.3040** — Spotlight `/spotlight` · `/mdutil` · `/mac-spotlight` + view/open NL.

## Tried / context

- Digester empty (0 Slowest / 7d). Design review due=false (grace; Mac TCC polish-without-capture). Layout-daily data-poster already marked 2026-11-24 (Dark next calendar day).
- Sibling OpenClaw/Hermes: no clear port. Debug.log: no ERROR/WARN clusters.

## Next

- Dark layout-daily next calendar day
- Soft-state leftover closed for SleepDisabled / ring — not another keep-pNN clone
