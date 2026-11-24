# Morning surprise — 2026-11-25

Overnight autoresearch opened with Spotlight indexing, then No Restore Sleep soft-state after digester stayed empty.

## Shipped
- **v0.1.3041** — No Restore Sleep `/norestoresleep` · `/no-restore-sleep` · `/mac-norestoresleep` + view/open NL (`pmset` Battery/AC `norestoresleep`, often absent on Apple Silicon). Bare `norestoresleep` / `no restore sleep` stay with the agent; `/systemsleep` · `/ttysclobber` · `/ttyskeepawake` · `/tcpkeepalive` intact; reducebrightness stays out.
- **v0.1.3040** — Spotlight `/spotlight` · `/mdutil` · `/spotlight-indexing` · `/mac-spotlight` + view/open NL (`mdutil -s /` → Enabled/Disabled). Bare `spotlight` / `mdutil` / `indexing` stay with the agent; `/gatekeeper` · `/sip` · `/filevault` · `/timemachine` · `/ssd` · `/free` intact.
- Prior night (already on main before this window): **v0.1.3039** TTY Clobber `/ttysclobber` · `/ttys-clobber`.

## Tried / context
- Digester empty (0 Slowest / 7d).
- Design review: surfaces in polish grace (Mac TCC, polish-without-capture).
- Layout daily: data-poster marked 2026-11-24; **Dark** next calendar quota.
- Sibling OpenClaw/Hermes: no clear port this tick.
- Debug.log: quiet (no ERROR/WARN clusters in scan windows).
- Remote raced Spotlight onto v0.1.3040 while this tick built No Restore Sleep; rebased as v0.1.3041.

## Next fuel
- Dark layout-daily next calendar day
- Soft-state leftover (`reducebrightness` / …) — not another keep-pNN clone
