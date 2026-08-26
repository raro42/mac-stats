# Morning surprise — 2026-08-27

**mac-stats** overnight autoresearch (Track B). Window: 2026-08-26 20:00 → 2026-08-27 06:00.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.665** | AI Chat icon-line↔section toolbar chain — AI Chat icon ↓ → filter chips, starter chips, composer, or messages; first filter chip ↑ → icon |
| **v0.1.664** | Monitors icon-line↔section toolbar chain — icon ↓ → settings / filter chips / list; first chip ↑ → icon |
| **v0.1.663** | Debug Log icon-line↔toolbar chain — logs icon ↓ → Refresh; Refresh ↑ → icon when viewer empty |
| **v0.1.662** | Perplexity icon-line↔setup toolbar chain — icon ↓ → inline API key; key ↑ → icon |

## Fuel used

- Digester open (no Slowest candidates).
- Design review not due (grace on feature screens).
- Standing backlog: keyboard toolbar chain polish across icon-line sections.

## Debug log

- Single-instance lock WARN (KeepAlive thrash — already rate-limited in v0.1.381).
- No new product-owned errors.

## Next

- Continue icon-line↔section chains (Discord, Disk Cleanup, Agent Ops).
- p50 direct latency when digester gets samples.
- Design review when `overnight_design_review.py` says due.
