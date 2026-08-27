# Morning surprise — 2026-08-27

**mac-stats** overnight autoresearch (Track B). Window: 2026-08-26 20:00 → 2026-08-27 06:00.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.675** | Power strip↔Details collapsed glance toolbar chain — strip last ↓ → Details glance when grid collapsed; glance ↑ → power strip · ↓ → Top Processes; Details header collapse keeps header + glance (grid only hides) |
| **v0.1.674** | History sparkline↔Top Processes filter toolbar chain — last chart ↓ → first filter chip; filter first ↑ → last chart |
| **v0.1.673** | Ring gauge↔Top Processes filter toolbar chain — temperature ring ↓ → first filter chip; filter first ↑ → temperature ring |
| **v0.1.672** | Details↔ring gauge toolbar chain — Details first value ↑ → temperature ring; ring first ↑ or last ↓ → Details first when open |
| **v0.1.671** | Settings Credentials↔header + Top Processes filter↔header toolbar chains |
| **v0.1.670** | AI Chat icon-line↔Ollama settings toolbar chain |
| **v0.1.669** | Perplexity icon-line↔Settings toolbar chain |
| **v0.1.668** | Discord icon-line↔Settings toolbar chain |
| **v0.1.667** | Agent Ops icon-line↔section toolbar chain |
| **v0.1.666** | Disk Cleanup icon-line↔section toolbar chain |
| **v0.1.665** | AI Chat icon-line↔section toolbar chain |
| **v0.1.664** | Monitors icon-line↔section toolbar chain |
| **v0.1.663** | Debug Log icon-line↔toolbar chain |
| **v0.1.662** | Perplexity icon-line↔setup toolbar chain |

## Fuel used

- Digester open (no Slowest candidates).
- Design review not due (grace on feature screens).
- Standing backlog / morning-surprise next: power-strip↔Details collapsed glance.

## Debug log

- Single-instance lock WARN (KeepAlive thrash — rate-limited in v0.1.381).
- No new product-owned errors.

## Next

- p50 direct latency when digester gets samples.
- Design review when `overnight_design_review.py` says due.
- More toolbar wrap polish (footer ↔ collapsed glances).
