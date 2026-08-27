# Morning surprise — 2026-08-28

**mac-stats** overnight autoresearch (Track B). Window: 2026-08-27 20:00 → 2026-08-28 06:00.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.678** | Disk Cleanup collapsed glance↔icon↔footer toolbar chain — icon ↓ → glance when collapsed; glance ↑ → icon · ↓ → footer; footer ↑ → glance |
| **v0.1.677** | Monitors collapsed glance↔icon↔footer toolbar chain — icon ↓ → glance when collapsed; glance ↑ → icon · ↓ → footer; footer ↑ → glance |

## Fuel used

- Digester open (no Slowest candidates).
- Design review not due (grace on feature screens).
- Standing backlog: collapsed glance↔icon↔footer chains (Monitors → Disk Cleanup).

## Debug log

- Single-instance lock WARN (KeepAlive thrash — rate-limited in v0.1.381).
- Soul/RUN_CMD noise from agent turns — not product bugs.
- No new product-owned panics.

## Next

- AI Chat / Agent Ops / Perplexity / Debug Log collapsed glance↔icon↔footer chains.
- Design review when `overnight_design_review.py` says due.
- p50 direct latency when digester gets samples.
