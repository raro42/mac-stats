# Morning surprise — 2026-09-20

Overnight Track B (20:00–06:00 local, window opened 2026-09-19 20:00). Digester open: empty. Debug log quiet.

## Shipped tonight

| Version | What |
| --- | --- |
| **v0.1.1175** | Disk free instant — “how much free space?”, “how much space is left”, and “how many GB free” answer with free disk bytes (no LLM); “how much disk is used” and bare “free space” still use the percent chip; Disk Cleanup stays on `/disk`; “why” stays with the agent |

## Why it matters

Ask “how much free space?” and you get free disk bytes. The chat model does not have to look that up.

## Next

- Digester open / product-owned `debug.log` errors when they appear
- Design review when screens age past grace (Agent Ops is in grace; recapture when Screen Recording allows)
- Instant-lane leftovers: config changes (`set url`, `change model`) stay with the agent on purpose
