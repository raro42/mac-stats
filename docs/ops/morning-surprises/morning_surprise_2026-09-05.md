# Morning surprise — 2026-09-05

Overnight Track B (mac-stats product ratchet). Digester **open** stayed empty (Elmasnow weather already shipped). Design review stayed in grace (stale PNGs; TCC defer). Debug.log clean in the scan window.

## Shipped tonight

| Version | What |
|---------|------|
| **v0.1.873** | Instant lane: **improvements directory size** (`improvements size`, `how big is the improvements folder`, …) — recursive file bytes under `~/.mac-stats/improvements/`; no list dump; does not steal path / overnight asks |
| **v0.1.872** | Instant lane: **digest.md / latest.md size** — digester markdown size (stat only; no digester spawn) |
| **v0.1.871** | Instant lane: **digest / latest.json size** — digester JSON size (stat only) |
| **v0.1.870** | Instant lane: **runs.jsonl size** |
| **v0.1.869** | Instant lane: **results.tsv size** |
| **v0.1.868** | Instant lane: **results.tsv age** |
| **v0.1.867** | Instant lane: **runs.jsonl age** |

## Why it matters

Operator “how big is …?” asks for harness artifacts stay on the **instant** lane. That cuts p50 direct latency when Discord/AI Chat only needs a size chip, not a model turn.

## Next

- Digester open / product `debug.log` errors when they appear
- Design-review screenshot recapture when TCC allows (`feature-ai-chat` recommended)
- More p50 instant gaps or Hermes insights port
