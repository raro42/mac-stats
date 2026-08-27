# Morning surprise — 2026-08-27

**mac-stats** overnight autoresearch (Track B). Window: 2026-08-26 20:00 → 2026-08-27 06:00.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.673** | Ring gauge↔Top Processes filter toolbar chain — temperature ring ↓ → first filter chip; filter first ↑ → temperature ring (header Refresh ← still pairs) |
| **v0.1.672** | Details↔ring gauge toolbar chain — Details first value ↑ → temperature ring; ring first ↑ or last ↓ → Details first when Details is open |
| **v0.1.671** | Settings Credentials↔header toolbar chain — CPU header Settings ↓ → Discord token; token ↑ → Settings header; Perplexity key ↑ → Perplexity icon |
| **v0.1.671** | Top Processes filter↔header toolbar chain — filter chips first ↑ → Refresh; Refresh first ← → filter chips when section open |
| **v0.1.670** | AI Chat icon-line↔Ollama settings toolbar chain — icon ↓ → system prompt when popover open; prompt first ↑ → AI Chat icon |
| **v0.1.669** | Perplexity icon-line↔Settings toolbar chain — icon ↓ → API key when Settings open; key first ↑ → Perplexity icon |
| **v0.1.668** | Discord icon-line↔Settings toolbar chain — icon ↓ → token when Settings open; token first ↑ → Discord icon |
| **v0.1.667** | Agent Ops icon-line↔section toolbar chain — icon ↓ → health strip, refresh row, or tab bar; health first card ↑ → icon |
| **v0.1.666** | Disk Cleanup icon-line↔section toolbar chain — icon ↓ → filter chips, meta, scopes, categories; first chip ↑ → icon |
| **v0.1.665** | AI Chat icon-line↔section toolbar chain — icon ↓ → filter chips, starter chips, composer, messages; first chip ↑ → icon |
| **v0.1.664** | Monitors icon-line↔section toolbar chain — icon ↓ → settings / filter chips / list; first chip ↑ → icon |
| **v0.1.663** | Debug Log icon-line↔toolbar chain — logs icon ↓ → Refresh; Refresh ↑ → icon when empty |
| **v0.1.662** | Perplexity icon-line↔setup toolbar chain — icon ↓ → inline API key; key ↑ → icon |

## Fuel used

- Digester open (no Slowest candidates).
- Design review not due (grace on feature screens).
- Standing backlog: keyboard toolbar chain polish (ring gauge↔Top Processes filters).

## Debug log

- Single-instance lock WARN (KeepAlive thrash — already rate-limited in v0.1.381).
- No new product-owned errors.

## Next

- Power-strip↔Details glance chain or history sparkline↔processes shortcut.
- p50 direct latency when digester gets samples.
- Design review when `overnight_design_review.py` says due.
