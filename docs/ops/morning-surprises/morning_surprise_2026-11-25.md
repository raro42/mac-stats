# Morning surprise — 2026-11-25

Overnight 2026-11-24 → 25 (CET). Digester open empty; design-review due=false (grace / Mac TCC); layout-daily data-poster already marked 2026-11-24 (Dark next calendar day).

## Shipped

- **v0.1.3046** — Sleep assertions instant lane: `/assertions` · `/sleep-preventers` · `/preventers` · `/who-keeps-awake` · `/mac-assertions` + view/open NL. Reads `pmset -g assertions` (Prevent* flags + owning processes). Bare phrases stay with the agent; System Sleep / Sleep Disabled / Display·Disk·No-Restore Sleep lanes stay intact.
- **v0.1.3045** (earlier tick / remote) — Sleep Disabled `/sleepdisabled` · `/sleep-disabled` · `/disable-sleep`.
- **v0.1.3044** — Wake on Modem Ring `/ring` · `/modem-ring` · `/wake-on-ring`.
- Ratchet: keep @ eab938656 (3046); SoftDisabled keep backfill @ 594ea40.

## Tried / context

- SoftDisabled was already on origin when this tick started → pulled + backfilled keep, then picked assertions (not SoftDisabled clone).
- Sibling OpenClaw/Hermes: no clear port; insights/cost already present.

## Next

- Dark layout-daily next calendar day.
- Soft-state leftover closed for SoftDisabled / assertions — not another keep-pNN clone.
